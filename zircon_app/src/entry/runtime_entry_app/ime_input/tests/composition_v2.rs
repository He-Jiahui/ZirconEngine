use zircon_runtime_interface::ui::dispatch::{
    UiImePreeditClause, UiImePreeditClauseKind, UiTextByteRange,
};
use zircon_runtime_interface::{
    ZrRuntimeImeCompositionNegotiation, ZrRuntimeImeCompositionV2,
    ZrRuntimeImePreeditClauseAvailabilityV2, ZrRuntimeViewportHandle,
    ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY, ZR_RUNTIME_IME_COMPOSITION_V2_EVENT_STATE,
};

use super::*;

fn negotiated() -> ZrRuntimeImeCompositionNegotiation {
    ZrRuntimeImeCompositionNegotiation::from_peer(8, [ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY])
}

#[test]
fn old_peer_keeps_truthful_winit_v1_fallback_without_v2_configuration() {
    let producer = RuntimeImeCompositionProducer::new(
        ZrRuntimeImeCompositionNegotiation::legacy(),
        RuntimeImeCompositionContextAllocator::new(7).unwrap(),
    );
    let event = producer
        .winit_preedit(
            ZrRuntimeViewportHandle::new(1),
            "你🙂",
            Some(UiTextByteRange::new(0, 7)),
        )
        .unwrap();
    assert_eq!(event.state(), ZR_RUNTIME_IME_STATE_PREEDIT_V1);
    assert!(ZrRuntimeImeCompositionV2::decode(event.payload()).is_err());
    assert_eq!(producer.candidate_rect_for_window_generation(7), None);
}

#[test]
fn native_v2_event_carries_real_attributes_and_host_lifetime_context() {
    let mut producer = RuntimeImeCompositionProducer::new(
        negotiated(),
        RuntimeImeCompositionContextAllocator::new(13).unwrap(),
    );
    let context = producer
        .context_allocator_mut()
        .begin_composition()
        .unwrap();
    let clauses = [UiImePreeditClause::new(
        UiTextByteRange::new(0, 3),
        UiImePreeditClauseKind::Converted,
    )];
    let event = producer
        .native_preedit(
            ZrRuntimeViewportHandle::new(1),
            "你",
            Some(UiTextByteRange::new(0, 3)),
            ImeCompositionClauseAvailability::Available(&clauses),
        )
        .unwrap();
    assert_eq!(event.state(), ZR_RUNTIME_IME_COMPOSITION_V2_EVENT_STATE);
    let decoded = ZrRuntimeImeCompositionV2::decode(event.payload()).unwrap();
    assert_eq!(decoded.context, context);
    assert_eq!(decoded.clauses, clauses);
    assert_eq!(
        decoded.clause_availability,
        ZrRuntimeImePreeditClauseAvailabilityV2::Available
    );
}

#[test]
fn generation_exhaustion_fails_closed_without_reusing_a_token() {
    let mut allocator = RuntimeImeCompositionContextAllocator {
        window_generation: 1,
        focus_generation: u64::MAX,
        composition_generation: 9,
    };
    assert_eq!(
        allocator.focus_changed(),
        Err(ImeCompositionProducerError::GenerationExhausted)
    );
}

#[test]
fn two_window_producers_keep_candidate_rect_topology_independent() {
    let first_rect = ZrRuntimeImeCandidateRectV2::window_relative(4, 8, 0, 18);
    let second_rect = ZrRuntimeImeCandidateRectV2::window_relative(19, 23, 0, 20);
    let first = RuntimeImeCompositionProducer::new_with_candidate_rect(
        negotiated(),
        RuntimeImeCompositionContextAllocator::new(201).unwrap(),
        first_rect,
    );
    let second = RuntimeImeCompositionProducer::new_with_candidate_rect(
        negotiated(),
        RuntimeImeCompositionContextAllocator::new(202).unwrap(),
        second_rect,
    );

    assert_eq!(
        first.candidate_rect_for_window_generation(201),
        Some(first_rect)
    );
    assert_eq!(
        second.candidate_rect_for_window_generation(202),
        Some(second_rect)
    );
    assert_eq!(first.candidate_rect_for_window_generation(202), None);
    assert_eq!(second.candidate_rect_for_window_generation(201), None);
}

#[test]
fn reconfigured_window_rejects_late_candidate_publication() {
    let old_rect = ZrRuntimeImeCandidateRectV2::window_relative(1, 2, 0, 18);
    let new_rect = ZrRuntimeImeCandidateRectV2::window_relative(7, 11, 0, 19);
    let old = RuntimeImeCompositionProducer::new_with_candidate_rect(
        negotiated(),
        RuntimeImeCompositionContextAllocator::new(211).unwrap(),
        old_rect,
    );
    let current = RuntimeImeCompositionProducer::new_with_candidate_rect(
        negotiated(),
        RuntimeImeCompositionContextAllocator::new(212).unwrap(),
        new_rect,
    );

    assert_eq!(old.candidate_rect_for_window_generation(212), None);
    assert_eq!(current.candidate_rect_for_window_generation(211), None);
    assert_eq!(
        current.candidate_rect_for_window_generation(212),
        Some(new_rect)
    );
}

#[test]
fn teardown_drops_candidate_publication_before_window_drop() {
    let rect = ZrRuntimeImeCandidateRectV2::window_relative(3, 5, 0, 18);
    let producer = RuntimeImeCompositionProducer::new_with_candidate_rect(
        negotiated(),
        RuntimeImeCompositionContextAllocator::new(221).unwrap(),
        rect,
    );
    let mut active = Some(producer);
    assert_eq!(
        active
            .as_ref()
            .and_then(|producer| producer.candidate_rect_for_window_generation(221)),
        Some(rect)
    );

    active = None;
    assert!(active.is_none());
}
