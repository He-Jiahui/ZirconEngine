use super::*;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{UiRenderCommandKind, UiRenderList, UiResolvedStyle};

#[test]
fn ordered_segments_preserve_extract_allocations() {
    let first = extract("first");
    let second = extract("second");

    let submission =
        UiRenderSubmission::from_frame_segments(vec![Arc::clone(&first), Arc::clone(&second)]);

    assert!(Arc::ptr_eq(submission.segments()[0].extract(), &first));
    assert!(Arc::ptr_eq(submission.segments()[1].extract(), &second));
    assert_eq!(submission.segments()[0].route_tree_id().as_ref(), "first");
    assert_eq!(submission.segments()[1].route_tree_id().as_ref(), "second");
    assert_eq!(submission.command_count(), 0);
    assert!(submission.is_empty());
}

#[test]
fn commands_preserve_segment_and_command_order() {
    let first = extract_with_nodes("first", &[1, 2]);
    let second = extract_with_nodes("second", &[3]);
    let submission = UiRenderSubmission::from_segments(vec![first, second]);

    assert_eq!(
        submission
            .commands()
            .map(|command| command.node_id.0)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(submission.command_count(), 3);
    assert!(!submission.is_empty());
}

#[test]
fn projected_segment_shares_commands_and_projects_only_route_identity() {
    let flat = extract_with_nodes("surface-local", &[7]);
    let frame = Arc::new(UiRenderFrameExtract::from_extract(&flat));
    let segment = UiRenderSubmissionSegment::projected(
        Arc::clone(&frame),
        UiTreeId::new("runtime-project-ui"),
        UiRenderNodeIdProjection::new(2_u64 << 48, (1_u64 << 48) - 1),
    );

    let submission = UiRenderSubmission::from_submission_segments(vec![segment]);

    assert!(Arc::ptr_eq(submission.segments()[0].extract(), &frame));
    assert_eq!(frame.list.commands[0].node_id, UiNodeId::new(7));
    assert_eq!(
        submission.commands().next().unwrap().node_id,
        UiNodeId::new((2_u64 << 48) | 7)
    );
    assert_eq!(
        submission.segments()[0].route_tree_id().as_ref(),
        "runtime-project-ui"
    );
}

fn extract(tree_id: &str) -> Arc<UiRenderFrameExtract> {
    Arc::new(UiRenderFrameExtract::from_extract(&UiRenderExtract {
        tree_id: UiTreeId::new(tree_id),
        list: UiRenderList::default(),
        raster_scale: 1.0,
    }))
}

fn extract_with_nodes(tree_id: &str, node_ids: &[u64]) -> Arc<UiRenderExtract> {
    Arc::new(UiRenderExtract {
        tree_id: UiTreeId::new(tree_id),
        list: UiRenderList {
            commands: node_ids
                .iter()
                .copied()
                .map(|node_id| UiRenderCommand {
                    node_id: zircon_runtime_interface::ui::event_ui::UiNodeId::new(node_id),
                    kind: UiRenderCommandKind::Quad,
                    frame: UiFrame::new(0.0, 0.0, 1.0, 1.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle::default(),
                    text_layout: None,
                    text: None,
                    image: None,
                    opacity: 1.0,
                })
                .collect(),
        },
        raster_scale: 1.0,
    })
}
