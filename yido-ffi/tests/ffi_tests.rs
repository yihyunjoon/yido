use std::ffi::{CStr, CString};

use yido_ffi::{
    yido_engine_backspace, yido_engine_cancel, yido_engine_create_result_free, yido_engine_flush,
    yido_engine_free, yido_engine_input_key, yido_engine_new, yido_input_effect_free,
};

const DUBEOLSIK: &str = include_str!("../../layouts/ko-dubeolsik.toml");

#[test]
fn creates_engine_and_returns_effects() {
    let layout = CString::new(DUBEOLSIK).unwrap();
    let result = yido_engine_new(layout.as_ptr());
    assert!(result.error.is_null());
    assert!(!result.engine.is_null());
    let engine = result.engine;
    yido_engine_create_result_free(result);

    let g = CString::new("g").unwrap();
    let effect = yido_engine_input_key(engine, g.as_ptr(), false);
    assert!(effect.error.is_null());
    assert_eq!(unsafe { CStr::from_ptr(effect.composing) }.to_str().unwrap(), "ㅎ");
    assert_eq!(unsafe { CStr::from_ptr(effect.committed) }.to_str().unwrap(), "");
    assert!(effect.handled);
    yido_input_effect_free(effect);

    yido_engine_free(engine);
}

#[test]
fn flushes_and_cancels_preedit() {
    let engine = create_engine();
    input(engine, "g");
    input(engine, "k");
    input(engine, "s");

    let flushed = yido_engine_flush(engine);
    assert!(flushed.error.is_null());
    assert_eq!(unsafe { CStr::from_ptr(flushed.committed) }.to_str().unwrap(), "한");
    assert_eq!(unsafe { CStr::from_ptr(flushed.composing) }.to_str().unwrap(), "");
    assert!(flushed.handled);
    yido_input_effect_free(flushed);

    input(engine, "g");
    let cancelled = yido_engine_cancel(engine);
    assert!(cancelled.error.is_null());
    assert_eq!(unsafe { CStr::from_ptr(cancelled.committed) }.to_str().unwrap(), "");
    assert_eq!(unsafe { CStr::from_ptr(cancelled.composing) }.to_str().unwrap(), "");
    assert!(cancelled.handled);
    yido_input_effect_free(cancelled);

    yido_engine_free(engine);
}

#[test]
fn unmapped_key_returns_committed_delta_without_handling_key() {
    let engine = create_engine();
    input(engine, "g");
    input(engine, "k");
    input(engine, "s");

    let key = CString::new("1").unwrap();
    let effect = yido_engine_input_key(engine, key.as_ptr(), false);

    assert!(effect.error.is_null());
    assert_eq!(unsafe { CStr::from_ptr(effect.committed) }.to_str().unwrap(), "한");
    assert_eq!(unsafe { CStr::from_ptr(effect.composing) }.to_str().unwrap(), "");
    assert!(!effect.handled);
    yido_input_effect_free(effect);

    yido_engine_free(engine);
}

#[test]
fn backspace_without_preedit_is_not_handled() {
    let engine = create_engine();

    let effect = yido_engine_backspace(engine);

    assert!(effect.error.is_null());
    assert_eq!(unsafe { CStr::from_ptr(effect.committed) }.to_str().unwrap(), "");
    assert_eq!(unsafe { CStr::from_ptr(effect.composing) }.to_str().unwrap(), "");
    assert!(!effect.handled);
    yido_input_effect_free(effect);

    yido_engine_free(engine);
}

#[test]
fn invalid_layout_returns_create_error() {
    let layout = CString::new("not toml").unwrap();
    let result = yido_engine_new(layout.as_ptr());

    assert!(result.engine.is_null());
    assert!(!result.error.is_null());
    let error = unsafe { CStr::from_ptr(result.error) }.to_str().unwrap();
    assert!(error.contains("배열 TOML을 파싱하지 못했습니다"));

    yido_engine_create_result_free(result);
}

fn create_engine() -> *mut yido_ffi::YidoEngine {
    let layout = CString::new(DUBEOLSIK).unwrap();
    let result = yido_engine_new(layout.as_ptr());
    assert!(result.error.is_null());
    assert!(!result.engine.is_null());
    let engine = result.engine;
    yido_engine_create_result_free(result);
    engine
}

fn input(engine: *mut yido_ffi::YidoEngine, key: &str) {
    let key = CString::new(key).unwrap();
    let effect = yido_engine_input_key(engine, key.as_ptr(), false);
    assert!(effect.error.is_null());
    yido_input_effect_free(effect);
}
