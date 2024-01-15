use dioxus::prelude::*;
use tpaint::prelude::*;

use crate::app::SharedState;

pub fn SceneViewer(cx: Scope) -> Element {
    let shared_state = use_shared_state::<SharedState>(cx).unwrap();

    render! {
        view {
            class: "w-25% bg-zinc-900 rounded-5 h-full p-10 text-white overflow-y-scroll flex-col scrollbar-default gap-10 items-start",

            view {
                class: "bg-zinc-800 hover:bg-zinc-700 text-white rounded-5 p-8 px-8 text-14",
                "Spawn entity"
            }
        }
    }
}
