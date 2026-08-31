use std::collections::BTreeMap;
use std::sync::Arc;

use zircon_runtime_interface::math::UVec2;

use crate::core::editor_event::{ConsoleMessageFilter, ConsoleSourceFilter};
use crate::scene::viewport::SceneViewportChromeSettings;
use crate::ui::layouts::views::blank_viewport_chrome;
use crate::ui::layouts::windows::workbench_host_window::{
    build_pane_body_presentation, ConsolePaneViewData, PaneContentSize, PanePayloadBuildContext,
    PanePresentation, PaneShellPresentation,
};
use crate::ui::retained_host::template_node_command_summary_for_test;
use crate::ui::retained_host::to_host_contract_console_pane_from_host_pane;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, ConsoleOutputLevelCounts, ConsoleOutputSnapshot, EditorChromeSnapshot,
    EditorConsoleMessageLevel, ProjectOverviewSnapshot, WorkbenchSnapshot,
    CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace,
};

fn chrome_fixture(status_text: &str) -> EditorChromeSnapshot {
    EditorChromeSnapshot {
        focused_document_kind: None,
        workbench: WorkbenchSnapshot {
            active_main_page: MainPageId::workbench(),
            main_pages: Vec::new(),
            drawers: BTreeMap::new(),
            floating_windows: Vec::new(),
        },
        scene_entries: Default::default(),
        inspector: None,
        status_line: status_text.to_string(),
        console_output: status_text.into(),
        status_task_progress: None,
        hovered_axis: None,
        viewport_size: UVec2::new(1280, 720),
        scene_viewport_settings: SceneViewportChromeSettings::default(),
        mesh_import_path: String::new(),
        project_overview: ProjectOverviewSnapshot::default(),
        asset_activity: AssetWorkspaceSnapshot::default(),
        asset_browser: AssetWorkspaceSnapshot::default(),
        project_path: "sandbox-project".to_string(),
        session_mode: EditorSessionMode::Project,
        welcome: WelcomePaneSnapshot::default(),
        project_open: true,
        can_undo: true,
        can_redo: false,
        menu_overflow_mode: Default::default(),
    }
}

fn console_pane(status_text: &str) -> crate::ui::layouts::windows::workbench_host_window::PaneData {
    console_pane_with_output(status_text, status_text.into())
}

fn console_pane_with_output(
    status_text: &str,
    output: ConsoleOutputSnapshot,
) -> crate::ui::layouts::windows::workbench_host_window::PaneData {
    let mut chrome = chrome_fixture(status_text);
    chrome.console_output = output;
    let pane_presentation = PanePresentation::new(
        PaneShellPresentation::new(
            "Console",
            "console",
            "Task Output",
            chrome.status_line.clone(),
            None,
            false,
            blank_viewport_chrome(),
        ),
        build_pane_body_presentation(
            &PaneBodySpec::new(
                "res://ui/editor/host/console_body.zui",
                PanePayloadKind::ConsoleV1,
                PaneRouteNamespace::Dock,
                PaneInteractionMode::TemplateOnly,
            ),
            &PanePayloadBuildContext::new(&chrome),
        ),
    );

    crate::ui::layouts::windows::workbench_host_window::PaneData {
        id: "editor.console#1".into(),
        slot: "bottom_left".into(),
        kind: "Console".into(),
        title: "Console".into(),
        icon_key: "console".into(),
        subtitle: "Task Output".into(),
        info: chrome.status_line.clone().into(),
        show_empty: false,
        empty_title: "".into(),
        empty_body: "".into(),
        primary_action_label: "".into(),
        primary_action_id: "".into(),
        secondary_action_label: "".into(),
        secondary_action_id: "".into(),
        secondary_hint: "".into(),
        show_toolbar: false,
        viewport: blank_viewport_chrome(),
        native_body: crate::ui::layouts::windows::workbench_host_window::PaneNativeBodyData {
            hierarchy: Default::default(),
            inspector: Default::default(),
            console: ConsolePaneViewData {
                nodes: crate::ui::retained_host::primitives::ModelRc::default(),
                output: "legacy status".into(),
            },
            assets_activity: Default::default(),
            asset_browser: Default::default(),
            project_overview: Default::default(),
            performance_timeline: Default::default(),
            module_plugins: Default::default(),
            build_export: Default::default(),
            generated_bottom: Default::default(),
            ui_asset: Default::default(),
            animation: Default::default(),
        },
        pane_presentation: Some(pane_presentation),
    }
}

#[test]
fn console_template_body_projection_replaces_legacy_console_nodes_for_retained_conversion() {
    let projected = to_host_contract_console_pane_from_host_pane(
        &console_pane("compile started\ncache ready"),
        PaneContentSize::new(320.0, 180.0),
    );

    assert_eq!(projected.output.as_ref(), "compile started\ncache ready");
    let nodes = (0..projected.nodes.row_count())
        .filter_map(|row| projected.nodes.row_data(row))
        .collect::<Vec<_>>();
    let body_section = nodes
        .iter()
        .find(|node| node.control_id == "ConsoleBodySection")
        .expect("console body section node");
    let clear = nodes
        .iter()
        .find(|node| node.control_id == "ClearConsole")
        .expect("console clear button");
    let footer = nodes
        .iter()
        .find(|node| node.control_id == "ConsoleFooter")
        .expect("console footer");
    assert!(body_section.frame.width > 0.0);
    assert!(body_section.frame.height > 0.0);
    assert!(!nodes.iter().any(|node| node.control_id == "FocusConsole"));
    assert_eq!(clear.role, "Button");
    assert_eq!(clear.text, "Clear");
    assert_eq!(clear.button_variant, "outlined");
    assert!(clear.icon_name.is_empty());
    assert_eq!(clear.binding_id, "ConsolePaneBody/ClearConsole");
    assert_eq!(clear.frame.width, 58.0);
    assert_eq!(clear.frame.y, footer.frame.y);
    assert!(clear.frame.x + clear.frame.width <= footer.frame.x + footer.frame.width);
    assert!(footer.frame.y >= body_section.frame.y + body_section.frame.height + 6.0);
    let clear_commands = template_node_command_summary_for_test(clear);
    assert_eq!(clear_commands.text_count, 1);
    assert!(clear_commands.image_frames.is_empty());
    let clear_text = clear_commands
        .text_frames
        .first()
        .expect("projected Console clear text command");
    assert!(clear_text.width >= 24.0);
    assert!(clear_text.x + clear_text.width <= clear.frame.x + clear.frame.width + f32::EPSILON);
    let output_lines = nodes
        .iter()
        .filter(|node| node.control_id.starts_with("ConsoleOutputLine"))
        .collect::<Vec<_>>();
    assert_eq!(output_lines.len(), 2);
    assert_eq!(output_lines[0].control_id, "ConsoleOutputLine0000");
    assert_eq!(output_lines[0].role, "Label");
    assert_eq!(output_lines[0].text, "compile started");
    assert_eq!(output_lines[0].component_variant, "code");
    assert_eq!(output_lines[0].text_tone, "secondary");
    assert_eq!(output_lines[0].overflow, "elide");
    assert_eq!(output_lines[0].frame.height, 18.0);
    assert!(output_lines[0].frame.x > body_section.frame.x);
    assert!(
        body_section.frame.x + body_section.frame.width
            - (output_lines[0].frame.x + output_lines[0].frame.width)
            >= 12.0
    );
    assert_eq!(output_lines[1].control_id, "ConsoleOutputLine0001");
    assert_eq!(output_lines[1].text, "cache ready");
    assert_eq!(
        output_lines[1].frame.y,
        output_lines[0].frame.y + output_lines[0].frame.height
    );
    assert!(!nodes
        .iter()
        .any(|node| node.control_id == "ConsoleOutputLinePrototype"));
    let severity_labels = nodes
        .iter()
        .filter(|node| node.control_id.starts_with("ConsoleOutputSeverity"))
        .collect::<Vec<_>>();
    assert_eq!(severity_labels.len(), 2);
    assert_eq!(severity_labels[0].text, "[Info]");
    assert_eq!(severity_labels[0].text_tone, "secondary");
    assert_eq!(severity_labels[0].frame.y, output_lines[0].frame.y);
    assert_eq!(severity_labels[0].frame.width, 64.0);
    assert_eq!(output_lines[0].frame.x, severity_labels[0].frame.x + 64.0);
}

#[test]
fn console_template_body_projects_a_runtime_text_empty_state() {
    let projected = to_host_contract_console_pane_from_host_pane(
        &console_pane(""),
        PaneContentSize::new(320.0, 180.0),
    );
    let empty_state = (0..projected.nodes.row_count())
        .filter_map(|row| projected.nodes.row_data(row))
        .find(|node| node.control_id == "ConsoleOutputLine0000")
        .expect("console empty-state output line");

    assert_eq!(projected.output.as_ref(), "");
    assert_eq!(empty_state.text, "No output yet");
    assert_eq!(empty_state.text_tone, "muted");
}

#[test]
fn console_template_body_preserves_one_blank_runtime_text_logical_line() {
    let output = ConsoleOutputSnapshot::new(
        Arc::from(""),
        Arc::from([EditorConsoleMessageLevel::Warning]),
    );
    let projected = to_host_contract_console_pane_from_host_pane(
        &console_pane_with_output("", output),
        PaneContentSize::new(320.0, 180.0),
    );
    let nodes = (0..projected.nodes.row_count())
        .filter_map(|row| projected.nodes.row_data(row))
        .collect::<Vec<_>>();
    let severity = nodes
        .iter()
        .find(|node| node.control_id == "ConsoleOutputSeverity0000")
        .expect("blank logical line severity");
    let message = nodes
        .iter()
        .find(|node| node.control_id == "ConsoleOutputLine0000")
        .expect("blank logical line message");

    assert_eq!(severity.text, "[Warning]");
    assert_eq!(severity.text_tone, "warning");
    assert!(message.text.is_empty());
    assert_eq!(message.text_tone, "secondary");
    assert!(!nodes.iter().any(|node| node.text == "No output yet"));
}

#[test]
fn console_native_fallback_builds_legacy_nodes_only_after_template_projection_is_absent() {
    let mut pane = console_pane("ignored template output");
    pane.pane_presentation = None;

    let projected =
        to_host_contract_console_pane_from_host_pane(&pane, PaneContentSize::new(320.0, 180.0));

    assert_eq!(projected.output.as_ref(), "legacy status");
    assert!(projected.nodes.row_count() > 0);
}

#[test]
fn console_template_body_projects_runtime_text_severity_tones() {
    let status_text = "ready\nshader fallback\npipeline failed";
    let output = ConsoleOutputSnapshot::new(
        Arc::from(status_text),
        Arc::from(vec![
            EditorConsoleMessageLevel::Info,
            EditorConsoleMessageLevel::Warning,
            EditorConsoleMessageLevel::Error,
        ]),
    );
    let projected = to_host_contract_console_pane_from_host_pane(
        &console_pane_with_output(status_text, output),
        PaneContentSize::new(320.0, 180.0),
    );
    let nodes = (0..projected.nodes.row_count())
        .filter_map(|row| projected.nodes.row_data(row))
        .collect::<Vec<_>>();
    let message_tones = nodes
        .iter()
        .filter(|node| node.control_id.starts_with("ConsoleOutputLine"))
        .map(|node| node.text_tone.to_string())
        .collect::<Vec<_>>();
    let severity_labels = nodes
        .iter()
        .filter(|node| node.control_id.starts_with("ConsoleOutputSeverity"))
        .map(|node| (node.text.to_string(), node.text_tone.to_string()))
        .collect::<Vec<_>>();

    assert_eq!(message_tones, ["secondary", "secondary", "secondary"]);
    assert_eq!(
        severity_labels,
        [
            ("[Info]".to_string(), "secondary".to_string()),
            ("[Warning]".to_string(), "warning".to_string()),
            ("[Error]".to_string(), "error".to_string()),
        ]
    );
    let severity_nodes = nodes
        .iter()
        .filter(|node| node.control_id.starts_with("ConsoleOutputSeverity"))
        .collect::<Vec<_>>();
    let message_nodes = nodes
        .iter()
        .filter(|node| node.control_id.starts_with("ConsoleOutputLine"))
        .collect::<Vec<_>>();
    for (severity, message) in severity_nodes.into_iter().zip(message_nodes) {
        let severity_commands = template_node_command_summary_for_test(severity);
        let message_commands = template_node_command_summary_for_test(message);
        assert_eq!(severity_commands.text_count, 1);
        assert_eq!(message_commands.text_count, 1);
        let severity_text = severity_commands
            .text_frames
            .first()
            .expect("painted Console severity text");
        assert!(severity_text.width >= 24.0);
        assert!(
            severity_text.x + severity_text.width
                <= severity.frame.x + severity.frame.width + f32::EPSILON
        );
        assert_eq!(message.frame.x, severity.frame.x + severity.frame.width);
    }
    let level_counts = nodes
        .iter()
        .find(|node| node.control_id == "ConsoleLevelCounts")
        .expect("Console level-count group");
    assert!((186.0..=214.0).contains(&level_counts.frame.width));
    for (control_id, text, tone, validation_level, icon, width) in [
        (
            "ConsoleLevelAll",
            "All",
            "accent",
            "normal",
            "editor_pages/console_profiler/logs/filter-logs.svg",
            54.0,
        ),
        (
            "ConsoleLevelError",
            "1",
            "error",
            "error",
            "editor_pages/console_profiler/logs/log-error.svg",
            46.0,
        ),
        (
            "ConsoleLevelWarning",
            "1",
            "warning",
            "warning",
            "editor_pages/console_profiler/logs/log-warning.svg",
            46.0,
        ),
        (
            "ConsoleLevelInfo",
            "1",
            "info",
            "info",
            "editor_pages/console_profiler/logs/log-info.svg",
            46.0,
        ),
    ] {
        let node = nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("missing {control_id}"));
        assert_eq!(node.text, text);
        assert_eq!(node.text_tone, tone);
        assert_eq!(node.validation_level, validation_level);
        assert_eq!(node.button_variant, "text");
        assert_eq!(node.icon_name, icon);
        assert_eq!(node.icon_placement, "leading");
        assert_eq!(node.component_variant, "code compact_icon_text");
        assert_eq!(node.frame.width, width);
        assert_eq!(node.selected, control_id == "ConsoleLevelAll");
        assert_eq!(node.checked, node.selected);
        let command_summary = template_node_command_summary_for_test(node);
        assert_eq!(command_summary.text_count, 1);
        let icon_frame = command_summary
            .image_frames
            .first()
            .unwrap_or_else(|| panic!("missing painted icon for {control_id}"));
        let text_frame = command_summary
            .text_frames
            .first()
            .unwrap_or_else(|| panic!("missing painted text for {control_id}"));
        assert_eq!(icon_frame.width, 16.0);
        assert_eq!(icon_frame.height, 16.0);
        assert!(
            text_frame.width
                >= if control_id == "ConsoleLevelAll" {
                    16.0
                } else {
                    8.0
                }
        );
        assert!(text_frame.x >= icon_frame.x + icon_frame.width + 4.0 - f32::EPSILON);
        assert!(text_frame.x + text_frame.width <= node.frame.x + node.frame.width + f32::EPSILON);
    }
}

#[test]
fn console_template_body_keeps_controls_and_message_text_inside_narrow_panes() {
    for width in [220.0, 240.0] {
        let projected = to_host_contract_console_pane_from_host_pane(
            &console_pane("compile started"),
            PaneContentSize::new(width, 180.0),
        );
        let nodes = (0..projected.nodes.row_count())
            .filter_map(|row| projected.nodes.row_data(row))
            .collect::<Vec<_>>();

        for control_id in [
            "ConsoleLevelCounts",
            "ConsoleLevelAll",
            "ConsoleLevelError",
            "ConsoleLevelWarning",
            "ConsoleLevelInfo",
        ] {
            let node = nodes
                .iter()
                .find(|node| node.control_id == control_id)
                .unwrap_or_else(|| panic!("missing {control_id} at width {width}"));
            assert!(node.frame.x >= 0.0);
            assert!(node.frame.x + node.frame.width <= width + f32::EPSILON);
        }

        let severity = nodes
            .iter()
            .find(|node| node.control_id == "ConsoleOutputSeverity0000")
            .expect("narrow Console severity");
        let message = nodes
            .iter()
            .find(|node| node.control_id == "ConsoleOutputLine0000")
            .expect("narrow Console message");
        assert!(message.frame.width > 0.0);
        assert_eq!(message.frame.x, severity.frame.x + severity.frame.width);
        assert!(message.frame.x + message.frame.width <= width + f32::EPSILON);
    }
}

#[test]
fn console_template_body_keeps_the_snapshot_logical_window_but_materializes_only_slots() {
    let status_text = (0..8_000)
        .map(|index| format!("line {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let output = ConsoleOutputSnapshot::from(status_text.clone());
    assert_eq!(output.levels().len(), CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
    assert_eq!(
        output.as_ref().lines().count(),
        CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY
    );
    assert!(output.as_ref().ends_with("line 7999"));
    let projected = to_host_contract_console_pane_from_host_pane(
        &console_pane_with_output(&status_text, output),
        PaneContentSize::new(320.0, 180.0),
    );
    let nodes = (0..projected.nodes.row_count())
        .filter_map(|row| projected.nodes.row_data(row))
        .collect::<Vec<_>>();
    let message_nodes = nodes
        .iter()
        .filter(|node| node.control_id.starts_with("ConsoleOutputLine"))
        .collect::<Vec<_>>();
    let severity_nodes = nodes
        .iter()
        .filter(|node| node.control_id.starts_with("ConsoleOutputSeverity"))
        .collect::<Vec<_>>();
    assert_eq!(message_nodes.len(), severity_nodes.len());
    assert!(!message_nodes.is_empty());
    assert!(message_nodes.len() < CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
    assert_eq!(
        message_nodes.first().map(|node| node.text.as_ref()),
        Some("line 7744")
    );
}

#[test]
fn console_template_body_projects_filtered_lines_total_counts_and_selected_segment() {
    let output = ConsoleOutputSnapshot::filtered(
        Arc::from("shader fallback"),
        Arc::from([EditorConsoleMessageLevel::Warning]),
        ConsoleOutputLevelCounts {
            info: 2,
            warning: 1,
            error: 3,
        },
        ConsoleMessageFilter::Warning,
    );
    let projected = to_host_contract_console_pane_from_host_pane(
        &console_pane_with_output("Shader fallback", output),
        PaneContentSize::new(320.0, 180.0),
    );
    let nodes = (0..projected.nodes.row_count())
        .filter_map(|row| projected.nodes.row_data(row))
        .collect::<Vec<_>>();
    let output_lines = nodes
        .iter()
        .filter(|node| node.control_id.starts_with("ConsoleOutputLine"))
        .collect::<Vec<_>>();
    assert_eq!(output_lines.len(), 1);
    assert_eq!(output_lines[0].text, "shader fallback");
    assert_eq!(output_lines[0].text_tone, "secondary");
    let severity = nodes
        .iter()
        .find(|node| node.control_id == "ConsoleOutputSeverity0000")
        .expect("filtered warning severity label");
    assert_eq!(severity.text, "[Warning]");
    assert_eq!(severity.text_tone, "warning");

    for (control_id, text, selected, binding_id) in [
        ("ConsoleLevelAll", "All", false, "ConsolePaneBody/FilterAll"),
        (
            "ConsoleLevelError",
            "3",
            false,
            "ConsolePaneBody/FilterError",
        ),
        (
            "ConsoleLevelWarning",
            "1",
            true,
            "ConsolePaneBody/FilterWarning",
        ),
        ("ConsoleLevelInfo", "2", false, "ConsolePaneBody/FilterInfo"),
    ] {
        let node = nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("missing {control_id}"));
        assert_eq!(node.text, text);
        assert_eq!(node.selected, selected);
        assert_eq!(node.checked, selected);
        assert_eq!(node.binding_id, binding_id);
        assert_eq!(node.role, "Button");
    }
}

#[test]
fn console_template_body_projects_source_filter_and_typed_jump_action_tokens() {
    let output = ConsoleOutputSnapshot::activity(
        Arc::from("#77 [frame 12] [import] material warning\n#78 [frame 13] [editor] plain"),
        Arc::from([
            EditorConsoleMessageLevel::Warning,
            EditorConsoleMessageLevel::Info,
        ]),
        ConsoleMessageFilter::All,
        ConsoleSourceFilter::Import,
        Arc::from([Some(77), None]),
    );
    let projected = to_host_contract_console_pane_from_host_pane(
        &console_pane_with_output("ignored legacy text", output),
        PaneContentSize::new(320.0, 220.0),
    );
    let nodes = (0..projected.nodes.row_count())
        .filter_map(|row| projected.nodes.row_data(row))
        .collect::<Vec<_>>();

    let row = nodes
        .iter()
        .find(|node| node.control_id == "ConsoleOutputLine0000")
        .expect("activity log row");
    assert_eq!(row.action_id, "workbench.activity_log.jump.77");
    assert_eq!(row.dispatch_kind, "activity_log_jump");
    assert_eq!(row.text_tone, "accent");
    let plain_row = nodes
        .iter()
        .find(|node| node.control_id == "ConsoleOutputLine0001")
        .expect("plain activity log row");
    assert!(plain_row.action_id.is_empty());
    assert!(plain_row.dispatch_kind.is_empty());

    for (control_id, selected, binding_id) in [
        ("ConsoleSourceAll", false, "ConsolePaneBody/SourceAll"),
        ("ConsoleSourceEditor", false, "ConsolePaneBody/SourceEditor"),
        (
            "ConsoleSourceRuntime",
            false,
            "ConsolePaneBody/SourceRuntime",
        ),
        ("ConsoleSourcePlay", false, "ConsolePaneBody/SourcePlay"),
        ("ConsoleSourcePlugin", false, "ConsolePaneBody/SourcePlugin"),
        ("ConsoleSourceImport", true, "ConsolePaneBody/SourceImport"),
        (
            "ConsoleSourceScriptBuild",
            false,
            "ConsolePaneBody/SourceScriptBuild",
        ),
    ] {
        let node = nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("missing {control_id}"));
        assert_eq!(node.selected, selected);
        assert_eq!(node.checked, selected);
        assert_eq!(node.binding_id, binding_id);
    }
}

#[test]
fn console_payload_builder_preserves_the_generation_owned_output_snapshot() {
    let source = include_str!(
        "../../ui/layouts/windows/workbench_host_window/pane_payload_builders/console.rs"
    );

    assert!(source.contains("output: context.chrome.console_output.clone()"));
    assert!(!source.contains("console_output.text_arc()"));
    assert!(!source.contains("console_output.to_string()"));

    let pane_projection =
        include_str!("../../ui/layouts/windows/workbench_host_window/pane_projection.rs");
    assert!(pane_projection.contains("output: chrome.console_output.clone()"));
    assert!(!pane_projection.contains("chrome.console_output.text_arc()"));

    let scene_projection =
        include_str!("../../ui/layouts/windows/workbench_host_window/scene_projection.rs");
    assert!(!scene_projection.contains("console_pane_nodes("));

    let retained_projection =
        include_str!("../../ui/retained_host/ui/pane_data_conversion/console_projection.rs");
    assert!(retained_projection.contains("unwrap_or_else"));
    assert!(retained_projection.contains("new_virtualized_snapshot"));
    assert!(!retained_projection.contains("console_payload.output.as_ref()"));

    let builtin_bindings = include_str!("../../ui/template_runtime/builtin/template_bindings.rs");
    assert!(!builtin_bindings.contains("ConsolePaneBody/FocusConsole"));
    let legacy_projection = include_str!("../../ui/layouts/views/console.rs");
    assert!(!legacy_projection.contains("mark_console_node(nodes, \"FocusConsole\""));
}
