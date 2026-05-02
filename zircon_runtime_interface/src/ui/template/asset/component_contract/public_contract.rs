use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    UiComponentApiVersion, UiComponentBindingContract, UiComponentFocusContract, UiPublicPart,
    UiRootClassPolicy,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiComponentPublicContract {
    #[serde(default)]
    pub api_version: UiComponentApiVersion,
    #[serde(default)]
    pub public_parts: BTreeMap<String, UiPublicPart>,
    #[serde(default)]
    pub root_class_policy: UiRootClassPolicy,
    #[serde(default)]
    pub focus: UiComponentFocusContract,
    #[serde(default)]
    pub bindings: UiComponentBindingContract,
}

impl Default for UiComponentPublicContract {
    fn default() -> Self {
        Self {
            api_version: UiComponentApiVersion::DEFAULT,
            public_parts: BTreeMap::new(),
            root_class_policy: UiRootClassPolicy::AppendOnly,
            focus: UiComponentFocusContract::default(),
            bindings: UiComponentBindingContract::default(),
        }
    }
}
