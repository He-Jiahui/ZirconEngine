use std::collections::BTreeMap;

use zircon_runtime::core::framework::picking::{
    hovered_hits_for_pointer, resolve_picking_outputs, HitData, HitRecord, PickingDebugFeed,
    PickingHoverMap, PickingPipelineReport, PointerHits, PointerId,
};
use zircon_runtime_interface::math::Vec2;
use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiPoint};

use crate::scene::viewport::pointer::{
    precision::{CandidateScore, PrecisionCandidate},
    viewport_pointer_route::ViewportPointerRoute,
};

const EDITOR_VIEWPORT_POINTER_ID: PointerId = PointerId::new(1);
const EDITOR_VIEWPORT_CAMERA_ID: u64 = 0;
const SCORE_TOLERANCE_PX: f32 = 0.5;
const DEPTH_TIE_BREAKER_SCALE: f32 = 0.000_001;
const NON_FINITE_DEPTH: f32 = 1.0e30;

pub(in crate::scene::viewport::pointer) fn resolve_runtime_route(
    candidates: &BTreeMap<UiNodeId, PrecisionCandidate>,
    stacked: &[UiNodeId],
    point: UiPoint,
) -> Option<ViewportPointerRoute> {
    let outputs = runtime_pointer_hits_for_candidates(candidates, stacked, point);
    route_from_outputs(&outputs)
}

pub(in crate::scene::viewport::pointer) fn resolve_runtime_route_for_candidates(
    candidates: &[PrecisionCandidate],
    point: UiPoint,
) -> Option<ViewportPointerRoute> {
    let outputs = runtime_pointer_hits_for_candidates_with_renderer_candidates(
        &BTreeMap::new(),
        &[],
        point,
        candidates,
    );
    route_from_outputs(&outputs)
}

pub(in crate::scene::viewport::pointer) fn resolve_runtime_route_and_debug_feed(
    candidates: &BTreeMap<UiNodeId, PrecisionCandidate>,
    stacked: &[UiNodeId],
    point: UiPoint,
) -> (Option<ViewportPointerRoute>, PickingDebugFeed) {
    resolve_runtime_route_and_debug_feed_with_renderer_candidates(candidates, stacked, point, &[])
}

pub(in crate::scene::viewport::pointer) fn resolve_runtime_route_and_debug_feed_with_renderer_candidates(
    candidates: &BTreeMap<UiNodeId, PrecisionCandidate>,
    stacked: &[UiNodeId],
    point: UiPoint,
    renderer_candidates: &[PrecisionCandidate],
) -> (Option<ViewportPointerRoute>, PickingDebugFeed) {
    let outputs = runtime_pointer_hits_for_candidates_with_renderer_candidates(
        candidates,
        stacked,
        point,
        renderer_candidates,
    );
    let (hover_map, report) = resolve_picking_outputs(&outputs);
    let route = route_from_hover_map(&hover_map);
    let debug_feed = PickingDebugFeed::from_report(&report);
    (route, debug_feed)
}

fn route_from_outputs(outputs: &[PointerHits]) -> Option<ViewportPointerRoute> {
    let target = hovered_hits_for_pointer(outputs, EDITOR_VIEWPORT_POINTER_ID)
        .first()
        .map(|hit| hit.target)?;
    Some(ViewportPointerRoute::from_target(target))
}

fn route_from_hover_map(hover_map: &PickingHoverMap) -> Option<ViewportPointerRoute> {
    let target = hover_map.get(EDITOR_VIEWPORT_POINTER_ID).first()?.target;
    Some(ViewportPointerRoute::from_target(target))
}

pub(in crate::scene::viewport::pointer) fn runtime_pointer_hits_for_candidates(
    candidates: &BTreeMap<UiNodeId, PrecisionCandidate>,
    stacked: &[UiNodeId],
    point: UiPoint,
) -> Vec<PointerHits> {
    runtime_pointer_hits_for_candidates_with_renderer_candidates(candidates, stacked, point, &[])
}

fn runtime_pointer_hits_for_candidates_with_renderer_candidates(
    candidates: &BTreeMap<UiNodeId, PrecisionCandidate>,
    stacked: &[UiNodeId],
    point: UiPoint,
    renderer_candidates: &[PrecisionCandidate],
) -> Vec<PointerHits> {
    let cursor = Vec2::new(point.x, point.y);
    let mut hits = stacked
        .iter()
        .filter_map(|node_id| {
            let candidate = candidates.get(node_id)?;
            let score = candidate.score(cursor)?;
            Some(runtime_hit_record(candidate, score))
        })
        .collect::<Vec<_>>();
    hits.extend(renderer_candidates.iter().filter_map(|candidate| {
        let score = candidate.score(cursor)?;
        Some(runtime_hit_record(candidate, score))
    }));
    if hits.is_empty() {
        Vec::new()
    } else {
        vec![PointerHits::new(EDITOR_VIEWPORT_POINTER_ID, hits, 0.0)]
    }
}

pub(in crate::scene::viewport::pointer) fn runtime_debug_feed_for_candidates(
    candidates: &BTreeMap<UiNodeId, PrecisionCandidate>,
    stacked: &[UiNodeId],
    point: UiPoint,
) -> PickingDebugFeed {
    let outputs = runtime_pointer_hits_for_candidates(candidates, stacked, point);
    PickingDebugFeed::from_report(&PickingPipelineReport::from_outputs(&outputs))
}

fn runtime_hit_record(candidate: &PrecisionCandidate, score: CandidateScore) -> HitRecord {
    HitRecord::new(
        candidate.route.target(),
        HitData::new(
            EDITOR_VIEWPORT_CAMERA_ID,
            runtime_depth_for_score(score),
            None,
            None,
        ),
    )
}

fn runtime_depth_for_score(score: CandidateScore) -> f32 {
    // Runtime picking sorts by target priority and then depth; quantizing the
    // screen-space score preserves the editor's half-pixel tolerance before
    // using projected depth as a tie-breaker within the same target class.
    let score_bucket = finite_or_large(score.score.max(0.0)) / SCORE_TOLERANCE_PX;
    score_bucket.floor() * SCORE_TOLERANCE_PX
        + finite_or_large(score.depth.max(0.0)) * DEPTH_TIE_BREAKER_SCALE
}

fn finite_or_large(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        NON_FINITE_DEPTH
    }
}
