use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::UiActionSideEffectClass;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiActionHostPolicy {
    #[serde(default)]
    pub allowed_side_effects: BTreeSet<UiActionSideEffectClass>,
}

impl UiActionHostPolicy {
    pub fn runtime_default() -> Self {
        Self::new([UiActionSideEffectClass::LocalUi])
    }

    pub fn editor_authoring() -> Self {
        Self::new([
            UiActionSideEffectClass::LocalUi,
            UiActionSideEffectClass::EditorMutation,
            UiActionSideEffectClass::AssetIo,
        ])
    }

    pub fn new(classes: impl IntoIterator<Item = UiActionSideEffectClass>) -> Self {
        Self {
            allowed_side_effects: classes.into_iter().collect(),
        }
    }

    pub fn allows(&self, side_effect: UiActionSideEffectClass) -> bool {
        self.allowed_side_effects.contains(&side_effect)
    }
}
