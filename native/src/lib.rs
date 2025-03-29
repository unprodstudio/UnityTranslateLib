mod bpe;

use std::cmp::max;
use std::fs;
use ct2rs::{Config, Device, Tokenizer, TranslationOptions, Translator};
use jni::JNIEnv;
use jni::objects::{JClass, JObjectArray, JString};
use jni::sys::{jboolean, jlong, jobjectArray, jsize};
use rust_tokenizers::tokenizer::{SentencePieceTokenizer, Tokenizer as RTTokenizer};
use tokenizers::Model;
use crate::bpe::BPETokenizer;

// I know I'm not good at Rust.
// But at least it's functional.... I think.

struct UnityTranslateTokenizer {
    sentence_piece_tokenizer: Option<SentencePieceTokenizer>,
    bpe_tokenizer: Option<BPETokenizer>
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

#[no_mangle]
pub extern "system" fn Java_xyz_bluspring_unitytranslate_library_UnityTranslateLib_loadModel<'local>(
    mut env: JNIEnv<'local>, class: JClass<'local>,
    modelPath: JString<'local>, spModelPath: JString<'local>, bpeModelPath: JString<'local>,
    useCuda: jboolean,
) -> jlong {
    let modelPathValue: String = String::from(env.get_string(&modelPath).expect("Couldn't get java string value"));

    let device = if useCuda != 0 {
        Device::CUDA
    } else {
        Device::CPU
    };

    let tokenizer: UnityTranslateTokenizer = if !spModelPath.is_null() {
        let spModelValue: String = String::from(env.get_string(&spModelPath).expect("Couldn't get java string value"));
        let tokenizer = SentencePieceTokenizer::from_file(spModelValue, false)
            .expect("Couldn't load SentencePiece model path!");

        UnityTranslateTokenizer { sentence_piece_tokenizer: Some(tokenizer), bpe_tokenizer: None }
    } else if !bpeModelPath.is_null() {
        let bpeModelValue: String = String::from(env.get_string(&bpeModelPath).unwrap());
        let bpeModelData = fs::read_to_string(bpeModelValue).expect("Couldn't read BPE model file!");
        let tokenizer = BPETokenizer::new(bpeModelData.as_str());

        UnityTranslateTokenizer { sentence_piece_tokenizer: None, bpe_tokenizer: Some(tokenizer) }
    } else {
        panic!("No tokenizer path was provided!");
    };

    let mut config = Config::default();
    config.device = device;
    
    let translatorResult = Translator::with_tokenizer(modelPathValue, tokenizer, &config);
    
    if translatorResult.is_ok() {
        Box::into_raw(Box::new(translatorResult.unwrap())) as jlong
    } else if translatorResult.is_err() {
        let err = translatorResult.unwrap_err();
        println!("Error: {:?}", err);
        panic!("An error occurred!")
    } else {
        panic!("How the fuck?")
    }
}

#[no_mangle]
pub extern "system" fn Java_xyz_bluspring_unitytranslate_library_UnityTranslateLib_batchTranslate<'local>(
    mut env: JNIEnv<'local>, class: JClass<'local>,
    modelPtr: jlong, textArray: JObjectArray<'local>
) -> JObjectArray<'local> {
    let translator: &mut Translator<UnityTranslateTokenizer>;

    unsafe {
        translator = Box::leak(Box::from_raw(modelPtr as *mut Translator<UnityTranslateTokenizer>));
    }

    // Recreate Argos Translate's behaviour
    let numHypotheses = 4;

    let mut options = TranslationOptions::default();
    options.replace_unknowns = true;
    options.beam_size = max(numHypotheses, 4);
    options.num_hypotheses = numHypotheses;
    options.length_penalty = 0.2;
    options.return_scores = true;

    let stringLength = env.get_array_length(&textArray).expect("Couldn't get java length!") as usize;
    let mut texts: Vec<String> = Vec::with_capacity(stringLength);

    for i in 0..stringLength {
        let token: JString<'local> = env.get_object_array_element(&textArray, i as jsize).expect(format!("Couldn't get element at pos {i}!").as_str()).into();
        let tokenValue = env.get_string(&token).expect("Couldn't get java string value");
        let str = String::from(tokenValue);
        texts.push(str);
    }

    let translationResult = Translator::translate_batch(translator, &texts, &options, None);

    if translationResult.is_err() {
        panic!("Failed to translate batch!")
    }

    let resultsVec=  translationResult.unwrap();
    let stringClass = env.find_class("java/lang/String").unwrap();
    let objectArray = env.new_object_array(resultsVec.len() as jsize, stringClass, JString::default()).unwrap();

    let mut i: jsize = 0;
    for (result, score) in resultsVec {
        let resultStr: JString = env.new_string(result).expect("Couldn't create java string!");
        env.set_object_array_element(&objectArray, i, resultStr)
            .expect(format!("Couldn't set string at pos {i}!").as_str());
        
        i += 1;
    }

    objectArray
}