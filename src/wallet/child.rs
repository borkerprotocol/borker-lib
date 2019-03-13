use super::HmacSha512;
use crate::big_array::BigArray;
use crate::NewBork;
use crate::Network;
use base58::ToBase58;
use failure::Error;
use hmac::Mac;
use ripemd160::{Digest, Ripemd160};
use secp256k1::curve::Scalar;
use secp256k1::{PublicKey, SecretKey};
use sha2::Sha256;

#[derive(Clone)]
pub struct ChildWallet {
    seed: [u8; 64],
    mpriv: Option<SecretKey>,
    mpub: Option<PublicKey>,
    children: Vec<Option<ChildWallet>>,
    hardened_children: Vec<Option<ChildWallet>>,
    nonce: u8,
}
impl ChildWallet {
    pub fn new(seed: [u8; 64]) -> Self {
        let mut res = ChildWallet {
            seed,
            mpriv: None,
            mpub: None,
            children: Vec::new(),
            hardened_children: Vec::new(),
            nonce: 0,
        };
        res.init();
        res
    }

    pub fn init(&mut self) {
        self.init_mpriv();
        self.init_mpub();
    }

    pub fn mpriv_bits(&self) -> &[u8] {
        &self.seed[0..32]
    }

    pub fn chain_code(&self) -> &[u8] {
        &self.seed[32..64]
    }

    fn init_mpriv(&mut self) {
        self.mpriv = Some(SecretKey::parse_slice(self.mpriv_bits()).unwrap());
    }

    pub fn mpriv(&self) -> &SecretKey {
        self.mpriv.as_ref().expect("wallet uninitialized")
    }

    fn init_mpub(&mut self) {
        self.mpub = Some(PublicKey::from_secret_key(self.mpriv()));
    }

    pub fn mpub(&self) -> &PublicKey {
        self.mpub.as_ref().expect("wallet uninitialized")
    }

    pub fn next_child(&mut self, hardened: bool) -> Result<&mut ChildWallet, Error> {
        if !hardened {
            self.load_child(self.children.len() as u32, false)
        } else {
            self.load_child(self.hardened_children.len() as u32, true)
        }
    }

    pub fn load_child(&mut self, i: u32, hardened: bool) -> Result<&mut ChildWallet, Error> {
        if !hardened {
            let min_len = i + 1;
            if (self.children.len() as u32) < min_len {
                self.children.resize(min_len as usize, None);
            }

            if self.children[i as usize].is_none() {
                let mut mac =
                    HmacSha512::new_varkey(self.chain_code()).map_err(|e| format_err!("{}", e))?;
                let mut v = self.mpub().serialize_compressed().to_vec();
                v.extend(&i.to_be_bytes());
                mac.input(&v);
                let mut l: [u8; 64] = [0; 64];
                l.clone_from_slice(mac.result().code().as_slice());
                let ll: Scalar = SecretKey::parse_slice(&l[0..32])
                    .map_err(|e| format_err!("{:?}", e))?
                    .into();
                let cpriv = ll + self.mpriv().clone().into();
                let cpriv_bytes = cpriv.b32();
                for n in 0..32 {
                    l[n] = cpriv_bytes[n];
                }
                self.children[i as usize] = Some(ChildWallet::new(l));
            }

            Ok(self.children[i as usize].as_mut().unwrap())
        } else {
            let hardened_i: u32 = 2_u32.pow(31) + i;
            let min_len = i + 1;
            if (self.hardened_children.len() as u32) < min_len {
                self.hardened_children.resize(min_len as usize, None);
            }

            if self.hardened_children[i as usize].is_none() {
                let mut mac =
                    HmacSha512::new_varkey(self.chain_code()).map_err(|e| format_err!("{}", e))?;
                let mut v = [&[0x0], &self.mpriv().serialize()[..]].concat().to_vec();
                v.extend(&hardened_i.to_be_bytes());
                mac.input(&v);
                let mut l: [u8; 64] = [0; 64];
                l.clone_from_slice(mac.result().code().as_slice());
                let ll: Scalar = SecretKey::parse_slice(&l[0..32])
                    .map_err(|e| format_err!("{:?}", e))?
                    .into();
                let cpriv = ll + self.mpriv().clone().into();
                let cpriv_bytes = cpriv.b32();
                for n in 0..32 {
                    l[n] = cpriv_bytes[n];
                }
                self.hardened_children[i as usize] = Some(ChildWallet::new(l));
            }

            Ok(self.hardened_children[i as usize].as_mut().unwrap())
        }
    }

    pub fn get_child(&self, i: u32, hardened: bool) -> Option<&ChildWallet> {
        if !hardened {
            self.children.get(i as usize).and_then(|a| a.as_ref())
        } else {
            self.hardened_children
                .get(i as usize)
                .and_then(|a| a.as_ref())
        }
    }

    pub fn pubkey_hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.input(&self.mpub().serialize_compressed()[..]);
        let sha_bytes = hasher.result();

        let mut hasher = Ripemd160::new();
        hasher.input(&sha_bytes);
        let ripe_bytes = hasher.result();

        ripe_bytes.to_vec()
    }

    pub fn address(&self, network: Network) -> String {
        let version_byte: u8 = match network {
            Network::Dogecoin => 0x1E,
            Network::Litecoin => 0x30,
            Network::Bitcoin => 0x00,
        };

        let mut addr_bytes: Vec<u8> = vec![version_byte];
        addr_bytes.extend(&self.pubkey_hash());

        let mut hasher = Sha256::new();
        hasher.input(&addr_bytes);
        let res = hasher.result();
        let mut hasher = Sha256::new();
        hasher.input(&res);
        let chksum = hasher.result();

        addr_bytes.extend(&chksum[0..4]);

        ToBase58::to_base58(addr_bytes.as_slice())
    }

    fn serializable(&self) -> Result<SerializableChildWallet, Error> {
        let seed = self.seed.clone();
        let mpriv = self.mpriv.clone().map(|k| k.serialize());
        let mpub = self.mpub.clone().map(|k| k.serialize());
        let children: Vec<Option<ByteVec>> = self
            .children
            .iter()
            .map(|c| match c {
                Some(c) => Ok(Some(ByteVec(c.as_bytes()?))),
                None => Ok(None),
            })
            .collect::<Result<Vec<Option<ByteVec>>, Error>>()?;

        let hardened_children: Vec<Option<ByteVec>> = self
            .hardened_children
            .iter()
            .map(|c| match c {
                Some(c) => Ok(Some(ByteVec(c.as_bytes()?))),
                None => Ok(None),
            })
            .collect::<Result<Vec<Option<ByteVec>>, Error>>()?;

        Ok(SerializableChildWallet {
            seed,
            mpriv,
            mpub,
            children,
            hardened_children,
            nonce: self.nonce,
        })
    }

    fn from_serializable(w: SerializableChildWallet) -> Result<Self, Error> {
        let seed = w.seed;

        let mpriv = match w.mpriv {
            Some(data) => Some(SecretKey::parse(&data).map_err(|e| format_err!("{:?}", e))?),
            None => None,
        };

        let mpub = match w.mpub {
            Some(data) => Some(PublicKey::parse(&data).map_err(|e| format_err!("{:?}", e))?),
            None => None,
        };

        let children: Vec<Option<ChildWallet>> = w
            .children
            .iter()
            .map(|c| match c {
                Some(ByteVec(ref c)) => Ok(Some(ChildWallet::from_bytes(c)?)),
                None => Ok(None),
            })
            .collect::<Result<Vec<Option<ChildWallet>>, Error>>()?;

        let hardened_children: Vec<Option<ChildWallet>> = w
            .hardened_children
            .iter()
            .map(|c| match c {
                Some(ByteVec(ref c)) => Ok(Some(ChildWallet::from_bytes(c)?)),
                None => Ok(None),
            })
            .collect::<Result<Vec<Option<ChildWallet>>, Error>>()?;

        Ok(ChildWallet {
            seed,
            mpriv,
            mpub,
            children,
            hardened_children,
            nonce: w.nonce,
        })
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        Ok(serde_cbor::to_vec(&self.serializable()?)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let w: SerializableChildWallet = serde_cbor::from_slice(bytes)?;
        Self::from_serializable(w)
    }

    // Data -> Vec<SignedTx>
    pub fn bork(
        &mut self,
        bork: Bork,
        fee_rate: usize,
        network: Network,
    ) -> Result<(Vec<Vec<u8>>, usize), Error> {
        let mut buf_vec: Vec<Vec<u8>> = Vec::new();
        let mut buf: Vec<u8> = Vec::new();
        let (address, message, ats) = match bork {
            Bork::Bork { message, ats } => {
                buf.push(0x00);
                buf.push(0x00);
                buf.push(0x03);
                buf.push(self.nonce);
                self.nonce = self.nonce + 1;
                (None, message, ats)
            }
            Bork::Reply {
                address,
                reference_nonce,
                message,
                ats,
            } => {
                buf.push(0x00);
                buf.push(0x00);
                buf.push(0x04);
                buf.push(self.nonce);
                self.nonce = self.nonce + 1;
                buf.push(reference_nonce);
                (Some(address), message, ats)
            }
            Bork::Rebork {
                address,
                reference_nonce,
                message,
                ats,
            } => {
                buf.push(0x00);
                buf.push(0x00);
                buf.push(0x08);
                buf.push(self.nonce);
                self.nonce = self.nonce + 1;
                buf.push(reference_nonce);
                (Some(address), message, ats)
            }
            Bork::Like {
                address,
                reference_nonce,
            } => {
                buf.push(0x00);
                buf.push(0x00);
                buf.push(0x07);
                buf.push(reference_nonce);

                (Some(address), "".to_owned(), vec![])
            }
        };
        for c in message.bytes() {
            buf.push(c);
            if buf.len() >= 80 {
                buf_vec.push(buf);
                buf = Vec::new();
                buf.push(0x00);
                buf.push(0x00);
                buf.push(0x04);
                buf.push(self.nonce);
                buf.push(self.nonce - 1);
                self.nonce = self.nonce + 1;
            }
        }

        Ok((Vec::new(), 0))
    }
}

#[derive(Deserialize, Serialize)]
struct ByteVec(#[serde(with = "serde_bytes")] pub Vec<u8>);

#[derive(Deserialize, Serialize)]
pub struct SerializableChildWallet {
    #[serde(with = "BigArray")]
    seed: [u8; 64],
    mpriv: Option<[u8; 32]>,
    #[serde(with = "BigArray")]
    mpub: Option<[u8; 65]>,
    children: Vec<Option<ByteVec>>,
    hardened_children: Vec<Option<ByteVec>>,
    nonce: u8,
}
