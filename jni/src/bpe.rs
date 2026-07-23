use ct2rs::Tokenizer;
use lazy_static::lazy_static;
use regex::Regex;
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
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashMap;

// surely this is enough, right?
// every previous time, we literally broke the C++ compiler and made Rust actually panic every time
// we were to call into the BPE code.
// static IS_ALPHA: &str = r##"([^\d\s$&+,:;=?@#|'<>.^*()%!\-"`\[\]_{}/~])"##;

// https://github.com/hplt-project/sacremoses/blob/master/sacremoses/tokenize.py
static FINNISH_MORPHSET_1: &str = "N n A a Ä ä ssa Ssa ssä Ssä sta stä Sta Stä hun Hun hyn Hyn han Han hän Hän hön Hön un Un yn Yn an An än Än ön Ön seen Seen lla Lla llä Llä lta Lta ltä Ltä lle Lle ksi Ksi kse Kse tta Tta ine Ine";
static FINNISH_MORPHSET_2: &str = "ni si mme nne nsa";
static FINNISH_MORPHSET_3: &str = "ko kö han hän pa pä kaan kään kin";

// kinda praying this works how i think it does
lazy_static! {
    static ref IS_ALNUM: &'static str = r"\p{L}\p{N}"; // [\p{L}\p{N}] maybe?
    static ref IS_N: &'static str = r"\p{N}";
    static ref IS_ALPHA: &'static str = r"\p{L}";
    static ref IS_SYMBOL: &'static str = r"\p{S}";

    static ref DEDUPLICATE_SPACE: Regex = Regex::new(r"\s+").unwrap();
    static ref ASCII_JUNK: Regex = Regex::new(r"[\x00-\x37]").unwrap();

    static ref PAD_NOT_ISALNUM: Regex = Regex::new(&format!(r"([^{}\s\.'\`\,\-])", *IS_ALNUM)).unwrap();
    // static ref AGGRESSIVE_HYPHEN_SPLIT: Regex = Regex::new(&format!(r"([{}])\-(?=[{}])", *IS_ALNUM, *IS_ALNUM)).unwrap();

    static ref EN_SPECIFIC_1: (Regex, String) = (Regex::new(&format!(r"([^{}])[']([^{}])", *IS_ALPHA, *IS_ALPHA)).unwrap(), r"$1 ' $2".to_string());
    static ref EN_SPECIFIC_2: (Regex, String) = (Regex::new(&format!(r"([^{}{}])[']([{}])", *IS_ALPHA, *IS_N, *IS_ALPHA)).unwrap(), r"$1 ' $2".to_string());
    static ref EN_SPECIFIC_3: (Regex, String) = (Regex::new(&format!(r"([{}])[']([^{}])", *IS_ALPHA, *IS_ALPHA)).unwrap(), r"$1 ' $2".to_string());
    static ref EN_SPECIFIC_4: (Regex, String) = (Regex::new(&format!(r"([{}])[']([{}])", *IS_ALPHA, *IS_ALPHA)).unwrap(), r"$1 '$2".to_string());
    static ref EN_SPECIFIC_5: (Regex, String) = (Regex::new(&format!(r"([{}])[']([s])", *IS_N)).unwrap(), r"$1 '$2".to_string());

    static ref FR_IT_SPECIFIC_1: (Regex, String) = (Regex::new(&format!(r"([^{}])[']([^{}])", *IS_ALPHA, *IS_ALPHA)).unwrap(), r"$1 ' $2".to_string());
    static ref FR_IT_SPECIFIC_2: (Regex, String) = (Regex::new(&format!(r"([^{}])[']([{}])", *IS_ALPHA, *IS_ALPHA)).unwrap(), r"$1 ' $2".to_string());
    static ref FR_IT_SPECIFIC_3: (Regex, String) = (Regex::new(&format!(r"([{}])[']([^{}])", *IS_ALPHA, *IS_ALPHA)).unwrap(), r"$1 ' $2".to_string());
    static ref FR_IT_SPECIFIC_4: (Regex, String) = (Regex::new(&format!(r"([{}])[']([{}])", *IS_ALPHA, *IS_ALPHA)).unwrap(), r"$1' $2".to_string());

    static ref COMMA_SEPARATE_1: Regex = Regex::new(&format!(r"([^{}])[,]", *IS_N)).unwrap();
    static ref COMMA_SEPARATE_2: Regex = Regex::new(&format!(r"[,]([^{}])", *IS_N)).unwrap();
    static ref COMMA_SEPARATE_3: Regex = Regex::new(&format!(r"([{}])[,]$", *IS_N)).unwrap();

    static ref ENGLISH_SPECIFIC_APOSTROPHE: Vec<&'static (Regex, String)> = vec![
        &EN_SPECIFIC_1,
        &EN_SPECIFIC_2,
        &EN_SPECIFIC_3,
        &EN_SPECIFIC_4,
        &EN_SPECIFIC_5,
    ];
    static ref FR_IT_SPECIFIC_APOSTROPHE: Vec<&'static (Regex, String)> = vec![
        &FR_IT_SPECIFIC_1,
        &FR_IT_SPECIFIC_2,
        &FR_IT_SPECIFIC_3,
        &FR_IT_SPECIFIC_4,
    ];

    static ref NON_SPECIFIC_APOSTROPHE: (Regex, String) = (Regex::new(r"\'").unwrap(), " ' ".to_string());

    static ref TRAILING_DOT_APOSTROPHE: (Regex, String) = (Regex::new(r"\.' ?$").unwrap(), " . ' ".to_string());

    static ref ESCAPE_AMPERSAND: (Regex, String) = (Regex::new(r"&").unwrap(), r"&amp;".to_string());
    static ref ESCAPE_PIPE: (Regex, String) = (Regex::new(r"\|").unwrap(), r"&#124;".to_string());
    static ref ESCAPE_LEFT_ANGLE_BRACKET: (Regex, String) = (Regex::new(r"<").unwrap(), r"&lt;".to_string());
    static ref ESCAPE_RIGHT_ANGLE_BRACKET: (Regex, String) = (Regex::new(r">").unwrap(), r"&gt;".to_string());
    static ref ESCAPE_SINGLE_QUOTE: (Regex, String) = (Regex::new(r"\'").unwrap(), r"&apos;".to_string());
    static ref ESCAPE_DOUBLE_QUOTE: (Regex, String) = (Regex::new(r#"""#).unwrap(), r"&quot;".to_string());
    static ref ESCAPE_LEFT_SQUARE_BRACKET: (Regex, String) = (Regex::new(r"\[").unwrap(), r"&#91;".to_string());
    static ref ESCAPE_RIGHT_SQUARE_BRACKET: (Regex, String) = (Regex::new(r"\]").unwrap(), r"&#93;".to_string());

    static ref MOSES_ESCAPE_XML_REGEXES: Vec<&'static (Regex, String)> = vec![
        &ESCAPE_AMPERSAND,
        &ESCAPE_PIPE,
        &ESCAPE_LEFT_ANGLE_BRACKET,
        &ESCAPE_RIGHT_ANGLE_BRACKET,
        &ESCAPE_SINGLE_QUOTE,
        &ESCAPE_DOUBLE_QUOTE,
        &ESCAPE_LEFT_SQUARE_BRACKET,
        &ESCAPE_RIGHT_SQUARE_BRACKET,
    ];
}

pub struct BPEConstants {
    AGGRESSIVE_HYPHEN_SPLIT: (Regex, String),

    // Merge multiple spaces.
    ONE_SPACE: (Regex, String),

    // Unescape special characters.
    UNESCAPE_FACTOR_SEPARATOR: (Regex, String),
    UNESCAPE_LEFT_ANGLE_BRACKET: (Regex, String),
    UNESCAPE_RIGHT_ANGLE_BRACKET: (Regex, String),
    UNESCAPE_DOUBLE_QUOTE: (Regex, String),
    UNESCAPE_SINGLE_QUOTE: (Regex, String),
    UNESCAPE_SYNTAX_NONTERMINAL_LEFT: (Regex, String),
    UNESCAPE_SYNTAX_NONTERMINAL_RIGHT: (Regex, String),
    UNESCAPE_AMPERSAND: (Regex, String),
    // The legacy regexes are used to support outputs from older Moses versions.
    UNESCAPE_FACTOR_SEPARATOR_LEGACY: (Regex, String),
    UNESCAPE_SYNTAX_NONTERMINAL_LEFT_LEGACY: (Regex, String),
    UNESCAPE_SYNTAX_NONTERMINAL_RIGHT_LEGACY: (Regex, String),

    FINNISH_REGEX: Regex,
    IS_CURRENCY_SYMBOL: Regex,
    IS_ENGLISH_CONTRACTION: Regex,
    IS_FRENCH_CONTRACTION: Regex,
    STARTS_WITH_ALPHA: Regex,
    IS_PUNCT: Regex,
    IS_OPEN_QUOTE: Regex,

    SYMBOLS: Regex,
    NUMBERS: Regex,
    S_END: Regex,
    COLON: Regex,
    OPEN_QUOTES: Regex,

    CJK_RANGES: Vec<(usize, usize)>,

    DASH_REGEX: Regex,
    MAIL_REGEX: Regex,
    DOT_COMMA_REGEX: Regex,
}

impl BPEConstants {
    pub fn substitute(text: String, pair: &(Regex, String)) -> String {
        pair.0.replace_all(text.as_str(), &pair.1).to_string()
    }

    pub fn unescape_xml(&self, text: &str) -> String {
        let mut unescaped_text = text.to_string();
        unescaped_text = Self::substitute(unescaped_text, &self.UNESCAPE_FACTOR_SEPARATOR_LEGACY);
        unescaped_text = Self::substitute(unescaped_text, &self.UNESCAPE_FACTOR_SEPARATOR);
        unescaped_text = Self::substitute(unescaped_text, &self.UNESCAPE_LEFT_ANGLE_BRACKET);
        unescaped_text = Self::substitute(unescaped_text, &self.UNESCAPE_RIGHT_ANGLE_BRACKET);
        unescaped_text = Self::substitute(unescaped_text, &self.UNESCAPE_SYNTAX_NONTERMINAL_LEFT_LEGACY);
        unescaped_text = Self::substitute(unescaped_text, &self.UNESCAPE_SYNTAX_NONTERMINAL_RIGHT_LEGACY);
        unescaped_text = Self::substitute(unescaped_text, &self.UNESCAPE_DOUBLE_QUOTE);
        unescaped_text = Self::substitute(unescaped_text, &self.UNESCAPE_SINGLE_QUOTE);
        unescaped_text = Self::substitute(unescaped_text, &self.UNESCAPE_SYNTAX_NONTERMINAL_LEFT);
        unescaped_text = Self::substitute(unescaped_text, &self.UNESCAPE_SYNTAX_NONTERMINAL_RIGHT);
        unescaped_text = Self::substitute(unescaped_text, &self.UNESCAPE_AMPERSAND);

        unescaped_text.to_string()
    }

    pub fn is_cjk(&self, char: char) -> bool {
        let char = char as u32;
        let cjk_ranges = &self.CJK_RANGES;

        for (start, end) in cjk_ranges {
            if char < *end as u32 {
                return char > *start as u32;
            }
        }

        false
    }
}

impl Default for BPEConstants {
    fn default() -> BPEConstants {
        BPEConstants {
            AGGRESSIVE_HYPHEN_SPLIT: (Regex::new(r"@-@").unwrap(), "".to_string()),

            // Merge multiple spaces.
            ONE_SPACE: (Regex::new(r" {2,}").unwrap(), " ".to_string()),

            // Unescape special characters.
            UNESCAPE_FACTOR_SEPARATOR: (Regex::new(r"&#124;").unwrap(), "|".to_string()),
            UNESCAPE_LEFT_ANGLE_BRACKET: (Regex::new(r"&lt;").unwrap(), "<".to_string()),
            UNESCAPE_RIGHT_ANGLE_BRACKET: (Regex::new(r"&gt;").unwrap(), ">".to_string()),
            UNESCAPE_DOUBLE_QUOTE: (Regex::new(r"&quot;").unwrap(), "\"".to_string()),
            UNESCAPE_SINGLE_QUOTE: (Regex::new(r"&apos;").unwrap(), "'".to_string()),
            UNESCAPE_SYNTAX_NONTERMINAL_LEFT: (Regex::new(r"&#91;").unwrap(), "[".to_string()),
            UNESCAPE_SYNTAX_NONTERMINAL_RIGHT: (Regex::new(r"&#93;").unwrap(), "]".to_string()),
            UNESCAPE_AMPERSAND: (Regex::new(r"&amp;").unwrap(), "&".to_string()),

            // The legacy regexes are used to support outputs from older Moses versions.
            UNESCAPE_FACTOR_SEPARATOR_LEGACY: (Regex::new(r"&bar;").unwrap(), "|".to_string()),
            UNESCAPE_SYNTAX_NONTERMINAL_LEFT_LEGACY: (Regex::new(r"&bra;").unwrap(), "[".to_string()),
            UNESCAPE_SYNTAX_NONTERMINAL_RIGHT_LEGACY: (Regex::new(r"&ket;").unwrap(), "]".to_string()),

            FINNISH_REGEX: Regex::new(format!(r"^({})({})?({})$",
                                              FINNISH_MORPHSET_1.split(" ").collect::<Vec<&str>>().join("|"),
                                              FINNISH_MORPHSET_2.split(" ").collect::<Vec<&str>>().join("|"),
                                              FINNISH_MORPHSET_3.split(" ").collect::<Vec<&str>>().join("|"),
            ).as_str()).unwrap(),

            IS_CURRENCY_SYMBOL: Regex::new(
                r"^[(\[$¢£¤¥֏؋৲৳৻૱௹฿៛₠₡₢₣₤₥₦₧₨₩₪₫€₭₮₯₰₱₲₳₴₵₶₷₸₹₺₻₼₽꠸﷼﹩＄￠￡￥￦;{¿¡]+$"
            ).unwrap(),
            IS_ENGLISH_CONTRACTION: Regex::new(
                &format!(r"^['][{}]", *IS_ALPHA).as_str()
            ).unwrap(),
            IS_FRENCH_CONTRACTION: Regex::new(
                &format!(r"[{}][']$", *IS_ALPHA).as_str()
            ).unwrap(),
            STARTS_WITH_ALPHA: Regex::new(
                &format!(r"^[{}]", *IS_ALPHA).as_str()
            ).unwrap(),
            IS_PUNCT: Regex::new(
                r"^[,.?!:;\\%}]\)]+$"
            ).unwrap(),
            IS_OPEN_QUOTE: Regex::new(
                r#"^['"„“`]+$"#
            ).unwrap(),

            SYMBOLS: Regex::new(r"^[?!:;\\%]$").unwrap(),
            NUMBERS: Regex::new(r"^[0-9]+$").unwrap(),
            S_END: Regex::new(r"s$").unwrap(),
            COLON: Regex::new(r"^:$").unwrap(),
            OPEN_QUOTES: Regex::new(r"^[„“”]+$").unwrap(),

            CJK_RANGES: vec![
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
            ],

            DASH_REGEX: Regex::new(r"^[--]$").unwrap(),
            MAIL_REGEX: Regex::new(r"(?i)^li$|^mail.*").unwrap(),
            DOT_COMMA_REGEX: Regex::new(r"^[.,]+$").unwrap(),
        }
    }
}

pub struct BPETokenizer {
    codes: HashMap<(String, String), usize>,
    cache: Cell<HashMap<String, Vec<String>>>,
    version: String,
    constants: BPEConstants,
    pub normalizer: PunctNormalizer,

    from_lang: String,
    to_lang: String,
}

impl BPETokenizer {
    pub fn new(codes: &str, from_lang: &str, to_lang: &str) -> BPETokenizer {
        let mut offset = 1;
        let mut bpe_codes: HashMap<(String, String), usize> = HashMap::new();
        let mut version: String = "0.1".to_string();

        for line in codes.lines() {
            if line.starts_with("version: ") {
                if line.starts_with("#version: 0.1") {
                    version = "0.1".to_string();
                } else if line.starts_with("#version: 0.2") {
                    version = "0.2".to_string();
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
            constants: BPEConstants::default(),
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
         if (false) {
             println!($($arg)*);
         }
    }};
}

impl BPETokenizer {
    // https://github.com/hplt-project/sacremoses/blob/master/sacremoses/tokenize.py#L431
    pub fn tokenize(&self, text: &str) -> String {
        let mut text = text.to_string();

        // de-duplicate spaces and clean ASCII junk
        text = DEDUPLICATE_SPACE.replace(&text, " ").to_string();
        text = ASCII_JUNK.replace(&text, "").to_string();

        // (we don't do protected patterns)

        // trim leading and trailing
        text = text.trim().to_string();

        // separate special characters outside IsAlnum charset
        text = PAD_NOT_ISALNUM.replace(&text, r" $1 ").to_string();

        // (we don't do aggressive dash splits)

        // replace multidots with "DOTDOTMULTI" literal
        text = self.replace_multidots(text);

        text = COMMA_SEPARATE_1.replace(&text, r"$1 , ").to_string();
        text = COMMA_SEPARATE_2.replace(&text, r" , $1").to_string();
        text = COMMA_SEPARATE_3.replace(&text, r"$1 , ").to_string();

        if self.from_lang == "en" {
            for x in ENGLISH_SPECIFIC_APOSTROPHE.iter() {
                text = x.0.replace(&text, x.1.as_str()).to_string();
            }
        } else if self.from_lang == "fr" || self.from_lang == "it" {
            for x in FR_IT_SPECIFIC_APOSTROPHE.iter() {
                text = x.0.replace(&text, x.1.as_str()).to_string();
            }
        } else {
            text = NON_SPECIFIC_APOSTROPHE.0.replace(&text, NON_SPECIFIC_APOSTROPHE.1.as_str()).to_string();
        }

        text = self.handles_nonbreaking_prefixes(text);

        text = DEDUPLICATE_SPACE.replace(&text, " ").trim().to_string();
        text = TRAILING_DOT_APOSTROPHE.0.replace(&text, TRAILING_DOT_APOSTROPHE.1.as_str()).to_string();

        // no protected patterns here

        text = self.restore_multidots(text);

        // we handle XML escapes
        text = self.escape_xml(text);

        text
    }

    fn escape_xml(&self, text: String) -> String {
        let mut text = text;
        for (regex, sub) in MOSES_ESCAPE_XML_REGEXES.iter() {
            text = regex.replace(&text, sub).to_string();
        }

        text
    }

    fn restore_multidots(&self, text: String) -> String {
        let mut text = text;
        let dotmulti = Regex::new(r"DOTDOTMULTI").unwrap();
        while dotmulti.find(text.as_str()).is_some() {
            text = dotmulti.replace(&text, r"DOTMULTI.").to_string();
        }

        let regex = Regex::new(r"DOTMULTI").unwrap();
        regex.replace(&text, ".").to_string()
    }

    fn replace_multidots(&self, text: String) -> String {
        let mut text = text;
        let dotmulti = Regex::new(r"DOTMULTI\.").unwrap();

        while dotmulti.find(text.as_str()).is_some() {
            let regex = Regex::new(r"DOTMULTI\.([^.])").unwrap();
            text = regex.replace(&text, r"DOTDOTMULTI $1").to_string();
            text = dotmulti.replace(&text, "DOTDOTMULTI").to_string();
        }

        text
    }

    fn handles_nonbreaking_prefixes(&self, text: String) -> String {
        let mut tokens = text.split(" ").collect::<Vec<&str>>();
        let num_tokens = tokens.len();

        for (i, token) in tokens.iter().enumerate() {
            // check if token ends w/ a full stop
            if let Some(token_ends_with_period) = Regex::new(r"^(\S+)\.$").unwrap().captures(token) {
                if let Some(prefix) = token_ends_with_period.get(1) {
                    let prefix_str = &prefix.as_str();
                    if (prefix_str.contains(".") && self.isanyalpha(prefix_str.to_string()))
                        //|| NONBREAKING_PREFIXES // we don't have prefix data
                        || (
                            i != num_tokens - 1 && tokens.len() > i + 1 && self.islower(tokens[i + 1].chars().next().unwrap())
                        )
                    {
                        continue; // no change to the token
                    }

                    // we don't have numeric only prefixes here

                    else {
                        // tokens[i] = (&prefix.to_string() + " .").as_str(); // how in the fuck
                    }
                }
            }
        }

        let joined = tokens.join(" ");
        joined
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

        let mut word = if self.version == "0.1" {
            debug_println_encode!("handle v0.1");
            let vec: Vec<String> = vec![
                input.to_string(),
                "</w>".to_string(),
            ];
            debug_println_encode!("done handle v0.1");

            vec
        } else if self.version == "0.2" {
            debug_println_encode!("handle v0.2");
            let mut vec: Vec<String> = Vec::new();
            let mut chars = input.chars();
            let fucksake = chars.next_back().unwrap().to_string();
            let last = fucksake.as_str();

            vec.push(chars.as_str().to_string());

            let fuckoff = last.to_owned();
            let fuckoff2 = fuckoff + "</w>";
            let combined = fuckoff2.as_str();
            vec.push(combined.to_string());
            debug_println_encode!("done handle v0.2");

            vec
        } else {
            panic!("Unsupported version: {} (input: {})", self.version, input);
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
        let constants = &self.constants;

        let regexp = &constants.AGGRESSIVE_HYPHEN_SPLIT.0;
        let substitution = &constants.AGGRESSIVE_HYPHEN_SPLIT.1;
        let mut text = format!(" {} ", tokens.join(" "));
        debug_println_decode!("Combined text {text}");
        text = regexp.replace_all(&text, substitution).to_string();
        debug_println_decode!("Replaced text {text}");
        text = self.constants.unescape_xml(&text);
        debug_println_decode!("Unescaped text {text}");

        let mut prepend_space = " ".to_string();
        let mut detokenized_text = "".to_string();

        let tokens = text.split(" ").collect::<Vec<&str>>();
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
        while i_mut < iter.len() {
            let i = i_mut;
            let token = iter.next().unwrap();
            i_mut += 1;
            let token = token.trim();

            if token.is_empty() {
                continue;
            }

            debug_println_decode!("Start loop tokens, on {token} at index {i}");

            let chars = token.chars().collect::<Vec<char>>();
            if self.constants.is_cjk(chars[0]) && lang != "ko" {
                debug_println_decode!("cjk char found, not ko");

                if i > 0 && self.constants.is_cjk(tokens[i - 1].chars().last().unwrap()) {
                    detokenized_text += token;
                } else {
                    detokenized_text += prepend_space.as_str();
                    detokenized_text += token;
                }

                prepend_space = " ".to_string();
            } else if self.constants.IS_CURRENCY_SYMBOL.is_match(token) {
                detokenized_text += prepend_space.as_str();
                detokenized_text += token;
                prepend_space = "".to_string();
            } else if self.constants.IS_PUNCT.is_match(token) {
                if lang == "fr" && self.constants.SYMBOLS.is_match(token) {
                    detokenized_text += " ";
                }

                detokenized_text += token;
                prepend_space = " ".to_string();
            } else if lang == "en" && i > 0 && self.constants.IS_ENGLISH_CONTRACTION.is_match(token) {
                detokenized_text += token;
                prepend_space = " ".to_string();
            } else if lang == "cs" && i > 1
                && self.constants.NUMBERS.is_match(tokens[tokens.len() - 2])
                && self.constants.DOT_COMMA_REGEX.is_match(tokens[tokens.len() - 1])
                && self.constants.NUMBERS.is_match(token) {
                detokenized_text += token;
                prepend_space = " ".to_string();
            } else if (lang == "fr" || lang == "it" || lang == "ga") && i <= tokens.len() - 2
                && self.constants.IS_FRENCH_CONTRACTION.is_match(token)
                && self.constants.STARTS_WITH_ALPHA.is_match(tokens[i + 1]) {
                detokenized_text += prepend_space.as_str();
                detokenized_text += token;
                prepend_space = "".to_string();
            } else if lang == "cs" && i <= tokens.len() - 3 && self.constants.IS_FRENCH_CONTRACTION.is_match(token)
                && self.constants.DASH_REGEX.is_match(tokens[i + 1])
                && self.constants.MAIL_REGEX.is_match(tokens[i + 2].to_lowercase().as_str()) {
                detokenized_text += prepend_space.as_str();
                detokenized_text += token;
                detokenized_text += tokens[i + 1];
                i_mut += 1;
                iter.next();
                prepend_space = "".to_string();
            } else if self.constants.IS_OPEN_QUOTE.is_match(token) {
                debug_println_decode!("open quote found");
                let mut normalized_quo = token;
                if self.constants.OPEN_QUOTES.is_match(token) {
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
                    if lang == "en" && token == "'" && i > 0 && self.constants.S_END.is_match(tokens[i - 1]) {
                        detokenized_text += token;
                        prepend_space = " ".to_string();
                    } else {
                        detokenized_text += prepend_space.as_str();
                        detokenized_text += token;
                        quote_counts.insert(normalized_quo, *quote_counts.get(normalized_quo).unwrap_or(&0) + 1);
                    }
                } else {
                    detokenized_text += token;
                    prepend_space = " ".to_string();
                    quote_counts.insert(normalized_quo, *quote_counts.get(normalized_quo).unwrap_or(&0) + 1);
                }

                debug_println_decode!("completed open quote");
            } else if lang == "fi" && self.constants.COLON.is_match(tokens[i - 1]) && self.constants.FINNISH_REGEX.is_match(token) {
                debug_println_decode!("is fi lang, colon matches and finnish regex");
                detokenized_text += prepend_space.as_str();
                detokenized_text += token;
                prepend_space = " ".to_string();
            } else {
                debug_println_decode!("none match, regular add {token}");
                detokenized_text += prepend_space.as_str();
                detokenized_text += token;
                prepend_space = " ".to_string();
            }

            debug_println_decode!("Detokenized text now {detokenized_text}, prepend space \"{prepend_space}\"");
        }

        detokenized_text = BPEConstants::substitute(detokenized_text, &self.constants.ONE_SPACE);
        detokenized_text = detokenized_text.trim().to_string();

        Ok(detokenized_text)
    }
}

struct NormalizerConstants {
    EXTRA_WHITESPACE: Vec<(String, String)>,
    REPLACE_UNICODE_PUNCTUATION: Vec<(String, String)>,
    NORMALIZE_UNICODE_IF_NOT_PENN: Vec<(String, String)>,
    NORMALIZE_UNICODE: Vec<(String, String)>,
    FRENCH_QUOTES: Vec<(String, String)>,
    HANDLE_PSEUDO_SPACES: Vec<(String, String)>,
    EN_QUOTATION_FOLLOWED_BY_COMMA: Vec<(String, String)>,
    DE_ES_FR_QUOTATION_FOLLOWED_BY_COMMA: Vec<(String, String)>,
    DE_ES_CZ_CS_FR: Vec<(String, String)>,
    OTHER: Vec<(String, String)>,
}

// ported from https://github.com/hplt-project/sacremoses/blob/master/sacremoses/normalize.py
pub struct PunctNormalizer {
    constants: NormalizerConstants,
    substitutions: Vec<(String, String)>,

    pre_replace_unicode_punct: bool,
    post_remove_control_chars: bool,
}

impl PunctNormalizer {
    pub fn new(lang: &str) -> PunctNormalizer {
        let constants = NormalizerConstants {
            REPLACE_UNICODE_PUNCTUATION: vec![
                ("，".to_string(), ",".to_string()),
                (r"。\s*".to_string(), ". ".to_string()),
                ("、".to_string(), ",".to_string()),
                ("”".to_string(), '"'.to_string()),
                ("“".to_string(), '"'.to_string()),
                ("∶".to_string(), ":".to_string()),
                ("：".to_string(), ":".to_string()),
                ("？".to_string(), "?".to_string()),
                ("《".to_string(), '"'.to_string()),
                ("》".to_string(), '"'.to_string()),
                ("）".to_string(), ")".to_string()),
                ("！".to_string(), "!".to_string()),
                ("（".to_string(), "(".to_string()),
                ("；".to_string(), ";".to_string()),
                ("」".to_string(), '"'.to_string()),
                ("「".to_string(), '"'.to_string()),
                ("０".to_string(), "0".to_string()),
                ("１".to_string(), "1".to_string()),
                ("２".to_string(), "2".to_string()),
                ("３".to_string(), "3".to_string()),
                ("４".to_string(), "4".to_string()),
                ("５".to_string(), "5".to_string()),
                ("６".to_string(), "6".to_string()),
                ("７".to_string(), "7".to_string()),
                ("８".to_string(), "8".to_string()),
                ("９".to_string(), "9".to_string()),
                (r"．\s*".to_string(), ". ".to_string()),
                ("～".to_string(), "~".to_string()),
                ("’".to_string(), "'".to_string()),
                ("…".to_string(), "...".to_string()),
                ("━".to_string(), "-".to_string()),
                ("〈".to_string(), "<".to_string()),
                ("〉".to_string(), ">".to_string()),
                ("【".to_string(), "[".to_string()),
                ("】".to_string(), "]".to_string()),
                ("％".to_string(), "%".to_string()),
            ],
            EXTRA_WHITESPACE: vec![  //lines 21 - 30
                (r"\r".to_string(), r"".to_string()),
                (r"\(".to_string(), r" (".to_string()),
                (r"\)".to_string(), r") ".to_string()),
                (r" +".to_string(), r" ".to_string()),
                (r"\) ([.!:?;,])".to_string(), r")\g<1>".to_string()),
                (r"\( ".to_string(), r"(".to_string()),
                (r" \)".to_string(), r")".to_string()),
                (r"(\d) %".to_string(), r"\g<1>%".to_string()),
                (r" :".to_string(), r":".to_string()),
                (r" ;".to_string(), r";".to_string()),
            ],

            NORMALIZE_UNICODE_IF_NOT_PENN: vec![
                (r"`".to_string(), r"'".to_string()),
                (r"''".to_string(), r##" " "##.to_string())
            ],  //lines 33 - 34

            NORMALIZE_UNICODE: vec![  //lines 37 - 50
                ("„".to_string(), r##"""##.to_string()),
                ("“".to_string(), r##"""##.to_string()),
                ("”".to_string(), r##"""##.to_string()),
                ("–".to_string(), r"-".to_string()),
                ("—".to_string(), r" - ".to_string()),
                (r" +".to_string(), r" ".to_string()),
                ("´".to_string(), r"'".to_string()),
                ("([a-zA-Z])‘([a-zA-Z])".to_string(), r"\g<1>'\g<2>".to_string()),
                ("([a-zA-Z])’([a-zA-Z])".to_string(), r"\g<1>'\g<2>".to_string()),
                ("‘".to_string(), r"'".to_string()),
                ("‚".to_string(), r"'".to_string()),
                ("’".to_string(), r"'".to_string()),
                (r"''".to_string(), r##"""##.to_string()),
                ("´´".to_string(), r##"""##.to_string()),
                ("…".to_string(), r"...".to_string()),
            ],

            FRENCH_QUOTES: vec![  //lines 52 - 57
                ("\u{00A0}«\u{00A0}".to_string(), r#"""#.to_string()),
                ("«\u{00A0}".to_string(), r#"""#.to_string()),
                ("«".to_string(), r#"""#.to_string()),
                ("\u{00A0}»\u{00A0}".to_string(), r#"""#.to_string()),
                ("\u{00A0}»".to_string(), r#"""#.to_string()),
                ("»".to_string(), r#"""#.to_string()),
            ],

            HANDLE_PSEUDO_SPACES: vec![  //lines 59 - 67
                ("\u{00A0}%".to_string(), r"%".to_string()),
                ("nº\u{00A0}".to_string(), "nº ".to_string()),
                ("\u{00A0}:".to_string(), r":".to_string()),
                ("\u{00A0}ºC".to_string(), " ºC".to_string()),
                ("\u{00A0}cm".to_string(), r" cm".to_string()),
                ("\u{00A0}\\?".to_string(), "?".to_string()),
                ("\u{00A0}\\!".to_string(), "!".to_string()),
                ("\u{00A0};".to_string(), r";".to_string()),
                (",\u{00A0}".to_string(), r", ".to_string()),
                (r" +".to_string(), r" ".to_string()),
            ],

            EN_QUOTATION_FOLLOWED_BY_COMMA: vec![
                (r##""([,.]+)"##.to_string(), r##"\g<1>""##.to_string())
            ],

            DE_ES_FR_QUOTATION_FOLLOWED_BY_COMMA: vec![
                (r##",""##.to_string(), r#"","#.to_string()),
                (r#"(\.+)"(\s*[^<])"#.to_string(), r#""\g<1>\g<2>"#.to_string()),  //don't fix period at end of sentence
            ],

            DE_ES_CZ_CS_FR: vec![
                ("(\\d)\u{00A0}(\\d)".to_string(), r"\g<1>,\g<2>".to_string()),
            ],

            OTHER: vec![
                ("(\\d)\u{00A0}(\\d)".to_string(), r"\g<1>.\g<2>".to_string()),
            ],
        };

        let mut substitutions = vec![
            constants.EXTRA_WHITESPACE.clone(),
            constants.NORMALIZE_UNICODE.clone(),
            constants.FRENCH_QUOTES.clone(),
            constants.HANDLE_PSEUDO_SPACES.clone(),
        ];

        let penn = true;
        let norm_quote_commas = true;
        let norm_numbers = true;
        let pre_replace_unicode_punct = false;
        let post_remove_control_chars = false;

        if penn {
            substitutions.insert(1, constants.NORMALIZE_UNICODE_IF_NOT_PENN.clone());
        }

        if norm_quote_commas {
            if lang == "en" {
                substitutions.push(constants.EN_QUOTATION_FOLLOWED_BY_COMMA.clone());
            } else if lang == "de" || lang == "es" || lang == "fr" {
                substitutions.push(constants.DE_ES_FR_QUOTATION_FOLLOWED_BY_COMMA.clone());
            }
        }

        if norm_numbers {
            if lang == "de" || lang == "es" || lang == "cz" || lang == "cs" || lang == "fr" {
                substitutions.push(constants.DE_ES_CZ_CS_FR.clone());
            } else {
                substitutions.push(constants.OTHER.clone());
            }
        }

        PunctNormalizer {
            constants,
            substitutions: substitutions.iter().flatten().cloned().collect(),
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
            let regex = Regex::new(x.0.as_str()).unwrap();
            let replaced = regex.replace(&text, x.1.as_str());
            text = replaced.to_string();
        }

        if self.post_remove_control_chars {
            text = self.remove_control_chars(text);
        }

        text.trim().to_string()
    }

    fn replace_unicode_punct(&self, text: String) -> String {
        let mut text: String = text.to_string();

        for x in &self.constants.REPLACE_UNICODE_PUNCTUATION {
            let regex = Regex::new(x.0.as_str()).unwrap();
            let replaced = regex.replace(&text, x.1.as_str());
            let str = replaced.to_string();
            text = str;
        }

        text
    }

    fn remove_control_chars(&self, text: String) -> String {
        let regex = Regex::new(r"\p{C}").unwrap();
        let replaced = regex.replace(&text, "");
        replaced.to_string()
    }
}
