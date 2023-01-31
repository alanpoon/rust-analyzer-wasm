use indexed_db_futures::prelude::*;
use web_sys::DomException;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use serde_json::{json,Value};
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

#[wasm_bindgen]
pub async fn example() -> Result<js_sys::Array, DomException> {
    // Open my_db v1
    log("example");
    let mut db_req: OpenDbRequest = IdbDatabase::open_u32("wishes", 1)?;
    db_req.set_on_upgrade_needed(Some(|evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
        // Check if the object store exists; create it if it doesn't
        if let None = evt.db().object_store_names().find(|n| n == "mywishes") {
            evt.db().create_object_store("mywishes")?;
        }
        Ok(())
    }));

    let db: IdbDatabase = db_req.into_future().await?;

    // Insert/overwrite a record
    let tx: IdbTransaction = db
      .transaction_on_one_with_mode("mywishes", IdbTransactionMode::Readwrite)?;
    let store: IdbObjectStore = tx.object_store("mywishes")?;
    // let obj = js_sys::Object::new();
    // js_sys::Reflect::set(&obj, &"file_name".into(), &"bar".into());
    // js_sys::Reflect::set(&obj, &"text".into(), &"bar".into());
    // store.put_val(&obj)?;

    // // IDBTransactions can have an Error or an Abort event; into_result() turns both into a
    // // DOMException
     tx.await.into_result()?;

    // // Delete a record
    // let tx = db.transaction_on_one_with_mode("mywishes", IdbTransactionMode::Readwrite)?;
    // let store = tx.object_store("mywishes")?;
    // store.delete_owned("my_key")?;
    // tx.await.into_result()?;

    // // Get a record
    let tx = db.transaction_on_one("mywishes")?;
    let store = tx.object_store("mywishes")?;
    let arr = store.get_all()?.await?;
    //let obj = js_sys::Object::new();
    for v in arr.iter(){
        log(&format!("found in crates {:?}",v));
    }
    //let value: Option<JsValue> = store.get_owned("my_key")?.await?;
    
    //use_value(value);
    
    // All of the requests in the transaction have already finished so we can just drop it to
    // avoid the unused future warning, or assign it to _.
    //let _ = tx;

    Ok(arr)
}