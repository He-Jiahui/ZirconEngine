use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiVisualAssetRef {
    Icon(String),
    Image(String),
}
