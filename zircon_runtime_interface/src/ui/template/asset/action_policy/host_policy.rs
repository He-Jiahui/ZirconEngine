use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::UiActionSideEffectClass;

/// 动作绑定的宿主准入表；编译包按运行时或编辑器配置检查副作用类别。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiActionHostPolicy {
    #[serde(default)]
    pub allowed_side_effects: BTreeSet<UiActionSideEffectClass>,
}

impl UiActionHostPolicy {
    /// 运行时预设仅准入本地 UI 操作；编辑器预设另准入编辑写入和资产 I/O。
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
