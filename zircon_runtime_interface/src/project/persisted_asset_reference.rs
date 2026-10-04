use std::error::Error;
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::project::AssetRef;
use crate::resource::{ResourceLocator, ResourceScheme};

/// Current project-file reference contract. Runtime-only locators never serialize through it.
/// 项目文件只持久化项目资产的 AssetRef 或 builtin:// 定位符。
/// 运行时专用定位符不得借此写入文档；读取时也须保持两类引用的区分。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PersistedAssetReference(PersistedAssetReferenceKind);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum PersistedAssetReferenceKind {
    Project {
        #[serde(flatten)]
        reference: AssetRef,
    },
    Builtin {
        locator: ResourceLocator,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersistedAssetReferenceError {
    locator: ResourceLocator,
}

impl Display for PersistedAssetReferenceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "persisted builtin reference requires builtin://, found {}",
            self.locator
        )
    }
}

impl Error for PersistedAssetReferenceError {}

impl PersistedAssetReference {
    pub fn project(reference: AssetRef) -> Self {
        Self(PersistedAssetReferenceKind::Project { reference })
    }

    pub fn try_builtin(locator: ResourceLocator) -> Result<Self, PersistedAssetReferenceError> {
        if locator.scheme() != ResourceScheme::Builtin {
            return Err(PersistedAssetReferenceError { locator });
        }
        Ok(Self(PersistedAssetReferenceKind::Builtin { locator }))
    }

    /// 仅供已判定为 builtin:// 的内部调用；不可信定位符应使用 `try_builtin`。
    /// 传入其他 scheme 会 panic。
    pub fn builtin(locator: ResourceLocator) -> Self {
        Self::try_builtin(locator)
            .expect("PersistedAssetReference::builtin requires a builtin:// locator")
    }

    pub fn project_ref(&self) -> Option<&AssetRef> {
        match &self.0 {
            PersistedAssetReferenceKind::Project { reference } => Some(reference),
            PersistedAssetReferenceKind::Builtin { .. } => None,
        }
    }

    pub fn builtin_locator(&self) -> Option<&ResourceLocator> {
        match &self.0 {
            PersistedAssetReferenceKind::Builtin { locator } => Some(locator),
            PersistedAssetReferenceKind::Project { .. } => None,
        }
    }
}

impl Serialize for PersistedAssetReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PersistedAssetReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match PersistedAssetReferenceKind::deserialize(deserializer)? {
            PersistedAssetReferenceKind::Project { reference } => Ok(Self::project(reference)),
            PersistedAssetReferenceKind::Builtin { locator } => {
                Self::try_builtin(locator).map_err(serde::de::Error::custom)
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/persisted_asset_reference.rs"]
mod tests;
