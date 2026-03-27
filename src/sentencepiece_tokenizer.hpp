#pragma once
#include <string>
#include <vector>
#include "tokenizer.hpp"
#include "sentencepiece_processor.h"

using namespace std;

class SentencePieceTokenizer : public Tokenizer {
    public:
        sentencepiece::SentencePieceProcessor *processor;

        explicit SentencePieceTokenizer(sentencepiece::SentencePieceProcessor *processor);

        vector<string> encode(string input) override;
        string decode(vector<string> tokens) override;
        void freeTokenizer() override;
};