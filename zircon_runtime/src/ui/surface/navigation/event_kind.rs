use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiNavigationEventKind {
    Activate,
    Cancel,
    Next,
    Previous,
    Up,
    Down,
    Left,
    Right,
}
