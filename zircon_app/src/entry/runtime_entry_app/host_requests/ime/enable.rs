use winit::dpi::{LogicalPosition, LogicalSize};
use winit::window::{
    ImeCapabilities, ImeEnableRequest, ImeHint, ImePurpose, ImeRequest, ImeRequestData,
    ImeRequestError, Window,
};
use zircon_runtime::diagnostic_log::write_warn;

use super::surrounding_text::default_ime_surrounding_text;

pub(super) fn enable_window_ime(window: &dyn Window) -> Result<(), ImeRequestError> {
    let capabilities = ImeCapabilities::new()
        .with_hint_and_purpose()
        .with_cursor_area()
        .with_surrounding_text();
    let Some(request) = ImeEnableRequest::new(capabilities, default_ime_request_data()) else {
        write_warn("runtime_ime", "runtime_ime_enable_request_invalid");
        return Ok(());
    };
    match window.request_ime_update(ImeRequest::Enable(request)) {
        Err(ImeRequestError::AlreadyEnabled) => Ok(()),
        result => result,
    }
}

fn default_ime_request_data() -> ImeRequestData {
    ImeRequestData::default()
        .with_hint_and_purpose(ImeHint::NONE, ImePurpose::Normal)
        .with_cursor_area(
            LogicalPosition::new(0.0, 0.0).into(),
            LogicalSize::new(1.0, 1.0).into(),
        )
        .with_surrounding_text(default_ime_surrounding_text())
}
