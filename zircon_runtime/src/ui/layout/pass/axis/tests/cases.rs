use std::{hint::black_box, time::Instant};

use zircon_runtime_interface::ui::{
    event_ui::{UiNodePath, UiTreeId},
    layout::UiContainerKind,
    tree::{UiTreeNode, UiVisibility},
};

use super::*;

#[test]
fn linear_extent_gap_counts_hidden_but_not_collapsed_children() {
    let root = UiNodeId::new(1);
    let visible = UiNodeId::new(2);
    let collapsed = UiNodeId::new(3);
    let hidden = UiNodeId::new(4);
    let mut tree = UiTree::new(UiTreeId::new("linear-axis-visibility"));
    tree.insert_root(
        UiTreeNode::new(root, UiNodePath::new("root"))
            .with_container(UiContainerKind::HorizontalBox(Default::default())),
    );
    for (id, visibility) in [
        (visible, UiVisibility::Visible),
        (collapsed, UiVisibility::Collapsed),
        (hidden, UiVisibility::Hidden),
    ] {
        tree.insert_child(
            root,
            UiTreeNode::new(id, UiNodePath::new(format!("child-{}", id.0)))
                .with_visibility(visibility),
        )
        .expect("root exists");
    }

    let children = [visible, collapsed, hidden];
    let mut scratch = UiLinearArrangeScratch::default();
    resolve_linear_child_main_extents(
        &tree,
        root,
        &children,
        UiAxis::Horizontal,
        100.0,
        10.0,
        &UiLayoutSlotIndex::default(),
        &mut scratch,
    )
    .expect("valid children");

    assert_eq!(scratch.resolved.len(), 3);
    assert_eq!(scratch.resolved[1].resolved, 0.0);
    assert_eq!(scratch.resolved[0].resolved, 45.0);
    assert_eq!(scratch.resolved[2].resolved, 45.0);

    let mut legacy_scratch = UiLinearArrangeScratch::default();
    resolve_linear_child_main_extents_legacy(
        &tree,
        root,
        &children,
        UiAxis::Horizontal,
        100.0,
        10.0,
        &UiLayoutSlotIndex::default(),
        &mut legacy_scratch,
    )
    .expect("old resolver accepts valid children");
    assert_eq!(scratch.constraints, legacy_scratch.constraints);
    assert_eq!(scratch.resolved, legacy_scratch.resolved);

    let missing = UiNodeId::new(99);
    assert_eq!(
        resolve_linear_child_main_extents(
            &tree,
            root,
            &[visible, missing],
            UiAxis::Horizontal,
            100.0,
            10.0,
            &UiLayoutSlotIndex::default(),
            &mut scratch,
        ),
        Err(UiTreeError::MissingNode(missing))
    );
    assert_eq!(
        resolve_linear_child_main_extents_legacy(
            &tree,
            root,
            &[visible, missing],
            UiAxis::Horizontal,
            100.0,
            10.0,
            &UiLayoutSlotIndex::default(),
            &mut legacy_scratch,
        ),
        Err(UiTreeError::MissingNode(missing))
    );
}

// Keep the pre-optimization resolver here so the release comparison measures
// the actual old count pass and constraint loop, without the new in-loop count.
fn resolve_linear_child_main_extents_legacy(
    tree: &UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    axis: UiAxis,
    available_extent: f32,
    gap: f32,
    slot_index: &UiLayoutSlotIndex,
    scratch: &mut UiLinearArrangeScratch,
) -> Result<(), UiTreeError> {
    let layout_child_count = children
        .iter()
        .filter(|child_id| {
            tree.node(**child_id)
                .is_some_and(|node| node.effective_visibility().occupies_layout())
        })
        .count();
    let gap_total = gap.max(0.0) * layout_child_count.saturating_sub(1) as f32;
    let available_extent = (available_extent - gap_total).max(0.0);
    scratch.constraints.clear();

    for child_id in children {
        let node = tree
            .node(*child_id)
            .ok_or(UiTreeError::MissingNode(*child_id))?;
        if !node.effective_visibility().occupies_layout() {
            scratch.constraints.push(collapsed_axis_constraint());
            continue;
        }
        let slot = slot_for_container_child(
            tree,
            slot_index,
            parent_id,
            *child_id,
            linear_container(axis),
        );
        let padding = slot_padding(slot);
        let padding_extent = match axis {
            UiAxis::Horizontal => padding.horizontal(),
            UiAxis::Vertical => padding.vertical(),
        };
        let desired_extent = size_axis_extent(
            UiSize::new(
                node.layout_cache.desired_size.width,
                node.layout_cache.desired_size.height,
            ),
            axis,
        );
        let preserve_stretch = match axis {
            UiAxis::Horizontal => node.layout_stretch_width,
            UiAxis::Vertical => node.layout_stretch_height,
        };
        scratch.constraints.push(linear_main_axis_constraint(
            match axis {
                UiAxis::Horizontal => node.constraints.width,
                UiAxis::Vertical => node.constraints.height,
            },
            desired_extent,
            padding_extent,
            preserve_stretch,
            slot.and_then(|slot| slot.linear_sizing),
        ));
    }

    let UiLinearArrangeScratch {
        constraints,
        resolved,
        priorities,
        active_indices,
    } = scratch;
    solve_axis_constraints_into(
        available_extent,
        constraints,
        resolved,
        priorities,
        active_indices,
    );
    Ok(())
}

#[test]
#[ignore = "release-only linear layout performance evidence"]
fn linear_child_visibility_single_pass_release_p95() {
    const CHILDREN: usize = 256;
    const PROBES: usize = 64;
    const SAMPLES: usize = 17;
    const MARKER: &str = "RUNTIME76_LINEAR_CHILD_VISIBILITY_SINGLE_PASS_BENCH_V1";

    fn measure(
        tree: &UiTree,
        root: UiNodeId,
        children: &[UiNodeId],
        slot_index: &UiLayoutSlotIndex,
        scratch: &mut UiLinearArrangeScratch,
        legacy: bool,
    ) -> u128 {
        let started = Instant::now();
        for _ in 0..PROBES {
            if legacy {
                resolve_linear_child_main_extents_legacy(
                    tree,
                    root,
                    children,
                    UiAxis::Horizontal,
                    4_000.0,
                    4.0,
                    slot_index,
                    scratch,
                )
                .expect("valid linear children");
            } else {
                resolve_linear_child_main_extents(
                    tree,
                    root,
                    children,
                    UiAxis::Horizontal,
                    4_000.0,
                    4.0,
                    slot_index,
                    scratch,
                )
                .expect("valid linear children");
            }
            black_box(&scratch.resolved);
        }
        started.elapsed().as_nanos()
    }

    let root = UiNodeId::new(1);
    let mut tree = UiTree::new(UiTreeId::new("linear-axis-visibility-bench"));
    tree.insert_root(
        UiTreeNode::new(root, UiNodePath::new("root"))
            .with_container(UiContainerKind::HorizontalBox(Default::default())),
    );
    let mut children = Vec::with_capacity(CHILDREN);
    for index in 0..CHILDREN {
        let id = UiNodeId::new((index + 2) as u64);
        let visibility = if index % 5 == 0 {
            UiVisibility::Collapsed
        } else {
            UiVisibility::Visible
        };
        tree.insert_child(
            root,
            UiTreeNode::new(id, UiNodePath::new(format!("child-{index}")))
                .with_visibility(visibility),
        )
        .expect("root exists");
        children.push(id);
    }
    let slot_index = UiLayoutSlotIndex::default();
    let mut scratch = UiLinearArrangeScratch::default();
    let mut legacy_scratch = UiLinearArrangeScratch::default();
    resolve_linear_child_main_extents_legacy(
        &tree,
        root,
        &children,
        UiAxis::Horizontal,
        4_000.0,
        4.0,
        &slot_index,
        &mut legacy_scratch,
    )
    .expect("old resolver accepts benchmark children");
    resolve_linear_child_main_extents(
        &tree,
        root,
        &children,
        UiAxis::Horizontal,
        4_000.0,
        4.0,
        &slot_index,
        &mut scratch,
    )
    .expect("new resolver accepts benchmark children");
    assert_eq!(scratch.constraints, legacy_scratch.constraints);
    assert_eq!(scratch.resolved, legacy_scratch.resolved);
    for _ in 0..4 {
        black_box(measure(
            &tree,
            root,
            &children,
            &slot_index,
            &mut scratch,
            true,
        ));
        black_box(measure(
            &tree,
            root,
            &children,
            &slot_index,
            &mut scratch,
            false,
        ));
    }

    let mut legacy = Vec::with_capacity(SAMPLES);
    let mut single_pass = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        if sample % 2 == 0 {
            legacy.push(measure(
                &tree,
                root,
                &children,
                &slot_index,
                &mut scratch,
                true,
            ));
            single_pass.push(measure(
                &tree,
                root,
                &children,
                &slot_index,
                &mut scratch,
                false,
            ));
        } else {
            single_pass.push(measure(
                &tree,
                root,
                &children,
                &slot_index,
                &mut scratch,
                false,
            ));
            legacy.push(measure(
                &tree,
                root,
                &children,
                &slot_index,
                &mut scratch,
                true,
            ));
        }
    }
    legacy.sort_unstable();
    single_pass.sort_unstable();
    let p95_index = (SAMPLES * 95).div_ceil(100) - 1;
    let legacy_p95 = legacy[p95_index];
    let single_pass_p95 = single_pass[p95_index];
    println!("{MARKER} legacy_p95_ns={legacy_p95} single_pass_p95_ns={single_pass_p95}");
    assert!(
        single_pass_p95.saturating_mul(10) <= legacy_p95.saturating_mul(9),
        "single-pass linear child visibility P95 must be at least 10% faster"
    );
}
