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
use regex::Regex;
use std::cell::Cell;
use std::collections::HashMap;
use tokenizers::models::bpe::BPE;

// surely this is enough, right?
// every previous time, we literally broke the C++ compiler and made Rust actually panic every time
// we were to call into the BPE code.
static IS_ALPHA: &str = r##"([^\d\s$&+,:;=?@#|'<>.^*()%!\-"`\[\]_{}/~])"##;

// https://github.com/hplt-project/sacremoses/blob/master/sacremoses/tokenize.py
static FINNISH_MORPHSET_1: &str = "N n A a Ä ä ssa Ssa ssä Ssä sta stä Sta Stä hun Hun hyn Hyn han Han hän Hän hön Hön un Un yn Yn an An än Än ön Ön seen Seen lla Lla llä Llä lta Lta ltä Ltä lle Lle ksi Ksi kse Kse tta Tta ine Ine";
static FINNISH_MORPHSET_2: &str = "ni si mme nne nsa";
static FINNISH_MORPHSET_3: &str = "ko kö han hän pa pä kaan kään kin";

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
                format!(r"^['][{}]", IS_ALPHA).as_str()
            ).unwrap(),
            IS_FRENCH_CONTRACTION: Regex::new(
                format!(r"[{}][']$", IS_ALPHA).as_str()
            ).unwrap(),
            STARTS_WITH_ALPHA: Regex::new(
                format!(r"^[{}]", IS_ALPHA).as_str()
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
            ]
        }
    }
}

pub struct BPETokenizer {
    //decoder: tokenizers::decoders::bpe::BPEDecoder,
    pub(crate) tokenizer: BPE,
    codes: HashMap<(String, String), usize>,
    cache: Cell<HashMap<String, Vec<String>>>,
    version: String,
    constants: BPEConstants
}

impl BPETokenizer {
    pub fn new(codes: &str) -> BPETokenizer {
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
            let token1 = split.get(0).unwrap();
            let token2 = split.get(1).unwrap();

            bpe_codes.insert((token1.to_string(), token2.to_string()), offset);
            offset += 1;
        }

        BPETokenizer {
            codes: bpe_codes,
            cache: Cell::new(HashMap::new()),
            version,
            tokenizer: BPE::builder()
                .end_of_word_suffix("</w>".to_string())
                .build().unwrap(),
            constants: BPEConstants::default()
        }
    }

    fn min(&self, pairs: &Vec<(String, String)>) -> (String, String) {
        let mut current_key = usize::MAX;
        let mut min: &(String, String) = &(String::new(), String::new());

        for pair in pairs {
            let key = self.codes.get(&pair).or_else(|| { Some(&usize::MAX) }).unwrap();

            if std::cmp::min(current_key, *key) != current_key {
                min = pair;
                current_key = *key;
            }
        }

        min.clone()
    }

    pub fn segment_tokens(&self, tokens: Vec<String>) -> Vec<String> {
        let mut output: Vec<String> = Vec::new();

        for token in tokens {
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

impl Tokenizer for BPETokenizer {
    fn encode(&self, input: &str) -> anyhow::Result<Vec<String>> {
        let mut cache = self.cache.take();

        if cache.contains_key(input) {
            let cached = cache.get(input).unwrap();
            return Ok(cached.clone());
        }

        let mut word = if self.version == "0.1" {
            let mut vec: Vec<String> = Vec::new();
            for char in input.chars() {
                vec.push(char.to_string());
            }

            vec.push("</w>".to_string());

            vec
        } else if self.version == "0.2" {
            let mut vec: Vec<String> = Vec::new();
            let mut chars = input.chars();
            let fucksake = chars.next_back().unwrap().to_string();
            let last = fucksake.as_str();

            for char in chars {
                vec.push(char.to_string());
            }

            let fuckoff = last.to_owned();
            let fuckoff2 = fuckoff + "</w>";
            let combined = fuckoff2.as_str();
            vec.push(combined.to_string());

            vec
        } else {
            panic!("Unsupported version: {} (input: {})", self.version, input);
        };

        let mut pairs = get_pairs(input);

        if pairs.is_empty() {
            return Ok(vec![input.to_string()]);
        };

        loop {
            let bigram = self.min(&pairs);
            if !self.codes.contains_key(&bigram) {
                break;
            }

            let first = bigram.0.as_str();
            let second = bigram.1.as_str();
            let mut new_word: Vec<String> = Vec::new();
            let mut i: usize = 0;

            while i < word.len() {
                let j = word.iter().position(|c| *c == first);

                if j.is_none() {
                    for pos in i..word.len() {
                        new_word.push(word[pos].to_string());
                    }
                } else {
                    for pos in i..j.unwrap() {
                        new_word.push(word[pos].to_string());
                    }
                }

                if word[i] == first && i < word.len() - 1 && word[i + 1] == second {
                    new_word.push(first.to_owned() + second);
                    i += 2;
                } else {
                    new_word.push(word[i].to_owned());
                    i += 1;
                }
            }

            word = new_word;
            if word.len() == 1 {
                break;
            } else {
                let str = word.join("");
                pairs = get_pairs(str.as_str());
            }
        }

        if word.last() == Some(&"</w>".to_string()) {
            word.pop();
        } else if word.last().unwrap().ends_with("</w>") {
            let last = word.last().unwrap().as_str();
            word.push(last.replace("</w>", "").to_string());
        }

        cache.insert(input.to_string(), word.clone());
        Ok(word)
    }

    fn decode(&self, tokens: Vec<String>) -> anyhow::Result<String> {
        let constants = &self.constants;

        let regexp = &constants.AGGRESSIVE_HYPHEN_SPLIT.0;
        let substitution = &constants.AGGRESSIVE_HYPHEN_SPLIT.1;
        let mut text = format!(" {} ", tokens.join(" "));
        text = regexp.replace_all(substitution, &text).to_string();
        text = self.constants.unescape_xml(&text);

        let mut prepend_space = " ".to_string();
        let mut detokenized_text = "".to_string();

        let tokens = text.split(" ").collect::<Vec<&str>>();
        let lang = "en"; // TODO: make this support other langs

        let mut quote_counts: HashMap<&str, usize> = HashMap::new();
        quote_counts.insert("'", 0);
        quote_counts.insert("\"", 0);
        quote_counts.insert("``", 0);
        quote_counts.insert("`", 0);
        quote_counts.insert("''", 0);

        let mut i = 0;
        for token in &tokens {
            let chars = token.chars().collect::<Vec<char>>();
            if self.constants.is_cjk(chars[0]) && lang != "ko" {
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
                && Regex::new(r"^[.,]+$").unwrap().is_match(tokens[tokens.len() - 1])
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
                && Regex::new(r"^[--]$").unwrap().is_match(tokens[i + 1])
                && Regex::new(r"(?i)^li$|^mail.*").unwrap().is_match(tokens[i + 2]) {
                detokenized_text += prepend_space.as_str();
                detokenized_text += token;
                detokenized_text += tokens[i + 1];
                prepend_space = "".to_string();
            } else if self.constants.IS_OPEN_QUOTE.is_match(token) {
                let mut normalized_quo = token;
                if self.constants.OPEN_QUOTES.is_match(token) {
                    normalized_quo = &"\"";
                }

                quote_counts.insert(normalized_quo, *quote_counts.get(normalized_quo).unwrap_or(&0));

                if lang == "cs" && *token == "„" {
                    quote_counts.insert(normalized_quo, 0);
                }

                if lang == "cs" && *token == "“" {
                    quote_counts.insert(normalized_quo, 1);
                }

                if quote_counts[normalized_quo] % 2 == 0 {
                    if lang == "en" && *token == "'" && i > 0 && self.constants.S_END.is_match(tokens[i - 1]) {
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
            } else if lang == "fi" && self.constants.COLON.is_match(tokens[i - 1]) && self.constants.FINNISH_REGEX.is_match(token) {
                detokenized_text += prepend_space.as_str();
                detokenized_text += token;
                prepend_space = " ".to_string();
            } else {
                detokenized_text += prepend_space.as_str();
                detokenized_text += token;
                prepend_space = " ".to_string();
            }

            i += 1;
        }

        detokenized_text = BPEConstants::substitute(detokenized_text, &self.constants.ONE_SPACE);
        detokenized_text = detokenized_text.trim_end().to_string();

        Ok(detokenized_text)
    }
}
