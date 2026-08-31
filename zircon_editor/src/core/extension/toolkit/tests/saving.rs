use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

use crate::core::editor_message::DocumentId;

use super::super::save::DocumentSourceWriteReceipt;
use super::FixtureToolkit;
use crate::core::extension::{
    DocumentToolkitRegistry, SaveError, SaveReason, ToolkitRegistryError,
};

#[test]
fn document_toolkit_save_records_context_and_close_removes_the_hook() {
    let registry = DocumentToolkitRegistry::<()>::default();
    registry
        .register(Arc::new(FixtureToolkit::new(
            7,
            "view.asset.7",
            |context| {
                assert_eq!(context.reason(), SaveReason::Explicit);
                context.record_written_bytes(41)?;
                context.record_written_bytes(1)?;
                Ok(())
            },
        )))
        .unwrap();

    let report = registry
        .save(DocumentId::new(7), &(), SaveReason::Explicit)
        .unwrap();
    assert_eq!(report.document_id(), DocumentId::new(7));
    assert_eq!(report.written_bytes(), 42);

    let instance = registry.snapshot().descriptors()[0].instance_id().clone();
    registry.unregister(&instance).unwrap().unwrap();
    assert!(matches!(
        registry.save(DocumentId::new(7), &(), SaveReason::Explicit),
        Err(SaveError::DocumentNotRegistered { document })
            if document == DocumentId::new(7)
    ));
}

#[test]
fn document_toolkit_failed_save_remains_registered_and_can_be_retried() {
    let should_fail = Arc::new(AtomicBool::new(true));
    let should_fail_in_save = Arc::clone(&should_fail);
    let registry = DocumentToolkitRegistry::<()>::default();
    registry
        .register(Arc::new(FixtureToolkit::new(
            7,
            "view.asset.7",
            move |context| {
                if should_fail_in_save.swap(false, Ordering::SeqCst) {
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "read only",
                    )));
                }
                context.record_written_bytes(20)?;
                Ok(())
            },
        )))
        .unwrap();

    assert!(matches!(
        registry.save(DocumentId::new(7), &(), SaveReason::SaveAll),
        Err(SaveError::HookFailed { document, .. }) if document == DocumentId::new(7)
    ));
    assert_eq!(registry.snapshot().descriptors().len(), 1);

    let report = registry
        .save(DocumentId::new(7), &(), SaveReason::SaveAll)
        .unwrap();
    assert_eq!(report.written_bytes(), 20);
}

#[test]
fn reference_validation_rejects_save_before_the_persistence_hook_runs() {
    let save_called = Arc::new(AtomicBool::new(false));
    let save_called_by_hook = Arc::clone(&save_called);
    let registry = DocumentToolkitRegistry::<()>::default();
    registry
        .register(Arc::new(
            FixtureToolkit::new(7, "view.asset.7", move |_| {
                save_called_by_hook.store(true, Ordering::SeqCst);
                Ok(())
            })
            .with_reference_validation(|| {
                Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "dangling reference",
                )))
            }),
        ))
        .unwrap();

    assert!(matches!(
        registry.save(DocumentId::new(7), &(), SaveReason::SaveAll),
        Err(SaveError::ReferenceValidationFailed { document, .. })
            if document == DocumentId::new(7)
    ));
    assert!(!save_called.load(Ordering::SeqCst));
}

#[test]
fn document_save_report_exposes_the_project_source_write_guarantee() {
    let registry = DocumentToolkitRegistry::<()>::default();
    registry
        .register(Arc::new(FixtureToolkit::new(
            7,
            "view.asset.7",
            |context| {
                context.record_serialized_project_source_write(
                    12,
                    DocumentSourceWriteReceipt::fixture_for_report_contract(),
                )?;
                Ok(())
            },
        )))
        .unwrap();

    let report = registry
        .save(DocumentId::new(7), &(), SaveReason::Explicit)
        .unwrap();

    assert!(report.cooperating_source_writes_are_serialized());
    assert!(report.external_conflict_detection_is_best_effort());
    assert_eq!(report.written_bytes(), 12);
}

#[test]
fn document_toolkit_close_rejects_an_in_flight_save_without_holding_the_io_lock() {
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let registry = Arc::new(DocumentToolkitRegistry::<()>::default());
    let entered_for_save = Arc::clone(&entered);
    let release_for_save = Arc::clone(&release);
    registry
        .register(Arc::new(FixtureToolkit::new(
            7,
            "view.asset.7",
            move |_| {
                entered_for_save.wait();
                release_for_save.wait();
                Ok(())
            },
        )))
        .unwrap();
    let instance = registry.snapshot().descriptors()[0].instance_id().clone();
    let open_generation = registry.snapshot().generation();

    let registry_for_save = Arc::clone(&registry);
    let save = thread::spawn(move || {
        registry_for_save.save(DocumentId::new(7), &(), SaveReason::Explicit)
    });
    entered.wait();

    assert!(matches!(
        registry.unregister(&instance),
        Err(ToolkitRegistryError::DocumentBusy {
            document,
            active_saves: 1,
        }) if document == DocumentId::new(7)
    ));
    assert!(matches!(
        registry.save(DocumentId::new(7), &(), SaveReason::SaveAll),
        Err(SaveError::SaveAlreadyInProgress { document })
            if document == DocumentId::new(7)
    ));
    assert!(matches!(
        registry.capture_autosave(DocumentId::new(7), &()),
        Err(SaveError::SaveAlreadyInProgress { document })
            if document == DocumentId::new(7)
    ));
    assert!(matches!(
        registry.clear(),
        Err(ToolkitRegistryError::DocumentsBusy { documents })
            if documents == vec![DocumentId::new(7)]
    ));
    assert_eq!(registry.snapshot().descriptors().len(), 1);
    assert_eq!(registry.snapshot().generation(), open_generation);

    release.wait();
    save.join().unwrap().unwrap();
    assert_eq!(
        registry
            .unregister(&instance)
            .unwrap()
            .unwrap()
            .document_id(),
        DocumentId::new(7)
    );
}

#[test]
fn document_toolkit_panicking_save_releases_the_close_lease() {
    let registry = Arc::new(DocumentToolkitRegistry::<()>::default());
    registry
        .register(Arc::new(FixtureToolkit::new(7, "view.asset.7", |_| {
            panic!("fixture save panic")
        })))
        .unwrap();
    let instance = registry.snapshot().descriptors()[0].instance_id().clone();

    let registry_for_save = Arc::clone(&registry);
    let save = thread::spawn(move || {
        registry_for_save.save(DocumentId::new(7), &(), SaveReason::Explicit)
    });
    assert!(save.join().is_err());

    assert_eq!(
        registry
            .unregister(&instance)
            .unwrap()
            .unwrap()
            .document_id(),
        DocumentId::new(7)
    );
}
