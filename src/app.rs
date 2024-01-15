use std::process::Stdio;

use crate::{drawer::Drawer, scene_viewer::SceneViewer, RootContext};
use dioxus::prelude::*;
use futures_util::stream::StreamExt;
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use roth_shared::{EditorToRuntimeMsg, RuntimeToEditorMsg};
use tokio::io::{AsyncBufReadExt, BufReader};
use tpaint::{components::image::Image, prelude::*};

#[derive(PartialEq, Clone, Copy)]
pub enum RuntimeStatus {
    Stopped,
    Running,
}

pub struct SharedState {
    pub project_path: String,
    pub runtime_status: RuntimeStatus,
    runtime_sender: Option<tokio::sync::mpsc::UnboundedSender<EditorToRuntimeMsg>>,
    runtime_receiver: Option<tokio::sync::mpsc::UnboundedReceiver<EditorToRuntimeMsg>>,
    pub runtime_output: String,
}

impl SharedState {
    pub fn start_runtime(&mut self) {
        let (runtime_sender, runtime_receiver) =
            tokio::sync::mpsc::unbounded_channel::<EditorToRuntimeMsg>();
        self.runtime_sender = Some(runtime_sender);
        self.runtime_receiver = Some(runtime_receiver);
        self.runtime_status = RuntimeStatus::Running;
    }

    pub fn stop_runtime(&mut self) {
        let sender = self.runtime_sender.take();
        if let Some(sender) = sender {
            sender.send(EditorToRuntimeMsg::Shutdown).unwrap();
        }
        self.runtime_status = RuntimeStatus::Stopped;
    }

    pub fn send_to_runtime(&self, msg: EditorToRuntimeMsg) {
        if let Some(runtime_sender) = &self.runtime_sender {
            runtime_sender.send(msg).unwrap();
        }
    }
}

pub fn app(cx: Scope) -> Element {
    use_shared_state_provider(cx, || SharedState {
        project_path: "/home/dylan/dev/roth/example_bevy".to_string(),
        runtime_status: RuntimeStatus::Stopped,
        runtime_sender: None,
        runtime_receiver: None,
        runtime_output: String::new(),
    });
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
                    class: "gap-x-16",

                    view {
                        class: "text-white text-18",
                        tabindex: 0,
                        onclick: move |_| {
                            shared_state.read().send_to_runtime(EditorToRuntimeMsg::Save);
                        },

                        "Save"
                    }

                    view {
                        class: "text-white text-18",
                        tabindex: 0,
                        onclick: move |_| {
                            if runtime_status == RuntimeStatus::Stopped {
                                shared_state.write().start_runtime();
                            } else {
                                shared_state.write().stop_runtime();
                            }
                        },

                        if runtime_status == RuntimeStatus::Stopped { rsx! { "Play" } } else { rsx! { "Stop" } }
                    }
                }


            }

            view {
                class: "w-full h-full gap-x-8",

                // sidebar
                SceneViewer {}

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

            Drawer {}
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

    let runtime_sender = use_coroutine(cx, |mut rx: UnboundedReceiver<EditorToRuntimeMsg>| {
        to_owned![shared_state];
        async move {
            let (server, server_name) =
                IpcOneShotServer::<(IpcSender<EditorToRuntimeMsg>, String)>::new().unwrap();

            let project_path = shared_state.read().project_path.clone();
            let mut runtime_process = tokio::process::Command::new("cargo")
                .arg("run")
                .arg("--manifest-path")
                .arg(format!("{}/Cargo.toml", project_path))
                .arg("--")
                .arg("--window-id")
                .arg(window_id)
                .arg("--ipc-server")
                .arg(server_name)
                .kill_on_drop(true)
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("failed to start runtime process");

            let stdout = runtime_process
                .stdout
                .take()
                .expect("runtime_process did not have a handle to stdout");
            let stderr = runtime_process
                .stderr
                .take()
                .expect("runtime_process did not have a handle to stderr");
            let mut stdout_reader = BufReader::new(stdout).lines();
            let mut stderr_reader = BufReader::new(stderr).lines();

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

            let mut app_runtime_receiver = {
                let mut shared_state = shared_state.write();
                shared_state
                    .runtime_receiver
                    .take()
                    .expect("Runtime component was created without any runtime receiver")
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

                    Some(msg) = app_runtime_receiver.recv() => {
                        runtime_sender.send(msg).unwrap();
                    }

                    // handle the process things here
                    Ok(line) = stdout_reader.next_line() => {
                        let mut shared_state = shared_state.write();
                        shared_state.runtime_output.push_str(&line.unwrap());
                        shared_state.runtime_output.push_str(&"\n");
                    }

                    Ok(line) = stderr_reader.next_line() => {
                        let mut shared_state = shared_state.write();
                        shared_state.runtime_output.push_str(&line.unwrap());
                        shared_state.runtime_output.push_str(&"\n");
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
