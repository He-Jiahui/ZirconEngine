use winit::window::{ImeRequest, ImeRequestData, ImeRequestError, Window};
use zircon_runtime::diagnostic_log::write_warn;
use zircon_runtime_interface::{ZrRuntimeImeHostRequestKindV1, ZrRuntimeImeHostRequestV1};

use super::enable::enable_window_ime;
use super::geometry::{ime_logical_position, ime_logical_size};
use super::surrounding_text::runtime_ime_surrounding_text;

pub(in crate::entry::runtime_entry_app) fn apply_runtime_ime_host_request(
    window: &dyn Window,
    request: ZrRuntimeImeHostRequestV1,
) -> Result<(), ImeRequestError> {
    match request.kind {
        ZrRuntimeImeHostRequestKindV1::Enable => enable_window_ime(window),
        ZrRuntimeImeHostRequestKindV1::Disable => window.request_ime_update(ImeRequest::Disable),
        ZrRuntimeImeHostRequestKindV1::SetCursorArea => {
            if let Some(area) = request.cursor_area {
                window.request_ime_update(ImeRequest::Update(
                    ImeRequestData::default()
                        .with_cursor_area(ime_logical_position(area), ime_logical_size(area)),
                ))
            } else {
                write_warn("runtime_ime", "runtime_ime_cursor_area_missing");
                Ok(())
            }
        }
        ZrRuntimeImeHostRequestKindV1::SetSurroundingText => {
            if let Some(text) = request.surrounding_text {
                if let Some(text) = runtime_ime_surrounding_text(text) {
                    window.request_ime_update(ImeRequest::Update(
                        ImeRequestData::default().with_surrounding_text(text),
                    ))
                } else {
                    Ok(())
                }
            } else {
                write_warn("runtime_ime", "runtime_ime_surrounding_text_missing");
                Ok(())
            }
        }
    }
}
