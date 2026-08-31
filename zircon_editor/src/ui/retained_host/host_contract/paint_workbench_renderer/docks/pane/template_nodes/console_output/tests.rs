use crate::ui::retained_host::console_output::{
    ConsoleOutputLogicalLine, ConsoleOutputPaintMetadata, ConsoleOutputViewport,
    CONSOLE_OUTPUT_OVERSCAN_LINES,
};
use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPaneInteractionStateData, TemplateNodeFrameData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::host_contract::paint_template_nodes::TemplateNodePaintTransform;
use crate::ui::retained_host::primitives::ModelRc;

use super::ConsoleOutputProjector;

#[test]
fn console_output_projector_visits_visible_rows_and_applies_scroll_with_fixed_clip() {
    let nodes = console_model();
    let interaction = HostPaneInteractionStateData {
        console_scroll_px: 18.0,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        ConsoleOutputProjector::new(&nodes, &frame(5.0, 7.0, 120.0, 80.0), &interaction)
            .expect("console output projector");
    let pane_clip = frame(0.0, 0.0, 200.0, 200.0);

    assert_eq!(
        streamed_rows(&projector, nodes.row_count(), &pane_clip),
        vec![0, 1, 3, 4, 7]
    );
    assert!(projector
        .transform_row(
            2,
            nodes.row_data(2).expect("first output line"),
            pane_clip.clone()
        )
        .is_none());
    let (line, line_clip) = projector
        .transform_row(
            3,
            nodes.row_data(3).expect("second output line"),
            pane_clip.clone(),
        )
        .expect("second line scrolls into the output viewport");
    assert_eq!(line.frame.y, 20.0);
    assert_eq!(line.clip_frame.x, 10.0);
    assert_eq!(line.clip_frame.y, 20.0);
    assert_eq!(line.clip_frame.width, 100.0);
    assert_eq!(line.clip_frame.height, 36.0);
    assert_eq!(line_clip, frame(15.0, 27.0, 100.0, 36.0));

    let (header, header_clip) = projector
        .transform_row(0, nodes.row_data(0).expect("header"), pane_clip.clone())
        .expect("fixed header");
    assert_eq!(header.frame.y, 0.0);
    assert_eq!(header_clip, pane_clip);
}

#[test]
fn console_output_projector_scrolls_severity_and_message_as_one_logical_line() {
    let nodes = composite_console_model_with_line_count(5);
    let interaction = HostPaneInteractionStateData {
        console_scroll_px: 18.0,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        ConsoleOutputProjector::new(&nodes, &frame(5.0, 7.0, 120.0, 80.0), &interaction)
            .expect("composite console output projector");
    let pane_clip = frame(0.0, 0.0, 200.0, 200.0);

    assert_eq!(
        streamed_rows(&projector, nodes.row_count(), &pane_clip),
        vec![0, 1, 4, 5, 6, 7, 12]
    );
    let (severity, _) = projector
        .transform_row(
            4,
            nodes.row_data(4).expect("second severity label"),
            pane_clip.clone(),
        )
        .expect("severity label scrolls with its logical line");
    let (message, _) = projector
        .transform_row(
            5,
            nodes.row_data(5).expect("second message label"),
            pane_clip,
        )
        .expect("message label scrolls with its logical line");
    assert_eq!(severity.frame.y, 20.0);
    assert_eq!(message.frame.y, severity.frame.y);
}

#[test]
fn console_output_projector_rebinds_ring_slots_from_generation_owned_logical_lines() {
    let nodes = virtualized_console_model(8_000);
    let interaction = HostPaneInteractionStateData {
        console_scroll_px: 1_800.0,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        ConsoleOutputProjector::new(&nodes, &frame(5.0, 7.0, 120.0, 80.0), &interaction)
            .expect("virtualized console output projector");
    let pane_clip = frame(0.0, 0.0, 200.0, 200.0);

    assert_eq!(
        streamed_rows(&projector, nodes.row_count(), &pane_clip),
        vec![0, 1, 6, 7, 8, 9, 16]
    );
    let (severity, _) = projector
        .transform_row(
            6,
            nodes.row_data(6).expect("logical line 100 severity slot"),
            pane_clip.clone(),
        )
        .expect("visible virtualized severity");
    let (message, _) = projector
        .transform_row(
            7,
            nodes.row_data(7).expect("logical line 100 message slot"),
            pane_clip,
        )
        .expect("visible virtualized message");
    assert_eq!(severity.text, "[Warning]");
    assert_eq!(message.text, "line-0100");
    assert_eq!(message.action_id, "workbench.activity_log.jump.100");
    assert_eq!(severity.frame.y, 20.0);
    assert_eq!(message.frame.y, 20.0);
}

#[test]
fn console_output_projector_keeps_hover_identity_while_rebinding_slot_payload() {
    let nodes = virtualized_console_model(8_000);
    let pane_clip = frame(0.0, 0.0, 200.0, 200.0);
    let project = |scroll_px: f32| {
        let projector = ConsoleOutputProjector::new(
            &nodes,
            &frame(5.0, 7.0, 120.0, 80.0),
            &HostPaneInteractionStateData {
                console_scroll_px: scroll_px,
                ..HostPaneInteractionStateData::default()
            },
        )
        .expect("virtualized console output projector");
        projector
            .transform_row(
                7,
                nodes.row_data(7).expect("message slot two"),
                pane_clip.clone(),
            )
            .map(|(node, _)| node)
            .expect("message slot two is visible")
    };

    let before = project(1_800.0);
    let after = project(1_926.0);

    assert_eq!(before.control_id, "ConsoleOutputLine0002");
    assert_eq!(after.control_id, before.control_id);
    assert_eq!(before.text, "line-0100");
    assert_eq!(after.text, "line-0107");
    assert_eq!(before.action_id, "workbench.activity_log.jump.100");
    assert_eq!(after.action_id, "workbench.activity_log.jump.107");
    assert_eq!(before.frame.y, after.frame.y);
}

#[test]
fn console_output_projector_requires_projection_metadata() {
    assert!(ConsoleOutputProjector::new(
        &ModelRc::with_metadata(Vec::<TemplatePaneNodeData>::new(), ()),
        &frame(0.0, 0.0, 100.0, 60.0),
        &HostPaneInteractionStateData::default(),
    )
    .is_none());
}

#[test]
fn console_output_projector_draws_shared_scrollbar_only_for_overflowing_rows() {
    let clip = frame(0.0, 0.0, 200.0, 200.0);
    let mut pixels = HostRgbaFrame::filled(200, 200, [0, 0, 0, 255]);
    let overflowing = console_model_with_line_count(5);
    let projector = ConsoleOutputProjector::new(
        &overflowing,
        &frame(5.0, 7.0, 120.0, 80.0),
        &HostPaneInteractionStateData {
            console_scroll_px: 18.0,
            ..HostPaneInteractionStateData::default()
        },
    )
    .expect("overflowing console output projector");

    assert!(projector.draw_scrollbar(&mut pixels, &clip));

    let fitting = console_model_with_line_count(2);
    let projector = ConsoleOutputProjector::new(
        &fitting,
        &frame(5.0, 7.0, 120.0, 80.0),
        &HostPaneInteractionStateData::default(),
    )
    .expect("fitting console output projector");

    assert!(!projector.draw_scrollbar(&mut pixels, &clip));
}

fn console_model() -> ModelRc<TemplatePaneNodeData> {
    console_model_with_line_count(5)
}

fn streamed_rows(
    projector: &ConsoleOutputProjector,
    row_count: usize,
    clip: &FrameRect,
) -> Vec<usize> {
    let mut rows = Vec::new();
    assert!(projector.stream_row_visit_indices(row_count, clip, &mut |row| rows.push(row)));
    rows
}

fn console_model_with_line_count(line_count: usize) -> ModelRc<TemplatePaneNodeData> {
    let mut nodes = vec![node("ConsoleHeader", 0.0), node("ConsoleBodySection", 20.0)];
    nodes.extend((0..line_count).map(|index| {
        node(
            format!("ConsoleOutputLine{index:04}").as_str(),
            20.0 + index as f32 * 18.0,
        )
    }));
    nodes.push(node("ConsoleFooter", 74.0));
    ModelRc::with_metadata(
        nodes,
        ConsoleOutputPaintMetadata::new(
            ConsoleOutputViewport {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 36.0,
            },
            20.0,
            2,
            line_count,
        )
        .expect("valid console metadata"),
    )
}

fn composite_console_model_with_line_count(line_count: usize) -> ModelRc<TemplatePaneNodeData> {
    let mut nodes = vec![node("ConsoleHeader", 0.0), node("ConsoleBodySection", 20.0)];
    for index in 0..line_count {
        let y = 20.0 + index as f32 * 18.0;
        nodes.push(node(format!("ConsoleOutputSeverity{index:04}").as_str(), y));
        nodes.push(node(format!("ConsoleOutputLine{index:04}").as_str(), y));
    }
    nodes.push(node("ConsoleFooter", 74.0));
    ModelRc::with_metadata(
        nodes,
        ConsoleOutputPaintMetadata::new_with_nodes_per_line(
            ConsoleOutputViewport {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 36.0,
            },
            20.0,
            2,
            line_count,
            2,
        )
        .expect("valid composite console metadata"),
    )
}

fn virtualized_console_model(line_count: usize) -> ModelRc<TemplatePaneNodeData> {
    let logical_lines = (0..line_count)
        .map(|index| {
            ConsoleOutputLogicalLine::new(format!("line-{index:04}"), "accent".into())
                .with_severity("[Warning]".into(), "warning".into())
                .with_action(
                    "activity_log_jump".into(),
                    format!("workbench.activity_log.jump.{index}"),
                )
        })
        .collect();
    let metadata = ConsoleOutputPaintMetadata::new_virtualized(
        ConsoleOutputViewport {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 36.0,
        },
        20.0,
        2,
        logical_lines,
        2,
        CONSOLE_OUTPUT_OVERSCAN_LINES,
    )
    .expect("valid virtualized console metadata");
    let mut nodes = vec![node("ConsoleHeader", 0.0), node("ConsoleBodySection", 20.0)];
    for slot in 0..metadata.materialized_line_count() {
        let y = 20.0 + slot as f32 * 18.0;
        nodes.push(node(format!("ConsoleOutputSeverity{slot:04}").as_str(), y));
        nodes.push(node(format!("ConsoleOutputLine{slot:04}").as_str(), y));
    }
    nodes.push(node("ConsoleFooter", 74.0));
    ModelRc::with_metadata(nodes, metadata)
}

fn node(control_id: &str, y: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        frame: template_frame(10.0, y, 100.0, 18.0),
        ..TemplatePaneNodeData::default()
    }
}

fn template_frame(x: f32, y: f32, width: f32, height: f32) -> TemplateNodeFrameData {
    TemplateNodeFrameData {
        x,
        y,
        width,
        height,
    }
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
