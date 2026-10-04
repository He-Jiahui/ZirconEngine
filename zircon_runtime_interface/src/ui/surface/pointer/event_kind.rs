use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiPointerEventKind {
    Down,
    Up,
    Move,
    Scroll,
    Cancel,
}
