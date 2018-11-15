use std::env::args;

macro_rules! map_str {
    ($x:expr) => {
        $x.as_ref().map(|s| s.as_str())
    };
}

fn main() {
    let cmd: Vec<String> = args().collect();
    let name = cmd.get(0).unwrap();
    match map_str!(cmd.get(1)) {
        Some("hello") => println!("Hello!"),
        None | Some("help") => println!("usage: {} <command> [args...]", name),
        Some(a) => println!("'{}' is not a valid command", a),
    }
}
