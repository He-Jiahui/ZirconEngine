//! 核对视图脏集按视图分别合并失效掩码，共享总线借入批量更新后不丢失既有状态。
use crate::core::editor_message::{
    EditorViewInvalidationMask, SharedEditorMessageBus, ViewDirtySet,
};

use super::fixture::view;

#[test]
fn invalidation_mask_serde_accepts_defined_bits_and_rejects_unknown_bits() {
    let all_defined = EditorViewInvalidationMask::LAYOUT
        .union(EditorViewInvalidationMask::TREE_STRUCTURE)
        .union(EditorViewInvalidationMask::PRESENTATION_DATA)
        .union(EditorViewInvalidationMask::PAINT_ONLY)
        .union(EditorViewInvalidationMask::POINTER_HOVER)
        .union(EditorViewInvalidationMask::VIEWPORT_IMAGE)
        .union(EditorViewInvalidationMask::HIT_TEST)
        .union(EditorViewInvalidationMask::WINDOW_METRICS)
        .union(EditorViewInvalidationMask::RENDER);
    let encoded = serde_json::to_value(all_defined).expect("defined mask serializes");
    assert_eq!(encoded, serde_json::json!(all_defined.bits()));
    assert_eq!(
        serde_json::from_value::<EditorViewInvalidationMask>(encoded)
            .expect("defined mask deserializes"),
        all_defined
    );

    let unknown_bit = 1_u16 << 9;
    assert!(
        serde_json::from_value::<EditorViewInvalidationMask>(serde_json::json!(unknown_bit))
            .is_err()
    );
    assert!(
        serde_json::from_value::<EditorViewInvalidationMask>(serde_json::json!(
            all_defined.bits() | unknown_bit
        ))
        .is_err()
    );
}

#[test]
fn dirty_set_merges_masks_per_view_and_keeps_views_separate() {
    let scene_view = view("scene.workspace");
    let inspector_view = view("inspector.properties");
    let mut dirty = ViewDirtySet::default();

    dirty.mark(scene_view.clone(), EditorViewInvalidationMask::PAINT_ONLY);
    dirty.mark(scene_view.clone(), EditorViewInvalidationMask::HIT_TEST);
    dirty.mark(inspector_view.clone(), EditorViewInvalidationMask::LAYOUT);

    assert_eq!(dirty.len(), 2);
    assert_eq!(
        dirty.mask_for(&scene_view),
        Some(EditorViewInvalidationMask::PAINT_ONLY.union(EditorViewInvalidationMask::HIT_TEST))
    );
    assert_eq!(
        dirty.mask_for(&inspector_view),
        Some(EditorViewInvalidationMask::LAYOUT)
    );
}

#[test]
fn shared_bus_merges_a_borrowed_dirty_batch_with_existing_view_state() {
    let existing = view("scene.workspace");
    let inspector = view("inspector.properties");
    let bus = SharedEditorMessageBus::default();
    let mut batch = ViewDirtySet::default();

    bus.mark_view_dirty(existing.clone(), EditorViewInvalidationMask::PAINT_ONLY);
    batch.mark_ref(&existing, EditorViewInvalidationMask::HIT_TEST);
    batch.mark_ref(&inspector, EditorViewInvalidationMask::LAYOUT);
    bus.mark_view_dirty_set(&batch);

    let dirty = bus.drain_dirty();
    assert_eq!(dirty.len(), 2);
    assert_eq!(
        dirty.mask_for(&existing),
        Some(EditorViewInvalidationMask::PAINT_ONLY.union(EditorViewInvalidationMask::HIT_TEST))
    );
    assert_eq!(
        dirty.mask_for(&inspector),
        Some(EditorViewInvalidationMask::LAYOUT)
    );
}
