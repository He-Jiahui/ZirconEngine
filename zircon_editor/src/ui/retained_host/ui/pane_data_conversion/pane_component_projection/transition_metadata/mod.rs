use std::collections::BTreeMap;

use toml::Value;

mod direction;
mod kind;
mod model;
mod state;
mod timing;

pub(super) use self::model::ProjectedTransitionMetadata;

pub(super) fn projected_transition_metadata(
    attributes: &BTreeMap<String, Value>,
    component_role: &str,
    popup_open: bool,
) -> ProjectedTransitionMetadata {
    let kind = kind::projected_transition_kind(attributes, component_role);
    if kind.is_empty() {
        return ProjectedTransitionMetadata::without_transition(kind);
    }

    let transition_in = state::projected_transition_in(attributes, kind.as_str(), popup_open);
    let status = state::projected_transition_status(attributes, transition_in);
    let progress = state::projected_transition_progress(attributes, status.as_str(), transition_in);
    let entered =
        state::projected_transition_entered(attributes, transition_in, status.as_str(), progress);
    let duration_ms =
        timing::projected_transition_duration_ms(attributes, kind.as_str(), transition_in);
    let easing = timing::projected_transition_easing(attributes, kind.as_str(), transition_in);
    let direction = direction::projected_transition_direction(attributes, kind.as_str());

    ProjectedTransitionMetadata {
        kind,
        active: transition_in,
        entered,
        progress,
        duration_ms,
        easing,
        direction,
    }
}
