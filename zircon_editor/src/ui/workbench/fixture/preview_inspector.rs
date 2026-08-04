use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreviewInspector {
    pub id: u64,
    pub name: String,
    pub parent: String,
    pub translation: [String; 3],
    #[serde(default = "default_scale")]
    pub scale: [String; 3],
}

fn default_scale() -> [String; 3] {
    ["1.00".to_string(), "1.00".to_string(), "1.00".to_string()]
}
