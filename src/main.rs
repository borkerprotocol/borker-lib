extern crate borker_rs as lib;

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
        Some("new_wallet") => {
            let ent = lib::Wallet::new();
            println!("{}", ent.words().join(" "))
        }
        Some("restore_wallet") => match cmd.get(2..14) {
            Some(a) => {
                let ent = lib::Wallet::from_words(a)?;
                println!("{}", hex::encode(&ent.as_bytes()?));
            }
            None => eprintln!("usage: {} restore_seed <word1> <word2> ... <word12>", name),
        Some("wallet_from_bytes") => match cmd.get(2) {
            Some(a) => {
                let ent = lib::Wallet::from_bytes(&hex::decode(a)?)?;
                println!("{}", ent.words().join(" "))
            }
            None => eprintln!("usage: {} wallet_from_bytes <hexdata>", name),
        Some("parse_block") => {
            let mut v: serde_json::Value =
                serde_json::from_reader(std::fs::File::open("./blockdata.json").unwrap()).unwrap();
            let v = v.get_mut("result").unwrap().take();
            let s: String = serde_json::from_value(v).unwrap();
            println!("{:?}", lib::processBlock(s, 1.0, lib::Network::Dogecoin));
        }
        None | Some("help") => eprintln!("usage: {} <command> [args...]", name),
        Some(a) => eprintln!("'{}' is not a valid command", a),
    }
    Ok(())
}
