extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{Request, Response, ResponseInit};

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc:WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

/*
#[wasm_bindgen]
pub fn handle(kv: JsValue, req: JsValue) -> Result<Response, JsValue> {
    let req: Request = req.dyn_into()?;
    let kv: KVNamespace = kv.dyn_into()?;

    let mut init = ResponseInit::new();
    init.status(200);
    init.headers(&JsValue::from_serde(&json!({
        "content-type": "text/plain",
    }))?.into());

    let mut resp = Response::new_with_opt_str_and_init(Some("Hello from Workers!"), &init)?;

    Ok(resp)
}
*/


/*
#[wasm_bindgen]
pub fn handle(kv: JsValue, req: JsValue) -> Result<Response, JsValue> {
    let req: Request = req.dyn_into()?;
    let mut init = ResponseInit::new();
    init.status(200);
    Response::new_with_opt_str_and_init(None, &init)
}
*/





#[wasm_bindgen]
extern "C" {
    pub type WorkersKvJs;

    #[wasm_bindgen(structural, method, catch)]
    pub async fn put(
        this: &WorkersKvJs,
        k: JsValue,
        v: JsValue,
        options: JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(structural, method, catch)]
    pub async fn get(
        this: &WorkersKvJs,
        k: JsValue,
        options: JsValue,
    ) -> Result<JsValue, JsValue>;
}





use js_sys::{ArrayBuffer, Object, Reflect, Uint8Array, Float64Array};

struct WorkersKv {
    kv: WorkersKvJs,
}

/*
impl WorkersKv {
    pub fn new(kv: WorkersKvJs) -> Self {
        Self { kv }
    }

    pub async fn put(&self, k: &str, v: &[u8]) -> Result<(), JsValue> {
        let k = JsValue::from_str(k);
        let v = Uint8Array::new_with_byte_offset_and_length(
            &ArrayBuffer::from(v),
            0,
            v.len() as u32,
        );
        let options = Object::new();
        Reflect::set(&options, &JsValue::from_str("expirationTtl"), &JsValue::from(0))?;
        Reflect::set(&options, &JsValue::from_str("expiration"), &JsValue::from(0))?;
        Reflect::set(&options, &JsValue::from_str("metadata"), &JsValue::from_str("{}"))?;
        self.kv.put(k, v, options).await?;
        Ok(())
    }

    pub async fn get(&self, k: &str) -> Result<Option<Vec<u8>>, JsValue> {
        let k = JsValue::from_str(k);
        let options = Object::new();
        let v = self.kv.get(k, options).await?;
        if v.is_undefined() {
            return Ok(None);
        }
        let v = v.dyn_into::<Uint8Array>()?;
        let mut buf = vec![0; v.length() as usize];
        v.copy_to(&mut buf);
        Ok(Some(buf))
    }
}
*/



impl WorkersKv {
    async fn put_text(&self, key: &str, value: &str, expiration_ttl: u64) -> Result<(), JsValue> {
        let options = Object::new();
        Reflect::set(&options, &"expirationTtl".into(), &(expiration_ttl as f64).into())?;
        self.kv
            .put(JsValue::from_str(key), value.into(), options.into())
            .await?;
        Ok(())
    }


    async fn put_vec(&self, key: &str, value: &[u8], expiration_ttl: u64) -> Result<(), JsValue> {
        let options = Object::new();
        Reflect::set(&options, &"expirationTtl".into(), &(expiration_ttl as f64).into())?;
        let typed_array = Uint8Array::new_with_length(value.len() as u32);
        typed_array.copy_from(value);
        self.kv
            .put(
                JsValue::from_str(key),
                typed_array.buffer().into(),
                options.into(),
            )
            .await?;
        Ok(())
    }


    async fn put_vec_f64(&self, key: &str, value: &[f64], expiration_ttl: u64) -> Result<(), JsValue> {
        let options = Object::new();
        Reflect::set(&options, &"expirationTtl".into(), &(expiration_ttl as f64).into())?;
        let typed_array = Float64Array::new_with_length(value.len() as u32);
        typed_array.copy_from(value);
        self.kv
            .put(
                JsValue::from_str(key),
                typed_array.buffer().into(),
                options.into(),
            )
            .await?;
        Ok(())
    }






    async fn get_text(&self, key: &str) -> Result<Option<String>, JsValue> {
        let options = Object::new();
        Reflect::set(&options, &"type".into(), &"text".into())?;
        Ok(self
            .kv
            .get(JsValue::from_str(key), options.into())
            .await?
            .as_string())
    }


    async fn get_vec(&self, key: &str) -> Result<Option<Vec<u8>>, JsValue> {
        let options = Object::new();
        Reflect::set(&options, &"type".into(), &"arrayBuffer".into())?;
        let value = self.kv.get(JsValue::from_str(key), options.into()).await?;
        if value.is_null() {
            Ok(None)
        } else {
            let buffer = ArrayBuffer::from(value);
            let typed_array = Uint8Array::new_with_byte_offset(&buffer, 0);
            let mut v = vec![0; typed_array.length() as usize];
            typed_array.copy_to(v.as_mut_slice());
            Ok(Some(v))
        }
    }


    async fn get_vec_f64(&self, key: &str) -> Result<Option<Vec<f64>>, JsValue> {
        let options = Object::new();
        Reflect::set(&options, &"type".into(), &"arrayBuffer".into())?;
        let value = self.kv.get(JsValue::from_str(key), options.into()).await?;
        if value.is_null() {
            Ok(None)
        } else {
            let buffer = ArrayBuffer::from(value);
            let typed_array = Float64Array::new_with_byte_offset(&buffer, 0);
            let mut v = vec![0.0; typed_array.length() as usize];
            typed_array.copy_to(v.as_mut_slice());
            Ok(Some(v))
        }
    }


}




#[wasm_bindgen]
pub async fn handle(kv: WorkersKvJs, req: JsValue) -> Result<Response, JsValue> {
    let req: Request = req.dyn_into()?;
    let url = web_sys::Url::new(&req.url())?;
    let pathname = url.pathname();
    let query_params = url.search_params();
    let kv = WorkersKv { kv };
    match req.method().as_str() {
        "GET" => {
            let value = kv.get_text(&pathname).await?.unwrap_or_default();
            let mut init = ResponseInit::new();
            init.status(200);
            Response::new_with_opt_str_and_init(Some(&format!("\"{}\"\n", value)), &init)
        },
        "PUT" => {
            let value = query_params.get("value").unwrap_or_default();
            // set a TTL of 60 seconds:
            kv.put_text(&pathname, &value, 60).await?;
            let mut init = ResponseInit::new();
            init.status(200);
            Response::new_with_opt_str_and_init(None, &init)
        },
        _ => {
            let mut init = ResponseInit::new();
            init.status(400);
            Response::new_with_opt_str_and_init(None, &init)
        }
    }
}