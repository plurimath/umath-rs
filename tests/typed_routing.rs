//! Typed-model routing: `parse_model` keeps the public strict route and
//! `parse_model_legacy` follows the producer's legacy route, reporting
//! refusals in caller positions. Offsets come from pinned unicode
//! generation 5 rows (raw: grammar input; typed: caller input).
#![cfg(feature = "models")]
use umath::Parser;

/// Frozen generation-5 unicode-external-167.model: a present font model
/// serializes to `{}` and must survive reading and writing that portable JSON.
#[test]
fn empty_font_model_survives_portable_roundtrip() {
    use umath::generated::json_codecs::{
        read_Umath_Model_Document_json, write_Umath_Model_Document_json,
    };
    let row: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/empty-font-model.json")).unwrap();
    let parser = Parser::new().unwrap();
    let model = parser
        .parse_model(row["input"].as_str().unwrap(), ".")
        .unwrap();
    let once = umath::typed::to_portable_json(&model).unwrap();
    assert!(once.contains("\"mitBbb\":{}"), "{once}");
    let decoded = read_Umath_Model_Document_json(&once).unwrap();
    let twice = write_Umath_Model_Document_json(&decoded).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&twice).unwrap(),
        serde_json::from_str::<serde_json::Value>(&once).unwrap()
    );
}

// (input, raw grammar byte offset, typed caller byte offset)
const REFUSALS: &[(&str, usize, usize)] = &[
    ("\u{3b1}&b", 8, 3),
    ("\u{27e8}a", 9, 4),
    ("\u{2592}", 1, 0),
    ("\u{3b1}\u{2061}", 15, 5),
];

#[test]
fn legacy_model_follows_the_legacy_raw_route() {
    let parser = Parser::new().unwrap();
    for &(input, grammar, caller) in REFUSALS {
        let raw = parser.parse_legacy_with_decimal(input, ".").unwrap_err();
        assert_eq!(raw.byte_offset(), Some(grammar), "{input:?} raw");
        let typed = parser.parse_model_legacy(input, ".").unwrap_err();
        let umath::typed::ModelError::Parse(error) = &typed else {
            panic!("{input:?}: {typed}")
        };
        assert_eq!(error.byte_offset(), Some(grammar), "{input:?} grammar");
        assert_eq!(
            typed.caller_byte_offset(input),
            Some(caller),
            "{input:?} caller"
        );
    }
}

#[test]
fn public_model_keeps_the_strict_route() {
    let parser = Parser::new().unwrap();
    for &(input, _, _) in REFUSALS {
        let strict = parser
            .parse_with_decimal(input, ".")
            .map(|_| ())
            .map_err(|e| e.byte_offset());
        let typed = parser
            .parse_model(input, ".")
            .map(|_| ())
            .map_err(|e| match e {
                umath::typed::ModelError::Parse(error) => error.byte_offset(),
                other => panic!("{input:?}: {other}"),
            });
        assert_eq!(typed, strict, "{input:?}");
    }
}

#[test]
fn caller_offsets_skip_label_comment_and_escapes() {
    use umath::typed::ModelError;
    let parser = Parser::new().unwrap();
    // The same refusal behind leading spaces and before a label.
    let plain = parser.parse_model_legacy("\u{3b1}&b", ".").unwrap_err();
    let spaced = parser
        .parse_model_legacy("  \u{3b1}&b#(1)", ".")
        .unwrap_err();
    let (ModelError::Parse(a), ModelError::Parse(b)) = (&plain, &spaced) else {
        panic!("{plain} / {spaced}")
    };
    assert_eq!(a.byte_offset(), b.byte_offset());
    assert_eq!(spaced.caller_byte_offset("  \u{3b1}&b#(1)"), Some(5));
}

/// Ruby models keep `empty: ""` / `present: ""` markers in memory, while the
/// declared lutaml mapping (render_empty default false) omits empty values on
/// serialization. The typed structs must retain the marker; only the JSON wire
/// omits it, so a wire consumer cannot read it back.
#[test]
fn typed_structs_keep_empty_markers_that_the_wire_omits() {
    let parser = Parser::new().unwrap();
    let input = "\u{25ad}(4&\u{270e}(#e01f32&\u{3b1}))";
    let model = parser.parse_model_legacy(input, ".").unwrap();
    let debug = format!("{model:?}");
    assert!(debug.contains("empty: Value(\"\")"), "{debug}");
    let wire = umath::typed::to_portable_json(&model).unwrap();
    assert!(!wire.contains("\"empty\""), "{wire}");
}

/// Legacy refusals report the Deepest position the Ruby producer reports.
#[test]
fn broken_bar_refusals_report_deepest_offsets() {
    let parser = Parser::new().unwrap();
    for (input, grammar) in [("\u{a6}", 1), ("a\u{a6}", 7), ("\u{3b1}\u{a6}", 13)] {
        let raw = parser.parse_legacy_with_decimal(input, ".").unwrap_err();
        assert_eq!(raw.byte_offset(), Some(grammar), "{input:?} raw");
        let typed = parser.parse_model_legacy(input, ".").unwrap_err();
        let umath::typed::ModelError::Parse(error) = &typed else {
            panic!("{input:?}: {typed}")
        };
        assert_eq!(error.byte_offset(), Some(grammar), "{input:?} typed");
    }
}
