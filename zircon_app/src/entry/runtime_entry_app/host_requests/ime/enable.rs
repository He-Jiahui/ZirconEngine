use winit::dpi::{LogicalPosition, LogicalSize};
use winit::window::{
    ImeCapabilities, ImeEnableRequest, ImeHint, ImePurpose, ImeRequest, ImeRequestData,
    ImeRequestError, Window,
};
use zircon_runtime::diagnostic_log::write_warn;

use super::surrounding_text::default_ime_surrounding_text;

pub(super) fn enable_window_ime(window: &dyn Window) -> Result<(), ImeRequestError> {
    apply_ime_enable_with_request_data(default_ime_request_data(), |request| {
        window.request_ime_update(request)
    })
}

fn apply_ime_enable_with_request_data(
    request_data: Option<ImeRequestData>,
    request_ime_update: impl FnOnce(ImeRequest) -> Result<(), ImeRequestError>,
) -> Result<(), ImeRequestError> {
    let capabilities = ImeCapabilities::new()
        .with_hint_and_purpose()
        .with_cursor_area()
        .with_surrounding_text();
    let Some(request_data) = request_data else {
        write_warn("runtime_ime", "runtime_ime_default_request_data_invalid");
        return Ok(());
    };
    let Some(request) = ImeEnableRequest::new(capabilities, request_data) else {
        write_warn("runtime_ime", "runtime_ime_enable_request_invalid");
        return Ok(());
    };
    match request_ime_update(ImeRequest::Enable(request)) {
        Err(ImeRequestError::AlreadyEnabled) => Ok(()),
        result => result,
    }
}

fn default_ime_request_data() -> Option<ImeRequestData> {
    Some(
        ImeRequestData::default()
            .with_hint_and_purpose(ImeHint::NONE, ImePurpose::Normal)
            .with_cursor_area(
                LogicalPosition::new(0.0, 0.0).into(),
                LogicalSize::new(1.0, 1.0).into(),
            )
            .with_surrounding_text(default_ime_surrounding_text()?),
    )
}

#[cfg(test)]
mod tests {
    use super::{apply_ime_enable_with_request_data, default_ime_request_data};

    #[test]
    fn default_request_data_is_available_without_a_panic_contract() {
        assert!(default_ime_request_data().is_some());
    }

    #[test]
    fn invalid_default_request_data_skips_window_enable_submission() {
        let mut submitted = false;

        let result = apply_ime_enable_with_request_data(None, |_| {
            submitted = true;
            Ok(())
        });

        assert!(result.is_ok());
        assert!(!submitted);
    }
}
