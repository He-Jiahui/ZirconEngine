use super::sources::HotspotInventorySources;

pub(super) fn assert_profiling_trace_and_render_diversion(sources: &HotspotInventorySources) {
    assert_profiling_build_tooling(sources);
    assert_trace_and_schedule_docs(sources);
    assert_render_diversion_docs(sources);
}

fn assert_profiling_build_tooling(sources: &HotspotInventorySources) {
    for required_profiling_build_anchor in [
        "#### 切片 0.2 profiling 构建超时破解",
        "profile.profiling",
        "profiling-tracy",
        "profiling-chrome",
        "python tools/zircon_build.py --targets runtime",
        "./tools/dev-fast-build.ps1 -Profile client -Action check",
        "profiling_build_tooling_static_passed_cargo_deferred_active_lanes",
    ] {
        assert!(
            sources
                .runtime_07_plan
                .contains(required_profiling_build_anchor)
                || sources
                    .runtime_index
                    .contains(required_profiling_build_anchor)
                || sources
                    .hotspot_doc
                    .contains(required_profiling_build_anchor)
                || sources
                    .build_tool_doc
                    .contains(required_profiling_build_anchor)
                || sources
                    .profiling_doc
                    .contains(required_profiling_build_anchor),
            "Runtime 07 profiling build tooling should retain `{required_profiling_build_anchor}`"
        );
    }

    for required_cargo_profile_anchor in [
        "[profile.profiling]",
        "inherits = \"release\"",
        "debug = true",
        "strip = false",
    ] {
        assert!(
            sources
                .root_manifest
                .contains(required_cargo_profile_anchor),
            "root Cargo.toml should retain profiling profile anchor `{required_cargo_profile_anchor}`"
        );
    }

    for required_runtime_feature_anchor in [
        "profiling = []",
        "profiling-chrome = [\"profiling\"]",
        "profiling-tracy = [\"profiling\", \"dep:tracing-subscriber\", \"dep:tracing-tracy\"]",
        "profiling-memory = [\"profiling\"]",
    ] {
        assert!(
            sources
                .runtime_manifest
                .contains(required_runtime_feature_anchor),
            "zircon_runtime Cargo.toml should retain profiling feature anchor `{required_runtime_feature_anchor}`"
        );
    }

    for required_zircon_build_anchor in [
        "MODES = (\"debug\", \"release\", \"profiling\")",
        "TARGET_FEATURES = (\"target-client\", \"target-server\", \"target-editor-host\")",
        "def feature_arg_for_target(self, target_feature: str) -> str:",
        "parser.add_argument(",
        "--runtime-features",
        "--mode profiling is not supported for the hub/Tauri target.",
        "command.extend([\"--profile\", \"profiling\"])",
        "python tools/zircon_build.py --targets runtime --out E:\\builds\\zircon-smoke --mode profiling --runtime-features target-client,profiling,profiling-tracy --dry-run",
    ] {
        assert!(
            sources.zircon_build.contains(required_zircon_build_anchor)
                || sources.build_tool_doc.contains(required_zircon_build_anchor),
            "tools/zircon_build.py profiling path should retain `{required_zircon_build_anchor}`"
        );
    }

    for required_dev_fast_build_anchor in [
        "[ValidateSet(\"debug\", \"release\", \"profiling\")]",
        "[string]$CargoProfile = \"debug\"",
        "$CargoProfile -eq \"profiling\"",
        "$args.Add(\"--profile\")",
        "$args.Add(\"profiling\")",
        "./tools/dev-fast-build.ps1 -Profile client -Action check -Package zircon_runtime -CargoProfile profiling -FeatureOverride \"target-client profiling profiling-tracy\"",
    ] {
        assert!(
            sources
                .dev_fast_build
                .contains(required_dev_fast_build_anchor)
                || sources.build_tool_doc.contains(required_dev_fast_build_anchor)
                || sources.profiling_doc.contains(required_dev_fast_build_anchor),
            "tools/dev-fast-build.ps1 profiling path should retain `{required_dev_fast_build_anchor}`"
        );
    }
}

fn assert_trace_and_schedule_docs(sources: &HotspotInventorySources) {
    for required_trace_export_anchor in [
        "render_direct_runtime_frame_trace_export_static_passed_profile_timeout_fps_pending",
        "direct_runtime_frame_submit_exports_perfetto_trace_artifacts",
        "PROFILE_TIMELINE_PERFETTO_FILE",
        "timeline.perfetto.json",
        "runtime-frame-f3-trace-export",
    ] {
        assert!(
            sources.runtime_07_plan.contains(required_trace_export_anchor)
                || sources.runtime_index.contains(required_trace_export_anchor)
                || sources.render_index.contains(required_trace_export_anchor)
                || sources.hotspot_doc.contains(required_trace_export_anchor)
                || sources.profiling_doc.contains(required_trace_export_anchor)
                || sources.render_profiling.contains(required_trace_export_anchor),
            "Runtime 07 F3 direct runtime-frame trace export should retain `{required_trace_export_anchor}`"
        );
    }

    for required_schedule_doc_anchor in [
        "runtime_frame_schedule_stage",
        "SceneScheduleRunner",
        "stage-level span",
    ] {
        assert!(
            sources
                .runtime_07_plan
                .contains(required_schedule_doc_anchor)
                || sources.runtime_index.contains(required_schedule_doc_anchor)
                || sources.hotspot_doc.contains(required_schedule_doc_anchor)
                || sources
                    .dynamic_session_doc
                    .contains(required_schedule_doc_anchor)
                || sources.ecs_doc.contains(required_schedule_doc_anchor)
                || sources
                    .architecture_review
                    .contains(required_schedule_doc_anchor),
            "Runtime 07 schedule span docs should retain `{required_schedule_doc_anchor}`"
        );
    }

    for required_review_anchor in [
        "Runtime 07 Hotspot Inventory Guard",
        "zircon_runtime/src/scene/ecs/schedule_runner.rs",
        "runtime_frame_schedule_stage.<SystemStage>",
        "SceneScheduleRunner",
        "stage-level span",
    ] {
        assert!(
            sources
                .architecture_review
                .contains(required_review_anchor),
            "runtime architecture review should retain Runtime 07 stage-span anchor `{required_review_anchor}`"
        );
    }
}

fn assert_render_diversion_docs(sources: &HotspotInventorySources) {
    for required_render_anchor in [
        "230 draws",
        "231 pre-draw",
        "31 render passes",
        "render 计划 02/04",
        "Runtime 07 M2 is not allowed to fix render submission",
    ] {
        assert!(
            sources.runtime_07_plan.contains(required_render_anchor)
                || sources.hotspot_doc.contains(required_render_anchor),
            "Runtime 07 plan/docs should retain render diversion anchor `{required_render_anchor}`"
        );
    }
}
