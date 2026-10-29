# JSON parser behavior

- Accepts a complete UTF-8 JSON document with JSON whitespace around and between structural elements.
- Handles null, booleans, strings, number grammar, arrays, and objects.
- A Number owns validated source spelling. It does not silently round large values; `as_i64` is a separate fallible conversion requested by a consumer. Derived equality compares the stored representation, not mathematical equivalence of different spellings such as 1 and 1.0.
- Strings decode ordinary escapes and Unicode escape pairs to Rust String. Unpaired surrogate escapes are rejected. Literal non-ASCII UTF-8 is preserved. Byte offsets in errors are not character indices.
- Objects retain source order and duplicate members. ReadingConfig, the example consumer, rejects duplicate fields it uses. These are distinct policies at distinct boundaries.
- The parser permits at most 128 nested containers and reports a NestingLimit error for more. It does not allow arbitrarily deep input to exhaust the call stack.
- Value readers skip leading whitespace and leave the first byte after their value for the caller. Containers skip following whitespace before checking their punctuation. The document entry point checks trailing whitespace and end of input.
- The `&str` API receives valid UTF-8. File-byte decoding happens in the outer I/O layer.
- Syntactically valid JSON is not proof of an application's schema. ReadingConfig separately requires boolean enabled and array readings, rejects duplicates of those fields, and ignores unknown fields.

This behavior is verified against project tests and JSONTestSuite, with mandatory accept/reject cases distinguished from implementation-dependent cases. The test report records the exact suite revision.