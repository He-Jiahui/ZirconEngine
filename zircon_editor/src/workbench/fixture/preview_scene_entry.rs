use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreviewSceneEntry {
    pub id: u64,
    pub name: String,
    pub depth: usize,
    pub selected: bool,
}
