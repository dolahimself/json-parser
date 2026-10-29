#[path = "../01-one-item/lib.rs"] mod parser;

#[test]
fn empty() {
    assert_eq!(parser::parse("[]"), Ok(vec![]));
}

#[test]
fn one_integer() {
    assert_eq!(parser::parse("[12]"), Ok(vec![12]));
}

#[test]
fn flat_is_not_yet_supported() {
    assert!(parser::parse("[12, 3]").is_err());
}