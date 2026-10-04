use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::write_warn;
use zircon_runtime_interface::{
    ZrRuntimeGamepadRumbleRequestV1, ZrRuntimeHostRequestV1, ZrRuntimeImeHostRequestV1,
    ZrRuntimeViewportHandle, ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
};

use super::super::RuntimeEntryApp;
use super::clipboard::apply_runtime_clipboard_host_request;
use super::cursor::apply_runtime_cursor_host_request;
use super::ime::apply_runtime_ime_host_request;
use super::ui_action::report_unhandled_runtime_ui_action;
use super::ui_host_request::report_unhandled_runtime_ui_host_request;

pub(super) fn apply_runtime_host_request(
    app: &mut RuntimeEntryApp,
    event_loop: &dyn ActiveEventLoop,
    request: ZrRuntimeHostRequestV1,
) {
    let result = match request {
        ZrRuntimeHostRequestV1::Ime(request) => {
            if !ime_request_targets_viewport(&request, app.viewport) {
                write_warn(
                    "runtime_ime",
                    format!(
                        "runtime_ime_target_viewport_rejected target={:?} host={:?}",
                        request.target_viewport, app.viewport
                    ),
                );
                return;
            }
            let native_area = if app.native_ime_composition_requested
                && request.kind
                    == zircon_runtime_interface::ZrRuntimeImeHostRequestKindV1::SetCursorArea
            {
                let Some(area) = request.cursor_area else {
                    write_warn("runtime_ime", "runtime_ime_native_v2_cursor_area_missing");
                    return;
                };
                Some(area)
            } else {
                None
            };
            let native_candidate = if let Some(area) = native_area {
                match app.validate_native_ime_candidate(area) {
                    Ok(candidate) => Some(candidate),
                    Err(error) => {
                        app.reject_native_ime_host_publication();
                        write_warn(
                            "runtime_ime",
                            format!("runtime_ime_native_v2_rejected:{error}"),
                        );
                        return;
                    }
                }
            } else {
                None
            };
            let Some(window) = app.window.clone() else {
                if native_candidate.is_some() {
                    app.reject_native_ime_host_publication();
                }
                return;
            };
            // Validate the caret topology without mutating the runtime, then submit the real
            // Winit request. Producer/session admission follows only an accepted host request;
            // this prevents a failed platform packet from leaving a V2 producer live.
            match apply_runtime_ime_host_request(window.as_ref(), request, native_candidate) {
                Err(error) => {
                    if native_candidate.is_some() {
                        app.reject_native_ime_host_publication();
                    }
                    Err(error.to_string())
                }
                Ok(()) => {
                    if let Some(candidate) = native_candidate {
                        app.record_native_ime_host_publication(candidate);
                        match app.prepare_native_ime_candidate(
                            native_area.expect("native candidate has a cursor area"),
                        ) {
                            Ok(_) => Ok(()),
                            Err(error) => Err(error.to_string()),
                        }
                    } else {
                        Ok(())
                    }
                }
            }
        }
        ZrRuntimeHostRequestV1::GamepadRumble(request) => {
            apply_runtime_gamepad_rumble_request(app, request).map_err(str::to_string)
        }
        ZrRuntimeHostRequestV1::Cursor(request) => {
            let Some(window) = app.window.as_ref() else {
                return;
            };
            apply_runtime_cursor_host_request(window.as_ref(), request)
        }
        ZrRuntimeHostRequestV1::Clipboard(request) => {
            apply_runtime_clipboard_host_request(app, event_loop, request)
        }
        ZrRuntimeHostRequestV1::UiAction(request) => {
            if request.target_viewport != app.viewport {
                write_warn(
                    "runtime_ui_action",
                    format!(
                        "runtime_ui_action_target_viewport_rejected target={:?} host={:?}",
                        request.target_viewport, app.viewport
                    ),
                );
                return;
            }
            report_unhandled_runtime_ui_action(app, request);
            Ok(())
        }
        ZrRuntimeHostRequestV1::UiHost(request) => {
            if request.target_viewport != app.viewport {
                write_warn(
                    "runtime_ui_host_request",
                    format!(
                        "runtime_ui_host_request_target_viewport_rejected target={:?} host={:?}",
                        request.target_viewport, app.viewport
                    ),
                );
                return;
            }
            report_unhandled_runtime_ui_host_request(app, request);
            Ok(())
        }
    };
    if let Err(error) = result {
        write_warn(
            "runtime_host_request",
            format!("runtime_host_request_failed:{error}"),
        );
    }
}

fn apply_runtime_gamepad_rumble_request(
    app: &mut RuntimeEntryApp,
    request: ZrRuntimeGamepadRumbleRequestV1,
) -> Result<(), &'static str> {
    #[cfg(feature = "gamepad-gilrs")]
    {
        app.apply_runtime_gamepad_rumble_request(request)
    }
    #[cfg(not(feature = "gamepad-gilrs"))]
    {
        let _ = (app, request);
        Err("runtime_gamepad_rumble_feature_disabled")
    }
}

fn ime_request_targets_viewport(
    request: &ZrRuntimeImeHostRequestV1,
    viewport: ZrRuntimeViewportHandle,
) -> bool {
    match request.target_viewport {
        Some(target) => target == viewport,
        None => viewport == ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
    }
}

#[cfg(test)]
#[path = "tests/routing.rs"]
mod tests;
