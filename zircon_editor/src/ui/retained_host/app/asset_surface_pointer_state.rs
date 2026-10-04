use super::*;
use crate::ui::workbench::snapshot::{AssetViewMode, AssetWorkspaceItemGeneration};
use std::time::{Duration, Instant};

const ASSET_DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(500);

#[derive(Default)]
pub(crate) struct AssetActivationClickTracker {
    last_click: Option<(
        String,
        String,
        Option<u64>,
        AssetWorkspaceItemGeneration,
        u64,
        AssetViewMode,
        Instant,
    )>,
}

impl AssetActivationClickTracker {
    pub(crate) fn clear(&mut self) {
        self.last_click = None;
    }

    pub(crate) fn register_click(
        &mut self,
        asset_uuid: Option<&str>,
        asset_locator: Option<&str>,
        resource_revision: Option<u64>,
        items: &AssetWorkspaceItemGeneration,
        catalog_revision: u64,
        view_mode: AssetViewMode,
        at: Instant,
    ) -> bool {
        if let Some((
            last_uuid,
            last_locator,
            last_resource_revision,
            last_items,
            last_revision,
            last_view,
            last_at,
        )) = self.last_click.take()
        {
            if asset_uuid == Some(last_uuid.as_str())
                && asset_locator == Some(last_locator.as_str())
                && resource_revision == last_resource_revision
                && items.shares_item_identity_with(&last_items)
                && catalog_revision == last_revision
                && view_mode == last_view
                && at.saturating_duration_since(last_at) <= ASSET_DOUBLE_CLICK_WINDOW
            {
                return true;
            }
        }
        if let (Some(asset_uuid), Some(asset_locator)) = (asset_uuid, asset_locator) {
            self.last_click = Some((
                asset_uuid.to_owned(),
                asset_locator.to_owned(),
                resource_revision,
                items.clone(),
                catalog_revision,
                view_mode,
                at,
            ));
        }
        false
    }
}

pub(crate) struct AssetSurfacePointerState {
    pub(super) snapshot: Option<Arc<crate::ui::workbench::snapshot::AssetWorkspaceSnapshot>>,
    pub(super) tree_bridge: AssetFolderTreePointerBridge,
    pub(super) tree_state: AssetListPointerState,
    pub(super) tree_size: UiSize,
    pub(super) content_bridge: AssetContentListPointerBridge,
    pub(super) content_state: AssetListPointerState,
    pub(super) content_size: UiSize,
    pub(super) references: AssetReferenceListSurfacePointerState,
    pub(super) used_by: AssetReferenceListSurfacePointerState,
    pub(super) activation_clicks: AssetActivationClickTracker,
}

pub(crate) struct AssetReferenceListSurfacePointerState {
    pub(super) bridge: AssetReferenceListPointerBridge,
    pub(super) state: AssetListPointerState,
    pub(super) size: UiSize,
}

impl AssetReferenceListSurfacePointerState {
    pub(super) fn new() -> Self {
        Self {
            bridge: AssetReferenceListPointerBridge::new(),
            state: AssetListPointerState::default(),
            size: UiSize::new(0.0, 0.0),
        }
    }
}

#[cfg(test)]
#[path = "tests/asset_surface_pointer_state_activation_click_tests.rs"]
mod activation_click_tests;

impl AssetSurfacePointerState {
    pub(super) fn new() -> Self {
        Self {
            snapshot: None,
            tree_bridge: AssetFolderTreePointerBridge::new(),
            tree_state: AssetListPointerState::default(),
            tree_size: UiSize::new(0.0, 0.0),
            content_bridge: AssetContentListPointerBridge::new(),
            content_state: AssetListPointerState::default(),
            content_size: UiSize::new(0.0, 0.0),
            references: AssetReferenceListSurfacePointerState::new(),
            used_by: AssetReferenceListSurfacePointerState::new(),
            activation_clicks: AssetActivationClickTracker::default(),
        }
    }

    pub(super) fn reference_list(
        &self,
        list_kind: &str,
    ) -> Option<&AssetReferenceListSurfacePointerState> {
        match list_kind {
            "references" => Some(&self.references),
            "used_by" => Some(&self.used_by),
            _ => None,
        }
    }

    pub(super) fn reference_list_mut(
        &mut self,
        list_kind: &str,
    ) -> Option<&mut AssetReferenceListSurfacePointerState> {
        match list_kind {
            "references" => Some(&mut self.references),
            "used_by" => Some(&mut self.used_by),
            _ => None,
        }
    }
}
