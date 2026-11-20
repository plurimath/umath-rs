//! Native byte reference for the focused WASM smoke inputs; run by WASM CI only.
use serde_json::{json, Value};
use umath::{generated::json_codecs, typed, Parser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Value = serde_json::from_str(include_str!("../tests/fixtures/wasm-smoke.json"))?;
    let parser = Parser::new()?;
    let mut rows = Vec::new();
    for case in cases.as_array().unwrap() {
        let input = case["input"].as_str().unwrap();
        let decimal = case["decimal"].as_str().unwrap();
        // Empty input has a valid model, but the raw Umath API has no formula.
        // Record only the fixture's declared refusal; other errors still fail CI.
        let raw = match parser.parse_with_decimal(input, decimal) {
            Ok(parsed) => {
                assert!(
                    case.get("raw_error").is_none(),
                    "expected raw refusal: {input:?}"
                );
                Some(
                    json!({"preprocessed": parsed.parsed_input, "tree": parsed.canonical()})
                        .to_string(),
                )
            }
            Err(umath::Error::NoFormula) if case["raw_error"] == "input has no formula" => None,
            Err(error) => return Err(error.into()),
        };
        let model = typed::to_portable_json(&parser.parse_model(input, decimal)?)?;
        let decoded = json_codecs::read_Umath_Model_Document_json(&model)?;
        let serialized = typed::to_portable_json(&decoded)?;
        rows.push(
            json!({"input": input, "decimal": decimal, "raw": raw, "raw_error": case["raw_error"],
            "model": model, "serialized": serialized}),
        );
    }
    println!("{}", serde_json::to_string(&rows)?);
    Ok(())
}
