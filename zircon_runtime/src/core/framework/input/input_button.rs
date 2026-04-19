use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum InputButton {
    MouseLeft,
    MouseRight,
    MouseMiddle,
    Key(String),
}
