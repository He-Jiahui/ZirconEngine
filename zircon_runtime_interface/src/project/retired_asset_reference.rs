use crate::resource::{AssetUuid, ResourceLocator};

/// Validated retired `{ uuid, url }` shape passed to a caller-owned identity resolver.
/// 这里只保证旧字段已解析成 GUID 与定位符；不证明二者对应同一注册资产，scheme 和身份由解析器决定。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetiredAssetReference {
    guid: AssetUuid,
    locator: ResourceLocator,
}

impl RetiredAssetReference {
    pub(super) fn new(guid: AssetUuid, locator: ResourceLocator) -> Self {
        Self { guid, locator }
    }

    pub fn guid(&self) -> AssetUuid {
        self.guid
    }

    pub fn locator(&self) -> &ResourceLocator {
        &self.locator
    }
}
