use crate::project::RelPath;
use crate::resource::AssetUuid;

/// 持久化项目资产引用：GUID 是注册表解析的权威身份，path_hint 只供候选定位及迁移修复，
/// sub 与注册项中的子资产标签精确匹配。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssetRef {
    pub(super) guid: AssetUuid,
    pub(super) path_hint: RelPath,
    pub(super) sub: Option<String>,
}

impl AssetRef {
    pub fn guid(&self) -> AssetUuid {
        self.guid
    }

    pub fn path_hint(&self) -> &RelPath {
        &self.path_hint
    }

    pub fn sub(&self) -> Option<&str> {
        self.sub.as_deref()
    }
}
