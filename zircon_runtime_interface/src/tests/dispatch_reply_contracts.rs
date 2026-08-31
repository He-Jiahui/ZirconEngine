use crate::ui::{
    dispatch::{
        UiDispatchDisposition, UiDispatchEffect, UiDispatchHostRequestKind, UiDispatchPhase,
        UiDispatchReply, UiDispatchReplyStep, UiDispatchReplyStepTrace, UiFocusEffectReason,
        UiRedrawRequestReason, UiTransientDismissalReason, UiTransientDismissalTarget,
    },
    event_ui::UiNodeId,
    text::UiRichLinkTarget,
    tree::UiDirtyFlags,
};

#[test]
fn dispatch_reply_merge_drops_unhandled_step_effects_and_keeps_passthrough_effects() {
    let root = UiNodeId::new(1);
    let field = UiNodeId::new(2);
    let ignored = UiNodeId::new(3);

    let report = UiDispatchReply::merge_route([
        UiDispatchReplyStep::new(
            UiDispatchPhase::Preprocess,
            Some(ignored),
            UiDispatchReply::unhandled().with_effect(UiDispatchEffect::SetFocus {
                target: ignored,
                reason: UiFocusEffectReason::Input,
            }),
        ),
        UiDispatchReplyStep::new(
            UiDispatchPhase::PreviewTunnel,
            Some(root),
            UiDispatchReply::passthrough().with_effect(UiDispatchEffect::DirtyRedraw {
                target: root,
                dirty: UiDirtyFlags {
                    input: true,
                    ..UiDirtyFlags::default()
                },
                reason: UiRedrawRequestReason::Input,
            }),
        ),
        UiDispatchReplyStep::new(
            UiDispatchPhase::Target,
            Some(field),
            UiDispatchReply::handled().with_effect(UiDispatchEffect::SetFocus {
                target: field,
                reason: UiFocusEffectReason::Input,
            }),
        ),
    ]);

    assert_eq!(report.step_count, 3);
    assert!(report.stopped);
    assert_eq!(report.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(report.reply.handler, Some(field));
    assert_eq!(report.reply.phase, Some(UiDispatchPhase::Target));
    assert_eq!(report.reply.effects.len(), 2);
    assert!(matches!(
        report.reply.effects[0],
        UiDispatchEffect::DirtyRedraw { target, .. } if target == root
    ));
    assert!(matches!(
        report.reply.effects[1],
        UiDispatchEffect::SetFocus { target, .. } if target == field
    ));

    assert_eq!(report.steps.len(), 3);
    assert_eq!(report.steps[0].handler, Some(ignored));
    assert_eq!(report.steps[0].effect_start, 0);
    assert_eq!(report.steps[0].effect_count, 0);
    assert_eq!(report.steps[0].ignored_effect_count, 1);
    assert_eq!(
        report.steps[0].disposition,
        UiDispatchDisposition::Unhandled
    );
    assert!(!report.steps[0].stopped);

    assert_eq!(report.steps[1].handler, Some(root));
    assert_eq!(report.steps[1].effect_start, 0);
    assert_eq!(report.steps[1].effect_count, 1);
    assert_eq!(report.steps[1].ignored_effect_count, 0);
    assert_eq!(
        report.steps[1].disposition,
        UiDispatchDisposition::Passthrough
    );
    assert!(!report.steps[1].stopped);

    assert_eq!(report.steps[2].handler, Some(field));
    assert_eq!(report.steps[2].effect_start, 1);
    assert_eq!(report.steps[2].effect_count, 1);
    assert_eq!(report.steps[2].ignored_effect_count, 0);
    assert_eq!(report.steps[2].disposition, UiDispatchDisposition::Handled);
    assert!(report.steps[2].stopped);

    let mut legacy_step = serde_json::to_value(&report.steps[1]).unwrap();
    legacy_step
        .as_object_mut()
        .unwrap()
        .remove("ignored_effect_count");
    let legacy_step: UiDispatchReplyStepTrace = serde_json::from_value(legacy_step).unwrap();
    assert_eq!(legacy_step.ignored_effect_count, 0);
}

#[test]
fn dispatch_reply_merge_reserves_route_trace_capacity_from_exact_size_steps() {
    let steps = [
        UiDispatchReplyStep::new(
            UiDispatchPhase::Preprocess,
            None,
            UiDispatchReply::unhandled(),
        ),
        UiDispatchReplyStep::new(
            UiDispatchPhase::PreviewTunnel,
            None,
            UiDispatchReply::passthrough(),
        ),
        UiDispatchReplyStep::new(UiDispatchPhase::Target, None, UiDispatchReply::handled()),
    ];

    let report = UiDispatchReply::merge_route(steps);

    assert_eq!(report.steps.len(), 3);
    assert_eq!(
        report.steps.capacity(),
        3,
        "exact-size dispatch routes should allocate their trace once"
    );
}

#[test]
fn dispatch_reply_merge_does_not_trust_an_unbounded_size_hint() {
    struct MisleadingHint(Option<UiDispatchReplyStep>);

    impl Iterator for MisleadingHint {
        type Item = UiDispatchReplyStep;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.take()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (4096, None)
        }
    }

    let steps = MisleadingHint(Some(UiDispatchReplyStep::new(
        UiDispatchPhase::Target,
        None,
        UiDispatchReply::handled(),
    )));

    let report = UiDispatchReply::merge_route(steps);

    assert_eq!(report.steps.len(), 1);
    assert!(
        report.steps.capacity() < 4096,
        "non-exact iterators must not trigger unbounded eager route allocation"
    );
}

#[test]
#[ignore = "release-only dispatch reply route trace capacity benchmark"]
fn dispatch_reply_route_trace_capacity_release_benchmark_evidence() {
    use std::hint::black_box;
    use std::time::Instant;

    const ROUTE_LEN: usize = 64;
    const MERGES_PER_SAMPLE: usize = 10_000;
    const SAMPLE_PAIRS: usize = 21;

    struct UnhintedSteps<'a> {
        inner: std::slice::Iter<'a, UiDispatchReplyStep>,
    }

    impl<'a> Iterator for UnhintedSteps<'a> {
        type Item = UiDispatchReplyStep;

        fn next(&mut self) -> Option<Self::Item> {
            self.inner.next().cloned()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, None)
        }
    }

    fn measure_unhinted(steps: &[UiDispatchReplyStep]) -> u128 {
        let started = Instant::now();
        for _ in 0..MERGES_PER_SAMPLE {
            black_box(UiDispatchReply::merge_route(UnhintedSteps {
                inner: steps.iter(),
            }));
        }
        started.elapsed().as_nanos().max(1)
    }

    fn measure_exact(steps: &[UiDispatchReplyStep]) -> u128 {
        let started = Instant::now();
        for _ in 0..MERGES_PER_SAMPLE {
            black_box(UiDispatchReply::merge_route(steps.iter().cloned()));
        }
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    let steps = (0..ROUTE_LEN)
        .map(|index| {
            let terminal = index + 1 == ROUTE_LEN;
            UiDispatchReplyStep::new(
                if terminal {
                    UiDispatchPhase::Target
                } else {
                    UiDispatchPhase::PreviewTunnel
                },
                None,
                if terminal {
                    UiDispatchReply::handled()
                } else {
                    UiDispatchReply::passthrough()
                },
            )
        })
        .collect::<Vec<_>>();

    for _ in 0..4 {
        black_box(measure_unhinted(&steps));
        black_box(measure_exact(&steps));
    }

    let mut unhinted_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut exact_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unhinted_samples.push(measure_unhinted(&steps));
            exact_samples.push(measure_exact(&steps));
        } else {
            exact_samples.push(measure_exact(&steps));
            unhinted_samples.push(measure_unhinted(&steps));
        }
    }

    let unhinted_p50_ns = percentile(&unhinted_samples, 50);
    let exact_p50_ns = percentile(&exact_samples, 50);
    let unhinted_p95_ns = percentile(&unhinted_samples, 95);
    let exact_p95_ns = percentile(&exact_samples, 95);
    println!(
        "EDITOR01_DISPATCH_REPLY_ROUTE_TRACE_CAPACITY_BENCH_V1 \
route_len={ROUTE_LEN} merges_per_sample={MERGES_PER_SAMPLE} \
sample_pairs={SAMPLE_PAIRS} pair_order=alternating_unhinted_even \
unhinted_p50_ns={unhinted_p50_ns} exact_p50_ns={exact_p50_ns} \
unhinted_p95_ns={unhinted_p95_ns} exact_p95_ns={exact_p95_ns} \
unhinted_raw_ns={} exact_raw_ns={}",
        raw(&unhinted_samples),
        raw(&exact_samples),
    );

    assert!(
        exact_p95_ns.saturating_mul(100) <= unhinted_p95_ns.saturating_mul(95),
        "exact route trace reservation must reduce P95 by at least 5%: \
unhinted={unhinted_p95_ns}ns exact={exact_p95_ns}ns"
    );
}

#[test]
fn dispatch_reply_transient_dismissal_effect_roundtrips_with_host_request_kind() {
    let reply = UiDispatchReply::handled().with_effect(UiDispatchEffect::DismissTransientUi {
        target: UiTransientDismissalTarget::All,
        reason: UiTransientDismissalReason::WindowAction,
    });

    let reply_round_trip: UiDispatchReply =
        serde_json::from_value(serde_json::to_value(&reply).unwrap()).unwrap();
    assert_eq!(reply_round_trip, reply);
    assert!(matches!(
        reply_round_trip.effects[0],
        UiDispatchEffect::DismissTransientUi {
            target: UiTransientDismissalTarget::All,
            reason: UiTransientDismissalReason::WindowAction,
        }
    ));

    let host_request = UiDispatchHostRequestKind::DismissTransientUi {
        target: UiTransientDismissalTarget::PopupStack,
        reason: UiTransientDismissalReason::OutsideInteraction,
    };
    let host_request_round_trip: UiDispatchHostRequestKind =
        serde_json::from_value(serde_json::to_value(&host_request).unwrap()).unwrap();
    assert_eq!(host_request_round_trip, host_request);
}

#[test]
fn dispatch_reply_rich_link_activation_roundtrips_with_host_request_kind() {
    let target = UiNodeId::new(23);
    let effect = UiDispatchEffect::RequestLinkActivation {
        target,
        link_target: UiRichLinkTarget::parse("res://docs/help.md").unwrap(),
    };
    let effect_json = serde_json::to_value(&effect).unwrap();
    assert_eq!(
        effect_json["RequestLinkActivation"]["href"],
        "res://docs/help.md"
    );
    assert!(
        effect_json["RequestLinkActivation"]
            .get("link_target")
            .is_none()
    );
    let reply = UiDispatchReply::handled().with_effect(effect);

    let reply_round_trip: UiDispatchReply =
        serde_json::from_value(serde_json::to_value(&reply).unwrap()).unwrap();
    assert_eq!(reply_round_trip, reply);

    let host_request = UiDispatchHostRequestKind::ActivateLink {
        target,
        link_target: UiRichLinkTarget::parse("res://docs/help.md").unwrap(),
    };
    let host_request_json = serde_json::to_value(&host_request).unwrap();
    assert_eq!(
        host_request_json["ActivateLink"]["href"],
        "res://docs/help.md"
    );
    assert!(
        host_request_json["ActivateLink"]
            .get("link_target")
            .is_none()
    );
    let host_request_round_trip: UiDispatchHostRequestKind =
        serde_json::from_value(serde_json::to_value(&host_request).unwrap()).unwrap();
    assert_eq!(host_request_round_trip, host_request);
}
