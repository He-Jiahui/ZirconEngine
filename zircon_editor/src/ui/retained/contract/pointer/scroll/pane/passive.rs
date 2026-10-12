use super::super::super::routing::{PanePointerRoute, PanePointerTarget};

pub(super) fn is_passive_pane_scroll_target(pointer: &PanePointerRoute) -> bool {
    matches!(
        &pointer.target,
        PanePointerTarget::TemplateNode(_)
            | PanePointerTarget::ViewportToolbar { .. }
            | PanePointerTarget::UiAsset
            | PanePointerTarget::Other
    )
}
