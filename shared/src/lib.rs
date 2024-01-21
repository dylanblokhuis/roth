use ::ron::Value;
use bevy::ecs::entity::Entity;
use serde::{Deserialize, Serialize};
use winit::{dpi::PhysicalPosition, keyboard::SmolStr};

#[derive(Debug, Serialize, Deserialize)]
pub enum EditorToRuntimeMsg {
    Shutdown,
    Save,
    LoadScene {
        path: String,
    },
    LayoutChange {
        min: (f32, f32),
        width: f32,
        height: f32,
        window_position: PhysicalPosition<i32>,
    },

    CursorMoved {
        position: PhysicalPosition<f64>,
    },
    CursorEntered,
    CursorLeft,
    MouseInput {
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    },
    MouseWheel {
        delta: winit::event::MouseScrollDelta,
    },
    KeyboardInput {
        state: winit::event::ElementState,
        physical_key: winit::keyboard::PhysicalKey,
    },
    ReceivedCharacter {
        char: SmolStr,
    },
    GetEntities,
    InsertComponent {
        entity: Entity,
        component: RonComponentSerialized,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeToEditorMsg {
    Entities {
        entities: Vec<(Entity, Vec<RonComponentSerialized>)>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RonComponentSerialized {
    /// we store this, so we can show the name of the types that aren't serializable
    pub type_name: String,
    /// actual RON value, will again contain the type name as a key
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RonComponent {
    /// we store this, so we can show the name of the types that aren't serializable
    pub type_name: String,
    /// actual RON value, will again contain the type name as a key
    pub value: Value,
}

impl From<RonComponentSerialized> for RonComponent {
    fn from(serialized: RonComponentSerialized) -> Self {
        RonComponent {
            type_name: serialized.type_name,
            value: ron::from_str(&serialized.value).unwrap(),
        }
    }
}

impl From<&RonComponentSerialized> for RonComponent {
    fn from(component: &RonComponentSerialized) -> Self {
        RonComponent {
            type_name: component.type_name.clone(),
            value: ron::from_str(&component.value).unwrap(),
        }
    }
}

impl RonComponent {
    pub fn name(&self) -> String {
        match &self.value {
            Value::Map(map) => {
                let mut name = String::new();
                map.keys().for_each(|it| {
                    if let Value::String(s) = it {
                        name = s.clone();
                    }
                });
                name
            }
            _ => self.type_name.clone(),
        }
    }

    pub fn short_name(&self) -> String {
        bevy::utils::get_short_name(&self.name())
    }

    /// will return the map that contains the values. For a ``Name`` component, it will return:
    /// ```ron
    /// Map({String("hash"): Number(Float(Float(1.2169318712647776e19))), String("name"): String("Cube!")})
    /// ```
    pub fn components(&self) -> &Value {
        let Value::Map(map) = &self.value else {
            panic!("expected component value to always be a map");
        };

        map.values().next().unwrap()
    }

    pub fn components_mut(&mut self) -> &mut Value {
        let Value::Map(map) = &mut self.value else {
            panic!("expected component value to always be a map");
        };

        map.values_mut().next().unwrap()
    }
}

pub mod ron {
    pub use ron::*;
}

pub mod serde_json {
    pub use serde_json::*;
}
