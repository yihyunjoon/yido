use std::{
    ffi::{CStr, CString, c_char},
    panic::{AssertUnwindSafe, catch_unwind},
    ptr,
};

use yido_core::{Composer, InputEffect, Layout};

#[repr(C)]
pub struct YidoEngine {
    _private: [u8; 0],
}

struct Engine {
    composer: Composer,
}

#[repr(C)]
pub struct YidoEngineCreateResult {
    pub engine: *mut YidoEngine,
    pub error: *mut c_char,
}

#[repr(C)]
pub struct YidoInputEffect {
    pub committed: *mut c_char,
    pub composing: *mut c_char,
    pub handled: bool,
    pub error: *mut c_char,
}

#[unsafe(no_mangle)]
pub extern "C" fn yido_engine_new(layout_toml: *const c_char) -> YidoEngineCreateResult {
    match catch_unwind(AssertUnwindSafe(|| create_engine(layout_toml))) {
        Ok(result) => result,
        Err(_) => create_error("Rust panic while creating yido engine"),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn yido_engine_input_key(
    engine: *mut YidoEngine,
    key: *const c_char,
    shift: bool,
) -> YidoInputEffect {
    match catch_unwind(AssertUnwindSafe(|| {
        let key = read_c_string(key)?;
        with_engine(engine, |composer| composer.input_key(key, shift))
    })) {
        Ok(result) => result.unwrap_or_else(error_effect),
        Err(_) => error_effect("Rust panic while handling key".to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn yido_engine_backspace(engine: *mut YidoEngine) -> YidoInputEffect {
    match catch_unwind(AssertUnwindSafe(|| {
        with_engine(engine, |composer| composer.backspace())
    })) {
        Ok(result) => result.unwrap_or_else(error_effect),
        Err(_) => error_effect("Rust panic while handling backspace".to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn yido_engine_flush(engine: *mut YidoEngine) -> YidoInputEffect {
    match catch_unwind(AssertUnwindSafe(|| with_engine(engine, |composer| composer.flush()))) {
        Ok(result) => result.unwrap_or_else(error_effect),
        Err(_) => error_effect("Rust panic while flushing preedit".to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn yido_engine_cancel(engine: *mut YidoEngine) -> YidoInputEffect {
    match catch_unwind(AssertUnwindSafe(|| with_engine(engine, |composer| composer.cancel()))) {
        Ok(result) => result.unwrap_or_else(error_effect),
        Err(_) => error_effect("Rust panic while cancelling preedit".to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn yido_engine_create_result_free(result: YidoEngineCreateResult) {
    free_c_string(result.error);
}

#[unsafe(no_mangle)]
pub extern "C" fn yido_input_effect_free(effect: YidoInputEffect) {
    free_c_string(effect.committed);
    free_c_string(effect.composing);
    free_c_string(effect.error);
}

#[unsafe(no_mangle)]
pub extern "C" fn yido_engine_free(engine: *mut YidoEngine) {
    if engine.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(engine as *mut Engine));
    }
}

fn create_engine(layout_toml: *const c_char) -> YidoEngineCreateResult {
    let layout_toml = match read_c_string(layout_toml) {
        Ok(layout_toml) => layout_toml,
        Err(error) => return create_error(error),
    };

    match Layout::from_toml(layout_toml) {
        Ok(layout) => YidoEngineCreateResult {
            engine: Box::into_raw(Box::new(Engine {
                composer: Composer::new(layout),
            })) as *mut YidoEngine,
            error: ptr::null_mut(),
        },
        Err(error) => create_error(error.to_string()),
    }
}

fn with_engine(
    engine: *mut YidoEngine,
    action: impl FnOnce(&mut Composer) -> InputEffect,
) -> Result<YidoInputEffect, String> {
    if engine.is_null() {
        return Err("YidoEngine pointer is null".to_string());
    }

    let engine = unsafe { &mut *(engine as *mut Engine) };
    Ok(effect_to_ffi(action(&mut engine.composer)))
}

fn effect_to_ffi(effect: InputEffect) -> YidoInputEffect {
    YidoInputEffect {
        committed: into_c_string(effect.committed),
        composing: into_c_string(effect.composing),
        handled: effect.handled,
        error: ptr::null_mut(),
    }
}

fn error_effect(error: String) -> YidoInputEffect {
    YidoInputEffect {
        committed: into_c_string(String::new()),
        composing: into_c_string(String::new()),
        handled: false,
        error: into_c_string(error),
    }
}

fn create_error(error: impl Into<String>) -> YidoEngineCreateResult {
    YidoEngineCreateResult {
        engine: ptr::null_mut(),
        error: into_c_string(error.into()),
    }
}

fn read_c_string<'a>(value: *const c_char) -> Result<&'a str, String> {
    if value.is_null() {
        return Err("C string pointer is null".to_string());
    }

    unsafe { CStr::from_ptr(value) }
        .to_str()
        .map_err(|error| format!("C string is not valid UTF-8: {error}"))
}

fn into_c_string(value: String) -> *mut c_char {
    let value = value.replace('\0', "\\0");
    CString::new(value)
        .expect("null bytes were escaped before creating CString")
        .into_raw()
}

fn free_c_string(value: *mut c_char) {
    if value.is_null() {
        return;
    }

    unsafe {
        drop(CString::from_raw(value));
    }
}
