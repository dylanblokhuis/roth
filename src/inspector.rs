use bevy::ecs::entity::Entity;
use dioxus::prelude::*;
use roth_shared::{
    ron::{value::Float, Number, Value},
    RonComponent, RonComponentSerialized, RuntimeToEditorMsg,
};
use tpaint::{components::image::Image, prelude::*};

use crate::app::SharedState;

#[component]
pub fn Inspector(cx: Scope) -> Element {
    let shared_state = use_shared_state::<SharedState>(cx).unwrap();
    let components_state = use_state::<Vec<RonComponent>>(cx, move || vec![]);
    let selected_component_state = use_state::<Option<(Entity, RonComponent)>>(cx, move || None);

    use_effect(cx, (), move |()| {
        to_owned![shared_state, components_state, selected_component_state];
        async move {
            let mut rx = shared_state.read().runtime_response.subscribe();
            while let Ok(RuntimeToEditorMsg::Entities { entities }) = rx.recv().await {
                let Some(selected_entity) = shared_state.read().selected_entity else {
                    components_state.set(vec![]);
                    continue;
                };

                // reset selected component if it doesn't exist anymore
                if let Some((entity, _)) = selected_component_state.get() {
                    if !entities.iter().any(|(it, _)| *it == *entity) {
                        selected_component_state.set(None);
                    }
                }

                let selected_component = entities
                    .iter()
                    .find(|(entity, _)| *entity == selected_entity);

                if let Some((_, components)) = selected_component {
                    components_state
                        .set(components.iter().map(|it| RonComponent::from(it)).collect());
                } else {
                    components_state.set(vec![]);
                }
            }
        }
    });
    let selected_entity = shared_state.read().selected_entity;

    render! {
        view {
            class: "w-20% h-full bg-zinc-900  text-white rounded-5 flex-col",

            for (i, component) in components_state.get().iter().enumerate() {
                view {
                    key: "{i}-{component.short_name()}",
                    class: "text-white flex-col  w-full border-1 border-zinc-800 px-10 py-8",

                    view {
                        class: "text-white w-full items-center text-14",
                        onclick: move |_event: Event<_>| {
                            selected_component_state.set(Some((
                                selected_entity.unwrap(),
                                component.clone()
                            )));
                        },

                        Image {
                            class: "w-16 h-16 mr-10",
                            src: "./assets/toy-brick.svg".into(),
                        }

                        "{component.short_name()}",

                        if component.value == Value::Unit {
                            rsx! {
                                view {
                                    class: "text-14 text-zinc-700 ml-10",
                                    "Computed"
                                }
                            }
                        }
                    }

                    if selected_component_state.get().as_ref().map(|(entity, selected_component)| selected_entity.unwrap() == *entity && *selected_component.type_name == *component.type_name).unwrap_or(false) {
                        rsx!{
                            ComponentProperties {
                                entity: selected_entity.unwrap(),
                                ron_component: component.clone(),
                            }
                        }
                    }
                }
            }

        }
    }
}

#[component]
fn ComponentProperties(cx: Scope, entity: Entity, ron_component: RonComponent) -> Element {
    let ron_component = cx.use_hook(|| ron_component.clone());
    let shared_state = use_shared_state::<SharedState>(cx).unwrap();

    let type_name = ron_component.type_name.clone();
    let components_mut = ron_component.components_mut();
    let init_ptr = components_mut as *mut Value;

    fn recurse_value<'a>(
        value: &'a mut Value,
        state: UseSharedState<SharedState>,
        init_ptr: *mut Value,
        entity: Entity,
        type_name: String,
        mut parent_keys: Vec<String>,
    ) -> Vec<LazyNodes<'a, 'a>> {
        let ptr = value as *mut Value;

        match value {
            Value::Map(map) => map
                .iter_mut()
                .map(|(key, value)| {
                    let Value::String(key) = key else {
                        unreachable!("Key should always be a string")
                    };

                    let state = state.clone();
                    let type_name = type_name.clone();
                    let key: String = key.clone();
                    parent_keys.push(key.clone());
                    let parent_keys = parent_keys.clone();

                    rsx! {
                        view {
                            class: "w-full flex-row justify-between items-start text-white",
                            key: "{entity:?}-{type_name}-{parent_keys:?}-{key}",
                            "{key}: ",
                            view {
                                class: "flex-col",
                                recurse_value(value, state, init_ptr, entity, type_name, parent_keys).into_iter()
                            }
                        }
                    }
                })
                .collect(),
            Value::Number(number) => {
                let node = match number {
                    Number::Float(val) => {
                        rsx! {
                            view {
                                class: "text-white",
                                key: "{entity:?}-{type_name}-{parent_keys:?}-{val.get():.2}",
                                onclick: move |_event: Event<_>| {
                                    let new_value = val.get() + 1.0;
                                    unsafe {
                                        (*ptr) = Value::Number(Number::Float(Float::new(new_value)));
                                    }

                                    state.read().send_to_runtime(roth_shared::EditorToRuntimeMsg::InsertComponent {
                                        entity,
                                        component: RonComponentSerialized {
                                            type_name: type_name.clone(),
                                            value: roth_shared::serde_json::to_string(unsafe { &*init_ptr }).unwrap()
                                        }
                                    });
                                },
                                "{val.get():.2}",
                            }
                        }
                    }
                    Number::Integer(val) => {
                        rsx! {
                            view {
                                class: "text-white",
                                key: "{entity:?}-{type_name}-{parent_keys:?}-{val}",
                                "{val}",
                            }
                        }
                    }
                };

                vec![node]
            }
            unimplemented => {
                println!("unimplemented: {:?}", unimplemented);
                vec![]
            }
        }
    }

    render! {
        view {
            class: "w-full flex-col gap-10 mt-10",
            recurse_value(components_mut, shared_state.to_owned(), init_ptr, *entity, type_name.clone(), vec![type_name.clone()]).into_iter()
        }
    }
}
