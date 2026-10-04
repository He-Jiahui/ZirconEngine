use zircon_runtime_interface::ui::dispatch::{
    UiImeInputEventKind, UiImePreeditClause, UiImePreeditClauseKind, UiInputEventMetadata,
    UiTextByteRange,
};
use zircon_runtime_interface::{
    ZrRuntimeImeCandidateRectV2, ZrRuntimeImeCompositionContextV2,
    ZrRuntimeImeCompositionNegotiation, ZrRuntimeImeCompositionOperationV2,
    ZrRuntimeImeCompositionV2, ZrRuntimeImePreeditClauseAvailabilityV2,
    ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY,
};

use super::*;

fn route(window_generation: u64) -> RuntimeImeCompositionRoute {
    RuntimeImeCompositionRoute::new(
        ZrRuntimeImeCompositionNegotiation::from_peer(
            8,
            [ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY],
        ),
        window_generation,
    )
}

fn preedit(window: u64, focus: u64, composition: u64) -> Vec<u8> {
    ZrRuntimeImeCompositionV2 {
        operation: ZrRuntimeImeCompositionOperationV2::Preedit,
        context: ZrRuntimeImeCompositionContextV2 {
            window_generation: window,
            focus_generation: focus,
            composition_generation: composition,
        },
        text: "n你".to_string(),
        cursor_range: Some(UiTextByteRange::new(1, 4)),
        clauses: vec![UiImePreeditClause::new(
            UiTextByteRange::new(1, 4),
            UiImePreeditClauseKind::TargetConverted,
        )],
        clause_availability: ZrRuntimeImePreeditClauseAvailabilityV2::Available,
    }
    .encode()
    .unwrap()
}

#[test]
fn route_preserves_clause_cursor_and_core_commit_cancel_order() {
    let mut route = route(41);
    let preedit = route.consume(&preedit(41, 2, 3), UiInputEventMetadata::default());
    let preedit = preedit.expect("current preedit accepted");
    let UiInputEvent::Ime(ui_preedit) = preedit.ui_event else {
        panic!("preedit must reach the UI IME route");
    };
    assert_eq!(ui_preedit.kind, UiImeInputEventKind::Preedit);
    assert_eq!(ui_preedit.cursor_range, Some(UiTextByteRange::new(1, 4)));
    assert_eq!(ui_preedit.preedit_clauses.len(), 1);

    let commit = ZrRuntimeImeCompositionV2 {
        operation: ZrRuntimeImeCompositionOperationV2::Commit,
        context: ZrRuntimeImeCompositionContextV2 {
            window_generation: 41,
            focus_generation: 2,
            composition_generation: 3,
        },
        text: "n你".to_string(),
        cursor_range: None,
        clauses: Vec::new(),
        clause_availability: ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable,
    }
    .encode()
    .unwrap();
    let commit = route
        .consume(&commit, UiInputEventMetadata::default())
        .expect("matching commit accepted");
    assert!(matches!(
        commit.ui_event,
        UiInputEvent::Ime(ref event) if event.kind == UiImeInputEventKind::Commit
    ));

    let cancel = ZrRuntimeImeCompositionV2 {
        operation: ZrRuntimeImeCompositionOperationV2::Cancel,
        context: ZrRuntimeImeCompositionContextV2 {
            window_generation: 41,
            focus_generation: 2,
            composition_generation: 3,
        },
        text: String::new(),
        cursor_range: None,
        clauses: Vec::new(),
        clause_availability: ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable,
    }
    .encode()
    .unwrap();
    assert_eq!(
        route.consume(&cancel, UiInputEventMetadata::default()),
        Err(RuntimeImeCompositionRouteError::StaleGeneration)
    );
}

#[test]
fn actual_v1_event_envelope_routes_v2_payload_before_any_queue_mutation() {
    let mut route = route(43);
    let payload = preedit(43, 2, 3);
    let event = zircon_runtime_interface::ZrRuntimeEventV1::ime_composition_v2(
        zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1,
        zircon_runtime_interface::ZrRuntimeViewportHandle::new(1),
        zircon_runtime_interface::ZrByteSlice {
            data: payload.as_ptr(),
            len: payload.len(),
        },
    );
    let dispatch = route
        .consume_event(event, UiInputEventMetadata::default())
        .expect("versioned payload should traverse the C event route");
    assert!(matches!(dispatch.ui_event, UiInputEvent::Ime(_)));
    assert!(matches!(dispatch.core_event, InputEvent::Ime(_)));
}

#[test]
fn malformed_or_stale_events_do_not_reach_ui_or_core_queues() {
    let mut route = route(51);
    let valid = preedit(51, 2, 3);
    assert!(route
        .consume(&valid, UiInputEventMetadata::default())
        .is_ok());
    let before = route.generation;
    assert!(matches!(
        route.consume(&valid[..valid.len() - 1], UiInputEventMetadata::default()),
        Err(RuntimeImeCompositionRouteError::Malformed(_))
    ));
    assert_eq!(route.generation, before);
    let stale = preedit(51, 1, 3);
    assert_eq!(
        route.consume(&stale, UiInputEventMetadata::default()),
        Err(RuntimeImeCompositionRouteError::StaleGeneration)
    );
    assert_eq!(route.generation, before);
}

#[test]
fn independent_sessions_reject_cross_window_callbacks_with_reused_local_nodes() {
    let mut first = route(61);
    let mut second = route(62);
    assert!(first
        .consume(&preedit(61, 1, 1), UiInputEventMetadata::default())
        .is_ok());
    assert_eq!(
        second.consume(&preedit(61, 1, 1), UiInputEventMetadata::default()),
        Err(RuntimeImeCompositionRouteError::StaleGeneration)
    );
    assert!(second
        .consume(&preedit(62, 1, 1), UiInputEventMetadata::default())
        .is_ok());
}

#[test]
fn reconfigure_retires_old_window_generation_and_keeps_capability_fail_closed() {
    let mut route = RuntimeImeCompositionRoute::legacy(71);
    let payload = preedit(71, 1, 1);
    assert_eq!(
        route.consume(&payload, UiInputEventMetadata::default()),
        Err(RuntimeImeCompositionRouteError::CapabilityUnavailable)
    );

    route.reconfigure(
        ZrRuntimeImeCompositionNegotiation::from_peer(
            8,
            [ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY],
        ),
        72,
    );
    assert_eq!(
        route.consume(&payload, UiInputEventMetadata::default()),
        Err(RuntimeImeCompositionRouteError::StaleGeneration)
    );
    assert!(route
        .consume(&preedit(72, 1, 1), UiInputEventMetadata::default())
        .is_ok());
}

#[test]
fn two_window_candidate_rects_remain_independent() {
    let first_rect = ZrRuntimeImeCandidateRectV2::window_relative(4, 8, 0, 18);
    let second_rect = ZrRuntimeImeCandidateRectV2::window_relative(19, 23, 0, 20);
    let negotiation = ZrRuntimeImeCompositionNegotiation::from_peer(
        8,
        [ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY],
    );
    let mut first = RuntimeImeCompositionRoute::legacy(81);
    let mut second = RuntimeImeCompositionRoute::legacy(82);
    first.reconfigure_with_candidate_rect(negotiation.clone(), 81, first_rect);
    second.reconfigure_with_candidate_rect(negotiation, 82, second_rect);

    assert_eq!(
        first.candidate_rect_for_window_generation(81),
        Some(first_rect)
    );
    assert_eq!(
        second.candidate_rect_for_window_generation(82),
        Some(second_rect)
    );
    assert_eq!(first.candidate_rect_for_window_generation(82), None);
    assert_eq!(second.candidate_rect_for_window_generation(81), None);
}

#[test]
fn reconfigured_window_rejects_late_candidate_publication() {
    let old_rect = ZrRuntimeImeCandidateRectV2::window_relative(1, 2, 0, 18);
    let new_rect = ZrRuntimeImeCandidateRectV2::window_relative(7, 11, 0, 19);
    let negotiation = ZrRuntimeImeCompositionNegotiation::from_peer(
        8,
        [ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY],
    );
    let mut route = RuntimeImeCompositionRoute::legacy(91);
    route.reconfigure_with_candidate_rect(negotiation.clone(), 91, old_rect);
    assert_eq!(
        route.candidate_rect_for_window_generation(91),
        Some(old_rect)
    );

    route.reconfigure_with_candidate_rect(negotiation, 92, new_rect);
    assert_eq!(route.candidate_rect_for_window_generation(91), None);
    assert_eq!(
        route.candidate_rect_for_window_generation(92),
        Some(new_rect)
    );
}

#[test]
fn teardown_clears_candidate_publication_before_window_drop() {
    let rect = ZrRuntimeImeCandidateRectV2::window_relative(3, 5, 0, 18);
    let negotiation = ZrRuntimeImeCompositionNegotiation::from_peer(
        8,
        [ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY],
    );
    let mut route = RuntimeImeCompositionRoute::legacy(101);
    route.reconfigure_with_candidate_rect(negotiation, 101, rect);
    assert_eq!(route.candidate_rect_for_window_generation(101), Some(rect));

    route.reconfigure(ZrRuntimeImeCompositionNegotiation::legacy(), 102);
    assert_eq!(route.candidate_rect_for_window_generation(101), None);
    assert_eq!(route.candidate_rect_for_window_generation(102), None);
}

#[test]
fn same_window_reconfigure_keeps_high_water_and_updates_candidate_topology() {
    let negotiation = ZrRuntimeImeCompositionNegotiation::from_peer(
        8,
        [ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY],
    );
    let first_rect = ZrRuntimeImeCandidateRectV2::window_relative(3, 5, 0, 18);
    let second_rect = ZrRuntimeImeCandidateRectV2::window_relative(9, 11, 0, 19);
    let mut route = RuntimeImeCompositionRoute::legacy(111);
    route.reconfigure_with_candidate_rect(negotiation.clone(), 111, first_rect);
    let active = preedit(111, 4, 7);
    assert!(route
        .consume(&active, UiInputEventMetadata::default())
        .is_ok());
    let commit = ZrRuntimeImeCompositionV2 {
        operation: ZrRuntimeImeCompositionOperationV2::Commit,
        context: ZrRuntimeImeCompositionContextV2 {
            window_generation: 111,
            focus_generation: 4,
            composition_generation: 7,
        },
        text: "done".to_string(),
        cursor_range: None,
        clauses: Vec::new(),
        clause_availability: ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable,
    }
    .encode()
    .unwrap();
    assert!(route
        .consume(&commit, UiInputEventMetadata::default())
        .is_ok());

    route.reconfigure_with_candidate_rect(negotiation, 111, second_rect);
    assert_eq!(
        route.candidate_rect_for_window_generation(111),
        Some(second_rect)
    );
    assert_eq!(
        route.consume(&active, UiInputEventMetadata::default()),
        Err(RuntimeImeCompositionRouteError::StaleGeneration)
    );
    assert!(route
        .consume(&preedit(111, 4, 8), UiInputEventMetadata::default())
        .is_ok());
}

#[test]
fn same_window_cancel_tombstone_survives_reconfigure_retry() {
    let negotiation = ZrRuntimeImeCompositionNegotiation::from_peer(
        8,
        [ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY],
    );
    let rect = ZrRuntimeImeCandidateRectV2::window_relative(3, 5, 0, 18);
    let mut route = RuntimeImeCompositionRoute::legacy(112);
    route.reconfigure_with_candidate_rect(negotiation.clone(), 112, rect);
    assert!(route
        .consume(&preedit(112, 6, 2), UiInputEventMetadata::default())
        .is_ok());
    let cancel = ZrRuntimeImeCompositionV2 {
        operation: ZrRuntimeImeCompositionOperationV2::Cancel,
        context: ZrRuntimeImeCompositionContextV2 {
            window_generation: 112,
            focus_generation: 6,
            composition_generation: 2,
        },
        text: String::new(),
        cursor_range: None,
        clauses: Vec::new(),
        clause_availability: ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable,
    }
    .encode()
    .unwrap();
    assert!(route
        .consume(&cancel, UiInputEventMetadata::default())
        .is_ok());
    route.reconfigure_with_candidate_rect(negotiation, 112, rect);
    assert_eq!(
        route.consume(&preedit(112, 6, 2), UiInputEventMetadata::default()),
        Err(RuntimeImeCompositionRouteError::StaleGeneration)
    );
    assert!(route
        .consume(&preedit(112, 6, 3), UiInputEventMetadata::default())
        .is_ok());
}
