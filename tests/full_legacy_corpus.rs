use serde_json::Value;
use umath::Parser;
#[test]
#[ignore = "complete frozen corpus runs in platform CI"]
fn full_frozen_raw_corpus() {
    let parser = Parser::new().unwrap();
    let corpus: Value = serde_json::from_str(include_str!("fixtures/full-raw.json")).unwrap();
    for row in corpus["cases"].as_array().unwrap() {
        let input = row["input"]["str"].as_str().unwrap();
        let result = parser
            .parse_legacy_with_decimal(input, row["options"]["decimal"].as_str().unwrap_or("."));
        if let Some(refusal) = row["expected"].get("refusal") {
            let error = result.unwrap_err();
            assert_eq!(
                error.byte_offset(),
                refusal["byte_offset"].as_u64().map(|v| v as usize),
                "{}",
                row["id"]
            );
        } else {
            let parsed = result.unwrap_or_else(|e| panic!("{}: {}", row["id"], e));
            assert_eq!(
                parsed.parsed_input,
                row["expected"]["preprocessed"]["str"].as_str().unwrap(),
                "{} preprocessing",
                row["id"]
            );
            assert_eq!(
                parsed.canonical(),
                row["expected"]["tree"],
                "{} tree/order/offsets",
                row["id"]
            );
        }
    }
}
