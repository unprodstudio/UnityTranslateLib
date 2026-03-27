#include "library.h"
#include "sentencepiece_tokenizer.hpp"
#include "bpe_tokenizer.hpp"
#include "tokenizer.hpp"
#include <fstream>
#include "ctranslate2/translator.h"

#if defined(_WIN32)
#define __export __declspec(dllexport)
#elif defined(__GNUC__) && ((__GNUC__ >= 4) || (__GNUC__ == 3 && __GNUC_MINOR__ >= 3))
#define __export __attribute__((visibility("default")))
#else
#define __export
#endif

extern "C" {
    __export UnityTranslateLibInstance* createInstance(const char* toLang, const char* translatorModelPath, TokenizerType type, char* tokenizerModelPath, bool useCuda) {
        Tokenizer* tokenizer;

        // Load based on tokenizer type
        if (type == TokenizerType::SENTENCEPIECE) {
            sentencepiece::SentencePieceProcessor processor;
            processor.LoadOrDie(tokenizerModelPath); // We want to crash instantly if SentencePiece fails.

            SentencePieceTokenizer spTokenizer(&processor);

            tokenizer = &spTokenizer;
        } else if (type == TokenizerType::BPE) {
            ifstream bpeModelFile(tokenizerModelPath);
            string codes;
            
            if (bpeModelFile.is_open()) {
                // relying on https://stackoverflow.com/a/2602060
                bpeModelFile.seekg(0, ios::end);
                codes.reserve(bpeModelFile.tellg());
                bpeModelFile.seekg(0, ios::beg);

                codes.assign((istreambuf_iterator<char>(bpeModelFile)), istreambuf_iterator<char>());
            } else {
                throw runtime_error("Could not open BPE model file for reading!");
            }

            bpeModelFile.close();

            BPETokenizer bpeTokenizer(toLang, codes);
            tokenizer = &bpeTokenizer;
        } else {
            // I'd imagine we'd somehow get here.
            throw runtime_error("Invalid tokenizer type!");
        }

        // Initialize translator
        ctranslate2::models::ModelFileReader modelReader(translatorModelPath);
        shared_ptr<ctranslate2::models::ModelFileReader> modelReaderPtr(&modelReader);
        ctranslate2::models::ModelLoader modelLoader(modelReaderPtr);
        
        if (useCuda) {
            modelLoader.device = ctranslate2::Device::CUDA;
        } else {
            modelLoader.device = ctranslate2::Device::CPU;
        }

        ctranslate2::ReplicaPoolConfig config;
        ctranslate2::Translator translator(modelLoader, config);

        UnityTranslateLibInstance* instancePtr;
        instancePtr = static_cast<UnityTranslateLibInstance*>(malloc(sizeof(UnityTranslateLibInstance)));

        // Now we initialize everything.
        UnityTranslateLibInstance instance{};
        instance.tokenizer = tokenizer;
        instance.translator = &translator;
        *instancePtr = instance;

        return instancePtr;
    }

    __export void batchTranslate(const UnityTranslateLibInstance* instance, const char** textToTranslate, const int arrayLength, const char** results) {
        vector<vector<string>> tokenizedTexts;

        for (int i = 0; i < arrayLength; i++) {
            const char* text = textToTranslate[i];
            vector<string> tokenizedText = instance->tokenizer->encode(text);
            tokenizedTexts.push_back(tokenizedText);
        }

        // Recreate Argos Translate's behaviour
        constexpr int numHypotheses = 4;

        ctranslate2::TranslationOptions options;
        options.replace_unknowns = true;
        options.beam_size = max(numHypotheses, 4);
        options.num_hypotheses = numHypotheses;
        options.length_penalty = 0.2;
        options.return_scores = true;

        const vector<ctranslate2::TranslationResult> translationResult = instance->translator->translate_batch(tokenizedTexts, options);

        for (int i = 0; i < arrayLength; i++) {
            ctranslate2::TranslationResult result = translationResult.at(i);
            const vector<string>& tokens = result.output();
            string parsed = instance->tokenizer->decode(tokens);

            results[i] = parsed.c_str();
        }
    }

    __export void freeInstance(UnityTranslateLibInstance* instance) {
        // Free translator
        instance->translator->clear_cache();
        free(instance->translator);

        // Free tokenizer
        instance->tokenizer->freeTokenizer();
        free(instance->tokenizer);

        free(instance);
    }
}