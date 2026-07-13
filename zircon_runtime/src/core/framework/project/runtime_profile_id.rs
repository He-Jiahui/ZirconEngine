use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeProfileId {
    Minimal,
    Client2d,
    Client3d,
    Editor,
    Dev,
    Server,
}
