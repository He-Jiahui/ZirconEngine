use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};

use super::{AssetUuid, ResourceLocator};

/// 跨资产文档传递 UUID 身份与可读定位符；定位符可随迁移变化，UUID 仍用于持久引用。
/// 序列化字段 `url` 保持既有资产文档格式。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssetReference {
    pub uuid: AssetUuid,
    pub locator: ResourceLocator,
}

impl AssetReference {
    pub fn new(uuid: AssetUuid, locator: ResourceLocator) -> Self {
        Self { uuid, locator }
    }

    /// 为仅有定位符的引用推导身份；已有资产 UUID 应用 `new` 保留。
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

        if !serializer.is_human_readable() {
            #[derive(Serialize)]
            struct BinaryRepr {
                uuid: String,
                url: String,
            }

            return BinaryRepr {
                uuid: self.uuid.to_string(),
                url: self.locator.to_string(),
            }
            .serialize(serializer);
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

        if !deserializer.is_human_readable() {
            #[derive(Deserialize)]
            struct BinaryRepr {
                uuid: String,
                url: String,
            }

            let BinaryRepr { uuid, url } = BinaryRepr::deserialize(deserializer)?;
            let uuid = uuid.parse::<AssetUuid>().map_err(D::Error::custom)?;
            let url = ResourceLocator::parse(&url).map_err(D::Error::custom)?;
            return Ok(Self::new(uuid, url));
        }

        let Repr { uuid, url } = Repr::deserialize(deserializer)?;
        Ok(Self::new(uuid, url))
    }
}
