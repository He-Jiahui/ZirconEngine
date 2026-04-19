use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use uuid::Uuid;

use super::stable_uuid_from_components;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetUuid(pub(crate) Uuid);

impl AssetUuid {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_stable_label(label: &str) -> Self {
        Self(stable_uuid_from_components("zircon-asset-uuid", &[label]))
    }
}

impl Display for AssetUuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for AssetUuid {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self)
    }
}
