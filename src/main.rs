extern crate borker_rs as lib;

use std::env::args;
use std::sync::{Arc, RwLock};

fn count_match(a: &str, b: &str, case_sensitive: bool) -> usize {
    let mut count = 0;
    for (c_a, c_b) in a.chars().zip(b.chars()) {
        if c_a == c_b
            || (!case_sensitive
                && c_a.to_lowercase().collect::<Vec<_>>() == c_b.to_lowercase().collect::<Vec<_>>())
        {
            count += 1;
        } else {
            return count;
        }
    }
    count
}

fn main() -> Result<(), failure::Error> {
    let cmd: Vec<String> = args().collect();
    let name = cmd.get(0).unwrap();
    let int = Arc::new(RwLock::new(false));
    #[cfg(feature = "ctrlc")]
    {
        let int_c = int.clone();
        ctrlc::set_handler(move || {
            *int_c.write().unwrap() = true;
        })?;
    }
    match cmd.get(1).as_ref().map(|s| s.as_str()) {
        Some("new_wallet") => {
            let ent = lib::Wallet::new();
            println!("{}", ent.words().join(" "));
        }
        Some("restore_wallet") => match cmd.get(2..14) {
            Some(a) => {
                let ent = lib::Wallet::from_words(a)?;
                println!("{}", hex::encode(&ent.as_bytes()?));
            }
            None => eprintln!("usage: {} restore_seed <word1> <word2> ... <word12>", name),
        },
        Some("wallet_from_bytes") => match cmd.get(2) {
            Some(a) => {
                let ent = lib::Wallet::from_bytes(&hex::decode(a)?)?;
                println!("{}", ent.words().join(" "));
            }
            None => eprintln!("usage: {} wallet_from_bytes <hexdata>", name),
        },
        Some("parse_block") => {
            let mut v: serde_json::Value =
                serde_json::from_reader(std::fs::File::open("./blockdata.json").unwrap()).unwrap();
            let v = v.get_mut("result").unwrap().take();
            let s: String = serde_json::from_value(v).unwrap();
            println!("{:?}", lib::processBlock(s, lib::Network::Dogecoin));
        }
        Some("vanity") | Some("vanity_insensitive") => match cmd.get(2) {
            Some(target) => {
                let sensitive = cmd.get(1).unwrap() == "vanity";
                let mut best: Option<(usize, lib::Wallet)> = None;
                bitcoin::util::base58::from(target)?;
                if target.chars().next().unwrap() != 'D' {
                    failure::bail!("doge address must start with 'D'")
                };
                loop {
                    let mut wallet = lib::Wallet::new();
                    let child = wallet
                        .parent_mut()
                        .load_child(44, true)?
                        .load_child(3, true)?
                        .load_child(0, true)?
                        .load_child(0, false)?
                        .load_child(0, false)?;
                    let addr = child.address(lib::Network::Dogecoin);
                    let count = count_match(target, &addr, sensitive);
                    match best {
                        Some((c, _)) if count > c => {
                            best = Some((count, wallet));
                            println!("{}", addr);
                        }
                        None => {
                            best = Some((count, wallet));
                            println!("{}", addr);
                        }
                        _ => (),
                    }
                    if count == target.len() {
                        break;
                    }
                    if *int.read().unwrap() {
                        break;
                    }
                }
                match best {
                    Some(a) => println!("\n{}", a.1.words().join(" ")),
                    None => (),
                }
            }
            None => eprintln!("usage: {} vanity <address_prefix>", name),
        },
        None | Some("help") => eprintln!("usage: {} <command> [args...]", name),
        Some(a) => eprintln!("'{}' is not a valid command", a),
    };
    Ok(())
}
