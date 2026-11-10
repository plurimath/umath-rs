# umath Rust raw parser

Local unpublished Rust package using the upstream shared PARG artifact and declared
Rust bindings; no handwritten grammar. `Parser::new()` verifies embedded checksums.

```rust
let parser = umath::Parser::new()?;
let parsed = parser.parse("x+1")?;
println!("{}", parsed.canonical());
# Ok::<(), umath::Error>(())
```

`parse_with_decimal(input, decimal)` accepts `.`, `,`, and `٫`. The native
Parsanol engine is pinned to 0.20.7. Unicode preprocessing and stage transforms
are reused from the pinned shared testsuite, with attribution in UPSTREAM.json.
Errors preserve the native error and expose UTF-8 byte offsets relative to the
preprocessed grammar input. Parsed slices reference `parsed_input`; caller offsets
are not inferred after preprocessing. Canonical output includes character offsets.

This package exposes raw stage trees and canonical tagged JSON only. Typed Lutaml
models, public Plurimath facade and math writers are not implemented or claimed.
Use `cargo run --example parse -- 'x+1'` and `cargo test` in a normal Rust environment.
