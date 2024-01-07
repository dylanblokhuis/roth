use serde::{Deserialize, Serialize};
use tpaint::epaint::Rect;

#[derive(Debug, Serialize, Deserialize)]
pub enum EditorToRuntimeMsg {
    Todo,
    LayoutChange(Rect),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RuntimeToEditorMsg {
    Todo,
}
