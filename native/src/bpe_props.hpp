#pragma once
#include <string>
#include "utils.hpp"
#include <regex>
#include "re2/re2.h"

using namespace std;

struct Range {
    int from;
    int to;
};

inline Range range(const int from, const int to) {
    Range range{};

    range.from = from;
    range.to = to;

    return range;
}

inline string substitute(const string& input, const pair<regex, string>& pair) {
    return regex_replace(input, pair.first, pair.second);
}

namespace BPEProps {
    // custom
    static string IS_ALPHA = R"([^\d\s$&+,:;=?@#|'<>.^*()%!\-"`\[\]\(\)_{}\/~])"; // surely this is enough, right? (last time we broke the compiler lmao)

    // https://www.kaggle.com/datasets/nltkdata/perluniprops
    static string FINNISH_MORPHSET_1 = "N n A a Ä ä ssa Ssa ssä Ssä sta stä Sta Stä hun Hun hyn Hyn han Han hän Hän hön Hön un Un yn Yn an An än Än ön Ön seen Seen lla Lla llä Llä lta Lta ltä Ltä lle Lle ksi Ksi kse Kse tta Tta ine Ine";
    static string FINNISH_MORPHSET_2 = "ni si mme nne nsa";
    static string FINNISH_MORPHSET_3 = "ko kö han hän pa pä kaan kään kin";

    static pair<regex, string> AGGRESSIVE_HYPHEN_SPLIT(regex("@-@"), "");

    // Merge multiple spaces.
    static pair<regex, string> ONE_SPACE(regex(" {2,}"), " ");

    // Unescape special characters.
    static pair<regex, string> UNESCAPE_FACTOR_SEPARATOR(regex("&#124;"), "|");
    static pair<regex, string> UNESCAPE_LEFT_ANGLE_BRACKET(regex("&lt;"), "<");
    static pair<regex, string> UNESCAPE_RIGHT_ANGLE_BRACKET(regex("&gt;"), ">");
    static pair<regex, string> UNESCAPE_DOUBLE_QUOTE(regex("&quot;"), "\"");
    static pair<regex, string> UNESCAPE_SINGLE_QUOTE(regex("&apos;"), "'");
    static pair<regex, string> UNESCAPE_SYNTAX_NONTERMINAL_LEFT(regex("&#91;"), "[");
    static pair<regex, string> UNESCAPE_SYNTAX_NONTERMINAL_RIGHT(regex("&#93;"), "]");
    static pair<regex, string> UNESCAPE_AMPERSAND(regex("&amp;"), "&");

    // The legacy regexes are used to support outputs from older Moses versions.
    static pair<regex, string> UNESCAPE_FACTOR_SEPARATOR_LEGACY(regex("&bar;"), "|");
    static pair<regex, string> UNESCAPE_SYNTAX_NONTERMINAL_LEFT_LEGACY(regex("&bra;"), "[");
    static pair<regex, string> UNESCAPE_SYNTAX_NONTERMINAL_RIGHT_LEGACY(regex("&ket;"), "]");

    static RE2 FINNISH_REGEX("^(N|n|A|a|Ä|ä|ssa|Ssa|ssä|Ssä|sta|stä|Sta|Stä|hun|Hun|hyn|Hyn|han|Han|hän|Hän|hön|Hön|un|Un|yn|Yn|an|An|än|Än|ön|Ön|seen|Seen|lla|Lla|llä|Llä|lta|Lta|ltä|Ltä|lle|Lle|ksi|Ksi|kse|Kse|tta|Tta|ine|Ine)(ni|si|mme|nne|nsa)(ko|kö|han|hän|pa|pä|kaan|kään|kin)$");

    static RE2 IS_CURRENCY_SYMBOL("^[$¢£¤¥֏؋৲৳৻૱௹฿៛₠₡₢₣₤₥₦₧₨₩₪₫€₭₮₯₰₱₲₳₴₵₶₷₸₹₺₻₼₽꠸﷼﹩＄￠￡￥￦;{¿¡]+$");
    static RE2 IS_ENGLISH_CONTRACTION("^'" + IS_ALPHA);
    static RE2 IS_FRENCH_CONTRACTION(IS_ALPHA + "'");
    static RE2 STARTS_WITH_ALPHA("^" + IS_ALPHA);
    static RE2 IS_PUNCT("^[,.?!:;%}]\\)]+$");
    static RE2 IS_OPEN_QUOTE("^['\"„“`]+$");

    static RE2 SYMBOLS("^[?!:;%]$");
    static RE2 NUMBERS("^[0-9]+$");
    static RE2 S_END("s$");
    static RE2 COLON("^:$");
    static RE2 OPEN_QUOTES("^[„“”]+$");

    static vector CJK_RANGES = {
        range(4352, 4607),
        range(11904, 42191),
        range(43072, 43135),
        range(44032, 55215),
        range(63744, 64255),
        range(65072, 65103),
        range(65381, 65500),
        range(94176, 94207),
        range(94208, 101119),
        range(110592, 110895),
        range(110960, 111359),
        range(131072, 196607)
    };

    static string unescapeXml(const string &input) {
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

    static bool isCJK(const char c) {
        for (const auto [from, to] : CJK_RANGES) {
            if (c >= from && c <= to) {
                return true;
            }
        }

        return false;
    }

}
