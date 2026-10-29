// use json_step_by_step::*;

// fn number(s: &str) -> Value {
//     parse(s).unwrap()
// }

// #[test]
// fn nested_arrays_preserve_structure() {
//     assert_eq!(
//         parse("[1,[2,3],4]"),
//         Ok(Value::Array(vec![
//             number("1"),
//             Value::Array(vec![number("2"), number("3")]),
//             number("4")
//         ]))
//     );
//     assert_eq!(
//         parse("[[],4]"),
//         Ok(Value::Array(vec![Value::Array(vec![]), number("4")]))
//     );
// }

// #[test]
// fn all_value_kinds_and_whitespace() {
//     assert_eq!(
//         parse(" \n\t[null, true, false, \"x\", {}, []]\r "),
//         Ok(Value::Array(vec![
//             Value::Null,
//             Value::Bool(true),
//             Value::Bool(false),
//             Value::String("x".into()),
//             Value::Object(vec![]),
//             Value::Array(vec![])
//         ]))
//     );
// }

// #[test]
// fn numbers_preserve_spelling_without_range_loss() {
//     for text in [
//         "0",
//         "-0",
//         "12",
//         "-12.75",
//         "1e20",
//         "1E-20",
//         "1e+99999",
//         "9007199254740993",
//         "123456789012345678901234567890",
//     ] {
//         let Value::Number(n) = parse(text).unwrap() else {
//             panic!()
//         };
//         assert_eq!(n.as_str(), text);
//     }
//     let Value::Number(n) = parse("9007199254740993").unwrap() else {
//         panic!()
//     };
//     assert_eq!(n.as_i64().unwrap(), 9007199254740993);
// }

// #[test]
// fn invalid_numbers_are_rejected() {
//     for text in [
//         "+1", "01", "-01", "1.", ".1", "1e", "1e+", "--1", "NaN", "Infinity", "-", "1 2",
//     ] {
//         assert!(parse(text).is_err(), "{text}");
//     }
// }

// #[test]
// fn strings_decode_escapes_and_unicode() {
//     assert_eq!(
//         parse(r#""A\nB\"C\\D\/E\tF\rG\bH\fI""#),
//         Ok(Value::String("A\nB\"C\\D/E\tF\rG\u{8}H\u{c}I".into()))
//     );
//     assert_eq!(
//         parse(r#""\u0041\u00e9\uD83D\uDE00""#),
//         Ok(Value::String("Aé😀".into()))
//     );
//     assert_eq!(parse("\"é😀ሀ\""), Ok(Value::String("é😀ሀ".into())));
//     assert_eq!(parse(r#""\u0000""#), Ok(Value::String("\0".into())));
// }

// #[test]
// fn strings_reject_invalid_forms_with_positions() {
//     for text in [
//         "\"unfinished",
//         "\"raw\nline\"",
//         r#""\x""#,
//         r#""\u12""#,
//         r#""\uZZZZ""#,
//         r#""\uD800""#,
//         r#""\uDC00""#,
//         r#""\uD800\u0041""#,
//     ] {
//         assert!(parse(text).is_err(), "{text}");
//     }
//     assert_eq!(
//         parse(r#""a\x""#),
//         Err(ParseError {
//             offset: 3,
//             kind: ErrorKind::InvalidEscape
//         })
//     );
// }

// #[test]
// fn objects_preserve_duplicate_members() {
//     assert_eq!(parse(r#"{"x":1,"x":2}""#).is_ok(), false);
//     assert_eq!(
//         parse(r#"{"x":1,"x":2}"#),
//         Ok(Value::Object(vec![
//             ("x".into(), number("1")),
//             ("x".into(), number("2"))
//         ]))
//     );
// }

// #[test]
// fn punctuation_and_end_are_checked() {
//     for text in [
//         "",
//         "[",
//         "[1",
//         "[1,]",
//         "[,1]",
//         "[1 2]",
//         "{",
//         "{x:1}",
//         "{\"x\" 1}",
//         "{\"x\":}",
//         "{\"x\":1,}",
//         "truex",
//     ] {
//         assert!(parse(text).is_err(), "{text}");
//     }
//     assert_eq!(
//         parse("[1]x"),
//         Err(ParseError {
//             offset: 3,
//             kind: ErrorKind::TrailingInput
//         })
//     );
//     assert_eq!(
//         parse("[1,]"),
//         Err(ParseError {
//             offset: 3,
//             kind: ErrorKind::Expected("a JSON value")
//         })
//     );
//     assert_eq!(
//         parse("nul"),
//         Err(ParseError {
//             offset: 3,
//             kind: ErrorKind::Expected("the rest of the literal")
//         })
//     );
// }

// #[test]
// fn nesting_limit_is_controlled() {
//     let accepted = format!("{}0{}", "[".repeat(128), "]".repeat(128));
//     assert!(parse(&accepted).is_ok());
//     let rejected = format!("{}0{}", "[".repeat(129), "]".repeat(129));
//     assert_eq!(parse(&rejected).unwrap_err().kind, ErrorKind::NestingLimit);
// }

// #[test]
// fn application_has_a_separate_validation_boundary() {
//     let v = parse(r#"{"readings":[1,[2,3],4],"enabled":true}"#).unwrap();
//     let config = ReadingConfig::from_value(v).unwrap();
//     assert!(config.enabled);
//     assert_eq!(config.readings.len(), 3);
//     let syntactically_valid = parse(r#"{"readings":[],"enabled":"yes"}"#).unwrap();
//     assert_eq!(
//         ReadingConfig::from_value(syntactically_valid),
//         Err(ConfigError::WrongType("enabled"))
//     );
//     let duplicate = parse(r#"{"enabled":true,"enabled":false,"readings":[]}"#).unwrap();
//     assert_eq!(
//         ReadingConfig::from_value(duplicate),
//         Err(ConfigError::Duplicate("enabled"))
//     );
//     assert_eq!(
//         ReadingConfig::from_value(parse("{}").unwrap()),
//         Err(ConfigError::Missing("enabled"))
//     );
// }

fn main() {
    println!("Hello, world!");
}