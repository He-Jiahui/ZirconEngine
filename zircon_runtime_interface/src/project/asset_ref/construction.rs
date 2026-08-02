use crate::project::RelPath;
use crate::resource::AssetUuid;

use super::{validation::validate_sub_path, AssetRef, AssetRefError};

impl AssetRef {
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
