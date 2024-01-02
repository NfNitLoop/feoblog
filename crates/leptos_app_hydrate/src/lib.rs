
use wasm_bindgen::prelude::wasm_bindgen;


#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();

    leptos::mount_to_body(leptos_app::App);
}
