use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiNavigationEventKind {
    Activate,
    Cancel,
    Next,
    Previous,
    Home,
    End,
    Up,
    Down,
    Left,
    Right,
}
