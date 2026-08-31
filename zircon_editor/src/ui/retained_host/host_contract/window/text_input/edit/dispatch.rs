use super::super::super::UiHostWindow;
use crate::ui::retained_host::asset_control_ids::asset_dispatch_source;
use crate::ui::retained_host::host_contract::data::HostTextInputFocusData;
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::primitives::SharedString;

use super::redraw::text_input_focus_redraw;

pub(super) fn dispatch_text_focus_value(
    window: &UiHostWindow,
    mut focus: HostTextInputFocusData,
    target_id: SharedString,
    value: SharedString,
) -> NativePointerDispatchResult {
    let is_commit_target = target_id.as_str() == text_focus_edit_target_id(&focus);
    let control_id = take_text_focus_control_id(&mut focus);
    let pane_host = window.global::<PaneSurfaceHostContext>();
    match focus.dispatch_kind.as_str() {
        "welcome_text" => pane_host.invoke_welcome_control_changed(target_id, value),
        "showcase" => {
            pane_host.invoke_component_showcase_control_edited(control_id, target_id, value)
        }
        "inspector" => pane_host.invoke_inspector_control_changed(control_id, value),
        kind if asset_dispatch_source(kind).is_some() => pane_host.invoke_asset_control_changed(
            asset_dispatch_source(kind).unwrap_or("activity").into(),
            control_id,
            value,
        ),
        "commit_only" if is_commit_target => {
            return text_input_focus_redraw(&focus);
        }
        "commit_only" => pane_host.invoke_surface_control_edited(control_id, target_id, value),
        _ if !focus.edit_action_id.is_empty() => {
            pane_host.invoke_surface_control_edited(control_id, target_id, value)
        }
        _ => return NativePointerDispatchResult::idle(),
    }
    text_input_focus_redraw(&focus)
}

fn text_focus_edit_target_id(focus: &HostTextInputFocusData) -> &str {
    if !focus.edit_action_id.is_empty() {
        focus.edit_action_id.as_str()
    } else if !focus.action_id.is_empty() {
        focus.action_id.as_str()
    } else {
        focus.control_id.as_str()
    }
}

fn take_text_focus_control_id(focus: &mut HostTextInputFocusData) -> SharedString {
    std::mem::take(&mut focus.control_id)
}

#[cfg(test)]
#[path = "dispatch/owned_control_id_tests.rs"]
mod owned_control_id_tests;
