use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};

use super::{AssetUuid, ResourceLocator};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

impl Serialize for AssetReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Repr<'a> {
            uuid: AssetUuid,
            url: &'a ResourceLocator,
        }

        Repr {
            uuid: self.uuid,
            url: &self.locator,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AssetReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Repr {
            uuid: AssetUuid,
            url: ResourceLocator,
        }

        let Repr { uuid, url } = Repr::deserialize(deserializer)?;
        Ok(Self::new(uuid, url))
    }
}
