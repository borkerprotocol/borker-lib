#![feature(int_to_from_bytes)]

#[macro_use] extern crate failure;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;
use failure::Error;

mod big_array;
#[macro_use] mod macros;
mod wallet;

pub use self::wallet::{Wallet};


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

    pub fn child(&mut self, i: u32) -> Result<Vec<u8>, JsValue> {
        Ok(js_try!(self.inner.parent_mut().load_child(i)).mpriv_bits().to_vec())
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
