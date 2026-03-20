#include <vector>
#include <string>
#include "tokenizer.hpp"
#include "sentencepiece_processor.h"
#include "sentencepiece_tokenizer.hpp"

using namespace std;

SentencePieceTokenizer::SentencePieceTokenizer(sentencepiece::SentencePieceProcessor *processor) {
    this->processor = processor;
}

vector<string> SentencePieceTokenizer::encode(string input) {
    vector<string> pieces;
    sentencepiece::util::Status status = this->processor->Encode(input, &pieces);

    if (!status.ok()) {
        const string error = status.error_message();
        printf("Failed to encode: %s", error.c_str());
    }

    return pieces;
}

string SentencePieceTokenizer::decode(vector<string> const tokens) {
    string result;
    sentencepiece::util::Status status = this->processor->Decode(tokens, &result);

    if (!status.ok()) {
        string error = status.error_message();
        printf("Failed to decode: %s", error.c_str());
    }
    
    return result;
}

void SentencePieceTokenizer::freeTokenizer() {
    free(this->processor);
}