use std::any::Any;
use std::collections::BTreeSet;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use crate::core::editing::engine::{
    EditCommandError, EditContext, EditWorldRoute, EditorTransactionEngine, HistoryContextId,
    SelectionSnapshot,
};
use crate::core::editor_message::DocumentId;
use crate::core::extension::{
    DocumentAutosavePayload, DocumentToolkit, DocumentToolkitDescriptor, DocumentToolkitRegistry,
    SaveCtx, ToolkitInstanceId, ToolkitLayout, ToolkitSaveFailure,
};
use crate::core::play::WorldDomain;

use super::{
    SaveDirtyViewCandidate, SaveDirtyViewCompletion, SaveDirtyViewFailure,
    SaveDirtyViewFailureKind, SaveDirtyViewOutcomeStatus, SaveDirtyViewsApplyError,
    SaveDirtyViewsPreflightErrorKind, SaveDirtyViewsRequest,
};
use crate::core::asset::dirty::{DirtyExternalEffectId, DirtyRegistry};

#[derive(Default)]
struct FixtureContext;

impl EditContext for FixtureContext {
    fn capture_world_route(
        &self,
        world_domain: WorldDomain,
    ) -> Result<EditWorldRoute, EditCommandError> {
        Ok(EditWorldRoute::logical(world_domain))
    }

    fn activate_world_route(&mut self, _route: &EditWorldRoute) -> Result<(), EditCommandError> {
        Ok(())
    }

    fn retire_world_route(&mut self, _world_domain: WorldDomain) -> Result<(), EditCommandError> {
        Ok(())
    }

    fn selection_snapshot(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }

    fn restore_selection(&mut self, _snapshot: &SelectionSnapshot) -> Result<(), EditCommandError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct FixtureToolkit {
    descriptor: DocumentToolkitDescriptor,
}

impl DocumentToolkit<()> for FixtureToolkit {
    fn descriptor(&self) -> &DocumentToolkitDescriptor {
        &self.descriptor
    }

    fn validate_references(&self, _host: &()) -> Result<(), ToolkitSaveFailure> {
        Ok(())
    }

    fn save(&self, _host: &(), _context: &mut SaveCtx) -> Result<(), ToolkitSaveFailure> {
        Ok(())
    }

    fn autosave_source_path(&self, _host: &()) -> Result<std::path::PathBuf, ToolkitSaveFailure> {
        Ok("fixture.zdoc".into())
    }

    fn capture_autosave(&self, _host: &()) -> Result<DocumentAutosavePayload, ToolkitSaveFailure> {
        Ok(DocumentAutosavePayload::new("fixture.zdoc", Vec::new()))
    }
}

fn document(value: u64) -> DocumentId {
    DocumentId::new(value)
}

fn instance(value: u64) -> ToolkitInstanceId {
    ToolkitInstanceId::parse(format!("view.asset.{value}")).unwrap()
}

fn effect() -> DirtyExternalEffectId {
    DirtyExternalEffectId::parse("ui.source_buffer").unwrap()
}

fn state() -> (Arc<EditorTransactionEngine>, DirtyRegistry) {
    let transactions = Arc::new(EditorTransactionEngine::new(FixtureContext::default()));
    let dirty = DirtyRegistry::new(Arc::clone(&transactions));
    (transactions, dirty)
}

fn register_toolkit(registry: &DocumentToolkitRegistry<()>, value: u64) {
    let document = document(value);
    registry
        .register(Arc::new(FixtureToolkit {
            descriptor: DocumentToolkitDescriptor::new(
                document,
                instance(value),
                format!("Document {value}"),
                ToolkitLayout::single_tab(
                    format!("layout.document.{value}"),
                    format!("tab.document.{value}"),
                )
                .unwrap(),
            ),
        }))
        .unwrap();
}

fn candidate(
    transactions: &EditorTransactionEngine,
    dirty: &DirtyRegistry,
    value: u64,
    estimated_bytes: u64,
) -> SaveDirtyViewCandidate {
    let document = document(value);
    SaveDirtyViewCandidate::new(
        dirty.snapshot(document).unwrap(),
        instance(value),
        transactions
            .capture_save_token(HistoryContextId::Document(document))
            .unwrap(),
        format!("project://assets/document-{value}.zasset"),
        estimated_bytes,
    )
}

#[test]
fn save_dirty_views_preflight_accumulates_the_whole_batch_before_admission() {
    let (transactions, dirty) = state();
    dirty.register_document(document(1)).unwrap();
    dirty.register_document(document(2)).unwrap();
    dirty.mark_external_effect(document(2), effect()).unwrap();
    let toolkits = DocumentToolkitRegistry::<()>::default();
    register_toolkit(&toolkits, 1);

    let wrong_instance = ToolkitInstanceId::parse("view.asset.wrong").unwrap();
    let clean = dirty.snapshot(document(1)).unwrap();
    let clean_token = transactions
        .capture_save_token(HistoryContextId::Document(document(1)))
        .unwrap();
    let invalid = SaveDirtyViewCandidate::new(
        clean.clone(),
        wrong_instance.clone(),
        clean_token.clone(),
        " project://invalid ",
        u64::MAX,
    )
    .with_writable(false)
    .with_references_valid(false);
    let duplicate = SaveDirtyViewCandidate::new(
        clean,
        wrong_instance,
        clean_token,
        "project://assets/duplicate.zasset",
        1,
    );
    let missing_toolkit = candidate(&transactions, &dirty, 2, 1);

    let report =
        SaveDirtyViewsRequest::prepare(&toolkits.snapshot(), [invalid, duplicate, missing_toolkit])
            .unwrap_err();
    let contains = |kind| {
        report
            .failures()
            .iter()
            .any(|failure| failure.kind() == kind)
    };

    assert!(contains(
        SaveDirtyViewsPreflightErrorKind::DuplicateDocument
    ));
    assert!(contains(SaveDirtyViewsPreflightErrorKind::DocumentClean));
    assert!(contains(SaveDirtyViewsPreflightErrorKind::ToolkitMissing));
    assert!(contains(
        SaveDirtyViewsPreflightErrorKind::ToolkitInstanceMismatch
    ));
    assert!(contains(
        SaveDirtyViewsPreflightErrorKind::InvalidResourceKey
    ));
    assert!(contains(SaveDirtyViewsPreflightErrorKind::ReadOnly));
    assert!(contains(
        SaveDirtyViewsPreflightErrorKind::ReferencePolicyRejected
    ));
    assert!(contains(
        SaveDirtyViewsPreflightErrorKind::EstimatedBytesOverflow
    ));
}

#[test]
fn save_dirty_views_returns_stable_partial_results_and_retries_only_dirty_items() {
    let (transactions, dirty) = state();
    let toolkits = DocumentToolkitRegistry::<()>::default();
    for value in 1..=3 {
        dirty.register_document(document(value)).unwrap();
        dirty
            .mark_external_effect(document(value), effect())
            .unwrap();
        register_toolkit(&toolkits, value);
    }
    let request = SaveDirtyViewsRequest::prepare(
        &toolkits.snapshot(),
        (1..=3).map(|value| candidate(&transactions, &dirty, value, 64)),
    )
    .unwrap();

    dirty.mark_external_effect(document(2), effect()).unwrap();
    let result = request
        .apply_completions(
            [
                (
                    document(1),
                    SaveDirtyViewCompletion::Saved { written_bytes: 61 },
                ),
                (
                    document(2),
                    SaveDirtyViewCompletion::Saved { written_bytes: 62 },
                ),
                (
                    document(3),
                    SaveDirtyViewCompletion::Failed(SaveDirtyViewFailure::new(
                        SaveDirtyViewFailureKind::Write,
                        "disk full",
                    )),
                ),
            ],
            &dirty,
            transactions.as_ref(),
        )
        .unwrap();

    assert!(matches!(
        result.outcomes()[0].status(),
        SaveDirtyViewOutcomeStatus::Saved { written_bytes: 61 }
    ));
    assert_eq!(
        result.outcomes()[1].status(),
        &SaveDirtyViewOutcomeStatus::StaleGeneration
    );
    assert!(matches!(
        result.outcomes()[2].status(),
        SaveDirtyViewOutcomeStatus::Failed(failure)
            if failure.kind() == SaveDirtyViewFailureKind::Write
    ));
    assert!(!dirty.snapshot(document(1)).unwrap().is_dirty());
    assert!(dirty.snapshot(document(2)).unwrap().is_dirty());
    assert!(dirty.snapshot(document(3)).unwrap().is_dirty());
    assert_eq!(
        result.retry_documents().collect::<Vec<_>>(),
        vec![document(2), document(3)]
    );
    assert!(!result.all_saved());
}

#[test]
fn cancelled_dirty_view_keeps_its_document_dirty_for_retry() {
    let (transactions, dirty) = state();
    dirty.register_document(document(1)).unwrap();
    dirty.mark_external_effect(document(1), effect()).unwrap();
    let toolkits = DocumentToolkitRegistry::<()>::default();
    register_toolkit(&toolkits, 1);
    let request = SaveDirtyViewsRequest::prepare(
        &toolkits.snapshot(),
        [candidate(&transactions, &dirty, 1, 64)],
    )
    .unwrap();

    let result = request
        .apply_completions(
            [(document(1), SaveDirtyViewCompletion::Cancelled)],
            &dirty,
            transactions.as_ref(),
        )
        .unwrap();

    assert_eq!(
        result.outcomes()[0].status(),
        &SaveDirtyViewOutcomeStatus::Cancelled
    );
    assert!(dirty.snapshot(document(1)).unwrap().is_dirty());
    assert_eq!(
        result.retry_documents().collect::<Vec<_>>(),
        vec![document(1)]
    );
    assert!(!result.all_saved());
}

#[test]
fn malformed_completion_sets_are_rejected_before_dirty_state_changes() {
    let (transactions, dirty) = state();
    dirty.register_document(document(1)).unwrap();
    dirty.mark_external_effect(document(1), effect()).unwrap();
    let toolkits = DocumentToolkitRegistry::<()>::default();
    register_toolkit(&toolkits, 1);
    let request = SaveDirtyViewsRequest::prepare(
        &toolkits.snapshot(),
        [candidate(&transactions, &dirty, 1, 64)],
    )
    .unwrap();

    assert!(matches!(
        request.clone().apply_completions(
            [(document(9), SaveDirtyViewCompletion::Saved { written_bytes: 1 })],
            &dirty,
            transactions.as_ref(),
        ),
        Err(SaveDirtyViewsApplyError::UnknownCompletion { document: unknown })
            if unknown == document(9)
    ));
    assert!(dirty.snapshot(document(1)).unwrap().is_dirty());

    assert!(matches!(
        request.apply_completions(
            [
                (document(1), SaveDirtyViewCompletion::Saved { written_bytes: 1 }),
                (document(1), SaveDirtyViewCompletion::Cancelled),
            ],
            &dirty,
            transactions.as_ref(),
        ),
        Err(SaveDirtyViewsApplyError::DuplicateCompletion { document: duplicate })
            if duplicate == document(1)
    ));
    assert!(dirty.snapshot(document(1)).unwrap().is_dirty());
}

#[test]
fn optimization_wave_20260824h_editor02_save_preflight_reports_each_sorted_duplicate() {
    let (transactions, dirty) = state();
    let toolkits = DocumentToolkitRegistry::<()>::default();
    for value in 1..=2 {
        dirty.register_document(document(value)).unwrap();
        dirty
            .mark_external_effect(document(value), effect())
            .unwrap();
        register_toolkit(&toolkits, value);
    }

    let report = SaveDirtyViewsRequest::prepare(
        &toolkits.snapshot(),
        [2, 1, 2, 1].map(|value| candidate(&transactions, &dirty, value, 64)),
    )
    .unwrap_err();
    let duplicate_documents = report
        .failures()
        .iter()
        .filter(|failure| failure.kind() == SaveDirtyViewsPreflightErrorKind::DuplicateDocument)
        .map(|failure| failure.document_id())
        .collect::<Vec<_>>();

    assert_eq!(duplicate_documents, vec![document(1), document(2)]);
}

#[test]
fn optimization_wave_20260824h_editor02_save_preflight_uses_sorted_adjacency() {
    const PRODUCTION: &str = include_str!("../save_batch.rs");

    assert!(PRODUCTION.contains("let mut previous_document = None;"));
    assert!(PRODUCTION.contains("previous_document == Some(document)"));
    assert!(!PRODUCTION.contains("let mut seen = BTreeSet::new()"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_wave_20260824h_editor02_save_preflight_adjacent_dedup_evidence() {
    const CANDIDATE_COUNT: usize = 65_536;
    const SAMPLE_COUNT: usize = 21;
    let documents = (0..CANDIDATE_COUNT)
        .map(|index| document((index / 2) as u64))
        .collect::<Vec<_>>();

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || legacy_duplicate_count(&documents),
        || adjacent_duplicate_count(&documents),
    );
    let expected_duplicates = CANDIDATE_COUNT / 2;
    assert_eq!(legacy_duplicate_count(&documents), expected_duplicates);
    assert_eq!(adjacent_duplicate_count(&documents), expected_duplicates);

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT EDITOR02_SAVE_PREFLIGHT_DEDUP_BENCH_V1 candidates={CANDIDATE_COUNT} unique_documents={} samples={SAMPLE_COUNT} sample_order=alternating legacy_set_entries={} optimized_set_entries=0 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} reduction_set_entries_percent=100.0000",
        CANDIDATE_COUNT / 2,
        CANDIDATE_COUNT / 2,
    );
    assert!(
        optimized_p95 * 2 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
    );
}

fn legacy_duplicate_count(documents: &[DocumentId]) -> usize {
    let mut seen = BTreeSet::new();
    documents
        .iter()
        .copied()
        .filter(|document| !seen.insert(*document))
        .count()
}

fn adjacent_duplicate_count(documents: &[DocumentId]) -> usize {
    let mut previous_document = None;
    documents
        .iter()
        .copied()
        .filter(|document| {
            let duplicate = previous_document == Some(*document);
            previous_document = Some(*document);
            duplicate
        })
        .count()
}

fn benchmark_paired_samples<const SAMPLE_COUNT: usize>(
    mut legacy: impl FnMut() -> usize,
    mut optimized: impl FnMut() -> usize,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    sorted[(sorted.len() * percentile).div_ceil(100) - 1]
}
