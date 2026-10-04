use crate::scene::dynamic_scene::{DynamicScene, DynamicSceneError};
use zircon_runtime_interface::serialization::write_versioned_text;

impl DynamicScene {
    /// 输出可持久化的当前版本文档；先检查场景身份和来源唯一性，供编辑器快照及会话归档复用。
    pub fn to_versioned_json_pretty(&self) -> Result<String, DynamicSceneError> {
        self.ensure_supported()?;
        Ok(write_versioned_text(self)?)
    }
}
