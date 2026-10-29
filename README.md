# JSON Parser in Rust

A recursive-descent JSON parser built with the Rust standard library. It parses nested arrays and objects, preserves number spelling, decodes string escapes, and reports errors with byte offsets.

## The problem

In `[1,[2,3],4]`, splitting on every comma loses the nested structure. The result has three outer items: `1`, the array `[2,3]`, and `4`.

The parser reads one complete value at a time. An array asks the same value reader to read each child. A child returns just after its closing bracket, leaving the parent to handle its next comma or closing bracket.

## Run the application example

Use a Rust toolchain supporting edition 2024. From the directory containing `Cargo.toml`:

```sh
cargo run -- example.json
```

The included [example.json](example.json) contains:

```json
{"readings":[1,[2,3],4],"enabled":true}
```

Output:

```text
Enabled: true
Outer readings: 3
Second reading contains 2 items
Number: 2
Number: 3
```

This executable parses the document, then checks the application's configuration rules. The library can parse any supported JSON value, including a top-level array, string, or number.

## Use the parser

```rust
use json_step_by_step::{Value, parse};

let value: Value = parse("[1,[2,3],4]").expect("valid JSON");
```

`parse` returns `Result<Value, ParseError>`. `Value` represents null, booleans, numbers, strings, arrays, and objects. A parse error includes its kind and a zero-based byte offset.

To check a file's JSON syntax without applying the configuration schema:

```sh
cargo run --example validate -- example.json
```

The validator produces no output. Its process exit code is `0` for valid JSON, `1` for invalid JSON or UTF-8, and `2` for a missing argument or unreadable file.

## Find the code

| File | Purpose |
| --- | --- |
| [src/lib.rs](src/lib.rs) | Complete parser, value and error types, plus the `ReadingConfig` application example. |
| [src/main.rs](src/main.rs) | Read a file, parse JSON, validate the configuration, and print the result. |
| [examples/validate.rs](examples/validate.rs) | Syntax-only file validator. |
| [tests/parser.rs](tests/parser.rs) | Tests for nested values, numbers, strings, errors, nesting limits, and application validation. |
| [BEHAVIOR.md](BEHAVIOR.md) | Exact parsing and representation policies. |

## Follow the nine stages

Each stage contains `lib.rs`, a runnable `run.rs`, and `results.txt` showing the example results. Early stages intentionally accept only a subset of JSON.

| Stage | Source | What changes |
| --- | --- | --- |
| 1 | [01-one-item](checkpoints/01-one-item/lib.rs) | Parse an empty array or one integer. |
| 2 | [02-split](checkpoints/02-split/lib.rs) | Split a flat integer list; nested arrays expose the limitation. |
| 3 | [03-recursive](checkpoints/03-recursive/lib.rs) | Read nested arrays and integers recursively, with positioned errors. |
| 4 | [04-whitespace-literals](checkpoints/04-whitespace-literals/lib.rs) | Add JSON whitespace, booleans, and null. |
| 5 | [05-strings-objects](checkpoints/05-strings-objects/lib.rs) | Add objects and plain strings. |
| 6 | [06-full-numbers](checkpoints/06-full-numbers/lib.rs) | Handle fractions and exponents while preserving number spelling. |
| 7 | [07-simple-escapes](checkpoints/07-simple-escapes/lib.rs) | Decode short string escapes. |
| 8 | [08-unicode](checkpoints/08-unicode/lib.rs) | Decode Unicode escapes and surrogate pairs. |
| 9 | [09-application](checkpoints/09-application/lib.rs) | Separate JSON syntax from configuration validation. |

For example, run the recursive stage:

```sh
rustc --edition=2024 checkpoints/03-recursive/run.rs -o checkpoint.exe
./checkpoint.exe
```

With Python installed, `python check_checkpoints.py` compiles and runs all nine stages and the checkpoint tests.

## Tests and boundaries

```sh
cargo test
```

The parser preserves object order and duplicate names. `ReadingConfig` separately requires boolean `enabled` and array `readings`, rejects duplicates of those fields, and ignores unknown fields.

Numbers retain their original validated text; converting to an integer is an explicit, fallible operation. Strings decode to Unicode scalar values, so unpaired surrogate escapes are rejected. At most 128 containers may be nested. The parser accepts a complete UTF-8 document and rejects trailing non-whitespace input.

The [JSONTestSuite report](conformance.json) records 95 required valid cases accepted, 188 required invalid cases rejected, and 35 implementation-dependent cases, along with the exact suite revision and fixture hashes. The fixtures come from [JSONTestSuite](https://github.com/nst/JSONTestSuite).