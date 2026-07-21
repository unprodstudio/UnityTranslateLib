use rust_tokenizers::tokenizer::{SentencePieceTokenizer, Tokenizer as RTTokenizer};

// I know I'm not good at Rust.
// But at least it's functional.... I think.

struct UnityTranslateTokenizer<'tokenizer> {
    sentence_piece_tokenizer: Option<SentencePieceTokenizer>,
    bpe_tokenizer: Option<BPETokenizer<'tokenizer>>
}

