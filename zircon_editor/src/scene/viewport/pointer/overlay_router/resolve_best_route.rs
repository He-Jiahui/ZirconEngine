use std::collections::BTreeMap;

use zircon_runtime_interface::math::Vec2;
use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiPoint};

use crate::scene::viewport::pointer::{
    precision::{CandidateScore, PrecisionCandidate},
    viewport_pointer_route::ViewportPointerRoute,
};

use super::better_score::better_score;

pub(in crate::scene::viewport::pointer) fn resolve_best_route(
    candidates: &BTreeMap<UiNodeId, PrecisionCandidate>,
    stacked: &[UiNodeId],
    point: UiPoint,
) -> Option<ViewportPointerRoute> {
    let cursor = Vec2::new(point.x, point.y);
    let mut best: Option<(&PrecisionCandidate, CandidateScore)> = None;

    for node_id in stacked {
        let Some(candidate) = candidates.get(node_id) else {
            continue;
        };
        let Some(score) = candidate.score(cursor) else {
            continue;
        };
        if better_score(score, best.map(|(_, score)| score)) {
            best = Some((candidate, score));
        }
    }

    best.map(|(candidate, _)| candidate.route.clone())
}
