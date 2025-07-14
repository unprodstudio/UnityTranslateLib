use std::cell::Cell;
use std::sync::Mutex;
use ct2rs::Tokenizer;
use jni::descriptors::Desc;
use jni::JNIEnv;
use jni::objects::{JClass, JMethodID, JObject, JObjectArray, JString, JValue};
use jni::sys::jsize;

pub struct BPETokenizer<'tokenizer> {
    //decoder: tokenizers::decoders::bpe::BPEDecoder,
    pub(crate) tokenizer: JObject<'tokenizer>,
    env: Mutex<JNIEnv<'tokenizer>>
}

impl<'tokenizer> BPETokenizer<'tokenizer> {
    pub fn new(mut env: JNIEnv<'tokenizer>, to_lang: &str, codes: &str) -> BPETokenizer<'tokenizer> {
        let kt_tokenizer_class = env.find_class("xyz/bluspring/unitytranslate/library/util/BPETokenizer").unwrap();

        let to_lang_java = env.new_string(to_lang).expect("Couldn't create java string!");
        let codes_java = env.new_string(codes).expect("Couldn't create java string!");

        let kt_tokenizer = env.new_object(&kt_tokenizer_class, "(Ljava/lang/String;Ljava/lang/String;)V", &[JValue::Object(to_lang_java.as_ref()), JValue::Object(codes_java.as_ref())]).expect("Couldn't create tokenizer!");
        let env_cell = Mutex::new(env);

        BPETokenizer {
            tokenizer: kt_tokenizer,
            env: env_cell
        }
    }

    pub fn segment_tokens(&self, result: Vec<String>) -> Vec<String> {
        let tokenizer = &self.tokenizer;
        let env_cell = &self.env;
        let mut env = env_cell.lock().unwrap();

        let string_class = env.find_class("java/lang/String").unwrap();
        let input_java = env.new_object_array(result.len() as jsize, string_class, JObject::null()).unwrap();

        let mut i: jsize = 0;
        for token in result {
            let result_str: JString = env.new_string(token).expect("Couldn't create java string!");
            env.set_object_array_element(&input_java, i as jsize, result_str)
                .expect(format!("Couldn't set string at pos {i}!").as_str());

            i += 1;
        }

        let result = env.call_method(tokenizer, "segmentTokens", "([Ljava/lang/String;)[Ljava/lang/String;", &[JValue::Object(input_java.as_ref())]).expect("Failed to call decode on tokenizer!");
        let obj = result.l().expect("Result is not an array!");
        let array = JObjectArray::from(obj);
        let len = env.get_array_length(&array).expect("Failed to get array length!");

        let mut vec: Vec<String> = Vec::with_capacity(len as usize);

        for i in 0..len {
            let item: JString = env.get_object_array_element(&array, i).expect("Failed to get array element!").into();
            let str_java = env.get_string(item.as_ref()).expect("Failed to get java string!");
            let str = String::from(str_java);

            vec.push(str);
        }

        vec
    }
}

impl<'tokenizer> Tokenizer for BPETokenizer<'tokenizer> {
    fn encode(&self, input: &str) -> anyhow::Result<Vec<String>> {
        // i hate Rust
        let tokenizer = &self.tokenizer;
        let env_cell = &self.env;
        let mut env = env_cell.lock().unwrap();
        let input_java = env.new_string(input).expect("Couldn't create java string!");
        let result = env.call_method(tokenizer, "encode", "(Ljava/lang/String;)[Ljava/lang/String;", &[JValue::Object(input_java.as_ref())]).expect("Failed to call encode on tokenizer!");

        let obj = result.l().expect("Result is not an array!");
        let array = JObjectArray::from(obj);
        let len = env.get_array_length(&array).expect("Failed to get array length!");

        let mut vec: Vec<String> = Vec::with_capacity(len as usize);

        for i in 0..len {
            let item: JString = env.get_object_array_element(&array, i).expect("Failed to get array element!").into();
            let str_java = env.get_string(item.as_ref()).expect("Failed to get java string!");
            let str = String::from(str_java);

            vec.push(str);
        }

        Ok(vec)
    }

    fn decode(&self, tokens: Vec<String>) -> anyhow::Result<String> {
        let tokenizer = &self.tokenizer;
        let env_cell = &self.env;
        let mut env = env_cell.lock().unwrap();

        let string_class = env.find_class("java/lang/String").unwrap();
        let input_java = env.new_object_array(tokens.len() as jsize, string_class, JObject::null()).unwrap();

        let mut i: jsize = 0;
        for token in tokens {
            let result_str: JString = env.new_string(token).expect("Couldn't create java string!");
            env.set_object_array_element(&input_java, i as jsize, result_str)
                .expect(format!("Couldn't set string at pos {i}!").as_str());

            i += 1;
        }

        let result = env.call_method(tokenizer, "decode", "([Ljava/lang/String;)Ljava/lang/String;", &[JValue::Object(input_java.as_ref())]).expect("Failed to call decode on tokenizer!");
        let obj: JString = result.l().expect("Result is not a string!").into();
        let str_java = env.get_string(&obj).expect("Failed to get java string!");
        let str = String::from(str_java);

        Ok(str)
    }
}