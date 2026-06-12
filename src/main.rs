mod actions;
mod components;
mod conversation;
mod design_tokens;
mod ipc;
mod markdown;
mod verify;
mod state;

use dioxus::prelude::*;

static CSS_TOKENS: Asset = asset!("/assets/css/01-tokens.css");
static CSS_BASE: Asset = asset!("/assets/css/02-base.css");
static CSS_LAYOUT: Asset = asset!("/assets/css/03-layout.css");
static CSS_SIDEBAR: Asset = asset!("/assets/css/04-sidebar.css");
static CSS_THREAD: Asset = asset!("/assets/css/05-thread.css");
static CSS_COMPOSER: Asset = asset!("/assets/css/06-composer.css");
static CSS_PANELS: Asset = asset!("/assets/css/07-panels.css");
static CSS_MODALS: Asset = asset!("/assets/css/08-modals.css");
static CSS_SETTINGS: Asset = asset!("/assets/css/09-settings.css");
static CSS_COMPONENTS: Asset = asset!("/assets/css/10-components.css");
static CSS_ANIMATIONS: Asset = asset!("/assets/css/11-animations.css");
static CSS_RESPONSIVE: Asset = asset!("/assets/css/12-responsive.css");

fn main() {
    launch(|| {
        rsx! {
            Stylesheet { href: CSS_TOKENS }
            Stylesheet { href: CSS_BASE }
            Stylesheet { href: CSS_LAYOUT }
            Stylesheet { href: CSS_SIDEBAR }
            Stylesheet { href: CSS_THREAD }
            Stylesheet { href: CSS_COMPOSER }
            Stylesheet { href: CSS_PANELS }
            Stylesheet { href: CSS_MODALS }
            Stylesheet { href: CSS_SETTINGS }
            Stylesheet { href: CSS_COMPONENTS }
            Stylesheet { href: CSS_ANIMATIONS }
            Stylesheet { href: CSS_RESPONSIVE }
            components::App {}
        }
    });
}
