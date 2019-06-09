
use super::addr_to_script;

use super::pubkey_hash_to_addr;
use super::HmacSha512;
use crate::big_array::BigArray;
use crate::Network;
use failure::Error;
use hmac::Mac;
use ripemd160::Digest;
use ripemd160::Ripemd160;
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
        use rand::Rng;
        let mut rng = rand::rngs::EntropyRng::new();
        let mut res = ChildWallet {
            seed,
            mpriv: None,
            mpub: None,
            children: Vec::new(),
            hardened_children: Vec::new(),
            nonce: rng.gen(),
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

    pub fn nonce(&mut self) -> u8 {
        self.nonce = self.nonce.wrapping_add(1);
        self.nonce
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
        pubkey_hash_to_addr(&self.pubkey_hash(), network)
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

    pub fn script(&self) -> bitcoin::Script {
        addr_to_script(&self.address(Network::Bitcoin)).unwrap()
    }

    pub fn construct_signed(
        &self,
        inputs: &[Vec<u8>],
        outputs: &[(&str, u64)],
        fee: u64,
        op_return: Option<&[u8]>,
    ) -> Result<Vec<u8>, Error> {
        use bitcoin::consensus::Decodable;
        use bitcoin::consensus::Encodable;
        use bitcoin::{OutPoint, Transaction, TxIn, TxOut};
        use bitcoin_hashes::Hash;
        use std::io::Cursor;

        let script = self.script();

        let inputs = inputs
            .into_iter()
            .map(|i| Transaction::consensus_decode(&mut Cursor::new(i)))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flat_map(|tx| {
                tx.output
                    .clone()
                    .into_iter()
                    .enumerate()
                    .filter(|(_, o)| o.script_pubkey == script)
                    .map(|(vout, o)| {
                        (
                            OutPoint {
                                txid: tx.txid(),
                                vout: vout as u32,
                            },
                            o,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let input_size = inputs.iter().fold(0, |acc, tx| acc + tx.1.value);
        let output_size = outputs.iter().fold(0, |acc, o| acc + o.1);
        if output_size > input_size - fee || input_size < fee {
            bail!("insufficient funds")
        }
        let mut outputs = outputs.iter().cloned().collect::<Vec<_>>();
        let address = self.address(Network::Bitcoin);
        outputs.push((address.as_str(), input_size - output_size - fee));

        let output = outputs
            .into_iter()
            .map(|(addr, val)| -> Result<_, Error> {
                Ok(TxOut {
                    script_pubkey: addr_to_script(addr)?,
                    value: val,
                })
            })
            .chain(op_return.into_iter().map(|data| -> Result<_, Error> {
                let mut s: Vec<u8> = vec![0x6a, 0x4c, data.len() as u8];
                s.extend(data.iter());
                Ok(TxOut {
                    script_pubkey: bitcoin::Script::from(s),
                    value: 0,
                })
            }))
            .collect::<Result<Vec<_>, Error>>()?;
        let input: Vec<TxIn> = inputs
            .into_iter()
            .map(|a| TxIn {
                previous_output: a.0,
                script_sig: bitcoin::Script::new(),
                sequence: 0xFFFFFFFF_u32,
                witness: vec![],
            })
            .collect();
        let mut tx = Transaction {
            version: 1,
            lock_time: 0,
            input: input.clone(),
            output,
        };
        tx.input = input
            .into_iter()
            .enumerate()
            .map(|(i, vin)| -> Result<_, Error> {
                let sighash = tx.signature_hash(i, &script, 0x01).into_inner();
                let sig = secp256k1::sign(&secp256k1::Message::parse(&sighash), self.mpriv())
                    .map_err(|e| format_err!("{:?}", e))?
                    .0;
                let pubkey = self.mpub().serialize_compressed();
                let sig_der = sig.serialize_der();
                let script_sig = bitcoin::Script::from(
                    [
                        &[sig_der.as_ref().len() as u8 + 1][..],
                        sig_der.as_ref(),
                        &[0x01, pubkey.len() as u8][..],
                        &pubkey[..],
                    ]
                    .concat(),
                );
                Ok(TxIn {
                    previous_output: vin.previous_output,
                    script_sig,
                    sequence: vin.sequence,
                    witness: vin.witness,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let mut res = Vec::new();
        tx.consensus_encode(&mut res)?;
        Ok(res)
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
