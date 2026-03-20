#pragma once
#include <string>
#include <vector>
#include "tokenizer.hpp"
#include "sentencepiece_processor.h"

using namespace std;

class SentencePieceTokenizer : public Tokenizer {
    public:
        sentencepiece::SentencePieceProcessor *processor;

        SentencePieceTokenizer(sentencepiece::SentencePieceProcessor *processor);

        vector<string> encode(string input);
        string decode(vector<string> const tokens);
        void freeTokenizer();
};