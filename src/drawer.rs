use dioxus::prelude::*;
use tpaint::{components::image::Image, events::ClickEvent, prelude::*};

use crate::{asset_browser::AssetBrowser, console::Console};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    AssetBrowser,
    Console,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawerContext {
    pub height: f32,
}

pub fn Drawer(cx: Scope) -> Element {
    let tab_state = use_state::<Tab>(cx, || Tab::AssetBrowser);

    use_context_provider(cx, || DrawerContext { height: 200.0 });

    render! {
        view {
            class: "flex-col",

            // tabs
            view {
                class: "gap-x-8 px-5",

                for tab in [Tab::AssetBrowser, Tab::Console] {
                    view {
                        class: "bg-zinc-800 hover:bg-zinc-900 active:bg-zinc-900 text-white p-9 px-8 text-14 rounded-t-5",
                        is_active: tab_state.get() == &tab,
                        onclick: move |_| {
                            tab_state.set(tab);
                        },

                        match tab {
                            Tab::AssetBrowser => rsx!(" Assets "),
                            Tab::Console => rsx!(" Console "),
                        }
                    }
                }
            }

            // content
            view {
                class: "bg-zinc-900 p-16 text-white rounded-5",

                if tab_state.get() == &Tab::AssetBrowser {
                    rsx! { AssetBrowser {} }
                } else {
                    rsx! { Console {} }
                }
            }
        }
    }
}
