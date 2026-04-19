use crate::asset::AssetUri;

use super::{asset_change::AssetChange, asset_change_kind::AssetChangeKind};

impl AssetChange {
    pub fn new(kind: AssetChangeKind, uri: AssetUri, previous_uri: Option<AssetUri>) -> Self {
        Self {
            kind,
            uri,
            previous_uri,
        }
    }
}
