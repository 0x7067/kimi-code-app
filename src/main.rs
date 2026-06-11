mod actions;
mod components;
mod design_tokens;
mod ipc;
mod markdown;
mod state;

use dioxus::prelude::*;

static CSS: Asset = asset!("/assets/main.css");

fn main() {
    launch(|| {
        rsx! {
            Stylesheet { href: CSS }
            components::App {}
        }
    });
}
