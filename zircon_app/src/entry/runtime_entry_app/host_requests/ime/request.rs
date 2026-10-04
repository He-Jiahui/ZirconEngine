//! 按已授权的目标窗口消费动态 Runtime 的输入法控制请求。
//! 非法几何或文本在窗口 API 前被丢弃，避免把错误状态交给平台后端。

use winit::window::{ImeRequest, ImeRequestData, ImeRequestError, Window};
use zircon_runtime::diagnostic_log::write_warn;
use zircon_runtime_interface::{ZrRuntimeImeHostRequestKindV1, ZrRuntimeImeHostRequestV1};

use super::enable::enable_window_ime;
use super::geometry::{ime_candidate_rect_to_winit, ime_logical_cursor_area};
use super::surrounding_text::runtime_ime_surrounding_text;

pub(in crate::entry::runtime_entry_app) fn apply_runtime_ime_host_request(
    window: &dyn Window,
    request: ZrRuntimeImeHostRequestV1,
    native_candidate: Option<zircon_runtime_interface::ZrRuntimeImeCandidateRectV2>,
) -> Result<(), ImeRequestError> {
    // Runtime host requests are drained on the Winit event-loop thread. On Windows the pinned
    // backend applies the request directly on that thread; a successful return is therefore a
    // bounded host-ownership handoff, never a claim that IMM has installed a producer. The app
    // records that handoff first and admits V2 callbacks only after session configuration.
    match request.kind {
        ZrRuntimeImeHostRequestKindV1::Enable => enable_window_ime(window),
        ZrRuntimeImeHostRequestKindV1::Disable => window.request_ime_update(ImeRequest::Disable),
        ZrRuntimeImeHostRequestKindV1::SetCursorArea => {
            if let Some(area) = request.cursor_area {
                let Some((position, size)) = ime_logical_cursor_area(area) else {
                    write_warn("runtime_ime", "runtime_ime_cursor_area_invalid");
                    return Ok(());
                };
                let (position, size) = native_candidate
                    .map(ime_candidate_rect_to_winit)
                    .unwrap_or((position, size));
                window.request_ime_update(ImeRequest::Update(
                    ImeRequestData::default().with_cursor_area(position, size),
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
