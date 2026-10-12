use crate::ui::retained_host::console_output::{
    ConsoleOutputLogicalLine, ConsoleOutputPaintMetadata, ConsoleOutputViewport,
    CONSOLE_OUTPUT_OVERSCAN_LINES,
};
use crate::ui::retained_host::host_contract::data::{
    ConsolePaneData, FrameRect, PaneData, TemplateNodeFrameData, TemplatePaneNodeData,
};
use crate::ui::retained_host::primitives::ModelRc;

use super::super::hit_test_pane_template_node;
use super::support::{model, option};

#[test]
fn console_pane_hit_test_uses_scrolled_line_geometry() {
    let metadata = ConsoleOutputPaintMetadata::new(
        ConsoleOutputViewport {
            x: 8.0,
            y: 40.0,
            width: 240.0,
            height: 36.0,
        },
        40.0,
        1,
        3,
    )
    .expect("console output metadata");
    let nodes = ModelRc::with_metadata(
        vec![
            TemplatePaneNodeData {
                node_id: "source-filter".into(),
                control_id: "ConsoleSourceAll".into(),
                role: "Button".into(),
                action_id: "workbench.console.source.all".into(),
                frame: TemplateNodeFrameData {
                    x: 8.0,
                    y: 8.0,
                    width: 120.0,
                    height: 20.0,
                },
                ..TemplatePaneNodeData::default()
            },
            console_jump_node("line-1", "workbench.activity_log.jump.1", 40.0),
            console_jump_node("line-2", "workbench.activity_log.jump.2", 58.0),
            console_jump_node("line-3", "workbench.activity_log.jump.3", 76.0),
        ],
        metadata,
    );
    let pane = PaneData {
        id: "editor.console#1".into(),
        kind: "Console".into(),
        console: ConsolePaneData {
            nodes,
            output: "three activity rows".into(),
        },
        ..PaneData::default()
    };
    let body = FrameRect {
        x: 100.0,
        y: 50.0,
        width: 260.0,
        height: 100.0,
    };

    let hit = hit_test_pane_template_node(&pane, &body, 120.0, 99.0, 18.0)
        .expect("the second row should move into the first visible slot");

    assert_eq!(hit.action_id.as_str(), "workbench.activity_log.jump.2");
    assert_eq!(hit.frame.y, 90.0);

    assert!(
        hit_test_pane_template_node(&pane, &body, 200.0, 130.0, 18.0).is_none(),
        "a raw log row below the clipped viewport must not remain dispatchable"
    );
    let header = hit_test_pane_template_node(&pane, &body, 120.0, 65.0, 18.0)
        .expect("non-log controls outside the output viewport must remain dispatchable");
    assert_eq!(header.control_id.as_str(), "ConsoleSourceAll");
}

#[test]
fn console_pane_hit_test_resolves_action_from_the_virtualized_logical_generation() {
    let logical_lines = (0..8_000)
        .map(|index| {
            ConsoleOutputLogicalLine::new(format!("line-{index:04}"), "accent".into())
                .with_severity("[Info]".into(), "secondary".into())
                .with_action(
                    "activity_log_jump".into(),
                    format!("workbench.activity_log.jump.{index}"),
                )
        })
        .collect();
    let metadata = ConsoleOutputPaintMetadata::new_virtualized(
        ConsoleOutputViewport {
            x: 8.0,
            y: 40.0,
            width: 240.0,
            height: 36.0,
        },
        40.0,
        1,
        logical_lines,
        2,
        CONSOLE_OUTPUT_OVERSCAN_LINES,
    )
    .expect("virtualized console output metadata");
    let mut projected_nodes = vec![TemplatePaneNodeData {
        node_id: "source-filter".into(),
        control_id: "ConsoleSourceAll".into(),
        role: "Button".into(),
        action_id: "workbench.console.source.all".into(),
        frame: TemplateNodeFrameData {
            x: 8.0,
            y: 8.0,
            width: 120.0,
            height: 20.0,
        },
        ..TemplatePaneNodeData::default()
    }];
    for slot in 0..metadata.materialized_line_count() {
        let y = 40.0 + slot as f32 * 18.0;
        projected_nodes.push(TemplatePaneNodeData {
            node_id: format!("ConsoleOutputSeverity{slot:04}"),
            control_id: format!("ConsoleOutputSeverity{slot:04}"),
            frame: TemplateNodeFrameData {
                x: 8.0,
                y,
                width: 64.0,
                height: 18.0,
            },
            ..TemplatePaneNodeData::default()
        });
        projected_nodes.push(TemplatePaneNodeData {
            node_id: format!("ConsoleOutputLine{slot:04}"),
            control_id: format!("ConsoleOutputLine{slot:04}"),
            role: "Button".into(),
            frame: TemplateNodeFrameData {
                x: 72.0,
                y,
                width: 176.0,
                height: 18.0,
            },
            ..TemplatePaneNodeData::default()
        });
    }
    let pane = PaneData {
        id: "editor.console#1".into(),
        kind: "Console".into(),
        console: ConsolePaneData {
            nodes: ModelRc::with_metadata(projected_nodes, metadata),
            output: "virtualized activity rows".into(),
        },
        ..PaneData::default()
    };
    let body = FrameRect {
        x: 100.0,
        y: 50.0,
        width: 260.0,
        height: 100.0,
    };

    let first = hit_test_pane_template_node(&pane, &body, 200.0, 99.0, 1_800.0)
        .expect("logical row 100 through its ring slot");
    assert_eq!(first.control_id.as_str(), "ConsoleOutputLine0002");
    assert_eq!(first.action_id.as_str(), "workbench.activity_log.jump.100");
    assert_eq!(first.frame.y, 90.0);

    let next = hit_test_pane_template_node(&pane, &body, 200.0, 99.0, 1_818.0)
        .expect("logical row 101 after one-line scroll");
    assert_eq!(next.control_id.as_str(), "ConsoleOutputLine0003");
    assert_eq!(next.action_id.as_str(), "workbench.activity_log.jump.101");
}

#[test]
fn console_popup_rows_take_priority_over_scrolled_log_rows() {
    let metadata = ConsoleOutputPaintMetadata::new(
        ConsoleOutputViewport {
            x: 8.0,
            y: 40.0,
            width: 240.0,
            height: 36.0,
        },
        40.0,
        1,
        1,
    )
    .expect("console output metadata");
    let nodes = ModelRc::with_metadata(
        vec![
            TemplatePaneNodeData {
                node_id: "source-filter".into(),
                control_id: "ConsoleSourceFilter".into(),
                role: "Dropdown".into(),
                component_role: "dropdown".into(),
                edit_action_id: "workbench.console.source.select".into(),
                popup_open: true,
                structured_options: model(vec![option("all", false), option("runtime", false)]),
                frame: TemplateNodeFrameData {
                    x: 8.0,
                    y: 8.0,
                    width: 120.0,
                    height: 20.0,
                },
                ..TemplatePaneNodeData::default()
            },
            console_jump_node("line-2", "workbench.activity_log.jump.2", 58.0),
        ],
        metadata,
    );
    let pane = PaneData {
        id: "editor.console#1".into(),
        kind: "Console".into(),
        console: ConsolePaneData {
            nodes,
            output: "one activity row".into(),
        },
        ..PaneData::default()
    };
    let body = FrameRect {
        x: 100.0,
        y: 50.0,
        width: 260.0,
        height: 100.0,
    };

    let hit = hit_test_pane_template_node(&pane, &body, 120.0, 99.0, 18.0)
        .expect("the open source popup should cover the scrolled log row");

    assert_eq!(hit.dispatch_kind.as_str(), "workbench_option");
    assert_eq!(hit.value_text.as_str(), "all");
    assert_eq!(hit.action_id.as_str(), "workbench.console.source.select");
}

fn console_jump_node(control_id: &str, action_id: &str, y: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: control_id.into(),
        control_id: control_id.into(),
        role: "Label".into(),
        dispatch_kind: "activity_log_jump".into(),
        action_id: action_id.into(),
        frame: TemplateNodeFrameData {
            x: 72.0,
            y,
            width: 176.0,
            height: 18.0,
        },
        ..TemplatePaneNodeData::default()
    }
}
