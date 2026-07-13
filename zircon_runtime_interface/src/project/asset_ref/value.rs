use crate::project::RelPath;
use crate::resource::AssetUuid;

/// Persistent project-asset identity with a movable path hint and optional subasset path.
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
