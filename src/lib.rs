#![cfg(feature = "hydrate")]
 

mod leptos_app;
use leptos_app::*;
use wasm_bindgen::prelude::wasm_bindgen;



#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate() {
    use leptos::*;

    console_error_panic_hook::set_once();

    leptos::mount_to_body(App);
}
