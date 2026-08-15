use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::core::math::UVec2;
use crate::ui::{RuntimeUiFixture, RuntimeUiManager};
use zircon_runtime_interface::ui::{
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiDispatchDisposition, UiDispatchEffect, UiDispatchPhase, UiFocusEffectReason,
        UiInputEvent, UiInputEventMetadata, UiInputRoutePolicy, UiInputSequence, UiInputTimestamp,
        UiKeyboardInputState, UiNavigationDispatchEffect, UiNavigationInputEvent,
        UiPointerCaptureReason, UiPointerDispatchEffect, UiPointerId, UiPointerInputEvent,
        UiPointerSource, UiPreciseScrollDelta, UiWindowId,
    },
    event_ui::UiNodeId,
    focus::UiFocusedInputKind,
    layout::{UiPoint, UiSize},
    surface::{UiNavigationEventKind, UiPointerButton, UiPointerEventKind},
    window::{UiRuntimeEventAdapterContext, UiWindowInputContext, UiWindowPlatformInputEvent},
};
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZrRuntimeViewportMetricsV1,
    ZrRuntimeViewportSizeV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1,
    ZR_RUNTIME_KEY_ACTION_PRESSED_V1, ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
    ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1, ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
    ZR_RUNTIME_WINDOW_THEME_DARK_V1,
};

mod abi;
mod normalized;
mod platform;

fn runtime_event_context() -> UiRuntimeEventAdapterContext {
    UiRuntimeEventAdapterContext::for_window("runtime.main")
        .with_timestamp(UiInputTimestamp::from_micros(42))
        .with_sequence(UiInputSequence::new(7))
}
fn input_metadata() -> UiInputEventMetadata {
    UiInputEventMetadata::new(UiInputTimestamp::from_micros(99), UiInputSequence::new(11))
}
fn input_context() -> UiWindowInputContext {
    UiWindowInputContext {
        metadata: input_metadata(),
        ..UiWindowInputContext::default()
    }
    .with_pointer_id(UiPointerId::new(29))
}
fn navigation_input_context() -> UiWindowInputContext {
    UiWindowInputContext {
        metadata: input_metadata(),
        ..UiWindowInputContext::default()
    }
}
fn node_id_by_control_id(manager: &RuntimeUiManager, control_id: &str) -> UiNodeId {
    manager
        .surface()
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some(control_id)
        })
        .unwrap_or_else(|| panic!("{control_id} should be projected"))
        .node_id
}
const fn viewport() -> ZrRuntimeViewportHandle {
    ZrRuntimeViewportHandle::new(1)
}
