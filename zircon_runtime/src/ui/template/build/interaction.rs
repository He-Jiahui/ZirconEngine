use crate::ui::event_ui::UiStateFlags;
use crate::ui::template::UiTemplateNode;
use crate::ui::tree::UiInputPolicy;

pub(super) fn infer_interaction(node: &UiTemplateNode) -> (UiStateFlags, UiInputPolicy) {
    let is_interactive = !node.bindings.is_empty()
        || matches!(
            node.component.as_deref(),
            Some("Button" | "UiHostIconButton" | "TextField")
        );
    (
        UiStateFlags {
            visible: true,
            enabled: true,
            clickable: is_interactive,
            hoverable: is_interactive,
            focusable: is_interactive,
            pressed: false,
            checked: false,
            dirty: false,
        },
        if is_interactive {
            UiInputPolicy::Receive
        } else {
            UiInputPolicy::Inherit
        },
    )
}
