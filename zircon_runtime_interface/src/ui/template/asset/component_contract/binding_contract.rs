use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiComponentBindingContract {
    #[serde(default)]
    pub public_actions: BTreeMap<String, UiPublicBindingRoute>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPublicBindingRoute {
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub payload_kind: Option<String>,
}
