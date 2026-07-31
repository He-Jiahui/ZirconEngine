use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::surface::UiPointerButton;
use zircon_runtime_interface::ui::dispatch::UiInputModifiers;

use super::super::super::super::super::routing::PanePointerRoute;
use super::super::super::super::super::NativePointerButtonState;

pub(super) struct PaneButtonDispatchInput<'a> {
    pub(super) ui: &'a UiHostWindow,
    pub(super) presentation: &'a HostWindowPresentationData,
    pub(super) pointer: PanePointerRoute,
    pub(super) state: NativePointerButtonState,
    pub(super) button: UiPointerButton,
    pub(super) button_id: i32,
    pub(super) modifiers: UiInputModifiers,
    pub(super) cleared_text_input_frame: Option<FrameRect>,
}
