use crate::RootContext;
use dioxus::prelude::*;
use futures_util::stream::StreamExt;
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use roth::{EditorToRuntimeMsg, RuntimeToEditorMsg};
use tpaint::prelude::*;

pub fn app(cx: Scope) -> Element {
    render! {
        view {
            class: "w-full bg-slate-900  text-white h-full",

            view {
                class: "w-25% bg-slate-900 h-full p-10 text-white",

                "Sidebar"
            }

            view {
                class: "w-75% flex-col h-full bg-black",

                view {
                    class: "w-full h-75% p-10 text-white bg-slate-800",

                    runtime(cx)
                }

                view {
                    class: "w-full h-25% p-10 text-white bg-slate-700",

                    "Console"
                }
            }


        }
    }
}

fn runtime(cx: Scope) -> Element {
    let window_id = cx
        .consume_context::<RootContext>()
        .unwrap()
        .window_id
        .to_string();

    let runtime_sender = use_coroutine(
        cx,
        |mut rx: UnboundedReceiver<EditorToRuntimeMsg>| async move {
            let (server, server_name) =
                IpcOneShotServer::<(IpcSender<EditorToRuntimeMsg>, String)>::new().unwrap();

            let mut runtime_process = tokio::process::Command::new("cargo")
                .arg("run")
                .arg("--manifest-path")
                .arg("./example_wgpu_app/Cargo.toml")
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

            loop {
                tokio::select! {
                    Some(msg) = rx.next() => {
                        runtime_sender.send(msg).unwrap();
                    }
                    _ = runtime_message_stream.next() => {
                        println!("chat_client: runtime_receiver.recv() => _");
                    }
                    _ = runtime_process.wait() => break,
                }
            }
        },
    );

    render! {
        view {
            class: "w-full h-full bg-black p-0 hover:p-1",
            onclick: move |_| {
                runtime_sender.send(EditorToRuntimeMsg::Todo);
            },
            onlayout: move |event| {
                runtime_sender.send(EditorToRuntimeMsg::LayoutChange(event.rect));
            },
        }
    }
}
