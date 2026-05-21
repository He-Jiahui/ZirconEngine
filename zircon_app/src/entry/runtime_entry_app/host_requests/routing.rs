use winit::window::Window;
use zircon_runtime::diagnostic_log::write_warn;
use zircon_runtime_interface::ZrRuntimeHostRequestV1;

use super::ime::apply_runtime_ime_host_request;

pub(super) fn apply_runtime_host_request(window: &dyn Window, request: ZrRuntimeHostRequestV1) {
    let result = match request {
        ZrRuntimeHostRequestV1::Ime(request) => apply_runtime_ime_host_request(window, request),
    };
    if let Err(error) = result {
        write_warn(
            "runtime_ime",
            format!("runtime_ime_host_request_failed:{error}"),
        );
    }
}
