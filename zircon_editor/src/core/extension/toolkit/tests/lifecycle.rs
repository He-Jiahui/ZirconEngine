use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use crate::core::editor_extension::EditorMenuItemDescriptor;
use crate::core::editor_message::DocumentId;
use crate::core::editor_operation::EditorOperationPath;

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
fn document_toolkit_snapshot_retains_menu_contributions_and_supports_instance_lookup() {
    let instance = ToolkitInstanceId::parse("view.asset.11").unwrap();
    let descriptor = DocumentToolkitDescriptor::new(
        DocumentId::new(11),
        instance.clone(),
        "Asset",
        ToolkitLayout::single_tab("asset.layout", "asset.tab").unwrap(),
    )
    .with_menu_items([
        EditorMenuItemDescriptor::for_operation(
            EditorOperationPath::parse("asset.document.validate").unwrap(),
        )
        .with_priority(-10),
        EditorMenuItemDescriptor::for_operation(
            EditorOperationPath::parse("asset.document.reimport").unwrap(),
        ),
    ]);
    let registry = DocumentToolkitRegistry::<()>::default();
    registry
        .register(Arc::new(FixtureToolkit {
            descriptor,
            save: Arc::new(|_| Ok(())),
            descriptor_calls: None,
            drop_callback: None,
        }))
        .unwrap();

    let snapshot = registry.snapshot();
    let projected = snapshot
        .descriptor_for_instance(&instance)
        .expect("registered toolkit should be addressable by its stable instance id");

    assert_eq!(projected.menu_items().len(), 2);
    assert_eq!(
        projected.menu_items()[0].path(),
        "asset/document/asset.document.validate"
    );
    assert_eq!(
        projected.menu_items()[1].path(),
        "asset/document/asset.document.reimport"
    );
}

#[test]
fn document_toolkit_rejects_duplicate_typed_menu_paths_before_publication() {
    let first_operation = EditorOperationPath::parse("asset.validate").unwrap();
    let menu_path =
        crate::core::commands::EditorCommandMenuPath::builtin(&first_operation, "asset", &[]);
    let menu_items = vec![
        EditorMenuItemDescriptor::new(menu_path.clone(), first_operation),
        EditorMenuItemDescriptor::new(
            menu_path,
            EditorOperationPath::parse("asset.validate_again").unwrap(),
        ),
    ];
    let registry = DocumentToolkitRegistry::<()>::default();
    let mut toolkit = FixtureToolkit::new(12, "view.asset.12", |_| Ok(()));
    toolkit.descriptor = toolkit.descriptor.clone().with_menu_items(menu_items);

    let error = registry.register(Arc::new(toolkit)).unwrap_err();

    assert!(matches!(
        error,
        ToolkitRegistryError::DuplicateMenuPath { .. }
    ));
    assert_eq!(registry.snapshot().generation(), 0);
    assert!(registry.snapshot().descriptors().is_empty());
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

#[test]
fn optimization_wave_20260824tu_editor50_toolkit_descriptor_is_captured_once() {
    const TOOLKITS: u64 = 64;

    let descriptor_calls = Arc::new(AtomicUsize::new(0));
    let registry = DocumentToolkitRegistry::<()>::default();
    for document in 1..=TOOLKITS {
        let instance = format!("view.asset.{document}");
        registry
            .register(Arc::new(
                FixtureToolkit::new(document, &instance, |_| Ok(()))
                    .with_descriptor_counter(Arc::clone(&descriptor_calls)),
            ))
            .unwrap();
    }

    assert_eq!(registry.snapshot().descriptors().len(), TOOLKITS as usize);
    assert_eq!(descriptor_calls.load(Ordering::Relaxed), TOOLKITS as usize);
    assert_eq!(registry.clear().unwrap().len(), TOOLKITS as usize);
    assert_eq!(descriptor_calls.load(Ordering::Relaxed), TOOLKITS as usize);
}

#[test]
fn optimization_wave_20260824tu_editor50_toolkit_drop_reenters_after_registry_unlock() {
    let registry = Arc::new(DocumentToolkitRegistry::<()>::default());
    let weak_registry = Arc::downgrade(&registry);
    let (dropped_tx, dropped_rx) = mpsc::sync_channel(1);
    registry
        .register(Arc::new(
            FixtureToolkit::new(7, "view.asset.7", |_| Ok(())).with_drop_callback(move || {
                if let Some(registry) = weak_registry.upgrade() {
                    assert!(registry.snapshot().descriptors().is_empty());
                }
                let _ = dropped_tx.send(());
            }),
        ))
        .unwrap();

    let registry_for_clear = Arc::clone(&registry);
    let (clear_tx, clear_rx) = mpsc::sync_channel(1);
    let clear_thread = thread::spawn(move || {
        let result = registry_for_clear
            .clear()
            .map(|descriptors| descriptors.len());
        let _ = clear_tx.send(result);
    });

    assert_eq!(
        clear_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("toolkit clear must not deadlock a reentrant Drop"),
        Ok(1)
    );
    clear_thread.join().unwrap();
    dropped_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("toolkit must be retired after the empty snapshot is published");
}

#[test]
fn optimization_wave_20260824tu_editor50_toolkit_unregister_drop_reenters_after_registry_unlock() {
    let registry = Arc::new(DocumentToolkitRegistry::<()>::default());
    let weak_registry = Arc::downgrade(&registry);
    let (dropped_tx, dropped_rx) = mpsc::sync_channel(1);
    registry
        .register(Arc::new(
            FixtureToolkit::new(7, "view.asset.7", |_| Ok(())).with_drop_callback(move || {
                if let Some(registry) = weak_registry.upgrade() {
                    assert!(registry.snapshot().descriptors().is_empty());
                }
                let _ = dropped_tx.send(());
            }),
        ))
        .unwrap();

    let registry_for_unregister = Arc::clone(&registry);
    let (unregister_tx, unregister_rx) = mpsc::sync_channel(1);
    let unregister_thread = thread::spawn(move || {
        let result = registry_for_unregister
            .unregister(&ToolkitInstanceId::parse("view.asset.7").unwrap())
            .map(|descriptor| descriptor.map(|descriptor| descriptor.document_id()));
        let _ = unregister_tx.send(result);
    });

    assert_eq!(
        unregister_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("toolkit unregister must not deadlock a reentrant Drop"),
        Ok(Some(DocumentId::new(7)))
    );
    unregister_thread.join().unwrap();
    dropped_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("toolkit must be retired after the empty snapshot is published");
}

#[test]
fn optimization_wave_20260824tu_editor50_toolkit_registry_source_contract() {
    let source = include_str!("../registry.rs");

    assert_eq!(source.matches(".descriptor()").count(), 1);
    assert!(source.contains("descriptor: DocumentToolkitDescriptor"));
    assert!(source.contains("drop(state);\n        drop(retired);"));
    assert!(source.contains("drop(state);\n        drop(entry);"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_wave_20260824tu_editor50_toolkit_descriptor_capture_evidence() {
    const TOOLKITS: u64 = 1_000;
    const MAX_ELAPSED_NS: u128 = 3_000_000_000;

    let descriptor_calls = Arc::new(AtomicUsize::new(0));
    let registry = DocumentToolkitRegistry::<()>::default();
    let started = Instant::now();
    for document in 1..=TOOLKITS {
        let instance = format!("view.asset.{document}");
        registry
            .register(Arc::new(
                FixtureToolkit::new(document, &instance, |_| Ok(()))
                    .with_descriptor_counter(Arc::clone(&descriptor_calls)),
            ))
            .unwrap();
    }
    let closed = registry.clear().unwrap().len();
    let elapsed_ns = started.elapsed().as_nanos();
    let optimized_descriptor_calls = descriptor_calls.load(Ordering::Relaxed) as u64;

    assert_eq!(closed, TOOLKITS as usize);
    assert_eq!(optimized_descriptor_calls, TOOLKITS);
    assert!(
        elapsed_ns <= MAX_ELAPSED_NS,
        "toolkit descriptor capture took {elapsed_ns}ns; limit is {MAX_ELAPSED_NS}ns"
    );

    let legacy_descriptor_calls =
        TOOLKITS.saturating_mul(TOOLKITS.saturating_add(1)) / 2 + TOOLKITS;
    let call_reduction_bps = legacy_descriptor_calls
        .saturating_sub(optimized_descriptor_calls)
        .saturating_mul(10_000)
        / legacy_descriptor_calls;
    println!(
        "EDITOR_TOOLKIT_BENCH_V1 toolkits={TOOLKITS} legacy_descriptor_calls={legacy_descriptor_calls} optimized_descriptor_calls={optimized_descriptor_calls} call_reduction_bps={call_reduction_bps} elapsed_ns={elapsed_ns} max_elapsed_ns={MAX_ELAPSED_NS}"
    );

    assert_eq!(descriptor_calls.load(Ordering::Relaxed), TOOLKITS as usize);
}
