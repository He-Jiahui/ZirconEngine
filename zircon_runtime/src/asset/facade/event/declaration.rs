use serde::{Deserialize, Serialize};

use super::super::{Asset, Handle};
use crate::core::resource::ResourceLocator;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetEventKind {
    Added,
    Modified,
    Removed,
    Renamed,
    ReloadFailed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetEvent<TAsset: Asset> {
    Added {
        handle: Handle<TAsset>,
        locator: Option<ResourceLocator>,
        revision: u64,
    },
    Modified {
        handle: Handle<TAsset>,
        locator: Option<ResourceLocator>,
        revision: u64,
    },
    Removed {
        handle: Handle<TAsset>,
        locator: Option<ResourceLocator>,
        revision: u64,
    },
    Renamed {
        handle: Handle<TAsset>,
        locator: Option<ResourceLocator>,
        previous_locator: Option<ResourceLocator>,
        revision: u64,
    },
    ReloadFailed {
        handle: Handle<TAsset>,
        locator: Option<ResourceLocator>,
        revision: u64,
    },
}
