use taffy::prelude::Style;
use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::layout::UiFrame;

use super::super::UiTaffyChildContractScope;
use super::{TaffyParentProduct, TaffyParentProductCache};

fn product_with_children(count: usize) -> TaffyParentProduct {
    let node_ids = (0..count)
        .map(|index| UiNodeId::new(index as u64 + 1))
        .collect::<Vec<_>>();
    let styles = vec![Style::default(); count];
    TaffyParentProduct::new(&Style::default(), 1, &node_ids, &styles)
        .expect("test product should build")
}

#[test]
fn failed_exact_update_discards_product_and_previous_snapshot() {
    let parent_id = UiNodeId::new(100);
    let child_ids = [UiNodeId::new(101)];
    let child_styles = [Style::default()];
    let frame = UiFrame::new(0.0, 0.0, 100.0, 50.0);
    let mut cache = TaffyParentProductCache::default();
    let mut output = Vec::new();

    let update = cache
        .compute_child_frames(
            parent_id,
            Style::default(),
            frame,
            &child_ids,
            &child_styles,
            &mut output,
            UiTaffyChildContractScope::Full,
            1,
            None,
        )
        .expect("initial product update should succeed");
    assert_eq!(cache.last_update(parent_id), Some(update));

    assert!(cache
        .compute_child_frames(
            parent_id,
            Style::default(),
            frame,
            &child_ids,
            &child_styles,
            &mut output,
            UiTaffyChildContractScope::Exact,
            2,
            None,
        )
        .is_err());

    assert_eq!(cache.product_counts(), (0, 0));
    assert_eq!(cache.last_update(parent_id), None);
}

#[test]
fn topology_updates_reuse_the_retained_taffy_child_projection() {
    let mut product = product_with_children(16);
    let initial_capacity = product.taffy_children.capacity();
    let initial_pointer = product.taffy_children.as_ptr();

    let reduced_ids = [UiNodeId::new(1), UiNodeId::new(2)];
    let reduced_styles = [Style::default(), Style::default()];
    product
        .update_full(Style::default(), 2, &reduced_ids, &reduced_styles)
        .expect("topology update should succeed");

    assert_eq!(product.taffy_children.len(), reduced_ids.len());
    assert_eq!(product.taffy_children.capacity(), initial_capacity);
    assert_eq!(product.taffy_children.as_ptr(), initial_pointer);
    assert_eq!(
        product.taffy_children,
        product
            .children
            .iter()
            .map(|child| child.taffy_node)
            .collect::<Vec<_>>()
    );
}
