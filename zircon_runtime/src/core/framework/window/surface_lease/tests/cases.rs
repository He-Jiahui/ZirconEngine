use std::collections::HashSet;
use std::hint::black_box;
use std::num::NonZeroU32;
use std::time::Instant;

use zircon_runtime_interface::ZrRuntimeViewportHandle;

use super::{SurfaceLeaseError, SurfaceLeaseRegistry, SurfaceLeaseRequest};
use crate::core::framework::window::{
    DisplayId, DisplayKind, DisplayLogicalRect, DisplayObservation, DisplayOrientation,
    DisplayOutputCapabilities, DisplayPhysicalRect, DisplaySnapshot, DisplayTopologyGeneration,
    DisplayTopologySnapshot, WindowId, WindowRegistryId,
};

// 这些 fixture 用内存中的单输出拓扑隔离 lease 状态机；窗口注册表 admission 则由 driver 集成测试覆盖。
fn window(slot: u32) -> WindowId {
    WindowId::new(
        WindowRegistryId::new(17).expect("fixture registry identity is nonzero"),
        slot,
        NonZeroU32::MIN,
    )
}

fn viewport(raw: u64) -> ZrRuntimeViewportHandle {
    ZrRuntimeViewportHandle::new(raw)
}

fn output(key: &str) -> DisplayId {
    DisplayId::new(DisplayKind::PhysicalOutput, key).expect("fixture output identity is valid")
}

fn topology(generation: u64, output: DisplayId) -> DisplayTopologySnapshot {
    let snapshot = DisplaySnapshot::new(
        output.clone(),
        DisplayObservation {
            physical_bounds: DisplayPhysicalRect::new(
                0,
                0,
                NonZeroU32::new(1920).expect("fixture output width is nonzero"),
                NonZeroU32::new(1080).expect("fixture output height is nonzero"),
            ),
            usable_logical_bounds: DisplayLogicalRect::new(0.0, 0.0, 1920.0, 1080.0)
                .expect("fixture output logical bounds are valid"),
            scale_factor: 1.0,
            refresh_rate_millihertz: None,
            orientation: DisplayOrientation::Landscape,
            safe_area: None,
            output_capabilities: DisplayOutputCapabilities::default(),
        },
    )
    .expect("fixture display snapshot is valid");
    DisplayTopologySnapshot::new(
        DisplayTopologyGeneration::new(generation).expect("fixture topology generation is nonzero"),
        vec![snapshot],
        Some(output),
    )
    .expect("fixture display topology is valid")
}

fn request(
    window: WindowId,
    viewport: ZrRuntimeViewportHandle,
    output: DisplayId,
    generation: DisplayTopologyGeneration,
) -> SurfaceLeaseRequest {
    SurfaceLeaseRequest::new(window, viewport, output, generation)
}

#[test]
fn prepare_keeps_current_lease_routable_until_matching_candidate_publishes() {
    let output = output("edid:panel-a");
    let topology = topology(4, output.clone());
    let mut registry = SurfaceLeaseRegistry::default();
    let request = request(window(0), viewport(7), output, topology.generation());

    let first_prepared = registry
        .prepare(request.clone(), &topology)
        .expect("first prepare succeeds");
    let first = registry
        .publish(&first_prepared, &topology)
        .expect("first candidate publishes");
    let active = first.current().clone();
    assert_eq!(first.retired(), None);

    // replacement 尚未 publish 时旧 surface 仍承接路由；重复 prepare 必须被挡住。
    let replacement = registry
        .prepare(request, &topology)
        .expect("replacement prepare succeeds");
    assert_eq!(registry.active(&active), Ok(()));
    assert_eq!(registry.active_count(), 1);
    assert!(matches!(
        registry.prepare(replacement.request().clone(), &topology),
        Err(SurfaceLeaseError::ReplacementInFlight { .. })
    ));

    let published = registry
        .publish(&replacement, &topology)
        .expect("matching replacement publishes");
    assert_eq!(published.retired(), Some(&active));
    assert_ne!(published.current().generation(), active.generation());
    assert_eq!(
        registry.active(&active),
        Err(SurfaceLeaseError::StaleLease { lease: active })
    );
    assert_eq!(registry.active(published.current()), Ok(()));
}

#[test]
fn topology_mismatch_and_unknown_output_fail_without_reserving_a_lease() {
    let known_output = output("edid:panel-a");
    let topology = topology(2, known_output.clone());
    let mut registry = SurfaceLeaseRegistry::default();

    let stale_request = request(
        window(0),
        viewport(1),
        known_output.clone(),
        DisplayTopologyGeneration::new(1).expect("fixture topology generation is nonzero"),
    );
    assert_eq!(
        registry.prepare(stale_request, &topology),
        Err(SurfaceLeaseError::TopologyGenerationMismatch {
            requested: DisplayTopologyGeneration::new(1)
                .expect("fixture topology generation is nonzero"),
            observed: topology.generation(),
        })
    );

    let missing_output = output("edid:missing");
    let missing_request = request(
        window(0),
        viewport(1),
        missing_output.clone(),
        topology.generation(),
    );
    assert_eq!(
        registry.prepare(missing_request, &topology),
        Err(SurfaceLeaseError::OutputUnavailable {
            output: missing_output,
            topology_generation: topology.generation(),
        })
    );
    assert_eq!(registry.active_count(), 0);
    assert_eq!(registry.preparing_count(), 0);
}

#[test]
fn publish_rejects_a_candidate_prepared_against_an_older_topology_generation() {
    let output = output("edid:panel-a");
    let prepared_topology = topology(1, output.clone());
    let current_topology = topology(2, output.clone());
    let mut registry = SurfaceLeaseRegistry::default();
    let prepared = registry
        .prepare(
            request(
                window(0),
                viewport(1),
                output,
                prepared_topology.generation(),
            ),
            &prepared_topology,
        )
        .expect("candidate prepares against the observed topology");

    assert_eq!(
        registry.publish(&prepared, &current_topology),
        Err(SurfaceLeaseError::TopologyGenerationMismatch {
            requested: prepared_topology.generation(),
            observed: current_topology.generation(),
        })
    );
    assert_eq!(registry.preparing_count(), 1);
    assert_eq!(registry.active_count(), 0);
    // 拓扑变化只拒绝提交，不吞掉准备态；调用方仍可显式取消候选，继续清理对应图形资源。
    registry
        .cancel(&prepared)
        .expect("stale preparation remains cancelable after rejected publication");
}

#[test]
fn cancel_restores_the_previous_active_lease_without_changing_its_generation() {
    let output = output("edid:panel-a");
    let topology = topology(1, output.clone());
    let mut registry = SurfaceLeaseRegistry::default();
    let request = request(window(0), viewport(3), output, topology.generation());
    let first_prepared = registry
        .prepare(request.clone(), &topology)
        .expect("first prepare succeeds");
    let active = registry
        .publish(&first_prepared, &topology)
        .expect("first candidate publishes")
        .current()
        .clone();
    let replacement = registry
        .prepare(request, &topology)
        .expect("replacement prepare succeeds");

    registry
        .cancel(&replacement)
        .expect("prepared candidate cancels");
    assert_eq!(registry.active(&active), Ok(()));
    assert_eq!(registry.active_count(), 1);
    assert_eq!(registry.preparing_count(), 0);
    assert_eq!(
        registry.publish(&replacement, &topology),
        Err(SurfaceLeaseError::StaleLease {
            lease: replacement.candidate().clone(),
        })
    );
}

#[test]
fn window_retirement_revokes_all_active_leases_before_their_final_removal() {
    let output = output("edid:panel-a");
    let topology = topology(8, output.clone());
    let mut registry = SurfaceLeaseRegistry::default();
    let window = window(0);
    let first_request = request(window, viewport(9), output.clone(), topology.generation());
    let second_request = request(window, viewport(3), output, topology.generation());
    let first_prepared = registry
        .prepare(first_request, &topology)
        .expect("first prepare succeeds");
    let first = registry
        .publish(&first_prepared, &topology)
        .expect("first candidate publishes")
        .current()
        .clone();
    let second_prepared = registry
        .prepare(second_request, &topology)
        .expect("second prepare succeeds");
    let second = registry
        .publish(&second_prepared, &topology)
        .expect("second candidate publishes")
        .current()
        .clone();

    let retiring = registry
        .begin_retire_window(window)
        .expect("all active window leases begin retirement together");
    assert_eq!(retiring, vec![second.clone(), first.clone()]);
    assert_eq!(
        registry.active(&first),
        Err(SurfaceLeaseError::LeaseRetiring {
            lease: first.clone()
        })
    );
    assert_eq!(
        registry.active(&second),
        Err(SurfaceLeaseError::LeaseRetiring {
            lease: second.clone()
        })
    );
    assert!(matches!(
        registry.prepare(first.request().clone(), &topology),
        Err(SurfaceLeaseError::LeaseRetiring { .. })
    ));

    // 两个 viewport 均已不可路由；完成回执模拟图形所有者逐张清理后再释放注册表绑定。
    registry
        .complete_retirement(&first)
        .expect("first retired lease is removed after graphics teardown");
    registry
        .complete_retirement(&second)
        .expect("second retired lease is removed after graphics teardown");
    assert_eq!(registry.active_count(), 0);
}

#[test]
fn plan_all_retirement_deduplicates_windows_with_multiple_viewports() {
    let output = output("edid:panel-a");
    let topology = topology(9, output.clone());
    let mut registry = SurfaceLeaseRegistry::default();
    let shared_window = window(0);
    let first = registry
        .prepare(
            request(
                shared_window,
                viewport(9),
                output.clone(),
                topology.generation(),
            ),
            &topology,
        )
        .and_then(|prepared| registry.publish(&prepared, &topology))
        .expect("first viewport lease publishes")
        .current()
        .clone();
    let second = registry
        .prepare(
            request(shared_window, viewport(3), output, topology.generation()),
            &topology,
        )
        .and_then(|prepared| registry.publish(&prepared, &topology))
        .expect("second viewport lease publishes")
        .current()
        .clone();

    // 计划只做预检和快照，不会提前撤路由；提交由驱动在协调完窗口与图形所有者后执行。
    let plan = registry
        .plan_all_retirement()
        .expect("one unique window should plan both viewport leases");

    assert_eq!(plan.leases(), [second, first]);
    assert_eq!(registry.active_count(), 2);
}

#[test]
fn pending_candidate_prevents_window_retirement_without_revoking_existing_state() {
    let output = output("edid:panel-a");
    let topology = topology(1, output.clone());
    let mut registry = SurfaceLeaseRegistry::default();
    let window = window(0);
    let prepared = registry
        .prepare(
            request(window, viewport(1), output, topology.generation()),
            &topology,
        )
        .expect("candidate prepares");

    assert_eq!(
        registry.begin_retire_window(window),
        Err(SurfaceLeaseError::WindowHasPreparedLease { window })
    );
    assert_eq!(registry.preparing_count(), 1);
    registry
        .cancel(&prepared)
        .expect("candidate cancels after rejected retirement");
}

#[test]
fn viewport_cannot_move_to_another_window_until_the_old_lease_retires() {
    let output = output("edid:panel-a");
    let topology = topology(3, output.clone());
    let mut registry = SurfaceLeaseRegistry::default();
    let first_request = request(
        window(0),
        viewport(7),
        output.clone(),
        topology.generation(),
    );
    let second_request = request(window(1), viewport(7), output, topology.generation());
    let first_prepared = registry
        .prepare(first_request, &topology)
        .expect("first viewport owner prepares");
    let first = registry
        .publish(&first_prepared, &topology)
        .expect("first viewport owner publishes")
        .current()
        .clone();

    assert_eq!(
        registry.prepare(second_request.clone(), &topology),
        Err(SurfaceLeaseError::ViewportAlreadyBound {
            viewport: viewport(7),
            window: window(0),
        })
    );
    assert_eq!(registry.active(&first), Ok(()));

    registry
        .begin_retirement(&first)
        .expect("old viewport owner begins graphics retirement");
    assert_eq!(
        registry.prepare(second_request.clone(), &topology),
        Err(SurfaceLeaseError::LeaseRetiring {
            lease: first.clone(),
        })
    );
    registry
        .complete_retirement(&first)
        .expect("old viewport owner completes graphics retirement");
    assert_eq!(
        registry
            .prepare(second_request, &topology)
            .expect("viewport may bind after the prior lease is fully retired")
            .candidate()
            .window(),
        window(1)
    );
}

#[test]
fn canceling_an_initial_preparation_releases_its_viewport_owner() {
    let output = output("edid:panel-a");
    let topology = topology(3, output.clone());
    let mut registry = SurfaceLeaseRegistry::default();
    let first = registry
        .prepare(
            request(
                window(0),
                viewport(7),
                output.clone(),
                topology.generation(),
            ),
            &topology,
        )
        .expect("first viewport owner prepares");

    registry
        .cancel(&first)
        .expect("unpublished candidate cancels cleanly");
    assert_eq!(registry.preparing_count(), 0);
    assert_eq!(registry.active_count(), 0);
    assert_eq!(
        registry
            .prepare(
                request(window(1), viewport(7), output, topology.generation()),
                &topology,
            )
            .expect("canceled viewport binding is reusable by another window")
            .candidate()
            .window(),
        window(1)
    );
}

#[test]
fn optimization_batch_r6_wave7_runtime644_retirement_windows_reserve_entry_bound() {
    // 此源码守卫固定去重收集的预留策略，防止回退为逐个候选窗口扩容。
    let source = include_str!("../registry.rs");
    assert!(source.contains("unique_windows\n            .try_reserve(self.entries.len())"));
    assert!(source.contains("unique_windows.insert(key.window);"));
    assert!(!source.contains("unique_windows.contains(&key.window)"));
    assert!(!source.contains("debug_assert!(inserted)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_r6_wave7_runtime644_retirement_windows_capacity_p95() {
    // 交替执行旧、新策略以减轻顺序偏差；这是需要受管 Windows release 证据的忽略基准。
    const SAMPLE_PAIRS: usize = 17;
    const ENTRIES_PER_SAMPLE: usize = 65_536;
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(runtime644_measure_window_collection(
                ENTRIES_PER_SAMPLE,
                false,
            ));
            optimized_samples.push(runtime644_measure_window_collection(
                ENTRIES_PER_SAMPLE,
                true,
            ));
        } else {
            optimized_samples.push(runtime644_measure_window_collection(
                ENTRIES_PER_SAMPLE,
                true,
            ));
            legacy_samples.push(runtime644_measure_window_collection(
                ENTRIES_PER_SAMPLE,
                false,
            ));
        }
    }

    let legacy_p95 = runtime644_p95(&legacy_samples);
    let optimized_p95 = runtime644_p95(&optimized_samples);
    println!(
        "RUNTIME644_PREALLOCATED_SURFACE_RETIREMENT_WINDOWS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} entries_per_sample={ENTRIES_PER_SAMPLE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(85),
        "preallocated retirement windows must be at least 15% faster at P95"
    );
}

fn runtime644_measure_window_collection(entry_count: usize, optimized: bool) -> u128 {
    // 优化组一次性按 entry 数预留，旧组仅在发现新窗口时逐项扩容；输入窗口在此基准中互不重复。
    let started = Instant::now();
    let mut unique_windows = HashSet::new();
    if optimized {
        unique_windows
            .try_reserve(entry_count)
            .expect("benchmark reservation should fit");
    }
    for slot in 0..entry_count as u32 {
        let current = window(slot);
        if !optimized && !unique_windows.contains(&current) {
            unique_windows
                .try_reserve(1)
                .expect("benchmark reservation should fit");
        }
        unique_windows.insert(current);
    }
    let elapsed = started.elapsed().as_nanos().max(1);
    black_box(unique_windows);
    elapsed
}

fn runtime644_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100) - 1]
}
