use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use uuid::Uuid;

use super::{stable_uuid_from_components, AssetUuid, ResourceLocator, ResourceScheme};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceId(Uuid);

impl ResourceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_locator(locator: &ResourceLocator) -> Self {
        match locator.scheme() {
            ResourceScheme::Memory => Self::new(),
            ResourceScheme::Res
            | ResourceScheme::Library
            | ResourceScheme::Package
            | ResourceScheme::Builtin => Self::from_stable_label(&locator.to_string()),
        }
    }

    pub fn from_stable_label(label: &str) -> Self {
        Self(stable_uuid_from_components(
            "zircon-resource-id/stable-label",
            &[label],
        ))
    }

    pub fn from_asset_uuid(uuid: AssetUuid) -> Self {
        Self(stable_uuid_from_components(
            "zircon-resource-id/asset-uuid",
            &[uuid.to_string().as_str()],
        ))
    }
}

impl Display for ResourceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ResourceId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self)
    }
}
