use json_step_by_step::{ReadingConfig, Value, parse};

fn main() {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("Usage: json_step_by_step <config.json>");
        std::process::exit(2);
    };
    let input = match std::fs::read_to_string(path) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("Could not read UTF-8 input: {error}");
            std::process::exit(2);
        }
    };
    let value = match parse(&input) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("JSON error at byte {}: {:?}", error.offset, error.kind);
            std::process::exit(1);
        }
    };
    let config = match ReadingConfig::from_value(value) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Configuration error: {error:?}");
            std::process::exit(1);
        }
    };
    println!("Enabled: {}", config.enabled);
    println!("Outer readings: {}", config.readings.len());
    if let Some(Value::Array(group)) = config.readings.get(1) {
        println!("Second reading contains {} items", group.len());
        for item in group {
            if let Value::Number(number) = item {
                println!("Number: {}", number.as_str());
            }
        }
    }
}
