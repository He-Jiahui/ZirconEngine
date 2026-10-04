use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use uuid::Uuid;

use super::stable_uuid_from_components;

/// 资源目录与引用共用的身份；有稳定来源标签时可重建，否则通过 `new` 分配随机 UUID。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetUuid(pub(crate) Uuid);

impl AssetUuid {
    /// 为没有稳定来源标签的新资源随机生成 v4 身份。
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 从稳定来源标签派生可重建身份，例如资源 locator，使重新扫描不依赖发现顺序。
    pub fn from_stable_label(label: &str) -> Self {
        Self(stable_uuid_from_components("zircon-asset-uuid", &[label]))
    }

    /// Returns the UUID bytes for allocation-free deterministic ordering.
    pub fn binary_key(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }
}

impl Default for AssetUuid {
    fn default() -> Self {
        Self::new()
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
