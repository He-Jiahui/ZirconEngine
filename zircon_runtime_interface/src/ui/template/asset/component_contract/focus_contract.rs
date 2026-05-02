use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiComponentFocusContract {
    #[serde(default)]
    pub root_focusable: bool,
    #[serde(default)]
    pub initial_focus: Option<String>,
    #[serde(default)]
    pub public_targets: BTreeMap<String, String>,
}
