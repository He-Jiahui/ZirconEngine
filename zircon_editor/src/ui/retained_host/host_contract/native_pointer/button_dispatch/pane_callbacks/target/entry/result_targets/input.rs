use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::dispatch::UiInputModifiers;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::super::super::routing::PanePointerRoute;
use super::super::super::super::super::super::NativePointerButtonState;

pub(super) struct ResultPaneTargetInput<'host, 'pane> {
    pub(super) ui: &'host UiHostWindow,
    pub(super) pane_host: &'host PaneSurfaceHostContext<'pane>,
    pub(super) presentation: &'host HostWindowPresentationData,
    pub(super) pointer: &'host PanePointerRoute,
    pub(super) kind: i32,
    pub(super) state: NativePointerButtonState,
    pub(super) button: UiPointerButton,
    pub(super) button_id: i32,
    pub(super) modifiers: UiInputModifiers,
    pub(super) cleared_text_input_frame: Option<FrameRect>,
}
