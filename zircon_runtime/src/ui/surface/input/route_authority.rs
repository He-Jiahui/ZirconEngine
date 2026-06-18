use zircon_runtime_interface::ui::dispatch::{UiInputDispatchResult, UiInputRoutePolicy};

use crate::ui::dispatch::{UiInputRouteStage, UI_INPUT_ROUTE_ORDER};

pub(super) const UI_INPUT_ROUTE_AUTHORITY_ANCHOR: &str = "runtime_09_m1_1_ui_input_route_authority";
pub(super) const UI_INPUT_ROUTE_AUTHORITY_NOTE_PREFIX: &str = "route_authority=";

pub(super) fn annotate_authoritative_input_dispatch(result: &mut UiInputDispatchResult) {
    result
        .diagnostics
        .notes
        .retain(|note| !note.starts_with(UI_INPUT_ROUTE_AUTHORITY_NOTE_PREFIX));

    let stage_names = route_authority_stage_names_for_policy(result.diagnostics.route_policy);
    result.diagnostics.notes.push(format!(
        "{UI_INPUT_ROUTE_AUTHORITY_NOTE_PREFIX}{UI_INPUT_ROUTE_AUTHORITY_ANCHOR};policy={};stages={}",
        result.diagnostics.route_policy.as_str(),
        stage_names.join(">")
    ));
}

pub(super) fn route_authority_stage_names_for_policy(
    policy: UiInputRoutePolicy,
) -> Vec<&'static str> {
    UI_INPUT_ROUTE_ORDER
        .iter()
        .copied()
        .filter(|stage| route_policy_uses_stage(policy, *stage))
        .map(route_stage_name)
        .collect()
}

fn route_policy_uses_stage(policy: UiInputRoutePolicy, stage: UiInputRouteStage) -> bool {
    match policy {
        UiInputRoutePolicy::Unrouted => false,
        UiInputRoutePolicy::PreviewTunnel => matches!(
            stage,
            UiInputRouteStage::PopupStack | UiInputRouteStage::PreviewTunnel
        ),
        UiInputRoutePolicy::Bubble => matches!(
            stage,
            UiInputRouteStage::PopupStack
                | UiInputRouteStage::PreviewTunnel
                | UiInputRouteStage::DirectTarget
                | UiInputRouteStage::BubblePath
        ),
        UiInputRoutePolicy::Direct => matches!(stage, UiInputRouteStage::DirectTarget),
        UiInputRoutePolicy::FocusPath => matches!(
            stage,
            UiInputRouteStage::PopupStack | UiInputRouteStage::FocusPath
        ),
        UiInputRoutePolicy::PointerCapture => {
            matches!(stage, UiInputRouteStage::PointerCapture)
        }
        UiInputRoutePolicy::DefaultAction => matches!(
            stage,
            UiInputRouteStage::PopupStack | UiInputRouteStage::DefaultAction
        ),
    }
}

fn route_stage_name(stage: UiInputRouteStage) -> &'static str {
    match stage {
        UiInputRouteStage::PointerCapture => "pointer_capture",
        UiInputRouteStage::PopupStack => "popup_stack",
        UiInputRouteStage::PreviewTunnel => "preview_tunnel",
        UiInputRouteStage::DirectTarget => "direct_target",
        UiInputRouteStage::BubblePath => "bubble_path",
        UiInputRouteStage::FocusPath => "focus_path",
        UiInputRouteStage::DefaultAction => "default_action",
    }
}
