use zircon_runtime_interface::ui::{
    dispatch::{
        UiClipboardRequest, UiClipboardRequestKind, UiDispatchEffect, UiInputMethodRequest,
        UiInputMethodRequestKind,
    },
    event_ui::UiNodeId,
};

use super::super::super::surface::UiSurface;
use super::super::require_valid_input_owner;

pub(super) fn apply_text_service_effect(
    surface: &mut UiSurface,
    effect: &UiDispatchEffect,
) -> Result<Option<UiNodeId>, String> {
    match effect {
        UiDispatchEffect::RequestInputMethod { request } => {
            apply_input_method_request(surface, request)
        }
        UiDispatchEffect::RequestClipboard { request } => apply_clipboard_request(surface, request),
        _ => Err("expected input method or clipboard effect".to_string()),
    }
}

fn apply_input_method_request(
    surface: &mut UiSurface,
    request: &UiInputMethodRequest,
) -> Result<Option<UiNodeId>, String> {
    require_valid_input_owner(surface, request.owner)?;
    if let Some(surrounding_text) = &request.surrounding_text {
        surrounding_text
            .validate()
            .map_err(|error| format!("invalid input method surrounding text: {error}"))?;
    }
    match request.kind {
        UiInputMethodRequestKind::Enable => {
            surface.input.input_method_owner = Some(request.owner);
            surface.input.input_method_request = Some(request.clone());
        }
        UiInputMethodRequestKind::Reset | UiInputMethodRequestKind::UpdateCursor => {
            if surface.input.input_method_owner == Some(request.owner) {
                surface.input.input_method_request = Some(request.clone());
            } else {
                return Err("input method owner mismatch".to_string());
            }
        }
        UiInputMethodRequestKind::Disable => {
            if surface.input.input_method_owner == Some(request.owner) {
                surface.input.clear_input_method();
            } else {
                return Err("input method owner mismatch".to_string());
            }
        }
    }
    Ok(Some(request.owner))
}

fn apply_clipboard_request(
    surface: &UiSurface,
    request: &UiClipboardRequest,
) -> Result<Option<UiNodeId>, String> {
    require_valid_input_owner(surface, request.owner)?;
    match request.kind {
        UiClipboardRequestKind::ReadText if request.text.is_some() => {
            Err("clipboard read request cannot carry text".to_string())
        }
        UiClipboardRequestKind::WriteText if request.text.is_none() => {
            Err("clipboard write request missing text".to_string())
        }
        UiClipboardRequestKind::ReadText | UiClipboardRequestKind::WriteText => {
            Ok(Some(request.owner))
        }
    }
}
