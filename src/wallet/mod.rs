mod consts;

use crate::big_array::BigArray;
use failure::Error;
use secp256k1::{PublicKey, SecretKey};
use pbkdf2::pbkdf2;

type HmacSha512 = hmac::Hmac<sha2::Sha512>;

#[derive(Clone)]
pub struct Wallet {
    entropy: [u8; 16],
    long_seed: Option<[u8; 64]>,
    mpriv: Option<SecretKey>,
    mpub: Option<PublicKey>,
}
impl Wallet {
    pub fn new(entropy: [u8; 16]) -> Self {
        Wallet {
            entropy,
            long_seed: None,
            mpriv: None,
            mpub: None,
        }
    }

    pub fn entropy(&self) -> &[u8] {
        &self.entropy
    }

    fn sha256sum(&self) -> u8 {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.input(self.entropy());
        let result = hasher.result();
        result[0] >> 4
    }

    fn idxs(&self) -> [u16; 12] {
        let mut res: [u16; 12] = [0; 12];

        let mut bits_taken = 0;
        let mut bytes_taken = 0;
        let shasum = self.sha256sum() << 4;

        for idx in res.iter_mut() {
            let b1: u16 = *self.entropy().get(bytes_taken).unwrap_or(&shasum) as u16;
            let b2: u16 = *self.entropy().get(bytes_taken + 1).unwrap_or(&shasum) as u16;
            let offset = bits_taken;
            bits_taken = 3 + offset;
            bytes_taken = bytes_taken + 1;
            *idx = if offset <= 5 {
                (((b1 & mask_16!(8 - offset)) << bits_taken)
                + (b2 >> (8 - bits_taken)))
            } else if 6 <= offset && offset < 8 {
                let b3: u16 = *self.entropy().get(bytes_taken + 1).unwrap_or(&shasum) as u16;
                (((b1 & mask_16!(8 - offset)) << bits_taken)
                + (b2 << (bits_taken - 8))
                + (b3 >> (16 - bits_taken)))
            } else {
                unreachable!()
            };
            if bits_taken >= 8 {
                bits_taken = bits_taken - 8;
                bytes_taken = bytes_taken + 1;
            }
        }

        res
    }

    fn from_idxs(idxs: [u16; 12]) -> Result<Self, failure::Error> {
        let mut entropy: [u8; 16] = [0; 16];
        let mut idx = 0;
        let mut overflow: u16 = 0;
        let mut overflow_bits: u32 = 0;
        for byte in entropy.iter_mut() {
            if overflow_bits < 8 {
                let usable_bits = 8 - overflow_bits;
                overflow_bits = 11 - usable_bits;
                *byte = ((overflow << usable_bits) + (idxs[idx] >> overflow_bits)) as u8;
                overflow = idxs[idx] & mask_16!(overflow_bits);
                idx = idx + 1;
            } else {
                overflow_bits = overflow_bits - 8;
                *byte = (overflow >> overflow_bits) as u8;
                overflow = overflow & mask_16!(overflow_bits);
            }
        }
        let res = Self::new(entropy);
        ensure!(res.sha256sum() == overflow as u8, "checksum verification failed");
        Ok(res)
    }

    pub fn words(&self) -> [&'static str; 12] {
        let mut res = [""; 12];
        for (word, idx) in res.iter_mut().zip(self.idxs().into_iter()) {
            *word = consts::DICT[*idx as usize]
        }
        res
    }

    pub fn from_words(words: &[String]) -> Result<Self, failure::Error> {
        let dict_vec: Vec<&'static str> = consts::DICT.to_vec();
        let word_iter = words.into_iter().map(|s| dict_vec.binary_search(&s.as_str()).map_err(|_| format_err!("{} is not a valid bip39 word", s))).collect::<Result<Vec<usize>, failure::Error>>()?;
        let mut idxs: [u16; 12] = [0; 12];
        for (idx, word_idx) in idxs.iter_mut().zip(word_iter) {
            *idx = word_idx as u16;
        }
        Ok(Self::from_idxs(idxs)?)
    }

    pub fn long_seed(&mut self) -> &[u8] {
        match self.long_seed {
            Some(ref a) => a,
            None => {
                let mut seed: [u8; 64] = [0; 64];

                pbkdf2::<HmacSha512>(self.words().join(" ").as_bytes(), b"mnemonic", 2048, &mut seed);
                self.long_seed = Some(seed);

                self.long_seed()
            },
        }
    }

    pub fn mpriv_bits(&mut self) -> &[u8] {
        &self.long_seed()[0..32]
    }

    pub fn chain_code(&mut self) -> &[u8] {
        &self.long_seed()[32..64]
    }

    pub fn mpriv(&mut self) -> &SecretKey {
        match self.mpriv {
            Some(ref a) => a,
            None => {
                self.mpriv = Some(SecretKey::parse_slice(self.mpriv_bits()).unwrap());

                self.mpriv()
            }
        }
    }


    pub fn mpub(&mut self) -> &PublicKey {
        match self.mpub {
            Some(ref a) => a,
            None => {
                self.mpub = Some(PublicKey::from_secret_key(self.mpriv()));

                self.mpub()
            },
        }
    }

    fn as_bytes(&mut self) -> Result<Vec<u8>, Error> {
        Ok(bincode::serialize(&SerializableWallet::from(self))?)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let w: SerializableWallet = bincode::deserialize(bytes)?;

        let mut entropy: [u8; 16] = [0; 16];
        entropy.clone_from_slice(w.entropy);

        let long_seed = match w.long_seed {
            Some(data) => {
                let mut long_seed_data: [u8; 64] = [0; 64];
                long_seed_data.clone_from_slice(data);
                Some(long_seed_data)
            },
            None => None,
        };

        let mpriv = match w.mpriv {
            Some(data) => {
                Some(SecretKey::parse(&data).map_err(|e| format_err!("{:?}", e))?)
            },
            None => None,
        };

        let mpub = match w.mpub {
            Some(data) => {
                Some(PublicKey::parse(&data).map_err(|e| format_err!("{:?}", e))?)
            },
            None => None,
        };

        Ok(Wallet {
            entropy,
            long_seed,
            mpriv,
            mpub,
        })
    }
}

pub fn new_wallet() -> Wallet {
    use rand::RngCore;
    use rand::rngs::EntropyRng;

    let mut res: [u8; 16] = [0; 16];
    EntropyRng::new().fill_bytes(&mut res);
    Wallet::new(res)
}

pub fn restore_seed(words: &[String]) -> Result<Wallet, Error> {
    Wallet::from_words(words)
}



#[derive(Deserialize, Serialize)]
pub struct SerializableWallet<'a> {
    entropy: &'a [u8],
    long_seed: Option<&'a [u8]>,
    mpriv: Option<[u8; 32]>,
    #[serde(with = "BigArray")]
    mpub: Option<[u8; 65]>,
}

impl<'a> From<&'a mut Wallet> for SerializableWallet<'a> {
    fn from(wallet: &'a mut Wallet) -> Self {
        let entropy = &wallet.entropy;
        let long_seed = if let Some(ref data) = wallet.long_seed { Some(data as &[u8]) } else { None };
        let mpriv = wallet.mpriv.clone().map(|k| k.serialize());
        let mpub = wallet.mpub.clone().map(|k| k.serialize());
        SerializableWallet {
            entropy,
            long_seed,
            mpriv,
            mpub,
        }
    }
}
