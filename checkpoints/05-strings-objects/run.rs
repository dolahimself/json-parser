#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(i64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    Expected(&'static str),
    InvalidEscape,
    InvalidUnicodeEscape,
    UnescapedControl,
    InvalidNumber,
    TrailingInput,
    NestingLimit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub offset: usize,
    pub kind: ErrorKind,
}

const MAX_DEPTH: usize = 128;

/// Accept one complete JSON document, including surrounding JSON whitespace.
pub fn parse(input: &str) -> Result<Value, ParseError> {
    let mut pos = 0;
    let value = read_value(input, &mut pos, 0)?;
    skip_ws(input.as_bytes(), &mut pos);
    if pos != input.len() {
        return Err(ParseError {
            offset: pos,
            kind: ErrorKind::TrailingInput,
        });
    }
    Ok(value)
}

fn skip_ws(bytes: &[u8], pos: &mut usize) {
    while matches!(bytes.get(*pos), Some(b' ' | b'\t' | b'\r' | b'\n')) {
        *pos += 1;
    }
}

// A reader may skip leading whitespace. It leaves the first byte after its value unread.
// Its caller handles the following whitespace and its own punctuation.
fn read_value(input: &str, pos: &mut usize, depth: usize) -> Result<Value, ParseError> {
    let bytes = input.as_bytes();
    skip_ws(bytes, pos);
    if depth >= MAX_DEPTH && matches!(bytes.get(*pos), Some(b'[' | b'{')) {
        return Err(ParseError {
            offset: *pos,
            kind: ErrorKind::NestingLimit,
        });
    }
    match bytes.get(*pos) {
        Some(b'n') => read_literal(input, pos, "null", Value::Null),
        Some(b't') => read_literal(input, pos, "true", Value::Bool(true)),
        Some(b'f') => read_literal(input, pos, "false", Value::Bool(false)),
        Some(b'"') => Ok(Value::String(read_string(input, pos)?)),
        Some(b'-' | b'0'..=b'9') => Ok(Value::Number(read_number(input, pos)?)),
        Some(b'[') => read_array(input, pos, depth + 1),
        Some(b'{') => read_object(input, pos, depth + 1),
        _ => Err(ParseError {
            offset: *pos,
            kind: ErrorKind::Expected("a JSON value"),
        }),
    }
}

fn read_literal(
    input: &str,
    pos: &mut usize,
    spelling: &str,
    value: Value,
) -> Result<Value, ParseError> {
    for expected in spelling.bytes() {
        if input.as_bytes().get(*pos) != Some(&expected) {
            return Err(ParseError {
                offset: *pos,
                kind: ErrorKind::Expected("the rest of the literal"),
            });
        }
        *pos += 1;
    }
    Ok(value)
}

fn read_array(input: &str, pos: &mut usize, depth: usize) -> Result<Value, ParseError> {
    let bytes = input.as_bytes();
    *pos += 1;
    let mut items = Vec::new();
    skip_ws(bytes, pos);
    if bytes.get(*pos) == Some(&b']') {
        *pos += 1;
        return Ok(Value::Array(items));
    }
    loop {
        let item = read_value(input, pos, depth)?;
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
                    kind: ErrorKind::Expected("a comma or ]"),
                });
            }
        }
    }
}

fn read_object(input: &str, pos: &mut usize, depth: usize) -> Result<Value, ParseError> {
    let bytes = input.as_bytes();
    *pos += 1;
    let mut members = Vec::new();
    skip_ws(bytes, pos);
    if bytes.get(*pos) == Some(&b'}') {
        *pos += 1;
        return Ok(Value::Object(members));
    }
    loop {
        skip_ws(bytes, pos);
        if bytes.get(*pos) != Some(&b'"') {
            return Err(ParseError {
                offset: *pos,
                kind: ErrorKind::Expected("a quoted key"),
            });
        }
        let key = read_string(input, pos)?;
        skip_ws(bytes, pos);
        if bytes.get(*pos) != Some(&b':') {
            return Err(ParseError {
                offset: *pos,
                kind: ErrorKind::Expected(":"),
            });
        }
        *pos += 1;
        let value = read_value(input, pos, depth)?;
        members.push((key, value));
        skip_ws(bytes, pos);
        match bytes.get(*pos) {
            Some(b',') => {
                *pos += 1;
            }
            Some(b'}') => {
                *pos += 1;
                return Ok(Value::Object(members));
            }
            _ => {
                return Err(ParseError {
                    offset: *pos,
                    kind: ErrorKind::Expected("a comma or }"),
                });
            }
        }
    }
}

fn read_number(input: &str, pos: &mut usize) -> Result<i64, ParseError> {
    let bytes = input.as_bytes();
    let start = *pos;
    if bytes.get(*pos) == Some(&b'-') {
        *pos += 1;
    }
    let digit_start = *pos;
    read_digits(bytes, pos)?;
    if bytes[digit_start] == b'0' && *pos > digit_start + 1 {
        return Err(ParseError {
            offset: digit_start + 1,
            kind: ErrorKind::InvalidNumber,
        });
    }
    input[start..*pos].parse::<i64>().map_err(|_| ParseError {
        offset: start,
        kind: ErrorKind::InvalidNumber,
    })
}

fn read_digits(bytes: &[u8], pos: &mut usize) -> Result<(), ParseError> {
    let start = *pos;
    while matches!(bytes.get(*pos), Some(b'0'..=b'9')) {
        *pos += 1;
    }
    if *pos == start {
        return Err(ParseError {
            offset: *pos,
            kind: ErrorKind::Expected("a digit"),
        });
    }
    Ok(())
}

fn read_string(input: &str, pos: &mut usize) -> Result<String, ParseError> {
    let bytes = input.as_bytes();
    *pos += 1;
    let start = *pos;
    loop {
        match bytes.get(*pos) {
            Some(b'"') => {
                let result = input[start..*pos].to_owned();
                *pos += 1;
                return Ok(result);
            }
            Some(b'\\') => {
                return Err(ParseError {
                    offset: *pos,
                    kind: ErrorKind::InvalidEscape,
                });
            }
            Some(0..=31) => {
                return Err(ParseError {
                    offset: *pos,
                    kind: ErrorKind::UnescapedControl,
                });
            }
            Some(_) => {
                *pos += 1;
            }
            None => {
                return Err(ParseError {
                    offset: *pos,
                    kind: ErrorKind::Expected("a closing quote"),
                });
            }
        }
    }
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
