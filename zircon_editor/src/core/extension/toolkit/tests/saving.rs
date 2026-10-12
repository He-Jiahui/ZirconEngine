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

#[test]
fn failed_hook_after_real_zero_byte_source_write_preserves_report_and_source() {
    let mut fixture =
        crate::tests::support::OwnedSourceWriteFixture::new("zircon_toolkit_source_write").unwrap();
    let root = fixture.root().to_path_buf();
    let source_path = fixture.create_file("document.zdoc", b"before").unwrap();
    let registry = Arc::new(DocumentToolkitRegistry::<()>::default());
    let registry_for_hook = Arc::downgrade(&registry);
    let root_for_hook = root.clone();
    let path_for_hook = source_path.clone();
    registry
        .register(Arc::new(FixtureToolkit::new(
            7,
            "view.asset.7",
            move |context| {
                let registry = registry_for_hook.upgrade().unwrap();
                registry.with_source_write(&root_for_hook, &path_for_hook, |lease| {
                    let publication = lease.commit_if_matches(b"before", b"").into_publication()?;
                    match publication {
                        super::super::save::DocumentSourceWritePublication::Durable(receipt) => {
                            context.record_serialized_project_source_write(0, receipt)?;
                        }
                        super::super::save::DocumentSourceWritePublication::PublishedNotDurable(
                            source,
                        ) => {
                            return Err(Box::new(source) as super::super::ToolkitSaveFailure);
                        }
                    }
                    Err(Box::new(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "completion rejected after publication",
                    )) as super::super::ToolkitSaveFailure)
                })?
            },
        )))
        .unwrap();

    let error = registry
        .save(DocumentId::new(7), &(), SaveReason::SaveAll)
        .unwrap_err();
    assert_eq!(error.document_id(), DocumentId::new(7));
    let SaveError::HookFailedAfterWrite { report, source } = error else {
        panic!("a real durable zero-byte write must retain its report");
    };
    assert_eq!(report.written_bytes(), 0);
    assert_eq!(report.reason(), SaveReason::SaveAll);
    assert_eq!(report.instance_id().as_str(), "view.asset.7");
    assert!(report.cooperating_source_writes_are_serialized());
    assert_eq!(
        source.downcast_ref::<io::Error>().unwrap().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert_eq!(std::fs::read(&source_path).unwrap(), b"");
    assert_eq!(registry.snapshot().descriptors().len(), 1);
    let instance = registry.snapshot().descriptors()[0].instance_id().clone();
    registry.unregister(&instance).unwrap().unwrap();
    drop(registry);
    fixture.expect_written_bytes("document.zdoc", b"").unwrap();
}

#[test]
fn failed_hook_with_byte_count_without_source_receipt_retains_original_failure() {
    let registry = DocumentToolkitRegistry::<()>::default();
    registry
        .register(Arc::new(FixtureToolkit::new(
            7,
            "view.asset.7",
            |context| {
                context.record_written_bytes(12)?;
                Err(Box::new(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "no serialized source receipt",
                )))
            },
        )))
        .unwrap();

    let error = registry
        .save(DocumentId::new(7), &(), SaveReason::Explicit)
        .unwrap_err();
    let SaveError::HookFailed { document, source } = error else {
        panic!("a byte count must not be accepted as a durable source receipt");
    };
    assert_eq!(document, DocumentId::new(7));
    assert_eq!(
        source.downcast_ref::<io::Error>().unwrap().kind(),
        io::ErrorKind::PermissionDenied
    );
    let instance = registry.snapshot().descriptors()[0].instance_id().clone();
    registry.unregister(&instance).unwrap().unwrap();
}

#[test]
fn byte_count_overflow_after_real_source_write_preserves_guarantee_and_typed_source() {
    let mut fixture =
        crate::tests::support::OwnedSourceWriteFixture::new("zircon_toolkit_source_write").unwrap();
    let root = fixture.root().to_path_buf();
    let source_path = fixture.create_file("document.zdoc", b"before").unwrap();
    let registry = Arc::new(DocumentToolkitRegistry::<()>::default());
    let registry_for_hook = Arc::downgrade(&registry);
    let root_for_hook = root.clone();
    let path_for_hook = source_path.clone();
    registry
        .register(Arc::new(FixtureToolkit::new(
            7,
            "view.asset.7",
            move |context| {
                context.record_written_bytes(u64::MAX)?;
                let registry = registry_for_hook.upgrade().unwrap();
                registry.with_source_write(&root_for_hook, &path_for_hook, |lease| {
                    let publication = lease
                        .commit_if_matches(b"before", b"x")
                        .into_publication()?;
                    match publication {
                        super::super::save::DocumentSourceWritePublication::Durable(receipt) => {
                            context.record_serialized_project_source_write(1, receipt)?;
                            Ok(())
                        }
                        super::super::save::DocumentSourceWritePublication::PublishedNotDurable(
                            source,
                        ) => Err(Box::new(source) as super::super::ToolkitSaveFailure),
                    }
                })?
            },
        )))
        .unwrap();

    let error = registry
        .save(DocumentId::new(7), &(), SaveReason::Explicit)
        .unwrap_err();
    let SaveError::HookFailedAfterWrite { report, source } = error else {
        panic!("overflow cannot erase a real durable source write");
    };
    assert_eq!(report.written_bytes(), u64::MAX);
    assert!(report.cooperating_source_writes_are_serialized());
    assert!(matches!(
        source.downcast_ref::<super::super::save::SaveContextError>(),
        Some(super::super::save::SaveContextError::WrittenByteCountOverflow)
    ));
    assert_eq!(std::fs::read(&source_path).unwrap(), b"x");
    let instance = registry.snapshot().descriptors()[0].instance_id().clone();
    registry.unregister(&instance).unwrap().unwrap();
    drop(registry);
    fixture.expect_written_bytes("document.zdoc", b"x").unwrap();
}

struct OriginalSavePanicPayload {
    identity: Arc<()>,
}

#[test]
fn real_zero_byte_source_commit_then_panic_preserves_report_payload_and_releases_lease() {
    let mut fixture =
        crate::tests::support::OwnedSourceWriteFixture::new("zircon_toolkit_committed_panic")
            .unwrap();
    let root = fixture.root().to_path_buf();
    let source_path = fixture.create_file("document.zdoc", b"before").unwrap();
    let registry = Arc::new(DocumentToolkitRegistry::<()>::default());
    let registry_for_hook = Arc::downgrade(&registry);
    let root_for_hook = root.clone();
    let path_for_hook = source_path.clone();
    let identity = Arc::new(());
    let identity_for_hook = Arc::clone(&identity);
    registry
        .register(Arc::new(FixtureToolkit::new(
            7,
            "view.asset.7",
            move |context| {
                let registry = registry_for_hook.upgrade().unwrap();
                registry.with_source_write(&root_for_hook, &path_for_hook, |lease| {
                    match lease.commit_if_matches(b"before", b"").into_publication()? {
                        super::super::save::DocumentSourceWritePublication::Durable(receipt) => {
                            context.record_serialized_project_source_write(0, receipt)?;
                        }
                        super::super::save::DocumentSourceWritePublication::PublishedNotDurable(
                            source,
                        ) => {
                            return Err(Box::new(source) as super::super::ToolkitSaveFailure);
                        }
                    }
                    std::panic::panic_any(OriginalSavePanicPayload {
                        identity: Arc::clone(&identity_for_hook),
                    });
                })?
            },
        )))
        .unwrap();
    let open_generation = registry.snapshot().generation();

    let error = registry
        .save(DocumentId::new(7), &(), SaveReason::SaveAll)
        .unwrap_err();
    let SaveError::HookFailedAfterWrite { report, source } = error else {
        panic!("the original context must survive a panic after actual publication");
    };
    assert_eq!(report.document_id(), DocumentId::new(7));
    assert_eq!(report.instance_id().as_str(), "view.asset.7");
    assert_eq!(report.reason(), SaveReason::SaveAll);
    assert_eq!(report.written_bytes(), 0);
    assert!(report.cooperating_source_writes_are_serialized());
    let carrier = source
        .downcast::<super::super::save::SaveHookPanic>()
        .unwrap();
    let original = carrier
        .into_payload()
        .downcast::<OriginalSavePanicPayload>()
        .unwrap_or_else(|_| panic!("the original typed panic payload was replaced"));
    assert!(Arc::ptr_eq(&identity, &original.identity));
    assert_eq!(std::fs::read(&source_path).unwrap(), b"");
    assert_eq!(registry.snapshot().generation(), open_generation);
    assert_eq!(registry.snapshot().descriptors().len(), 1);
    let instance = registry.snapshot().descriptors()[0].instance_id().clone();
    // Successful close proves the original SaveLease released after the caught hook unwind.
    assert_eq!(
        registry
            .unregister(&instance)
            .unwrap()
            .unwrap()
            .document_id(),
        DocumentId::new(7)
    );
    drop(registry);
    fixture.expect_written_bytes("document.zdoc", b"").unwrap();
}

#[test]
fn panic_before_source_receipt_resumes_original_payload_and_releases_lease() {
    let registry = DocumentToolkitRegistry::<()>::default();
    let identity = Arc::new(());
    let identity_for_hook = Arc::clone(&identity);
    registry
        .register(Arc::new(FixtureToolkit::new(
            7,
            "view.asset.7",
            move |_| {
                std::panic::panic_any(OriginalSavePanicPayload {
                    identity: Arc::clone(&identity_for_hook),
                })
            },
        )))
        .unwrap();
    let open_generation = registry.snapshot().generation();

    let payload = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        registry.save(DocumentId::new(7), &(), SaveReason::Explicit)
    }))
    .unwrap_err();
    let original = payload
        .downcast::<OriginalSavePanicPayload>()
        .unwrap_or_else(|_| panic!("a prewrite panic must retain its original typed payload"));
    assert!(Arc::ptr_eq(&identity, &original.identity));
    assert_eq!(registry.snapshot().generation(), open_generation);
    let instance = registry.snapshot().descriptors()[0].instance_id().clone();
    assert_eq!(
        registry
            .unregister(&instance)
            .unwrap()
            .unwrap()
            .document_id(),
        DocumentId::new(7)
    );
}
