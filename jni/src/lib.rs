mod bpe;

use crate::bpe::BPETokenizer;
use ct2rs::{Config, Device, Tokenizer, TranslationOptions, Translator};
use jni::objects::{JClass, JObjectArray, JString};
use jni::sys::{jboolean, jint, jlong};
use jni::{Env, EnvUnowned};
use rust_tokenizers::tokenizer::{SentencePieceTokenizer, Tokenizer as RTTokenizer};
use std::cmp::max;
use std::fs;

// I know I'm not good at Rust.
// But at least it's functional.... I think.
// all this is old code lmao, this all comes from shit I actually wrote Jan 2025!
// I just ended up coming back here now that I've actually hopefully fixed the BPE problem!

struct UnityTranslateTokenizer {
    sentence_piece_tokenizer: Option<SentencePieceTokenizer>,
    bpe_tokenizer: Option<BPETokenizer>
}

enum TokenizerType {
    SentencePiece, Bpe
}

impl TokenizerType {
    fn from_jint(value: jint) -> TokenizerType {
        match value {
            0 => TokenizerType::SentencePiece,
            1 => TokenizerType::Bpe,
            _ => panic!("Unknown TokenizerType value: {}", value)
        }
    }
}

impl Tokenizer for UnityTranslateTokenizer {
    fn encode(&self, input: &str) -> anyhow::Result<Vec<String>> {
        if let Some(sp) = &self.sentence_piece_tokenizer {
            let result = sp.tokenize(input);
            Ok(result)
        } else if let Some(bpe) = &self.bpe_tokenizer {
            let result = bpe.encode(input);

            if let Ok(result) = result {
                let segmented = bpe.segment_tokens(result);
                Ok(segmented)
            } else {
                Err(result.err().unwrap())
            }
        } else {
            Err(anyhow::anyhow!("UnityTranslateTokenizer"))
        }
    }

    fn decode(&self, tokens: Vec<String>) -> anyhow::Result<String> {
        if let Some(sp) = &self.sentence_piece_tokenizer {
            let result = sp.convert_tokens_to_string(tokens);
            Ok(result)
        } else if let Some(bpe) = &self.bpe_tokenizer {
            let result = bpe.decode(tokens);
            result
        } else {
            Err(anyhow::anyhow!("UnityTranslateTokenizer"))
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_xyz_bluspring_unitytranslate_library_UnityTranslateLib_createInstance<'local>(
    mut unowned_env: EnvUnowned<'local>, class: JClass<'local>,
    to_lang: JString<'local>, translator_model_path: JString<'local>,
    tokenizer_type: jint, tokenizer_model_path: JString<'local>,
    use_cuda: jboolean
) -> jlong {
    let outcome = unowned_env.with_env(|env| -> jni::errors::Result<jlong> {
        let device = if use_cuda {
            Device::CUDA
        } else {
            Device::CPU
        };

        let tokenizer_type_value = TokenizerType::from_jint(tokenizer_type);
        let tokenizer_model_value = tokenizer_model_path.try_to_string(env).expect("Failed to read tokenizer model path string!");

        let tokenizer: UnityTranslateTokenizer = match (tokenizer_type_value) {
            TokenizerType::SentencePiece => {
                let tokenizer = SentencePieceTokenizer::from_file(tokenizer_model_value, false)
                    .expect("Couldn't load SentencePiece model path!");

                UnityTranslateTokenizer { sentence_piece_tokenizer: Some(tokenizer), bpe_tokenizer: None }
            }

            TokenizerType::Bpe => {
                let bpe_model_data_opt = fs::read_to_string(tokenizer_model_value.clone());
                let bpe_model_data = bpe_model_data_opt.unwrap_or_else(|e| panic!("Couldn't read BPE model file! {tokenizer_model_value} {e}"));
                let tokenizer = BPETokenizer::new(bpe_model_data.as_str());

                UnityTranslateTokenizer { sentence_piece_tokenizer: None, bpe_tokenizer: Some(tokenizer) }
            }
        };

        let mut config = Config::default();
        config.device = device;

        let model_path_value: String = translator_model_path.try_to_string(env).expect("Failed to read tokenizer model path string!");
        let translator_result = Translator::with_tokenizer(model_path_value, tokenizer, &config);

        if let Ok(translator) = translator_result {
            Ok(Box::into_raw(Box::new(translator)) as jlong)
        } else if let Err(err) = translator_result {
            println!("Error: {:?}", err);
            panic!("An error occurred!")
        } else {
            panic!("How the fuck?")
        }
    });

    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_xyz_bluspring_unitytranslate_library_UnityTranslateLib_batchTranslate<'local>(
    mut unowned_env: EnvUnowned<'local>, class: JClass<'local>,
    instance_ptr: jlong, text_to_translate: JObjectArray<'local>, results: JObjectArray<'local>
) {
    let outcome = unowned_env.with_env(|env| -> jni::errors::Result<_> {
        let translator: &mut Translator<UnityTranslateTokenizer>;
        unsafe {
            translator = Box::leak(Box::from_raw(instance_ptr as *mut Translator<UnityTranslateTokenizer>));
        }
        // Recreate Argos Translate's behaviour
        let num_hypotheses = 4;
        let mut options = TranslationOptions::default();
        options.replace_unknowns = true;
        options.beam_size = max(num_hypotheses, 4);
        options.num_hypotheses = num_hypotheses;
        options.length_penalty = 0.2;
        options.return_scores = true;

        let string_length = text_to_translate.len(env).expect("Couldn't get java length!");
        let mut texts: Vec<String> = Vec::with_capacity(string_length);
        for i in 0..string_length {
            let token_obj = text_to_translate.get_element(env, i).unwrap_or_else(|_| panic!("Couldn't get element at pos {i}!"));
            let token: JString<'local> = JString::cast_local(env, token_obj).unwrap_or_else(|_| panic!("Couldn't cast element at pos {i} to JString!"));
            let token_value = JString::to_string(&token);
            texts.push(token_value);
        }

        let translation_result = Translator::translate_batch(translator, &texts, &options, None);
        if translation_result.is_err() {
            panic!("Failed to translate batch!")
        }

        let results_vec =  translation_result.unwrap();

        for (i, (result, _score)) in results_vec.into_iter().enumerate() {
            let result_str: JString = env.new_string(result.trim()).expect("Couldn't create java string!");
            results.set_element(env, i, result_str)
                .unwrap_or_else(|_| panic!("Couldn't set string at pos {i}!"));
        }

        Ok(())
    });

    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>();
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_xyz_bluspring_unitytranslate_library_UnityTranslateLib_freeInstance<'local>(
    mut _env: Env<'local>, _class: JClass<'local>,
    _instance_ptr: jlong
) {
    // this might not actually be needed.....
}
