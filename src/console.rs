use dioxus::prelude::*;
use tpaint::prelude::*;

use crate::{app::SharedState, drawer::DrawerContext};

pub fn Console(cx: Scope) -> Element {
    let drawer_ctx = use_context::<DrawerContext>(cx).unwrap();
    let shared_state = use_shared_state::<SharedState>(cx).unwrap();

    // let shared_state = shared_state.read();
    let lines = use_state(cx, || String::new());

    use_future(cx, (shared_state,), |(shared_state,)| {
        to_owned![lines];
        async move {
            let Some(mut runtime_output) = shared_state.write_silent().runtime_output.take() else {
                return;
            };
            while let Some(line) = runtime_output.recv().await {
                // println!("yo! {}", line);
                let mut l = lines.make_mut();
                l.push_str(&line);
                l.push_str("\n");
                // lines.push(line);
            }
        }
    });

    // if let Some(mut runtime_output) = shared_state.write_silent().runtime_output.take() {
    //     cx.spawn({});
    //     //     use_coroutine(cx, |mut _rx: UnboundedReceiver<()>| {
    //     //         to_owned![lines];
    //     //         async move {
    //     //             while let Some(line) = runtime_output.recv().await {
    //     //                 let mut l = lines.make_mut();
    //     //                 l.push_str(&line);
    //     //                 l.push_str("\n");
    //     //                 // lines.push(line);
    //     //             }
    //     //         }
    //     //     });
    // }

    render! {
        view {
            class: "h-{drawer_ctx.height} w-full overflow-y-scroll scrollbar-default",

            view {
                class: "w-full flex-col",

                lines.lines().map(|line| rsx! {
                    view {
                        class: "w-full text-white text-12",
                        "{line}"
                    }
                })
            }

        }
    }
}
