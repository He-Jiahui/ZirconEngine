use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{Display, Formatter};

use crate::core::resource::ResourceLocator;

use super::AssetUuid;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct AssetReference {
    pub uuid: AssetUuid,
    pub locator: ResourceLocator,
}

impl AssetReference {
    pub fn new(uuid: AssetUuid, locator: ResourceLocator) -> Self {
        Self { uuid, locator }
    }

    pub fn from_locator(locator: ResourceLocator) -> Self {
        let uuid = AssetUuid::from_stable_label(&locator.to_string());
        Self::new(uuid, locator)
    }
}

impl Display for AssetReference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.locator)
    }
}

impl<'de> Deserialize<'de> for AssetReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Structured {
                uuid: AssetUuid,
                locator: ResourceLocator,
            },
            Legacy(ResourceLocator),
        }

        match Repr::deserialize(deserializer)? {
            Repr::Structured { uuid, locator } => Ok(Self::new(uuid, locator)),
            Repr::Legacy(locator) => Ok(Self::from_locator(locator)),
        }
    }
}
