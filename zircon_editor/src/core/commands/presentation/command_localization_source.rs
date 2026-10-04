//! 区分内建命令字典与插件自己的本地化包；该来源随贡献声明持久化，具体包在准入时绑定。

use serde::{Deserialize, Serialize};

use crate::core::i18n::EditorLocalizationBundleId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorCommandLocalizationSource {
    Builtin,
    Bundle(EditorLocalizationBundleId),
}

impl EditorCommandLocalizationSource {
    pub fn bundle_id(&self) -> Option<&EditorLocalizationBundleId> {
        match self {
            Self::Builtin => None,
            Self::Bundle(bundle_id) => Some(bundle_id),
        }
    }
}
