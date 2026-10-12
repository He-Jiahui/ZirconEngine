use crate::scene::LevelMetadata;

use super::model::RuntimeSessionMetadata;

impl RuntimeSessionMetadata {
    /// 捕获 Level 可持久化的项目和显示信息；调用方可另行补充时间戳与标签。
    pub fn from_level_metadata(metadata: LevelMetadata) -> Self {
        Self {
            project_root: metadata.project_root,
            asset_uri: metadata.asset_uri,
            display_name: metadata.display_name,
            ..Self::default()
        }
    }

    /// 还原 Level 的可编辑元数据；会话时间与标签留在档案，不写入 Level。
    pub fn to_level_metadata(&self) -> LevelMetadata {
        LevelMetadata {
            project_root: self.project_root.clone(),
            asset_uri: self.asset_uri.clone(),
            display_name: self.display_name.clone(),
        }
    }
}
