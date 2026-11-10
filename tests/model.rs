#![cfg(feature = "models")]
use umath::{
    generated::json_codecs::read_Umath_Model_Document_json, typed::to_portable_json, Parser,
};
#[test]
fn generated_model_symbol_label_and_decimal_contract() {
    let parser = Parser::new().unwrap();
    for (input, decimal) in [
        ("α+1", "."),
        ("(a)/(b)", "."),
        ("√x", "."),
        ("1,5", ","),
        ("", "."),
        ("a#eq1", "."),
        ("##", "."),
    ] {
        let model = parser.parse_model(input, decimal).unwrap();
        let encoded = to_portable_json(&model).unwrap();
        let decoded = read_Umath_Model_Document_json(&encoded).unwrap();
        decoded.validate().unwrap();
        assert_eq!(model.decimal, decoded.decimal);
        let json: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(json["decimal"], decimal);
        if input.is_empty() || input == "##" {
            assert!(json.get("expression").is_none());
        }
        if input == "α+1" {
            assert!(encoded.contains('α'));
        }
        if input == "a#eq1" {
            assert_eq!(json["label"], "eq1");
        }
    }
    assert!(parser.parse_model("x", "bad").is_err());
}
