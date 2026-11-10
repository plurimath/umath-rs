//! Stage-specific adapter into upstream generated Lutaml models and codecs.
use crate::{
    generated::{json_codecs, models::Umath_Model_Document},
    Error, Parser,
};
use serde_json::{json, Value};
use std::{collections::HashMap, fmt};
#[derive(Debug)]
pub enum ModelError {
    Parse(Error),
    Codec(Box<lutaml_rs::contracts::ContractError>),
}
impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "{e}"),
            Self::Codec(e) => write!(f, "{e:?}"),
        }
    }
}
impl std::error::Error for ModelError {}
fn strings(value: &Value, source: &str, catalog: &HashMap<String, Value>) -> Value {
    match value {
        Value::Array(a) => Value::Array(a.iter().map(|v| strings(v, source, catalog)).collect()),
        Value::Object(map) if crate::replay::is_input_ref(map) => {
            if let Some(text) = map.get("value").and_then(Value::as_str) {
                return Value::String(text.into());
            }
            let offset = map["offset"].as_u64().unwrap() as usize;
            let length = map["length"].as_u64().unwrap() as usize;
            Value::String(source[offset..].chars().take(length).collect())
        }
        Value::Object(map) => {
            let mut object = map
                .iter()
                .map(|(k, v)| (k.clone(), strings(v, source, catalog)))
                .collect::<serde_json::Map<String, Value>>();
            if let Some(spelling) = object
                .get("spelling")
                .and_then(Value::as_str)
                .map(str::to_string)
            {
                let stripped = spelling.trim_matches(|c| {
                    matches!(c, '\0' | '\t' | '\n' | '\x0b' | '\x0c' | '\r' | ' ')
                });
                let found = catalog.get(stripped);
                object.insert(
                    "value".into(),
                    found
                        .and_then(|v| v.get("value"))
                        .cloned()
                        .unwrap_or_else(|| Value::String(decode_numeric(&spelling))),
                );
                object.insert(
                    "kind".into(),
                    found
                        .and_then(|v| v.get("kind"))
                        .cloned()
                        .unwrap_or_else(|| Value::String("unlisted".into())),
                );
            }
            Value::Object(object)
        }
        _ => value.clone(),
    }
}
impl Parser {
    pub fn parse_model(
        &self,
        input: &str,
        decimal: &str,
    ) -> Result<Umath_Model_Document, ModelError> {
        if ![".", ",", "٫"].contains(&decimal) {
            return Err(ModelError::Parse(Error::UnsupportedDecimal(decimal.into())));
        }
        let label = crate::languages::umath_label(input);
        let mut wire = json!({"decimal":decimal});
        if let Some(label) = label {
            wire["label"] = Value::String(label);
        }
        if crate::languages::umath_preprocess(input).is_some() {
            let parsed = self
                .parse_with_decimal(input, decimal)
                .map_err(ModelError::Parse)?;
            let catalog = serde_json::from_str(include_str!("../artifacts/catalog.json"))
                .expect("embedded stage catalog");
            wire["expression"] = strings(&parsed.bound["model"], &parsed.parsed_input, &catalog);
        }
        json_codecs::read_Umath_Model_Document_json(&wire.to_string()).map_err(ModelError::Codec)
    }
}
pub fn to_portable_json(model: &Umath_Model_Document) -> Result<String, ModelError> {
    json_codecs::write_Umath_Model_Document_json(model).map_err(ModelError::Codec)
}

fn decode_numeric(input: &str) -> String {
    let mut out = String::new();
    let mut pos = 0;
    while let Some(start) = input[pos..].find("&#").map(|i| pos + i) {
        out.push_str(&input[pos..start]);
        let Some(end) = input[start..].find(';').map(|i| start + i) else {
            out.push_str(&input[start..]);
            return out;
        };
        let raw = &input[start + 2..end];
        let code = if let Some(hex) = raw.strip_prefix('x') {
            if hex.len() <= 6 {
                u32::from_str_radix(hex, 16).ok()
            } else {
                None
            }
        } else if raw.len() <= 7 && raw.chars().all(|c| c.is_ascii_digit()) {
            raw.parse().ok()
        } else {
            None
        };
        if let Some(c) = code.and_then(char::from_u32) {
            out.push(c);
            pos = end + 1;
        } else {
            out.push('&');
            pos = start + 1;
        }
    }
    out.push_str(&input[pos..]);
    out
}
