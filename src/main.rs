mod app;
mod state;
mod tauri;

use dioxus::prelude::*;

static CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(|| {
        rsx! {
            document::Stylesheet { href: CSS }
            app::App {}
        }
    });
}
