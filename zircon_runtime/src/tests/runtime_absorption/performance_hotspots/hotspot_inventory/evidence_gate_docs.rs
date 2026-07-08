use super::sources::HotspotInventorySources;

pub(super) fn assert_evidence_gate_docs(sources: &HotspotInventorySources) {
    let runtime_07_plan = sources.runtime_07_plan;
    let runtime_index = sources.runtime_index;
    let hotspot_doc = sources.hotspot_doc;
    let animation_doc = sources.animation_doc;
    let diagnostics_doc = sources.diagnostics_doc;
    let interface_profiling = sources.interface_profiling;
    let profiling_counter_hotspot = sources.profiling_counter_hotspot;
    let profiling_export = sources.profiling_export;
    let profiling_mod = sources.profiling_mod;
    let profiling_doc = sources.profiling_doc;

    for required_plan_anchor in [
        "M1 | 1.3 热点清单",
        "hotspot_inventory.md",
        "inventory_scaffold_static_passed_pending_authoritative_values",
        "无权威 runtime 数值不得进入 M2",
        "render 计划 02/04",
    ] {
        assert!(
            runtime_07_plan.contains(required_plan_anchor)
                || runtime_index.contains(required_plan_anchor),
            "Runtime 07 plan/index should record hotspot inventory anchor `{required_plan_anchor}`"
        );
    }

    assert!(
        !runtime_07_plan.contains("热点清单 top3：__"),
        "Runtime 07 should not leave the M1.3 hotspot inventory placeholder untouched"
    );

    for required_doc_anchor in [
        "Evidence Gate",
        "No Runtime 07 M2 optimization slice may start from an unmeasured suspicion",
        "Authoritative Top List",
        "Pending authoritative runtime sample",
        "Render-Plan Diversions",
        "vkCmdCopyBuffer",
        "Runtime 07 M2 is not allowed to fix render submission",
        "Candidate Evidence Matrix",
        "frame_extract_rebuild_skips_unchanged_entities",
        "query_state_reuses_archetype_matches_across_unchanged_frames",
        "change_detection_scan_skips_unmarked_archetypes",
        "asset.worker.budgeted_threads",
        "AnimationSceneFrameDiagnostics",
        "animation.scene.scanned_entities",
        "animation.scene.output_poses",
        "animation_scene_frame_diagnostics_static_passed_cargo_deferred",
        "CounterHotspotReport",
        "counter_hotspots.json",
    ] {
        assert!(
            hotspot_doc.contains(required_doc_anchor)
                || animation_doc.contains(required_doc_anchor)
                || diagnostics_doc.contains(required_doc_anchor),
            "Runtime 07 docs should keep evidence gate anchor `{required_doc_anchor}`"
        );
    }

    for required_counter_hotspot_anchor in [
        "PROFILE_COUNTER_HOTSPOTS_FILE",
        "pub struct CounterHotspotReport",
        "pub struct CounterHotspotEntry",
        "pub fn analyze_counter_hotspots",
        "counter_hotspots.json",
        "ProfileControlResponse.counter_hotspot_report",
        "summary.push_str(\"\\n## Counter Hotspots\\n\");",
        "response.counter_hotspot_report = Some(report.counter_hotspots);",
    ] {
        assert!(
            interface_profiling.contains(required_counter_hotspot_anchor)
                || profiling_counter_hotspot.contains(required_counter_hotspot_anchor)
                || profiling_export.contains(required_counter_hotspot_anchor)
                || profiling_mod.contains(required_counter_hotspot_anchor)
                || hotspot_doc.contains(required_counter_hotspot_anchor)
                || profiling_doc.contains(required_counter_hotspot_anchor),
            "Runtime 07 generic profiling counter hotspot export should retain `{required_counter_hotspot_anchor}`"
        );
    }
}
