pub mod app;

mod asciifier;
mod ascii_image;
mod basic;
mod font_handler;
mod output;

use cfg_if::cfg_if;

cfg_if! {
if #[cfg(feature = "csr")] {

    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    pub fn hydrate() {
        use app::*;
        use leptos::*;

        console_error_panic_hook::set_once();

        leptos::mount_to_body(move |cx| {
            view! { cx, <App/> }
        });
    }
}
}
