use crate::ui::layouts::windows::workbench_host_window::RuntimeDiagnosticsPanePayload;

use super::runtime_diagnostics_status_lines;

#[test]
fn hybrid_gi_priority_lines_keep_render_frame_status_in_the_visible_group() {
    let payload = RuntimeDiagnosticsPanePayload {
        summary: "1 runtime systems available".to_string(),
        render_status: "Render: wgpu(vulkan) (1 viewports, 42 frames)".to_string(),
        physics_status: "Physics: unavailable".to_string(),
        animation_status: "Animation: unavailable".to_string(),
        detail_items: vec![
            "Hybrid GI active probes: 0".to_string(),
            "Hybrid GI fallback: baked-lighting-unavailable".to_string(),
            "Hybrid GI budgets: trace=64, cards=256, voxels=64".to_string(),
            "Hybrid GI effective: profile=indoor-static, mode=dynamic-only, quality=high"
                .to_string(),
            "Virtual Geometry Debug: unavailable".to_string(),
        ],
        ui_debug_reflector_summary: String::new(),
        ui_debug_reflector_nodes: Vec::new(),
        ui_debug_reflector_details: Vec::new(),
        ui_debug_reflector_sections: Vec::new(),
        ui_debug_reflector_export_status: String::new(),
        ui_debug_reflector_overlay_primitives: Vec::new(),
        ui_debug_reflector_has_active_snapshot: false,
    };

    assert_eq!(
        &runtime_diagnostics_status_lines(&payload)[..5],
        [
            "Hybrid GI effective: profile=indoor-static, mode=dynamic-only, quality=high",
            "Hybrid GI budgets: trace=64, cards=256, voxels=64",
            "Hybrid GI fallback: baked-lighting-unavailable",
            "Render: wgpu(vulkan) (1 viewports, 42 frames)",
            "Hybrid GI active probes: 0",
        ]
    );
}

#[test]
fn optimization_batch_do_status_lines_avoid_intermediate_buckets() {
    let source = include_str!("../runtime_diagnostics.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("runtime diagnostics production source");
    assert!(production.contains("filter_map(|item|"));
    assert!(!production.contains("let mut primary ="));
    assert!(!production.contains("let mut active_probes ="));
    assert!(!production.contains("let mut remaining ="));
}

#[test]
fn debug_reflector_refresh_reads_published_frame_without_surface_rebuild() {
    let source = include_str!("../runtime_diagnostics.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("runtime diagnostics production source");
    assert!(production.contains("pane.body_surface_frame.as_ref()"));
    assert!(!production.contains("UiSurface::new("));
    assert!(!production.contains("surface.rebuild()"));
    assert!(!production.contains("runtime_diagnostics_debug_surface_frame"));
    let live_projection = production
        .split("fn runtime_debug_reflector_nodes_from_model")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn runtime_debug_reflector_nodes_from_parts")
                .next()
        })
        .expect("live reflector model projection");
    assert!(live_projection.contains("RuntimeDebugReflectorNodeWriter::new"));
    assert!(!live_projection.contains("section_display_lines()"));
    assert!(!live_projection.contains("collect::<Vec<_>>()"));
    assert!(!live_projection.contains("node_labels"));
}

#[test]
#[ignore = "release-only alternating p95 performance gate"]
fn optimization_batch_do_status_line_projection_p95() {
    use std::hint::black_box;
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const ITEMS_PER_SAMPLE: usize = 512;
    let payload = benchmark_payload(ITEMS_PER_SAMPLE);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_status_lines(&payload, true));
            optimized_samples.push(measure_status_lines(&payload, false));
        } else {
            optimized_samples.push(measure_status_lines(&payload, false));
            legacy_samples.push(measure_status_lines(&payload, true));
        }
    }

    let legacy_p95 = p95(&mut legacy_samples);
    let optimized_p95 = p95(&mut optimized_samples);
    println!(
        "EDITOR351_STATUS_LINE_SINGLE_BUFFER_BENCH_V1 items={ITEMS_PER_SAMPLE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "status line projection p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );

    fn measure_status_lines(payload: &RuntimeDiagnosticsPanePayload, legacy: bool) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..32 {
            let lines = if legacy {
                legacy_status_lines(payload)
            } else {
                runtime_diagnostics_status_lines(payload)
            };
            checksum = checksum.wrapping_add(lines.iter().map(|line| line.len()).sum::<usize>());
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn legacy_status_lines(payload: &RuntimeDiagnosticsPanePayload) -> Vec<&str> {
        const PRIMARY: [&str; 3] = [
            "Hybrid GI effective:",
            "Hybrid GI budgets:",
            "Hybrid GI fallback:",
        ];
        let mut primary = [Vec::new(), Vec::new(), Vec::new()];
        let mut active_probes = Vec::new();
        let mut remaining = Vec::new();
        for item in &payload.detail_items {
            if let Some(index) = PRIMARY.iter().position(|prefix| item.starts_with(prefix)) {
                primary[index].push(item.as_str());
            } else if item.starts_with("Hybrid GI active probes:") {
                active_probes.push(item.as_str());
            } else {
                remaining.push(item.as_str());
            }
        }
        let mut lines = Vec::with_capacity(payload.detail_items.len() + 3);
        for bucket in primary {
            lines.extend(bucket);
        }
        lines.push(payload.render_status.as_str());
        lines.extend(active_probes);
        lines.extend([
            payload.physics_status.as_str(),
            payload.animation_status.as_str(),
        ]);
        lines.extend(remaining);
        lines
    }

    fn benchmark_payload(item_count: usize) -> RuntimeDiagnosticsPanePayload {
        let detail_items = (0..item_count)
            .map(|index| match index % 8 {
                0 => format!("Hybrid GI effective: profile-{index}"),
                1 => format!("Hybrid GI budgets: trace={index}"),
                2 => format!("Hybrid GI fallback: reason-{index}"),
                3 => format!("Hybrid GI active probes: {index}"),
                _ => format!("Virtual Geometry Debug: item-{index}"),
            })
            .collect();
        RuntimeDiagnosticsPanePayload {
            summary: "runtime diagnostics".to_string(),
            render_status: "Render: benchmark".to_string(),
            physics_status: "Physics: benchmark".to_string(),
            animation_status: "Animation: benchmark".to_string(),
            detail_items,
            ui_debug_reflector_summary: String::new(),
            ui_debug_reflector_nodes: Vec::new(),
            ui_debug_reflector_details: Vec::new(),
            ui_debug_reflector_sections: Vec::new(),
            ui_debug_reflector_export_status: String::new(),
            ui_debug_reflector_overlay_primitives: Vec::new(),
            ui_debug_reflector_has_active_snapshot: false,
        }
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
