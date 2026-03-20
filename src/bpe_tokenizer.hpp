#pragma once
#include <string>
#include <vector>
#include "tokenizer.hpp"
#include "boost/compute/detail/lru_cache.hpp"

using namespace std;

class BPETokenizer : public Tokenizer {
    public:
        string toLang;
        vector<pair<string, string>> codes;
        string version;

        BPETokenizer(string toLang, string codes);
        vector<string> segmentTokens(vector<string> tokens);
        vector<string> encode(string input);
        string decode(vector<string> const tokens);
        void freeTokenizer();

    private:
        boost::compute::detail::lru_cache<string, vector<string>> cache;
        vector<pair<string, string>> createPairs(string input);
        pair<string, string> minPair(vector<pair<string, string>> input);
        bool replace(std::string& str, const std::string& from, const std::string& to);
        char last(string input);
};