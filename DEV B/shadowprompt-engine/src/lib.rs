#![allow(non_snake_case)]

use jni::objects::{JClass, JString};
use jni::sys::{jstring, JNI_VERSION_1_6};
use jni::JNIEnv;
use serde::Serialize;

#[no_mangle]
pub extern "system" fn JNI_OnLoad(
    _vm: *mut jni::sys::JavaVM,
    _reserved: *mut std::ffi::c_void,
) -> jni::sys::jint {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Debug),
    );
    JNI_VERSION_1_6
}

unsafe fn jstring_from(env: &mut JNIEnv, s: String) -> jstring {
    env.new_string(s).expect("jstring").into_raw()
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_shadowprompt_engine_NativeBridge_maskText(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
) -> jstring {
    let input_str: String = env.get_string(&input).expect("utf8").into();
    jstring_from(&mut env, engine_runtime::mask_text(input_str))
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_shadowprompt_engine_NativeBridge_unmaskReply(
    mut env: JNIEnv,
    _class: JClass,
    reply: JString,
) -> jstring {
    let reply_str: String = env.get_string(&reply).expect("utf8").into();
    jstring_from(&mut env, engine_runtime::unmask_reply(reply_str))
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_shadowprompt_engine_NativeBridge_unmaskReplyJson(
    mut env: JNIEnv,
    _class: JClass,
    reply: JString,
) -> jstring {
    let reply_str: String = env.get_string(&reply).expect("utf8").into();
    jstring_from(&mut env, engine_runtime::unmask_reply_json(reply_str))
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_shadowprompt_engine_NativeBridge_wipeStm(
    _env: JNIEnv,
    _class: JClass,
) {
    engine_runtime::wipe_stm();
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_shadowprompt_engine_NativeBridge_maskWithRulesJson(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
    rules_json: JString,
) -> jstring {
    let input_str: String = env.get_string(&input).expect("utf8").into();
    let rules: String = env.get_string(&rules_json).expect("utf8").into();
    jstring_from(&mut env, engine_runtime::mask_with_rules_json(input_str, rules))
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_shadowprompt_engine_NativeBridge_maskWithKeywordsJson(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
    keywords_json: JString,
) -> jstring {
    let input_str: String = env.get_string(&input).expect("utf8").into();
    let kw: String = env.get_string(&keywords_json).expect("utf8").into();
    jstring_from(&mut env, engine_runtime::mask_with_keywords_json(input_str, kw))
}

#[derive(Serialize)]
struct ProfiledOut {
    text: String,
    mask_ms: u128,
    input_len: usize,
    output_len: usize,
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_shadowprompt_engine_NativeBridge_maskWithNerJson(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
    entities_json: JString,
    keywords_json: JString,
    use_mock_ner: jni::sys::jboolean,
) -> jstring {
    let input_str: String = env.get_string(&input).expect("utf8").into();
    let entities: String = env.get_string(&entities_json).expect("utf8").into();
    let keywords: String = env.get_string(&keywords_json).expect("utf8").into();
    let mock = use_mock_ner != jni::sys::JNI_FALSE;
    jstring_from(
        &mut env,
        engine_runtime::mask_with_ner_json(input_str, entities, keywords, mock),
    )
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_shadowprompt_engine_NativeBridge_maskTextProfiledJson(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
) -> jstring {
    let input_str: String = env.get_string(&input).expect("utf8").into();
    let (text, sample) = engine_runtime::mask_text_profiled(input_str);
    let body = ProfiledOut {
        text,
        mask_ms: sample.mask_ms,
        input_len: sample.input_len,
        output_len: sample.output_len,
    };
    let json = serde_json::to_string(&body).unwrap_or_else(|_| "{}".into());
    jstring_from(&mut env, json)
}
