use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_backend_types_are_child_owners() {
    let parent = read_runtime_src("core/framework/render/backend_types.rs");
    let handles = read_runtime_src("core/framework/render/backend_types/handles.rs");
    let history = read_runtime_src("core/framework/render/backend_types/history.rs");
    let camera_target = read_runtime_src("core/framework/render/backend_types/camera_target.rs");
    let graph_reports = read_runtime_src("core/framework/render/backend_types/graph_reports.rs");
    let backend_status = read_runtime_src("core/framework/render/backend_types/backend_status.rs");
    let capability = read_runtime_src("core/framework/render/backend_types/capability.rs");
    let command = read_runtime_src("core/framework/render/backend_types/command.rs");
    let quality = read_runtime_src("core/framework/render/backend_types/quality.rs");
    let tests = read_runtime_src("core/framework/render/backend_types/tests.rs");
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let backend_doc = read_repo("docs/zircon_runtime/core/framework/render/backend_types.md");

    assert_contains_all(
        "backend_types parent keeps the facade and RenderStats owner only",
        &parent,
        &[
            "mod camera_target;",
            "mod capability;",
            "mod command;",
            "mod graph_reports;",
            "mod handles;",
            "mod history;",
            "mod quality;",
            "#[cfg(test)]\nmod tests;",
            "pub use camera_target::{",
            "pub use capability::{",
            "pub use command::{",
            "pub use graph_reports::{",
            "pub enum RenderGpuSceneUploadPath",
            "pub struct RenderStats",
        ],
    );

    for moved_owner in [
        "pub struct RenderViewportHandle",
        "pub struct FrameHistoryStatus",
        "pub struct RenderCameraTargetResolutionReport",
        "pub struct RenderGraphTransientPoolReport",
        "pub enum RenderCapabilityKind",
        "pub enum RenderCommand",
        "pub struct RenderQualityProfile",
        "fn history_copy_report_counts_copied_slots_from_slot_flags",
        "fn capability_class_report_splits_default_advanced_and_experimental_requirements",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "backend_types.rs should mount child owners instead of defining {moved_owner}"
        );
    }

    assert_contains_all(
        "backend handles child owns stable opaque IDs",
        &handles,
        &[
            "pub struct RenderViewportHandle",
            "pub struct RenderPipelineHandle",
            "pub struct FrameHistoryHandle",
        ],
    );
    assert_contains_all(
        "backend history child owns history status and copy reports",
        &history,
        &[
            "pub enum FrameHistoryInvalidationReason",
            "pub struct FrameHistoryStatus",
            "pub struct RenderHistoryCopyReport",
        ],
    );
    assert_contains_all(
        "backend camera-target child owns target resolution/writeback/import reports",
        &camera_target,
        &[
            "pub struct RenderCameraTargetResolutionReport",
            "pub enum RenderCameraTargetWritebackStatus",
            "pub struct RenderCameraTargetWritebackReport",
            "pub enum RenderCameraTargetGraphImportStatus",
            "pub struct RenderCameraTargetGraphImportReport",
        ],
    );
    assert_contains_all(
        "backend graph reports child owns graph execution DTOs",
        &graph_reports,
        &[
            "pub struct RenderGraphTransientPoolReport",
            "pub struct RenderGraphExecutionResourceReport",
            "pub struct RenderGraphMaterializationReport",
            "pub struct RenderGraphExecutionAliasReport",
            "pub struct RenderGraphExecutionCoverageReport",
            "pub struct RenderGraphStageExecutionReport",
            "pub struct RenderSceneVelocityReadbackReport",
            "pub enum MotionVectorCameraStatus",
        ],
    );
    assert_contains_all(
        "backend status child owns backend/debugger status DTOs",
        &backend_status,
        &[
            "pub struct RenderingBackendInfo",
            "pub struct GraphicsDebuggerStatus",
        ],
    );
    assert_contains_all(
        "backend capability child owns capability classes and summary checks",
        &capability,
        &[
            "pub enum RenderQueueCapability",
            "pub enum RenderCapabilityKind",
            "pub enum RenderCapabilityClass",
            "pub struct RenderCapabilitySummary",
            "pub fn capability_class_report(",
        ],
    );
    assert_contains_all(
        "backend command child owns command/query/payload/viewport DTOs",
        &command,
        &[
            "pub enum RenderCommand",
            "pub enum RenderQuery",
            "pub enum RenderHybridGiPayloadSource",
            "pub enum RenderVirtualGeometryPayloadSource",
            "pub struct RenderViewportDescriptor",
        ],
    );
    assert_contains_all(
        "backend quality child owns submit quality settings",
        &quality,
        &[
            "pub struct RenderFeatureQualitySettings",
            "pub struct RenderQualityProfile",
        ],
    );
    assert_contains_all(
        "backend tests child owns direct DTO coverage",
        &tests,
        &[
            "fn history_copy_report_counts_copied_slots_from_slot_flags",
            "fn camera_target_writeback_report_separates_copy_and_conversion_debug_markers",
            "fn graph_stage_execution_report_preserves_neutral_counts",
            "fn render_quality_profile_preserves_taa_quality_preset",
            "fn capability_class_report_splits_default_advanced_and_experimental_requirements",
        ],
    );

    for (path, source) in [
        ("core/framework/render/backend_types.rs", parent.as_str()),
        (
            "core/framework/render/backend_types/handles.rs",
            handles.as_str(),
        ),
        (
            "core/framework/render/backend_types/history.rs",
            history.as_str(),
        ),
        (
            "core/framework/render/backend_types/camera_target.rs",
            camera_target.as_str(),
        ),
        (
            "core/framework/render/backend_types/graph_reports.rs",
            graph_reports.as_str(),
        ),
        (
            "core/framework/render/backend_types/backend_status.rs",
            backend_status.as_str(),
        ),
        (
            "core/framework/render/backend_types/capability.rs",
            capability.as_str(),
        ),
        (
            "core/framework/render/backend_types/command.rs",
            command.as_str(),
        ),
        (
            "core/framework/render/backend_types/quality.rs",
            quality.as_str(),
        ),
        (
            "core/framework/render/backend_types/tests.rs",
            tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production/test soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("backend types doc", backend_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Render backend-types owner split",
                "render_backend_types_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "core/framework/render/backend_types.rs",
                "core/framework/render/backend_types/camera_target.rs",
                "core/framework/render/backend_types/capability.rs",
                "core/framework/render/backend_types/graph_reports.rs",
                "runtime_15_render_backend_types_are_child_owners",
            ],
        );
    }
}
