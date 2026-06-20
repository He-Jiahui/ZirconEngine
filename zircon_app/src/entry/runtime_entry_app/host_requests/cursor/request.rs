use winit::dpi::LogicalPosition;
use winit::window::{CursorGrabMode, Window};
use zircon_runtime::diagnostic_log::write_warn;
use zircon_runtime_interface::{
    ZrRuntimeCursorGrabModeV1, ZrRuntimeCursorHostRequestKindV1, ZrRuntimeCursorHostRequestV1,
};

pub(in crate::entry::runtime_entry_app) fn apply_runtime_cursor_host_request(
    window: &dyn Window,
    request: ZrRuntimeCursorHostRequestV1,
) -> Result<(), String> {
    match request.kind {
        ZrRuntimeCursorHostRequestKindV1::SetVisible => {
            window.set_cursor_visible(request.value);
            Ok(())
        }
        ZrRuntimeCursorHostRequestKindV1::SetGrabMode => {
            let Some(grab_mode) = request.grab_mode else {
                write_warn("runtime_cursor", "runtime_cursor_grab_mode_missing");
                return Ok(());
            };
            apply_grab_mode(window, grab_mode)
        }
        ZrRuntimeCursorHostRequestKindV1::SetHitTest => window
            .set_cursor_hittest(request.value)
            .map_err(|error| format!("runtime_cursor_hit_test_failed:{error}")),
        ZrRuntimeCursorHostRequestKindV1::SetPosition => {
            let Some(position) = request.position else {
                write_warn("runtime_cursor", "runtime_cursor_position_missing");
                return Ok(());
            };
            if !position.x.is_finite() || !position.y.is_finite() {
                write_warn("runtime_cursor", "runtime_cursor_position_non_finite");
                return Ok(());
            }
            window
                .set_cursor_position(
                    LogicalPosition::new(position.x as f64, position.y as f64).into(),
                )
                .map_err(|error| format!("runtime_cursor_position_failed:{error}"))
        }
    }
}

fn apply_grab_mode(
    window: &dyn Window,
    grab_mode: ZrRuntimeCursorGrabModeV1,
) -> Result<(), String> {
    match grab_mode {
        ZrRuntimeCursorGrabModeV1::None => window
            .set_cursor_grab(CursorGrabMode::None)
            .map_err(|error| format!("runtime_cursor_ungrab_failed:{error}")),
        ZrRuntimeCursorGrabModeV1::Confined => window
            .set_cursor_grab(CursorGrabMode::Confined)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked))
            .map_err(|error| cursor_grab_error("confined", error)),
        ZrRuntimeCursorGrabModeV1::Locked => window
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
            .map_err(|error| cursor_grab_error("locked", error)),
    }
}

fn cursor_grab_error(mode: &str, error: impl std::fmt::Display) -> String {
    format!("runtime_cursor_grab_{mode}_failed:{error}")
}
