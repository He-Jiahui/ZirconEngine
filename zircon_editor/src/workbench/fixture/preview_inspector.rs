use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreviewInspector {
    pub id: u64,
    pub name: String,
    pub parent: String,
    pub translation: [String; 3],
}
