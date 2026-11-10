#[cfg(feature = "models")]
fn main() {
    let parser = umath::Parser::new().unwrap();
    let model = parser
        .parse_model(&std::env::args().nth(1).unwrap_or("α+1".into()), ".")
        .unwrap();
    println!("{}", umath::typed::to_portable_json(&model).unwrap());
}
#[cfg(not(feature = "models"))]
fn main() {
    eprintln!("enable --features models");
    std::process::exit(1)
}
