use super::*;

impl AssetReferenceListSurfacePointerState {
    pub(super) fn new() -> Self {
        Self {
            bridge: AssetReferenceListPointerBridge::new(),
            state: AssetListPointerState::default(),
            size: UiSize::new(0.0, 0.0),
        }
    }
}

impl AssetSurfacePointerState {
    pub(super) fn new() -> Self {
        Self {
            tree_bridge: AssetFolderTreePointerBridge::new(),
            tree_state: AssetListPointerState::default(),
            tree_size: UiSize::new(0.0, 0.0),
            content_bridge: AssetContentListPointerBridge::new(),
            content_state: AssetListPointerState::default(),
            content_size: UiSize::new(0.0, 0.0),
            references: AssetReferenceListSurfacePointerState::new(),
            used_by: AssetReferenceListSurfacePointerState::new(),
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
