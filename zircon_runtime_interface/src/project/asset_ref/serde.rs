use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::project::RelPath;
use crate::resource::AssetUuid;

use super::AssetRef;

/// 项目资产引用的持久化格式；读取后须经 RelPath 和 try_new 对外部输入做词法路径及子资产标签约束。
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AssetRefRepr {
    guid: AssetUuid,
    path_hint: RelPath,
    sub: Option<String>,
}

impl Serialize for AssetRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        AssetRefRepr {
            guid: self.guid,
            path_hint: self.path_hint.clone(),
            sub: self.sub.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AssetRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let repr = AssetRefRepr::deserialize(deserializer)?;
        Self::try_new(repr.guid, repr.path_hint, repr.sub).map_err(serde::de::Error::custom)
    }
}
