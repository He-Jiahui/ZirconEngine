use super::*;

use crate::core::extension::DocumentSourceWritePublication;
use crate::tests::editor_event::support::EventRuntimeHarness;
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::workbench::view::ViewDescriptorId;
use zircon_runtime::core::manager::ManagerResolver;

#[derive(Clone, Copy)]
pub(crate) enum CompletionAction {
    RemoveView,
    KeepView,
    FailAfterWrite,
    PanicAfterWrite,
}

#[derive(Debug, thiserror::Error)]
#[error("failure after actual source publication")]
struct CompletionHookFailure;

pub(crate) struct CompletionToolkit {
    descriptor: DocumentToolkitDescriptor,
    instance: ViewInstanceId,
    root: std::path::PathBuf,
    source_path: std::path::PathBuf,
    write_source: bool,
    action: CompletionAction,
}

impl DocumentToolkit<EditorUiHost> for CompletionToolkit {
    fn descriptor(&self) -> &DocumentToolkitDescriptor {
        &self.descriptor
    }

    fn validate_references(&self, _host: &EditorUiHost) -> Result<(), ToolkitSaveFailure> {
        Ok(())
    }

    fn save(&self, host: &EditorUiHost, context: &mut SaveCtx) -> Result<(), ToolkitSaveFailure> {
        if self.write_source {
            host.document_toolkits.with_source_write(
                &self.root,
                &self.source_path,
                |lease| {
                    match lease.commit_if_matches(b"before", b"").into_publication()? {
                        DocumentSourceWritePublication::Durable(receipt) => {
                            context.record_serialized_project_source_write(0, receipt)?;
                        }
                        DocumentSourceWritePublication::PublishedNotDurable(source) => {
                            return Err(Box::new(source) as ToolkitSaveFailure);
                        }
                    }
                    Ok(())
                },
            )??;
        } else {
            context.record_written_bytes(12)?;
        }
        match self.action {
            CompletionAction::RemoveView => {
                assert!(host
                    .lock_session()
                    .open_view_instances
                    .remove(&self.instance)
                    .is_some());
                Ok(())
            }
            CompletionAction::KeepView => Ok(()),
            CompletionAction::FailAfterWrite => Err(Box::new(CompletionHookFailure)),
            CompletionAction::PanicAfterWrite => std::panic::panic_any(CompletionHookFailure),
        }
    }

    fn autosave_source_path(
        &self,
        _host: &EditorUiHost,
    ) -> Result<std::path::PathBuf, ToolkitSaveFailure> {
        Ok(self.source_path.clone())
    }

    fn capture_autosave(
        &self,
        _host: &EditorUiHost,
    ) -> Result<DocumentAutosavePayload, ToolkitSaveFailure> {
        Ok(DocumentAutosavePayload::new(
            self.source_path.clone(),
            std::fs::read(&self.source_path)?,
        ))
    }
}

fn save_with_failed_projection(
    write_source: bool,
) -> (
    EditorError,
    Vec<u8>,
    crate::core::editor_message::DocumentId,
    ViewInstanceId,
) {
    let _environment = crate::tests::support::env_lock().lock().unwrap();
    let mut fixture =
        crate::tests::support::OwnedSourceWriteFixture::new("zircon_editor_completion_receipt")
            .unwrap();
    let config_path = fixture.create_file("config.json", b"{}").unwrap();
    let root = fixture.root().to_path_buf();
    let source_path = fixture.create_file("document.zdoc", b"before").unwrap();
    let harness = EventRuntimeHarness::with_enabled_subsystems_at_config_path(config_path, &[]);
    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = manager
        .open_view(ViewDescriptorId::new("editor.hierarchy"), None)
        .unwrap();
    let host = &manager.host;
    let document = host.document_toolkits.allocate_document_id().unwrap();
    let toolkit_instance = ToolkitInstanceId::parse(instance.0.clone()).unwrap();
    host.document_toolkits
        .register(Arc::new(CompletionToolkit {
            descriptor: DocumentToolkitDescriptor::new(
                document,
                toolkit_instance.clone(),
                "Completion receipt",
                ToolkitLayout::single_tab("layout.completion.receipt", "tab.completion.receipt")
                    .unwrap(),
            ),
            instance: instance.clone(),
            root: root.clone(),
            source_path: source_path.clone(),
            write_source,
            action: CompletionAction::RemoveView,
        }))
        .unwrap();
    host.dirty_documents.register_document(document).unwrap();

    let error = host
        .save_document_toolkit_canonical(&instance, SaveReason::Explicit)
        .unwrap_err();
    let contents = std::fs::read(&source_path).unwrap();
    assert!(
        host.document_toolkits
            .unregister(&toolkit_instance)
            .unwrap()
            .is_some(),
        "the actual save lease must settle before completion returns"
    );
    host.dirty_documents.unregister_document(document).unwrap();
    drop(manager);
    drop(harness);
    fixture
        .expect_written_bytes("document.zdoc", if write_source { b"" } else { b"before" })
        .unwrap();
    (error, contents, document, instance)
}

#[test]
fn completion_failure_without_source_receipt_preserves_original_error() {
    let (error, contents, _, instance) = save_with_failed_projection(false);
    match error {
        EditorError::Registry(message) => {
            assert_eq!(message, format!("missing view instance {}", instance.0))
        }
        other => panic!("a count-only hook has no source commit: {other:?}"),
    }
    assert_eq!(contents, b"before");
}

#[test]
fn completion_failure_after_real_zero_byte_write_preserves_report_and_source() {
    let (error, contents, document, instance) = save_with_failed_projection(true);
    let EditorError::DocumentSaveCommitted { report, source } = error else {
        panic!("a real source commit must remain visible on projection failure: {error:?}");
    };
    assert_eq!(report.document_id(), document);
    assert_eq!(report.instance_id().as_str(), instance.0);
    assert_eq!(report.reason(), SaveReason::Explicit);
    assert_eq!(report.written_bytes(), 0);
    assert!(report.cooperating_source_writes_are_serialized());
    match *source {
        EditorError::Registry(message) => {
            assert_eq!(message, format!("missing view instance {}", instance.0))
        }
        other => panic!("the original completion source must remain typed: {other:?}"),
    }
    assert!(contents.is_empty());
}

pub(crate) fn register_completion_toolkit(
    manager: &EditorManager,
    fixture: &mut crate::tests::support::OwnedSourceWriteFixture,
    action: CompletionAction,
) -> (
    crate::core::editor_message::DocumentId,
    ViewInstanceId,
    ToolkitInstanceId,
    std::path::PathBuf,
) {
    let source_path = fixture.create_file("document.zdoc", b"before").unwrap();
    let instance = manager
        .open_view(ViewDescriptorId::new("editor.hierarchy"), None)
        .unwrap();
    let host = &manager.host;
    let document = host.document_toolkits.allocate_document_id().unwrap();
    let toolkit_instance = ToolkitInstanceId::parse(instance.0.clone()).unwrap();
    host.document_toolkits
        .register(Arc::new(CompletionToolkit {
            descriptor: DocumentToolkitDescriptor::new(
                document,
                toolkit_instance.clone(),
                "Registered completion",
                ToolkitLayout::single_tab(
                    "layout.registered.completion",
                    "tab.registered.completion",
                )
                .unwrap(),
            ),
            instance: instance.clone(),
            root: fixture.root().to_path_buf(),
            source_path: source_path.clone(),
            write_source: true,
            action,
        }))
        .unwrap();
    host.dirty_documents.register_document(document).unwrap();
    host.dirty_documents
        .mark_external_effect(
            document,
            crate::core::asset::DirtyExternalEffectId::ui_source_buffer(),
        )
        .unwrap();
    (document, instance, toolkit_instance, source_path)
}

fn with_registered_completion(
    action: CompletionAction,
    body: impl FnOnce(
        &EventRuntimeHarness,
        zircon_runtime::core::runtime::ServiceHandle<EditorManager>,
        crate::core::editor_message::DocumentId,
        ViewInstanceId,
        ToolkitInstanceId,
        &std::path::Path,
    ),
) {
    let _environment = crate::tests::support::env_lock().lock().unwrap();
    let mut fixture =
        crate::tests::support::OwnedSourceWriteFixture::new("registered_batch_completion").unwrap();
    let config = fixture.create_file("config.json", b"{}").unwrap();
    let harness = EventRuntimeHarness::with_enabled_subsystems_at_config_path(config, &[]);
    let original = harness
        .core
        .resolve_manager_handle::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let (document, instance, toolkit, source) = {
        let manager = original.enter().unwrap();
        register_completion_toolkit(&manager, &mut fixture, action)
    };
    body(&harness, original, document, instance, toolkit, &source);
    let actual = std::fs::read(&source).unwrap();
    assert!(
        actual == b"before" || actual.is_empty(),
        "only the registered hook may replace the source"
    );
    fixture
        .expect_written_bytes("document.zdoc", &actual)
        .unwrap();
    drop(harness);
}

fn poll_registered_batch(
    original: &zircon_runtime::core::runtime::ServiceHandle<EditorManager>,
) -> Result<crate::core::asset::SaveDirtyViewsResult, EditorError> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let result = original
            .enter()?
            .poll_dirty_document_save(crate::ui::host::DirtyDocumentSaveOwner::SaveAll);
        match result {
            Ok(Some(result)) => return Ok(result),
            Err(error) => return Err(error),
            Ok(None) if std::time::Instant::now() < deadline => std::thread::yield_now(),
            Ok(None) => panic!("registered save batch did not terminalize before deadline"),
        }
    }
}

#[test]
fn registered_batch_projection_failure_retains_real_result_and_original_error() {
    with_registered_completion(
        CompletionAction::RemoveView,
        |_, original, document, instance, toolkit, path| {
            assert_eq!(
                EditorManager::begin_dirty_document_save(
                    original.clone(),
                    crate::ui::host::DirtyDocumentSaveOwner::SaveAll,
                    [document],
                    SaveReason::SaveAll,
                )
                .unwrap(),
                crate::ui::host::DirtyDocumentSaveStart::Scheduled
            );
            let error = poll_registered_batch(&original).unwrap_err();
            let EditorError::DirtySave {
                source:
                    crate::ui::host::EditorDirtySaveError::CompletionProjection { result, source },
            } = error
            else {
                panic!("postprojection failure must own the actual completed batch: {error:?}");
            };
            assert_eq!(result.outcomes().len(), 1);
            assert_eq!(result.committed_documents(), 1);
            assert!(result.all_saved());
            let report = result.outcomes()[0].save_report().unwrap();
            assert_eq!(report.document_id(), document);
            assert_eq!(report.instance_id(), &toolkit);
            assert_eq!(report.reason(), SaveReason::SaveAll);
            assert_eq!(report.written_bytes(), 0);
            assert!(report.cooperating_source_writes_are_serialized());
            assert!(matches!(*source, EditorError::Registry(ref message)
            if message == &format!("missing view instance {}", instance.0)));
            assert!(std::fs::read(path).unwrap().is_empty());
            let manager = original.enter().unwrap();
            assert!(
                !manager
                    .context()
                    .dirty_documents()
                    .snapshot(document)
                    .unwrap()
                    .is_dirty(),
                "the original lower batch applied before the real projection failed"
            );
            assert_eq!(manager.dirty_document_save_owner(), None);
            assert!(manager
                .host
                .document_toolkits
                .unregister(&toolkit)
                .unwrap()
                .is_some());
        },
    );
}

#[test]
fn registered_batch_preserves_committed_hook_error_and_panic_as_failed_results() {
    for action in [
        CompletionAction::FailAfterWrite,
        CompletionAction::PanicAfterWrite,
    ] {
        with_registered_completion(action, |_, original, document, _, toolkit, path| {
            EditorManager::begin_dirty_document_save(
                original.clone(),
                crate::ui::host::DirtyDocumentSaveOwner::SaveAll,
                [document],
                SaveReason::SaveAll,
            )
            .unwrap();
            let result = poll_registered_batch(&original).unwrap();
            assert!(!result.all_saved());
            assert_eq!(result.committed_documents(), 1);
            let outcome = &result.outcomes()[0];
            let report = outcome.save_report().unwrap();
            assert_eq!(report.document_id(), document);
            assert_eq!(report.instance_id(), &toolkit);
            assert_eq!(report.written_bytes(), 0);
            assert!(report.cooperating_source_writes_are_serialized());
            let crate::core::asset::SaveDirtyViewOutcomeStatus::Failed(failure) = outcome.status()
            else {
                panic!(
                    "a real hook failure cannot become a successful batch: {:?}",
                    outcome.status()
                );
            };
            let error = failure
                .job_error()
                .unwrap()
                .downcast_ref::<EditorError>()
                .unwrap();
            let EditorError::DocumentToolkitSave {
                source: crate::core::extension::SaveError::HookFailed { source, .. },
            } = error
            else {
                panic!(
                    "the original hook failure must survive the actual worker/adapter: {error:?}"
                );
            };
            match action {
                CompletionAction::FailAfterWrite => {
                    assert!(source.downcast_ref::<CompletionHookFailure>().is_some())
                }
                CompletionAction::PanicAfterWrite => assert!(source
                    .downcast_ref::<crate::core::extension::SaveHookPanic>()
                    .is_some()),
                _ => unreachable!(),
            }
            assert!(std::fs::read(path).unwrap().is_empty());
            let manager = original.enter().unwrap();
            assert!(manager
                .context()
                .dirty_documents()
                .snapshot(document)
                .unwrap()
                .is_dirty());
            assert!(manager
                .host
                .document_toolkits
                .unregister(&toolkit)
                .unwrap()
                .is_some());
        });
    }
}

#[test]
fn registered_batch_original_generation_rejects_after_real_deactivation_and_reactivation() {
    with_registered_completion(
        CompletionAction::KeepView,
        |harness, original, document, _, _, path| {
            harness
                .core
                .deactivate_module(crate::ui::host::module::EDITOR_MODULE_NAME)
                .unwrap();
            harness
                .core
                .activate_module(crate::ui::host::module::EDITOR_MODULE_NAME)
                .unwrap();
            assert!(
                original.enter().is_err(),
                "the original registration token must remain stale"
            );
            let error = EditorManager::begin_dirty_document_save(
                original,
                crate::ui::host::DirtyDocumentSaveOwner::SaveAll,
                [document],
                SaveReason::SaveAll,
            )
            .unwrap_err();
            assert!(matches!(error, EditorError::Core { .. }));
            assert_eq!(std::fs::read(path).unwrap(), b"before");
        },
    );
}

#[test]
fn malformed_and_stale_completions_preserve_real_zero_byte_publication() {
    for mode in 0..3 {
        with_registered_completion(
            CompletionAction::KeepView,
            |_, original, document, _, toolkit, path| {
                let manager = original.enter().unwrap();
                let dirty = manager.context().dirty_documents();
                let transactions = manager.context().transactions();
                let request = crate::core::asset::SaveDirtyViewsRequest::prepare(
                    &manager.host.document_toolkit_snapshot(),
                    [crate::core::asset::SaveDirtyViewCandidate::new(
                        dirty.snapshot(document).unwrap(),
                        toolkit.clone(),
                        dirty.capture_save_token(document).unwrap(),
                        path.to_string_lossy().into_owned(),
                        6,
                    )],
                )
                .unwrap();
                let report = manager
                    .host
                    .write_document_toolkit(document, SaveReason::SaveAll)
                    .unwrap();
                assert!(report.cooperating_source_writes_are_serialized());
                assert_eq!(report.written_bytes(), 0);
                if mode == 2 {
                    dirty
                        .mark_external_effect(
                            document,
                            crate::core::asset::DirtyExternalEffectId::ui_source_buffer(),
                        )
                        .unwrap();
                    let result = request
                        .apply_completions(
                            [(
                                document,
                                crate::core::asset::SaveDirtyViewCompletion::SavedDocument {
                                    report,
                                },
                            )],
                            dirty,
                            transactions,
                        )
                        .unwrap();
                    assert_eq!(
                        result.outcomes()[0].status(),
                        &crate::core::asset::SaveDirtyViewOutcomeStatus::StaleGeneration
                    );
                    assert_eq!(
                        result.outcomes()[0].save_report().unwrap().document_id(),
                        document
                    );
                    assert_eq!(result.committed_documents(), 1);
                } else {
                    let unknown = crate::core::editor_message::DocumentId::new(u64::MAX);
                    let first = if mode == 0 { unknown } else { document };
                    let error = request
                        .apply_completions(
                            [
                                (
                                    first,
                                    crate::core::asset::SaveDirtyViewCompletion::Cancelled,
                                ),
                                (
                                    document,
                                    crate::core::asset::SaveDirtyViewCompletion::SavedDocument {
                                        report,
                                    },
                                ),
                            ],
                            dirty,
                            transactions,
                        )
                        .unwrap_err();
                    assert_eq!(error.request().intents()[0].document_id(), document);
                    assert_eq!(error.completions().len(), 2);
                    let (_, crate::core::asset::SaveDirtyViewCompletion::SavedDocument { report }) =
                        &error.completions()[1]
                    else {
                        panic!("rejected batch discarded its actual publication");
                    };
                    assert_eq!(report.document_id(), document);
                    assert_eq!(report.instance_id(), &toolkit);
                    assert!(report.cooperating_source_writes_are_serialized());
                    assert_eq!(report.written_bytes(), 0);
                }
                assert!(dirty.snapshot(document).unwrap().is_dirty());
                assert!(std::fs::read(path).unwrap().is_empty());
                assert!(manager
                    .host
                    .document_toolkits
                    .unregister(&toolkit)
                    .unwrap()
                    .is_some());
            },
        );
    }
}
