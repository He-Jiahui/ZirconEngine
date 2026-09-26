use crate::project::RelPath;
use crate::resource::AssetUuid;

use super::{validation::validate_sub_path, AssetRef, AssetRefError};

impl AssetRef {
    /// 创建、读取或迁移引用时统一约束子资产标签；GUID 与提示路径的对应关系由项目注册表解析。
    pub fn try_new(
        guid: AssetUuid,
        path_hint: RelPath,
        sub: Option<String>,
    ) -> Result<Self, AssetRefError> {
        if let Some(sub) = sub.as_deref() {
            validate_sub_path(sub)?;
        }
        Ok(Self {
            guid,
            path_hint,
            sub,
        })
    }
}
