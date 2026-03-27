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

        BPETokenizer(string toLang, const string& codes);
        vector<string> segmentTokens(const vector<string>& tokens);
        vector<string> encode(string input) override;
        string decode(vector<string> tokens) override;
        void freeTokenizer() override;

    private:
        boost::compute::detail::lru_cache<string, vector<string>> cache;

        static vector<pair<string, string>> createPairs(const string &input);
        pair<string, string> minPair(const vector<pair<string, string>>& input);

        static bool replace(std::string& str, const std::string& from, const std::string& to);

        static char last(const string &input);
};