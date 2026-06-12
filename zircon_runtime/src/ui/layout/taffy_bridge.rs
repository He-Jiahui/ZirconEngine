use taffy::style::{Display, Style};
use zircon_runtime_interface::ui::layout::{
    BoxConstraints, UiContainerKind, UiLayoutEngineFamily, UiLayoutEngineRequest,
};

use super::style_mapping::{
    taffy_display_for_family as mapped_taffy_display_for_family, taffy_style_from_ui_layout_style,
    ui_layout_style_from_container,
};

/// Converts the subset of Zircon layout contracts that can be solved by Taffy.
/// Overlay, Canvas, Popup, Scroll, and VirtualList stay outside this bridge by design.
pub fn taffy_style_for_container(
    container: UiContainerKind,
    constraints: BoxConstraints,
) -> Option<Style> {
    let request = UiLayoutEngineRequest::from_container_kind(container);
    request.family.is_taffy_owned().then_some(())?;
    let style = ui_layout_style_from_container(container, constraints).ok()?;
    taffy_style_from_ui_layout_style(&style).ok()
}

pub fn taffy_display_for_family(family: UiLayoutEngineFamily) -> Option<Display> {
    mapped_taffy_display_for_family(family)
}
