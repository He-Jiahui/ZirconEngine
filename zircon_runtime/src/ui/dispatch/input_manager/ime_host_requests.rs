use crate::core::framework::input::{
    ImeCursorArea, ImeCursorRange, ImeHostRequest, ImeSurroundingText,
};
use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchHostRequestKind, UiInputDispatchResult, UiInputMethodRequest,
        UiInputMethodRequestKind, UiInputMethodSurroundingText,
    },
    layout::UiFrame,
};

const MAX_HOST_REQUESTS_PER_INPUT_METHOD_REQUEST: usize = 3;

pub(super) fn append_ime_host_requests_for_result(
    result: &UiInputDispatchResult,
    output: &mut Vec<ImeHostRequest>,
) {
    for host_request in &result.host_requests {
        let UiDispatchHostRequestKind::InputMethod(request) = &host_request.request else {
            continue;
        };
        append_ime_host_requests_for_input_method_request(request, output);
    }
}

pub(super) fn append_ime_host_requests_for_input_method_requests(
    requests: impl IntoIterator<Item = UiInputMethodRequest>,
    output: &mut Vec<ImeHostRequest>,
) {
    let requests = requests.into_iter();
    reserve_ime_host_request_capacity(output, requests.size_hint().0);
    for request in requests {
        append_ime_host_requests_for_input_method_request(&request, output);
    }
}

fn reserve_ime_host_request_capacity(output: &mut Vec<ImeHostRequest>, request_count: usize) {
    output.reserve(request_count.saturating_mul(MAX_HOST_REQUESTS_PER_INPUT_METHOD_REQUEST));
}

fn append_ime_host_requests_for_input_method_request(
    request: &UiInputMethodRequest,
    output: &mut Vec<ImeHostRequest>,
) {
    match request.kind {
        UiInputMethodRequestKind::Enable => output.push(ImeHostRequest::Enable),
        UiInputMethodRequestKind::Disable => {
            output.push(ImeHostRequest::Disable);
            return;
        }
        UiInputMethodRequestKind::Reset | UiInputMethodRequestKind::UpdateCursor => {}
    }

    if let Some(cursor_rect) = request.cursor_rect {
        output.push(ImeHostRequest::SetCursorArea(ime_cursor_area(cursor_rect)));
    }
    if let Some(surrounding_text) = request
        .surrounding_text
        .as_ref()
        .and_then(ime_surrounding_text)
    {
        output.push(ImeHostRequest::SetSurroundingText(surrounding_text));
    }
}

fn ime_cursor_area(frame: UiFrame) -> ImeCursorArea {
    ImeCursorArea::new(frame.x, frame.y, frame.width, frame.height)
}

fn ime_surrounding_text(text: &UiInputMethodSurroundingText) -> Option<ImeSurroundingText> {
    text.validate().ok()?;
    Some(
        ImeSurroundingText::new(
            text.text.clone(),
            text.cursor_byte as usize,
            text.anchor_byte as usize,
        )
        .with_composition_range(
            text.composition_range.map(|range| {
                ImeCursorRange::new(range.start_byte as usize, range.end_byte as usize)
            }),
        ),
    )
}

#[cfg(test)]
#[path = "ime_host_requests/reserve_capacity_tests.rs"]
mod reserve_capacity_tests;
