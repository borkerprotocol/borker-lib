mod consts;

#[derive(Clone, Debug)]
pub struct Seed(pub [u8; 16]);
impl Seed {
    pub fn new(entropy: [u8; 16]) -> Self {
        Seed(entropy)
    }

    fn entropy(&self) -> &[u8] {
        match self {
            Seed(data) => data,
        }
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

    pub fn from_words(words: [String; 12]) -> Result<Self, failure::Error> {
        let dict_vec: Vec<&'static str> = consts::DICT.to_vec();
        let word_iter = words.into_iter().map(|s| dict_vec.binary_search(&s.as_str()).map_err(|_| format_err!("{} is not a valid bip39 word", s))).collect::<Result<Vec<usize>, failure::Error>>()?;
        let mut idxs: [u16; 12] = [0; 12];
        for (idx, word_idx) in idxs.iter_mut().zip(word_iter) {
            *idx = word_idx as u16;
        }
        Ok(Self::from_idxs(idxs)?)
    }
}