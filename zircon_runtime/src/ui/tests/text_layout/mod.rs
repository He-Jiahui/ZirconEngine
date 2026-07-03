use crate::ui::surface::UiSurface;
use crate::ui::text::measure_text_size;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    surface::{
        UiRenderCommand, UiResolvedStyle, UiResolvedTextLayout, UiTextAlign, UiTextDirection,
        UiTextOverflow, UiTextRange, UiTextRunKind, UiTextWrap, UiTextWritingMode,
    },
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

mod alignment;
mod direction;
mod edit_state;
mod overflow;
mod wrapping;

fn visible_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: false,
        hoverable: false,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

fn first_text_layout(surface: &UiSurface) -> &UiResolvedTextLayout {
    first_text_layout_command(surface)
        .text_layout
        .as_ref()
        .expect("render extract should contain a resolved text layout")
}

fn first_text_layout_command(surface: &UiSurface) -> &UiRenderCommand {
    surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.text_layout.is_some())
        .expect("render extract should contain a text layout command")
}
