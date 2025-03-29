use std::cell::Cell;
// Ported from argos-translate's apply_bpe.py - https://github.com/argosopentech/argos-translate/blob/master/argostranslate/apply_bpe.py
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
use std::collections::HashMap;
use std::ops::Index;
use ct2rs::Tokenizer;
use tokenizers::models::bpe::BPE;
use tokenizers::{Normalizer, Token};

pub struct BPETokenizer {
    //decoder: tokenizers::decoders::bpe::BPEDecoder,
    pub(crate) tokenizer: BPE,
    codes: HashMap<(String, String), usize>,
    cache: Cell<HashMap<String, Vec<String>>>,
    version: String
}

impl BPETokenizer {
    pub fn new(codes: &str) -> BPETokenizer {
        let mut offset = 1;
        let mut bpe_codes: HashMap<(String, String), usize> = HashMap::new();
        let mut version: String = "0.1".to_string();

        for line in codes.lines() {
            if line.starts_with("#version: ") {
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
                .build().unwrap()
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
            return Ok(cache.get(input).unwrap().clone());
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
        todo!()
    }
}