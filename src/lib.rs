#[macro_use] extern crate failure;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use failure::Error;

#[macro_use] mod macros;
mod bip39;

use self::bip39::Seed;

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

#[wasm_bindgen]
pub fn new_raw_seed() -> Box<[u8]> {
    let Seed(raw_seed) = new_seed();

    Box::new(raw_seed)
}

#[wasm_bindgen]
pub fn words_from_raw_seed(raw_seed: &[u8]) -> Vec<JsValue> {
    let mut a: [u8; 16] = [0; 16];
    a.copy_from_slice(&raw_seed[0..16]);
    Seed(a).words().into_iter().map(|a| JsValue::from_str(a)).collect()
}

#[wasm_bindgen]
pub fn restore_raw_seed(words: JsValue) -> Option<Box<[u8]>> {
    let Seed(raw_seed) = restore_seed(words.into_serde().ok()?).ok()?;

    Some(Box::new(raw_seed))
}