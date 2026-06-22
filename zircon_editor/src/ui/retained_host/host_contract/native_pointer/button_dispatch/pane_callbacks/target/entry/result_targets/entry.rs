use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::super::super::routing::PanePointerRoute;
use super::super::super::super::super::super::NativePointerButtonState;
use super::input::ResultPaneTargetInput;
use super::sequence::dispatch_result_pane_target_sequence;

pub(in super::super) fn dispatch_result_pane_targets(
    ui: &UiHostWindow,
    pane_host: &PaneSurfaceHostContext<'_>,
    presentation: &HostWindowPresentationData,
    pointer: &PanePointerRoute,
    kind: i32,
    state: NativePointerButtonState,
    button: UiPointerButton,
    button_id: i32,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    dispatch_result_pane_target_sequence(ResultPaneTargetInput {
        ui,
        pane_host,
        presentation,
        pointer,
        kind,
        state,
        button,
        button_id,
        cleared_text_input_frame,
    })
}
