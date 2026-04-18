use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SelectionCommand {
    SelectSceneNode { node_id: u64 },
}
