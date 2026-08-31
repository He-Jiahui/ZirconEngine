use std::{hint::black_box, time::Instant};

use super::*;

#[test]
fn pointer_binding_target_in_place_filter_p95_beats_allocating_filter() {
    const SAMPLE_PAIRS: usize = 21;
    const EVENT_BATCHES_PER_SAMPLE: usize = 32;
    const EVENTS_PER_BATCH: usize = 512;

    let mut targeted = binding("Showcase/FilterBenchmarkTarget", UiEventKind::Click);
    targeted.targets = vec![target(UiBindingTarget::prop("text"), r#""Filtered""#)];
    let mut surface = bound_button_surface(vec![
        targeted,
        binding("Showcase/FilterBenchmarkPassthrough", UiEventKind::Click),
    ]);
    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let initial = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(initial.component_events.len(), 2);
    let targeted_event = initial.component_events[0].clone();
    let passthrough_event = initial.component_events[1].clone();
    let mut workload = Vec::with_capacity(EVENTS_PER_BATCH);
    workload.push(targeted_event);
    workload.extend(std::iter::repeat_n(
        passthrough_event,
        EVENTS_PER_BATCH.saturating_sub(1),
    ));

    let _ =
        sample_pointer_binding_target_filter(&surface, &workload, EVENT_BATCHES_PER_SAMPLE, true);
    let _ =
        sample_pointer_binding_target_filter(&surface, &workload, EVENT_BATCHES_PER_SAMPLE, false);

    let mut legacy_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples_us.push(sample_pointer_binding_target_filter(
                &surface,
                &workload,
                EVENT_BATCHES_PER_SAMPLE,
                true,
            ));
            optimized_samples_us.push(sample_pointer_binding_target_filter(
                &surface,
                &workload,
                EVENT_BATCHES_PER_SAMPLE,
                false,
            ));
        } else {
            optimized_samples_us.push(sample_pointer_binding_target_filter(
                &surface,
                &workload,
                EVENT_BATCHES_PER_SAMPLE,
                false,
            ));
            legacy_samples_us.push(sample_pointer_binding_target_filter(
                &surface,
                &workload,
                EVENT_BATCHES_PER_SAMPLE,
                true,
            ));
        }
    }

    let legacy_p95_us = nearest_rank_p95(&legacy_samples_us);
    let optimized_p95_us = nearest_rank_p95(&optimized_samples_us);
    assert!(
        optimized_p95_us.saturating_mul(100) <= legacy_p95_us.saturating_mul(85),
        "in-place event filter P95 {optimized_p95_us}us must improve allocating filter P95 {legacy_p95_us}us by at least 15%"
    );
    println!(
        "PERF-RUNTIME74-BINDING-EVENT-FILTER sample_pairs={SAMPLE_PAIRS} event_batches_per_sample={EVENT_BATCHES_PER_SAMPLE} events_per_batch={EVENTS_PER_BATCH} targeted_events_per_batch=1 pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 legacy_event_buffer_allocations_per_sample={EVENT_BATCHES_PER_SAMPLE} optimized_event_buffer_allocations_per_sample=0 allocation_reduction_percent=100 legacy_samples_us={} optimized_samples_us={} legacy_p95_us={legacy_p95_us} optimized_p95_us={optimized_p95_us} improvement_threshold_percent=15",
        joined_samples(&legacy_samples_us),
        joined_samples(&optimized_samples_us),
    );
}

fn sample_pointer_binding_target_filter(
    surface: &UiSurface,
    workload: &[zircon_runtime_interface::ui::dispatch::UiPointerComponentEvent],
    batches: usize,
    legacy: bool,
) -> u128 {
    let mut surface = surface.clone();
    let mut event_batches = (0..batches).map(|_| workload.to_vec()).collect::<Vec<_>>();
    let started = Instant::now();
    for events in &mut event_batches {
        let reports = if legacy {
            surface.apply_pointer_binding_targets_legacy_for_benchmark(events)
        } else {
            surface.apply_pointer_binding_targets(events)
        }
        .expect("benchmark target application should remain valid");
        black_box(reports);
        black_box(events.len());
    }
    started.elapsed().as_micros().max(1)
}

#[test]
fn dense_action_payload_override_handoff_p95_beats_string_clone_handoff() {
    const SAMPLE_PAIRS: usize = 21;
    const DISPATCHES_PER_SAMPLE: usize = 256;
    const PAYLOAD_FIELDS: usize = 16;
    const BYTES_PER_VALUE: usize = 1_024;

    let mut click = binding("Showcase/DensePayloadBenchmark", UiEventKind::Click);
    let mut source_payload = BTreeMap::new();
    for field_index in 0..PAYLOAD_FIELDS {
        source_payload.insert(
            format!("field_{field_index:02}"),
            toml::Value::String("source".to_string()),
        );
    }
    click.action = Some(UiActionRef {
        route: Some("showcase.dense_payload_benchmark".to_string()),
        action: None,
        payload: source_payload,
        payload_missing_policy: Default::default(),
    });
    let mut surface = bound_button_surface(vec![click]);
    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let handle = result.component_events[0]
        .compiled_binding
        .expect("benchmark binding should carry its compiled handle");
    let string_overrides = (0..PAYLOAD_FIELDS)
        .map(|field_index| {
            (
                format!("field_{field_index:02}"),
                UiValue::String("v".repeat(BYTES_PER_VALUE)),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let dense_overrides = surface
        .dense_compiled_payload_overrides_for_benchmark(handle, string_overrides.clone())
        .expect("benchmark fields should resolve to compiled property ids");

    let _ = sample_compiled_payload_override_handoff(
        &surface,
        handle,
        &string_overrides,
        &dense_overrides,
        DISPATCHES_PER_SAMPLE,
        true,
    );
    let _ = sample_compiled_payload_override_handoff(
        &surface,
        handle,
        &string_overrides,
        &dense_overrides,
        DISPATCHES_PER_SAMPLE,
        false,
    );

    let mut legacy_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples_us.push(sample_compiled_payload_override_handoff(
                &surface,
                handle,
                &string_overrides,
                &dense_overrides,
                DISPATCHES_PER_SAMPLE,
                true,
            ));
            optimized_samples_us.push(sample_compiled_payload_override_handoff(
                &surface,
                handle,
                &string_overrides,
                &dense_overrides,
                DISPATCHES_PER_SAMPLE,
                false,
            ));
        } else {
            optimized_samples_us.push(sample_compiled_payload_override_handoff(
                &surface,
                handle,
                &string_overrides,
                &dense_overrides,
                DISPATCHES_PER_SAMPLE,
                false,
            ));
            legacy_samples_us.push(sample_compiled_payload_override_handoff(
                &surface,
                handle,
                &string_overrides,
                &dense_overrides,
                DISPATCHES_PER_SAMPLE,
                true,
            ));
        }
    }

    let legacy_p95_us = nearest_rank_p95(&legacy_samples_us);
    let optimized_p95_us = nearest_rank_p95(&optimized_samples_us);
    assert!(
        optimized_p95_us.saturating_mul(100) <= legacy_p95_us.saturating_mul(80),
        "dense payload override handoff P95 {optimized_p95_us}us must improve string clone handoff P95 {legacy_p95_us}us by at least 20%"
    );
    println!(
        "PERF-RUNTIME74-DENSE-PAYLOAD-OVERRIDES sample_pairs={SAMPLE_PAIRS} dispatches_per_sample={DISPATCHES_PER_SAMPLE} payload_fields={PAYLOAD_FIELDS} bytes_per_value={BYTES_PER_VALUE} pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 legacy_override_key_kind=string optimized_override_key_kind=ui_property_id legacy_handoff_value_clones_per_dispatch={PAYLOAD_FIELDS} optimized_handoff_value_clones_per_dispatch=0 clone_reduction_percent=100 legacy_samples_us={} optimized_samples_us={} legacy_p95_us={legacy_p95_us} optimized_p95_us={optimized_p95_us} improvement_threshold_percent=20",
        joined_samples(&legacy_samples_us),
        joined_samples(&optimized_samples_us),
    );
}

fn sample_compiled_payload_override_handoff(
    surface: &UiSurface,
    handle: zircon_runtime_interface::ui::template::UiCompiledBindingHandle,
    string_overrides: &BTreeMap<String, UiValue>,
    dense_overrides: &BTreeMap<zircon_runtime_interface::ui::template::UiPropertyId, UiValue>,
    dispatches: usize,
    legacy: bool,
) -> u128 {
    let legacy_batches = legacy
        .then(|| std::iter::repeat_n(string_overrides.clone(), dispatches).collect::<Vec<_>>());
    let optimized_batches = (!legacy)
        .then(|| std::iter::repeat_n(dense_overrides.clone(), dispatches).collect::<Vec<_>>());
    let started = Instant::now();
    if let Some(batches) = legacy_batches {
        for overrides in batches {
            black_box(
                surface
                    .template_action_for_compiled_binding_with_legacy_overrides_for_benchmark(
                        UiNodeId::new(2),
                        handle,
                        overrides,
                    )
                    .expect("legacy benchmark dispatch should resolve"),
            );
        }
    } else if let Some(batches) = optimized_batches {
        for overrides in batches {
            black_box(
                surface
                    .template_action_for_compiled_binding_with_overrides(
                        UiNodeId::new(2),
                        handle,
                        overrides,
                    )
                    .expect("optimized benchmark dispatch should resolve"),
            );
        }
    }
    started.elapsed().as_micros().max(1)
}

#[test]
fn compiled_binding_event_index_p95_beats_authored_binding_scan() {
    const SAMPLE_PAIRS: usize = 21;
    const LOOKUPS_PER_SAMPLE: usize = 4_096;
    const BINDINGS_PER_NODE: usize = 256;
    const MATCHING_BINDINGS: usize = 16;
    const EVENT_KINDS: [UiEventKind; 16] = [
        UiEventKind::Click,
        UiEventKind::DoubleClick,
        UiEventKind::Hover,
        UiEventKind::Press,
        UiEventKind::Release,
        UiEventKind::Change,
        UiEventKind::Submit,
        UiEventKind::Toggle,
        UiEventKind::Focus,
        UiEventKind::Blur,
        UiEventKind::Scroll,
        UiEventKind::Resize,
        UiEventKind::DragBegin,
        UiEventKind::DragUpdate,
        UiEventKind::DragEnd,
        UiEventKind::Drop,
    ];

    let bindings = (0..BINDINGS_PER_NODE)
        .map(|index| {
            binding(
                &format!("Showcase/EventIndex{index:03}"),
                EVENT_KINDS[index % EVENT_KINDS.len()],
            )
        })
        .collect();
    let surface = bound_button_surface(bindings);
    assert_eq!(
        surface.compiled_binding_event_source_count_for_test(UiNodeId::new(2), UiEventKind::Click),
        Some(MATCHING_BINDINGS)
    );

    let _ =
        sample_compiled_binding_event_lookup(&surface, LOOKUPS_PER_SAMPLE, MATCHING_BINDINGS, true);
    let _ = sample_compiled_binding_event_lookup(
        &surface,
        LOOKUPS_PER_SAMPLE,
        MATCHING_BINDINGS,
        false,
    );
    let mut legacy_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples_us.push(sample_compiled_binding_event_lookup(
                &surface,
                LOOKUPS_PER_SAMPLE,
                MATCHING_BINDINGS,
                true,
            ));
            optimized_samples_us.push(sample_compiled_binding_event_lookup(
                &surface,
                LOOKUPS_PER_SAMPLE,
                MATCHING_BINDINGS,
                false,
            ));
        } else {
            optimized_samples_us.push(sample_compiled_binding_event_lookup(
                &surface,
                LOOKUPS_PER_SAMPLE,
                MATCHING_BINDINGS,
                false,
            ));
            legacy_samples_us.push(sample_compiled_binding_event_lookup(
                &surface,
                LOOKUPS_PER_SAMPLE,
                MATCHING_BINDINGS,
                true,
            ));
        }
    }

    let legacy_p95_us = nearest_rank_p95(&legacy_samples_us);
    let optimized_p95_us = nearest_rank_p95(&optimized_samples_us);
    assert!(
        optimized_p95_us.saturating_mul(100) <= legacy_p95_us.saturating_mul(50),
        "compiled event index P95 {optimized_p95_us}us must improve authored binding scan P95 {legacy_p95_us}us by at least 50%"
    );
    println!(
        "PERF-RUNTIME74-COMPILED-EVENT-INDEX sample_pairs={SAMPLE_PAIRS} lookups_per_sample={LOOKUPS_PER_SAMPLE} bindings_per_node={BINDINGS_PER_NODE} matching_bindings={MATCHING_BINDINGS} pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 legacy_binding_probes_per_lookup={BINDINGS_PER_NODE} optimized_binding_probes_per_lookup={MATCHING_BINDINGS} probe_reduction_percent=93 legacy_samples_us={} optimized_samples_us={} legacy_p95_us={legacy_p95_us} optimized_p95_us={optimized_p95_us} improvement_threshold_percent=50",
        joined_samples(&legacy_samples_us),
        joined_samples(&optimized_samples_us),
    );
}

fn sample_compiled_binding_event_lookup(
    surface: &UiSurface,
    lookups: usize,
    matching_bindings: usize,
    legacy: bool,
) -> u128 {
    let started = Instant::now();
    let mut matched = 0usize;
    let metadata = surface
        .tree
        .node(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .expect("benchmark node should retain authored binding metadata");
    for _ in 0..lookups {
        if legacy {
            for (source_index, binding) in metadata.bindings.iter().enumerate() {
                if binding.event == UiEventKind::Click {
                    let handle = surface
                        .compiled_binding_handle_for_source(
                            UiNodeId::new(2),
                            source_index,
                            binding,
                            UiEventKind::Click,
                        )
                        .expect("legacy scan match should resolve the compiled handle");
                    black_box((source_index, binding.id.as_str(), handle));
                    matched += 1;
                }
            }
        } else {
            for (_, indexed_handle, _) in surface
                .compiled_binding_event_sources_for_benchmark(UiNodeId::new(2), UiEventKind::Click)
            {
                black_box(indexed_handle);
                matched += 1;
            }
        }
    }
    let elapsed_us = started.elapsed().as_micros().max(1);
    assert_eq!(matched, lookups.saturating_mul(matching_bindings));
    elapsed_us
}

#[test]
#[ignore = "release-only direct compiled event handle evidence"]
fn direct_compiled_event_handle_p95_beats_index_revalidation() {
    const SAMPLE_PAIRS: usize = 21;
    const LOOKUPS_PER_SAMPLE: usize = 4_096;
    const MATCHING_BINDINGS: usize = 128;

    let bindings = (0..MATCHING_BINDINGS)
        .map(|index| {
            binding(
                &format!("Showcase/DirectHandle{index:03}"),
                UiEventKind::Click,
            )
        })
        .collect();
    let surface = bound_button_surface(bindings);
    let _ =
        sample_direct_compiled_event_handle(&surface, LOOKUPS_PER_SAMPLE, MATCHING_BINDINGS, true);
    let _ =
        sample_direct_compiled_event_handle(&surface, LOOKUPS_PER_SAMPLE, MATCHING_BINDINGS, false);

    let mut legacy_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples_us.push(sample_direct_compiled_event_handle(
                &surface,
                LOOKUPS_PER_SAMPLE,
                MATCHING_BINDINGS,
                true,
            ));
            optimized_samples_us.push(sample_direct_compiled_event_handle(
                &surface,
                LOOKUPS_PER_SAMPLE,
                MATCHING_BINDINGS,
                false,
            ));
        } else {
            optimized_samples_us.push(sample_direct_compiled_event_handle(
                &surface,
                LOOKUPS_PER_SAMPLE,
                MATCHING_BINDINGS,
                false,
            ));
            legacy_samples_us.push(sample_direct_compiled_event_handle(
                &surface,
                LOOKUPS_PER_SAMPLE,
                MATCHING_BINDINGS,
                true,
            ));
        }
    }

    let legacy_p95_us = nearest_rank_p95(&legacy_samples_us);
    let optimized_p95_us = nearest_rank_p95(&optimized_samples_us);
    assert!(
        optimized_p95_us.saturating_mul(100) <= legacy_p95_us.saturating_mul(50),
        "direct compiled handle P95 {optimized_p95_us}us must improve indexed revalidation P95 {legacy_p95_us}us by at least 50%"
    );
    println!(
        "PERF-RUNTIME74-DIRECT-COMPILED-EVENT-HANDLE sample_pairs={SAMPLE_PAIRS} lookups_per_sample={LOOKUPS_PER_SAMPLE} matching_bindings={MATCHING_BINDINGS} pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 legacy_handle_revalidations_per_lookup={MATCHING_BINDINGS} optimized_handle_revalidations_per_lookup=0 legacy_binding_name_comparisons_per_lookup={MATCHING_BINDINGS} optimized_binding_name_comparisons_per_lookup=0 revalidation_reduction_percent=100 legacy_samples_us={} optimized_samples_us={} legacy_p95_us={legacy_p95_us} optimized_p95_us={optimized_p95_us} improvement_threshold_percent=50",
        joined_samples(&legacy_samples_us),
        joined_samples(&optimized_samples_us),
    );
}

fn sample_direct_compiled_event_handle(
    surface: &UiSurface,
    lookups: usize,
    matching_bindings: usize,
    legacy: bool,
) -> u128 {
    let metadata = surface
        .tree
        .node(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .expect("benchmark node should retain authored binding metadata");
    let started = Instant::now();
    let mut matched = 0usize;
    for _ in 0..lookups {
        for (source_index, indexed_handle, component_event) in surface
            .compiled_binding_event_sources_for_benchmark(UiNodeId::new(2), UiEventKind::Click)
        {
            if legacy {
                let binding = metadata
                    .bindings
                    .get(source_index)
                    .expect("legacy source slot should remain available");
                let handle = surface
                    .compiled_binding_handle_for_source(
                        UiNodeId::new(2),
                        source_index,
                        binding,
                        UiEventKind::Click,
                    )
                    .filter(|handle| *handle == indexed_handle)
                    .expect("legacy indexed handle should revalidate");
                black_box((binding.id.as_str(), handle, component_event));
            } else {
                black_box((indexed_handle, component_event));
            }
            matched += 1;
        }
    }
    let elapsed_us = started.elapsed().as_micros().max(1);
    assert_eq!(matched, lookups.saturating_mul(matching_bindings));
    elapsed_us
}

#[test]
#[ignore = "release-only single binding event payload move evidence"]
fn single_binding_event_payload_move_p95_beats_clone() {
    const SAMPLE_PAIRS: usize = 21;
    const EVENTS_PER_SAMPLE: usize = 2_048;
    const BYTES_PER_PAYLOAD: usize = 4_096;

    let surface = bound_button_surface(vec![binding(
        "Showcase/PayloadMoveBenchmark",
        UiEventKind::Change,
    )]);
    let _ = sample_single_binding_event_payload_move(
        &surface,
        EVENTS_PER_SAMPLE,
        BYTES_PER_PAYLOAD,
        true,
    );
    let _ = sample_single_binding_event_payload_move(
        &surface,
        EVENTS_PER_SAMPLE,
        BYTES_PER_PAYLOAD,
        false,
    );

    let mut legacy_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples_us.push(sample_single_binding_event_payload_move(
                &surface,
                EVENTS_PER_SAMPLE,
                BYTES_PER_PAYLOAD,
                true,
            ));
            optimized_samples_us.push(sample_single_binding_event_payload_move(
                &surface,
                EVENTS_PER_SAMPLE,
                BYTES_PER_PAYLOAD,
                false,
            ));
        } else {
            optimized_samples_us.push(sample_single_binding_event_payload_move(
                &surface,
                EVENTS_PER_SAMPLE,
                BYTES_PER_PAYLOAD,
                false,
            ));
            legacy_samples_us.push(sample_single_binding_event_payload_move(
                &surface,
                EVENTS_PER_SAMPLE,
                BYTES_PER_PAYLOAD,
                true,
            ));
        }
    }

    let legacy_p95_us = nearest_rank_p95(&legacy_samples_us);
    let optimized_p95_us = nearest_rank_p95(&optimized_samples_us);
    assert!(
        optimized_p95_us.saturating_mul(100) <= legacy_p95_us.saturating_mul(80),
        "moved event payload P95 {optimized_p95_us}us must improve cloned payload P95 {legacy_p95_us}us by at least 20%"
    );
    println!(
        "PERF-RUNTIME74-SINGLE-EVENT-PAYLOAD-MOVE sample_pairs={SAMPLE_PAIRS} events_per_sample={EVENTS_PER_SAMPLE} bytes_per_payload={BYTES_PER_PAYLOAD} matching_bindings=1 pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 legacy_payload_clones_per_sample={EVENTS_PER_SAMPLE} optimized_payload_clones_per_sample=0 clone_reduction_percent=100 legacy_samples_us={} optimized_samples_us={} legacy_p95_us={legacy_p95_us} optimized_p95_us={optimized_p95_us} improvement_threshold_percent=20",
        joined_samples(&legacy_samples_us),
        joined_samples(&optimized_samples_us),
    );
}

fn sample_single_binding_event_payload_move(
    surface: &UiSurface,
    events_per_sample: usize,
    bytes_per_payload: usize,
    legacy: bool,
) -> u128 {
    let payload = "p".repeat(bytes_per_payload);
    let workload = (0..events_per_sample)
        .map(|_| UiComponentEvent::KeyboardText {
            text: payload.clone(),
        })
        .collect::<Vec<_>>();
    let mut emitted = Vec::with_capacity(events_per_sample);
    let started = Instant::now();
    for event in workload {
        if legacy {
            surface.push_pointer_component_events_legacy_for_benchmark(
                &mut emitted,
                UiNodeId::new(2),
                UiEventKind::Change,
                event,
                UiPointerComponentEventReason::DirectBinding,
            )
        } else {
            surface.push_pointer_component_events_for_test(
                &mut emitted,
                UiNodeId::new(2),
                UiEventKind::Change,
                event,
                UiPointerComponentEventReason::DirectBinding,
            )
        }
        .expect("benchmark event should resolve its single binding");
    }
    let elapsed_us = started.elapsed().as_micros().max(1);
    assert_eq!(emitted.len(), events_per_sample);
    black_box(emitted);
    elapsed_us
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(95).div_ceil(100).max(1);
    sorted[rank - 1]
}

fn joined_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
