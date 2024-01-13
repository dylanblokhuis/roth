use crate::RootContext;
use dioxus::prelude::*;
use futures_util::stream::StreamExt;
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use roth_shared::{EditorToRuntimeMsg, RuntimeToEditorMsg};
use tpaint::{components::image::Image, prelude::*};

#[derive(PartialEq, Clone, Copy)]
enum RuntimeStatus {
    Stopped,
    Stopping,
    Running,
}

struct SharedState {
    runtime_status: RuntimeStatus,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            runtime_status: RuntimeStatus::Stopped,
        }
    }
}

pub fn app(cx: Scope) -> Element {
    // let entities = use_state::<Vec<Entity>>(cx, || vec![]);
    use_shared_state_provider(cx, || SharedState::default());
    let shared_state = use_shared_state::<SharedState>(cx).unwrap();

    let runtime_status = shared_state.read().runtime_status.clone();

    render! {
        view {
            class: "w-full h-full p-5 bg-zinc-700 flex-col gap-y-8",

            view {
                class: "rounded-5 w-full py-5 px-15 bg-zinc-800 text-white justify-between items-center",


                view {
                    class: "text-white gap-15 items-center",

                    Image {
                        class: "h-40",
                        src: "https://raw.githubusercontent.com/bevyengine/bevy/78b5f323f87500bfd20eb5eb45a599d06324ba7b/assets/branding/bevy_bird_dark.svg".to_string(),
                    }

                    "File",
                    "Edit",
                    "View",
                    "Window",
                }

                view {
                    class: "text-white text-18",
                    tabindex: 0,
                    onclick: move |_| {
                        println!("start runtime");
                        shared_state.with_mut(|state| {
                            if state.runtime_status == RuntimeStatus::Stopped {
                                state.runtime_status = RuntimeStatus::Running;
                            } else if state.runtime_status == RuntimeStatus::Running {
                                state.runtime_status = RuntimeStatus::Stopping;
                            }
                        });
                    },

                    if runtime_status == RuntimeStatus::Stopped { rsx! { "Play" } } else { rsx! { "Stop" } }
                }
            }

            view {
                class: "w-full h-full gap-x-8",

                // sidebar
                view {
                    class: "w-25% bg-zinc-800 rounded-5 h-full p-10 text-white overflow-y-scroll flex-col scrollbar-default gap-10",
                }

                // viewport
                view {
                    class: "w-75% flex-col h-full bg-transparent rounded-5",

                    if runtime_status != RuntimeStatus::Stopped {
                        rsx! {
                            RuntimeWindow {}
                        }
                    } else {
                        rsx! {
                            view {
                                class: "w-full h-full justify-center items-center bg-zinc-900",
                            }
                        }
                    }
                }
            }

            view {
                class: "w-full h-300 bg-zinc-800 text-white rounded-5 p-10",

                "Assets"
            }
        }
    }
}

fn RuntimeWindow<'a>(cx: Scope<'a>) -> Element {
    let window_id = cx
        .consume_context::<RootContext>()
        .unwrap()
        .window_id
        .to_string();
    let shared_state = use_shared_state::<SharedState>(cx).unwrap();

    // let entity_state = cx.props.entity_state;

    let runtime_sender = use_coroutine(cx, |mut rx: UnboundedReceiver<EditorToRuntimeMsg>| {
        to_owned![shared_state];
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
                // RuntimeToEditorMsg::Entities { entities } => entity_state.set(entities),
                _ => {}
            };

            let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));

            loop {
                tokio::select! {
                    Some(msg) = rx.next() => {
                        runtime_sender.send(msg).unwrap();
                    }
                    Some(msg) = runtime_message_stream.next() => {
                        let msg = msg.unwrap();
                        handle_runtime_message(msg);
                    }

                    _ = interval.tick() => {
                        let mut shared_state = shared_state.write();
                        if shared_state.runtime_status == RuntimeStatus::Stopping {
                            runtime_sender.send(EditorToRuntimeMsg::Shutdown).unwrap();
                            shared_state.runtime_status = RuntimeStatus::Stopped;
                        }
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
                runtime_sender.send(EditorToRuntimeMsg::Shutdown);
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
