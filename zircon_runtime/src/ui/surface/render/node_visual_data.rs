use zircon_runtime_interface::ui::event_ui::UiStateFlags;
use zircon_runtime_interface::ui::surface::{
    UiEditableTextState, UiResolvedStyle, UiVisualAssetRef,
};
use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

use super::resolve::{
    resolve_editable_text_state, resolve_image, resolve_opacity, resolve_painter_family,
    resolve_painter_state, resolve_style, resolve_text,
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
    pub(super) fn resolve(
        metadata: Option<&UiTemplateNodeMetadata>,
        state_flags: &UiStateFlags,
    ) -> Self {
        let text = resolve_text(metadata);
        let editable = resolve_editable_text_state(metadata, text.as_deref());
        Self {
            style: resolve_style(metadata).with_painter_state(
                resolve_painter_family(metadata),
                resolve_painter_state(metadata, state_flags),
            ),
            text,
            editable,
            image: resolve_image(metadata),
            opacity: resolve_opacity(metadata),
        }
    }
}
