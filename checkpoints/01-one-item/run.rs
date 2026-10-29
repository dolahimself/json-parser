// checkpoint 1: arrays containing zero or one integer
pub fn parse(input: &str) -> Result<Vec<i64>, &'static str> {
    if !input.starts_with("[") || !input.ends_with("]") {
        return Err("expected an array");
    }

    let inside = &input[1..input.len() - 1];
    if inside.is_empty() {
        return Ok(Vec::new());
    }

    let number = inside.parse::<i64>().map_err(|_| "expected an integer")?;

    Ok(vec![number])
}

fn main() {
    for input in [
        "[]",
        "[12]",
        "[12,3]",
        "[1,[2,3],4]",
        "[[],4]",
        "[1,]",
        "[1]x",
    ] {
        println!("{}\t{:?}", input, parse(input));
    }
}
