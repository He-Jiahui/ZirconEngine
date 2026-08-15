use super::support::{
    animation_fixture, build_export_fixture, chrome_fixture, module_plugins_fixture,
    pane_body_spec, runtime_diagnostics_fixture,
};
use crate::ui::layouts::windows::workbench_host_window::{
    PanePayload, PanePayloadBuildContext, PerformanceTimelineCaptureControlPayload,
    build_pane_body_presentation,
};
use crate::ui::workbench::view::PanePayloadKind;

#[test]
fn pane_payload_builders_emit_stable_body_metadata_for_first_wave_views() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let chrome = chrome_fixture();
    let animation = animation_fixture();
    let runtime_diagnostics = runtime_diagnostics_fixture();
    let module_plugins = module_plugins_fixture();
    let build_export = build_export_fixture();
    let context = PanePayloadBuildContext::new(&chrome)
        .with_animation_pane(&animation)
        .with_runtime_diagnostics(&runtime_diagnostics)
        .with_module_plugins(&module_plugins)
        .with_build_export(&build_export);

    let cases = [
        (
            "editor.console",
            "res://ui/editor/host/console_body.zui",
            PanePayloadKind::ConsoleV1,
        ),
        (
            "editor.inspector",
            "res://ui/editor/host/inspector_body.zui",
            PanePayloadKind::InspectorV1,
        ),
        (
            "editor.hierarchy",
            "res://ui/editor/host/hierarchy_body.zui",
            PanePayloadKind::HierarchyV1,
        ),
        (
            "editor.animation_sequence",
            "res://ui/editor/host/animation_sequence_body.zui",
            PanePayloadKind::AnimationSequenceV1,
        ),
        (
            "editor.animation_graph",
            "res://ui/editor/host/animation_graph_body.zui",
            PanePayloadKind::AnimationGraphV1,
        ),
        (
            "editor.runtime_diagnostics",
            "res://ui/editor/host/runtime_diagnostics_body.zui",
            PanePayloadKind::RuntimeDiagnosticsV1,
        ),
        (
            "editor.performance_timeline",
            "res://ui/editor/host/performance_timeline_body.zui",
            PanePayloadKind::PerformanceTimelineV1,
        ),
        (
            "editor.module_plugins",
            "res://ui/editor/host/module_plugins_body.zui",
            PanePayloadKind::ModulePluginsV1,
        ),
        (
            "editor.build_export_desktop",
            "res://ui/editor/host/build_export_desktop_body.zui",
            PanePayloadKind::BuildExportV1,
        ),
        (
            "editor.generated_bottom",
            "res://ui/editor/host/generated_bottom_body.zui",
            PanePayloadKind::GeneratedBottomV1,
        ),
    ];

    for (descriptor_id, document_id, payload_kind) in cases {
        let spec = pane_body_spec(descriptor_id);
        let body = build_pane_body_presentation(&spec, &context);

        assert_eq!(body.document_id, document_id);
        assert_eq!(body.payload_kind, payload_kind);
        assert_eq!(body.payload_kind, spec.payload_kind);
        assert_eq!(body.route_namespace, spec.route_namespace);
        assert_eq!(body.interaction_mode, spec.interaction_mode);

        match (descriptor_id, body.payload) {
            ("editor.console", PanePayload::ConsoleV1(payload)) => {
                assert_eq!(payload.status_text, "Console ready");
            }
            ("editor.inspector", PanePayload::InspectorV1(payload)) => {
                assert_eq!(payload.node_id, 7);
                assert_eq!(payload.name, "Root");
                assert_eq!(payload.translation, ["1.0", "2.0", "3.0"]);
            }
            ("editor.hierarchy", PanePayload::HierarchyV1(payload)) => {
                assert_eq!(payload.nodes.len(), 2);
                assert_eq!(payload.nodes[0].node_id, 7);
                assert_eq!(payload.nodes[0].name, "Root");
                assert!(payload.nodes[0].selected);
            }
            ("editor.animation_sequence", PanePayload::AnimationSequenceV1(payload)) => {
                assert_eq!(payload.asset_path, "res://animations/hero.anim");
                assert_eq!(payload.timeline_start_frame, 5);
                assert_eq!(payload.timeline_end_frame, 42);
                assert_eq!(payload.track_items, vec!["Root/Hero:Transform.position"]);
            }
            ("editor.animation_graph", PanePayload::AnimationGraphV1(payload)) => {
                assert_eq!(payload.asset_path, "res://animations/hero.anim");
                assert_eq!(payload.node_items, vec!["Blend"]);
                assert_eq!(payload.state_items, vec!["Idle", "Run"]);
                assert_eq!(payload.transition_items, vec!["Idle -> Run"]);
            }
            ("editor.runtime_diagnostics", PanePayload::RuntimeDiagnosticsV1(payload)) => {
                assert_eq!(payload.summary, "3 runtime systems available");
                assert_eq!(
                    payload.render_status,
                    "Render: wgpu-test (3 viewports, 11 frames)"
                );
                assert_eq!(payload.physics_status, "Physics: jolt (Ready, 120 Hz)");
                assert_eq!(
                    payload.animation_status,
                    "Animation: enabled (graphs on, state machines on)"
                );
                assert!(
                    payload
                        .detail_items
                        .contains(&"Virtual Geometry Debug: available".to_string())
                );
                assert!(
                    payload
                        .detail_items
                        .contains(&"Hybrid GI active probes: 4".to_string())
                );
                assert!(
                    payload
                        .detail_items
                        .contains(&"Profiling: active (1 frames, 1 spans, 1 counters)".to_string())
                );
                assert!(
                    payload
                        .detail_items
                        .contains(&"Profiling over-budget frames: 1".to_string())
                );
                assert_eq!(
                    payload.ui_debug_reflector_summary,
                    "UI Debug Reflector: no active surface snapshot"
                );
                assert_eq!(
                    payload.ui_debug_reflector_details,
                    vec!["Waiting for Runtime Diagnostics to receive a UiSurfaceDebugSnapshot"]
                );
                assert!(
                    payload
                        .ui_debug_reflector_export_status
                        .contains("Export unavailable")
                );
                assert!(payload.ui_debug_reflector_overlay_primitives.is_empty());
                assert!(!payload.ui_debug_reflector_has_active_snapshot);
            }
            ("editor.performance_timeline", PanePayload::PerformanceTimelineV1(payload)) => {
                assert_eq!(
                    payload.summary,
                    "Profiling active: 1 frame, 1 span, 1 counter"
                );
                assert_eq!(payload.session_label, "Session local");
                assert_eq!(payload.output_label, "Output target/zircon-profiles/local");
                assert_eq!(payload.frame_rows.len(), 1);
                assert_eq!(payload.frame_rows[0].stream, "editor");
                assert_eq!(payload.frame_rows[0].name, "retained_host_tick");
                assert_eq!(payload.frame_rows[0].duration_label, "18.00 ms");
                assert_eq!(payload.frame_rows[0].budget_label, "16.67 ms budget");
                assert_eq!(payload.frame_rows[0].budget_usage_label, "108% budget");
                assert!((payload.frame_rows[0].duration_ratio - 1.079_784).abs() < 0.000_1);
                assert_eq!(payload.frame_rows[0].bar_fill_ratio, 1.0);
                assert!((payload.frame_rows[0].budget_marker_ratio - 0.926_111).abs() < 0.000_1);
                assert!(payload.frame_rows[0].over_budget);
                assert_eq!(payload.span_summary_rows.len(), 1);
                assert_eq!(payload.span_summary_rows[0].name, "recompute_if_dirty");
                assert_eq!(payload.span_summary_rows[0].duration_label, "12.00 ms");
                assert_eq!(payload.hotspot_rows.len(), 1);
                assert_eq!(payload.hotspot_rows[0].name, "recompute_if_dirty");
                assert_eq!(payload.hotspot_rows[0].total_label, "12.00 ms total");
                assert!(payload.capture_controls.iter().any(|control| control
                    == &PerformanceTimelineCaptureControlPayload {
                        label: "Stop Capture".to_string(),
                        action_id: "workbench.performance_timeline.capture.stop".to_string(),
                        enabled: true,
                    }));
                assert!(
                    payload
                        .capture_controls
                        .iter()
                        .any(|control| control.action_id
                            == "workbench.performance_timeline.report.export")
                );
            }
            ("editor.module_plugins", PanePayload::ModulePluginsV1(payload)) => {
                assert_eq!(payload.diagnostics, "plugin catalog ready");
                assert_eq!(payload.plugins.len(), 1);
                assert_eq!(payload.plugins[0].plugin_id, "physics");
                assert_eq!(payload.plugins[0].display_name, "Physics");
                assert!(payload.plugins[0].enabled);
                assert_eq!(
                    payload.plugins[0].optional_features,
                    "Ray Cast Queries [ready]"
                );
                assert_eq!(payload.plugins[0].feature_action_label, "Enable Feature");
                assert_eq!(
                    payload.plugins[0].feature_action_id,
                    "workbench.plugin.feature.enable.physics.physics.raycast_queries"
                );
                assert_eq!(payload.plugins[0].primary_action_label, "Disable");
                assert_eq!(
                    payload.plugins[0].primary_action_id,
                    "workbench.plugin.disable.physics"
                );
                assert_eq!(
                    payload.plugins[0].packaging_action_id,
                    "workbench.plugin.packaging.next.physics"
                );
                assert_eq!(
                    payload.plugins[0].target_modes_action_id,
                    "workbench.plugin.target_modes.next.physics"
                );
                assert_eq!(payload.plugins[0].unload_action_label, "Unload");
                assert_eq!(
                    payload.plugins[0].unload_action_id,
                    "workbench.plugin.unload.physics"
                );
                assert_eq!(payload.plugins[0].hot_reload_action_label, "Hot Reload");
                assert_eq!(
                    payload.plugins[0].hot_reload_action_id,
                    "workbench.plugin.hot_reload.physics"
                );
            }
            ("editor.build_export_desktop", PanePayload::BuildExportV1(payload)) => {
                assert_eq!(payload.diagnostics, "export catalog ready");
                assert_eq!(payload.targets.len(), 1);
                assert_eq!(payload.targets[0].profile_name, "desktop_windows");
                assert_eq!(payload.targets[0].platform, "Windows");
                assert_eq!(payload.targets[0].status, "Ready");
                assert_eq!(payload.targets[0].enabled_plugins, "3");
                assert_eq!(payload.targets[0].native_dynamic_packages, "1");
                assert_eq!(
                    payload.targets[0].diagnostics,
                    "native plugin package ready"
                );
            }
            ("editor.generated_bottom", PanePayload::GeneratedBottomV1(payload)) => {
                assert_eq!(payload.status, "Generated editor feedback panels");
            }
            (unexpected_id, unexpected_payload) => panic!(
                "builder for `{unexpected_id}` produced unexpected payload {unexpected_payload:?}"
            ),
        }
    }
}
