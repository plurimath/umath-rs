//! C ABI: the additive legacy endpoint next to the unchanged strict one.
#![cfg(feature = "ffi")]
use serde_json::Value;
use std::ffi::{c_char, CStr};
use umath::ffi::*;

type Endpoint = unsafe extern "C" fn(*const u8, usize, *const u8, usize) -> *mut c_char;

fn call(f: Endpoint, input: &[u8], decimal: &[u8]) -> Value {
    unsafe {
        let ptr = f(input.as_ptr(), input.len(), decimal.as_ptr(), decimal.len());
        let value = serde_json::from_str(CStr::from_ptr(ptr).to_str().unwrap()).unwrap();
        umath_free_json_v1(ptr);
        value
    }
}

#[test]
fn legacy_endpoint_reports_caller_offsets_and_strict_keeps_grammar_offsets() {
    for (input, grammar, caller) in [("\u{3b1}&b", 8, 3), ("\u{27e8}a", 9, 4), ("\u{2592}", 1, 0)] {
        let legacy = call(umath_parse_model_legacy_json_v1, input.as_bytes(), b".");
        assert_eq!(legacy["ok"], false, "{input:?}");
        let error = &legacy["error"];
        assert_eq!(error["byte_offset"], caller, "{input:?}");
        assert_eq!(error["grammar_byte_offset"], grammar, "{input:?}");
        assert_eq!(error["offset_source"], "caller_input");
        let strict = call(umath_parse_model_json_v1, input.as_bytes(), b".");
        assert_eq!(strict["error"]["offset_source"], "preprocessed_input");
    }
}

#[test]
fn legacy_endpoint_success_matches_strict_model_and_contracts_hold() {
    let a = call(umath_parse_model_legacy_json_v1, "α+1".as_bytes(), b".");
    let b = call(umath_parse_model_json_v1, "α+1".as_bytes(), b".");
    assert_eq!(a["ok"], true);
    assert_eq!(a, b);
    for f in [
        umath_parse_model_legacy_json_v1 as Endpoint,
        umath_parse_model_json_v1,
    ] {
        assert_eq!(
            call(f, &[0xff], b".")["error"]["category"],
            "invalid_argument"
        );
        assert_eq!(call(f, b"a", b";")["error"]["category"], "invalid_argument");
        let empty = call(f, b"", b".");
        assert_eq!(empty["abi_version"], 1);
    }
    unsafe {
        let p = umath_parse_model_legacy_json_v1(std::ptr::null(), 1, b".".as_ptr(), 1);
        assert!(CStr::from_ptr(p)
            .to_str()
            .unwrap()
            .contains("invalid_argument"));
        umath_free_json_v1(p);
        umath_free_json_v1(std::ptr::null_mut());
    }
}
