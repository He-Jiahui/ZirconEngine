use serde::{Deserialize, Serialize};

/// Names the invocation arguments used to resolve an asset mutation target.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetWriteTargetDescriptor {
    asset_type_argument: String,
    locator_argument: String,
}

impl AssetWriteTargetDescriptor {
    pub fn new(
        asset_type_argument: impl Into<String>,
        locator_argument: impl Into<String>,
    ) -> Self {
        Self {
            asset_type_argument: asset_type_argument.into(),
            locator_argument: locator_argument.into(),
        }
    }

    pub fn asset_type_argument(&self) -> &str {
        &self.asset_type_argument
    }

    pub fn locator_argument(&self) -> &str {
        &self.locator_argument
    }
}
