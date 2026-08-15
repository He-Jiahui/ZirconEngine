use crate::core::editor_message::{
    EditorViewInvalidationMask, SharedEditorMessageBus, ViewDirtySet,
};

use super::fixture::view;

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
