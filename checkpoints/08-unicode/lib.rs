// 08-unicode — genuine intermediate construction checkpoint.

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

/// Only the number reader can construct this validated number representation.
#[derive(Debug, Clone, PartialEq)]
pub struct Number(String);

impl Number {
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn as_i64(&self) -> Result<i64, std::num::ParseIntError> {
        self.0.parse()
    }
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

fn read_number(input: &str, pos: &mut usize) -> Result<Number, ParseError> {
    let bytes = input.as_bytes();
    let start = *pos;
    if bytes.get(*pos) == Some(&b'-') {
        *pos += 1;
    }
    match bytes.get(*pos) {
        Some(b'0') => {
            *pos += 1;
            if matches!(bytes.get(*pos), Some(b'0'..=b'9')) {
                return Err(ParseError {
                    offset: *pos,
                    kind: ErrorKind::InvalidNumber,
                });
            }
        }
        Some(b'1'..=b'9') => read_digits(bytes, pos)?,
        _ => {
            return Err(ParseError {
                offset: *pos,
                kind: ErrorKind::Expected("a digit"),
            });
        }
    }
    if bytes.get(*pos) == Some(&b'.') {
        *pos += 1;
        read_digits(bytes, pos)?;
    }
    if matches!(bytes.get(*pos), Some(b'e' | b'E')) {
        *pos += 1;
        if matches!(bytes.get(*pos), Some(b'+' | b'-')) {
            *pos += 1;
        }
        read_digits(bytes, pos)?;
    }
    Ok(Number(input[start..*pos].to_owned()))
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
    let mut output = String::new();
    let mut run_start = *pos;
    loop {
        match bytes.get(*pos) {
            Some(b'"') => {
                output.push_str(&input[run_start..*pos]);
                *pos += 1;
                return Ok(output);
            }
            Some(b'\\') => {
                output.push_str(&input[run_start..*pos]);
                *pos += 1;
                output.push(read_escape(input, pos)?);
                run_start = *pos;
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

fn read_escape(input: &str, pos: &mut usize) -> Result<char, ParseError> {
    let offset = *pos;
    let escaped = match input.as_bytes().get(*pos) {
        Some(b'"') => '"',
        Some(b'\\') => '\\',
        Some(b'/') => '/',
        Some(b'b') => '\u{0008}',
        Some(b'f') => '\u{000c}',
        Some(b'n') => '\n',
        Some(b'r') => '\r',
        Some(b't') => '\t',
        Some(b'u') => {
            *pos += 1;
            return read_unicode(input, pos);
        }
        _ => {
            return Err(ParseError {
                offset,
                kind: ErrorKind::InvalidEscape,
            });
        }
    };
    *pos += 1;
    Ok(escaped)
}

fn read_hex4(input: &str, pos: &mut usize) -> Result<u32, ParseError> {
    let mut value = 0;
    for _ in 0..4 {
        let digit = input
            .as_bytes()
            .get(*pos)
            .and_then(|byte| char::from(*byte).to_digit(16))
            .ok_or(ParseError {
                offset: *pos,
                kind: ErrorKind::InvalidUnicodeEscape,
            })?;
        value = value * 16 + digit;
        *pos += 1;
    }
    Ok(value)
}

fn read_unicode(input: &str, pos: &mut usize) -> Result<char, ParseError> {
    let offset = *pos;
    let first = read_hex4(input, pos)?;
    let scalar = match first {
        0xD800..=0xDBFF => {
            if !input.as_bytes()[*pos..].starts_with(b"\\u") {
                return Err(ParseError {
                    offset: *pos,
                    kind: ErrorKind::InvalidUnicodeEscape,
                });
            }
            *pos += 2;
            let low_offset = *pos;
            let second = read_hex4(input, pos)?;
            if !(0xDC00..=0xDFFF).contains(&second) {
                return Err(ParseError {
                    offset: low_offset,
                    kind: ErrorKind::InvalidUnicodeEscape,
                });
            }
            0x10000 + ((first - 0xD800) << 10) + (second - 0xDC00)
        }
        0xDC00..=0xDFFF => {
            return Err(ParseError {
                offset,
                kind: ErrorKind::InvalidUnicodeEscape,
            });
        }
        _ => first,
    };
    char::from_u32(scalar).ok_or(ParseError {
        offset,
        kind: ErrorKind::InvalidUnicodeEscape,
    })
}
