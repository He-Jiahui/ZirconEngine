use crate::ui::tree::UiTemplateNodeMetadata;

use super::resolve::{resolve_image, resolve_opacity, resolve_style, resolve_text};
use super::{UiResolvedStyle, UiVisualAssetRef};

#[derive(Default)]
pub(super) struct UiNodeVisualData {
    pub(super) style: UiResolvedStyle,
    pub(super) text: Option<String>,
    pub(super) image: Option<UiVisualAssetRef>,
    pub(super) opacity: f32,
}

impl UiNodeVisualData {
    pub(super) fn resolve(metadata: Option<&UiTemplateNodeMetadata>) -> Self {
        Self {
            style: resolve_style(metadata),
            text: resolve_text(metadata),
            image: resolve_image(metadata),
            opacity: resolve_opacity(metadata),
        }
    }
}
