#pragma once
#include <vector>
#include <string>

using namespace std;

class Tokenizer {
    public:
    virtual ~Tokenizer() = default;

    virtual vector<string> encode(string input) = 0;
        virtual string decode(vector<string> tokens) = 0;
        virtual void freeTokenizer() = 0;
};