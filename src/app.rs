use crate::RootContext;
use bevy::ecs::entity::Entity;
use dioxus::prelude::*;
use futures_util::stream::StreamExt;
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use roth_shared::{EditorToRuntimeMsg, RuntimeToEditorMsg};
use tpaint::prelude::*;

pub fn app(cx: Scope) -> Element {
    let entities = use_state::<Vec<Entity>>(cx, || vec![]);

    render! {
        view {
            class: "w-full   text-white h-full",

            view {
                class: "w-25% bg-slate-900 h-full p-10 text-white overflow-y-scroll flex-col scrollbar-default gap-10",
                tabindex: 1,
                oninput: move |event| {
                    println!("sidebar oninput: {:?}", event);
                },
                onclick: move |_| {
                    println!("sidebar onclick");
                },

                "Entities",

                for entity in entities.iter() {
                    rsx! {
                        view {
                            class: "text-white",
                            "{entity.index()}"
                        }
                    }
                }
            }

            view {
                class: "w-75% flex-col h-full",

                view {
                    class: "w-full h-75% text-white",

                    RuntimeWindow {
                        entity_state: &entities,
                    }
                }

                view {
                    class: "w-full h-25% p-10 text-white bg-slate-700",

                    "Console"
                }
            }


        }
    }
}

#[derive(Props)]
struct RuntimeProps<'a> {
    entity_state: &'a UseState<Vec<Entity>>,
}

fn RuntimeWindow<'a>(cx: Scope<'a, RuntimeProps<'a>>) -> Element {
    let window_id = cx
        .consume_context::<RootContext>()
        .unwrap()
        .window_id
        .to_string();

    let entity_state = cx.props.entity_state;

    let runtime_sender = use_coroutine(cx, |mut rx: UnboundedReceiver<EditorToRuntimeMsg>| {
        to_owned![entity_state];
        async move {
            let (server, server_name) =
                IpcOneShotServer::<(IpcSender<EditorToRuntimeMsg>, String)>::new().unwrap();

            let mut runtime_process = tokio::process::Command::new("cargo")
                .arg("run")
                .arg("--manifest-path")
                .arg("./example_bevy/Cargo.toml")
                .arg("--")
                .arg("--window-id")
                .arg(window_id)
                .arg("--ipc-server")
                .arg(server_name)
                .spawn()
                .expect("failed to start runtime process");

            let (_, (runtime_sender, oneshot_server_name)) = server.accept().unwrap();

            let (tx1, runtime_receiver): (
                IpcSender<RuntimeToEditorMsg>,
                IpcReceiver<RuntimeToEditorMsg>,
            ) = ipc_channel::ipc::channel().unwrap();

            let tx: IpcSender<IpcSender<RuntimeToEditorMsg>> =
                IpcSender::connect(oneshot_server_name).unwrap();
            tx.send(tx1).unwrap();

            let mut runtime_message_stream = runtime_receiver.to_stream();

            let handle_runtime_message = |msg: RuntimeToEditorMsg| match msg {
                RuntimeToEditorMsg::Entities { entities } => entity_state.set(entities),
                _ => {}
            };

            loop {
                tokio::select! {
                    Some(msg) = rx.next() => {
                        runtime_sender.send(msg).unwrap();
                    }
                    Some(msg) = runtime_message_stream.next() => {
                        let msg = msg.unwrap();
                        handle_runtime_message(msg);
                    }
                    _ = runtime_process.wait() => break,
                }
            }
        }
    });

    render! {
        view {
            class: "w-full h-full bg-transparent",
            tabindex: 0,
            onclick: move |_| {
                runtime_sender.send(EditorToRuntimeMsg::Todo);
            },
            onlayout: move |event| {
                runtime_sender.send(EditorToRuntimeMsg::LayoutChange {
                    width: event.rect.width(),
                    height: event.rect.height(),
                    min: (event.rect.min.x, event.rect.min.y),
                    window_position: event.state.window_position,
                });
            },
            onmousemove: move |event| {
                let pos = event.state.cursor_state.current_position;
                runtime_sender.send(EditorToRuntimeMsg::CursorMoved { position: winit::dpi::PhysicalPosition::new(pos.x as f64, pos.y as f64) });
            },

            onmousedown: move |event| {
                runtime_sender.send(EditorToRuntimeMsg::MouseInput {
                    button: event.button,
                    state: event.element_state
                });
            },

            onmouseup: move |event| {
                runtime_sender.send(EditorToRuntimeMsg::MouseInput {
                    button: event.button,
                    state: event.element_state
                });
            },
            oninput: move |event| {
                runtime_sender.send(EditorToRuntimeMsg::ReceivedCharacter {
                    char: event.text.clone()
                });
            },
            onkeydown: move |event| {
                runtime_sender.send(EditorToRuntimeMsg::KeyboardInput {
                    state: event.element_state,
                    physical_key: event.physical_key
                });
            },
            onkeyup: move |event| {
                runtime_sender.send(EditorToRuntimeMsg::KeyboardInput {
                    state: event.element_state,
                    physical_key: event.physical_key
                });
            },
        }
    }
}
