use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityDrawerMode {
    Pinned,
    AutoHide,
    Collapsed,
}
