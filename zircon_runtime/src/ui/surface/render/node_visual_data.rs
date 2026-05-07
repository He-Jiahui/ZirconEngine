use zircon_runtime_interface::ui::surface::{
    UiEditableTextState, UiResolvedStyle, UiVisualAssetRef,
};
use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

use super::resolve::{
    resolve_editable_text_state, resolve_image, resolve_opacity, resolve_style, resolve_text,
};

#[derive(Default)]
pub(super) struct UiNodeVisualData {
    pub(super) style: UiResolvedStyle,
    pub(super) text: Option<String>,
    pub(super) editable: Option<UiEditableTextState>,
    pub(super) image: Option<UiVisualAssetRef>,
    pub(super) opacity: f32,
}

impl UiNodeVisualData {
    pub(super) fn resolve(metadata: Option<&UiTemplateNodeMetadata>) -> Self {
        let text = resolve_text(metadata);
        let editable = resolve_editable_text_state(metadata, text.as_deref());
        Self {
            style: resolve_style(metadata),
            text,
            editable,
            image: resolve_image(metadata),
            opacity: resolve_opacity(metadata),
        }
    }
}
