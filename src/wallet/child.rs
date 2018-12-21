use crate::big_array::BigArray;
use failure::Error;
use hmac::Mac;
use secp256k1::{PublicKey, SecretKey};
use secp256k1::curve::{Scalar};
use super::HmacSha512;

#[derive(Clone)]
pub struct ChildWallet {
    seed: [u8; 64],
    mpriv: Option<SecretKey>,
    mpub: Option<PublicKey>,
    children: Vec<Option<ChildWallet>>,
}
impl ChildWallet {
    pub fn new(seed: [u8; 64]) -> Self {
        let mut res = ChildWallet {
            seed,
            mpriv: None,
            mpub: None,
            children: Vec::new(),
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

    pub fn next_child(&mut self) -> Result<&mut ChildWallet, Error> {
        self.load_child(self.children.len() as u32)
    }

    pub fn load_child(&mut self, i: u32) -> Result<&mut ChildWallet, Error> {
        let min_len = i + 1;
        if (self.children.len() as u32) < min_len {
            self.children.resize(min_len as usize, None);
        }

        if self.children[i as usize].is_none() {
            let mut mac = HmacSha512::new_varkey(self.chain_code()).map_err(|e| format_err!("{}", e))?;
            let mut v = self.mpub().serialize_compressed().to_vec();
            v.extend(&i.to_be_bytes());
            mac.input(&v);
            let mut l: [u8; 64] = [0; 64];
            l.clone_from_slice(mac.result().code().as_slice());
            let ll: Scalar = SecretKey::parse_slice(&l[0..32]).map_err(|e| format_err!("{:?}", e))?.into();
            let cpriv = ll + self.mpriv().clone().into();
            let cpriv_bytes = cpriv.b32();
            for n in 0..32 {
                l[n] = cpriv_bytes[n];
            }
            self.children[i as usize] = Some(ChildWallet::new(l));
        }

        Ok(self.children[i as usize].as_mut().unwrap())
    }

    pub fn get_child(&self, i: u32) -> Option<&ChildWallet> {
        self.children.get(i as usize).and_then(|a| a.as_ref())
    }

    fn serializable(&self) -> Result<SerializableChildWallet, Error> {
        let seed = self.seed.clone();
        let mpriv = self.mpriv.clone().map(|k| k.serialize());
        let mpub = self.mpub.clone().map(|k| k.serialize());
        let children: Vec<Option<ByteVec>> =
            self.children.iter().map(|c| match c {
                Some(c) => Ok(Some(ByteVec(c.as_bytes()?))),
                None => Ok(None),
            }).collect::<Result<Vec<Option<ByteVec>>, Error>>()?;
        Ok(SerializableChildWallet {
            seed,
            mpriv,
            mpub,
            children,
        })
    }

    fn from_serializable(w: SerializableChildWallet) -> Result<Self, Error> {
        let seed = w.seed;

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

        let children: Vec<Option<ChildWallet>> =
            w.children.iter().map(|c| match c {
                Some(ByteVec(ref c)) => Ok(Some(ChildWallet::from_bytes(c)?)),
                None => Ok(None),
            }).collect::<Result<Vec<Option<ChildWallet>>, Error>>()?;

        Ok(ChildWallet {
            seed,
            mpriv,
            mpub,
            children,
        })
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        Ok(serde_cbor::to_vec(&self.serializable()?)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let w: SerializableChildWallet = serde_cbor::from_slice(bytes)?;
        Self::from_serializable(w)
    }
}

#[derive(Deserialize, Serialize)]
struct ByteVec (
    #[serde(with = "serde_bytes")]
    pub Vec<u8>,
);

#[derive(Deserialize, Serialize)]
pub struct SerializableChildWallet {
    #[serde(with = "BigArray")]
    seed: [u8; 64],
    mpriv: Option<[u8; 32]>,
    #[serde(with = "BigArray")]
    mpub: Option<[u8; 65]>,
    children: Vec<Option<ByteVec>>,
}
