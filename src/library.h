#pragma once
#include "tokenizer.hpp"
#include <ctranslate2/translator.h>

#ifdef __cplusplus
extern "C" {
#endif

#if defined(_WIN32)
#define __export __declspec(dllexport)
#elif defined(__GNUC__) && ((__GNUC__ >= 4) || (__GNUC__ == 3 && __GNUC_MINOR__ >= 3))
#define __export __attribute__((visibility("default")))
#else
#define __export
#endif

__export struct UnityTranslateLibInstance {
    Tokenizer* tokenizer;
    ctranslate2::Translator* translator;
};

__export enum TokenizerType {
    SENTENCEPIECE, BPE
};

/**
 * Used for creating a new UnityTranslateLib instance.
 */
__export UnityTranslateLibInstance* createInstance(char* toLang, char* translatorModelPath, TokenizerType type, char* tokenizerModelPath, bool useCuda);

/**
 * 
 */
__export const char** batchTranslate(UnityTranslateLibInstance* instance, const char** textToTranslate, int arrayLength);

/**
 * Used for freeing the UnityTranslateLib instance pointers. Must be called for closing the translator.
 */
__export void freeInstance(UnityTranslateLibInstance* instance);

#ifdef __cplusplus
}
#endif
