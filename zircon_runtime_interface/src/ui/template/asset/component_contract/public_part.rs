use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPublicPart {
    #[serde(default)]
    pub node_id: String,
    #[serde(default)]
    pub control_id: Option<String>,
}
