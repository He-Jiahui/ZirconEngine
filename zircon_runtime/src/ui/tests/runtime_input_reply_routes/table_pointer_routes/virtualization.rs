use super::*;

const VIRTUAL_WINDOW_PERF_SAMPLE_COUNT: usize = 31;
const VIRTUAL_WINDOW_PERF_WHEEL_COUNT: usize = 1_000;
const VIRTUAL_WINDOW_PERF_LOGICAL_COUNTS: [i64; 4] = [1, 100, 10_000, 100_000];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct VirtualWindowWheelWork {
    transaction_count: usize,
    component_event_count: usize,
    binding_update_count: usize,
    dirty_node_count: usize,
    layout_visited_node_count: usize,
    arranged_outer_node_visit_count: usize,
    hit_grid_outer_node_visit_count: usize,
    render_outer_node_visit_count: usize,
    render_command_rebuilt_count: usize,
    render_command_reused_count: usize,
}

#[test]
fn table_scroll_updates_virtual_window_and_emits_visible_range() {
    let mut surface = table_pointer_route_surface(false, false);

    let result = dispatch_table_scroll(&mut surface, UiPoint::new(20.0, 76.0), 48.0);

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event == UiComponentEvent::SetVisibleRange { start: 2, count: 4 }
    }));
    assert_eq!(result.binding_reports.len(), 1);
    assert_eq!(result.binding_reports[0].applied_count, 28);
    assert_eq!(result.binding_reports[0].updates.len(), 28);
    assert_table_attr_int(&surface, "viewport_start", 2);
    assert_table_attr_int(&surface, "viewport_count", 4);
    assert_table_attr_int(&surface, "visible_end", 6);
    assert_table_attr_int(&surface, "requested_start", 0);
    assert_table_attr_int(&surface, "requested_count", 8);
    assert_table_attr_float(&surface, "scrollTop", 48.0);
}

#[test]
fn data_grid_scroll_updates_mui_virtual_window_aliases() {
    let mut surface = table_pointer_route_surface(true, false);

    let result = dispatch_table_scroll(&mut surface, UiPoint::new(20.0, 100.0), 72.0);

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event == UiComponentEvent::SetVisibleRange { start: 3, count: 4 }
    }));
    assert_table_attr_int(&surface, "viewport_start", 3);
    assert_table_attr_int(&surface, "viewport_count", 4);
    assert_table_attr_int(&surface, "visibleEnd", 7);
    assert_table_attr_int(&surface, "requestedStart", 1);
    assert_table_attr_int(&surface, "requestedCount", 8);
    assert_table_attr_float(&surface, "scrollTop", 72.0);
}

#[test]
fn data_grid_disable_virtualization_blocks_default_virtual_scroll() {
    let mut surface =
        table_pointer_route_surface_with_virtualization_options(true, false, false, false, true);

    let result = dispatch_table_scroll(&mut surface, UiPoint::new(20.0, 100.0), 72.0);

    assert!(!result
        .component_events
        .iter()
        .any(|event| matches!(event.event, UiComponentEvent::SetVisibleRange { .. })));
    assert_table_attr_int(&surface, "viewport_start", 0);
    assert_table_attr_float(&surface, "scrollTop", 0.0);
}

#[test]
fn table_virtual_window_at_boundary_emits_no_transaction() {
    let mut surface = table_pointer_route_surface(false, false);

    let first = dispatch_table_scroll(&mut surface, UiPoint::new(20.0, 76.0), 10_000.0);
    let boundary = dispatch_table_scroll(&mut surface, UiPoint::new(20.0, 76.0), 10_000.0);

    assert_eq!(first.binding_reports.len(), 1);
    assert!(boundary.binding_reports.is_empty());
    assert!(!boundary
        .component_events
        .iter()
        .any(|event| matches!(event.event, UiComponentEvent::SetVisibleRange { .. })));
    assert_table_attr_int(&surface, "viewport_start", 36);
}

#[test]
#[ignore = "manual Runtime09 31-sample/1000-wheel virtual-window p50/p95 evidence"]
fn table_virtual_window_1000_wheel_benchmark_reports_bounded_work_across_logical_scales() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    assert!(
        !crate::core::diagnostics::profiling::feature_enabled()
            && !cfg!(feature = "profiling-tracy"),
        "Runtime09 wall-clock evidence requires profiling and Tracy features to stay disabled"
    );

    let mut expected_scrolling_work = None;
    for logical_count in VIRTUAL_WINDOW_PERF_LOGICAL_COUNTS {
        let mut storm_samples_ns = Vec::with_capacity(VIRTUAL_WINDOW_PERF_SAMPLE_COUNT);
        let mut event_samples_ns =
            Vec::with_capacity(VIRTUAL_WINDOW_PERF_SAMPLE_COUNT * VIRTUAL_WINDOW_PERF_WHEEL_COUNT);
        let mut expected_work = None;

        for _ in 0..VIRTUAL_WINDOW_PERF_SAMPLE_COUNT {
            let mut surface = table_pointer_route_surface(false, false);
            set_table_logical_count(&mut surface, logical_count);
            surface
                .rebuild_dirty(UiSize::new(300.0, 160.0))
                .expect("virtual-window performance surface should prime");

            let storm_started = std::time::Instant::now();
            let mut work = VirtualWindowWheelWork::default();
            for wheel_index in 0..VIRTUAL_WINDOW_PERF_WHEEL_COUNT {
                let scroll_delta = if wheel_index % 2 == 0 { 24.0 } else { -24.0 };
                let event_started = std::time::Instant::now();
                let result =
                    dispatch_table_scroll(&mut surface, UiPoint::new(20.0, 76.0), scroll_delta);
                let rebuild = surface
                    .rebuild_dirty(UiSize::new(300.0, 160.0))
                    .expect("virtual-window wheel rebuild should succeed");
                event_samples_ns.push(event_started.elapsed().as_nanos());

                work.transaction_count += result.binding_reports.len();
                work.component_event_count += result
                    .component_events
                    .iter()
                    .filter(|event| {
                        event.target == UiNodeId::new(2)
                            && matches!(&event.event, UiComponentEvent::SetVisibleRange { .. })
                    })
                    .count();
                work.binding_update_count += result
                    .binding_reports
                    .iter()
                    .map(|report| report.updates.len())
                    .sum::<usize>();
                work.dirty_node_count += rebuild.dirty_node_count;
                work.layout_visited_node_count += rebuild.layout_visited_node_count;
                work.arranged_outer_node_visit_count += rebuild.arranged_outer_node_visit_count;
                work.hit_grid_outer_node_visit_count += rebuild.hit_grid_outer_node_visit_count;
                work.render_outer_node_visit_count += rebuild.render_outer_node_visit_count;
                work.render_command_rebuilt_count += rebuild.render_command_rebuilt_count;
                work.render_command_reused_count += rebuild.render_command_reused_count;
            }
            storm_samples_ns.push(storm_started.elapsed().as_nanos());

            let expected_transaction_count = if logical_count > 4 {
                VIRTUAL_WINDOW_PERF_WHEEL_COUNT
            } else {
                0
            };
            assert_eq!(work.transaction_count, expected_transaction_count);
            assert_eq!(work.component_event_count, expected_transaction_count);
            if let Some(expected) = expected_work {
                assert_eq!(work, expected, "work counters must be sample-stable");
            } else {
                expected_work = Some(work);
            }
        }

        let work = expected_work.expect("at least one performance sample should run");
        if logical_count > 4 {
            if let Some(expected) = expected_scrolling_work {
                assert_eq!(
                    work, expected,
                    "virtual-window work must stay constant as logical row count grows"
                );
            } else {
                expected_scrolling_work = Some(work);
            }
        }
        let (storm_p50_ns, storm_p95_ns) = p50_p95(&mut storm_samples_ns);
        let (event_p50_ns, event_p95_ns) = p50_p95(&mut event_samples_ns);
        println!(
            "runtime09_virtual_window_wheel logical_count={logical_count} samples={} \
             wheel_events_per_sample={} transactions_per_sample={} \
             component_events_per_sample={} binding_updates_per_sample={} \
             dirty_nodes_per_sample={} layout_visited_per_sample={} \
             arranged_outer_visits_per_sample={} hit_grid_outer_visits_per_sample={} \
             render_outer_visits_per_sample={} render_commands_rebuilt_per_sample={} \
             render_commands_reused_per_sample={} storm_p50_ns={storm_p50_ns} \
             storm_p95_ns={storm_p95_ns} event_p50_ns={event_p50_ns} \
             event_p95_ns={event_p95_ns}",
            VIRTUAL_WINDOW_PERF_SAMPLE_COUNT,
            VIRTUAL_WINDOW_PERF_WHEEL_COUNT,
            work.transaction_count,
            work.component_event_count,
            work.binding_update_count,
            work.dirty_node_count,
            work.layout_visited_node_count,
            work.arranged_outer_node_visit_count,
            work.hit_grid_outer_node_visit_count,
            work.render_outer_node_visit_count,
            work.render_command_rebuilt_count,
            work.render_command_reused_count,
        );
    }
}

fn set_table_logical_count(surface: &mut UiSurface, logical_count: i64) {
    let metadata = surface
        .tree
        .node_mut(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_mut())
        .expect("table metadata should exist");
    metadata
        .attributes
        .insert("row_count".to_string(), toml::Value::Integer(logical_count));
    metadata
        .attributes
        .insert("rowCount".to_string(), toml::Value::Integer(logical_count));
}

fn p50_p95(samples_ns: &mut [u128]) -> (u128, u128) {
    samples_ns.sort_unstable();
    let p50_ns = samples_ns[samples_ns.len() / 2];
    let p95_index = (samples_ns.len() * 95).div_ceil(100) - 1;
    (p50_ns, samples_ns[p95_index])
}
