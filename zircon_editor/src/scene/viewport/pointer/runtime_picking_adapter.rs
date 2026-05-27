use std::collections::BTreeMap;

use zircon_runtime::core::framework::{
    picking::{
        hovered_hits_for_pointer, HitData, HitRecord, PickingDebugFeed, PickingPipelineReport,
        PointerAction, PointerButton, PointerHits, PointerId, PointerInput, PointerLocation,
        PointerScrollUnit,
    },
    render::RenderViewportHandle,
};
use zircon_runtime_interface::math::Vec2;
use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent,
    event_ui::UiNodeId,
    layout::UiPoint,
    surface::{UiPointerButton, UiPointerEventKind},
};

use crate::scene::viewport::pointer::{
    precision::{CandidateScore, PrecisionCandidate},
    viewport_pointer_route::ViewportPointerRoute,
};

const EDITOR_VIEWPORT_POINTER_ID: PointerId = PointerId::new(1);
const EDITOR_VIEWPORT_HANDLE: RenderViewportHandle = RenderViewportHandle::new(0);
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
    let target = hovered_hits_for_pointer(&outputs, EDITOR_VIEWPORT_POINTER_ID)
        .first()
        .map(|hit| hit.target)?;
    Some(ViewportPointerRoute::from_target(target))
}

pub(in crate::scene::viewport::pointer) fn runtime_pointer_hits_for_candidates(
    candidates: &BTreeMap<UiNodeId, PrecisionCandidate>,
    stacked: &[UiNodeId],
    point: UiPoint,
) -> Vec<PointerHits> {
    let cursor = Vec2::new(point.x, point.y);
    let hits = stacked
        .iter()
        .filter_map(|node_id| {
            let candidate = candidates.get(node_id)?;
            let score = candidate.score(cursor)?;
            Some(runtime_hit_record(candidate, score))
        })
        .collect::<Vec<_>>();
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

pub(in crate::scene::viewport::pointer) fn runtime_pointer_input_for_event(
    event: &UiPointerEvent,
) -> PointerInput {
    let location = PointerLocation::new(
        EDITOR_VIEWPORT_POINTER_ID,
        EDITOR_VIEWPORT_HANDLE,
        Vec2::new(event.point.x, event.point.y),
    );
    PointerInput::new(location, runtime_pointer_action_for_event(event))
}

fn runtime_pointer_action_for_event(event: &UiPointerEvent) -> PointerAction {
    match event.kind {
        UiPointerEventKind::Down => PointerAction::Press(runtime_button(event.button)),
        UiPointerEventKind::Up => PointerAction::Release(runtime_button(event.button)),
        UiPointerEventKind::Move => {
            // UI pointer events currently provide the absolute cursor only; the
            // runtime bridge preserves the location and leaves delta synthesis
            // for a later stateful input collector.
            PointerAction::Move { delta: Vec2::ZERO }
        }
        UiPointerEventKind::Scroll => PointerAction::Scroll {
            unit: PointerScrollUnit::Pixel,
            delta: Vec2::new(0.0, event.scroll_delta),
        },
    }
}

fn runtime_button(button: Option<UiPointerButton>) -> PointerButton {
    match button.unwrap_or(UiPointerButton::Primary) {
        UiPointerButton::Primary => PointerButton::Primary,
        UiPointerButton::Secondary => PointerButton::Secondary,
        UiPointerButton::Middle => PointerButton::Middle,
    }
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
