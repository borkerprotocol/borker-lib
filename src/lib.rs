#[macro_use] extern crate failure;

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

pub fn restore_seed(words: [&str; 12]) -> Result<Seed, failure::Error> {
    Seed::from_words(words)
}