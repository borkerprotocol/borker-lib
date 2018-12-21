#[macro_use] extern crate failure;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate wasm_bindgen;

use failure::Error;
use wasm_bindgen::prelude::*;

mod big_array;
#[macro_use] mod macros;
mod wallet;

pub use self::wallet::{ChildWallet, Wallet};


// JS Wrappers

#[wasm_bindgen]
pub struct JsWallet {
    inner: Wallet
}

#[wasm_bindgen]
impl JsWallet {

    #[wasm_bindgen(constructor)]
    pub fn new(words: Option<Vec<JsValue>>) -> Result<JsWallet, JsValue> {
        Ok(match words {
            Some(w) => JsWallet {
                inner: js_try!(Wallet::from_words(&js_try!(w.iter().map(|a| a.into_serde::<String>().map_err(Error::from)).collect::<Result<Vec<String>, Error>>())))
            },
            None => JsWallet {
                inner: Wallet::new()
            },
        })
    }

    pub fn words(&self) -> Vec<JsValue> {
        self.inner.words().into_iter().map(|a| JsValue::from_serde(a).unwrap()).collect()
    }

    #[allow(non_snake_case)]
    pub fn childAt(&mut self, derivation_path: Vec<f64>) -> Result<JsChildWallet, JsValue> {
        let mut cur: &mut ChildWallet = &mut self.inner.parent_mut();

        for idx in derivation_path {
            cur = js_try!(cur.load_child(idx.abs() as u32, idx.is_sign_negative()))
        }
        Ok(JsChildWallet {
            inner: cur.clone(),
        })
    }

    #[allow(non_snake_case)]
    pub fn toBuffer(&self) -> Result<Vec<u8>, JsValue> {
        Ok(js_try!(self.inner.as_bytes()))
    }

    #[allow(non_snake_case)]
    pub fn fromBuffer(buffer: Vec<u8>) -> Result<JsWallet, JsValue> {
        Ok(JsWallet {
            inner: js_try!(Wallet::from_bytes(&buffer))
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
    inner: ChildWallet
}

#[wasm_bindgen]
impl JsChildWallet {
    pub fn address(&self, network: Network) -> String {
        self.inner.address(network)
    }
}
