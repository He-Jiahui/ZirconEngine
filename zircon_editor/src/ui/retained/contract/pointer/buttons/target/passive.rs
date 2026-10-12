use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::super::routing::{PanePointerRoute, PanePointerTarget};
use super::super::super::super::NativePointerButtonState;
use super::super::fallback::dispatch_passive_pane_button;

pub(super) fn dispatch_passive_pane_target_button(
    pointer: &PanePointerRoute,
    state: NativePointerButtonState,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    match &pointer.target {
        PanePointerTarget::Console
        | PanePointerTarget::Inspector
        | PanePointerTarget::BrowserAssetDetails
        | PanePointerTarget::UiAsset
        | PanePointerTarget::Other => dispatch_passive_pane_button(state, cleared_text_input_frame),
        _ => None,
    }
}
