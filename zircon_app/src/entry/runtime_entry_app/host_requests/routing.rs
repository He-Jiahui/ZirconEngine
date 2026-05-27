use zircon_runtime::diagnostic_log::write_warn;
use zircon_runtime_interface::ZrRuntimeHostRequestV1;

use super::super::RuntimeEntryApp;
use super::ime::apply_runtime_ime_host_request;

pub(super) fn apply_runtime_host_request(
    app: &mut RuntimeEntryApp,
    request: ZrRuntimeHostRequestV1,
) {
    #[cfg(feature = "gamepad-gilrs")]
    super::super::gamepad::clear_finished_rumble_effects(app.gamepad_rumble_effects.as_mut());
    let result = match request {
        ZrRuntimeHostRequestV1::Ime(request) => {
            let Some(window) = app.window.as_ref() else {
                return;
            };
            apply_runtime_ime_host_request(window.as_ref(), request)
                .map_err(|error| error.to_string())
        }
        ZrRuntimeHostRequestV1::GamepadRumble(request) => app
            .apply_runtime_gamepad_rumble_request(request)
            .map_err(str::to_string),
    };
    if let Err(error) = result {
        write_warn(
            "runtime_host_request",
            format!("runtime_host_request_failed:{error}"),
        );
    }
}
