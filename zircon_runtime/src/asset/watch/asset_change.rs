use crate::asset::AssetUri;

use super::asset_change_kind::AssetChangeKind;

#[derive(Clone, Debug, PartialEq, Eq)]
/// 向项目增量导入传播的逻辑变化；重命名必须保留 previous_uri 才能退役原目录项。
pub struct AssetChange {
    pub kind: AssetChangeKind,
    pub uri: AssetUri,
    pub previous_uri: Option<AssetUri>,
}
