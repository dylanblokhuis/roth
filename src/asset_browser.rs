use dioxus::prelude::*;
use tpaint::{components::image::Image, events::ClickEvent, prelude::*};

use crate::{app::SharedState, drawer::DrawerContext};

#[derive(Debug, PartialEq)]
enum AssetType {
    Folder,
    Unknown,
}

#[derive(Debug, PartialEq)]
struct Asset {
    name: String,
    path: String,
    asset_type: AssetType,
}

pub fn AssetBrowser(cx: Scope) -> Element {
    let drawer_ctx = use_context::<DrawerContext>(cx).unwrap();
    let shared_state = use_shared_state::<SharedState>(cx).unwrap();
    let current_path_state = use_state::<String>(cx, || {
        format!("{}/assets", shared_state.read().project_path.clone())
    });

    let maybe_assets = use_future(cx, (current_path_state,), |(current_path_state,)| {
        // to_owned![];
        async move {
            let mut assets = Vec::new();
            let mut entries = tokio::fs::read_dir(current_path_state.get()).await.unwrap();
            while let Some(entry) = entries.next_entry().await.unwrap() {
                let path = entry.path();
                let file_name = path.file_name().unwrap().to_str().unwrap();
                if file_name.starts_with(".") {
                    continue;
                }
                let asset_type = if path.is_dir() {
                    AssetType::Folder
                } else {
                    AssetType::Unknown
                };
                assets.push(Asset {
                    name: file_name.to_string(),
                    path: path.to_str().unwrap().to_string(),
                    asset_type,
                });
            }
            assets.sort_by(|a, b| a.name.cmp(&b.name));
            assets
        }
    })
    .value();

    render! {
        view {
            class: "w-full h-{drawer_ctx.height} flex-col overflow-y-scroll scrollbar-default",
            if let Some(assets) = maybe_assets {
                rsx! {
                    view {
                        class: "w-full gap-x-8 mt-6",
                        for asset in assets {
                            AssetItem {
                                key: "{asset.path}",
                                asset: &asset,
                                onclick: move |_event: Event<ClickEvent>| {
                                    if asset.asset_type == AssetType::Folder {
                                        current_path_state.set(asset.path.clone());
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

// asset component

#[component]
fn AssetItem<'a>(
    cx: Scope,
    asset: &'a Asset,
    // current_path_state: &'a UseState<Option<String>>,
    onclick: EventHandler<'a, Event<ClickEvent>>,
) -> Element {
    let src = match asset.asset_type {
        AssetType::Folder => "./assets/folder.svg".to_string(),
        AssetType::Unknown => "./assets/file-unknown.svg".to_string(),
    };

    render! {
        view {
            class: "w-84 h-84 p-8 hover:bg-zinc-700 flex-col justify-center items-center text-white rounded-5",
            onclick: move |evt| onclick.call(evt),

            Image {
                src: src,
                class: "mb-4",
            }

            view {
                class: "text-white text-12 mt-4 justify-center",
                "{asset.name}"
            }
        }
    }
}
