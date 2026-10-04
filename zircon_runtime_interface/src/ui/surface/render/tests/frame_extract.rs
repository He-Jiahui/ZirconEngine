use super::*;
use crate::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiRenderCommandKind, UiRenderList, UiResolvedStyle},
};

#[test]
fn local_patch_preserves_untouched_segments_and_flat_order() {
    let extract = extract_with_commands(130);
    let frame = UiRenderFrameExtract::from_extract(&extract);
    let mut changed = extract.clone();
    changed.list.commands[65].opacity = 0.25;

    let (patched, stats) = frame
        .patch_ranges_from_extract(&changed, &[65..66])
        .expect("fixed-cardinality patch should preserve the frame directory");

    assert_eq!(stats.cloned_command_count, 64);
    assert_eq!(stats.cloned_segment_count, 1);
    assert_eq!(stats.cloned_directory_node_count, 1);
    assert_eq!(
        frame
            .list
            .commands
            .shared_segment_count(&patched.list.commands),
        2
    );
    assert_eq!(patched.list.commands[65].opacity, 0.25);
    assert_eq!(patched.to_extract(), changed);
}

#[test]
fn cross_segment_patch_clones_each_touched_leaf_once() {
    let extract = extract_with_commands(130);
    let frame = UiRenderFrameExtract::from_extract(&extract);
    let mut changed = extract.clone();
    changed.list.commands[63].opacity = 0.5;
    changed.list.commands[64].opacity = 0.75;

    let (patched, stats) = frame
        .patch_ranges_from_extract(&changed, &[63..65, 64..65])
        .expect("overlapping ranges should be coalesced by segment identity");

    assert_eq!(stats.cloned_command_count, 128);
    assert_eq!(stats.cloned_segment_count, 2);
    assert_eq!(stats.cloned_directory_node_count, 1);
    assert_eq!(
        frame
            .list
            .commands
            .shared_segment_count(&patched.list.commands),
        1
    );
}

#[test]
fn deep_directory_patch_clones_one_node_per_level() {
    let command_count = UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE * 32 + 1;
    let extract = extract_with_commands(command_count);
    let frame = UiRenderFrameExtract::from_extract(&extract);
    let mut changed = extract.clone();
    changed.list.commands[command_count - 1].opacity = 0.5;

    let (patched, stats) = frame
        .patch_ranges_from_extract(&changed, &[command_count - 1..command_count])
        .expect("the last partial leaf should remain addressable through the directory");

    assert_eq!(frame.list.commands.segment_count(), 33);
    assert_eq!(frame.list.commands.directory_depth(), 2);
    assert_eq!(stats.cloned_command_count, 1);
    assert_eq!(stats.cloned_segment_count, 1);
    assert_eq!(stats.cloned_directory_node_count, 2);
    assert_eq!(
        frame
            .list
            .commands
            .shared_segment_count(&patched.list.commands),
        32
    );
    assert_eq!(patched.to_extract(), changed);
}

#[test]
fn serialized_frame_extract_keeps_the_flat_command_schema() {
    let extract = extract_with_commands(65);
    let frame = UiRenderFrameExtract::from_extract(&extract);
    let flat = serde_json::to_value(&extract).unwrap();
    let segmented = serde_json::to_value(&frame).unwrap();

    assert_eq!(segmented, flat);
    assert_eq!(
        serde_json::from_value::<UiRenderFrameExtract>(segmented).unwrap(),
        frame
    );
}

#[test]
fn sequential_iterator_keeps_exact_flat_order_without_heap_frontier() {
    let extract = extract_with_commands(UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE * 33 + 7);
    let frame = UiRenderFrameExtract::from_extract(&extract);

    assert_eq!(
        frame
            .list
            .commands
            .iter()
            .map(|command| command.node_id)
            .collect::<Vec<_>>(),
        extract
            .list
            .commands
            .iter()
            .map(|command| command.node_id)
            .collect::<Vec<_>>()
    );
}

#[test]
fn published_node_ranges_are_shared_across_local_payload_patches() {
    let mut extract = extract_with_commands(5);
    for (command, node_id) in extract.list.commands.iter_mut().zip([7_u64, 7, 9, 9, 11]) {
        command.node_id = UiNodeId::new(node_id);
    }
    let frame = UiRenderFrameExtract::from_extract(&extract);

    assert_eq!(frame.command_range(UiNodeId::new(7)), Some(0..2));
    assert_eq!(frame.command_range(UiNodeId::new(9)), Some(2..4));
    assert_eq!(
        frame
            .commands_for_node(UiNodeId::new(9))
            .expect("node commands")
            .map(|command| command.node_id)
            .collect::<Vec<_>>(),
        vec![UiNodeId::new(9), UiNodeId::new(9)]
    );

    let mut changed = extract.clone();
    changed.list.commands[2].opacity = 0.25;
    let (patched, _) = frame
        .patch_ranges_from_extract(&changed, &[2..3])
        .expect("payload-only patch");

    assert!(Arc::ptr_eq(
        &frame.list.commands.command_ranges,
        &patched.list.commands.command_ranges
    ));
}

#[test]
fn frame_command_refs_resolve_only_inside_the_owner_range() {
    let mut extract = extract_with_commands(4);
    for (command, node_id) in extract.list.commands.iter_mut().zip([7_u64, 7, 9, 9]) {
        command.node_id = UiNodeId::new(node_id);
    }
    let frame = UiRenderFrameExtract::from_extract(&extract);

    let second = frame
        .command_by_ref(UiRenderFrameCommandRef::new(UiNodeId::new(7), 1))
        .expect("second command in node range");
    assert_eq!(second.frame.x, 1.0);
    assert!(frame
        .command_by_ref(UiRenderFrameCommandRef::new(UiNodeId::new(7), 2))
        .is_none());
    assert!(frame
        .command_by_ref(UiRenderFrameCommandRef::new(UiNodeId::new(11), 0))
        .is_none());
}

#[test]
fn non_contiguous_owner_ranges_and_owner_changing_patches_fail_closed() {
    let mut extract = extract_with_commands(3);
    for (command, node_id) in extract.list.commands.iter_mut().zip([7_u64, 9, 7]) {
        command.node_id = UiNodeId::new(node_id);
    }
    let frame = UiRenderFrameExtract::from_extract(&extract);
    assert_eq!(frame.command_range(UiNodeId::new(7)), None);

    let mut changed = extract.clone();
    changed.list.commands[1].node_id = UiNodeId::new(11);
    assert!(frame.patch_ranges_from_extract(&changed, &[0..1]).is_none());
}

fn extract_with_commands(command_count: usize) -> UiRenderExtract {
    UiRenderExtract {
        tree_id: UiTreeId::new("frame.segmented.render"),
        list: UiRenderList {
            commands: (0..command_count).map(command).collect(),
        },
        raster_scale: 1.0,
    }
}

fn command(index: usize) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(index as u64 + 1),
        kind: UiRenderCommandKind::Quad,
        frame: UiFrame::new(index as f32, 0.0, 1.0, 1.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle::default(),
        text_layout: None,
        text: None,
        image: None,
        opacity: 1.0,
    }
}
