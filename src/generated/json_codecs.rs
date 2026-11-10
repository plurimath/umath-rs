// Generated model-specific JSON codecs. Schema declarations are compiled out.
#![allow(unused_mut)]
use super::models::*;
use super::contracts::{ContractError, ContractResult, ErrorReason};
use serde_json::{Value, Map};
fn failure(reason: ErrorReason, id: &str, member: &str, location: &str, detail: impl ToString)->Box<ContractError>{
 Box::new(ContractError{reason,model_id:Some(id.into()),attribute:(!member.is_empty()).then(||member.into()),format:Some("json".into()),rule:None,location:location.into(),klass:None,detail:Some(detail.to_string())})
}
fn object(value:Value,id:&str)->ContractResult<Map<String,Value>> { value.as_object().cloned().ok_or_else(||failure(ErrorReason::InvalidDocument,id,"","/input","expected object")) }
#[allow(dead_code)]
fn empty(v:&Value)->bool { match v { Value::String(v)=>v.is_empty(),Value::Array(v)=>v.is_empty(),Value::Object(v)=>v.is_empty(),_=>false } }
fn numeric_json_err()->&'static str{"numeric JSON is not a portable exact boundary: the portable-exact-json-v1 profile requires exact values as strings; this looks like ruby-native-v1 numeric JSON (Ruby to_json), which is not accepted"}
// Only a plain 64-bit integer (ordered-content indexes) may be a JSON number; any
// fractional, exponent, negative-zero or oversized spelling is not a portable exact boundary.
fn plain_integer(n:&str)->Result<Value,&'static str>{
 let digits=n.strip_prefix('-').unwrap_or(n);
 let ok=!digits.is_empty()&&digits.bytes().all(|c|c.is_ascii_digit())&&n!="-0"&&(n.parse::<i64>().is_ok()||n.parse::<u64>().is_ok());
 if ok{serde_json::from_str::<Value>(n).map_err(|_|numeric_json_err())}else{Err(numeric_json_err())}
}
// RFC 8259 number grammar, nothing else (no sign prefix, spaces, quotes or leading zeros).
#[allow(dead_code)]
fn json_number(s:&str)->bool{
 let b=s.as_bytes();let mut i=usize::from(b.first()==Some(&b'-'));
 let digits=|i:&mut usize|{let at=*i;while *i<b.len()&&b[*i].is_ascii_digit(){*i+=1;}*i-at};
 let whole=i;let n=digits(&mut i);
 if n==0||(n>1&&b[whole]==b'0'){return false;}
 if i<b.len()&&b[i]==b'.'{i+=1;if digits(&mut i)==0{return false;}}
 if i<b.len()&&(b[i]==b'e'||b[i]==b'E'){i+=1;
  if i<b.len()&&(b[i]==b'+'||b[i]==b'-'){i+=1;}
  if digits(&mut i)==0{return false;}}
 i==b.len()
}
// Numbers in a keyed-collection body that no member mapping extracts would be dropped
// unseen; refuse them like every other loose number (`keep` lists the extracted paths).
#[allow(dead_code)]
fn no_loose_except(v:&Value,keep:&[&[&str]],id:&str)->ContractResult<()>{
 if keep.iter().any(|p|p.is_empty()){return Ok(());}
 match v {
  Value::Object(o)=>o.iter().try_for_each(|(k,x)|{let sub:Vec<&[&str]>=keep.iter().filter(|p|p[0]==k.as_str()).map(|p|&p[1..]).collect();no_loose_except(x,&sub,id)}),
  other=>no_loose(other,id),
 }
}
// Bundles with a `hash` attribute parse numbers loosely; every other position must
// still refuse them, so the decoded model is walked once (`strict_*`).
#[allow(dead_code)]
fn no_loose(v:&Value,id:&str)->ContractResult<()>{
 match v {
  Value::Number(n)=>plain_integer(&n.to_string()).map(|_|()).map_err(|e|failure(ErrorReason::InvalidDocument,id,"","/input",e)),
  Value::Array(a)=>a.iter().try_for_each(|x|no_loose(x,id)),
  Value::Object(o)=>o.values().try_for_each(|x|no_loose(x,id)),
  _=>Ok(()),
 }
}
// Inside a hash a number must be exact (the same rule the TS strict reader applies).
#[allow(dead_code)]
fn neutral_numbers(v:&Value,id:&str)->ContractResult<()>{
 match v {
  Value::Number(n)=>NeutralNumber::parse(&n.to_string()).map(|_|()).map_err(|e|failure(ErrorReason::InvalidDocument,id,"","/input",format!("{e}: {n}"))),
  Value::Array(a)=>a.iter().try_for_each(|x|neutral_numbers(x,id)),
  Value::Object(o)=>o.values().try_for_each(|x|neutral_numbers(x,id)),
  _=>Ok(()),
 }
}
#[allow(dead_code)]
fn strict_each(v:&Value,f:&dyn std::ops::Fn(&Value)->ContractResult<()>)->ContractResult<()>{ match v { Value::Array(a)=>a.iter().try_for_each(f), other=>f(other) } }
// A hash keeps its key order only when serde_json keeps insertion order.
#[allow(dead_code)]
fn ordered_maps(id:&str)->ContractResult<()>{
 let mut m=Map::new(); m.insert("b".into(),Value::Null); m.insert("a".into(),Value::Null);
 if m.keys().next().map(String::as_str)==Some("b"){Ok(())}else{Err(failure(ErrorReason::InvalidDocument,id,"","/input","hash attributes need serde_json with the preserve_order feature"))}
}
fn parse_json(input:&str,id:&str)->ContractResult<Value>{
 // Reject duplicate keys before mapping aliases. serde_json::Value alone would
 // silently keep only the last key, erasing an observable input ambiguity.
 struct Unique(Value);
 impl<'de> serde::Deserialize<'de> for Unique {
  fn deserialize<D:serde::Deserializer<'de>>(d:D)->Result<Self,D::Error>{
   struct Visitor;
   impl<'de> serde::de::Visitor<'de> for Visitor {
    type Value=Unique;
    fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("JSON without duplicate keys")}
    fn visit_bool<E:serde::de::Error>(self,v:bool)->Result<Unique,E>{Ok(Unique(Value::Bool(v)))}
    // serde_json reads "-0" as the integer 0, so a signed zero is -0 spelled as an integer; keep it as a float
    // that no integer, string or hash position accepts (negative zero is not exact).
    fn visit_i64<E:serde::de::Error>(self,v:i64)->Result<Unique,E>{if NEUTRAL_NUMBERS&&v==0{return Ok(Unique(serde_json::Number::from_f64(-0.0).map_or(Value::Null,Value::Number)));}Ok(Unique(v.into()))}
    fn visit_u64<E:serde::de::Error>(self,v:u64)->Result<Unique,E>{Ok(Unique(v.into()))}
    fn visit_f64<E:serde::de::Error>(self,_:f64)->Result<Unique,E>{Err(E::custom(numeric_json_err()))}
    fn visit_str<E:serde::de::Error>(self,v:&str)->Result<Unique,E>{Ok(Unique(Value::String(v.into())))}
    fn visit_string<E:serde::de::Error>(self,v:String)->Result<Unique,E>{Ok(Unique(Value::String(v)))}
    fn visit_unit<E:serde::de::Error>(self)->Result<Unique,E>{Ok(Unique(Value::Null))}
    fn visit_none<E:serde::de::Error>(self)->Result<Unique,E>{Ok(Unique(Value::Null))}
    fn visit_seq<A:serde::de::SeqAccess<'de>>(self,mut a:A)->Result<Unique,A::Error>{let mut v=Vec::new();while let Some(Unique(x))=a.next_element()?{v.push(x);}Ok(Unique(Value::Array(v)))}
    fn visit_map<A:serde::de::MapAccess<'de>>(self,mut a:A)->Result<Unique,A::Error>{
     let mut v=Map::new();let mut first=true;
     while let Some(k)=a.next_key::<String>()?{
      // With serde_json arbitrary_precision (always on for consumers of this crate)
      // every JSON number arrives as a one-entry map, never through visit_f64.
      // The marker's value must be a lone string in the strict JSON number grammar: nothing
      // else may travel through it (no quoted text, objects, spaces or sibling keys).
      if first && k=="$serde_json::private::Number"{let n:String=a.next_value()?;if a.next_key::<String>()?.is_some(){return Err(serde::de::Error::custom("not a JSON number"));}
      if NEUTRAL_NUMBERS{if !json_number(&n){return Err(serde::de::Error::custom("not a JSON number"));}return serde_json::from_str::<Value>(&n).map(Unique).map_err(serde::de::Error::custom);}
      return plain_integer(&n).map(Unique).map_err(serde::de::Error::custom);}
      first=false;
      let Unique(x)=a.next_value()?;
      if v.insert(k,x).is_some(){return Err(serde::de::Error::custom("duplicate JSON key"));}
     }
     Ok(Unique(Value::Object(v)))
    }
   }
   d.deserialize_any(Visitor)
  }
 }
 serde_json::from_str::<Unique>(input).map(|v|v.0).map_err(|e|failure(ErrorReason::InvalidDocument,id,"","/input",e))
}
#[allow(dead_code)]
fn get_path(v:&Value,path:&[&str])->Option<Value>{let mut at=v;for part in path{at=at.as_object()?.get(*part)?;}Some(at.clone())}
#[allow(dead_code)]
fn put_path(v:&mut Value,path:&[&str],value:Value,id:&str)->ContractResult<()>{
 if path.is_empty(){if !v.is_null(){return Err(failure(ErrorReason::UnsupportedMapping,id,"","/output","overlapping value mappings"));}*v=value;return Ok(());}
 if v.is_null(){*v=Value::Object(Map::new());}
 let map=v.as_object_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,id,"","/output","overlapping mapping paths"))?;
 let slot=map.entry(path[0]).or_insert(Value::Null);put_path(slot,&path[1..],value,id)
}
const NEUTRAL_NUMBERS: bool = false;
#[allow(non_snake_case)]
pub fn read_Umath_Model_Accents_json(input:&str)->ContractResult<Umath_Model_Accents>{let data=decode_Umath_Model_Accents(parse_json(input,"Umath::Model::Accents@default")?)?; let model:Umath_Model_Accents=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Accents@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Accents_json(model:&Umath_Model_Accents)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Accents@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Accents(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Accents@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Accents(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Accents@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("empty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Accents@default", "empty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("empty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Accents@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("list") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Accents@default", "list", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("list".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("prime_accent_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Accents@default", "prime_accent_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("prime_accent_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Accents@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Accents(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Accents(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("empty"){
data.insert("empty".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("list"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Umath::Model::Accents@default", "list", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Umath_Model_ListItem(v)?;
*slot=v; }
}
data.insert("list".into(),v);}
if let Some(mut v)=data.remove("prime_accent_symbols"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Umath::Model::Accents@default", "prime_accent_symbols", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Umath_Model_PrimeAccentSymbolsItem(v)?;
*slot=v; }
}
data.insert("prime_accent_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Accents(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Accents@default")?;let mut wire=Map::new();
for mut v in data.remove("empty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("empty".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("list").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Umath::Model::Accents@default", "list", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Umath_Model_ListItem(v)?;
*slot=v; }
}
wire.insert("list".into(),v);}
for mut v in data.remove("prime_accent_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Umath::Model::Accents@default", "prime_accent_symbols", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Umath_Model_PrimeAccentSymbolsItem(v)?;
*slot=v; }
}
wire.insert("prime_accent_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_AccentsSubsup_json(input:&str)->ContractResult<Umath_Model_AccentsSubsup>{let data=decode_Umath_Model_AccentsSubsup(parse_json(input,"Umath::Model::AccentsSubsup@default")?)?; let model:Umath_Model_AccentsSubsup=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::AccentsSubsup@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_AccentsSubsup_json(model:&Umath_Model_AccentsSubsup)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::AccentsSubsup@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_AccentsSubsup(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::AccentsSubsup@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_AccentsSubsup(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::AccentsSubsup@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::AccentsSubsup@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::AccentsSubsup@default", "sub", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::AccentsSubsup@default", "sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::AccentsSubsup@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_AccentsSubsup(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_AccentsSubsup(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("sub"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sub".into(),v);}
if let Some(mut v)=data.remove("sup"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sup".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_AccentsSubsup(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::AccentsSubsup@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("sub").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sub".into(),v);}
for mut v in data.remove("sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sup".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_ArgArguments_json(input:&str)->ContractResult<Umath_Model_ArgArguments>{let data=decode_Umath_Model_ArgArguments(parse_json(input,"Umath::Model::ArgArguments@default")?)?; let model:Umath_Model_ArgArguments=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::ArgArguments@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_ArgArguments_json(model:&Umath_Model_ArgArguments)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::ArgArguments@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_ArgArguments(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::ArgArguments@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_ArgArguments(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::ArgArguments@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::ArgArguments@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::ArgArguments@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_ArgArguments(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_ArgArguments(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Umath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_ArgArguments(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::ArgArguments@default")?;let mut wire=Map::new();
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_ArrayNode_json(input:&str)->ContractResult<Umath_Model_ArrayNode>{let data=decode_Umath_Model_ArrayNode(parse_json(input,"Umath::Model::ArrayNode@default")?)?; let model:Umath_Model_ArrayNode=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::ArrayNode@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_ArrayNode_json(model:&Umath_Model_ArrayNode)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::ArrayNode@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_ArrayNode(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::ArrayNode@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_ArrayNode(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::ArrayNode@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("tr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::ArrayNode@default", "tr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("tr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("trs") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::ArrayNode@default", "trs", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("trs".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::ArrayNode@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_ArrayNode(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_ArrayNode(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("tr"){
if !v.is_null() {
v=decode_Umath_Model_Tr(v)?;
}
data.insert("tr".into(),v);}
if let Some(mut v)=data.remove("trs"){
if !v.is_null() {
v=decode_Umath_Model_Trs(v)?;
}
data.insert("trs".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_ArrayNode(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::ArrayNode@default")?;let mut wire=Map::new();
for mut v in data.remove("tr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Tr(v)?;
}
wire.insert("tr".into(),v);}
for mut v in data.remove("trs").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Trs(v)?;
}
wire.insert("trs".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Atom_json(input:&str)->ContractResult<Umath_Model_Atom>{let data=decode_Umath_Model_Atom(parse_json(input,"Umath::Model::Atom@default")?)?; let model:Umath_Model_Atom=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Atom@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Atom_json(model:&Umath_Model_Atom)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Atom@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Atom(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Atom@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Atom(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Atom@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("alphanumeric") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Atom@default", "alphanumeric", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("alphanumeric".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("char") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Atom@default", "char", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("char".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("diacritics") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Atom@default", "diacritics", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("diacritics".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Atom@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Atom@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Atom@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Atom(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Atom(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("alphanumeric"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("alphanumeric".into(),v);}
if let Some(mut v)=data.remove("char"){
if !v.is_null() {
v=decode_Umath_Model_Char(v)?;
}
data.insert("char".into(),v);}
if let Some(mut v)=data.remove("diacritics"){
if !v.is_null() {
v=decode_Umath_Model_Diacritics(v)?;
}
data.insert("diacritics".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Atom(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Atom@default")?;let mut wire=Map::new();
for mut v in data.remove("alphanumeric").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("alphanumeric".into(),v);}
for mut v in data.remove("char").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Char(v)?;
}
wire.insert("char".into(),v);}
for mut v in data.remove("diacritics").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Diacritics(v)?;
}
wire.insert("diacritics".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Atoms_json(input:&str)->ContractResult<Umath_Model_Atoms>{let data=decode_Umath_Model_Atoms(parse_json(input,"Umath::Model::Atoms@default")?)?; let model:Umath_Model_Atoms=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Atoms@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Atoms_json(model:&Umath_Model_Atoms)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Atoms@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Atoms(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Atoms@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Atoms(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Atoms@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("atom") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Atoms@default", "atom", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("atom".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("atoms") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Atoms@default", "atoms", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("atoms".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Atoms@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Atoms(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Atoms(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("atom"){
if !v.is_null() {
v=decode_Umath_Model_Atom(v)?;
}
data.insert("atom".into(),v);}
if let Some(mut v)=data.remove("atoms"){
if !v.is_null() {
v=decode_Umath_Model_Atoms(v)?;
}
data.insert("atoms".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Atoms(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Atoms@default")?;let mut wire=Map::new();
for mut v in data.remove("atom").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Atom(v)?;
}
wire.insert("atom".into(),v);}
for mut v in data.remove("atoms").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Atoms(v)?;
}
wire.insert("atoms".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Backcolor_json(input:&str)->ContractResult<Umath_Model_Backcolor>{let data=decode_Umath_Model_Backcolor(parse_json(input,"Umath::Model::Backcolor@default")?)?; let model:Umath_Model_Backcolor=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Backcolor@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Backcolor_json(model:&Umath_Model_Backcolor)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Backcolor@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Backcolor(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Backcolor@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Backcolor(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Backcolor@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("backcolor_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Backcolor@default", "backcolor_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("backcolor_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Backcolor@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Backcolor@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Backcolor(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Backcolor(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("backcolor_value"){
data.insert("backcolor_value".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("first_value".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Backcolor(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Backcolor@default")?;let mut wire=Map::new();
for mut v in data.remove("backcolor_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("backcolor_value".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("first_value".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_BelowAfter_json(input:&str)->ContractResult<Umath_Model_BelowAfter>{let data=decode_Umath_Model_BelowAfter(parse_json(input,"Umath::Model::BelowAfter@default")?)?; let model:Umath_Model_BelowAfter=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::BelowAfter@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_BelowAfter_json(model:&Umath_Model_BelowAfter)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::BelowAfter@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_BelowAfter(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::BelowAfter@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_BelowAfter(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::BelowAfter@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("diacritic_belows") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::BelowAfter@default", "diacritic_belows", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("diacritic_belows".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::BelowAfter@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_BelowAfter(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_BelowAfter(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("diacritic_belows"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("diacritic_belows".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_BelowAfter(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::BelowAfter@default")?;let mut wire=Map::new();
for mut v in data.remove("diacritic_belows").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("diacritic_belows".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_BelowBefore_json(input:&str)->ContractResult<Umath_Model_BelowBefore>{let data=decode_Umath_Model_BelowBefore(parse_json(input,"Umath::Model::BelowBefore@default")?)?; let model:Umath_Model_BelowBefore=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::BelowBefore@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_BelowBefore_json(model:&Umath_Model_BelowBefore)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::BelowBefore@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_BelowBefore(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::BelowBefore@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_BelowBefore(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::BelowBefore@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("diacritic_belows") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::BelowBefore@default", "diacritic_belows", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("diacritic_belows".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::BelowBefore@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_BelowBefore(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_BelowBefore(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("diacritic_belows"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("diacritic_belows".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_BelowBefore(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::BelowBefore@default")?;let mut wire=Map::new();
for mut v in data.remove("diacritic_belows").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("diacritic_belows".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Char_json(input:&str)->ContractResult<Umath_Model_Char>{let data=decode_Umath_Model_Char(parse_json(input,"Umath::Model::Char@default")?)?; let model:Umath_Model_Char=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Char@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Char_json(model:&Umath_Model_Char)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Char@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Char(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Char@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Char(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Char@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Char@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Char@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Char(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Char(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Char(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Char@default")?;let mut wire=Map::new();
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_CloseParenMask_json(input:&str)->ContractResult<Umath_Model_CloseParenMask>{let data=decode_Umath_Model_CloseParenMask(parse_json(input,"Umath::Model::CloseParenMask@default")?)?; let model:Umath_Model_CloseParenMask=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::CloseParenMask@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_CloseParenMask_json(model:&Umath_Model_CloseParenMask)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::CloseParenMask@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_CloseParenMask(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::CloseParenMask@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_CloseParenMask(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::CloseParenMask@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::CloseParenMask@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::CloseParenMask@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_CloseParenMask(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_CloseParenMask(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_CloseParenMask(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::CloseParenMask@default")?;let mut wire=Map::new();
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Closer_json(input:&str)->ContractResult<Umath_Model_Closer>{let data=decode_Umath_Model_Closer(parse_json(input,"Umath::Model::Closer@default")?)?; let model:Umath_Model_Closer=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Closer@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Closer_json(model:&Umath_Model_Closer)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Closer@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Closer(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Closer@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Closer(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Closer@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("close_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Closer@default", "close_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("close_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("close_paren_mask") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Closer@default", "close_paren_mask", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("close_paren_mask".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("paren_close_prefix") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Closer@default", "paren_close_prefix", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("paren_close_prefix".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Closer@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Closer(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Closer(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("close_paren"){
if !v.is_null() {
v=decode_Umath_Model_Paren(v)?;
}
data.insert("close_paren".into(),v);}
if let Some(mut v)=data.remove("close_paren_mask"){
if !v.is_null() {
v=decode_Umath_Model_CloseParenMask(v)?;
}
data.insert("close_paren_mask".into(),v);}
if let Some(mut v)=data.remove("paren_close_prefix"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("paren_close_prefix".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Closer(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Closer@default")?;let mut wire=Map::new();
for mut v in data.remove("close_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Paren(v)?;
}
wire.insert("close_paren".into(),v);}
for mut v in data.remove("close_paren_mask").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_CloseParenMask(v)?;
}
wire.insert("close_paren_mask".into(),v);}
for mut v in data.remove("paren_close_prefix").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("paren_close_prefix".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Color_json(input:&str)->ContractResult<Umath_Model_Color>{let data=decode_Umath_Model_Color(parse_json(input,"Umath::Model::Color@default")?)?; let model:Umath_Model_Color=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Color@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Color_json(model:&Umath_Model_Color)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Color@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Color(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Color@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Color(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Color@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("color_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Color@default", "color_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Color@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Color@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Color(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Color(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("color_value"){
data.insert("color_value".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("first_value".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Color(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Color@default")?;let mut wire=Map::new();
for mut v in data.remove("color_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("color_value".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("first_value".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_DecimalNumber_json(input:&str)->ContractResult<Umath_Model_DecimalNumber>{let data=decode_Umath_Model_DecimalNumber(parse_json(input,"Umath::Model::DecimalNumber@default")?)?; let model:Umath_Model_DecimalNumber=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::DecimalNumber@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_DecimalNumber_json(model:&Umath_Model_DecimalNumber)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::DecimalNumber@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_DecimalNumber(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::DecimalNumber@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_DecimalNumber(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::DecimalNumber@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("decimal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::DecimalNumber@default", "decimal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("decimal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fractional") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::DecimalNumber@default", "fractional", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fractional".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("whole") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::DecimalNumber@default", "whole", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("whole".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::DecimalNumber@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_DecimalNumber(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_DecimalNumber(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("decimal"){
data.insert("decimal".into(),v);}
if let Some(mut v)=data.remove("fractional"){
if !v.is_null() {
v=decode_Umath_Model_Fractional(v)?;
}
data.insert("fractional".into(),v);}
if let Some(mut v)=data.remove("whole"){
if !v.is_null() {
v=decode_Umath_Model_Whole(v)?;
}
data.insert("whole".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_DecimalNumber(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::DecimalNumber@default")?;let mut wire=Map::new();
for mut v in data.remove("decimal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("decimal".into(),v);}
for mut v in data.remove("fractional").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Fractional(v)?;
}
wire.insert("fractional".into(),v);}
for mut v in data.remove("whole").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Whole(v)?;
}
wire.insert("whole".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Diacritics_json(input:&str)->ContractResult<Umath_Model_Diacritics>{let data=decode_Umath_Model_Diacritics(parse_json(input,"Umath::Model::Diacritics@default")?)?; let model:Umath_Model_Diacritics=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Diacritics@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Diacritics_json(model:&Umath_Model_Diacritics)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Diacritics@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Diacritics(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Diacritics@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Diacritics(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Diacritics@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("char") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Diacritics@default", "char", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("char".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("diacritics") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Diacritics@default", "diacritics", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("diacritics".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Diacritics@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Diacritics(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Diacritics(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("char"){
if !v.is_null() {
v=decode_Umath_Model_Char(v)?;
}
data.insert("char".into(),v);}
if let Some(mut v)=data.remove("diacritics"){
if !v.is_null() {
v=decode_Umath_Model_Diacritics(v)?;
}
data.insert("diacritics".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Diacritics(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Diacritics@default")?;let mut wire=Map::new();
for mut v in data.remove("char").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Char(v)?;
}
wire.insert("char".into(),v);}
for mut v in data.remove("diacritics").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Diacritics(v)?;
}
wire.insert("diacritics".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_DiacriticsAccents_json(input:&str)->ContractResult<Umath_Model_DiacriticsAccents>{let data=decode_Umath_Model_DiacriticsAccents(parse_json(input,"Umath::Model::DiacriticsAccents@default")?)?; let model:Umath_Model_DiacriticsAccents=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::DiacriticsAccents@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_DiacriticsAccents_json(model:&Umath_Model_DiacriticsAccents)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::DiacriticsAccents@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_DiacriticsAccents(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::DiacriticsAccents@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_DiacriticsAccents(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::DiacriticsAccents@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("below_after") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::DiacriticsAccents@default", "below_after", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("below_after".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("below_before") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::DiacriticsAccents@default", "below_before", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("below_before".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::DiacriticsAccents@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("overlay_after") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::DiacriticsAccents@default", "overlay_after", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("overlay_after".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("overlay_before") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::DiacriticsAccents@default", "overlay_before", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("overlay_before".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::DiacriticsAccents@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_DiacriticsAccents(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_DiacriticsAccents(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("below_after"){
if !v.is_null() {
v=decode_Umath_Model_BelowAfter(v)?;
}
data.insert("below_after".into(),v);}
if let Some(mut v)=data.remove("below_before"){
if !v.is_null() {
v=decode_Umath_Model_BelowBefore(v)?;
}
data.insert("below_before".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("overlay_after"){
if !v.is_null() {
v=decode_Umath_Model_OverlayAfter(v)?;
}
data.insert("overlay_after".into(),v);}
if let Some(mut v)=data.remove("overlay_before"){
if !v.is_null() {
v=decode_Umath_Model_OverlayBefore(v)?;
}
data.insert("overlay_before".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_DiacriticsAccents(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::DiacriticsAccents@default")?;let mut wire=Map::new();
for mut v in data.remove("below_after").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_BelowAfter(v)?;
}
wire.insert("below_after".into(),v);}
for mut v in data.remove("below_before").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_BelowBefore(v)?;
}
wire.insert("below_before".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("overlay_after").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_OverlayAfter(v)?;
}
wire.insert("overlay_after".into(),v);}
for mut v in data.remove("overlay_before").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_OverlayBefore(v)?;
}
wire.insert("overlay_before".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Digit_json(input:&str)->ContractResult<Umath_Model_Digit>{let data=decode_Umath_Model_Digit(parse_json(input,"Umath::Model::Digit@default")?)?; let model:Umath_Model_Digit=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Digit@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Digit_json(model:&Umath_Model_Digit)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Digit@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Digit(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Digit@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Digit(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Digit@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Digit@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Digit@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Digit(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Digit(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Digit(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Digit@default")?;let mut wire=Map::new();
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Document_json(input:&str)->ContractResult<Umath_Model_Document>{let data=decode_Umath_Model_Document(parse_json(input,"Umath::Model::Document@default")?)?; let model:Umath_Model_Document=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Document@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Document_json(model:&Umath_Model_Document)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Document@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Document(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Document@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Document(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Document@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Document@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("label") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Document@default", "label", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("label".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("decimal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Document@default", "decimal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("decimal".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Document@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Document(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Document(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("label"){
data.insert("label".into(),v);}
if let Some(mut v)=data.remove("decimal"){
data.insert("decimal".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Document(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Document@default")?;let mut wire=Map::new();
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("label").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("label".into(),v);}
for mut v in data.remove("decimal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("decimal".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_ExpIteration_json(input:&str)->ContractResult<Umath_Model_ExpIteration>{let data=decode_Umath_Model_ExpIteration(parse_json(input,"Umath::Model::ExpIteration@default")?)?; let model:Umath_Model_ExpIteration=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::ExpIteration@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_ExpIteration_json(model:&Umath_Model_ExpIteration)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::ExpIteration@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_ExpIteration(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::ExpIteration@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_ExpIteration(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::ExpIteration@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("exp_iteration") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::ExpIteration@default", "exp_iteration", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("exp_iteration".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_sub") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::ExpIteration@default", "mini_sub", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_sub".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_sub_sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::ExpIteration@default", "mini_sub_sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_sub_sup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::ExpIteration@default", "mini_sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_sup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursion") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::ExpIteration@default", "sub_recursion", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursion".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursion") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::ExpIteration@default", "sup_recursion", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursion".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::ExpIteration@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_ExpIteration(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_ExpIteration(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("exp_iteration"){
if !v.is_null() {
v=decode_Umath_Model_ExpIteration(v)?;
}
data.insert("exp_iteration".into(),v);}
if let Some(mut v)=data.remove("mini_sub"){
if !v.is_null() {
v=decode_Umath_Model_MiniSub(v)?;
}
data.insert("mini_sub".into(),v);}
if let Some(mut v)=data.remove("mini_sub_sup"){
if !v.is_null() {
v=decode_Umath_Model_MiniSubSup(v)?;
}
data.insert("mini_sub_sup".into(),v);}
if let Some(mut v)=data.remove("mini_sup"){
if !v.is_null() {
v=decode_Umath_Model_MiniSup(v)?;
}
data.insert("mini_sup".into(),v);}
if let Some(mut v)=data.remove("sub_recursion"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sub_recursion".into(),v);}
if let Some(mut v)=data.remove("sup_recursion"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sup_recursion".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_ExpIteration(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::ExpIteration@default")?;let mut wire=Map::new();
for mut v in data.remove("exp_iteration").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_ExpIteration(v)?;
}
wire.insert("exp_iteration".into(),v);}
for mut v in data.remove("mini_sub").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniSub(v)?;
}
wire.insert("mini_sub".into(),v);}
for mut v in data.remove("mini_sub_sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniSubSup(v)?;
}
wire.insert("mini_sub_sup".into(),v);}
for mut v in data.remove("mini_sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniSup(v)?;
}
wire.insert("mini_sup".into(),v);}
for mut v in data.remove("sub_recursion").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sub_recursion".into(),v);}
for mut v in data.remove("sup_recursion").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sup_recursion".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Expression_json(input:&str)->ContractResult<Umath_Model_Expression>{let data=decode_Umath_Model_Expression(parse_json(input,"Umath::Model::Expression@default")?)?; let model:Umath_Model_Expression=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Expression_json(model:&Umath_Model_Expression)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Expression(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Expression(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Expression@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("accents") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "accents", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("accents".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("accents_subsup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "accents_subsup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("accents_subsup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("alphanumeric") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "alphanumeric", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("alphanumeric".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("arg") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "arg", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("arg".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("arg_arguments") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "arg_arguments", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("arg_arguments".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("atom") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "atom", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("atom".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("atoms") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "atoms", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("atoms".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("backcolor") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "backcolor", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("backcolor".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_negated_operator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "binary_negated_operator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_negated_operator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "binary_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("char") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "char", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("char".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("close_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "close_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("close_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("close_paren_mask") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "close_paren_mask", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("close_paren_mask".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("color") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "color", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("color".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("combined_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "combined_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("combined_symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("comma") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "comma", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("comma".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("decimal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "decimal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("decimal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("diacritics") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "diacritics", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("diacritics".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("diacritics_accents") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "diacritics_accents", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("diacritics_accents".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("digit") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "digit", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("digit".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("empty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "empty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("empty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("exclamation_symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "exclamation_symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("exclamation_symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("exp_iteration") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "exp_iteration", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("exp_iteration".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("factor") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "factor", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("factor".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("frac") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "frac", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("frac".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("func_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "func_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("func_expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hbrack") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "hbrack", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hbrack".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hbracket_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "hbracket_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hbracket_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("infty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "infty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("infty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("int_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "int_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("int_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intent") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "intent", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intent".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intent_arguments") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "intent_arguments", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intent_arguments".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intent_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "intent_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intent_expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "left_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal_2") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "literal_2", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal_2".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mask") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "mask", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mask".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mid_symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "mid_symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mid_symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "mini_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "mini_intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_sub") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "mini_sub", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_sub".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_sub_sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "mini_sub_sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_sub_sup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "mini_sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_sup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("monospace") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "monospace", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("monospace".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("nary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "nary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("nary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("nary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "nary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("nary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("naryand_recursion") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "naryand_recursion", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("naryand_recursion".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("negated_operator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "negated_operator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("negated_operator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("open_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "open_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("open_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("open_paren_mask") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "open_paren_mask", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("open_paren_mask".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operand") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "operand", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operand".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "operator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ordinary_negated_operator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "ordinary_negated_operator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ordinary_negated_operator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ordinary_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "ordinary_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ordinary_symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("other") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "other", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("other".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("override_subsup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "override_subsup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("override_subsup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("paren_close_prefix") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "paren_close_prefix", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("paren_close_prefix".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("paren_open_prefix") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "paren_open_prefix", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("paren_open_prefix".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("phantom") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "phantom", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("phantom".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("pre_override_subsup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "pre_override_subsup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("pre_override_subsup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("pre_script") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "pre_script", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("pre_script".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("pre_subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "pre_subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("pre_subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("pre_supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "pre_supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("pre_supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("rect") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "rect", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("rect".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("recursive_denominator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "recursive_denominator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("recursive_denominator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("recursive_numerator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "recursive_numerator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("recursive_numerator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("relational_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "relational_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("relational_symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "right_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("root") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "root", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("root".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("root_first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "root_first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("root_first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("root_second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "root_second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("root_second_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("size_overrides") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "size_overrides", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("size_overrides".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("slashed_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "slashed_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("slashed_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("spaces") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "spaces", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("spaces".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sub", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_alpha") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sub_alpha", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_alpha".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sub_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sub_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_operators") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sub_operators", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_operators".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sub_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursion") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sub_recursion", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursion".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursion_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sub_recursion_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursion_expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sub_recursions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_script") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sub_script", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_script".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subsup_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "subsup_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subsup_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_alpha") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sup_alpha", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_alpha".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sup_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sup_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_operators") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sup_operators", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_operators".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sup_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursion") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sup_recursion", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursion".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursion_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sup_recursion_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursion_expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sup_recursions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_script") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "sup_script", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_script".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "table", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_function") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "unary_function", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_function".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_subsup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "unary_subsup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_subsup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("under") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "under", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("under".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_fractions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "unicode_fractions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_fractions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Expression@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Expression(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Expression(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("accents"){
if !v.is_null() {
v=decode_Umath_Model_Accents(v)?;
}
data.insert("accents".into(),v);}
if let Some(mut v)=data.remove("accents_subsup"){
if !v.is_null() {
v=decode_Umath_Model_AccentsSubsup(v)?;
}
data.insert("accents_subsup".into(),v);}
if let Some(mut v)=data.remove("alphanumeric"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("alphanumeric".into(),v);}
if let Some(mut v)=data.remove("arg"){
data.insert("arg".into(),v);}
if let Some(mut v)=data.remove("arg_arguments"){
if !v.is_null() {
v=decode_Umath_Model_ArgArguments(v)?;
}
data.insert("arg_arguments".into(),v);}
if let Some(mut v)=data.remove("atom"){
if !v.is_null() {
v=decode_Umath_Model_Atom(v)?;
}
data.insert("atom".into(),v);}
if let Some(mut v)=data.remove("atoms"){
if !v.is_null() {
v=decode_Umath_Model_Atoms(v)?;
}
data.insert("atoms".into(),v);}
if let Some(mut v)=data.remove("backcolor"){
if !v.is_null() {
v=decode_Umath_Model_Backcolor(v)?;
}
data.insert("backcolor".into(),v);}
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("binary_negated_operator"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("binary_negated_operator".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("binary_symbols"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("binary_symbols".into(),v);}
if let Some(mut v)=data.remove("char"){
if !v.is_null() {
v=decode_Umath_Model_Char(v)?;
}
data.insert("char".into(),v);}
if let Some(mut v)=data.remove("close_paren"){
if !v.is_null() {
v=decode_Umath_Model_Paren(v)?;
}
data.insert("close_paren".into(),v);}
if let Some(mut v)=data.remove("close_paren_mask"){
if !v.is_null() {
v=decode_Umath_Model_CloseParenMask(v)?;
}
data.insert("close_paren_mask".into(),v);}
if let Some(mut v)=data.remove("color"){
if !v.is_null() {
v=decode_Umath_Model_Color(v)?;
}
data.insert("color".into(),v);}
if let Some(mut v)=data.remove("combined_symbols"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("combined_symbols".into(),v);}
if let Some(mut v)=data.remove("comma"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("comma".into(),v);}
if let Some(mut v)=data.remove("decimal_number"){
if !v.is_null() {
v=decode_Umath_Model_DecimalNumber(v)?;
}
data.insert("decimal_number".into(),v);}
if let Some(mut v)=data.remove("diacritics"){
if !v.is_null() {
v=decode_Umath_Model_Diacritics(v)?;
}
data.insert("diacritics".into(),v);}
if let Some(mut v)=data.remove("diacritics_accents"){
if !v.is_null() {
v=decode_Umath_Model_DiacriticsAccents(v)?;
}
data.insert("diacritics_accents".into(),v);}
if let Some(mut v)=data.remove("digit"){
if !v.is_null() {
v=decode_Umath_Model_Digit(v)?;
}
data.insert("digit".into(),v);}
if let Some(mut v)=data.remove("empty"){
data.insert("empty".into(),v);}
if let Some(mut v)=data.remove("exclamation_symbol"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("exclamation_symbol".into(),v);}
if let Some(mut v)=data.remove("exp"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("exp".into(),v);}
if let Some(mut v)=data.remove("exp_iteration"){
if !v.is_null() {
v=decode_Umath_Model_ExpIteration(v)?;
}
data.insert("exp_iteration".into(),v);}
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("factor"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("factor".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
if !v.is_null() {
v=decode_Umath_Model_Fonts(v)?;
}
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("frac"){
if !v.is_null() {
v=decode_Umath_Model_Frac(v)?;
}
data.insert("frac".into(),v);}
if let Some(mut v)=data.remove("func_expr"){
if !v.is_null() {
v=decode_Umath_Model_FuncExpr(v)?;
}
data.insert("func_expr".into(),v);}
if let Some(mut v)=data.remove("hbrack"){
if !v.is_null() {
v=decode_Umath_Model_Hbrack(v)?;
}
data.insert("hbrack".into(),v);}
if let Some(mut v)=data.remove("hbracket_class"){
data.insert("hbracket_class".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("infty"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("infty".into(),v);}
if let Some(mut v)=data.remove("int_exp"){
if !v.is_null() {
v=decode_Umath_Model_IntExp(v)?;
}
data.insert("int_exp".into(),v);}
if let Some(mut v)=data.remove("intent"){
data.insert("intent".into(),v);}
if let Some(mut v)=data.remove("intent_arguments"){
if !v.is_null() {
v=decode_Umath_Model_IntentArguments(v)?;
}
data.insert("intent_arguments".into(),v);}
if let Some(mut v)=data.remove("intent_expr"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("intent_expr".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left_value"){
if !v.is_null() {
v=decode_Umath_Model_LeftValue(v)?;
}
data.insert("left_value".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("literal_2"){
data.insert("literal_2".into(),v);}
if let Some(mut v)=data.remove("mask"){
if !v.is_null() {
v=decode_Umath_Model_Mask(v)?;
}
data.insert("mask".into(),v);}
if let Some(mut v)=data.remove("mid_symbol"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("mid_symbol".into(),v);}
if let Some(mut v)=data.remove("mini_base"){
if !v.is_null() {
v=decode_Umath_Model_MiniBase(v)?;
}
data.insert("mini_base".into(),v);}
if let Some(mut v)=data.remove("mini_intermediate_exp"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("mini_intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("mini_sub"){
if !v.is_null() {
v=decode_Umath_Model_MiniSub(v)?;
}
data.insert("mini_sub".into(),v);}
if let Some(mut v)=data.remove("mini_sub_sup"){
if !v.is_null() {
v=decode_Umath_Model_MiniSubSup(v)?;
}
data.insert("mini_sub_sup".into(),v);}
if let Some(mut v)=data.remove("mini_sup"){
if !v.is_null() {
v=decode_Umath_Model_MiniSup(v)?;
}
data.insert("mini_sup".into(),v);}
if let Some(mut v)=data.remove("monospace"){
if !v.is_null() {
v=decode_Umath_Model_Monospace(v)?;
}
data.insert("monospace".into(),v);}
if let Some(mut v)=data.remove("nary"){
if !v.is_null() {
v=decode_Umath_Model_Nary(v)?;
}
data.insert("nary".into(),v);}
if let Some(mut v)=data.remove("nary_class"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("nary_class".into(),v);}
if let Some(mut v)=data.remove("naryand_recursion"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("naryand_recursion".into(),v);}
if let Some(mut v)=data.remove("negated_operator"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("negated_operator".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("open_paren"){
if !v.is_null() {
v=decode_Umath_Model_Paren(v)?;
}
data.insert("open_paren".into(),v);}
if let Some(mut v)=data.remove("open_paren_mask"){
if !v.is_null() {
v=decode_Umath_Model_OpenParenMask(v)?;
}
data.insert("open_paren_mask".into(),v);}
if let Some(mut v)=data.remove("operand"){
if !v.is_null() {
v=decode_Umath_Model_Operand(v)?;
}
data.insert("operand".into(),v);}
if let Some(mut v)=data.remove("operator"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("operator".into(),v);}
if let Some(mut v)=data.remove("ordinary_negated_operator"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("ordinary_negated_operator".into(),v);}
if let Some(mut v)=data.remove("ordinary_symbols"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("ordinary_symbols".into(),v);}
if let Some(mut v)=data.remove("other"){
if !v.is_null() {
v=decode_Umath_Model_Other(v)?;
}
data.insert("other".into(),v);}
if let Some(mut v)=data.remove("over"){
data.insert("over".into(),v);}
if let Some(mut v)=data.remove("override_subsup"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("override_subsup".into(),v);}
if let Some(mut v)=data.remove("paren_close_prefix"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("paren_close_prefix".into(),v);}
if let Some(mut v)=data.remove("paren_open_prefix"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("paren_open_prefix".into(),v);}
if let Some(mut v)=data.remove("phantom"){
if !v.is_null() {
v=decode_Umath_Model_Phantom(v)?;
}
data.insert("phantom".into(),v);}
if let Some(mut v)=data.remove("pre_override_subsup"){
if !v.is_null() {
v=decode_Umath_Model_PreOverrideSubsup(v)?;
}
data.insert("pre_override_subsup".into(),v);}
if let Some(mut v)=data.remove("pre_script"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("pre_script".into(),v);}
if let Some(mut v)=data.remove("pre_subscript"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("pre_subscript".into(),v);}
if let Some(mut v)=data.remove("pre_supscript"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("pre_supscript".into(),v);}
if let Some(mut v)=data.remove("rect"){
if !v.is_null() {
v=decode_Umath_Model_Rect(v)?;
}
data.insert("rect".into(),v);}
if let Some(mut v)=data.remove("recursive_denominator"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("recursive_denominator".into(),v);}
if let Some(mut v)=data.remove("recursive_numerator"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("recursive_numerator".into(),v);}
if let Some(mut v)=data.remove("relational_symbols"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("relational_symbols".into(),v);}
if let Some(mut v)=data.remove("right_value"){
if !v.is_null() {
v=decode_Umath_Model_RightValue(v)?;
}
data.insert("right_value".into(),v);}
if let Some(mut v)=data.remove("root"){
if !v.is_null() {
v=decode_Umath_Model_RootNode(v)?;
}
data.insert("root".into(),v);}
if let Some(mut v)=data.remove("root_first_value"){
if !v.is_null() {
v=decode_Umath_Model_RootFirstValue(v)?;
}
data.insert("root_first_value".into(),v);}
if let Some(mut v)=data.remove("root_second_value"){
if !v.is_null() {
v=decode_Umath_Model_RootSecondValue(v)?;
}
data.insert("root_second_value".into(),v);}
if let Some(mut v)=data.remove("size_overrides"){
data.insert("size_overrides".into(),v);}
if let Some(mut v)=data.remove("slashed_value"){
if !v.is_null() {
v=decode_Umath_Model_SlashedValue(v)?;
}
data.insert("slashed_value".into(),v);}
if let Some(mut v)=data.remove("spaces"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("spaces".into(),v);}
if let Some(mut v)=data.remove("sub"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sub".into(),v);}
if let Some(mut v)=data.remove("sub_alpha"){
data.insert("sub_alpha".into(),v);}
if let Some(mut v)=data.remove("sub_digits"){
data.insert("sub_digits".into(),v);}
if let Some(mut v)=data.remove("sub_exp"){
if !v.is_null() {
v=decode_Umath_Model_SubExp(v)?;
}
data.insert("sub_exp".into(),v);}
if let Some(mut v)=data.remove("sub_operators"){
data.insert("sub_operators".into(),v);}
if let Some(mut v)=data.remove("sub_paren"){
if !v.is_null() {
v=decode_Umath_Model_SubParen(v)?;
}
data.insert("sub_paren".into(),v);}
if let Some(mut v)=data.remove("sub_recursion"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sub_recursion".into(),v);}
if let Some(mut v)=data.remove("sub_recursion_expr"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sub_recursion_expr".into(),v);}
if let Some(mut v)=data.remove("sub_recursions"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sub_recursions".into(),v);}
if let Some(mut v)=data.remove("sub_script"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sub_script".into(),v);}
if let Some(mut v)=data.remove("subsup_exp"){
if !v.is_null() {
v=decode_Umath_Model_SubsupExp(v)?;
}
data.insert("subsup_exp".into(),v);}
if let Some(mut v)=data.remove("sup"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sup".into(),v);}
if let Some(mut v)=data.remove("sup_alpha"){
data.insert("sup_alpha".into(),v);}
if let Some(mut v)=data.remove("sup_digits"){
data.insert("sup_digits".into(),v);}
if let Some(mut v)=data.remove("sup_exp"){
if !v.is_null() {
v=decode_Umath_Model_SupExp(v)?;
}
data.insert("sup_exp".into(),v);}
if let Some(mut v)=data.remove("sup_operators"){
data.insert("sup_operators".into(),v);}
if let Some(mut v)=data.remove("sup_paren"){
if !v.is_null() {
v=decode_Umath_Model_SupParen(v)?;
}
data.insert("sup_paren".into(),v);}
if let Some(mut v)=data.remove("sup_recursion"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sup_recursion".into(),v);}
if let Some(mut v)=data.remove("sup_recursion_expr"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sup_recursion_expr".into(),v);}
if let Some(mut v)=data.remove("sup_recursions"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sup_recursions".into(),v);}
if let Some(mut v)=data.remove("sup_script"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sup_script".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Umath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("table"){
if !v.is_null() {
v=decode_Umath_Model_Table(v)?;
}
data.insert("table".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("unary_function"){
if !v.is_null() {
v=decode_Umath_Model_UnaryFunction(v)?;
}
data.insert("unary_function".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("unary_subsup"){
if !v.is_null() {
v=decode_Umath_Model_UnarySubsup(v)?;
}
data.insert("unary_subsup".into(),v);}
if let Some(mut v)=data.remove("under"){
data.insert("under".into(),v);}
if let Some(mut v)=data.remove("unicode_fractions"){
data.insert("unicode_fractions".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Expression(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Expression@default")?;let mut wire=Map::new();
for mut v in data.remove("accents").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Accents(v)?;
}
wire.insert("accents".into(),v);}
for mut v in data.remove("accents_subsup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_AccentsSubsup(v)?;
}
wire.insert("accents_subsup".into(),v);}
for mut v in data.remove("alphanumeric").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("alphanumeric".into(),v);}
for mut v in data.remove("arg").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("arg".into(),v);}
for mut v in data.remove("arg_arguments").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_ArgArguments(v)?;
}
wire.insert("arg_arguments".into(),v);}
for mut v in data.remove("atom").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Atom(v)?;
}
wire.insert("atom".into(),v);}
for mut v in data.remove("atoms").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Atoms(v)?;
}
wire.insert("atoms".into(),v);}
for mut v in data.remove("backcolor").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Backcolor(v)?;
}
wire.insert("backcolor".into(),v);}
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("binary_negated_operator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("binary_negated_operator".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("binary_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("binary_symbols".into(),v);}
for mut v in data.remove("char").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Char(v)?;
}
wire.insert("char".into(),v);}
for mut v in data.remove("close_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Paren(v)?;
}
wire.insert("close_paren".into(),v);}
for mut v in data.remove("close_paren_mask").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_CloseParenMask(v)?;
}
wire.insert("close_paren_mask".into(),v);}
for mut v in data.remove("color").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Color(v)?;
}
wire.insert("color".into(),v);}
for mut v in data.remove("combined_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("combined_symbols".into(),v);}
for mut v in data.remove("comma").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("comma".into(),v);}
for mut v in data.remove("decimal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_DecimalNumber(v)?;
}
wire.insert("decimal_number".into(),v);}
for mut v in data.remove("diacritics").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Diacritics(v)?;
}
wire.insert("diacritics".into(),v);}
for mut v in data.remove("diacritics_accents").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_DiacriticsAccents(v)?;
}
wire.insert("diacritics_accents".into(),v);}
for mut v in data.remove("digit").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Digit(v)?;
}
wire.insert("digit".into(),v);}
for mut v in data.remove("empty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("empty".into(),v);}
for mut v in data.remove("exclamation_symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("exclamation_symbol".into(),v);}
for mut v in data.remove("exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("exp".into(),v);}
for mut v in data.remove("exp_iteration").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_ExpIteration(v)?;
}
wire.insert("exp_iteration".into(),v);}
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("factor").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("factor".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Fonts(v)?;
}
wire.insert("fonts".into(),v);}
for mut v in data.remove("frac").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Frac(v)?;
}
wire.insert("frac".into(),v);}
for mut v in data.remove("func_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_FuncExpr(v)?;
}
wire.insert("func_expr".into(),v);}
for mut v in data.remove("hbrack").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Hbrack(v)?;
}
wire.insert("hbrack".into(),v);}
for mut v in data.remove("hbracket_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hbracket_class".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("infty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("infty".into(),v);}
for mut v in data.remove("int_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_IntExp(v)?;
}
wire.insert("int_exp".into(),v);}
for mut v in data.remove("intent").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("intent".into(),v);}
for mut v in data.remove("intent_arguments").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_IntentArguments(v)?;
}
wire.insert("intent_arguments".into(),v);}
for mut v in data.remove("intent_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("intent_expr".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_LeftValue(v)?;
}
wire.insert("left_value".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("literal_2").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal_2".into(),v);}
for mut v in data.remove("mask").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Mask(v)?;
}
wire.insert("mask".into(),v);}
for mut v in data.remove("mid_symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("mid_symbol".into(),v);}
for mut v in data.remove("mini_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniBase(v)?;
}
wire.insert("mini_base".into(),v);}
for mut v in data.remove("mini_intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("mini_intermediate_exp".into(),v);}
for mut v in data.remove("mini_sub").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniSub(v)?;
}
wire.insert("mini_sub".into(),v);}
for mut v in data.remove("mini_sub_sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniSubSup(v)?;
}
wire.insert("mini_sub_sup".into(),v);}
for mut v in data.remove("mini_sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniSup(v)?;
}
wire.insert("mini_sup".into(),v);}
for mut v in data.remove("monospace").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Monospace(v)?;
}
wire.insert("monospace".into(),v);}
for mut v in data.remove("nary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Nary(v)?;
}
wire.insert("nary".into(),v);}
for mut v in data.remove("nary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("nary_class".into(),v);}
for mut v in data.remove("naryand_recursion").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("naryand_recursion".into(),v);}
for mut v in data.remove("negated_operator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("negated_operator".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("open_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Paren(v)?;
}
wire.insert("open_paren".into(),v);}
for mut v in data.remove("open_paren_mask").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_OpenParenMask(v)?;
}
wire.insert("open_paren_mask".into(),v);}
for mut v in data.remove("operand").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Operand(v)?;
}
wire.insert("operand".into(),v);}
for mut v in data.remove("operator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("operator".into(),v);}
for mut v in data.remove("ordinary_negated_operator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("ordinary_negated_operator".into(),v);}
for mut v in data.remove("ordinary_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("ordinary_symbols".into(),v);}
for mut v in data.remove("other").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Other(v)?;
}
wire.insert("other".into(),v);}
for mut v in data.remove("over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("over".into(),v);}
for mut v in data.remove("override_subsup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("override_subsup".into(),v);}
for mut v in data.remove("paren_close_prefix").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("paren_close_prefix".into(),v);}
for mut v in data.remove("paren_open_prefix").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("paren_open_prefix".into(),v);}
for mut v in data.remove("phantom").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Phantom(v)?;
}
wire.insert("phantom".into(),v);}
for mut v in data.remove("pre_override_subsup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_PreOverrideSubsup(v)?;
}
wire.insert("pre_override_subsup".into(),v);}
for mut v in data.remove("pre_script").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("pre_script".into(),v);}
for mut v in data.remove("pre_subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("pre_subscript".into(),v);}
for mut v in data.remove("pre_supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("pre_supscript".into(),v);}
for mut v in data.remove("rect").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Rect(v)?;
}
wire.insert("rect".into(),v);}
for mut v in data.remove("recursive_denominator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("recursive_denominator".into(),v);}
for mut v in data.remove("recursive_numerator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("recursive_numerator".into(),v);}
for mut v in data.remove("relational_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("relational_symbols".into(),v);}
for mut v in data.remove("right_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_RightValue(v)?;
}
wire.insert("right_value".into(),v);}
for mut v in data.remove("root").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_RootNode(v)?;
}
wire.insert("root".into(),v);}
for mut v in data.remove("root_first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_RootFirstValue(v)?;
}
wire.insert("root_first_value".into(),v);}
for mut v in data.remove("root_second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_RootSecondValue(v)?;
}
wire.insert("root_second_value".into(),v);}
for mut v in data.remove("size_overrides").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("size_overrides".into(),v);}
for mut v in data.remove("slashed_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SlashedValue(v)?;
}
wire.insert("slashed_value".into(),v);}
for mut v in data.remove("spaces").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("spaces".into(),v);}
for mut v in data.remove("sub").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sub".into(),v);}
for mut v in data.remove("sub_alpha").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_alpha".into(),v);}
for mut v in data.remove("sub_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_digits".into(),v);}
for mut v in data.remove("sub_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SubExp(v)?;
}
wire.insert("sub_exp".into(),v);}
for mut v in data.remove("sub_operators").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_operators".into(),v);}
for mut v in data.remove("sub_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SubParen(v)?;
}
wire.insert("sub_paren".into(),v);}
for mut v in data.remove("sub_recursion").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sub_recursion".into(),v);}
for mut v in data.remove("sub_recursion_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sub_recursion_expr".into(),v);}
for mut v in data.remove("sub_recursions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sub_recursions".into(),v);}
for mut v in data.remove("sub_script").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sub_script".into(),v);}
for mut v in data.remove("subsup_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SubsupExp(v)?;
}
wire.insert("subsup_exp".into(),v);}
for mut v in data.remove("sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sup".into(),v);}
for mut v in data.remove("sup_alpha").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_alpha".into(),v);}
for mut v in data.remove("sup_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_digits".into(),v);}
for mut v in data.remove("sup_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SupExp(v)?;
}
wire.insert("sup_exp".into(),v);}
for mut v in data.remove("sup_operators").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_operators".into(),v);}
for mut v in data.remove("sup_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SupParen(v)?;
}
wire.insert("sup_paren".into(),v);}
for mut v in data.remove("sup_recursion").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sup_recursion".into(),v);}
for mut v in data.remove("sup_recursion_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sup_recursion_expr".into(),v);}
for mut v in data.remove("sup_recursions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sup_recursions".into(),v);}
for mut v in data.remove("sup_script").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sup_script".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("table").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Table(v)?;
}
wire.insert("table".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("unary_function").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_UnaryFunction(v)?;
}
wire.insert("unary_function".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("unary_subsup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_UnarySubsup(v)?;
}
wire.insert("unary_subsup".into(),v);}
for mut v in data.remove("under").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("under".into(),v);}
for mut v in data.remove("unicode_fractions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unicode_fractions".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Fonts_json(input:&str)->ContractResult<Umath_Model_Fonts>{let data=decode_Umath_Model_Fonts(parse_json(input,"Umath::Model::Fonts@default")?)?; let model:Umath_Model_Fonts=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Fonts@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Fonts_json(model:&Umath_Model_Fonts)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Fonts@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Fonts(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Fonts@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Fonts(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Fonts@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("font_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Fonts@default", "font_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("font_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Fonts@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Fonts@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Fonts@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Fonts@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicoded_font_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Fonts@default", "unicoded_font_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicoded_font_class".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Fonts@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Fonts(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Fonts(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("font_class"){
data.insert("font_class".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Umath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("unicoded_font_class"){
if !v.is_null() {
v=decode_Umath_Model_UnicodedFontClass(v)?;
}
data.insert("unicoded_font_class".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Fonts(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Fonts@default")?;let mut wire=Map::new();
for mut v in data.remove("font_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("font_class".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("unicoded_font_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_UnicodedFontClass(v)?;
}
wire.insert("unicoded_font_class".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Frac_json(input:&str)->ContractResult<Umath_Model_Frac>{let data=decode_Umath_Model_Frac(parse_json(input,"Umath::Model::Frac@default")?)?; let model:Umath_Model_Frac=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Frac@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Frac_json(model:&Umath_Model_Frac)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Frac@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Frac(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Frac@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Frac(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Frac@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("atop") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Frac@default", "atop", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("atop".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("bevelled") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Frac@default", "bevelled", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("bevelled".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("choose") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Frac@default", "choose", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("choose".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("denominator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Frac@default", "denominator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("denominator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ldiv") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Frac@default", "ldiv", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ldiv".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_denominator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Frac@default", "mini_denominator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_denominator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_numerator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Frac@default", "mini_numerator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_numerator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("no_display_style") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Frac@default", "no_display_style", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("no_display_style".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numerator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Frac@default", "numerator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numerator".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Frac@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Frac(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Frac(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("atop"){
data.insert("atop".into(),v);}
if let Some(mut v)=data.remove("bevelled"){
data.insert("bevelled".into(),v);}
if let Some(mut v)=data.remove("choose"){
data.insert("choose".into(),v);}
if let Some(mut v)=data.remove("denominator"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("denominator".into(),v);}
if let Some(mut v)=data.remove("ldiv"){
data.insert("ldiv".into(),v);}
if let Some(mut v)=data.remove("mini_denominator"){
if !v.is_null() {
v=decode_Umath_Model_MiniDenominator(v)?;
}
data.insert("mini_denominator".into(),v);}
if let Some(mut v)=data.remove("mini_numerator"){
if !v.is_null() {
v=decode_Umath_Model_MiniNumerator(v)?;
}
data.insert("mini_numerator".into(),v);}
if let Some(mut v)=data.remove("no_display_style"){
data.insert("no_display_style".into(),v);}
if let Some(mut v)=data.remove("numerator"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("numerator".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Frac(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Frac@default")?;let mut wire=Map::new();
for mut v in data.remove("atop").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("atop".into(),v);}
for mut v in data.remove("bevelled").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("bevelled".into(),v);}
for mut v in data.remove("choose").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("choose".into(),v);}
for mut v in data.remove("denominator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("denominator".into(),v);}
for mut v in data.remove("ldiv").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ldiv".into(),v);}
for mut v in data.remove("mini_denominator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniDenominator(v)?;
}
wire.insert("mini_denominator".into(),v);}
for mut v in data.remove("mini_numerator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniNumerator(v)?;
}
wire.insert("mini_numerator".into(),v);}
for mut v in data.remove("no_display_style").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("no_display_style".into(),v);}
for mut v in data.remove("numerator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("numerator".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Fractional_json(input:&str)->ContractResult<Umath_Model_Fractional>{let data=decode_Umath_Model_Fractional(parse_json(input,"Umath::Model::Fractional@default")?)?; let model:Umath_Model_Fractional=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Fractional@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Fractional_json(model:&Umath_Model_Fractional)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Fractional@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Fractional(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Fractional@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Fractional(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Fractional@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Fractional@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Fractional@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Fractional(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Fractional(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Fractional(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Fractional@default")?;let mut wire=Map::new();
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_FuncExpr_json(input:&str)->ContractResult<Umath_Model_FuncExpr>{let data=decode_Umath_Model_FuncExpr(parse_json(input,"Umath::Model::FuncExpr@default")?)?; let model:Umath_Model_FuncExpr=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::FuncExpr@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_FuncExpr_json(model:&Umath_Model_FuncExpr)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::FuncExpr@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_FuncExpr(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::FuncExpr@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_FuncExpr(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::FuncExpr@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::FuncExpr@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("func_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::FuncExpr@default", "func_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("func_expr".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::FuncExpr@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_FuncExpr(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_FuncExpr(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("func_expr"){
if !v.is_null() {
v=decode_Umath_Model_FuncExpr(v)?;
}
data.insert("func_expr".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_FuncExpr(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::FuncExpr@default")?;let mut wire=Map::new();
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("func_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_FuncExpr(v)?;
}
wire.insert("func_expr".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Hbrack_json(input:&str)->ContractResult<Umath_Model_Hbrack>{let data=decode_Umath_Model_Hbrack(parse_json(input,"Umath::Model::Hbrack@default")?)?; let model:Umath_Model_Hbrack=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Hbrack@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Hbrack_json(model:&Umath_Model_Hbrack)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Hbrack@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Hbrack(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Hbrack@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Hbrack(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Hbrack@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Hbrack@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hbracket_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Hbrack@default", "hbracket_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hbracket_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("scripted_first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Hbrack@default", "scripted_first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("scripted_first_value".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Hbrack@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Hbrack(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Hbrack(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("hbracket_class"){
data.insert("hbracket_class".into(),v);}
if let Some(mut v)=data.remove("scripted_first_value"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("scripted_first_value".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Hbrack(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Hbrack@default")?;let mut wire=Map::new();
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("hbracket_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hbracket_class".into(),v);}
for mut v in data.remove("scripted_first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("scripted_first_value".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_IntExp_json(input:&str)->ContractResult<Umath_Model_IntExp>{let data=decode_Umath_Model_IntExp(parse_json(input,"Umath::Model::IntExp@default")?)?; let model:Umath_Model_IntExp=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::IntExp@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_IntExp_json(model:&Umath_Model_IntExp)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::IntExp@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_IntExp(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::IntExp@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_IntExp(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::IntExp@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("closer") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::IntExp@default", "closer", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("closer".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("opener") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::IntExp@default", "opener", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("opener".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operand") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::IntExp@default", "operand", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operand".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::IntExp@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_IntExp(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_IntExp(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("closer"){
if !v.is_null() {
v=decode_Umath_Model_Closer(v)?;
}
data.insert("closer".into(),v);}
if let Some(mut v)=data.remove("opener"){
if !v.is_null() {
v=decode_Umath_Model_Opener(v)?;
}
data.insert("opener".into(),v);}
if let Some(mut v)=data.remove("operand"){
if !v.is_null() {
v=decode_Umath_Model_Operand(v)?;
}
data.insert("operand".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_IntExp(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::IntExp@default")?;let mut wire=Map::new();
for mut v in data.remove("closer").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Closer(v)?;
}
wire.insert("closer".into(),v);}
for mut v in data.remove("opener").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Opener(v)?;
}
wire.insert("opener".into(),v);}
for mut v in data.remove("operand").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Operand(v)?;
}
wire.insert("operand".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_IntentArguments_json(input:&str)->ContractResult<Umath_Model_IntentArguments>{let data=decode_Umath_Model_IntentArguments(parse_json(input,"Umath::Model::IntentArguments@default")?)?; let model:Umath_Model_IntentArguments=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::IntentArguments@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_IntentArguments_json(model:&Umath_Model_IntentArguments)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::IntentArguments@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_IntentArguments(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::IntentArguments@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_IntentArguments(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::IntentArguments@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::IntentArguments@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::IntentArguments@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_IntentArguments(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_IntentArguments(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_IntentArguments(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::IntentArguments@default")?;let mut wire=Map::new();
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_LeftValue_json(input:&str)->ContractResult<Umath_Model_LeftValue>{let data=decode_Umath_Model_LeftValue(parse_json(input,"Umath::Model::LeftValue@default")?)?; let model:Umath_Model_LeftValue=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::LeftValue@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_LeftValue_json(model:&Umath_Model_LeftValue)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::LeftValue@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_LeftValue(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::LeftValue@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_LeftValue(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::LeftValue@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("infty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::LeftValue@default", "infty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("infty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("list") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::LeftValue@default", "list", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("list".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("negative") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::LeftValue@default", "negative", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("negative".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::LeftValue@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("positive") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::LeftValue@default", "positive", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("positive".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::LeftValue@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_LeftValue(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_LeftValue(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("infty"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("infty".into(),v);}
if let Some(mut v)=data.remove("list"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Umath::Model::LeftValue@default", "list", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Umath_Model_ListItem(v)?;
*slot=v; }
}
data.insert("list".into(),v);}
if let Some(mut v)=data.remove("negative"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("negative".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("positive"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("positive".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_LeftValue(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::LeftValue@default")?;let mut wire=Map::new();
for mut v in data.remove("infty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("infty".into(),v);}
for mut v in data.remove("list").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Umath::Model::LeftValue@default", "list", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Umath_Model_ListItem(v)?;
*slot=v; }
}
wire.insert("list".into(),v);}
for mut v in data.remove("negative").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("negative".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("positive").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("positive".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_ListItem_json(input:&str)->ContractResult<Umath_Model_ListItem>{let data=decode_Umath_Model_ListItem(parse_json(input,"Umath::Model::ListItem@default")?)?; let model:Umath_Model_ListItem=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::ListItem@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_ListItem_json(model:&Umath_Model_ListItem)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::ListItem@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_ListItem(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::ListItem@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_ListItem(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::ListItem@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("accent_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::ListItem@default", "accent_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("accent_symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::ListItem@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::ListItem@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::ListItem@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_ListItem(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_ListItem(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("accent_symbols"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("accent_symbols".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Umath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_ListItem(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::ListItem@default")?;let mut wire=Map::new();
for mut v in data.remove("accent_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("accent_symbols".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Mask_json(input:&str)->ContractResult<Umath_Model_Mask>{let data=decode_Umath_Model_Mask(parse_json(input,"Umath::Model::Mask@default")?)?; let model:Umath_Model_Mask=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Mask@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Mask_json(model:&Umath_Model_Mask)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Mask@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Mask(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Mask@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Mask(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Mask@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Mask@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("decimal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Mask@default", "decimal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("decimal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("digit") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Mask@default", "digit", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("digit".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Mask@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Mask@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Mask@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Mask(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Mask(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("decimal_number"){
if !v.is_null() {
v=decode_Umath_Model_DecimalNumber(v)?;
}
data.insert("decimal_number".into(),v);}
if let Some(mut v)=data.remove("digit"){
if !v.is_null() {
v=decode_Umath_Model_Digit(v)?;
}
data.insert("digit".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Mask(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Mask@default")?;let mut wire=Map::new();
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("decimal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_DecimalNumber(v)?;
}
wire.insert("decimal_number".into(),v);}
for mut v in data.remove("digit").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Digit(v)?;
}
wire.insert("digit".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_MathSymbol_json(input:&str)->ContractResult<Umath_Model_MathSymbol>{let data=decode_Umath_Model_MathSymbol(parse_json(input,"Umath::Model::MathSymbol@default")?)?; let model:Umath_Model_MathSymbol=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MathSymbol@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_MathSymbol_json(model:&Umath_Model_MathSymbol)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MathSymbol@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_MathSymbol(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MathSymbol@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_MathSymbol(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::MathSymbol@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("spelling") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MathSymbol@default", "spelling", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("spelling".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MathSymbol@default", "value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("kind") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MathSymbol@default", "kind", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("kind".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MathSymbol@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_MathSymbol(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_MathSymbol(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("spelling"){
data.insert("spelling".into(),v);}
if let Some(mut v)=data.remove("value"){
data.insert("value".into(),v);}
if let Some(mut v)=data.remove("kind"){
data.insert("kind".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_MathSymbol(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::MathSymbol@default")?;let mut wire=Map::new();
for mut v in data.remove("spelling").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("spelling".into(),v);}
for mut v in data.remove("value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("value".into(),v);}
for mut v in data.remove("kind").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("kind".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_MiniBase_json(input:&str)->ContractResult<Umath_Model_MiniBase>{let data=decode_Umath_Model_MiniBase(parse_json(input,"Umath::Model::MiniBase@default")?)?; let model:Umath_Model_MiniBase=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniBase@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_MiniBase_json(model:&Umath_Model_MiniBase)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniBase@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_MiniBase(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniBase@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_MiniBase(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::MiniBase@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("sub_alpha") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniBase@default", "sub_alpha", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_alpha".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniBase@default", "sub_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_operators") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniBase@default", "sub_operators", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_operators".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_alpha") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniBase@default", "sup_alpha", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_alpha".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniBase@default", "sup_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_operators") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniBase@default", "sup_operators", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_operators".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniBase@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_MiniBase(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_MiniBase(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("sub_alpha"){
data.insert("sub_alpha".into(),v);}
if let Some(mut v)=data.remove("sub_digits"){
data.insert("sub_digits".into(),v);}
if let Some(mut v)=data.remove("sub_operators"){
data.insert("sub_operators".into(),v);}
if let Some(mut v)=data.remove("sup_alpha"){
data.insert("sup_alpha".into(),v);}
if let Some(mut v)=data.remove("sup_digits"){
data.insert("sup_digits".into(),v);}
if let Some(mut v)=data.remove("sup_operators"){
data.insert("sup_operators".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_MiniBase(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::MiniBase@default")?;let mut wire=Map::new();
for mut v in data.remove("sub_alpha").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_alpha".into(),v);}
for mut v in data.remove("sub_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_digits".into(),v);}
for mut v in data.remove("sub_operators").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_operators".into(),v);}
for mut v in data.remove("sup_alpha").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_alpha".into(),v);}
for mut v in data.remove("sup_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_digits".into(),v);}
for mut v in data.remove("sup_operators").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_operators".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_MiniDenominator_json(input:&str)->ContractResult<Umath_Model_MiniDenominator>{let data=decode_Umath_Model_MiniDenominator(parse_json(input,"Umath::Model::MiniDenominator@default")?)?; let model:Umath_Model_MiniDenominator=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniDenominator@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_MiniDenominator_json(model:&Umath_Model_MiniDenominator)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniDenominator@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_MiniDenominator(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniDenominator@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_MiniDenominator(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::MiniDenominator@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniDenominator@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniDenominator@default", "mini_intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_alpha") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniDenominator@default", "sub_alpha", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_alpha".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniDenominator@default", "sub_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_operators") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniDenominator@default", "sub_operators", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_operators".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursion") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniDenominator@default", "sub_recursion", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursion".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursion_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniDenominator@default", "sub_recursion_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursion_expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniDenominator@default", "sub_recursions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursions".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniDenominator@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_MiniDenominator(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_MiniDenominator(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("mini_intermediate_exp"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("mini_intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("sub_alpha"){
data.insert("sub_alpha".into(),v);}
if let Some(mut v)=data.remove("sub_digits"){
data.insert("sub_digits".into(),v);}
if let Some(mut v)=data.remove("sub_operators"){
data.insert("sub_operators".into(),v);}
if let Some(mut v)=data.remove("sub_recursion"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sub_recursion".into(),v);}
if let Some(mut v)=data.remove("sub_recursion_expr"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sub_recursion_expr".into(),v);}
if let Some(mut v)=data.remove("sub_recursions"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sub_recursions".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_MiniDenominator(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::MiniDenominator@default")?;let mut wire=Map::new();
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("mini_intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("mini_intermediate_exp".into(),v);}
for mut v in data.remove("sub_alpha").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_alpha".into(),v);}
for mut v in data.remove("sub_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_digits".into(),v);}
for mut v in data.remove("sub_operators").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_operators".into(),v);}
for mut v in data.remove("sub_recursion").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sub_recursion".into(),v);}
for mut v in data.remove("sub_recursion_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sub_recursion_expr".into(),v);}
for mut v in data.remove("sub_recursions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sub_recursions".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_MiniExpression_json(input:&str)->ContractResult<Umath_Model_MiniExpression>{let data=decode_Umath_Model_MiniExpression(parse_json(input,"Umath::Model::MiniExpression@default")?)?; let model:Umath_Model_MiniExpression=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_MiniExpression_json(model:&Umath_Model_MiniExpression)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_MiniExpression(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_MiniExpression(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::MiniExpression@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "mini_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "mini_intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_alpha") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "sub_alpha", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_alpha".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_close_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "sub_close_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_close_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "sub_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_open_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "sub_open_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_open_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_operators") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "sub_operators", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_operators".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursion") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "sub_recursion", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursion".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursion_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "sub_recursion_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursion_expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "sub_recursions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_alpha") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "sup_alpha", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_alpha".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_close_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "sup_close_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_close_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "sup_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_open_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "sup_open_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_open_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_operators") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "sup_operators", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_operators".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursion") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "sup_recursion", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursion".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursion_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "sup_recursion_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursion_expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "sup_recursions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursions".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniExpression@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_MiniExpression(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_MiniExpression(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("mini_expr"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("mini_expr".into(),v);}
if let Some(mut v)=data.remove("mini_intermediate_exp"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("mini_intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("sub_alpha"){
data.insert("sub_alpha".into(),v);}
if let Some(mut v)=data.remove("sub_close_paren"){
data.insert("sub_close_paren".into(),v);}
if let Some(mut v)=data.remove("sub_digits"){
data.insert("sub_digits".into(),v);}
if let Some(mut v)=data.remove("sub_open_paren"){
data.insert("sub_open_paren".into(),v);}
if let Some(mut v)=data.remove("sub_operators"){
data.insert("sub_operators".into(),v);}
if let Some(mut v)=data.remove("sub_recursion"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sub_recursion".into(),v);}
if let Some(mut v)=data.remove("sub_recursion_expr"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sub_recursion_expr".into(),v);}
if let Some(mut v)=data.remove("sub_recursions"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sub_recursions".into(),v);}
if let Some(mut v)=data.remove("sup_alpha"){
data.insert("sup_alpha".into(),v);}
if let Some(mut v)=data.remove("sup_close_paren"){
data.insert("sup_close_paren".into(),v);}
if let Some(mut v)=data.remove("sup_digits"){
data.insert("sup_digits".into(),v);}
if let Some(mut v)=data.remove("sup_open_paren"){
data.insert("sup_open_paren".into(),v);}
if let Some(mut v)=data.remove("sup_operators"){
data.insert("sup_operators".into(),v);}
if let Some(mut v)=data.remove("sup_recursion"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sup_recursion".into(),v);}
if let Some(mut v)=data.remove("sup_recursion_expr"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sup_recursion_expr".into(),v);}
if let Some(mut v)=data.remove("sup_recursions"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sup_recursions".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_MiniExpression(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::MiniExpression@default")?;let mut wire=Map::new();
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("mini_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("mini_expr".into(),v);}
for mut v in data.remove("mini_intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("mini_intermediate_exp".into(),v);}
for mut v in data.remove("sub_alpha").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_alpha".into(),v);}
for mut v in data.remove("sub_close_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_close_paren".into(),v);}
for mut v in data.remove("sub_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_digits".into(),v);}
for mut v in data.remove("sub_open_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_open_paren".into(),v);}
for mut v in data.remove("sub_operators").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_operators".into(),v);}
for mut v in data.remove("sub_recursion").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sub_recursion".into(),v);}
for mut v in data.remove("sub_recursion_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sub_recursion_expr".into(),v);}
for mut v in data.remove("sub_recursions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sub_recursions".into(),v);}
for mut v in data.remove("sup_alpha").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_alpha".into(),v);}
for mut v in data.remove("sup_close_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_close_paren".into(),v);}
for mut v in data.remove("sup_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_digits".into(),v);}
for mut v in data.remove("sup_open_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_open_paren".into(),v);}
for mut v in data.remove("sup_operators").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_operators".into(),v);}
for mut v in data.remove("sup_recursion").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sup_recursion".into(),v);}
for mut v in data.remove("sup_recursion_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sup_recursion_expr".into(),v);}
for mut v in data.remove("sup_recursions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sup_recursions".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_MiniNumerator_json(input:&str)->ContractResult<Umath_Model_MiniNumerator>{let data=decode_Umath_Model_MiniNumerator(parse_json(input,"Umath::Model::MiniNumerator@default")?)?; let model:Umath_Model_MiniNumerator=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniNumerator@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_MiniNumerator_json(model:&Umath_Model_MiniNumerator)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniNumerator@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_MiniNumerator(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniNumerator@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_MiniNumerator(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::MiniNumerator@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniNumerator@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniNumerator@default", "mini_intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_alpha") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniNumerator@default", "sup_alpha", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_alpha".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniNumerator@default", "sup_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_operators") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniNumerator@default", "sup_operators", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_operators".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursion") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniNumerator@default", "sup_recursion", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursion".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursion_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniNumerator@default", "sup_recursion_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursion_expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniNumerator@default", "sup_recursions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursions".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniNumerator@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_MiniNumerator(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_MiniNumerator(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("mini_intermediate_exp"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("mini_intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("sup_alpha"){
data.insert("sup_alpha".into(),v);}
if let Some(mut v)=data.remove("sup_digits"){
data.insert("sup_digits".into(),v);}
if let Some(mut v)=data.remove("sup_operators"){
data.insert("sup_operators".into(),v);}
if let Some(mut v)=data.remove("sup_recursion"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sup_recursion".into(),v);}
if let Some(mut v)=data.remove("sup_recursion_expr"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sup_recursion_expr".into(),v);}
if let Some(mut v)=data.remove("sup_recursions"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sup_recursions".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_MiniNumerator(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::MiniNumerator@default")?;let mut wire=Map::new();
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("mini_intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("mini_intermediate_exp".into(),v);}
for mut v in data.remove("sup_alpha").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_alpha".into(),v);}
for mut v in data.remove("sup_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_digits".into(),v);}
for mut v in data.remove("sup_operators").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_operators".into(),v);}
for mut v in data.remove("sup_recursion").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sup_recursion".into(),v);}
for mut v in data.remove("sup_recursion_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sup_recursion_expr".into(),v);}
for mut v in data.remove("sup_recursions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sup_recursions".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_MiniSub_json(input:&str)->ContractResult<Umath_Model_MiniSub>{let data=decode_Umath_Model_MiniSub(parse_json(input,"Umath::Model::MiniSub@default")?)?; let model:Umath_Model_MiniSub=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_MiniSub_json(model:&Umath_Model_MiniSub)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_MiniSub(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_MiniSub(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::MiniSub@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "mini_intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "sub", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_alpha") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "sub_alpha", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_alpha".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "sub_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_operators") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "sub_operators", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_operators".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursion") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "sub_recursion", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursion".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursion_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "sub_recursion_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursion_expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "sub_recursions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_alpha") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "sup_alpha", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_alpha".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "sup_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_operators") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "sup_operators", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_operators".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSub@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_MiniSub(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_MiniSub(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("mini_intermediate_exp"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("mini_intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("sub"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sub".into(),v);}
if let Some(mut v)=data.remove("sub_alpha"){
data.insert("sub_alpha".into(),v);}
if let Some(mut v)=data.remove("sub_digits"){
data.insert("sub_digits".into(),v);}
if let Some(mut v)=data.remove("sub_operators"){
data.insert("sub_operators".into(),v);}
if let Some(mut v)=data.remove("sub_recursion"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sub_recursion".into(),v);}
if let Some(mut v)=data.remove("sub_recursion_expr"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sub_recursion_expr".into(),v);}
if let Some(mut v)=data.remove("sub_recursions"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sub_recursions".into(),v);}
if let Some(mut v)=data.remove("sup_alpha"){
data.insert("sup_alpha".into(),v);}
if let Some(mut v)=data.remove("sup_digits"){
data.insert("sup_digits".into(),v);}
if let Some(mut v)=data.remove("sup_operators"){
data.insert("sup_operators".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_MiniSub(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::MiniSub@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("mini_intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("mini_intermediate_exp".into(),v);}
for mut v in data.remove("sub").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sub".into(),v);}
for mut v in data.remove("sub_alpha").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_alpha".into(),v);}
for mut v in data.remove("sub_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_digits".into(),v);}
for mut v in data.remove("sub_operators").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_operators".into(),v);}
for mut v in data.remove("sub_recursion").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sub_recursion".into(),v);}
for mut v in data.remove("sub_recursion_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sub_recursion_expr".into(),v);}
for mut v in data.remove("sub_recursions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sub_recursions".into(),v);}
for mut v in data.remove("sup_alpha").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_alpha".into(),v);}
for mut v in data.remove("sup_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_digits".into(),v);}
for mut v in data.remove("sup_operators").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_operators".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_MiniSubSup_json(input:&str)->ContractResult<Umath_Model_MiniSubSup>{let data=decode_Umath_Model_MiniSubSup(parse_json(input,"Umath::Model::MiniSubSup@default")?)?; let model:Umath_Model_MiniSubSup=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSubSup@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_MiniSubSup_json(model:&Umath_Model_MiniSubSup)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSubSup@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_MiniSubSup(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSubSup@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_MiniSubSup(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::MiniSubSup@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSubSup@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSubSup@default", "sub", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSubSup@default", "sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSubSup@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_MiniSubSup(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_MiniSubSup(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("sub"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sub".into(),v);}
if let Some(mut v)=data.remove("sup"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sup".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_MiniSubSup(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::MiniSubSup@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("sub").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sub".into(),v);}
for mut v in data.remove("sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sup".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_MiniSup_json(input:&str)->ContractResult<Umath_Model_MiniSup>{let data=decode_Umath_Model_MiniSup(parse_json(input,"Umath::Model::MiniSup@default")?)?; let model:Umath_Model_MiniSup=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_MiniSup_json(model:&Umath_Model_MiniSup)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_MiniSup(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_MiniSup(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::MiniSup@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "mini_intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_alpha") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "sub_alpha", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_alpha".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "sub_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_operators") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "sub_operators", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_operators".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_alpha") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "sup_alpha", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_alpha".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "sup_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_operators") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "sup_operators", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_operators".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursion") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "sup_recursion", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursion".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursion_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "sup_recursion_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursion_expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "sup_recursions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursions".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MiniSup@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_MiniSup(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_MiniSup(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("mini_intermediate_exp"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("mini_intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("sub_alpha"){
data.insert("sub_alpha".into(),v);}
if let Some(mut v)=data.remove("sub_digits"){
data.insert("sub_digits".into(),v);}
if let Some(mut v)=data.remove("sub_operators"){
data.insert("sub_operators".into(),v);}
if let Some(mut v)=data.remove("sup"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sup".into(),v);}
if let Some(mut v)=data.remove("sup_alpha"){
data.insert("sup_alpha".into(),v);}
if let Some(mut v)=data.remove("sup_digits"){
data.insert("sup_digits".into(),v);}
if let Some(mut v)=data.remove("sup_operators"){
data.insert("sup_operators".into(),v);}
if let Some(mut v)=data.remove("sup_recursion"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sup_recursion".into(),v);}
if let Some(mut v)=data.remove("sup_recursion_expr"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sup_recursion_expr".into(),v);}
if let Some(mut v)=data.remove("sup_recursions"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sup_recursions".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_MiniSup(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::MiniSup@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("mini_intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("mini_intermediate_exp".into(),v);}
for mut v in data.remove("sub_alpha").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_alpha".into(),v);}
for mut v in data.remove("sub_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_digits".into(),v);}
for mut v in data.remove("sub_operators").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_operators".into(),v);}
for mut v in data.remove("sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sup".into(),v);}
for mut v in data.remove("sup_alpha").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_alpha".into(),v);}
for mut v in data.remove("sup_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_digits".into(),v);}
for mut v in data.remove("sup_operators").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_operators".into(),v);}
for mut v in data.remove("sup_recursion").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sup_recursion".into(),v);}
for mut v in data.remove("sup_recursion_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sup_recursion_expr".into(),v);}
for mut v in data.remove("sup_recursions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sup_recursions".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_MitBbb_json(input:&str)->ContractResult<Umath_Model_MitBbb>{let data=decode_Umath_Model_MitBbb(parse_json(input,"Umath::Model::MitBbb@default")?)?; let model:Umath_Model_MitBbb=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MitBbb@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_MitBbb_json(model:&Umath_Model_MitBbb)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MitBbb@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_MitBbb(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::MitBbb@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_MitBbb(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::MitBbb@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("present") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MitBbb@default", "present", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("present".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::MitBbb@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_MitBbb(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_MitBbb(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("present"){
data.insert("present".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_MitBbb(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::MitBbb@default")?;let mut wire=Map::new();
for mut v in data.remove("present").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("present".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Monospace_json(input:&str)->ContractResult<Umath_Model_Monospace>{let data=decode_Umath_Model_Monospace(parse_json(input,"Umath::Model::Monospace@default")?)?; let model:Umath_Model_Monospace=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Monospace@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Monospace_json(model:&Umath_Model_Monospace)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Monospace@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Monospace(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Monospace@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Monospace(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Monospace@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("monospace_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Monospace@default", "monospace_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("monospace_value".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Monospace@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Monospace(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Monospace(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("monospace_value"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("monospace_value".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Monospace(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Monospace@default")?;let mut wire=Map::new();
for mut v in data.remove("monospace_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("monospace_value".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Nary_json(input:&str)->ContractResult<Umath_Model_Nary>{let data=decode_Umath_Model_Nary(parse_json(input,"Umath::Model::Nary@default")?)?; let model:Umath_Model_Nary=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Nary@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Nary_json(model:&Umath_Model_Nary)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Nary@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Nary(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Nary@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Nary(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Nary@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("nary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Nary@default", "nary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("nary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("nary_sub_sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Nary@default", "nary_sub_sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("nary_sub_sup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("naryand") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Nary@default", "naryand", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("naryand".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Nary@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Nary(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Nary(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("nary_class"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("nary_class".into(),v);}
if let Some(mut v)=data.remove("nary_sub_sup"){
if !v.is_null() {
v=decode_Umath_Model_NarySubSup(v)?;
}
data.insert("nary_sub_sup".into(),v);}
if let Some(mut v)=data.remove("naryand"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("naryand".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Nary(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Nary@default")?;let mut wire=Map::new();
for mut v in data.remove("nary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("nary_class".into(),v);}
for mut v in data.remove("nary_sub_sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_NarySubSup(v)?;
}
wire.insert("nary_sub_sup".into(),v);}
for mut v in data.remove("naryand").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("naryand".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_NarySubSup_json(input:&str)->ContractResult<Umath_Model_NarySubSup>{let data=decode_Umath_Model_NarySubSup(parse_json(input,"Umath::Model::NarySubSup@default")?)?; let model:Umath_Model_NarySubSup=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::NarySubSup@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_NarySubSup_json(model:&Umath_Model_NarySubSup)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::NarySubSup@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_NarySubSup(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::NarySubSup@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_NarySubSup(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::NarySubSup@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("sub_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::NarySubSup@default", "sub_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subsup_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::NarySubSup@default", "subsup_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subsup_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::NarySubSup@default", "sup_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_exp".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::NarySubSup@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_NarySubSup(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_NarySubSup(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("sub_exp"){
if !v.is_null() {
v=decode_Umath_Model_SubExp(v)?;
}
data.insert("sub_exp".into(),v);}
if let Some(mut v)=data.remove("subsup_exp"){
if !v.is_null() {
v=decode_Umath_Model_SubsupExp(v)?;
}
data.insert("subsup_exp".into(),v);}
if let Some(mut v)=data.remove("sup_exp"){
if !v.is_null() {
v=decode_Umath_Model_SupExp(v)?;
}
data.insert("sup_exp".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_NarySubSup(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::NarySubSup@default")?;let mut wire=Map::new();
for mut v in data.remove("sub_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SubExp(v)?;
}
wire.insert("sub_exp".into(),v);}
for mut v in data.remove("subsup_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SubsupExp(v)?;
}
wire.insert("subsup_exp".into(),v);}
for mut v in data.remove("sup_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SupExp(v)?;
}
wire.insert("sup_exp".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_OpenParenMask_json(input:&str)->ContractResult<Umath_Model_OpenParenMask>{let data=decode_Umath_Model_OpenParenMask(parse_json(input,"Umath::Model::OpenParenMask@default")?)?; let model:Umath_Model_OpenParenMask=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::OpenParenMask@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_OpenParenMask_json(model:&Umath_Model_OpenParenMask)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::OpenParenMask@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_OpenParenMask(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::OpenParenMask@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_OpenParenMask(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::OpenParenMask@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::OpenParenMask@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::OpenParenMask@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_OpenParenMask(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_OpenParenMask(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_OpenParenMask(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::OpenParenMask@default")?;let mut wire=Map::new();
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Opener_json(input:&str)->ContractResult<Umath_Model_Opener>{let data=decode_Umath_Model_Opener(parse_json(input,"Umath::Model::Opener@default")?)?; let model:Umath_Model_Opener=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Opener@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Opener_json(model:&Umath_Model_Opener)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Opener@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Opener(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Opener@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Opener(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Opener@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("open_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Opener@default", "open_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("open_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("open_paren_mask") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Opener@default", "open_paren_mask", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("open_paren_mask".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("paren_open_prefix") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Opener@default", "paren_open_prefix", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("paren_open_prefix".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Opener@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Opener(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Opener(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("open_paren"){
if !v.is_null() {
v=decode_Umath_Model_Paren(v)?;
}
data.insert("open_paren".into(),v);}
if let Some(mut v)=data.remove("open_paren_mask"){
if !v.is_null() {
v=decode_Umath_Model_OpenParenMask(v)?;
}
data.insert("open_paren_mask".into(),v);}
if let Some(mut v)=data.remove("paren_open_prefix"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("paren_open_prefix".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Opener(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Opener@default")?;let mut wire=Map::new();
for mut v in data.remove("open_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Paren(v)?;
}
wire.insert("open_paren".into(),v);}
for mut v in data.remove("open_paren_mask").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_OpenParenMask(v)?;
}
wire.insert("open_paren_mask".into(),v);}
for mut v in data.remove("paren_open_prefix").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("paren_open_prefix".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Operand_json(input:&str)->ContractResult<Umath_Model_Operand>{let data=decode_Umath_Model_Operand(parse_json(input,"Umath::Model::Operand@default")?)?; let model:Umath_Model_Operand=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Operand_json(model:&Umath_Model_Operand)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Operand(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Operand(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Operand@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("accents") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "accents", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("accents".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_negated_operator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "binary_negated_operator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_negated_operator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("factor") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "factor", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("factor".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("infty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "infty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("infty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("negated_operator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "negated_operator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("negated_operator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operand") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "operand", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operand".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "operator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ordinary_negated_operator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "ordinary_negated_operator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ordinary_negated_operator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("phantom") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "phantom", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("phantom".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("rect") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "rect", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("rect".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Operand@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Operand(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Operand(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("accents"){
if !v.is_null() {
v=decode_Umath_Model_Accents(v)?;
}
data.insert("accents".into(),v);}
if let Some(mut v)=data.remove("binary_negated_operator"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("binary_negated_operator".into(),v);}
if let Some(mut v)=data.remove("factor"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("factor".into(),v);}
if let Some(mut v)=data.remove("fonts"){
if !v.is_null() {
v=decode_Umath_Model_Fonts(v)?;
}
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("infty"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("infty".into(),v);}
if let Some(mut v)=data.remove("negated_operator"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("negated_operator".into(),v);}
if let Some(mut v)=data.remove("operand"){
if !v.is_null() {
v=decode_Umath_Model_Operand(v)?;
}
data.insert("operand".into(),v);}
if let Some(mut v)=data.remove("operator"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("operator".into(),v);}
if let Some(mut v)=data.remove("ordinary_negated_operator"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("ordinary_negated_operator".into(),v);}
if let Some(mut v)=data.remove("phantom"){
if !v.is_null() {
v=decode_Umath_Model_Phantom(v)?;
}
data.insert("phantom".into(),v);}
if let Some(mut v)=data.remove("rect"){
if !v.is_null() {
v=decode_Umath_Model_Rect(v)?;
}
data.insert("rect".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Umath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Operand(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Operand@default")?;let mut wire=Map::new();
for mut v in data.remove("accents").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Accents(v)?;
}
wire.insert("accents".into(),v);}
for mut v in data.remove("binary_negated_operator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("binary_negated_operator".into(),v);}
for mut v in data.remove("factor").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("factor".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Fonts(v)?;
}
wire.insert("fonts".into(),v);}
for mut v in data.remove("infty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("infty".into(),v);}
for mut v in data.remove("negated_operator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("negated_operator".into(),v);}
for mut v in data.remove("operand").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Operand(v)?;
}
wire.insert("operand".into(),v);}
for mut v in data.remove("operator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("operator".into(),v);}
for mut v in data.remove("ordinary_negated_operator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("ordinary_negated_operator".into(),v);}
for mut v in data.remove("phantom").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Phantom(v)?;
}
wire.insert("phantom".into(),v);}
for mut v in data.remove("rect").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Rect(v)?;
}
wire.insert("rect".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Other_json(input:&str)->ContractResult<Umath_Model_Other>{let data=decode_Umath_Model_Other(parse_json(input,"Umath::Model::Other@default")?)?; let model:Umath_Model_Other=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Other@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Other_json(model:&Umath_Model_Other)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Other@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Other(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Other@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Other(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Other@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Other@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Other@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Other(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Other(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Umath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Other(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Other@default")?;let mut wire=Map::new();
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_OverlayAfter_json(input:&str)->ContractResult<Umath_Model_OverlayAfter>{let data=decode_Umath_Model_OverlayAfter(parse_json(input,"Umath::Model::OverlayAfter@default")?)?; let model:Umath_Model_OverlayAfter=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::OverlayAfter@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_OverlayAfter_json(model:&Umath_Model_OverlayAfter)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::OverlayAfter@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_OverlayAfter(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::OverlayAfter@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_OverlayAfter(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::OverlayAfter@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("diacritic_overlays") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::OverlayAfter@default", "diacritic_overlays", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("diacritic_overlays".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::OverlayAfter@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_OverlayAfter(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_OverlayAfter(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("diacritic_overlays"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("diacritic_overlays".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_OverlayAfter(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::OverlayAfter@default")?;let mut wire=Map::new();
for mut v in data.remove("diacritic_overlays").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("diacritic_overlays".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_OverlayBefore_json(input:&str)->ContractResult<Umath_Model_OverlayBefore>{let data=decode_Umath_Model_OverlayBefore(parse_json(input,"Umath::Model::OverlayBefore@default")?)?; let model:Umath_Model_OverlayBefore=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::OverlayBefore@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_OverlayBefore_json(model:&Umath_Model_OverlayBefore)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::OverlayBefore@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_OverlayBefore(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::OverlayBefore@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_OverlayBefore(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::OverlayBefore@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("diacritic_overlays") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::OverlayBefore@default", "diacritic_overlays", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("diacritic_overlays".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::OverlayBefore@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_OverlayBefore(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_OverlayBefore(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("diacritic_overlays"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("diacritic_overlays".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_OverlayBefore(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::OverlayBefore@default")?;let mut wire=Map::new();
for mut v in data.remove("diacritic_overlays").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("diacritic_overlays".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Paren_json(input:&str)->ContractResult<Umath_Model_Paren>{let data=decode_Umath_Model_Paren(parse_json(input,"Umath::Model::Paren@default")?)?; let model:Umath_Model_Paren=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Paren@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Paren_json(model:&Umath_Model_Paren)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Paren@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Paren(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Paren@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Paren(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Paren@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("close_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Paren@default", "close_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("close_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("close_paren_mask") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Paren@default", "close_paren_mask", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("close_paren_mask".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("open_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Paren@default", "open_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("open_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("open_paren_mask") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Paren@default", "open_paren_mask", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("open_paren_mask".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("paren_close_prefix") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Paren@default", "paren_close_prefix", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("paren_close_prefix".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("paren_open_prefix") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Paren@default", "paren_open_prefix", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("paren_open_prefix".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("spelling") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Paren@default", "spelling", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("spelling".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Paren@default", "value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("kind") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Paren@default", "kind", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("kind".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Paren@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Paren(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Paren(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("close_paren"){
if !v.is_null() {
v=decode_Umath_Model_Paren(v)?;
}
data.insert("close_paren".into(),v);}
if let Some(mut v)=data.remove("close_paren_mask"){
if !v.is_null() {
v=decode_Umath_Model_CloseParenMask(v)?;
}
data.insert("close_paren_mask".into(),v);}
if let Some(mut v)=data.remove("open_paren"){
if !v.is_null() {
v=decode_Umath_Model_Paren(v)?;
}
data.insert("open_paren".into(),v);}
if let Some(mut v)=data.remove("open_paren_mask"){
if !v.is_null() {
v=decode_Umath_Model_OpenParenMask(v)?;
}
data.insert("open_paren_mask".into(),v);}
if let Some(mut v)=data.remove("paren_close_prefix"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("paren_close_prefix".into(),v);}
if let Some(mut v)=data.remove("paren_open_prefix"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("paren_open_prefix".into(),v);}
if let Some(mut v)=data.remove("spelling"){
data.insert("spelling".into(),v);}
if let Some(mut v)=data.remove("value"){
data.insert("value".into(),v);}
if let Some(mut v)=data.remove("kind"){
data.insert("kind".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Paren(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Paren@default")?;let mut wire=Map::new();
for mut v in data.remove("close_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Paren(v)?;
}
wire.insert("close_paren".into(),v);}
for mut v in data.remove("close_paren_mask").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_CloseParenMask(v)?;
}
wire.insert("close_paren_mask".into(),v);}
for mut v in data.remove("open_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Paren(v)?;
}
wire.insert("open_paren".into(),v);}
for mut v in data.remove("open_paren_mask").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_OpenParenMask(v)?;
}
wire.insert("open_paren_mask".into(),v);}
for mut v in data.remove("paren_close_prefix").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("paren_close_prefix".into(),v);}
for mut v in data.remove("paren_open_prefix").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("paren_open_prefix".into(),v);}
for mut v in data.remove("spelling").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("spelling".into(),v);}
for mut v in data.remove("value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("value".into(),v);}
for mut v in data.remove("kind").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("kind".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Phantom_json(input:&str)->ContractResult<Umath_Model_Phantom>{let data=decode_Umath_Model_Phantom(parse_json(input,"Umath::Model::Phantom@default")?)?; let model:Umath_Model_Phantom=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Phantom@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Phantom_json(model:&Umath_Model_Phantom)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Phantom@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Phantom(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Phantom@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Phantom(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Phantom@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Phantom@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("phantom_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Phantom@default", "phantom_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("phantom_value".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Phantom@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Phantom(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Phantom(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("phantom_value"){
data.insert("phantom_value".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Phantom(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Phantom@default")?;let mut wire=Map::new();
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("phantom_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("phantom_value".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_PreOverrideSubsup_json(input:&str)->ContractResult<Umath_Model_PreOverrideSubsup>{let data=decode_Umath_Model_PreOverrideSubsup(parse_json(input,"Umath::Model::PreOverrideSubsup@default")?)?; let model:Umath_Model_PreOverrideSubsup=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::PreOverrideSubsup@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_PreOverrideSubsup_json(model:&Umath_Model_PreOverrideSubsup)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::PreOverrideSubsup@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_PreOverrideSubsup(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::PreOverrideSubsup@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_PreOverrideSubsup(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::PreOverrideSubsup@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::PreOverrideSubsup@default", "over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("pre_subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::PreOverrideSubsup@default", "pre_subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("pre_subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("pre_supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::PreOverrideSubsup@default", "pre_supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("pre_supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("size_overrides") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::PreOverrideSubsup@default", "size_overrides", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("size_overrides".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("under") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::PreOverrideSubsup@default", "under", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("under".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::PreOverrideSubsup@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_PreOverrideSubsup(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_PreOverrideSubsup(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("over"){
data.insert("over".into(),v);}
if let Some(mut v)=data.remove("pre_subscript"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("pre_subscript".into(),v);}
if let Some(mut v)=data.remove("pre_supscript"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("pre_supscript".into(),v);}
if let Some(mut v)=data.remove("size_overrides"){
data.insert("size_overrides".into(),v);}
if let Some(mut v)=data.remove("under"){
data.insert("under".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_PreOverrideSubsup(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::PreOverrideSubsup@default")?;let mut wire=Map::new();
for mut v in data.remove("over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("over".into(),v);}
for mut v in data.remove("pre_subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("pre_subscript".into(),v);}
for mut v in data.remove("pre_supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("pre_supscript".into(),v);}
for mut v in data.remove("size_overrides").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("size_overrides".into(),v);}
for mut v in data.remove("under").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("under".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_PrimeAccentSymbolsItem_json(input:&str)->ContractResult<Umath_Model_PrimeAccentSymbolsItem>{let data=decode_Umath_Model_PrimeAccentSymbolsItem(parse_json(input,"Umath::Model::PrimeAccentSymbolsItem@default")?)?; let model:Umath_Model_PrimeAccentSymbolsItem=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::PrimeAccentSymbolsItem@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_PrimeAccentSymbolsItem_json(model:&Umath_Model_PrimeAccentSymbolsItem)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::PrimeAccentSymbolsItem@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_PrimeAccentSymbolsItem(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::PrimeAccentSymbolsItem@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_PrimeAccentSymbolsItem(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::PrimeAccentSymbolsItem@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("prefixed_prime") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::PrimeAccentSymbolsItem@default", "prefixed_prime", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("prefixed_prime".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("spelling") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::PrimeAccentSymbolsItem@default", "spelling", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("spelling".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::PrimeAccentSymbolsItem@default", "value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("kind") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::PrimeAccentSymbolsItem@default", "kind", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("kind".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::PrimeAccentSymbolsItem@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_PrimeAccentSymbolsItem(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_PrimeAccentSymbolsItem(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("prefixed_prime"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("prefixed_prime".into(),v);}
if let Some(mut v)=data.remove("spelling"){
data.insert("spelling".into(),v);}
if let Some(mut v)=data.remove("value"){
data.insert("value".into(),v);}
if let Some(mut v)=data.remove("kind"){
data.insert("kind".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_PrimeAccentSymbolsItem(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::PrimeAccentSymbolsItem@default")?;let mut wire=Map::new();
for mut v in data.remove("prefixed_prime").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("prefixed_prime".into(),v);}
for mut v in data.remove("spelling").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("spelling".into(),v);}
for mut v in data.remove("value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("value".into(),v);}
for mut v in data.remove("kind").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("kind".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Rect_json(input:&str)->ContractResult<Umath_Model_Rect>{let data=decode_Umath_Model_Rect(parse_json(input,"Umath::Model::Rect@default")?)?; let model:Umath_Model_Rect=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Rect@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Rect_json(model:&Umath_Model_Rect)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Rect@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Rect(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Rect@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Rect(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Rect@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Rect@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("rect_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Rect@default", "rect_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("rect_value".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Rect@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Rect(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Rect(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("rect_value"){
data.insert("rect_value".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Rect(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Rect@default")?;let mut wire=Map::new();
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("rect_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("rect_value".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_RightValue_json(input:&str)->ContractResult<Umath_Model_RightValue>{let data=decode_Umath_Model_RightValue(parse_json(input,"Umath::Model::RightValue@default")?)?; let model:Umath_Model_RightValue=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::RightValue@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_RightValue_json(model:&Umath_Model_RightValue)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::RightValue@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_RightValue(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::RightValue@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_RightValue(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::RightValue@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("infty") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::RightValue@default", "infty", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("infty".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("list") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::RightValue@default", "list", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("list".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("negative") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::RightValue@default", "negative", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("negative".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::RightValue@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("positive") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::RightValue@default", "positive", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("positive".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::RightValue@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_RightValue(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_RightValue(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("infty"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("infty".into(),v);}
if let Some(mut v)=data.remove("list"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Umath::Model::RightValue@default", "list", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Umath_Model_ListItem(v)?;
*slot=v; }
}
data.insert("list".into(),v);}
if let Some(mut v)=data.remove("negative"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("negative".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("positive"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("positive".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_RightValue(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::RightValue@default")?;let mut wire=Map::new();
for mut v in data.remove("infty").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("infty".into(),v);}
for mut v in data.remove("list").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Umath::Model::RightValue@default", "list", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Umath_Model_ListItem(v)?;
*slot=v; }
}
wire.insert("list".into(),v);}
for mut v in data.remove("negative").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("negative".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("positive").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("positive".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_RootFirstValue_json(input:&str)->ContractResult<Umath_Model_RootFirstValue>{let data=decode_Umath_Model_RootFirstValue(parse_json(input,"Umath::Model::RootFirstValue@default")?)?; let model:Umath_Model_RootFirstValue=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::RootFirstValue@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_RootFirstValue_json(model:&Umath_Model_RootFirstValue)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::RootFirstValue@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_RootFirstValue(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::RootFirstValue@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_RootFirstValue(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::RootFirstValue@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::RootFirstValue@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("func_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::RootFirstValue@default", "func_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("func_expr".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::RootFirstValue@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_RootFirstValue(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_RootFirstValue(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("func_expr"){
if !v.is_null() {
v=decode_Umath_Model_FuncExpr(v)?;
}
data.insert("func_expr".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_RootFirstValue(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::RootFirstValue@default")?;let mut wire=Map::new();
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("func_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_FuncExpr(v)?;
}
wire.insert("func_expr".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_RootNode_json(input:&str)->ContractResult<Umath_Model_RootNode>{let data=decode_Umath_Model_RootNode(parse_json(input,"Umath::Model::RootNode@default")?)?; let model:Umath_Model_RootNode=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::RootNode@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_RootNode_json(model:&Umath_Model_RootNode)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::RootNode@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_RootNode(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::RootNode@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_RootNode(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::RootNode@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::RootNode@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("root_symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::RootNode@default", "root_symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("root_symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::RootNode@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::RootNode@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_RootNode(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_RootNode(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("root_symbol"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("root_symbol".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Umath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_RootNode(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::RootNode@default")?;let mut wire=Map::new();
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("root_symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("root_symbol".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_RootSecondValue_json(input:&str)->ContractResult<Umath_Model_RootSecondValue>{let data=decode_Umath_Model_RootSecondValue(parse_json(input,"Umath::Model::RootSecondValue@default")?)?; let model:Umath_Model_RootSecondValue=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::RootSecondValue@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_RootSecondValue_json(model:&Umath_Model_RootSecondValue)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::RootSecondValue@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_RootSecondValue(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::RootSecondValue@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_RootSecondValue(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::RootSecondValue@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::RootSecondValue@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("func_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::RootSecondValue@default", "func_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("func_expr".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::RootSecondValue@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_RootSecondValue(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_RootSecondValue(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("func_expr"){
if !v.is_null() {
v=decode_Umath_Model_FuncExpr(v)?;
}
data.insert("func_expr".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_RootSecondValue(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::RootSecondValue@default")?;let mut wire=Map::new();
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("func_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_FuncExpr(v)?;
}
wire.insert("func_expr".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_SecondValue_json(input:&str)->ContractResult<Umath_Model_SecondValue>{let data=decode_Umath_Model_SecondValue(parse_json(input,"Umath::Model::SecondValue@default")?)?; let model:Umath_Model_SecondValue=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SecondValue@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_SecondValue_json(model:&Umath_Model_SecondValue)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SecondValue@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_SecondValue(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SecondValue@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_SecondValue(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::SecondValue@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SecondValue@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("func_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SecondValue@default", "func_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("func_expr".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SecondValue@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_SecondValue(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_SecondValue(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("func_expr"){
if !v.is_null() {
v=decode_Umath_Model_FuncExpr(v)?;
}
data.insert("func_expr".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_SecondValue(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::SecondValue@default")?;let mut wire=Map::new();
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("func_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_FuncExpr(v)?;
}
wire.insert("func_expr".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_SlashedValue_json(input:&str)->ContractResult<Umath_Model_SlashedValue>{let data=decode_Umath_Model_SlashedValue(parse_json(input,"Umath::Model::SlashedValue@default")?)?; let model:Umath_Model_SlashedValue=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SlashedValue@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_SlashedValue_json(model:&Umath_Model_SlashedValue)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SlashedValue@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_SlashedValue(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SlashedValue@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_SlashedValue(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::SlashedValue@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("list") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SlashedValue@default", "list", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("list".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SlashedValue@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SlashedValue@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_SlashedValue(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_SlashedValue(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("list"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Umath::Model::SlashedValue@default", "list", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Umath_Model_ListItem(v)?;
*slot=v; }
}
data.insert("list".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_SlashedValue(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::SlashedValue@default")?;let mut wire=Map::new();
for mut v in data.remove("list").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Umath::Model::SlashedValue@default", "list", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Umath_Model_ListItem(v)?;
*slot=v; }
}
wire.insert("list".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_SubExp_json(input:&str)->ContractResult<Umath_Model_SubExp>{let data=decode_Umath_Model_SubExp(parse_json(input,"Umath::Model::SubExp@default")?)?; let model:Umath_Model_SubExp=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_SubExp_json(model:&Umath_Model_SubExp)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_SubExp(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_SubExp(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::SubExp@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("exp_iteration") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "exp_iteration", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("exp_iteration".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mask") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "mask", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mask".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "mini_intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_sub") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "mini_sub", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_sub".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_sub_sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "mini_sub_sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_sub_sup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "mini_sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_sup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("nary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "nary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("nary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "sub", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_alpha") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "sub_alpha", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_alpha".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "sub_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_operators") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "sub_operators", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_operators".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursion") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "sub_recursion", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursion".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursion_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "sub_recursion_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursion_expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "sub_recursions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursions".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubExp@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_SubExp(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_SubExp(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("exp_iteration"){
if !v.is_null() {
v=decode_Umath_Model_ExpIteration(v)?;
}
data.insert("exp_iteration".into(),v);}
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("mask"){
if !v.is_null() {
v=decode_Umath_Model_Mask(v)?;
}
data.insert("mask".into(),v);}
if let Some(mut v)=data.remove("mini_intermediate_exp"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("mini_intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("mini_sub"){
if !v.is_null() {
v=decode_Umath_Model_MiniSub(v)?;
}
data.insert("mini_sub".into(),v);}
if let Some(mut v)=data.remove("mini_sub_sup"){
if !v.is_null() {
v=decode_Umath_Model_MiniSubSup(v)?;
}
data.insert("mini_sub_sup".into(),v);}
if let Some(mut v)=data.remove("mini_sup"){
if !v.is_null() {
v=decode_Umath_Model_MiniSup(v)?;
}
data.insert("mini_sup".into(),v);}
if let Some(mut v)=data.remove("nary_class"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("nary_class".into(),v);}
if let Some(mut v)=data.remove("sub"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sub".into(),v);}
if let Some(mut v)=data.remove("sub_alpha"){
data.insert("sub_alpha".into(),v);}
if let Some(mut v)=data.remove("sub_digits"){
data.insert("sub_digits".into(),v);}
if let Some(mut v)=data.remove("sub_operators"){
data.insert("sub_operators".into(),v);}
if let Some(mut v)=data.remove("sub_recursion"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sub_recursion".into(),v);}
if let Some(mut v)=data.remove("sub_recursion_expr"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sub_recursion_expr".into(),v);}
if let Some(mut v)=data.remove("sub_recursions"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sub_recursions".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_SubExp(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::SubExp@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("exp_iteration").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_ExpIteration(v)?;
}
wire.insert("exp_iteration".into(),v);}
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("mask").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Mask(v)?;
}
wire.insert("mask".into(),v);}
for mut v in data.remove("mini_intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("mini_intermediate_exp".into(),v);}
for mut v in data.remove("mini_sub").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniSub(v)?;
}
wire.insert("mini_sub".into(),v);}
for mut v in data.remove("mini_sub_sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniSubSup(v)?;
}
wire.insert("mini_sub_sup".into(),v);}
for mut v in data.remove("mini_sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniSup(v)?;
}
wire.insert("mini_sup".into(),v);}
for mut v in data.remove("nary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("nary_class".into(),v);}
for mut v in data.remove("sub").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sub".into(),v);}
for mut v in data.remove("sub_alpha").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_alpha".into(),v);}
for mut v in data.remove("sub_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_digits".into(),v);}
for mut v in data.remove("sub_operators").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_operators".into(),v);}
for mut v in data.remove("sub_recursion").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sub_recursion".into(),v);}
for mut v in data.remove("sub_recursion_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sub_recursion_expr".into(),v);}
for mut v in data.remove("sub_recursions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sub_recursions".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_SubParen_json(input:&str)->ContractResult<Umath_Model_SubParen>{let data=decode_Umath_Model_SubParen(parse_json(input,"Umath::Model::SubParen@default")?)?; let model:Umath_Model_SubParen=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SubParen@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_SubParen_json(model:&Umath_Model_SubParen)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SubParen@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_SubParen(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SubParen@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_SubParen(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::SubParen@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubParen@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubParen@default", "mini_intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_alpha") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubParen@default", "sub_alpha", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_alpha".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubParen@default", "sub_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_operators") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubParen@default", "sub_operators", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_operators".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursion") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubParen@default", "sub_recursion", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursion".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursion_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubParen@default", "sub_recursion_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursion_expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_recursions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubParen@default", "sub_recursions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_recursions".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubParen@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_SubParen(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_SubParen(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("mini_intermediate_exp"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("mini_intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("sub_alpha"){
data.insert("sub_alpha".into(),v);}
if let Some(mut v)=data.remove("sub_digits"){
data.insert("sub_digits".into(),v);}
if let Some(mut v)=data.remove("sub_operators"){
data.insert("sub_operators".into(),v);}
if let Some(mut v)=data.remove("sub_recursion"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sub_recursion".into(),v);}
if let Some(mut v)=data.remove("sub_recursion_expr"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sub_recursion_expr".into(),v);}
if let Some(mut v)=data.remove("sub_recursions"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sub_recursions".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_SubParen(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::SubParen@default")?;let mut wire=Map::new();
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("mini_intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("mini_intermediate_exp".into(),v);}
for mut v in data.remove("sub_alpha").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_alpha".into(),v);}
for mut v in data.remove("sub_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_digits".into(),v);}
for mut v in data.remove("sub_operators").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sub_operators".into(),v);}
for mut v in data.remove("sub_recursion").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sub_recursion".into(),v);}
for mut v in data.remove("sub_recursion_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sub_recursion_expr".into(),v);}
for mut v in data.remove("sub_recursions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sub_recursions".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_SubsupExp_json(input:&str)->ContractResult<Umath_Model_SubsupExp>{let data=decode_Umath_Model_SubsupExp(parse_json(input,"Umath::Model::SubsupExp@default")?)?; let model:Umath_Model_SubsupExp=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SubsupExp@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_SubsupExp_json(model:&Umath_Model_SubsupExp)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SubsupExp@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_SubsupExp(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SubsupExp@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_SubsupExp(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::SubsupExp@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubsupExp@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mask") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubsupExp@default", "mask", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mask".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("nary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubsupExp@default", "nary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("nary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubsupExp@default", "sub", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubsupExp@default", "sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SubsupExp@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_SubsupExp(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_SubsupExp(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("mask"){
if !v.is_null() {
v=decode_Umath_Model_Mask(v)?;
}
data.insert("mask".into(),v);}
if let Some(mut v)=data.remove("nary_class"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("nary_class".into(),v);}
if let Some(mut v)=data.remove("sub"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sub".into(),v);}
if let Some(mut v)=data.remove("sup"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sup".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_SubsupExp(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::SubsupExp@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("mask").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Mask(v)?;
}
wire.insert("mask".into(),v);}
for mut v in data.remove("nary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("nary_class".into(),v);}
for mut v in data.remove("sub").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sub".into(),v);}
for mut v in data.remove("sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sup".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_SupExp_json(input:&str)->ContractResult<Umath_Model_SupExp>{let data=decode_Umath_Model_SupExp(parse_json(input,"Umath::Model::SupExp@default")?)?; let model:Umath_Model_SupExp=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_SupExp_json(model:&Umath_Model_SupExp)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_SupExp(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_SupExp(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::SupExp@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("exp_iteration") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "exp_iteration", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("exp_iteration".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mask") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "mask", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mask".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "mini_intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_sub") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "mini_sub", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_sub".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_sub_sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "mini_sub_sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_sub_sup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "mini_sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_sup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("nary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "nary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("nary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_alpha") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "sup_alpha", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_alpha".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "sup_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_operators") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "sup_operators", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_operators".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursion") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "sup_recursion", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursion".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursion_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "sup_recursion_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursion_expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "sup_recursions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursions".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupExp@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_SupExp(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_SupExp(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("exp_iteration"){
if !v.is_null() {
v=decode_Umath_Model_ExpIteration(v)?;
}
data.insert("exp_iteration".into(),v);}
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("mask"){
if !v.is_null() {
v=decode_Umath_Model_Mask(v)?;
}
data.insert("mask".into(),v);}
if let Some(mut v)=data.remove("mini_intermediate_exp"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("mini_intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("mini_sub"){
if !v.is_null() {
v=decode_Umath_Model_MiniSub(v)?;
}
data.insert("mini_sub".into(),v);}
if let Some(mut v)=data.remove("mini_sub_sup"){
if !v.is_null() {
v=decode_Umath_Model_MiniSubSup(v)?;
}
data.insert("mini_sub_sup".into(),v);}
if let Some(mut v)=data.remove("mini_sup"){
if !v.is_null() {
v=decode_Umath_Model_MiniSup(v)?;
}
data.insert("mini_sup".into(),v);}
if let Some(mut v)=data.remove("nary_class"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("nary_class".into(),v);}
if let Some(mut v)=data.remove("sup"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sup".into(),v);}
if let Some(mut v)=data.remove("sup_alpha"){
data.insert("sup_alpha".into(),v);}
if let Some(mut v)=data.remove("sup_digits"){
data.insert("sup_digits".into(),v);}
if let Some(mut v)=data.remove("sup_operators"){
data.insert("sup_operators".into(),v);}
if let Some(mut v)=data.remove("sup_recursion"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sup_recursion".into(),v);}
if let Some(mut v)=data.remove("sup_recursion_expr"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sup_recursion_expr".into(),v);}
if let Some(mut v)=data.remove("sup_recursions"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sup_recursions".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_SupExp(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::SupExp@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("exp_iteration").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_ExpIteration(v)?;
}
wire.insert("exp_iteration".into(),v);}
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("mask").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Mask(v)?;
}
wire.insert("mask".into(),v);}
for mut v in data.remove("mini_intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("mini_intermediate_exp".into(),v);}
for mut v in data.remove("mini_sub").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniSub(v)?;
}
wire.insert("mini_sub".into(),v);}
for mut v in data.remove("mini_sub_sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniSubSup(v)?;
}
wire.insert("mini_sub_sup".into(),v);}
for mut v in data.remove("mini_sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniSup(v)?;
}
wire.insert("mini_sup".into(),v);}
for mut v in data.remove("nary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("nary_class".into(),v);}
for mut v in data.remove("sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sup".into(),v);}
for mut v in data.remove("sup_alpha").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_alpha".into(),v);}
for mut v in data.remove("sup_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_digits".into(),v);}
for mut v in data.remove("sup_operators").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_operators".into(),v);}
for mut v in data.remove("sup_recursion").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sup_recursion".into(),v);}
for mut v in data.remove("sup_recursion_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sup_recursion_expr".into(),v);}
for mut v in data.remove("sup_recursions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sup_recursions".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_SupParen_json(input:&str)->ContractResult<Umath_Model_SupParen>{let data=decode_Umath_Model_SupParen(parse_json(input,"Umath::Model::SupParen@default")?)?; let model:Umath_Model_SupParen=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SupParen@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_SupParen_json(model:&Umath_Model_SupParen)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SupParen@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_SupParen(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SupParen@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_SupParen(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::SupParen@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupParen@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupParen@default", "mini_intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_alpha") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupParen@default", "sup_alpha", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_alpha".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_digits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupParen@default", "sup_digits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_digits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_operators") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupParen@default", "sup_operators", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_operators".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursion") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupParen@default", "sup_recursion", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursion".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursion_expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupParen@default", "sup_recursion_expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursion_expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_recursions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupParen@default", "sup_recursions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_recursions".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SupParen@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_SupParen(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_SupParen(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("mini_intermediate_exp"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("mini_intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("sup_alpha"){
data.insert("sup_alpha".into(),v);}
if let Some(mut v)=data.remove("sup_digits"){
data.insert("sup_digits".into(),v);}
if let Some(mut v)=data.remove("sup_operators"){
data.insert("sup_operators".into(),v);}
if let Some(mut v)=data.remove("sup_recursion"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("sup_recursion".into(),v);}
if let Some(mut v)=data.remove("sup_recursion_expr"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sup_recursion_expr".into(),v);}
if let Some(mut v)=data.remove("sup_recursions"){
if !v.is_null() {
v=decode_Umath_Model_MiniExpression(v)?;
}
data.insert("sup_recursions".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_SupParen(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::SupParen@default")?;let mut wire=Map::new();
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("mini_intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("mini_intermediate_exp".into(),v);}
for mut v in data.remove("sup_alpha").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_alpha".into(),v);}
for mut v in data.remove("sup_digits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_digits".into(),v);}
for mut v in data.remove("sup_operators").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sup_operators".into(),v);}
for mut v in data.remove("sup_recursion").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("sup_recursion".into(),v);}
for mut v in data.remove("sup_recursion_expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sup_recursion_expr".into(),v);}
for mut v in data.remove("sup_recursions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniExpression(v)?;
}
wire.insert("sup_recursions".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_SymbolNode_json(input:&str)->ContractResult<Umath_Model_SymbolNode>{let data=decode_Umath_Model_SymbolNode(parse_json(input,"Umath::Model::SymbolNode@default")?)?; let model:Umath_Model_SymbolNode=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SymbolNode@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_SymbolNode_json(model:&Umath_Model_SymbolNode)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SymbolNode@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_SymbolNode(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::SymbolNode@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_SymbolNode(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::SymbolNode@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("ldots") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SymbolNode@default", "ldots", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ldots".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("spelling") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SymbolNode@default", "spelling", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("spelling".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SymbolNode@default", "value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("kind") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SymbolNode@default", "kind", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("kind".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::SymbolNode@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_SymbolNode(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_SymbolNode(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("ldots"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("ldots".into(),v);}
if let Some(mut v)=data.remove("spelling"){
data.insert("spelling".into(),v);}
if let Some(mut v)=data.remove("value"){
data.insert("value".into(),v);}
if let Some(mut v)=data.remove("kind"){
data.insert("kind".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_SymbolNode(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::SymbolNode@default")?;let mut wire=Map::new();
for mut v in data.remove("ldots").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("ldots".into(),v);}
for mut v in data.remove("spelling").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("spelling".into(),v);}
for mut v in data.remove("value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("value".into(),v);}
for mut v in data.remove("kind").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("kind".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Table_json(input:&str)->ContractResult<Umath_Model_Table>{let data=decode_Umath_Model_Table(parse_json(input,"Umath::Model::Table@default")?)?; let model:Umath_Model_Table=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Table@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Table_json(model:&Umath_Model_Table)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Table@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Table(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Table@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Table(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Table@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("array") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Table@default", "array", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("array".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("identity_matrix_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Table@default", "identity_matrix_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("identity_matrix_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("matrixs") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Table@default", "matrixs", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("matrixs".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Table@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Table(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Table(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("array"){
if !v.is_null() {
v=decode_Umath_Model_ArrayNode(v)?;
}
data.insert("array".into(),v);}
if let Some(mut v)=data.remove("identity_matrix_number"){
data.insert("identity_matrix_number".into(),v);}
if let Some(mut v)=data.remove("matrixs"){
data.insert("matrixs".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Table(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Table@default")?;let mut wire=Map::new();
for mut v in data.remove("array").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_ArrayNode(v)?;
}
wire.insert("array".into(),v);}
for mut v in data.remove("identity_matrix_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("identity_matrix_number".into(),v);}
for mut v in data.remove("matrixs").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("matrixs".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Td_json(input:&str)->ContractResult<Umath_Model_Td>{let data=decode_Umath_Model_Td(parse_json(input,"Umath::Model::Td@default")?)?; let model:Umath_Model_Td=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Td@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Td_json(model:&Umath_Model_Td)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Td@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Td(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Td@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Td(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Td@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Td@default", "exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Td@default", "expr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Td@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Td@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Td(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Td(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("exp"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("exp".into(),v);}
if let Some(mut v)=data.remove("expr"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("expr".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Td(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Td@default")?;let mut wire=Map::new();
for mut v in data.remove("exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("exp".into(),v);}
for mut v in data.remove("expr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("expr".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Tds_json(input:&str)->ContractResult<Umath_Model_Tds>{let data=decode_Umath_Model_Tds(parse_json(input,"Umath::Model::Tds@default")?)?; let model:Umath_Model_Tds=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Tds@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Tds_json(model:&Umath_Model_Tds)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Tds@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Tds(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Tds@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Tds(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Tds@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("td") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Tds@default", "td", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("td".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("tds") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Tds@default", "tds", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("tds".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Tds@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Tds(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Tds(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("td"){
if !v.is_null() {
v=decode_Umath_Model_Td(v)?;
}
data.insert("td".into(),v);}
if let Some(mut v)=data.remove("tds"){
if !v.is_null() {
v=decode_Umath_Model_Tds(v)?;
}
data.insert("tds".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Tds(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Tds@default")?;let mut wire=Map::new();
for mut v in data.remove("td").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Td(v)?;
}
wire.insert("td".into(),v);}
for mut v in data.remove("tds").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Tds(v)?;
}
wire.insert("tds".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Tr_json(input:&str)->ContractResult<Umath_Model_Tr>{let data=decode_Umath_Model_Tr(parse_json(input,"Umath::Model::Tr@default")?)?; let model:Umath_Model_Tr=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Tr@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Tr_json(model:&Umath_Model_Tr)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Tr@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Tr(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Tr@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Tr(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Tr@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Tr@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("td") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Tr@default", "td", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("td".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("tds") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Tr@default", "tds", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("tds".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Tr@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Tr(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Tr(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("td"){
if !v.is_null() {
v=decode_Umath_Model_Td(v)?;
}
data.insert("td".into(),v);}
if let Some(mut v)=data.remove("tds"){
if !v.is_null() {
v=decode_Umath_Model_Tds(v)?;
}
data.insert("tds".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Tr(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Tr@default")?;let mut wire=Map::new();
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("td").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Td(v)?;
}
wire.insert("td".into(),v);}
for mut v in data.remove("tds").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Tds(v)?;
}
wire.insert("tds".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Trs_json(input:&str)->ContractResult<Umath_Model_Trs>{let data=decode_Umath_Model_Trs(parse_json(input,"Umath::Model::Trs@default")?)?; let model:Umath_Model_Trs=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Trs@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Trs_json(model:&Umath_Model_Trs)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Trs@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Trs(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Trs@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Trs(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Trs@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("tr") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Trs@default", "tr", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("tr".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("trs") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Trs@default", "trs", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("trs".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Trs@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Trs(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Trs(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("tr"){
if !v.is_null() {
v=decode_Umath_Model_Tr(v)?;
}
data.insert("tr".into(),v);}
if let Some(mut v)=data.remove("trs"){
if !v.is_null() {
v=decode_Umath_Model_Trs(v)?;
}
data.insert("trs".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Trs(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Trs@default")?;let mut wire=Map::new();
for mut v in data.remove("tr").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Tr(v)?;
}
wire.insert("tr".into(),v);}
for mut v in data.remove("trs").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Trs(v)?;
}
wire.insert("trs".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_UnaryFunction_json(input:&str)->ContractResult<Umath_Model_UnaryFunction>{let data=decode_Umath_Model_UnaryFunction(parse_json(input,"Umath::Model::UnaryFunction@default")?)?; let model:Umath_Model_UnaryFunction=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::UnaryFunction@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_UnaryFunction_json(model:&Umath_Model_UnaryFunction)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::UnaryFunction@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_UnaryFunction(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::UnaryFunction@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_UnaryFunction(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::UnaryFunction@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnaryFunction@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_arg_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnaryFunction@default", "unary_arg_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_arg_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnaryFunction@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnaryFunction@default", "unary_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnaryFunction@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_UnaryFunction(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_UnaryFunction(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("unary_arg_functions"){
data.insert("unary_arg_functions".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("unary_symbols"){
if !v.is_null() {
v=decode_Umath_Model_MathSymbol(v)?;
}
data.insert("unary_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_UnaryFunction(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::UnaryFunction@default")?;let mut wire=Map::new();
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("unary_arg_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary_arg_functions".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("unary_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MathSymbol(v)?;
}
wire.insert("unary_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_UnarySubSup_json(input:&str)->ContractResult<Umath_Model_UnarySubSup>{let data=decode_Umath_Model_UnarySubSup(parse_json(input,"Umath::Model::UnarySubSup@default")?)?; let model:Umath_Model_UnarySubSup=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubSup@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_UnarySubSup_json(model:&Umath_Model_UnarySubSup)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubSup@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_UnarySubSup(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubSup@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_UnarySubSup(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::UnarySubSup@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("mini_sub") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubSup@default", "mini_sub", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_sub".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_sub_sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubSup@default", "mini_sub_sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_sub_sup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mini_sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubSup@default", "mini_sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mini_sup".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sub_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubSup@default", "sub_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sub_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subsup_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubSup@default", "subsup_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subsup_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sup_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubSup@default", "sup_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sup_exp".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubSup@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_UnarySubSup(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_UnarySubSup(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("mini_sub"){
if !v.is_null() {
v=decode_Umath_Model_MiniSub(v)?;
}
data.insert("mini_sub".into(),v);}
if let Some(mut v)=data.remove("mini_sub_sup"){
if !v.is_null() {
v=decode_Umath_Model_MiniSubSup(v)?;
}
data.insert("mini_sub_sup".into(),v);}
if let Some(mut v)=data.remove("mini_sup"){
if !v.is_null() {
v=decode_Umath_Model_MiniSup(v)?;
}
data.insert("mini_sup".into(),v);}
if let Some(mut v)=data.remove("sub_exp"){
if !v.is_null() {
v=decode_Umath_Model_SubExp(v)?;
}
data.insert("sub_exp".into(),v);}
if let Some(mut v)=data.remove("subsup_exp"){
if !v.is_null() {
v=decode_Umath_Model_SubsupExp(v)?;
}
data.insert("subsup_exp".into(),v);}
if let Some(mut v)=data.remove("sup_exp"){
if !v.is_null() {
v=decode_Umath_Model_SupExp(v)?;
}
data.insert("sup_exp".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_UnarySubSup(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::UnarySubSup@default")?;let mut wire=Map::new();
for mut v in data.remove("mini_sub").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniSub(v)?;
}
wire.insert("mini_sub".into(),v);}
for mut v in data.remove("mini_sub_sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniSubSup(v)?;
}
wire.insert("mini_sub_sup".into(),v);}
for mut v in data.remove("mini_sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MiniSup(v)?;
}
wire.insert("mini_sup".into(),v);}
for mut v in data.remove("sub_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SubExp(v)?;
}
wire.insert("sub_exp".into(),v);}
for mut v in data.remove("subsup_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SubsupExp(v)?;
}
wire.insert("subsup_exp".into(),v);}
for mut v in data.remove("sup_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_SupExp(v)?;
}
wire.insert("sup_exp".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_UnarySubsup_json(input:&str)->ContractResult<Umath_Model_UnarySubsup>{let data=decode_Umath_Model_UnarySubsup(parse_json(input,"Umath::Model::UnarySubsup@default")?)?; let model:Umath_Model_UnarySubsup=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubsup@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_UnarySubsup_json(model:&Umath_Model_UnarySubsup)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubsup@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_UnarySubsup(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubsup@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_UnarySubsup(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::UnarySubsup@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubsup@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubsup@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_sub_sup") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubsup@default", "unary_sub_sup", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_sub_sup".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnarySubsup@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_UnarySubsup(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_UnarySubsup(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Umath_Model_Expression(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("unary_sub_sup"){
if !v.is_null() {
v=decode_Umath_Model_UnarySubSup(v)?;
}
data.insert("unary_sub_sup".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_UnarySubsup(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::UnarySubsup@default")?;let mut wire=Map::new();
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_Expression(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("unary_sub_sup").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_UnarySubSup(v)?;
}
wire.insert("unary_sub_sup".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_UnicodedFontClass_json(input:&str)->ContractResult<Umath_Model_UnicodedFontClass>{let data=decode_Umath_Model_UnicodedFontClass(parse_json(input,"Umath::Model::UnicodedFontClass@default")?)?; let model:Umath_Model_UnicodedFontClass=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::UnicodedFontClass@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_UnicodedFontClass_json(model:&Umath_Model_UnicodedFontClass)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::UnicodedFontClass@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_UnicodedFontClass(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::UnicodedFontClass@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_UnicodedFontClass(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::UnicodedFontClass@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnicodedFontClass@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("mitBbb") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnicodedFontClass@default", "mitBbb", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("mitBbb".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::UnicodedFontClass@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_UnicodedFontClass(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_UnicodedFontClass(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("mitBbb"){
if !v.is_null() {
v=decode_Umath_Model_MitBbb(v)?;
}
data.insert("mitBbb".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_UnicodedFontClass(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::UnicodedFontClass@default")?;let mut wire=Map::new();
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("mitBbb").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Umath_Model_MitBbb(v)?;
}
wire.insert("mitBbb".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Umath_Model_Whole_json(input:&str)->ContractResult<Umath_Model_Whole>{let data=decode_Umath_Model_Whole(parse_json(input,"Umath::Model::Whole@default")?)?; let model:Umath_Model_Whole=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Whole@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Umath_Model_Whole_json(model:&Umath_Model_Whole)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Whole@default", "", "/output",e))?;serde_json::to_string(&encode_Umath_Model_Whole(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Umath::Model::Whole@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Umath_Model_Whole(input:Value)->ContractResult<Value>{let mut wire=object(input,"Umath::Model::Whole@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Whole@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Umath::Model::Whole@default", "", "/input", "unknown wire key"));} hydrate_Umath_Model_Whole(data) }
#[allow(non_snake_case)]
fn hydrate_Umath_Model_Whole(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Umath_Model_Whole(input:Value)->ContractResult<Value>{let mut data=object(input,"Umath::Model::Whole@default")?;let mut wire=Map::new();
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
Ok(Value::Object(wire))}
