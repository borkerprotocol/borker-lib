#[macro_use] extern crate failure;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;
// use wasm_bindgen::JsValue;
use failure::Error;

#[macro_use] mod macros;
mod bip39;

use self::bip39::Seed;

#[wasm_bindgen]
pub fn new_seed() -> Seed {
    use rand::RngCore;
    use rand::rngs::EntropyRng;

    let mut res: [u8; 16] = [0; 16];
    EntropyRng::new().fill_bytes(&mut res);
    Seed::new(res)
}

pub fn restore_seed(words: [String; 12]) -> Result<Seed, Error> {
    Seed::from_words(words)
}
