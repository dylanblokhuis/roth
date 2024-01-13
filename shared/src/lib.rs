use bevy::ecs::entity::Entity;
use serde::{Deserialize, Serialize};
use winit::{dpi::PhysicalPosition, keyboard::SmolStr};

#[derive(Debug, Serialize, Deserialize)]
pub enum EditorToRuntimeMsg {
    Shutdown,
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
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RuntimeToEditorMsg {
    Todo,
    // Entities { entities: Vec<Entity> },
}
