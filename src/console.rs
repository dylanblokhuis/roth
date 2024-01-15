use dioxus::prelude::*;
use tpaint::prelude::*;

use crate::{app::SharedState, drawer::DrawerContext};

pub fn Console(cx: Scope) -> Element {
    let drawer_ctx = use_context::<DrawerContext>(cx).unwrap();
    let shared_state = use_shared_state::<SharedState>(cx).unwrap();

    let shared_state = shared_state.read();
    let lines = shared_state.runtime_output.lines();

    render! {
        view {
            class: "h-{drawer_ctx.height} w-full overflow-y-scroll scrollbar-default",

            view {
                class: "w-full flex-col",

                lines.map(|line| rsx! {
                    view {
                        class: "w-full text-white text-12",
                        "{line}"
                    }
                })
            }

        }
    }
}
