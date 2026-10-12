use crate::resource::{AssetUuid, ResourceLocator};

/// Validated retired `{ uuid, url }` shape passed to a caller-owned identity resolver.
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
