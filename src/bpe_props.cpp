#include <string>
#include <regex>
#include <ranges>
#include "boost/algorithm/string/join.hpp"
#include "utils.h"
#include "bpe_props.hpp"

using namespace std;

string BPEProps::unescapeXml(string input) {
    string copy = input;
    copy = substitute(copy, UNESCAPE_FACTOR_SEPARATOR_LEGACY);
    copy = substitute(copy, UNESCAPE_FACTOR_SEPARATOR);
    copy = substitute(copy, UNESCAPE_LEFT_ANGLE_BRACKET);
    copy = substitute(copy, UNESCAPE_RIGHT_ANGLE_BRACKET);
    copy = substitute(copy, UNESCAPE_SYNTAX_NONTERMINAL_LEFT_LEGACY);
    copy = substitute(copy, UNESCAPE_SYNTAX_NONTERMINAL_RIGHT_LEGACY);
    copy = substitute(copy, UNESCAPE_DOUBLE_QUOTE);
    copy = substitute(copy, UNESCAPE_SINGLE_QUOTE);
    copy = substitute(copy, UNESCAPE_SYNTAX_NONTERMINAL_LEFT);
    copy = substitute(copy, UNESCAPE_SYNTAX_NONTERMINAL_RIGHT);
    copy = substitute(copy, UNESCAPE_AMPERSAND);

    return copy;
}

bool BPEProps::isCJK(char c) {
    for (const Range range : CJK_RANGES) {
        if (c >= range.from && c <= range.to) {
            return true;
        }
    }

    return false;
}
