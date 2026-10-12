#[path = "asset_drag_payload/content.rs"]
mod content;
#[path = "asset_drag_payload/reference.rs"]
mod reference;

pub(super) use content::asset_drag_payload_from_snapshot;
pub(super) use content::is_asset_content_drag_payload;
pub(super) use reference::asset_drag_payload_from_reference;
