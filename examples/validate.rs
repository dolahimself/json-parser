use json_step_by_step::parse;

fn main() {
    let Some(path) = std::env::args().nth(1) else {
        std::process::exit(2);
    };
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => std::process::exit(2),
    };
    let valid = std::str::from_utf8(&bytes)
        .ok()
        .and_then(|s| parse(s).ok())
        .is_some();
    std::process::exit(if valid { 0 } else { 1 });
}