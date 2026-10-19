//! 性能热点的结构守卫核对拥有者、文件预算和证据文档。保存同组守卫使用的局部数据或辅助变换。
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
        "python tools/build/zircon_build.py --targets runtime",
        "./tools/dev/dev-fast-build.ps1 -Profile client -Action check",
        "profiling_build_tooling_static_passed_cargo_deferred_active_lanes",
    ] {
        let historical_source = match required_profiling_build_anchor {
            "profiling_build_tooling_static_passed_cargo_deferred_active_lanes" => {
                sources.runtime_07_archive
            }
            _ => sources.runtime_07_fixed_plan,
        };
        assert!(
            historical_source.contains(required_profiling_build_anchor),
            "fixed Runtime 07 profiling evidence should retain `{required_profiling_build_anchor}`"
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
        "profiling = [\"zr_resource/profiling\"]",
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

    let arguments = include_str!("../../../../../tools/build/zircon_build_arguments.py");
    let config = include_str!("../../../../../tools/build/zircon_build_config.py");
    for (owner, required) in [
        (
            sources.zircon_build,
            "from .zircon_build_arguments import (",
        ),
        (
            sources.zircon_build,
            "from .zircon_build_config import BuildConfig",
        ),
        (sources.zircon_build, "args = parse_args(argv)"),
        (
            arguments,
            "MODES = (\"debug\", \"release\", \"profiling\", \"shipping\", \"shipping-symbols\")",
        ),
        (arguments, "parser.add_argument("),
        (arguments, "--runtime-features"),
        (
            config,
            "TARGET_FEATURES = (\"target-client\", \"target-server\", \"target-editor-host\")",
        ),
        (
            config,
            "def feature_arg_for_target(self, target_feature: str) -> str:",
        ),
        (config, "if self.mode == \"profiling\":"),
        (config, "return \"profiling\""),
        (
            config,
            "if self.mode in {\"shipping\", \"shipping-symbols\"}:",
        ),
        (config, "return self.mode"),
        (
            sources.zircon_build,
            "--mode profiling is not supported for the hub/Tauri target.",
        ),
        (
            sources.zircon_build,
            "command.extend([\"--profile\", \"profiling\"])",
        ),
    ] {
        assert!(
            owner.contains(required),
            "current profiling owner must retain {required}"
        );
    }
    for required in [
        "[ValidateSet('debug', 'release', 'profiling')][string]$CargoProfile = 'debug'",
        "$CargoProfile -eq 'profiling'",
        "throw 'A requested option has no ordinary Jenkins validate recipe; no local fallback is available.'",
        "$taskArguments = @('-B', '-X', 'utf8', $taskEntry, 'cargo', '--repo-root', $taskRepo)",
        "& $taskPython @taskArguments",
        "exit 2",
    ] {
        assert!(sources.dev_fast_build.contains(required), "Jenkins wrapper policy: {required}");
    }
    let rejection = sources
        .dev_fast_build
        .find("$CargoProfile -eq 'profiling'")
        .expect("profiling reject condition");
    let pending = sources.dev_fast_build[rejection..]
        .find("throw 'A requested option has no ordinary Jenkins validate recipe; no local fallback is available.'")
        .expect("explicit missing-recipe rejection") + rejection;
    let submission = sources
        .dev_fast_build
        .find("& $taskPython @taskArguments")
        .expect("ordinary Jenkins submission");
    assert!(rejection < pending && pending < submission);
    assert!(!sources.dev_fast_build.contains("$args.Add(\"--profile\")"));
    assert!(!sources.dev_fast_build.contains("$args.Add(\"profiling\")"));
    let historical_command = "./tools/dev/dev-fast-build.ps1 -Profile client -Action check -Package zircon_runtime -CargoProfile profiling -FeatureOverride \"target-client profiling profiling-tracy\"";
    assert!(sources.runtime_07_archive.contains(historical_command));
    for required in [
        "ordinary Jenkins profiling recipe: pending",
        "formal profiling profile build: pending",
        "historical command identities are not executable admission",
    ] {
        assert!(
            sources.runtime_07_plan.contains(required),
            "profiling pending/identity: {required}"
        );
    }
    let retired_example = "python tools/build/zircon_build.py --targets runtime --out E:\\builds\\zircon-smoke --mode profiling --runtime-features target-client,profiling,profiling-tracy --dry-run";
    for current in [
        sources.zircon_build,
        sources.build_tool_doc,
        sources.runtime_07_plan,
    ] {
        assert!(
            !current.contains(retired_example),
            "retired non-physical output example must not be recommended"
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
                || sources.runtime_07_archive.contains(required_trace_export_anchor)
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
                || sources
                    .runtime_07_archive
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
                || sources.runtime_07_archive.contains(required_render_anchor)
                || sources.hotspot_doc.contains(required_render_anchor),
            "Runtime 07 plan/docs should retain render diversion anchor `{required_render_anchor}`"
        );
    }
}
