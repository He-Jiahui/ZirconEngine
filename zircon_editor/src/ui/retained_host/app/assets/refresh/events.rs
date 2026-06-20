use super::super::super::*;

mod runtime;
mod startup;

pub(super) struct AssetRefreshEvents {
    pub(super) asset_changes: Vec<AssetChange>,
    pub(super) editor_asset_changes: Vec<EditorAssetChange>,
    pub(super) resource_changes: Vec<ResourceEvent>,
}

impl AssetRefreshEvents {
    pub(super) fn is_empty(&self) -> bool {
        self.asset_changes.is_empty()
            && self.editor_asset_changes.is_empty()
            && self.resource_changes.is_empty()
    }
}
