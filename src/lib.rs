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

pub use self::wallet::{ChildWallet, Wallet};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockData<'a> {
    borker_txs: Vec<protocol::BorkTxData<'a>>,
    spent: Vec<protocol::UtxoId>,
    created: Vec<protocol::NewUtxo<'a>>,
}

#[wasm_bindgen]
pub fn process_block(block: String, network: Network) -> Result<JsValue, JsValue> {
    use bitcoin::consensus::encode::Decodable;

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
        borks: Vec::new(),
        spent: Vec::new(),
        created: Vec::new(),
    };
    for _ in 0..count.0 {
        let (bork, spent, created) =
            protocol::parse_tx(js_try!(Decodable::consensus_decode(&mut cur)), &timestamp);
        if let Some(bork) = bork {
            block_data.borks.push(bork);
        }
        block_data.spent.extend(spent);
        block_data.created.extend(created);
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
    pub fn toBuffer(&self) -> Result<Vec<u8>, JsValue> {
        Ok(js_try!(self.inner.as_bytes()))
    }

    #[allow(non_snake_case)]
    pub fn fromBuffer(buffer: Vec<u8>) -> Result<JsWallet, JsValue> {
        Ok(JsWallet {
            inner: js_try!(Wallet::from_bytes(&buffer)),
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

    pub fn new_bork(
        &mut self,
        data: JsValue,
        inputs: JsValue,
        fee: u64,
    ) -> Result<JsValue, JsValue> {
        use protocol::*;

        let inputs = js_try!(inputs.into_serde::<Vec<String>>());
        let inputs = js_try!(inputs
            .into_iter()
            .map(|i| hex::decode(i))
            .collect::<Result<Vec<_>, _>>());

        let op_rets = js_try!(encode(
            js_try!(NewBork::try_from(js_try!(data.into_serde::<NewBorkData>()))),
            self.inner.nonce_mut(),
        ));
        let mut txs = vec![];
        let mut prev_tx: Option<Vec<u8>> = None;
        for op_ret in op_rets {
            let tx = js_try!(self.inner.construct_signed(
                match prev_tx {
                    Some(t) => vec![t.clone()],
                    None => inputs.clone(),
                }
                .as_slice(),
                &[],
                fee,
                Some(op_ret.as_slice()),
            ));
            prev_tx = Some(tx.clone());
            txs.push(tx);
        }
        let txs = txs
            .into_iter()
            .map(|t| hex::encode(t))
            .collect::<Vec<String>>();

        Ok(js_try!(JsValue::from_serde(&txs)))
    }

}
