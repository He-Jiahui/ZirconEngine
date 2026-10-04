use crate::asset::AssetUri;

/// notify 适配层产生的路径变化；重命名同时保留旧、新 URI，供后续事件折叠与索引同步。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetWatchEvent {
    Added(AssetUri),
    Modified(AssetUri),
    Removed(AssetUri),
    Renamed { from: AssetUri, to: AssetUri },
}
