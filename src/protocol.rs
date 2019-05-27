
use crate::Network;
use bigdecimal::BigDecimal;
use failure::Error;
use std::io::Write;

use chrono::DateTime;
use chrono::Utc;

pub const MAGIC: [u8; 2] = [0xD0, 0x6E];

#[derive(Serialize)]
pub struct UtxoId {
    txid: String,
    index: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewUtxo<'a> {
    txid: String,
    index: u32,
    created_at: &'a DateTime<Utc>,
    address: String,
    value: u64,
    raw: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BorkType {
    SetName,
    SetBio,
    SetAvatar,
    Bork,
    Comment,
    Extension,
    Delete,
    Wag,
    Flag,
    Unflag,
    Follow,
    Unfollow,
    Block,
    Unblock,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BorkTxData<'a> {
    time: &'a DateTime<Utc>,
    txid: String,
    #[serde(rename = "type")]
    bork_type: BorkType,
    nonce: u8,
    skip: Option<u64>,
    reference_nonce: Option<u8>,
    content: Option<String>,
    value: Option<BigDecimal>,
    fee: BigDecimal,
    sender_address: String,
    recipient_address: Option<String>,
    mentions: Vec<String>,
}

#[derive(Deserialize)]
pub struct NewBorkData {
    #[serde(rename = "type")]
    bork_type: BorkType,
    skip: Option<u64>,
    reference_nonce: Option<u8>,
    content: Option<String>,
}

pub enum NewBork {
    Bork {
        content: String,
    },
    Comment {
        skip: Option<u64>,
        reference_nonce: u8,
        content: String,
    },
    Extension {
        reference_nonce: u8,
        content: String,
    },
    Delete {
        txid: Vec<u8>,
    },
    Wag {
        skip: Option<u64>,
        reference_nonce: u8,
    },
    Follow {
        address: Vec<u8>,
    },
    Unfollow {
        address: Vec<u8>,
    },
    Flag {
        txid: Vec<u8>,
    },
    Unflag {
        txid: Vec<u8>,
    },
    Block {
        address: Vec<u8>,
    },
    Unblock {
        address: Vec<u8>,
    },
    SetName {
        content: String,
    },
    SetBio {
        content: String,
    },
    SetAvatar {
        content: String,
    },
}

impl std::convert::TryFrom<NewBorkData> for NewBork {
    type Error = Error;

    fn try_from(data: NewBorkData) -> Result<Self, Self::Error> {
        match data.bork_type {
            BorkType::Bork => Ok(NewBork::Bork {
                content: data.content.ok_or(format_err!("missing content"))?,
            }),
            BorkType::Reply => Ok(NewBork::Reply {
                skip: data.skip,
                reference_nonce: data
                    .reference_nonce
                    .ok_or(format_err!("missing reference_nonce"))?,
                content: data.content.ok_or(format_err!("missing content"))?,
            }),
            BorkType::Extension => Ok(NewBork::Extension {
                reference_nonce: data
                    .reference_nonce
                    .ok_or(format_err!("missing reference_nonce"))?,
                content: data.content.ok_or(format_err!("missing content"))?,
            }),
            BorkType::Rebork => Ok(NewBork::Rebork {
                skip: data.skip,
                reference_nonce: data
                    .reference_nonce
                    .ok_or(format_err!("missing reference_nonce"))?,
            }),
            BorkType::Like => Ok(NewBork::Like {
                skip: data.skip,
                reference_nonce: data
                    .reference_nonce
                    .ok_or(format_err!("missing reference_nonce"))?,
            }),
            BorkType::Follow => Ok(NewBork::Follow {
                address: bitcoin::util::base58::from_check(
                    &data.content.ok_or(format_err!("missing content"))?,
                )?,
            }),
            BorkType::Unfollow => Ok(NewBork::Unfollow {
                address: bitcoin::util::base58::from_check(
                    &data.content.ok_or(format_err!("missing content"))?,
                )?,
            }),
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
        }
    }
}

pub fn encode(bork: NewBork, nonce: &mut u8) -> Result<Vec<Vec<u8>>, Error> {
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
            buf.push(*nonce);
            *nonce += 1;
            Some(content.into_bytes())
        }
        NewBork::Reply {
            skip,
            reference_nonce,
            content,
        } => {
            match skip {
                None => {
                    buf.push(0x04);
                    buf.push(*nonce);
                    *nonce += 1;
                }
                Some(skip) => {
                    buf.push(0x05);
                    buf.push(*nonce);
                    *nonce += 1;
                    use bitcoin::consensus::Encodable;
                    bitcoin::VarInt(skip).consensus_encode(&mut buf)?;
                }
            }
            buf.push(reference_nonce);
            Some(content.into_bytes())
        }
        NewBork::Extension {
            reference_nonce,
            content,
        } => {
            buf.push(0x06);
            buf.push(*nonce);
            buf.push(reference_nonce);
            Some(content.into_bytes())
        }
        NewBork::Follow { address } => {
            buf.push(0x07);
            buf.write(&address)?;
            None
        }
        NewBork::Unfollow { address } => {
            buf.push(0x08);
            buf.write(&address)?;
            None
        }
        NewBork::Like {
            skip,
            reference_nonce,
        } => {
            match skip {
                None => {
                    buf.push(0x09);
                }
                Some(skip) => {
                    buf.push(0x0a);
                    use bitcoin::consensus::Encodable;
                    bitcoin::VarInt(skip).consensus_encode(&mut buf)?;
                }
            }
            buf.push(reference_nonce);
            None
        }
        NewBork::Rebork {
            skip,
            reference_nonce,
        } => {
            match skip {
                None => {
                    buf.push(0x0b);
                    buf.push(*nonce);
                    *nonce += 1;
                }
                Some(skip) => {
                    buf.push(0x0c);
                    buf.push(*nonce);
                    *nonce += 1;
                    use bitcoin::consensus::Encodable;
                    bitcoin::VarInt(skip).consensus_encode(&mut buf)?;
                }
            }
            buf.push(reference_nonce);
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
            buf.push(*nonce);
            buf.push(*nonce - 1);
            *nonce += 1;
            buf.write(&c)?;
        }
        buf_vec.push(buf);
    }

    Ok(buf_vec)
}

pub fn decode<'a>(
    data: &[u8],
    out_addrs: &[String],
    from: String,
    time: &'a DateTime<Utc>,
) -> Result<Option<BorkTxData<'a>>, Error> {
    if data[0] != MAGIC[0] || data[1] != MAGIC[1] {
        return Ok(None);
    }

    // match data[2] {}
    unimplemented!()
}

pub fn parse_tx<'a>(
    tx: bitcoin::Transaction,
    time: &'a DateTime<Utc>,
) -> (Option<BorkTxData<'a>>, Vec<UtxoId>, Vec<NewUtxo<'a>>) {
    use bitcoin::consensus::Encodable;

    let mut tx_data: Vec<u8> = Vec::new();
    tx.consensus_encode(&mut tx_data).unwrap();
    let tx_hex = hex::encode(tx_data);
    let txid = format!("{:x}", tx.txid());
    let mut op_ret = None;
    let mut spent = Vec::new();
    let mut created = Vec::new();
    for (idx, o) in tx.output.iter().enumerate() {
        if o.script_pubkey.is_p2pkh() {
            created.push(NewUtxo {
                txid: txid.clone(),
                index: idx as u32,
                address: crate::wallet::script_to_addr(&o.script_pubkey).unwrap(),
                value: o.value,
                created_at: time,
                raw: tx_hex.clone(),
            });
        } else if o.script_pubkey.is_op_return() {
            let b = o.script_pubkey.as_bytes();
            match b[1] {
                0x4c => op_ret = Some(&b[3..]),
                0x4d => op_ret = Some(&b[4..]),
                0x4e => op_ret = Some(&b[6..]),
                _ => op_ret = Some(&b[2..]),
            };
        }
    }
    for (idx, i) in tx.input.into_iter().enumerate() {
        spent.push(UtxoId {
            txid: format!("{:x}", i.previous_output.txid),
            index: idx as u32,
        });
    }

    let bork = op_ret.and_then(|data| {
        decode(
            data,
            &created
                .iter()
                .map(|c| c.address.clone())
                .collect::<Vec<_>>(),
            String::new(),
            time,
        )
        .ok()
        .and_then(|a| a)
    });

    (bork, spent, created)
}
