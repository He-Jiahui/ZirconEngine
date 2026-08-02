use zircon_runtime_interface::ui::{
    dispatch::{
        UiClipboardRequest, UiClipboardRequestKind, UiDispatchEffect, UiInputMethodRequest,
        UiInputMethodRequestKind,
    },
    event_ui::UiNodeId,
};

use super::super::super::surface::UiSurface;
use super::super::{
    require_valid_input_owner, text_state::editable_text_input_is_secure,
    UiSurfaceInputEffectError, UiSurfaceInputEffectResult,
};

pub(super) fn apply_text_service_effect(
    surface: &mut UiSurface,
    effect: &UiDispatchEffect,
) -> UiSurfaceInputEffectResult<Option<UiNodeId>> {
    match effect {
        UiDispatchEffect::RequestInputMethod { request } => {
            apply_input_method_request(surface, request)
        }
        UiDispatchEffect::RequestClipboard { request } => apply_clipboard_request(surface, request),
        _ => Err(UiSurfaceInputEffectError::UnexpectedEffect {
            expected: "input method or clipboard",
        }),
    }
}

fn apply_input_method_request(
    surface: &mut UiSurface,
    request: &UiInputMethodRequest,
) -> UiSurfaceInputEffectResult<Option<UiNodeId>> {
    require_valid_input_owner(surface, request.owner)?;
    if let Some(surrounding_text) = &request.surrounding_text {
        surrounding_text.validate().map_err(|validation_error| {
            UiSurfaceInputEffectError::InvalidInputMethodSurroundingText { validation_error }
        })?;
    }
    match request.kind {
        UiInputMethodRequestKind::Enable => {
            if editable_text_input_is_secure(surface, request.owner) {
                return Err(UiSurfaceInputEffectError::InputMethodDisabledForSecureTextInput);
            }
            surface.input.input_method_owner = Some(request.owner);
            surface.input.input_method_request = Some(request.clone());
        }
        UiInputMethodRequestKind::Reset | UiInputMethodRequestKind::UpdateCursor => {
            if surface.input.input_method_owner == Some(request.owner) {
                surface.input.input_method_request = Some(request.clone());
            } else {
                return Err(UiSurfaceInputEffectError::InputMethodOwnerMismatch);
            }
        }
        UiInputMethodRequestKind::Disable => {
            if surface.input.input_method_owner == Some(request.owner) {
                surface.input.clear_input_method();
            } else {
                return Err(UiSurfaceInputEffectError::InputMethodOwnerMismatch);
            }
        }
    }
    Ok(Some(request.owner))
}

fn apply_clipboard_request(
    surface: &UiSurface,
    request: &UiClipboardRequest,
) -> UiSurfaceInputEffectResult<Option<UiNodeId>> {
    require_valid_input_owner(surface, request.owner)?;
    match request.kind {
        UiClipboardRequestKind::ReadText if request.text.is_some() => {
            Err(UiSurfaceInputEffectError::ClipboardReadRequestCarriesText)
        }
        UiClipboardRequestKind::WriteText if request.text.is_none() => {
            Err(UiSurfaceInputEffectError::ClipboardWriteRequestMissingText)
        }
        UiClipboardRequestKind::ReadText | UiClipboardRequestKind::WriteText => {
            Ok(Some(request.owner))
        }
    }
}
