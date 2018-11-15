extern crate borker_lib as lib;

use std::env::args;

macro_rules! map_str {
    ($x:expr) => {
        $x.as_ref().map(|s| s.as_str())
    };
}

fn main() -> Result<(), failure::Error> {
    let cmd: Vec<String> = args().collect();
    let name = cmd.get(0).unwrap();
    match map_str!(cmd.get(1)) {
        Some("new_seed") => {
            let ent = lib::new_seed();
            eprintln!("{:?}", ent);
            println!("{}", ent.words().join(" "))
        },
        Some("restore_seed") => {
            match cmd.get(2..14) {
                Some(a) => {
                    let mut arr = [""; 12];
                    for (out_word, in_word) in arr.iter_mut().zip(a) {
                        *out_word = in_word;
                    }
                    let ent = lib::restore_seed(arr)?;
                    eprintln!("{:?}", ent);
                    println!("{}", ent.words().join(" "));
                },
                None => eprintln!("usage: {} restore_seed <word1> <word2> ... <word12>", name)
            }
        },
        None | Some("help") => eprintln!("usage: {} <command> [args...]", name),
        Some(a) => eprintln!("'{}' is not a valid command", a),
    }
    Ok(())
}
