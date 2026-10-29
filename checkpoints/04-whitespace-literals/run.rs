#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(i64),
    Array(Vec<Value>),
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub offset: usize,
    pub expected: &'static str,
}

// Checkpoint 3: recursively read integer/array values.
// This chapter supports compact arrays of JSON integers that fit i64.
// Objects, other value kinds, whitespace, fractions, and exponents are later chapters.
pub fn parse(input: &str) -> Result<Value, ParseError> {
    let bytes = input.as_bytes();
    let mut pos = 0;
    let value = read_value(bytes, &mut pos)?;
    skip_ws(bytes, &mut pos);
    if pos != bytes.len() {
        return Err(ParseError {
            offset: pos,
            expected: "end of input",
        });
    }
    Ok(value)
}

fn read_value(bytes: &[u8], pos: &mut usize) -> Result<Value, ParseError> {
    skip_ws(bytes, pos);
    match bytes.get(*pos) {
        Some(b'n') => read_literal(bytes, pos, b"null", Value::Null),
        Some(b't') => read_literal(bytes, pos, b"true", Value::Bool(true)),
        Some(b'f') => read_literal(bytes, pos, b"false", Value::Bool(false)),
        Some(b'[') => read_array(bytes, pos),
        Some(b'-' | b'0'..=b'9') => read_number(bytes, pos),
        _ => Err(ParseError {
            offset: *pos,
            expected: "an integer or an array",
        }),
    }
}

fn read_number(bytes: &[u8], pos: &mut usize) -> Result<Value, ParseError> {
    let start = *pos;
    if bytes.get(*pos) == Some(&b'-') {
        *pos += 1;
    }
    let first_digit = *pos;
    while let Some(byte) = bytes.get(*pos) {
        if !byte.is_ascii_digit() {
            break;
        }
        *pos += 1;
    }
    if *pos == first_digit {
        return Err(ParseError {
            offset: *pos,
            expected: "a digit",
        });
    }
    if bytes[first_digit] == b'0' && *pos > first_digit + 1 {
        return Err(ParseError {
            offset: first_digit + 1,
            expected: "end of number after zero",
        });
    }
    let text = std::str::from_utf8(&bytes[start..*pos]).map_err(|_| ParseError {
        offset: start,
        expected: "UTF-8 number text",
    })?;
    let number = text.parse::<i64>().map_err(|_| ParseError {
        offset: start,
        expected: "an integer within i64 range",
    })?;
    Ok(Value::Number(number))
}

fn read_array(bytes: &[u8], pos: &mut usize) -> Result<Value, ParseError> {
    // read_value has already checked the opening bracket.
    *pos += 1;
    let mut items = Vec::new();
    skip_ws(bytes, pos);
    if bytes.get(*pos) == Some(&b']') {
        *pos += 1;
        return Ok(Value::Array(items));
    }
    loop {
        let item = read_value(bytes, pos)?;
        items.push(item);
        skip_ws(bytes, pos);
        match bytes.get(*pos) {
            Some(b',') => {
                *pos += 1;
            }
            Some(b']') => {
                *pos += 1;
                return Ok(Value::Array(items));
            }
            _ => {
                return Err(ParseError {
                    offset: *pos,
                    expected: "a comma or closing bracket",
                });
            }
        }
    }
}

fn skip_ws(bytes: &[u8], pos: &mut usize) {
    while matches!(bytes.get(*pos), Some(b' ' | b'\t' | b'\r' | b'\n')) {
        *pos += 1;
    }
}
fn read_literal(
    bytes: &[u8],
    pos: &mut usize,
    spelling: &[u8],
    value: Value,
) -> Result<Value, ParseError> {
    for expected in spelling {
        if bytes.get(*pos) != Some(expected) {
            return Err(ParseError {
                offset: *pos,
                expected: "the rest of the literal",
            });
        }
        *pos += 1;
    }
    Ok(value)
}

fn main() {
    for input in [
        "[]",
        "[12,3]",
        "[1,[2,3],4]",
        " [1, [2,3] ] ",
        "[true,null,false]",
        "{\"x\":1,\"items\":[2,3]}",
        "\"hello\"",
        "1.25e-2",
        "\"a\\nb\"",
        "\"\\u0041\\uD83D\\uDE00\"",
        "[1,]",
        "[01]",
        "[1]x",
    ] {
        println!("{}\t{:?}", input, parse(input));
    }
}
