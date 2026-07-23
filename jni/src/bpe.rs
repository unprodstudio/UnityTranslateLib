// Ported from argos-translate's apply_bpe.py - https://github.com/argosopentech/argos-translate/blob/master/argostranslate/apply_bpe.py
// and from SacreMoses' tokenize.py - https://github.com/hplt-project/sacremoses/blob/master/sacremoses/tokenize.py
// which is itself ported from Moses' tokenizer.perl and detokenizer.perl - https://github.com/moses-smt/mosesdecoder/blob/master/scripts/tokenizer/tokenizer.perl & https://github.com/moses-smt/mosesdecoder/blob/master/scripts/tokenizer/detokenizer.perl
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
use ct2rs::Tokenizer;
use regex::{regex, Regex};
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::LazyLock;

// surely this is enough, right?
// every previous time, we literally broke the C++ compiler and made Rust actually panic every time
// we were to call into the BPE code.
// static IS_ALPHA: &str = r##"([^\d\s$&+,:;=?@#|'<>.^*()%!\-"`\[\]_{}/~])"##;

// https://github.com/hplt-project/sacremoses/blob/master/sacremoses/tokenize.py
static FINNISH_MORPHSET_1: &str = "N n A a Ä ä ssa Ssa ssä Ssä sta stä Sta Stä hun Hun hyn Hyn han Han hän Hän hön Hön un Un yn Yn an An än Än ön Ön seen Seen lla Lla llä Llä lta Lta ltä Ltä lle Lle ksi Ksi kse Kse tta Tta ine Ine";
static FINNISH_MORPHSET_2: &str = "ni si mme nne nsa";
static FINNISH_MORPHSET_3: &str = "ko kö han hän pa pä kaan kään kin";

const IS_ALNUM: &str = r"\p{L}\p{N}"; // [\p{L}\p{N}] maybe?
const IS_N: &str = r"\p{N}";
const IS_ALPHA: &str = r"\p{L}";
const IS_SYMBOL: &str = r"\p{S}";

struct SubstitutionRule(LazyLock<Regex>, &'static str);

impl SubstitutionRule {
    const fn new(pattern: LazyLock<Regex>, replacement: &'static str) -> Self {
        Self(pattern, replacement)
    }

    fn substitute<'a>(&self, text: &'a str) -> Cow<'a, str> {
        self.0.replace_all(text, self.1)
    }
}

static DEDUPLICATE_SPACE: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"\s+").unwrap()), " ");
static ASCII_JUNK: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"[\x00-\x1f]").unwrap()), "");

static PAD_NOT_ISALNUM: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(&format!(r"([^{}\s\.'\`\,\-])", IS_ALNUM)).unwrap()), r" $1 ");

static EN_SPECIFIC_1: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(&format!(r"([^{}])[']([^{}])", IS_ALPHA, IS_ALPHA)).unwrap()), r"$1 ' $2");
static EN_SPECIFIC_2: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(&format!(r"([^{}{}])[']([{}])", IS_ALPHA, IS_N, IS_ALPHA)).unwrap()), r"$1 ' $2");
static EN_SPECIFIC_3: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(&format!(r"([{}])[']([^{}])", IS_ALPHA, IS_ALPHA)).unwrap()), r"$1 ' $2");
static EN_SPECIFIC_4: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(&format!(r"([{}])[']([{}])", IS_ALPHA, IS_ALPHA)).unwrap()), r"$1 '$2");
static EN_SPECIFIC_5: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(&format!(r"([{}])[']([s])", IS_N)).unwrap()), r"$1 '$2");

static ENGLISH_SPECIFIC_APOSTROPHE: &[&SubstitutionRule] = &[
    &EN_SPECIFIC_1,
    &EN_SPECIFIC_2,
    &EN_SPECIFIC_3,
    &EN_SPECIFIC_4,
    &EN_SPECIFIC_5,
];

static FR_IT_SPECIFIC_1: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(&format!(r"([^{}])[']([^{}])", IS_ALPHA, IS_ALPHA)).unwrap()), r"$1 ' $2");
static FR_IT_SPECIFIC_2: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(&format!(r"([^{}])[']([{}])", IS_ALPHA, IS_ALPHA)).unwrap()), r"$1 ' $2");
static FR_IT_SPECIFIC_3: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(&format!(r"([{}])[']([^{}])", IS_ALPHA, IS_ALPHA)).unwrap()), r"$1 ' $2");
static FR_IT_SPECIFIC_4: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(&format!(r"([{}])[']([{}])", IS_ALPHA, IS_ALPHA)).unwrap()), r"$1' $2");

static FR_IT_SPECIFIC_APOSTROPHE: &[&SubstitutionRule] = &[
    &FR_IT_SPECIFIC_1,
    &FR_IT_SPECIFIC_2,
    &FR_IT_SPECIFIC_3,
    &FR_IT_SPECIFIC_4,
];

static COMMA_SEPARATE_1: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(&format!(r"([^{}])[,]", IS_N)).unwrap()), r"$1 , ");
static COMMA_SEPARATE_2: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(&format!(r"[,]([^{}])", IS_N)).unwrap()), r" , $1");
static COMMA_SEPARATE_3: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(&format!(r"([{}])[,]$", IS_N)).unwrap()), r"$1 , ");

static NON_SPECIFIC_APOSTROPHE: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"'").unwrap()), " ' ");

static TRAILING_DOT_APOSTROPHE: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"\.' ?$").unwrap()), " . ' ");

static ESCAPE_AMPERSAND: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"&").unwrap()), r"&amp;");
static ESCAPE_PIPE: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"\|").unwrap()), r"&#124;");
static ESCAPE_LEFT_ANGLE_BRACKET: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"<").unwrap()), r"&lt;");
static ESCAPE_RIGHT_ANGLE_BRACKET: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r">").unwrap()), r"&gt;");
static ESCAPE_SINGLE_QUOTE: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"'").unwrap()), r"&apos;");
static ESCAPE_DOUBLE_QUOTE: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r#"""#).unwrap()), r"&quot;");
static ESCAPE_LEFT_SQUARE_BRACKET: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"\[").unwrap()), r"&#91;");
static ESCAPE_RIGHT_SQUARE_BRACKET: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"]").unwrap()), r"&#93;");

static MOSES_ESCAPE_XML_REGEXES: &[&SubstitutionRule] = &[
    &ESCAPE_AMPERSAND,
    &ESCAPE_PIPE,
    &ESCAPE_LEFT_ANGLE_BRACKET,
    &ESCAPE_RIGHT_ANGLE_BRACKET,
    &ESCAPE_SINGLE_QUOTE,
    &ESCAPE_DOUBLE_QUOTE,
    &ESCAPE_LEFT_SQUARE_BRACKET,
    &ESCAPE_RIGHT_SQUARE_BRACKET,
];

// Unescape special characters.
static UNESCAPE_FACTOR_SEPARATOR: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"&#124;").unwrap()), "|");
static UNESCAPE_LEFT_ANGLE_BRACKET: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"&lt;").unwrap()), "<");
static UNESCAPE_RIGHT_ANGLE_BRACKET: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"&gt;").unwrap()), ">");
static UNESCAPE_DOUBLE_QUOTE: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"&quot;").unwrap()), "\"");
static UNESCAPE_SINGLE_QUOTE: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"&apos;").unwrap()), "'");
static UNESCAPE_SYNTAX_NONTERMINAL_LEFT: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"&#91;").unwrap()), "[");
static UNESCAPE_SYNTAX_NONTERMINAL_RIGHT: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"&#93;").unwrap()), "]");
static UNESCAPE_AMPERSAND: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"&amp;").unwrap()), "&");

// The legacy regexes are used to support outputs from older Moses versions.
static UNESCAPE_FACTOR_SEPARATOR_LEGACY: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"&bar;").unwrap()), "|");
static UNESCAPE_SYNTAX_NONTERMINAL_LEFT_LEGACY: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"&bra;").unwrap()), "[");
static UNESCAPE_SYNTAX_NONTERMINAL_RIGHT_LEGACY: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"&ket;").unwrap()), "]");

static AGGRESSIVE_HYPHEN_SPLIT: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r"@-@").unwrap()), "");

static ONE_SPACE: SubstitutionRule = SubstitutionRule::new(LazyLock::new(|| Regex::new(r" {2,}").unwrap()), " ");

static FINNISH_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(&format!(r"^({})({})?({})$",
                                    FINNISH_MORPHSET_1.replace(' ', "|"),
                                    FINNISH_MORPHSET_2.replace(' ', "|"),
                                    FINNISH_MORPHSET_3.replace(' ', "|"),
)).unwrap());

static IS_CURRENCY_SYMBOL: LazyLock<Regex> = LazyLock::new(|| Regex::new(
    r"^[(\[$¢£¤¥֏؋৲৳৻૱௹฿៛₠₡₢₣₤₥₦₧₨₩₪₫€₭₮₯₰₱₲₳₴₵₶₷₸₹₺₻₼₽꠸﷼﹩＄￠￡￥￦;{¿¡]+$"
).unwrap());
static IS_ENGLISH_CONTRACTION: LazyLock<Regex> = LazyLock::new(|| Regex::new(
    &format!(r"^['][{}]", IS_ALPHA)
).unwrap());
static IS_FRENCH_CONTRACTION: LazyLock<Regex> = LazyLock::new(|| Regex::new(
    &format!(r"[{}][']$", IS_ALPHA)
).unwrap());
static STARTS_WITH_ALPHA: LazyLock<Regex> = LazyLock::new(|| Regex::new(
    &format!(r"^[{}]", IS_ALPHA)
).unwrap());
static IS_PUNCT: LazyLock<Regex> = LazyLock::new(|| Regex::new(
    r"^[,.?!:;\\%}]\)]+$"
).unwrap());
static IS_OPEN_QUOTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(
    r#"^['"„“`]+$"#
).unwrap());

static SYMBOLS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[?!:;\\%]$").unwrap());
static NUMBERS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]+$").unwrap());
static S_END: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"s$").unwrap());
static COLON: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^:$").unwrap());
static OPEN_QUOTES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[„“”]+$").unwrap());

static DASH_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[--]$").unwrap());
static MAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^li$|^mail.*").unwrap());
static DOT_COMMA_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[.,]+$").unwrap());

fn unescape_xml(text: &str) -> String {
    let text = UNESCAPE_FACTOR_SEPARATOR_LEGACY.substitute(text);
    let text = UNESCAPE_FACTOR_SEPARATOR.substitute(&text);
    let text = UNESCAPE_LEFT_ANGLE_BRACKET.substitute(&text);
    let text = UNESCAPE_RIGHT_ANGLE_BRACKET.substitute(&text);
    let text = UNESCAPE_SYNTAX_NONTERMINAL_LEFT_LEGACY.substitute(&text);
    let text = UNESCAPE_SYNTAX_NONTERMINAL_RIGHT_LEGACY.substitute(&text);
    let text = UNESCAPE_DOUBLE_QUOTE.substitute(&text);
    let text = UNESCAPE_SINGLE_QUOTE.substitute(&text);
    let text = UNESCAPE_SYNTAX_NONTERMINAL_LEFT.substitute(&text);
    let text = UNESCAPE_SYNTAX_NONTERMINAL_RIGHT.substitute(&text);
    let text = UNESCAPE_AMPERSAND.substitute(&text);

    text.to_string()
}

fn is_cjk(char: char) -> bool {
    const CJK_RANGES: [(u32, u32); 12] = [
        (4352, 4607),
        (11904, 42191),
        (43072, 43135),
        (44032, 55215),
        (63744, 64255),
        (65072, 65103),
        (65381, 65500),
        (94176, 94207),
        (94208, 101119),
        (110592, 110895),
        (110960, 111359),
        (131072, 196607)
    ];

    let char = char as u32;

    for (start, end) in CJK_RANGES {
        if char < end {
            return char > start;
        }
    }

    false
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum BPETokenizerVersion {
    V0_1,
    V0_2,
}

impl Display for BPETokenizerVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BPETokenizerVersion::V0_1 => f.write_str("0.1"),
            BPETokenizerVersion::V0_2 => f.write_str("0.2"),
        }
    }
}

pub struct BPETokenizer {
    codes: HashMap<(String, String), usize>,
    cache: Cell<HashMap<String, Vec<String>>>,
    version: BPETokenizerVersion,
    pub normalizer: PunctNormalizer,

    from_lang: String,
    to_lang: String,
}

impl BPETokenizer {
    pub fn new(codes: &str, from_lang: &str, to_lang: &str) -> BPETokenizer {
        let mut offset = 1;
        let mut bpe_codes: HashMap<(String, String), usize> = HashMap::new();
        let mut version = BPETokenizerVersion::V0_1;

        for line in codes.lines() {
            if line.starts_with("version: ") {
                if line.starts_with("#version: 0.1") {
                    version = BPETokenizerVersion::V0_1;
                } else if line.starts_with("#version: 0.2") {
                    version = BPETokenizerVersion::V0_2;
                }

                continue;
            }

            let split = line.split(" ").collect::<Vec<&str>>();
            let token1 = split.first().unwrap();
            let token2 = split.get(1).unwrap();

            bpe_codes.insert((token1.to_string(), token2.to_string()), offset);
            offset += 1;
        }

        BPETokenizer {
            codes: bpe_codes,
            cache: Cell::new(HashMap::new()),
            version,
            normalizer: PunctNormalizer::new(from_lang),
            from_lang: from_lang.to_string(),
            to_lang: to_lang.to_string(),
        }
    }

    fn min<'a>(&self, pairs: &'a [(String, String)]) -> &'a (String, String) {
        let codes = &self.codes;
        pairs.iter().min_by_key(|pair| codes.get(pair)).unwrap()
    }

    pub fn segment_tokens(&self, tokens: Vec<String>) -> Vec<String> {
        let mut output: Vec<String> = Vec::new();

        for token in tokens {
            // skip double spaces
            if token.is_empty() {
                continue;
            }

            for new_word in self.encode(token.as_str()).unwrap() {
                output.push(new_word);
            }
        }

        output
    }
}

fn get_pairs(word: &str) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut chars = word.chars();
    let mut prev_char = chars.next().unwrap();

    for char in chars {
        pairs.push((prev_char.to_string(), char.to_string()));
        prev_char = char;
    }

    pairs
}

macro_rules! debug_println_encode {
     ($($arg:tt)*) => {{
         if (false) {
             println!($($arg)*);
         }
    }};
}

macro_rules! debug_println_decode {
     ($($arg:tt)*) => {{
         if (true) {
             println!($($arg)*);
         }
    }};
}

macro_rules! debug_println_tokenize {
     ($($arg:tt)*) => {{
         if (false) {
             println!($($arg)*);
         }
    }};
}

impl BPETokenizer {
    // https://github.com/hplt-project/sacremoses/blob/master/sacremoses/tokenize.py#L431
    pub fn tokenize(&self, text: &str) -> String {
        debug_println_tokenize!("input: {text}");
        // de-duplicate spaces and clean ASCII junk
        let text = DEDUPLICATE_SPACE.substitute(text);
        debug_println_tokenize!("deduplicate space: {text}");
        let text = ASCII_JUNK.substitute(&text);
        debug_println_tokenize!("ascii junk: {text}");

        // (we don't do protected patterns)

        // trim leading and trailing
        let text = text.trim();
        debug_println_tokenize!("trimmed: {text}");

        // separate special characters outside IsAlnum charset
        let text = PAD_NOT_ISALNUM.substitute(text);
        debug_println_tokenize!("pad not is alphanumeric: {text}");

        // (we don't do aggressive dash splits)

        // replace multidots with "DOTDOTMULTI" literal
        let text = self.replace_multidots(&text);
        debug_println_tokenize!("replace multidots: {text}");

        let text = COMMA_SEPARATE_1.substitute(&text);
        debug_println_tokenize!("comma separate 1: {text}");
        let text = COMMA_SEPARATE_2.substitute(&text);
        debug_println_tokenize!("comma separate 2: {text}");
        let text = COMMA_SEPARATE_3.substitute(&text);
        debug_println_tokenize!("comma separate 3: {text}");

        let mut text = text.to_string();
        if self.from_lang == "en" {
            for x in ENGLISH_SPECIFIC_APOSTROPHE {
                text = x.substitute(&text).to_string();
            }
            debug_println_tokenize!("substituted all english specific apostrophes: {text}");
        } else if self.from_lang == "fr" || self.from_lang == "it" {
            for x in FR_IT_SPECIFIC_APOSTROPHE {
                text = x.substitute(&text).to_string();
            }

            debug_println_tokenize!("substituted all fr/it specific apostrophes: {text}");
        } else {
            text = NON_SPECIFIC_APOSTROPHE.substitute(&text).to_string();
            debug_println_tokenize!("substituted all non-specific apostrophes: {text}");
        }

        let text = self.handles_nonbreaking_prefixes(&text);
        debug_println_tokenize!("handled nonbreaking prefixes: {text}");

        let text = DEDUPLICATE_SPACE.substitute(&text);
        debug_println_tokenize!("dedup spaces: {text}");
        let text = TRAILING_DOT_APOSTROPHE.substitute(text.trim());
        debug_println_tokenize!("trailing dot apostrophes: {text}");

        // no protected patterns here

        let text = self.restore_multidots(&text);
        debug_println_tokenize!("restore multidots: {text}");

        // we handle XML escapes
        let text = self.escape_xml(&text);
        debug_println_tokenize!("escape xml: {text}");

        text.to_string()
    }

    fn escape_xml(&self, text: &str) -> String {
        let mut text = text.to_string();
        for pat in MOSES_ESCAPE_XML_REGEXES {
            text = pat.substitute(&text).to_string();
        }

        text.to_string()
    }

    fn restore_multidots(&self, text: &str) -> String {
        let mut text = text.to_string();
        while text.contains("DOTDOTMULTI") {
            text = text.replace("DOTDOTMULTI", r"DOTMULTI.");
        }

        text.replace("DOTMULTI", ".")
    }

    fn replace_multidots(&self, text: &str) -> String {
        let mut text = text.to_string();
        let dotmulti: &Regex = regex!(r"DOTMULTI\.");

        while dotmulti.is_match(text.as_str()) {
            let regex = regex!(r"DOTMULTI\.([^.])");
            text = regex.replace_all(&text, r"DOTDOTMULTI $1").to_string();
            text = dotmulti.replace_all(&text, "DOTDOTMULTI").to_string();
        }

        text
    }

    fn handles_nonbreaking_prefixes(&self, text: &str) -> String {
        let mut tokens: Vec<_> = text.split(' ').map(Cow::Borrowed).collect();
        let num_tokens = tokens.len();

        let mut i: usize = 0;
        let tokens_length = tokens.len();
        while i < tokens_length {
            let token = tokens.get(i).unwrap();
            // check if token ends w/ a full stop
            if let Some(token_ends_with_period) = regex!(r"^(\S+)\.$").captures(token)
                && let Some(prefix) = token_ends_with_period.get(1) {
                let prefix_str = prefix.as_str();
                if (prefix_str.contains(".") && self.isanyalpha(prefix_str.to_string()))
                    //|| NONBREAKING_PREFIXES // we don't have prefix data
                    || (i != num_tokens - 1 && tokens.len() > i + 1 && self.islower(tokens[i + 1].chars().next().unwrap())
                ) {
                    continue; // no change to the token
                }

                // we don't have numeric only prefixes here

                else {
                    tokens[i] = Cow::Owned(format!("{prefix_str} .")); // how in the fuck
                }
            }

            i += 1;
        }

        tokens.join(" ")
    }

    fn isanyalpha(&self, text: String) -> bool {
        for x in text.chars() {
            if x.is_alphabetic() {
                return true;
            }
        }

        false
    }

    fn islower(&self, c: char) -> bool {
        c.is_lowercase()
    }
}

impl Tokenizer for BPETokenizer {
    fn encode(&self, input: &str) -> anyhow::Result<Vec<String>> {
        let mut cache = self.cache.take();

        debug_println_encode!("Start BPE encode for {input}, we are v{0}", self.version);
        if cache.contains_key(input) {
            debug_println_encode!("Found cache");
            let cached = cache.get(input).unwrap();
            return Ok(cached.clone());
        }

        let mut word = match self.version {
            BPETokenizerVersion::V0_1 => {
                debug_println_encode!("handle v0.1");
                let vec: Vec<String> = vec![
                    input.to_string(),
                    "</w>".to_string(),
                ];
                debug_println_encode!("done handle v0.1");

                vec
            },
            BPETokenizerVersion::V0_2 => {
                debug_println_encode!("handle v0.2");
                let (head, tail) = input.split_at(input.floor_char_boundary(input.len() - 1));
                let vec = vec![head.to_string(), format!("{tail}</w>")];
                debug_println_encode!("done handle v0.2");

                vec
            }
        };

        let mut pairs = get_pairs(input);

        if pairs.is_empty() {
            debug_println_encode!("pairs are empty, returning.");
            return Ok(vec![input.to_string()]);
        };

        loop {
            debug_println_encode!("loop start");
            let bigram = self.min(&pairs);
            if !self.codes.contains_key(bigram) {
                debug_println_encode!("broke out, let's leave");
                break;
            }

            let first = bigram.0.as_str();
            let second = bigram.1.as_str();
            let mut new_word: Vec<String> = Vec::new();
            let mut i: usize = 0;

            while i < word.len() {
                debug_println_encode!("loop while");
                if let Some(j) = word.get(i..).unwrap().iter().position(|c| *c == first) {
                    debug_println_encode!("Got {i} start and {j} end");
                    for pos in i..j {
                        debug_println_encode!("loop for j exists with pos {pos}");
                        new_word.push(word[pos].to_string());
                    }

                    debug_println_encode!("Set to {}", j + i);
                    i += j;
                } else {
                    for pos in i..word.len() {
                        debug_println_encode!("loop for j none with pos {pos}");
                        new_word.push(word[pos].to_string());
                    }

                    debug_println_encode!("Breaking out");
                    break;
                }

                if word[i] == first && i < word.len() - 1 && word[i + 1] == second {
                    debug_println_encode!("loop while +2 (from {i})");
                    new_word.push(first.to_owned() + second);
                    i += 2;
                } else {
                    debug_println_encode!("loop while +1 (from {i})");
                    new_word.push(word[i].to_owned());
                    i += 1;
                }
            }

            debug_println_encode!("new word found");
            word = new_word;
            if word.len() == 1 {
                debug_println_encode!("word length == 1, break out");
                break;
            } else {
                debug_println_encode!("update pairs");
                let str = word.join("");
                pairs = get_pairs(str.as_str());
            }
        }

        if word.last() == Some(&"</w>".to_string()) {
            debug_println_encode!("equals close token, popping");
            word.pop();
        } else if word.last().unwrap().ends_with("</w>") {
            debug_println_encode!("ends with close token, popping");
            let last = word.last().unwrap().as_str();
            word.push(last.replace("</w>", "").to_string());
        }

        debug_println_encode!("insert");
        cache.insert(input.to_string(), word.clone());
        debug_println_encode!("done encode");
        Ok(word)
    }

    fn decode(&self, tokens: Vec<String>) -> anyhow::Result<String> {
        debug_println_decode!("Start BPE decode, we are v{0}", self.version);

        let mut text = format!(" {} ", tokens.join(" ")
            .replace("@@ ", "") // Argos adds this in
        );
        debug_println_decode!("Combined text {text}");
        text = AGGRESSIVE_HYPHEN_SPLIT.substitute(&text).to_string();
        debug_println_decode!("Replaced text {text}");
        text = unescape_xml(&text);
        debug_println_decode!("Unescaped text {text}");

        let mut prepend_space = " ";
        let mut detokenized_text = String::new();

        let tokens: Vec<_> = text.split(" ").collect();
        let lang = self.to_lang.as_str();
        debug_println_decode!("Language {lang}");

        debug_println_decode!("Quote counts init");
        let mut quote_counts: HashMap<&str, usize> = HashMap::new();
        quote_counts.insert("'", 0);
        quote_counts.insert("\"", 0);
        quote_counts.insert("``", 0);
        quote_counts.insert("`", 0);
        quote_counts.insert("''", 0);

        let mut i_mut: usize = 0;
        let mut iter = tokens.iter();
        while i_mut < tokens.len() {
            let i = i_mut;
            let token = iter.next().unwrap();
            i_mut += 1;
            let token = token.trim();

            if token.is_empty() {
                continue;
            }

            debug_println_decode!("Start loop tokens, on {token} at index {i}");

            let chars = token.chars().collect::<Vec<char>>();
            if is_cjk(chars[0]) && lang != "ko" {
                debug_println_decode!("cjk char found, not ko");

                if i > 0 && is_cjk(tokens[i - 1].chars().last().unwrap()) {
                    detokenized_text += token;
                } else {
                    detokenized_text += prepend_space;
                    detokenized_text += token;
                }

                prepend_space = " ";
            } else if IS_CURRENCY_SYMBOL.is_match(token) {
                detokenized_text += prepend_space;
                detokenized_text += token;
                prepend_space = "";
            } else if IS_PUNCT.is_match(token) {
                if lang == "fr" && SYMBOLS.is_match(token) {
                    detokenized_text += " ";
                }

                detokenized_text += token;
                prepend_space = " ";
            } else if lang == "en" && i > 0 && IS_ENGLISH_CONTRACTION.is_match(token) {
                detokenized_text += token;
                prepend_space = " ";
            } else if lang == "cs" && i > 1
                && NUMBERS.is_match(tokens[tokens.len() - 2])
                && DOT_COMMA_REGEX.is_match(tokens[tokens.len() - 1])
                && NUMBERS.is_match(token) {
                detokenized_text += token;
                prepend_space = " ";
            } else if (lang == "fr" || lang == "it" || lang == "ga") && i <= tokens.len() - 2
                && IS_FRENCH_CONTRACTION.is_match(token)
                && STARTS_WITH_ALPHA.is_match(tokens[i + 1]) {
                detokenized_text += prepend_space;
                detokenized_text += token;
                prepend_space = "";
            } else if lang == "cs" && i <= tokens.len() - 3 && IS_FRENCH_CONTRACTION.is_match(token)
                && DASH_REGEX.is_match(tokens[i + 1])
                && MAIL_REGEX.is_match(tokens[i + 2].to_lowercase().as_str()) {
                detokenized_text += prepend_space;
                detokenized_text += token;
                detokenized_text += tokens[i + 1];
                i_mut += 1;
                iter.next();
                prepend_space = "";
            } else if IS_OPEN_QUOTE.is_match(token) {
                debug_println_decode!("open quote found");
                let mut normalized_quo = token;
                if OPEN_QUOTES.is_match(token) {
                    normalized_quo = "\"";
                }

                quote_counts.insert(normalized_quo, *quote_counts.get(normalized_quo).unwrap_or(&0));

                if lang == "cs" && token == "„" {
                    quote_counts.insert(normalized_quo, 0);
                }

                if lang == "cs" && token == "“" {
                    quote_counts.insert(normalized_quo, 1);
                }

                if quote_counts[normalized_quo].is_multiple_of(2) {
                    if lang == "en" && token == "'" && i > 0 && S_END.is_match(tokens[i - 1]) {
                        detokenized_text += token;
                        prepend_space = " ";
                    } else {
                        detokenized_text += prepend_space;
                        detokenized_text += token;
                        prepend_space = "";
                        quote_counts.insert(normalized_quo, *quote_counts.get(normalized_quo).unwrap_or(&0) + 1);
                    }
                } else {
                    detokenized_text += token;
                    prepend_space = " ";
                    quote_counts.insert(normalized_quo, *quote_counts.get(normalized_quo).unwrap_or(&0) + 1);
                }

                debug_println_decode!("completed open quote");
            } else if lang == "fi" && COLON.is_match(tokens[i - 1]) && FINNISH_REGEX.is_match(token) {
                debug_println_decode!("is fi lang, colon matches and finnish regex");
                detokenized_text += prepend_space;
                detokenized_text += token;
                prepend_space = " ";
            } else {
                debug_println_decode!("none match, regular add {token}");
                detokenized_text += prepend_space;
                detokenized_text += token;
                prepend_space = " ";
            }

            debug_println_decode!("Detokenized text now {detokenized_text}, prepend space \"{prepend_space}\"");
        }

        Ok(ONE_SPACE.substitute(&detokenized_text).trim().to_string())
    }
}

static REPLACE_UNICODE_PUNCTUATION: [SubstitutionRule; 36] = [
    SubstitutionRule::new(LazyLock::new(|| Regex::new("，").unwrap()), ","),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r"。\s*").unwrap()), ". "),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("、").unwrap()), ","),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("”").unwrap()), "\""),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("“").unwrap()), "\""),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("∶").unwrap()), ":"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("：").unwrap()), ":"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("？").unwrap()), "?"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("《").unwrap()), "\""),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("》").unwrap()), "\""),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("）").unwrap()), ")"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("！").unwrap()), "!"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("（").unwrap()), "("),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("；").unwrap()), ";"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("」").unwrap()), "\""),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("「").unwrap()), "\""),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("０").unwrap()), "0"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("１").unwrap()), "1"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("２").unwrap()), "2"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("３").unwrap()), "3"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("４").unwrap()), "4"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("５").unwrap()), "5"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("６").unwrap()), "6"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("７").unwrap()), "7"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("８").unwrap()), "8"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("９").unwrap()), "9"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r"．\s*").unwrap()), ". "),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("～").unwrap()), "~"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("’").unwrap()), "'"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("…").unwrap()), "..."),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("━").unwrap()), "-"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("〈").unwrap()), "<"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("〉").unwrap()), ">"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("【").unwrap()), "["),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("】").unwrap()), "]"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("％").unwrap()), "%"),
];
static EXTRA_WHITESPACE: [SubstitutionRule; 10] = [  //lines 21 - 30
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r"\r").unwrap()), r""),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r"\(").unwrap()), r" ("),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r"\)").unwrap()), r") "),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r" +").unwrap()), r" "),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r"\) ([.!:?;,])").unwrap()), r")\g<1>"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r"\( ").unwrap()), r"("),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r" \)").unwrap()), r")"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r"(\d) %").unwrap()), r"\g<1>%"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r" :").unwrap()), r":"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r" ;").unwrap()), r";"),
];

static NORMALIZE_UNICODE_IF_NOT_PENN: [SubstitutionRule; 2] = [
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r"`").unwrap()), r"'"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r"''").unwrap()), r##" " "##)
];  //lines 33 - 34

static NORMALIZE_UNICODE: [SubstitutionRule; 15] = [  //lines 37 - 50
    SubstitutionRule::new(LazyLock::new(|| Regex::new("„").unwrap()), r##"""##),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("“").unwrap()), r##"""##),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("”").unwrap()), r##"""##),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("–").unwrap()), r"-"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("—").unwrap()), r" - "),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r" +").unwrap()), r" "),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("´").unwrap()), r"'"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("([a-zA-Z])‘([a-zA-Z])").unwrap()), r"\g<1>'\g<2>"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("([a-zA-Z])’([a-zA-Z])").unwrap()), r"\g<1>'\g<2>"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("‘").unwrap()), r"'"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("‚").unwrap()), r"'"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("’").unwrap()), r"'"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r"''").unwrap()), r##"""##),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("´´").unwrap()), r##"""##),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("…").unwrap()), r"..."),
];

static FRENCH_QUOTES: [SubstitutionRule; 6] = [  //lines 52 - 57
    SubstitutionRule::new(LazyLock::new(|| Regex::new("\u{00A0}«\u{00A0}").unwrap()), r#"""#),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("«\u{00A0}").unwrap()), r#"""#),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("«").unwrap()), r#"""#),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("\u{00A0}»\u{00A0}").unwrap()), r#"""#),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("\u{00A0}»").unwrap()), r#"""#),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("»").unwrap()), r#"""#),
];

static HANDLE_PSEUDO_SPACES: [SubstitutionRule; 10] = [  //lines 59 - 67
    SubstitutionRule::new(LazyLock::new(|| Regex::new("\u{00A0}%").unwrap()), r"%"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("nº\u{00A0}").unwrap()), "nº "),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("\u{00A0}:").unwrap()), r":"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("\u{00A0}ºC").unwrap()), " ºC"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("\u{00A0}cm").unwrap()), r" cm"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("\u{00A0}\\?").unwrap()), "?"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("\u{00A0}\\!").unwrap()), "!"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new("\u{00A0};").unwrap()), r";"),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(",\u{00A0}").unwrap()), r").unwrap()), "),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r" +").unwrap()), r" "),
];

static EN_QUOTATION_FOLLOWED_BY_COMMA: [SubstitutionRule; 1] = [
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r##""([,.]+)"##).unwrap()), r##"\g<1>""##)
];

static DE_ES_FR_QUOTATION_FOLLOWED_BY_COMMA: [SubstitutionRule; 2] = [
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r##",""##).unwrap()), r#"","#),
    SubstitutionRule::new(LazyLock::new(|| Regex::new(r#"(\.+)"(\s*[^<])"#).unwrap()), r#""\g<1>\g<2>"#),  //don't fix period at end of sentence
];

static DE_ES_CZ_CS_FR: [SubstitutionRule; 1] = [
    SubstitutionRule::new(LazyLock::new(|| Regex::new("(\\d)\u{00A0}(\\d)").unwrap()), r"\g<1>,\g<2>"),
];

static OTHER: [SubstitutionRule; 1] = [
    SubstitutionRule::new(LazyLock::new(|| Regex::new("(\\d)\u{00A0}(\\d)").unwrap()), r"\g<1>.\g<2>"),
];

// ported from https://github.com/hplt-project/sacremoses/blob/master/sacremoses/normalize.py
pub struct PunctNormalizer {
    substitutions: Vec<&'static SubstitutionRule>,

    pre_replace_unicode_punct: bool,
    post_remove_control_chars: bool,
}

impl PunctNormalizer {
    pub fn new(lang: &str) -> PunctNormalizer {
        let mut substitutions: Vec<&[SubstitutionRule]> = vec![
            &EXTRA_WHITESPACE,
            &NORMALIZE_UNICODE,
            &FRENCH_QUOTES,
            &HANDLE_PSEUDO_SPACES,
        ];

        let penn = true;
        let norm_quote_commas = true;
        let norm_numbers = true;
        let pre_replace_unicode_punct = false;
        let post_remove_control_chars = false;

        if penn {
            substitutions.insert(1, &NORMALIZE_UNICODE_IF_NOT_PENN);
        }

        if norm_quote_commas {
            if lang == "en" {
                substitutions.push(&EN_QUOTATION_FOLLOWED_BY_COMMA);
            } else if lang == "de" || lang == "es" || lang == "fr" {
                substitutions.push(&DE_ES_FR_QUOTATION_FOLLOWED_BY_COMMA);
            }
        }

        if norm_numbers {
            if lang == "de" || lang == "es" || lang == "cz" || lang == "cs" || lang == "fr" {
                substitutions.push(&DE_ES_CZ_CS_FR);
            } else {
                substitutions.push(&OTHER);
            }
        }

        PunctNormalizer {
            substitutions: substitutions.into_iter().flatten().collect(),
            pre_replace_unicode_punct,
            post_remove_control_chars,
        }
    }

    pub fn normalize<'a>(&'a self, text: &'a str) -> String {
        let mut text: String = text.to_string();
        if self.pre_replace_unicode_punct {
            text = self.replace_unicode_punct(text);
        }

        for x in &self.substitutions {
            text = x.substitute(&text).to_string();
        }

        if self.post_remove_control_chars {
            text = self.remove_control_chars(text);
        }

        text.trim().to_string()
    }

    fn replace_unicode_punct(&self, text: String) -> String {
        let mut text: String = text.to_string();

        for x in &REPLACE_UNICODE_PUNCTUATION {
            text = x.substitute(&text).to_string();
        }

        text
    }

    fn remove_control_chars(&self, text: String) -> String {
        regex!(r"\p{C}").replace_all(&text, "").to_string()
    }
}
