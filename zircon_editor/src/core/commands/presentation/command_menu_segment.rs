//! 菜单段把可合并的稳定身份与可翻译标签分离；同一分组的匹配依赖ID，显示语言变化不应改变命令路径。

use serde::{Deserialize, Serialize};

use crate::core::i18n::EditorLocalizationKey;

use super::EditorCommandMenuSegmentId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorCommandMenuSegment {
    id: EditorCommandMenuSegmentId,
    label_key: EditorLocalizationKey,
}

impl EditorCommandMenuSegment {
    pub fn parse(id: impl Into<String>, label_key: impl Into<String>) -> Result<Self, String> {
        Ok(Self {
            id: EditorCommandMenuSegmentId::parse(id)?,
            label_key: EditorLocalizationKey::parse(label_key)?,
        })
    }

    pub fn id(&self) -> &EditorCommandMenuSegmentId {
        &self.id
    }

    pub fn label_key(&self) -> &str {
        self.label_key.as_str()
    }
}
