// Ported from https://github.com/UnityMultiplayer/UnityTranslateLib/blob/master/library/src/main/kotlin/xyz/bluspring/unitytranslate/library/util/BPETokenizer.kt
// which was ported from argos-translate's apply_bpe.py - https://github.com/argosopentech/argos-translate/blob/master/argostranslate/apply_bpe.py
// and from SacreMoses' tokenize.py - https://github.com/hplt-project/sacremoses/blob/master/sacremoses/tokenize.py
/*
Copyright (c) 2004-2020 Joerg Tiedemann

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"""

"""Use operations learned with learn_bpe.py to encode a new text.
The text will not be smaller, but use only a fixed vocabulary, with rare words
encoded as variable-length sequences of subword units.

Reference:
Rico Sennrich, Barry Haddow and Alexandra Birch (2015). Neural Machine Translation of Rare Words with Subword Units.
Proceedings of the 54th Annual Meeting of the Association for Computational Linguistics (ACL 2016). Berlin, Germany.
 */

#include <set>
#include <sstream>
#include <vector>
#include <chrono>
#include <ctime>
#include "boost/algorithm/string.hpp"
#include "boost/algorithm/string/join.hpp"
#include "boost/compute/detail/lru_cache.hpp"
#include "bpe_props.hpp"
#include "bpe_tokenizer.hpp"
#include <re2/re2.h>

using namespace std;

regex DASH_REGEX("^[--]$");
regex MAIL_REGEX("(?i)^li$||^mail.*");

BPETokenizer::BPETokenizer(string toLang, string codes) : cache(50 * 1024 * 1024) /* 50 MB cache */ {
    vector<pair<string, string>> bpeCodes;
    string versionPrefix = "#version:";
    string version = "0.1";

    istringstream codesStream(codes);

    string current;
    while (getline(codesStream, current)) {
        if (current.rfind(versionPrefix, 0) == 0) { // Set current version
            version = current.substr(versionPrefix.length());
            continue;
        }

        // Add codes
        size_t pos = current.find(" ");
        string firstCode = current.substr(0, pos);
        string secondCode = current.substr(pos + 1);

        bpeCodes.push_back(make_pair(firstCode, secondCode));
    }

    this->codes = bpeCodes;
    this->toLang = toLang;
    this->version = version;
};

vector<string> BPETokenizer::segmentTokens(vector<string> tokens) {
    vector<string> output;

    for (const string word : tokens) {
        // Eliminate double spaces
        string wordCopy = word;
        boost::trim(wordCopy);
        if (wordCopy.length() == 0)
            continue;

        for (const string newWord : this->encode(word)) {
            output.push_back(newWord);
        }
    }

    return output;
}

vector<string> BPETokenizer::encode(string input) {
    if (this->cache.contains(input)) {
        return this->cache.get(input).get();
    }

    vector<string> word;
    if (this->version.starts_with("0.1")) {
        for (const char c : input) {
            string part(1, c);
            word.push_back(part);
        }

        word.push_back("</w>");
    } else if (this->version.starts_with("0.2")) {
        for (const char c : input.substr(0, input.length() - 1)) {
            string part(1, c);
            word.push_back(part);
        }

        word.push_back(input[input.length() - 1] + "</w>");
    } else {
        throw "Unsupported BPE version: " + this->version;
    }

    vector<pair<string, string>> pairs = createPairs(input);

    if (pairs.size() == 0) {
        vector<string> output;
        output.push_back(input);
        this->cache.insert(input, output);

        return output;
    }

    while (true) {
        pair<string, string> bigram = minPair(pairs);
                
        // If the bigram doesn't exist, just exit.
        if (this->codes.empty() || find(this->codes.begin(), this->codes.end(), bigram) == this->codes.end()) {
            break;
        }

        string first = bigram.first;
        string second = bigram.second;

        vector<string> newWord;
        int i = 0;

        while (i < word.size()) {
            int j = find(word.begin(), word.end(), first) - word.begin();

            if (j == -1) {
                for (int pos = i; pos <= word.size(); pos++) {
                    newWord.push_back(word[pos]);
                }
            } else {
                for (int pos = i; pos <= j; pos++) {
                    newWord.push_back(word[pos]);
                }
            }

            if (word[i] == first && i < word.size() - 1 && word[i + 1] == second) {
                newWord.push_back(first + second);
                i += 2;
            } else {
                newWord.push_back(word[i]);
                i += 1;
            }
        }

        word = newWord;

        if (word.size() == 1) {
            break;
        } else {
            string joined = boost::algorithm::join(word, "");
            pairs = createPairs(joined);
        }
    }

    string last = word[word.size() - 1];
    if (last == "</w>") {
        word.resize(word.size() - 1);
    } else if (last.ends_with("</w>")) {
        word.resize(word.size() - 1);
        replace(last, "</w>", "");
        word.push_back(last);
    }

    return word;
}

string BPETokenizer::decode(vector<string> const tokens) {
    string joined = boost::algorithm::join(tokens, " ");
    string newString = joined;
    replace(newString, "@@ ", "");

    vector<string> splitTokens = split(newString, " ");
            
    regex regexp = BPEProps::AGGRESSIVE_HYPHEN_SPLIT.first;
    string substitution = BPEProps::AGGRESSIVE_HYPHEN_SPLIT.second;
    string text = " " + boost::algorithm::join(splitTokens, " ") + " ";
    text = regex_replace(substitution, regexp, text);
    text = BPEProps::unescapeXml(text);

    string prependSpace = " ";
    string detokenizedText = "";

    vector<string> textTokens = split(text, " ");
    string lang = this->toLang;

    map<string, int> quoteCounts {
        {"'", 0},
        {"\"", 0},
        {"``", 0},
        {"`", 0},
        {"''", 0}
    };

    int i = 0;

    /*for (const string token : textTokens) {
        if (BPEProps::isCJK(token.at(0)) && lang != "ko") {
            if (i > 0 && BPEProps::isCJK(last(splitTokens.at(i - 1)))) {
                detokenizedText += token;
            } else {
                detokenizedText += prependSpace;
                detokenizedText += token;
            }

            prependSpace = " ";
        } else if (regex_match(token, BPEProps::IS_CURRENCY_SYMBOL)) {
            detokenizedText += prependSpace;
            detokenizedText += token;
            prependSpace = " ";
        } else if (regex_match(token, BPEProps::IS_PUNCT)) {
            if (lang == "fr" && regex_match(token, BPEProps::SYMBOLS)) {
                detokenizedText += " ";
            }

            detokenizedText += token;
            prependSpace = " ";
        } else if (lang == "en" && i > 0 && RE2::PartialMatch(token, BPEProps::IS_ENGLISH_CONTRACTION)) {
            detokenizedText += token;
            prependSpace = " ";
        } else if ((lang == "fr" || lang == "it" || lang == "ga") && i <= splitTokens.size() - 2
            && RE2::PartialMatch(token, BPEProps::IS_FRENCH_CONTRACTION)
            && RE2::PartialMatch(splitTokens.at(i + 1), BPEProps::STARTS_WITH_ALPHA)
        ) {
            detokenizedText += prependSpace;
            detokenizedText += token;

            prependSpace = "";
        } else if (lang == "cs" && i <= splitTokens.size() - 3 && RE2::PartialMatch(token, BPEProps::IS_FRENCH_CONTRACTION)
            && regex_match(splitTokens.at(i + 1), DASH_REGEX)
            && regex_match(splitTokens.at(i + 2), MAIL_REGEX)
        ) {
            detokenizedText += prependSpace;
            detokenizedText += token;
            detokenizedText += splitTokens.at(i + 1);

            prependSpace = "";
        } else if (regex_match(token, BPEProps::IS_OPEN_QUOTE)) {
            string normalizedQuo = token;
            if (regex_match(token, BPEProps::OPEN_QUOTES)) {
                normalizedQuo = "\"";
            }

            if (quoteCounts.contains(normalizedQuo))
                quoteCounts.insert_or_assign(normalizedQuo, quoteCounts.at(normalizedQuo) + 1);
            else
                quoteCounts.insert_or_assign(normalizedQuo, 1);

            if (lang == "cs" && token == "„") {
                quoteCounts.insert_or_assign(normalizedQuo, 0);
            }

            if (lang == "cs" && token == "“") {
                quoteCounts.insert_or_assign(normalizedQuo, 1);
            }

            if (!quoteCounts.contains(normalizedQuo) || quoteCounts.at(normalizedQuo) % 2 == 0) {
                if (lang == "en" && token == "'" && i > 0 && regex_match(splitTokens.at(i - 1), BPEProps::S_END)) {
                    detokenizedText += token;
                    prependSpace = " ";
                } else {
                    detokenizedText += prependSpace;
                    detokenizedText += token;
                    if (quoteCounts.contains(normalizedQuo))
                        quoteCounts.insert_or_assign(normalizedQuo, quoteCounts.at(normalizedQuo) + 1);
                    else
                        quoteCounts.insert_or_assign(normalizedQuo, 1);
                }
            } else {
                detokenizedText += token;
                prependSpace = " ";
                if (quoteCounts.contains(normalizedQuo))
                    quoteCounts.insert_or_assign(normalizedQuo, quoteCounts.at(normalizedQuo) + 1);
                else
                    quoteCounts.insert_or_assign(normalizedQuo, 1);
            }
        } else if (lang == "fi" && regex_match(splitTokens.at(i - 1), BPEProps::COLON) && regex_match(token, BPEProps::FINNISH_REGEX)) {
            detokenizedText += prependSpace;
            detokenizedText += token;
            prependSpace = " ";
        } else {
            detokenizedText += prependSpace;
            detokenizedText += token;
            prependSpace = " ";
        }

        i += 1;
    }*/

    detokenizedText = substitute(detokenizedText, BPEProps::ONE_SPACE);
    boost::algorithm::trim_right(detokenizedText);

    return detokenizedText;
}

vector<pair<string, string>> BPETokenizer::createPairs(string input) {
    vector<pair<string, string>> pairs;

    // Avoid an accidental illegal access
    if (input.length() == 0) {
        return pairs;
    }

    string prevChar(1, input[0]);
    for (int i = 0; i < input.length() - 1; i++) {
        string currentChar(1, input[i]);

        pair<string, string> currentPair(prevChar, currentChar);

        pairs.push_back(currentPair);
        prevChar = currentChar;
    }

    return pairs;
}

pair<string, string> BPETokenizer::minPair(vector<pair<string, string>> input) {
    int currentKey = numeric_limits<int>::max();
    pair<string, string> min("", "");

    for (const pair<string, string> pair : input) {
        int key = find(this->codes.begin(), this->codes.end(), pair) - this->codes.begin();
        if (key >= this->codes.size()) {
            key = numeric_limits<int>::max();
        }

        if (std::min(currentKey, key) != currentKey) {
            min = pair;
            currentKey = key;
        }
    }

    return min;
}

bool BPETokenizer::replace(std::string& str, const std::string& from, const std::string& to) {
    size_t start_pos = str.find(from);
    if (start_pos == std::string::npos)
        return false;

    str.replace(start_pos, from.length(), to);
    return true;
}

char BPETokenizer::last(string input) {
    return input[input.length() - 1];
}

void BPETokenizer::freeTokenizer() {
    this->cache.clear();
    this->codes.clear();
}