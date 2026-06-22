use super::super::super::UiHostWindow;
use crate::ui::retained_host::host_contract::data::HostTextInputFocusData;
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::primitives::SharedString;

use super::redraw::text_input_focus_redraw;

pub(super) fn dispatch_text_focus_value(
    window: &UiHostWindow,
    focus: HostTextInputFocusData,
    target_id: SharedString,
    value: String,
) -> NativePointerDispatchResult {
    let value: SharedString = value.into();
    let control_id = focus.control_id.clone();
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
        "commit_only" if target_id == focus.edit_target_id() => {
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

fn asset_dispatch_source(dispatch_kind: &str) -> Option<&str> {
    if dispatch_kind == "asset" {
        return Some("activity");
    }
    dispatch_kind.strip_prefix("asset:")
}
