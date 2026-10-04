use zircon_runtime_interface::ui::dispatch::{UiInputDispatchResult, UiInputRoutePolicy};

use crate::ui::dispatch::route_stage_names_for_policy;

pub(super) const UI_INPUT_ROUTE_AUTHORITY_ANCHOR: &str = "runtime_09_m1_1_ui_input_route_authority";
pub(super) const UI_INPUT_ROUTE_AUTHORITY_NOTE_PREFIX: &str = "route_authority=";

/// 让最终诊断保留唯一的当前路由权威说明；不同输入 owner 补注后再统一覆盖旧标签。
pub(super) fn annotate_authoritative_input_dispatch(result: &mut UiInputDispatchResult) {
    let policy = result.diagnostics.route_policy;
    replace_route_authority_note(
        &mut result.diagnostics.notes,
        route_authority_note_for_policy(policy),
    );
}

fn replace_route_authority_note(notes: &mut Vec<String>, authoritative_note: &str) {
    if let Some(last) = notes.last_mut() {
        if last.starts_with(UI_INPUT_ROUTE_AUTHORITY_NOTE_PREFIX) {
            last.clear();
            last.push_str(authoritative_note);
            return;
        }
    }

    notes.retain(|note| !note.starts_with(UI_INPUT_ROUTE_AUTHORITY_NOTE_PREFIX));
    notes.push(authoritative_note.to_string());
}

const fn route_authority_note_for_policy(policy: UiInputRoutePolicy) -> &'static str {
    match policy {
        UiInputRoutePolicy::Unrouted => {
            "route_authority=runtime_09_m1_1_ui_input_route_authority;policy=unrouted;stages="
        }
        UiInputRoutePolicy::PreviewTunnel => {
            "route_authority=runtime_09_m1_1_ui_input_route_authority;policy=preview_tunnel;stages=popup_stack>preview_tunnel"
        }
        UiInputRoutePolicy::Bubble => {
            "route_authority=runtime_09_m1_1_ui_input_route_authority;policy=bubble;stages=popup_stack>preview_tunnel>direct_target>bubble_path"
        }
        UiInputRoutePolicy::Direct => {
            "route_authority=runtime_09_m1_1_ui_input_route_authority;policy=direct;stages=direct_target"
        }
        UiInputRoutePolicy::FocusPath => {
            "route_authority=runtime_09_m1_1_ui_input_route_authority;policy=focus_path;stages=popup_stack>focus_path"
        }
        UiInputRoutePolicy::PointerCapture => {
            "route_authority=runtime_09_m1_1_ui_input_route_authority;policy=pointer_capture;stages=pointer_capture"
        }
        UiInputRoutePolicy::DefaultAction => {
            "route_authority=runtime_09_m1_1_ui_input_route_authority;policy=default_action;stages=popup_stack>default_action"
        }
    }
}

pub(super) fn route_authority_stage_names_for_policy(
    policy: UiInputRoutePolicy,
) -> Vec<&'static str> {
    route_stage_names_for_policy(policy)
}

#[cfg(test)]
#[path = "tests/route_authority.rs"]
mod tests;
