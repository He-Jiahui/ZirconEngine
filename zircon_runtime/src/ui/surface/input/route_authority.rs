use zircon_runtime_interface::ui::dispatch::{UiInputDispatchResult, UiInputRoutePolicy};

use crate::ui::dispatch::route_stage_names_for_policy;

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
    route_stage_names_for_policy(policy)
}
