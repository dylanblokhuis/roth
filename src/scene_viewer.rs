use bevy::ecs::entity::Entity;
use dioxus::prelude::*;
use roth_shared::{
    ron::Value, EditorToRuntimeMsg, RonComponent, RonComponentSerialized, RuntimeToEditorMsg,
};
use tpaint::{components::image::Image, prelude::*};

use crate::app::SharedState;

// in the future this should be editable in a config file
const KNOWN_COMPONENT_NAMES: [&str; 4] = ["PointLight", "DirectionalLight", "Camera", "Mesh"];

fn get_entity_name(entity: &Entity, components: &[RonComponent]) -> String {
    // first we check if any `Name` components exist
    if let Some(component) = components
        .iter()
        .find(|it| it.name() == "bevy_core::name::Name")
    {
        let Value::Map(map) = component.components() else {
            panic!("expected component value to always be a map");
        };
        let key = Value::String("name".to_string());
        if let Value::String(name) = &map[&key] {
            return format!("{} ({:?})", name, entity);
        } else {
            panic!("expected name to be a string");
        }
    }

    // if not we check there are any main
    for known_component_name in KNOWN_COMPONENT_NAMES.iter() {
        if components
            .iter()
            .any(|it| it.name().contains(known_component_name))
        {
            return format!("{} ({:?})", known_component_name, entity);
        }
    }

    format!("{:?}", entity)
}

pub fn SceneViewer(cx: Scope) -> Element {
    let shared_state = use_shared_state::<SharedState>(cx).unwrap();
    let entities_state =
        use_state::<Vec<(Entity, Vec<RonComponentSerialized>)>>(cx, move || vec![]);

    use_effect(cx, shared_state, |shared_state| async move {
        shared_state
            .read()
            .send_to_runtime(EditorToRuntimeMsg::GetEntities);
    });

    use_effect(cx, (), move |()| {
        to_owned![shared_state, entities_state];
        async move {
            let mut rx = shared_state.read().runtime_response.subscribe();
            while let Ok(RuntimeToEditorMsg::Entities { entities }) = rx.recv().await {
                entities_state.set(entities);
            }
        }
    });

    render! {
        view {
            class: "w-20% bg-zinc-900 rounded-5 h-full  text-white overflow-y-scroll flex-col justify-start scrollbar-default gap-10 items-start",

            entities_state.iter().map(|(entity, components)| rsx! {
                Entity {
                    key: "{entity.index()}v{entity.generation()}",
                    entity: entity,
                    components: components.iter().map(|it| RonComponent::from(it)).collect(),
                }
            })
        }
    }
}

#[component]
fn Entity<'a>(cx: Scope, entity: &'a Entity, components: Vec<RonComponent>) -> Element {
    // let is_open = use_state(cx, || false);
    let shared_state = use_shared_state::<SharedState>(cx).unwrap();

    render! {
    // view {
        // class: "text-white flex-col w-full",


        view {
            class: "flex-row p-8 justify-between items-center w-full text-14 text-white active:bg-zinc-800",
            is_active: "{shared_state.read().selected_entity == Some(**entity)}",
            onclick: move |_event: Event<_>| {
                shared_state.write().selected_entity = Some(**entity);
            },
            "{get_entity_name(entity, components)}",

            Image {
                class: "w-20 h-20",
                src: "./assets/more-vertical.svg".to_string(),
            }
        }

        // if *is_open.get() {
        //     rsx! {
        //         components.iter().map(|component| rsx! {
        //             Component {
        //                 key: "{component.name()}",
        //                 component: component,
        //             }
        //         })
        //     }

        // }

    }
    // }
}

#[component]
fn Component<'a>(cx: Scope, component: &'a RonComponent) -> Element {
    render! {
        view {
            class: "text-white flex-col text-14 p-4",
            "{component.short_name()}",
        }
    }
}
