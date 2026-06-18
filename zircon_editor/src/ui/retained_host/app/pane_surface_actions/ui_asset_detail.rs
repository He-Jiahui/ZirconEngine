pub(super) struct UiAssetDetailSurfaceBinding {
    pub(super) instance_id: String,
    pub(super) detail_id: String,
    pub(super) action_id: String,
    pub(super) item_index: i32,
}

impl UiAssetDetailSurfaceBinding {
    const PREFIX: &'static str = "ui_asset_detail";

    pub(super) fn parse(binding_id: &str) -> Option<Self> {
        let mut parts = binding_id.split('|');
        let prefix = parts.next()?;
        if prefix != Self::PREFIX {
            return None;
        }
        let instance_id = parts.next()?.to_string();
        let detail_id = parts.next()?.to_string();
        let action_id = parts.next()?.to_string();
        let item_index = parts.next()?.parse().ok()?;
        if parts.next().is_some()
            || instance_id.is_empty()
            || detail_id.is_empty()
            || action_id.is_empty()
        {
            return None;
        }
        Some(Self {
            instance_id,
            detail_id,
            action_id,
            item_index,
        })
    }
}
