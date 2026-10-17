//! Runtime commit, real projection IO, and host settlement regressions.
//! Native owner must preregister this ReloadHarness-owned physical E fixture.
use super::*;
use super::super::model_import_settlement::ModelImportSettlementPhase as Phase;
use crate::ui::host::editor_asset_manager::{
    DefaultEditorAssetManager, EditorAssetManager, EditorModelImportProjection,
};
use std::path::PathBuf;
use std::sync::Mutex;
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor,
    AssetKind, AssetManager, FunctionAssetImporter, ImportedAsset, ModelAsset,
};

static MODEL_CALLS: Mutex<Vec<(String, usize)>> = Mutex::new(Vec::new());

fn counted_model(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let key = context.uri.to_string();
    let mut calls = MODEL_CALLS.lock().unwrap();
    if let Some((_, count)) = calls.iter_mut().find(|(uri, _)| uri == &key) {
        *count += 1;
    } else {
        calls.push((key, 1));
    }
    drop(calls);
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Model(ModelAsset {
            uri: context.uri.clone(),
            primitives: Vec::new(),
        }),
    ))
}

struct ModelFixture {
    h: ReloadHarness,
    source: PathBuf,
    preview: PathBuf,
}

impl ModelFixture {
    fn new(prefix: &str) -> Self {
        use std::os::windows::fs::MetadataExt;
        let h = ReloadHarness::new(prefix);
        let manager = h.host.asset_runtime_access.project_asset_manager().unwrap();
        manager.register_asset_importer(FunctionAssetImporter::new(
            AssetImporterDescriptor::new(
                "test.editor.model.settlement", "test.editor.model.settlement", AssetKind::Model, 1,
            ).with_priority(i32::MAX).with_source_extensions(["obj"]),
            counted_model,
        )).unwrap();
        let (project, _) = manager.current_project_generation_snapshot().unwrap().into_parts();
        let cache = std::fs::canonicalize(project.paths().cache_root()).unwrap();
        let root = std::fs::canonicalize(&h._files.root).unwrap();
        assert!(cache.starts_with(&root), "fault must stay inside the owned project");
        assert_eq!(std::fs::symlink_metadata(&cache).unwrap().file_attributes() & 0x400, 0);
        let preview = cache.join("editor-previews");
        let source = h._files.root.join(format!(
            "settlement-{}.obj", h._files.root.file_name().unwrap().to_string_lossy()
        ));
        std::fs::write(&source, "# settlement model first revision\n").unwrap();
        drop(manager);
        Self { h, source, preview }
    }

    fn nodes(&self) -> usize {
        self.h.host.runtime.shell().lock().state.world
            .expect_with_world(|scene| scene.nodes().len())
    }

    fn block_projection(&self) {
        if self.preview.exists() {
            assert!(self.preview.is_dir());
            std::fs::remove_dir_all(&self.preview).unwrap();
        }
        std::fs::write(&self.preview, b"owned projection fault").unwrap();
    }

    fn unblock_projection(&self) {
        assert!(self.preview.is_file());
        std::fs::remove_file(&self.preview).unwrap();
    }

    fn submit(&mut self) {
        self.h.host.submit_model_import_source(self.source.clone()).unwrap();
    }

    fn await_committed_projection_failure(&mut self) -> ProjectImportReceipt {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            self.h.host.poll_model_import();
            let pending = self.h.host.pending_model_import.as_ref()
                .expect("durable result must remain owned by the host");
            if let Some(settlement) = pending.settlement.as_ref() {
                assert_eq!(settlement.phase, Phase::ProjectionRetryRequired,
                    "fixture must reach the actual projection IO failure: {}", settlement.error);
                assert!(!settlement.scene_apply_started);
                let receipt = settlement.receipt.as_ref()
                    .expect("real Runtime job must succeed before projection fails").clone();
                assert!(!receipt.committed_records().is_empty());
                let manager = self.h.host.asset_runtime_access.project_asset_manager().unwrap();
                let (project, _) = manager.current_project_generation_snapshot().unwrap().into_parts();
                assert!(project.catalog_input_generation().sequence() >= receipt.generation_sequence());
                for record in receipt.committed_records() {
                    assert_eq!(project.registry().get_by_locator(record.primary_locator()), Some(record));
                }
                // Runtime resource publication has happened even though the Editor projection failed.
                let resources = self.h.host.resolve_resource_manager().unwrap();
                assert!(resolve_ready_handle::<ModelMarker>(&*resources, receipt.source_uri()).is_ok());
                return receipt;
            }
            assert!(Instant::now() < deadline, "real import worker must finish");
            std::thread::yield_now();
        }
    }

    fn calls(&self, receipt: &ProjectImportReceipt) -> usize {
        MODEL_CALLS.lock().unwrap().iter()
            .find(|(uri, _)| uri == &receipt.source_uri().to_string())
            .map(|(_, count)| *count).unwrap_or(0)
    }

    fn decision(&self) -> DecisionTicket {
        self.h.host.pending_model_import.as_ref().unwrap().settlement.as_ref().unwrap()
            .decision.as_ref().expect("actual decision notification").clone()
    }

    fn choose(&mut self, ticket: &DecisionTicket, option: &str) {
        self.h.host.runtime.context().notifications().decisions().unwrap()
            .resolve(ticket, &DecisionOptionId::parse(option).unwrap()).unwrap();
        self.h.host.poll_model_import();
    }

    fn retry_projection(&mut self) {
        let ticket = self.decision();
        self.choose(&ticket, "retry");
    }

    fn phase(&self) -> Phase {
        self.h.host.pending_model_import.as_ref().unwrap().settlement.as_ref().unwrap().phase
    }
}

impl Drop for ModelFixture {
    fn drop(&mut self) {
        // Shutdown is only cleanup after the test observed the real terminal result.
        // It is never used as a successful import-completion barrier.
        let unfinished = self.h.host.runtime.context().jobs()
            .shutdown(Instant::now() + Duration::from_secs(5));
        if !std::thread::panicking() {
            assert!(unfinished.is_empty(), "all owned fixture jobs must stop before file cleanup");
        }
    }
}

#[test]
fn durable_import_projection_io_failure_retries_only_settlement_once() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut f = ModelFixture::new("model-io-retry");
    let nodes = f.nodes();
    let history = f.h.history();
    let before = f.h.host.editor_asset_manager_at_use_point().unwrap().catalog_snapshot();
    f.block_projection();
    f.submit();
    let receipt = f.await_committed_projection_failure();
    assert_eq!(f.calls(&receipt), 1);
    assert_eq!(f.nodes(), nodes);
    assert_eq!(f.h.history(), history);
    let projected = f.h.host.editor_asset_manager_at_use_point().unwrap().catalog_snapshot();
    assert!(Arc::ptr_eq(&before, &projected), "IO failure cannot publish a partial Editor catalog");
    for _ in 0..3 { f.h.host.poll_model_import(); }
    assert_eq!(f.calls(&receipt), 1);
    assert_eq!(f.nodes(), nodes);
    f.unblock_projection();
    f.retry_projection();
    assert!(f.h.host.pending_model_import.is_none());
    assert_eq!(f.nodes(), nodes + 1);
    assert_ne!(f.h.history(), history);
    for _ in 0..3 { f.h.host.poll_model_import(); }
    assert_eq!(f.nodes(), nodes + 1);
    assert_eq!(f.calls(&receipt), 1);
}

#[test]
fn deferred_and_replaced_decision_tickets_cannot_replay_a_committed_import() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut f = ModelFixture::new("model-expired-ticket");
    let nodes = f.nodes();
    f.block_projection();
    f.submit();
    let receipt = f.await_committed_projection_failure();
    let old = f.decision();
    f.h.host.runtime.context().notifications().decisions().unwrap().cancel(&old).unwrap();
    f.h.host.poll_model_import();
    assert_eq!(f.phase(), Phase::ProjectionRetryRequired);
    assert_eq!(f.calls(&receipt), 1);
    f.h.host.import_model_into_project().unwrap(); // review only; create a fresh exact ticket
    let current = f.decision();
    assert_ne!(current, old);
    let _ = f.h.host.runtime.context().notifications().decisions().unwrap()
        .resolve(&old, &DecisionOptionId::parse("retry").unwrap());
    f.h.host.poll_model_import();
    assert_eq!(f.decision(), current);
    assert_eq!(f.nodes(), nodes);
    f.choose(&current, "defer");
    assert_eq!(f.phase(), Phase::ProjectionRetryRequired);
    f.unblock_projection();
    for _ in 0..3 { f.h.host.poll_model_import(); }
    assert_eq!(f.nodes(), nodes, "defer must not start disk retry");
    f.h.host.import_model_into_project().unwrap();
    f.retry_projection();
    assert!(f.h.host.pending_model_import.is_none());
    assert_eq!(f.nodes(), nodes + 1);
    assert_eq!(f.calls(&receipt), 1);
}

#[test]
fn newer_generation_with_replaced_receipt_record_is_not_publication_proof() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut f = ModelFixture::new("model-record-replaced");
    let nodes = f.nodes();
    f.block_projection();
    f.submit();
    let receipt = f.await_committed_projection_failure();
    f.unblock_projection();
    let activation = f.h.host.pending_model_import.as_ref().unwrap()
        .ticket.project_activation_identity().clone();
    {
        let manager = f.h.host.asset_runtime_access.project_asset_manager().unwrap();
        let (project, _) = manager.current_project_generation_snapshot().unwrap().into_parts();
        std::fs::write(project.source_path_for_uri(receipt.source_uri()).unwrap(),
            "# settlement model different source revision\n").unwrap();
        assert!(manager.import_asset_for_project(&activation, &receipt.source_uri().to_string())
            .unwrap().is_some());
    }
    let editor = f.h.host.editor_asset_manager_at_use_point().unwrap();
    editor.refresh_from_runtime_project().unwrap();
    assert_eq!(editor.model_import_projection(&activation, &receipt),
        EditorModelImportProjection::Superseded,
        "same id or a greater generation cannot stand in for the original record");
    drop(editor);
    f.retry_projection();
    assert_eq!(f.phase(), Phase::Superseded);
    assert_eq!(f.nodes(), nodes);
    assert_eq!(f.calls(&receipt), 2, "only the explicit external reimport ran again");
}

#[test]
fn reopened_same_root_activation_retires_the_old_import_decision() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut f = ModelFixture::new("model-activation");
    let nodes = f.nodes();
    f.block_projection();
    f.submit();
    let receipt = f.await_committed_projection_failure();
    let old = f.decision();
    let activation = f.h.host.pending_model_import.as_ref().unwrap()
        .ticket.project_activation_identity().clone();
    {
        let manager = f.h.host.asset_runtime_access.project_asset_manager().unwrap();
        let (project, _) = manager.current_project_generation_snapshot().unwrap().into_parts();
        let reopened = ProjectManager::open(project.paths().root()).unwrap();
        AssetManager::open_prepared_project(&*manager, reopened).unwrap();
        let (current, _) = manager.current_project_generation_snapshot().unwrap().into_parts();
        assert_ne!(current.activation_identity(), &activation);
    }
    f.h.host.poll_model_import();
    assert_eq!(f.phase(), Phase::Superseded);
    assert_ne!(f.decision(), old);
    let _ = f.h.host.runtime.context().notifications().decisions().unwrap()
        .resolve(&old, &DecisionOptionId::parse("retry").unwrap());
    f.h.host.poll_model_import();
    assert_eq!(f.nodes(), nodes);
    assert_eq!(f.calls(&receipt), 1);
    assert!(f.h.host.pending_model_import.as_ref().unwrap()
        .settlement.as_ref().unwrap().receipt.is_some());
}

#[test]
fn close_retry_preserves_close_intent_until_explicit_resume() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut f = ModelFixture::new("model-close-retry");
    let nodes = f.nodes();
    f.block_projection();
    f.submit();
    let receipt = f.await_committed_projection_failure();
    assert_eq!(f.h.host.project_close_coordinator.phase(),
        crate::ui::host::ProjectCloseCoordinatorPhase::Decision);
    f.h.host.queued_document_save_all = true; // actual close preflight refusal
    assert!(!f.h.host.cancel_pending_model_import());
    f.h.host.poll_model_import();
    assert_eq!(f.phase(), Phase::CloseRetryRequired);
    assert!(f.h.host.pending_model_import.as_ref().unwrap().close_requested);
    let close_retry = f.decision();
    f.choose(&close_retry, "retry");
    assert_eq!(f.phase(), Phase::CloseRetryRequired);
    assert_eq!(f.nodes(), nodes);
    f.unblock_projection();
    for _ in 0..3 { f.h.host.poll_model_import(); }
    assert_eq!(f.phase(), Phase::CloseRetryRequired);
    assert_eq!(f.nodes(), nodes, "a tick cannot turn Close into scene insertion");
    f.h.host.queued_document_save_all = false;
    let resume = f.decision();
    f.choose(&resume, "resume");
    assert!(!f.h.host.pending_model_import.as_ref().unwrap().close_requested);
    assert_eq!(f.phase(), Phase::ProjectionRetryRequired);
    f.retry_projection();
    assert!(f.h.host.pending_model_import.is_none());
    assert_eq!(f.nodes(), nodes + 1);
    assert_eq!(f.calls(&receipt), 1);
}

#[test]
fn scene_apply_started_fence_preserves_receipt_and_forbids_retry() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut f = ModelFixture::new("model-apply-fence");
    let nodes = f.nodes();
    let history = f.h.history();
    f.block_projection();
    f.submit();
    let receipt = f.await_committed_projection_failure();
    let mut pending = f.h.host.pending_model_import.take().unwrap();
    let resources = f.h.host.resolve_resource_manager().unwrap();
    let model = resolve_ready_handle::<ModelMarker>(&*resources, receipt.source_uri()).unwrap();
    let material_uri = ResourceLocator::parse("res://materials/default.zmaterial").unwrap();
    let material = resolve_ready_handle::<MaterialMarker>(&*resources, &material_uri).unwrap();
    drop(resources);
    let settlement = pending.settlement.as_mut().unwrap();
    settlement.phase = Phase::ProjectionPending;
    let mut callbacks = 0;
    // Use the same production callback boundary. The real Scene command succeeds,
    // then this test closure reports Err. This is not an injected production Scene
    // error producer or a claim to cover every possible post-mutation failure.
    let result = settlement.run_scene_apply(
        "replay forbidden".to_owned(), "callback panicked".to_owned(), || {
            callbacks += 1;
            assert!(f.h.host.runtime.import_mesh_asset(model, material, "settlement fixture").unwrap());
            Err("fixture failure after the Scene command committed".to_owned())
        });
    assert!(result.is_err());
    assert_eq!(callbacks, 1);
    assert_eq!(settlement.phase, Phase::RepairRequired);
    assert!(settlement.scene_apply_started);
    assert_eq!(f.nodes(), nodes + 1);
    assert_ne!(f.h.history(), history);
    assert!(settlement.run_scene_apply(
        "replay forbidden".to_owned(), "callback panicked".to_owned(),
        || { callbacks += 1; Ok(true) }).is_err());
    assert_eq!(callbacks, 1);
    f.h.host.pending_model_import = Some(pending);
    f.h.host.import_model_into_project().unwrap();
    let ticket = f.decision();
    assert!(f.h.host.runtime.context().notifications().decisions().unwrap()
        .resolve(&ticket, &DecisionOptionId::parse("retry").unwrap()).is_err());
    for _ in 0..3 { f.h.host.poll_model_import(); }
    assert_eq!(f.phase(), Phase::RepairRequired);
    assert_eq!(f.nodes(), nodes + 1);
    assert_eq!(f.calls(&receipt), 1);
    assert!(f.h.host.pending_model_import.as_ref().unwrap()
        .settlement.as_ref().unwrap().receipt.is_some());
}

#[test]
fn refresh_ok_without_matching_project_projection_is_not_a_receipt_acknowledgement() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut f = ModelFixture::new("model-empty-projection");
    f.block_projection();
    f.submit();
    let receipt = f.await_committed_projection_failure();
    let activation = f.h.host.pending_model_import.as_ref().unwrap()
        .ticket.project_activation_identity().clone();
    let empty_editor = DefaultEditorAssetManager::new();
    assert!(empty_editor.refresh_from_runtime_project().is_ok());
    assert_eq!(EditorAssetManager::model_import_projection(&empty_editor, &activation, &receipt),
        EditorModelImportProjection::Superseded);
    assert_eq!(f.calls(&receipt), 1);
    assert!(!f.h.host.pending_model_import.as_ref().unwrap().settlement.as_ref().unwrap()
        .scene_apply_started);
}


#[test]
fn quiescing_close_failure_never_offers_resume_or_inserts_the_committed_model() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut f = ModelFixture::new("model-quiescing-close");
    let nodes = f.nodes();
    let history = f.h.history();
    f.block_projection();
    f.submit();
    let receipt = f.await_committed_projection_failure();

    assert!(matches!(f.h.host.commit_project_close(),
        Err(crate::ui::retained_host::app::project_close::RetainedProjectCloseError::PendingModelImport)));
    assert_eq!(f.h.host.project_close_coordinator.phase(),
        crate::ui::host::ProjectCloseCoordinatorPhase::Quiescing);
    f.h.host.queued_document_save_all = true;
    f.h.host.poll_model_import();
    assert_eq!(f.phase(), Phase::CloseRetryRequired);
    let ticket = f.decision();
    assert!(f.h.host.runtime.context().notifications().decisions().unwrap()
        .resolve(&ticket, &DecisionOptionId::parse("resume").unwrap()).is_err());
    f.choose(&ticket, "retry"); // actual close retry, not projection retry
    assert_eq!(f.phase(), Phase::CloseRetryRequired);
    assert!(f.h.host.pending_model_import.as_ref().unwrap().close_requested);
    let ticket = f.decision();
    assert!(f.h.host.runtime.context().notifications().decisions().unwrap()
        .resolve(&ticket, &DecisionOptionId::parse("resume").unwrap()).is_err());

    f.unblock_projection();
    for _ in 0..3 { f.h.host.poll_model_import(); }
    assert_eq!(f.nodes(), nodes);
    assert_eq!(f.h.history(), history);
    assert_eq!(f.calls(&receipt), 1);
    assert!(f.h.host.pending_model_import.as_ref().unwrap()
        .settlement.as_ref().unwrap().receipt.is_some());
    assert!(!f.h.host.pending_model_import.as_ref().unwrap()
        .settlement.as_ref().unwrap().scene_apply_started);
}

#[test]
fn stale_resume_ticket_cannot_reopen_a_real_quiescing_close() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut f = ModelFixture::new("model-stale-close-resume");
    let nodes = f.nodes();
    let history = f.h.history();
    f.block_projection();
    f.submit();
    let receipt = f.await_committed_projection_failure();
    f.h.host.queued_document_save_all = true;
    assert!(!f.h.host.cancel_pending_model_import());
    f.h.host.poll_model_import();
    assert_eq!(f.h.host.project_close_coordinator.phase(),
        crate::ui::host::ProjectCloseCoordinatorPhase::Decision);
    let stale = f.decision();
    f.h.host.runtime.context().notifications().decisions().unwrap()
        .resolve(&stale, &DecisionOptionId::parse("resume").unwrap()).unwrap();

    f.h.host.queued_document_save_all = false;
    assert!(matches!(f.h.host.commit_project_close(),
        Err(crate::ui::retained_host::app::project_close::RetainedProjectCloseError::PendingModelImport)));
    assert_eq!(f.h.host.project_close_coordinator.phase(),
        crate::ui::host::ProjectCloseCoordinatorPhase::Quiescing);
    f.h.host.queued_document_save_all = true;
    f.unblock_projection();
    f.h.host.poll_model_import();
    assert_eq!(f.phase(), Phase::CloseRetryRequired);
    assert_ne!(f.decision(), stale);
    assert!(f.h.host.pending_model_import.as_ref().unwrap().close_requested);
    let current = f.decision();
    assert!(f.h.host.runtime.context().notifications().decisions().unwrap()
        .resolve(&current, &DecisionOptionId::parse("resume").unwrap()).is_err());
    for _ in 0..3 { f.h.host.poll_model_import(); }
    assert_eq!(f.nodes(), nodes);
    assert_eq!(f.h.history(), history);
    assert_eq!(f.calls(&receipt), 1);
    assert!(f.h.host.pending_model_import.as_ref().unwrap()
        .settlement.as_ref().unwrap().receipt.is_some());
    assert!(!f.h.host.pending_model_import.as_ref().unwrap()
        .settlement.as_ref().unwrap().scene_apply_started);
}
