
use crate::Network;
use failure::Error;
use std::io::Write;

use crate::wallet::addr_to_script;
use crate::wallet::is_p2pkh;
use crate::wallet::pubkey_hash_to_addr;
use crate::wallet::pubkey_to_addr;
use chrono::DateTime;
use chrono::Utc;
use std::collections::HashSet;

pub const MAGIC: [u8; 2] = [0xD0, 0x6E];

#[derive(Debug, Serialize)]
pub struct UtxoId {
    txid: String,
    position: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewUtxo {
    block_height: u64,
    txid: String,
    position: u32,
    address: String,
    value: u64,
    raw: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BorkType {
    SetName,
    SetBio,
    SetAvatar,
    Bork,
    Comment,
    Rebork,
    Extension,
    Delete,
    Like,
    Flag,
    Follow,
    Block,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BorkTxData<'a> {
    time: &'a DateTime<Utc>,
    txid: String,
    #[serde(rename = "type")]
    bork_type: BorkType,
    nonce: Option<u8>,
    position: Option<u8>,
    reference_id: Option<String>,
    content: Option<String>,
    sender_address: String,
    recipient_address: Option<String>,
    mentions: Vec<String>,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewBorkData {
    #[serde(rename = "type")]
    bork_type: BorkType,
    content: Option<String>,
    reference_id: Option<String>,
}

pub enum NewBork {
    SetName {
        content: String,
    },
    SetBio {
        content: String,
    },
    SetAvatar {
        content: String,
    },
    Bork {
        content: String,
    },
    Comment {
        reference_id: Vec<u8>,
        content: String,
    },
    Rebork {
        reference_id: Vec<u8>,
        content: String,
    },
    Delete {
        reference_id: Vec<u8>,
    },
    Like {
        reference_id: Vec<u8>,
    },
    Follow {
        address: Vec<u8>,
    },
    Block {
        address: Vec<u8>,
    },
    Flag {
        txid: Vec<u8>,
    },
}

impl std::convert::TryFrom<NewBorkData> for NewBork {
    type Error = Error;

    fn try_from(data: NewBorkData) -> Result<Self, Self::Error> {
        match data.bork_type {
            BorkType::Extension => bail!("cannot directly generate an extension"),
            BorkType::SetName => {
                let content = data.content.ok_or(format_err!("missing content"))?;
                if content.as_bytes().len() > 77 {
                    bail!("content exceeds maximum length");
                }
                Ok(NewBork::SetName { content })
            }
            BorkType::SetBio => {
                let content = data.content.ok_or(format_err!("missing content"))?;
                if content.as_bytes().len() > 77 {
                    bail!("content exceeds maximum length");
                }
                Ok(NewBork::SetBio { content })
            }
            BorkType::SetAvatar => {
                let content = data.content.ok_or(format_err!("missing content"))?;
                if content.as_bytes().len() > 77 {
                    bail!("content exceeds maximum length");
                }
                Ok(NewBork::SetAvatar { content })
            }
            BorkType::Bork => Ok(NewBork::Bork {
                content: data.content.ok_or(format_err!("missing content"))?,
            }),
            BorkType::Comment => {
                let reference_id = hex::decode(
                    &data
                        .reference_id
                        .ok_or(format_err!("missing reference_id"))?,
                )?;
                if reference_id.len() > 32 {
                    bail!("reference_id exceeds maximum length");
                }
                Ok(NewBork::Comment {
                    content: data.content.ok_or(format_err!("missing content"))?,
                    reference_id,
                })
            }
            BorkType::Rebork => {
                let reference_id = hex::decode(
                    &data
                        .reference_id
                        .ok_or(format_err!("missing reference_id"))?,
                )?;
                if reference_id.len() > 32 {
                    bail!("reference_id exceeds maximum length");
                }
                Ok(NewBork::Rebork {
                    content: data.content.ok_or(format_err!("missing content"))?,
                    reference_id,
                })
            }
            BorkType::Delete => {
                let reference_id = hex::decode(
                    &data
                        .reference_id
                        .ok_or(format_err!("missing reference_id"))?,
                )?;
                if reference_id.len() > 32 {
                    bail!("reference_id exceeds maximum length");
                }
                Ok(NewBork::Delete { reference_id })
            }
            BorkType::Like => {
                let reference_id = hex::decode(
                    &data
                        .reference_id
                        .ok_or(format_err!("missing reference_id"))?,
                )?;
                if reference_id.len() > 32 {
                    bail!("reference_id exceeds maximum length");
                }
                Ok(NewBork::Like { reference_id })
            }
            BorkType::Flag => {
                let txid = hex::decode(&data.reference_id.ok_or(format_err!("missing txid"))?)?;
                if txid.len() != 32 {
                    bail!("invalid length for txid");
                }
                Ok(NewBork::Flag { txid })
            }
            BorkType::Follow => {
                let mut address = bitcoin::util::base58::from_check(
                    &data.content.ok_or(format_err!("missing content"))?,
                )?;
                if !is_p2pkh(address.remove(0)) {
                    bail!("address is not P2PKH");
                }
                Ok(NewBork::Follow { address })
            }
            BorkType::Block => {
                let mut address = bitcoin::util::base58::from_check(
                    &data.content.ok_or(format_err!("missing content"))?,
                )?;
                if !is_p2pkh(address.remove(0)) {
                    bail!("address is not P2PKH");
                }
                Ok(NewBork::Block { address })
            }
        }
    }
}

pub fn encode(bork: NewBork, nonce: u8) -> Result<Vec<Vec<u8>>, Error> {
    let mut buf_vec: Vec<Vec<u8>> = Vec::new();
    let mut buf: Vec<u8> = Vec::new();
    buf.push(MAGIC[0]);
    buf.push(MAGIC[1]);
    let content: Option<Vec<u8>> = match bork {
        NewBork::SetName { content } => {
            buf.push(0x00);
            buf.write(content.as_bytes())?;
            None
        }
        NewBork::SetBio { content } => {
            buf.push(0x01);
            buf.write(content.as_bytes())?;
            None
        }
        NewBork::SetAvatar { content } => {
            buf.push(0x02);
            buf.write(content.as_bytes())?;
            None
        }
        NewBork::Bork { content } => {
            buf.push(0x03);
            buf.push(nonce);
            Some(content.into_bytes())
        }
        NewBork::Comment {
            reference_id,
            content,
        } => {
            buf.push(0x04);
            buf.push(nonce);
            buf.push(reference_id.len() as u8);
            buf.extend(reference_id);
            Some(content.into_bytes())
        }
        NewBork::Rebork {
            reference_id,
            content,
        } => {
            buf.push(0x05);
            buf.push(nonce);
            buf.push(reference_id.len() as u8);
            buf.extend(reference_id);
            Some(content.into_bytes())
        }
        NewBork::Like { reference_id } => {
            buf.push(0x07);
            buf.push(reference_id.len() as u8);
            buf.extend(reference_id);
            None
        }
        NewBork::Flag { txid } => {
            buf.push(0x08);
            buf.extend(txid);
            None
        }
        NewBork::Follow { address } => {
            buf.push(0x09);
            buf.extend(address);
            None
        }
        NewBork::Block { address } => {
            buf.push(0x0A);
            buf.extend(address);
            None
        }
        NewBork::Delete { reference_id } => {
            buf.push(0x0B);
            buf.push(reference_id.len() as u8);
            buf.extend(reference_id);
            None
        }
    };
    if let Some(content) = content {
        let remaining = (80 - buf.len()).min(content.len());
        buf.write(&content[..remaining])?;
        for c in content[remaining..].chunks(75) {
            buf_vec.push(buf);
            buf = Vec::new();
            buf.push(MAGIC[0]);
            buf.push(MAGIC[1]);
            buf.push(0x06);
            buf.push(nonce);
            buf.push(buf_vec.len() as u8);
            buf.write(&c)?;
        }
        buf_vec.push(buf);
    }

    Ok(buf_vec)
}

struct Cur<'a, T: Clone>(&'a [T], usize);
impl<'a, T> Cur<'a, T>
where
    T: Clone,
{
    pub fn peek(&self) -> Result<T, Error> {
        self.0
            .get(self.1)
            .map(|a: &T| -> T { a.clone() })
            .ok_or(format_err!("unexpected end of input"))
    }

    pub fn peek_n(&self, n: usize) -> Result<&'a [T], Error> {
        self.0
            .get(self.1..(self.1 + n))
            .ok_or(format_err!("unexpected end of input"))
    }

    pub fn next(&mut self) -> Result<T, Error> {
        let ret = self.peek()?;
        self.1 += 1;
        Ok(ret)
    }

    pub fn next_n(&mut self, n: usize) -> Result<&[T], Error> {
        let ret = self.peek_n(n)?;
        self.1 += n;
        Ok(ret)
    }

    pub fn rest(self) -> &'a [T] {
        &self.0[self.1..]
    }
}
impl<'a> Cur<'a, u8> {

    pub fn var_peek(&self) -> Result<&'a [u8], Error> {
        let len = *self
            .0
            .get(self.1)
            .ok_or(format_err!("unexpected end of input"))?;
        self.0
            .get((self.1 + 1)..(self.1 + 1 + len as usize))
            .ok_or(format_err!("unexpected end of input"))
    }

    pub fn var_next(&mut self) -> Result<&'a [u8], Error> {
        let ret = self.var_peek()?;
        self.1 += 1 + ret.len();
        Ok(ret)
    }
}

pub fn get_tags(body: &str) -> Vec<String> {
    let mut res = HashSet::new();
    let mut tag = String::new();
    let mut in_tag = false;
    for c in body.chars() {
        if c == '#' {
            if in_tag && tag.len() > 0 {
                res.insert(tag);
                tag = String::new();
            } else {
                in_tag = true;
            }
            continue;
        }
        if in_tag {
            if c == ' ' || c == '\t' || c == '\n' {
                if tag.len() > 0 {
                    res.insert(tag);
                    tag = String::new();
                }
                in_tag = false;
            } else {
                for c in c.to_lowercase() {
                    tag.push(c);
                }
            }
        }
    }
    if tag.len() > 0 {
        res.insert(tag);
    }
    res.into_iter().collect()
}

pub fn decode<'a>(
    data: &[u8],
    out_addrs: &[&str],
    txid: String,
    from: String,
    time: &'a DateTime<Utc>,
    network: Network,
) -> Result<BorkTxData<'a>, Error> {
    let mut data = Cur(data, 0);
    let mut out_addrs = Cur(out_addrs, 0);
    if data.next()? != MAGIC[0] || data.next()? != MAGIC[1] {
        bail!("invalid version");
    }
    let mut res = match data.next()? {
        0x00 => BorkTxData {
            bork_type: BorkType::SetName,
            content: Some(std::str::from_utf8(data.rest())?.to_owned()),
            position: None,
            mentions: Vec::new(),
            nonce: None,
            recipient_address: None,
            reference_id: None,
            sender_address: from,
            time,
            txid,
            tags: Vec::new(),
        },
        0x01 => BorkTxData {
            bork_type: BorkType::SetBio,
            content: Some(std::str::from_utf8(data.rest())?.to_owned()),
            position: None,
            mentions: Vec::new(),
            nonce: None,
            recipient_address: None,
            reference_id: None,
            sender_address: from,
            time,
            txid,
            tags: Vec::new(),
        },
        0x02 => BorkTxData {
            bork_type: BorkType::SetAvatar,
            content: Some(std::str::from_utf8(data.rest())?.to_owned()),
            position: None,
            mentions: Vec::new(),
            nonce: None,
            recipient_address: None,
            reference_id: None,
            sender_address: from,
            time,
            txid,
            tags: Vec::new(),
        },
        0x03 => BorkTxData {
            bork_type: BorkType::Bork,
            position: Some(0),
            mentions: out_addrs
                .rest()
                .into_iter()
                .map(|a| a.clone().to_owned())
                .collect(),
            nonce: Some(data.next()?),
            content: Some(std::str::from_utf8(data.rest())?.to_owned()),
            recipient_address: None,
            reference_id: None,
            sender_address: from,
            time,
            txid,
            tags: Vec::new(),
        },
        0x04 => BorkTxData {
            bork_type: BorkType::Comment,
            position: Some(0),
            recipient_address: Some(out_addrs.next()?.to_owned()),
            mentions: out_addrs
                .rest()
                .into_iter()
                .map(|a| a.clone().to_owned())
                .collect(),
            nonce: Some(data.next()?),
            reference_id: Some(hex::encode(data.var_next()?)),
            content: Some(std::str::from_utf8(data.rest())?.to_owned()),
            sender_address: from,
            time,
            txid,
            tags: Vec::new(),
        },
        0x05 => BorkTxData {
            bork_type: BorkType::Rebork,
            position: Some(0),
            recipient_address: Some(out_addrs.next()?.to_owned()),
            mentions: out_addrs
                .rest()
                .into_iter()
                .map(|a| a.clone().to_owned())
                .collect(),
            nonce: Some(data.next()?),
            reference_id: Some(hex::encode(data.var_next()?)),
            content: Some(std::str::from_utf8(data.rest())?.to_owned()),
            sender_address: from,
            time,
            txid,
            tags: Vec::new(),
        },
        0x06 => BorkTxData {
            bork_type: BorkType::Extension,
            recipient_address: None,
            mentions: out_addrs
                .rest()
                .into_iter()
                .map(|a| a.clone().to_owned())
                .collect(),
            nonce: Some(data.next()?),
            position: Some(data.next()?),
            reference_id: None,
            content: Some(std::str::from_utf8(data.rest())?.to_owned()),
            sender_address: from,
            time,
            txid,
            tags: Vec::new(),
        },
        0x07 => BorkTxData {
            bork_type: BorkType::Like,
            recipient_address: Some(out_addrs.next()?.to_owned()),
            mentions: Vec::new(),
            nonce: None,
            position: None,
            reference_id: Some(hex::encode(data.var_next()?)),
            content: None,
            sender_address: from,
            time,
            txid,
            tags: Vec::new(),
        },
        0x08 => BorkTxData {
            bork_type: BorkType::Flag,
            recipient_address: None,
            mentions: Vec::new(),
            nonce: None,
            position: None,
            reference_id: Some(hex::encode(data.next_n(32)?)),
            content: None,
            sender_address: from,
            time,
            txid,
            tags: Vec::new(),
        },
        0x09 => BorkTxData {
            bork_type: BorkType::Follow,
            recipient_address: None,
            mentions: Vec::new(),
            nonce: None,
            position: None,
            reference_id: None,
            content: Some(pubkey_hash_to_addr(data.next_n(20)?, network)),
            sender_address: from,
            time,
            txid,
            tags: Vec::new(),
        },
        0x0A => BorkTxData {
            bork_type: BorkType::Block,
            recipient_address: None,
            mentions: Vec::new(),
            nonce: None,
            position: None,
            reference_id: None,
            content: Some(pubkey_hash_to_addr(data.next_n(20)?, network)),
            sender_address: from,
            time,
            txid,
            tags: Vec::new(),
        },
        0x0B => BorkTxData {
            bork_type: BorkType::Delete,
            recipient_address: None,
            mentions: Vec::new(),
            nonce: None,
            position: None,
            reference_id: Some(hex::encode(data.var_next()?)),
            content: None,
            sender_address: from,
            time,
            txid,
            tags: Vec::new(),
        },
        _ => bail!("invalid message type"),
    };
    res.tags = res
        .content
        .as_ref()
        .map(|s| get_tags(s.as_str()))
        .unwrap_or_default();
    Ok(res)
}

pub fn parse_tx<'a>(
    tx: bitcoin::Transaction,
    time: &'a DateTime<Utc>,
    block_height: u64,
    network: Network,
) -> (Option<BorkTxData<'a>>, Vec<UtxoId>, Vec<NewUtxo>) {
    use bitcoin::consensus::Encodable;

    let mut tx_data: Vec<u8> = Vec::new();
    tx.consensus_encode(&mut tx_data).unwrap();
    let from = {
        tx.input.get(0).and_then(|i| {
            use bitcoin_hashes::Hash;

            let mut script = i.script_sig.iter(true);
            let mut sig = match script.next() {
                Some(bitcoin::blockdata::script::Instruction::PushBytes(b)) => b.to_vec(),
                _ => return None,
            };
            let pubkey = match script.next() {
                Some(bitcoin::blockdata::script::Instruction::PushBytes(b)) => b,
                _ => return None,
            };
            if sig.len() == 0 {
                return None;
            }
            let sighash_type = sig.remove(sig.len() - 1);
            let addr = pubkey_to_addr(pubkey, network);
            let msg = secp256k1::Message::parse_slice(
                &tx.signature_hash(0, &addr_to_script(&addr).ok()?, sighash_type as u32)
                    .into_inner(),
            )
            .ok()?;
            if !secp256k1::verify(
                &msg,
                &secp256k1::Signature::parse_der_lax(&sig).ok()?,
                &secp256k1::PublicKey::parse_slice(
                    pubkey,
                    Some(secp256k1::PublicKeyFormat::Compressed),
                )
                .ok()?,
            ) {
                return None;
            }

            Some(addr)
        })
    };
    let tx_hex = hex::encode(tx_data);
    let txid = format!("{:x}", tx.txid());
    let mut op_ret = None;
    let mut spent = Vec::new();
    let mut created = Vec::new();
    for (idx, o) in tx.output.iter().enumerate() {
        if o.script_pubkey.is_p2pkh() {
            created.push(NewUtxo {
                block_height,
                txid: txid.clone(),
                position: idx as u32,
                address: crate::wallet::script_to_addr(&o.script_pubkey, network).unwrap(),
                value: o.value,
                raw: tx_hex.clone(),
            });
        } else if o.script_pubkey.is_op_return() {
            let b = o.script_pubkey.as_bytes();
            match b.get(1) {
                Some(0x4c) => op_ret = b.get(3..),
                Some(0x4d) => op_ret = b.get(4..),
                Some(0x4e) => op_ret = b.get(6..),
                _ => op_ret = b.get(2..),
            };
        }
    }
    for i in tx.input.into_iter() {
        spent.push(UtxoId {
            txid: format!("{:x}", i.previous_output.txid),
            position: i.previous_output.vout,
        });
    }

    let bork = op_ret.and_then(|data| {
        from.and_then(|from| {
            decode(
                data,
                created
                    .iter()
                    .map(|c| c.address.as_str())
                    .filter(|a| a != &from.as_str())
                    .collect::<Vec<&str>>()
                    .as_slice(),
                txid,
                from,
                time,
                network,
            )
            .ok()
        })
    });


    (bork, spent, created)
}
