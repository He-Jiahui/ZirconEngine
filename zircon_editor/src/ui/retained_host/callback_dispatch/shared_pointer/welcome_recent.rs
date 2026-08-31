use zircon_runtime_interface::ui::{
    binding::{UiBindingValue, UiEventKind},
    layout::UiPoint,
};

use crate::ui::binding_dispatch::WelcomeHostEvent;
use crate::ui::retained_host::welcome_recent_pointer::{
    WelcomeRecentPointerAction, WelcomeRecentPointerBridge, WelcomeRecentPointerDispatch,
};

use super::super::{BuiltinWelcomeSurfaceTemplateBridge, dispatch_builtin_welcome_surface_control};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedWelcomeRecentPointerClickDispatch {
    pub pointer: WelcomeRecentPointerDispatch,
    pub event: Option<WelcomeHostEvent>,
}

pub(crate) fn dispatch_shared_welcome_recent_pointer_click(
    bridge: &BuiltinWelcomeSurfaceTemplateBridge,
    pointer_bridge: &mut WelcomeRecentPointerBridge,
    point: UiPoint,
) -> Result<SharedWelcomeRecentPointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(point);
    let event = match pointer
        .route
        .and_then(|route| pointer_bridge.action_target_for_route(route))
    {
        Some((action, path)) => {
            let control_id = match action {
                WelcomeRecentPointerAction::Open => "OpenRecentProject",
                WelcomeRecentPointerAction::Safe => "SafeRecentProject",
                WelcomeRecentPointerAction::Recover => "RecoverRecentProject",
                WelcomeRecentPointerAction::Remove => "RemoveRecentProject",
            };
            dispatch_builtin_welcome_surface_control(
                bridge,
                control_id,
                UiEventKind::Click,
                vec![UiBindingValue::string(path)],
            )
            .transpose()?
        }
        _ => None,
    };

    Ok(SharedWelcomeRecentPointerClickDispatch { pointer, event })
}
