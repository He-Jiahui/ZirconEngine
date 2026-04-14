use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use uuid::Uuid;

use crate::ResourceLocator;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetUuid(Uuid);

impl AssetUuid {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_stable_label(label: &str) -> Self {
        Self(stable_uuid_from_components(
            "zircon-asset-uuid",
            &[label],
        ))
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

pub(crate) fn stable_uuid_from_components(namespace: &str, components: &[&str]) -> Uuid {
    fn hash_with(namespace: &str, value: &str) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        namespace.hash(&mut hasher);
        value.hash(&mut hasher);
        hasher.finish()
    }

    let mut joined = namespace.to_string();
    for component in components {
        joined.push('\x1f');
        joined.push_str(component);
    }

    let high = hash_with("zircon-stable-uuid/high", &joined).to_be_bytes();
    let low = hash_with("zircon-stable-uuid/low", &joined).to_be_bytes();
    let mut bytes = [0_u8; 16];
    bytes[..8].copy_from_slice(&high);
    bytes[8..].copy_from_slice(&low);
    Uuid::from_bytes(bytes)
}
