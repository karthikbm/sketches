#![feature(box_syntax)]

extern crate uuid;
extern crate wasm_bindgen;

use std::fmt::Debug;
use std::mem;
use std::panic::{self, PanicInfo};

use uuid::Uuid;
use wasm_bindgen::prelude::*;

pub mod color;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Math)]
    pub fn random() -> f64;
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn js_log(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    fn js_warn(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn js_error(s: &str);
}

pub fn debug<T: Debug>(x: T) -> String {
    format!("{:?}", x)
}

pub fn log<T: AsRef<str>>(msg: T) {
    js_log(msg.as_ref())
}

pub fn warn<T: AsRef<str>>(msg: T) {
    js_warn(msg.as_ref())
}

pub fn error<T: AsRef<str>>(msg: T) {
    js_error(msg.as_ref())
}

pub fn math_random() -> f64 {
    random()
}

/// Simulates a random UUID, but uses the rand crate with WebAssembly support.
pub fn v4_uuid() -> Uuid {
    // Because I really don't care, honestly.
    let high_quality_entropy: (f64, f64) = (math_random(), math_random());
    unsafe { mem::transmute(high_quality_entropy) }
}

pub fn set_panic_hook() {
    panic::set_hook(box |info: &PanicInfo| error(info.to_string()));
}
