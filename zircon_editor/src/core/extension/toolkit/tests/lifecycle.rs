use std::sync::Arc;

use crate::core::editor_message::DocumentId;

use super::{assert_document_history, FixtureToolkit};
use crate::core::extension::{
    DefaultWorkbenchPreset, DocumentToolkitDescriptor, DocumentToolkitRegistry, SaveError,
    SaveReason, ToolkitArea, ToolkitAreaSlot, ToolkitInstanceId, ToolkitLayout, ToolkitLayoutError,
    ToolkitRegistryError,
};

#[test]
fn document_toolkit_registry_publishes_stable_instances_and_immutable_snapshots() {
    let registry = DocumentToolkitRegistry::<()>::default();
    registry
        .register(Arc::new(FixtureToolkit::new(7, "view.asset.7", |_| Ok(()))))
        .unwrap();

    let before_close = registry.snapshot();
    assert_eq!(before_close.generation(), 1);
    assert_eq!(before_close.descriptors().len(), 1);
    assert_document_history(&before_close.descriptors()[0]);
    assert_eq!(
        registry.document_for_instance(before_close.descriptors()[0].instance_id()),
        Some(DocumentId::new(7))
    );

    let closed = registry
        .unregister(before_close.descriptors()[0].instance_id())
        .unwrap();
    assert_eq!(closed.unwrap().document_id(), DocumentId::new(7));
    assert!(registry.snapshot().descriptors().is_empty());
    assert_eq!(before_close.descriptors().len(), 1);
}

#[test]
fn document_toolkit_unchanged_snapshot_reuses_published_descriptor_storage() {
    let registry = DocumentToolkitRegistry::<()>::default();
    registry
        .register(Arc::new(FixtureToolkit::new(7, "view.asset.7", |_| Ok(()))))
        .unwrap();

    let first = registry.snapshot();
    let second = registry.snapshot();

    assert_eq!(first.generation(), second.generation());
    assert!(std::ptr::eq(first.descriptors(), second.descriptors()));
}

#[test]
fn document_toolkit_layout_rejects_duplicate_slots_and_unknown_active_tabs() {
    let center =
        ToolkitArea::new(ToolkitAreaSlot::Center, ["document", "preview"], "document").unwrap();
    let duplicate_center =
        ToolkitArea::new(ToolkitAreaSlot::Center, ["properties"], "properties").unwrap();
    assert!(matches!(
        ToolkitLayout::new("layout.asset", [center, duplicate_center]),
        Err(ToolkitLayoutError::DuplicateAreaSlot {
            slot: ToolkitAreaSlot::Center
        })
    ));

    assert!(matches!(
        ToolkitArea::new(ToolkitAreaSlot::Bottom, ["timeline"], "transport"),
        Err(ToolkitLayoutError::ActiveTabNotFound { .. })
    ));
}

#[test]
fn document_toolkit_declares_stable_default_preset_visibility() {
    let descriptor = DocumentToolkitDescriptor::new(
        DocumentId::new(11),
        ToolkitInstanceId::parse("view.asset.11").unwrap(),
        "Asset",
        ToolkitLayout::single_tab("asset.layout", "asset.tab").unwrap(),
    )
    .with_default_presets([
        DefaultWorkbenchPreset::Debug,
        DefaultWorkbenchPreset::Authoring,
        DefaultWorkbenchPreset::Debug,
    ]);

    assert_eq!(
        descriptor.default_presets(),
        &[
            DefaultWorkbenchPreset::Authoring,
            DefaultWorkbenchPreset::Debug,
        ]
    );
}

#[test]
fn document_toolkit_duplicate_keys_are_rejected_without_partial_publication() {
    let registry = DocumentToolkitRegistry::<()>::default();
    registry
        .register(Arc::new(FixtureToolkit::new(7, "view.asset.7", |_| Ok(()))))
        .unwrap();

    let duplicate_document = registry
        .register(Arc::new(FixtureToolkit::new(7, "view.asset.other", |_| {
            Ok(())
        })))
        .unwrap_err();
    assert!(matches!(
        duplicate_document,
        ToolkitRegistryError::DocumentAlreadyRegistered { document }
            if document == DocumentId::new(7)
    ));

    let duplicate_instance = registry
        .register(Arc::new(FixtureToolkit::new(8, "view.asset.7", |_| Ok(()))))
        .unwrap_err();
    assert!(matches!(
        duplicate_instance,
        ToolkitRegistryError::InstanceAlreadyRegistered { .. }
    ));

    let snapshot = registry.snapshot();
    assert_eq!(snapshot.generation(), 1);
    assert_eq!(snapshot.descriptors().len(), 1);
    assert_eq!(snapshot.descriptors()[0].document_id(), DocumentId::new(7));
}

#[test]
fn document_toolkit_clear_is_ordered_and_changes_generation_once() {
    let registry = DocumentToolkitRegistry::<()>::default();
    registry
        .register(Arc::new(FixtureToolkit::new(30, "view.asset.30", |_| {
            Ok(())
        })))
        .unwrap();
    registry
        .register(Arc::new(FixtureToolkit::new(10, "view.asset.10", |_| {
            Ok(())
        })))
        .unwrap();

    let closed = registry.clear().unwrap();
    assert_eq!(
        closed
            .iter()
            .map(|descriptor| descriptor.document_id())
            .collect::<Vec<_>>(),
        vec![DocumentId::new(10), DocumentId::new(30)]
    );
    assert_eq!(registry.snapshot().generation(), 3);
    assert!(registry.snapshot().descriptors().is_empty());

    assert!(registry.clear().unwrap().is_empty());
    assert_eq!(registry.snapshot().generation(), 3);
}

#[test]
fn document_toolkit_registry_allocates_monotonic_document_ids_without_failed_gaps() {
    let registry = DocumentToolkitRegistry::<()>::default();
    assert_eq!(registry.allocate_document_id().unwrap(), DocumentId::new(1));
    registry
        .register(Arc::new(FixtureToolkit::new(40, "view.asset.40", |_| {
            Ok(())
        })))
        .unwrap();

    assert!(matches!(
        registry.register(Arc::new(FixtureToolkit::new(400, "view.asset.40", |_| Ok(
            ()
        )))),
        Err(ToolkitRegistryError::InstanceAlreadyRegistered { .. })
    ));
    assert_eq!(
        registry.allocate_document_id().unwrap(),
        DocumentId::new(41)
    );
}

#[test]
fn document_toolkit_close_lease_blocks_new_saves_and_rolls_back_uncommitted_close() {
    let registry = DocumentToolkitRegistry::<()>::default();
    registry
        .register(Arc::new(FixtureToolkit::new(7, "view.asset.7", |_| Ok(()))))
        .unwrap();
    let instance = registry.snapshot().descriptors()[0].instance_id().clone();
    let generation = registry.snapshot().generation();

    {
        let close = registry.begin_close(&instance).unwrap().unwrap();
        assert_eq!(close.document_id(), DocumentId::new(7));
        assert_eq!(close.instance_id(), &instance);
        assert_eq!(registry.snapshot().generation(), generation);
        assert!(matches!(
            registry.save(DocumentId::new(7), &(), SaveReason::Explicit),
            Err(SaveError::DocumentClosing { document }) if document == DocumentId::new(7)
        ));
        assert!(matches!(
            registry.begin_close(&instance),
            Err(ToolkitRegistryError::CloseAlreadyInProgress { document })
                if document == DocumentId::new(7)
        ));
    }

    registry
        .save(DocumentId::new(7), &(), SaveReason::Explicit)
        .unwrap();
    let close = registry.begin_close(&instance).unwrap().unwrap();
    let descriptor = close.commit().unwrap();
    assert_eq!(descriptor.document_id(), DocumentId::new(7));
    assert_eq!(registry.snapshot().generation(), generation + 1);
    assert!(registry.snapshot().descriptors().is_empty());
}
