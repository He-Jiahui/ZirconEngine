use std::sync::Arc;

use super::viewport_icon_sprite::ViewportIconSprite;

#[derive(Clone)]
pub(super) enum IconEntry {
    Unloaded,
    Missing,
    Ready(Arc<ViewportIconSprite>),
}
