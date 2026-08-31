pub(super) struct UiAssetDetailSurfaceBinding<'a> {
    pub(super) instance_id: &'a str,
    pub(super) detail_id: &'a str,
    pub(super) action_id: &'a str,
    pub(super) item_index: i32,
}

impl<'a> UiAssetDetailSurfaceBinding<'a> {
    const PREFIX: &'static str = "ui_asset_detail";

    pub(super) fn parse(binding_id: &'a str) -> Option<Self> {
        let mut parts = binding_id.split('|');
        let prefix = parts.next()?;
        if prefix != Self::PREFIX {
            return None;
        }
        let instance_id = parts.next()?;
        let detail_id = parts.next()?;
        let action_id = parts.next()?;
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

#[cfg(test)]
#[path = "ui_asset_detail/borrowed_binding_tests.rs"]
mod borrowed_binding_tests;
