#[derive(Debug, PartialEq)]
pub enum Value {
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
    if pos != bytes.len() {
        return Err(ParseError { offset: pos, expected: "end of input" });
    }
    Ok(value)
}

fn read_value(bytes: &[u8], pos: &mut usize) -> Result<Value, ParseError> {
    match bytes.get(*pos) {
        Some(b'[') => read_array(bytes, pos),
        Some(b'-' | b'0'..=b'9') => read_number(bytes, pos),
        _ => Err(ParseError { offset: *pos, expected: "an integer or an array" }),
    }
}

fn read_number(bytes: &[u8], pos: &mut usize) -> Result<Value, ParseError> {
    let start = *pos;
    if bytes.get(*pos) == Some(&b'-') {
        *pos += 1;
    }
    let first_digit = *pos;
    while let Some(byte) = bytes.get(*pos) {
        if !byte.is_ascii_digit() { break; }
        *pos += 1;
    }
    if *pos == first_digit {
        return Err(ParseError { offset: *pos, expected: "a digit" });
    }
    if bytes[first_digit] == b'0' && *pos > first_digit + 1 {
        return Err(ParseError { offset: first_digit + 1, expected: "end of number after zero" });
    }
    let text = std::str::from_utf8(&bytes[start..*pos])
        .map_err(|_| ParseError { offset: start, expected: "UTF-8 number text" })?;
    let number = text.parse::<i64>()
        .map_err(|_| ParseError { offset: start, expected: "an integer within i64 range" })?;
    Ok(Value::Number(number))
}

fn read_array(bytes: &[u8], pos: &mut usize) -> Result<Value, ParseError> {
    // read_value has already checked the opening bracket.
    *pos += 1;
    let mut items = Vec::new();
    if bytes.get(*pos) == Some(&b']') {
        *pos += 1;
        return Ok(Value::Array(items));
    }
    loop {
        let item = read_value(bytes, pos)?;
        items.push(item);
        match bytes.get(*pos) {
            Some(b',') => { *pos += 1; }
            Some(b']') => {
                *pos += 1;
                return Ok(Value::Array(items));
            }
            _ => return Err(ParseError { offset: *pos, expected: "a comma or closing bracket" }),
        }
    }
}