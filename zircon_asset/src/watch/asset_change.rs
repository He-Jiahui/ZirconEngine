use crate::AssetUri;

use super::asset_change_kind::AssetChangeKind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetChange {
    pub kind: AssetChangeKind,
    pub uri: AssetUri,
    pub previous_uri: Option<AssetUri>,
}
