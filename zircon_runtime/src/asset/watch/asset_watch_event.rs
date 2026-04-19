use crate::asset::AssetUri;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetWatchEvent {
    Added(AssetUri),
    Modified(AssetUri),
    Removed(AssetUri),
    Renamed { from: AssetUri, to: AssetUri },
}
