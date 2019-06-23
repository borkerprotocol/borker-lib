#![feature(slice_concat_ext)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;

use failure::Error;
use std::convert::TryFrom;
use wasm_bindgen::prelude::*;

mod big_array;
#[macro_use]
mod macros;
mod protocol;
mod wallet;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

}

pub use self::wallet::{ChildWallet, Wallet};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockData<'a> {
    borker_txs: Vec<protocol::BorkTxData<'a>>,
    spent: Vec<Vec<protocol::UtxoId>>,
    created: Vec<Vec<protocol::NewUtxo>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    address: String,
    value: u64,
}
impl Output {
    pub fn as_tup(&self) -> (&str, u64) {
        (self.address.as_str(), self.value)
    }
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn processBlock(
    block: String,
    block_height: f64,
    network: Network,
) -> Result<JsValue, JsValue> {
    use bitcoin::consensus::encode::Decodable;

    let block = js_try!(hex::decode(&block));
    let mut cur = std::io::Cursor::new(&block);
    let block_header: bitcoin::BlockHeader = js_try!(Decodable::consensus_decode(&mut cur));
    match network {
        Network::Dogecoin | Network::Litecoin if block_header.version & 1 << 8 != 0 => {
            let _: bitcoin::Transaction = js_try!(Decodable::consensus_decode(&mut cur));
            let pos = cur.position() + 32;
            cur.set_position(pos);
            let len: bitcoin::VarInt = js_try!(Decodable::consensus_decode(&mut cur));
            let pos = cur.position() + 32 * len.0;
            cur.set_position(pos + 4);

            let len: bitcoin::VarInt = js_try!(Decodable::consensus_decode(&mut cur));
            let pos = cur.position() + 32 * len.0;
            cur.set_position(pos + 4);
            let _: bitcoin::BlockHeader = js_try!(Decodable::consensus_decode(&mut cur));
        }
        _ => (),
    }

    let count: bitcoin::VarInt = js_try!(Decodable::consensus_decode(&mut cur));
    let timestamp = chrono::DateTime::from_utc(
        chrono::NaiveDateTime::from_timestamp(block_header.time as i64, 0),
        chrono::Utc,
    );
    let mut block_data = BlockData {
        borker_txs: Vec::new(),
        spent: Vec::new(),
        created: Vec::new(),
    };
    let mut spent_inner = Vec::new();
    let mut created_inner = Vec::new();
    for _ in 0..count.0 {
        let (bork, mut spent, mut created) = protocol::parse_tx(
            js_try!(Decodable::consensus_decode(&mut cur)),
            &timestamp,
            block_height as u64,
            network,
        );
        if let Some(bork) = bork {
            block_data.borker_txs.push(bork);
        }
        while spent_inner.len() + spent.len() >= 100 {
            let at = spent_inner.len() + spent.len() - 100;
            spent_inner.extend(spent.split_off(at));
            block_data.spent.push(spent_inner);
            spent_inner = Vec::new();
        }
        spent_inner.extend(spent);
        while created_inner.len() + created.len() >= 100 {
            let at = created_inner.len() + created.len() - 100;
            created_inner.extend(created.split_off(at));
            block_data.created.push(created_inner);
            created_inner = Vec::new();
        }
        created_inner.extend(created);
    }
    if spent_inner.len() > 0 {
        block_data.spent.push(spent_inner);
    }
    if created_inner.len() > 0 {
        block_data.created.push(created_inner);
    }
    Ok(js_try!(JsValue::from_serde(&block_data)))
}

// JS Wrappers

#[wasm_bindgen]
pub struct JsWallet {
    inner: Wallet,
}

#[wasm_bindgen]
impl JsWallet {
    #[wasm_bindgen(constructor)]
    pub fn new(words: Option<Vec<JsValue>>) -> Result<JsWallet, JsValue> {
        console_error_panic_hook::set_once();
        Ok(match words {
            Some(w) => JsWallet {
                inner: js_try!(Wallet::from_words(&js_try!(w
                    .iter()
                    .map(|a| a.into_serde::<String>().map_err(Error::from))
                    .collect::<Result<Vec<String>, Error>>()))),
            },
            None => JsWallet {
                inner: Wallet::new(),
            },
        })
    }

    pub fn words(&self) -> Vec<JsValue> {
        self.inner
            .words()
            .into_iter()
            .map(|a| JsValue::from_serde(a).unwrap())
            .collect()
    }

    #[allow(non_snake_case)]
    pub fn childAt(&mut self, derivation_path: Vec<f64>) -> Result<JsChildWallet, JsValue> {
        let mut cur: &mut ChildWallet = &mut self.inner.parent_mut();

        for idx in derivation_path {
            cur = js_try!(cur.load_child(idx.abs() as u32, idx.is_sign_negative()))
        }
        Ok(JsChildWallet { inner: cur.clone() })
    }

    #[allow(non_snake_case)]
    pub fn toBuffer(&self) -> Result<String, JsValue> {
        Ok(hex::encode(js_try!(self.inner.as_bytes())))
    }

    #[allow(non_snake_case)]
    pub fn fromBuffer(buffer: String) -> Result<JsWallet, JsValue> {
        Ok(JsWallet {
            inner: js_try!(Wallet::from_bytes(&js_try!(hex::decode(&buffer)))),
        })
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum Network {
    Dogecoin,
    Litecoin,
    Bitcoin,
}

#[wasm_bindgen]
pub struct JsChildWallet {
    inner: ChildWallet,
}

#[wasm_bindgen]
impl JsChildWallet {
    pub fn address(&self, network: Network) -> String {
        self.inner.address(network)
    }

    #[allow(non_snake_case)]
    pub fn newBork(
        &mut self,
        data: JsValue,
        inputs: JsValue,
        recipient: JsValue,
        mentions: JsValue,
        fee: f64,
        network: Network,
    ) -> Result<JsValue, JsValue> {
        use protocol::*;

        let inputs = js_try!(inputs.into_serde::<Vec<String>>());
        let inputs = js_try!(inputs
            .into_iter()
            .map(|i| hex::decode(i))
            .collect::<Result<Vec<_>, _>>());

        let mut outputs = js_try!(recipient.into_serde::<Option<Output>>())
            .into_iter()
            .collect::<Vec<Output>>();
        outputs.extend(js_try!(mentions.into_serde::<Vec<Output>>()));

        let op_rets = js_try!(encode(
            js_try!(NewBork::try_from(js_try!(data.into_serde::<NewBorkData>()))),
            self.inner.nonce(),
        ));
        let mut txs = vec![];
        let mut prev_tx: Option<Vec<u8>> = None;
        let mut o = outputs.iter().map(|o| o.as_tup()).collect::<Vec<_>>();
        for op_ret in op_rets {
            let tx = js_try!(self.inner.construct_signed(
                match prev_tx {
                    Some(t) => vec![t.clone()],
                    None => inputs.clone(),
                }
                .as_slice(),
                &o,
                fee as u64,
                Some(op_ret.as_slice()),
                network,
            ));
            prev_tx = Some(tx.clone());
            txs.push(tx);
            o = Vec::new();
        }
        let txs = txs
            .into_iter()
            .map(|t| hex::encode(t))
            .collect::<Vec<String>>();

        Ok(js_try!(JsValue::from_serde(&txs)))
    }

    #[allow(non_snake_case)]
    pub fn constructSigned(
        &self,
        inputs: JsValue,
        destination: String,
        amount: f64,
        fee: f64,
        network: Network,
    ) -> Result<String, JsValue> {
        let inputs = js_try!(inputs.into_serde::<Vec<String>>());
        let inputs = js_try!(inputs
            .into_iter()
            .map(|i| hex::decode(i))
            .collect::<Result<Vec<_>, _>>());

        let signed = js_try!(self.inner.construct_signed(
            &inputs,
            &[(destination.as_str(), amount as u64)],
            fee as u64,
            None,
            network
        ));
        Ok(hex::encode(signed))
    }

}
