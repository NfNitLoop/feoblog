pub mod app;

use wasm_bindgen::prelude::wasm_bindgen;


#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate() {
    use leptos::*;
    use app::App;

    console_error_panic_hook::set_once();

    leptos::mount_to_body(App);
}
