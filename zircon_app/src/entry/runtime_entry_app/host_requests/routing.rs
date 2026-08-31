use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::write_warn;
use zircon_runtime_interface::{
    ZrRuntimeHostRequestV1, ZrRuntimeImeHostRequestV1, ZrRuntimeViewportHandle,
    ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
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
            let Some(window) = app.window.as_ref() else {
                return;
            };
            apply_runtime_ime_host_request(window.as_ref(), request)
                .map_err(|error| error.to_string())
        }
        ZrRuntimeHostRequestV1::GamepadRumble(request) => app
            .apply_runtime_gamepad_rumble_request(request)
            .map_err(str::to_string),
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
mod tests {
    use zircon_runtime_interface::{
        ZrRuntimeImeHostRequestV1, ZrRuntimeViewportHandle,
        ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
    };

    use super::ime_request_targets_viewport;

    #[test]
    fn ime_request_rejects_a_different_viewport() {
        let request = ZrRuntimeImeHostRequestV1::enable()
            .with_target_viewport(ZrRuntimeViewportHandle::new(7));

        assert!(ime_request_targets_viewport(
            &request,
            ZrRuntimeViewportHandle::new(7)
        ));
        assert!(!ime_request_targets_viewport(
            &request,
            ZrRuntimeViewportHandle::new(8)
        ));
    }

    #[test]
    fn legacy_ime_request_without_a_target_only_targets_the_default_viewport() {
        assert!(ime_request_targets_viewport(
            &ZrRuntimeImeHostRequestV1::enable(),
            ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1
        ));
        assert!(!ime_request_targets_viewport(
            &ZrRuntimeImeHostRequestV1::enable(),
            ZrRuntimeViewportHandle::new(7)
        ));
    }
}
