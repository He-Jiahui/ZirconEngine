use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiVirtualListWindow {
    pub first_visible: usize,
    pub last_visible_exclusive: usize,
}
