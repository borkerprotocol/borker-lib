
use crate::Network;
use bigdecimal::BigDecimal;
use failure::Error;
use std::io::Write;

pub const MAGIC: [u8; 2] = [0x00, 0x00];

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BorkType {
    Bork,
    Reply,
    Extension,
    Rebork,
    Like,
    Follow,
    Unfollow,
    SetName,
    SetBio,
    SetAvatar,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BorkTxData {
    timestamp: chrono::NaiveDateTime,
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
    Reply {
        skip: Option<u64>,
        reference_nonce: u8,
        content: String,
    },
    Extension {
        reference_nonce: u8,
        content: String,
    },
    Rebork {
        skip: Option<u64>,
        reference_nonce: u8,
    },
    Like {
        skip: Option<u64>,
        reference_nonce: u8,
    },
    Follow {
        address: Vec<u8>,
    },
    Unfollow {
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

pub enum Bork {
    Bork {
        nonce: u8,
        content: String,
    },
    Reply {
        nonce: u8,
        skip: Option<u64>,
        reference_nonce: u8,
        content: String,
    },
    Extension {
        nonce: u8,
        reference_nonce: u8,
        content: String,
    },
    Rebork {
        nonce: u8,
        skip: Option<u64>,
        reference_nonce: u8,
    },
    Like {
        skip: Option<u64>,
        reference_nonce: u8,
    },
    Follow {
        address: Vec<u8>,
    },
    Unfollow {
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

pub fn encode(bork: NewBork, nonce: &mut u8, network: Network) -> Result<Vec<Vec<u8>>, Error> {
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

pub fn decode(data: &[u8]) -> Result<Option<Bork>, Error> {
    if data[0] != MAGIC[0] || data[1] != MAGIC[1] {
        return Ok(None);
    }

    // match data[2] {}
    unimplemented!()
}

pub fn parse_tx(
    tx: bitcoin::Transaction,
    time: &chrono::NaiveDateTime,
) -> Result<Option<BorkTxData>, Error> {
    unimplemented!()
}
