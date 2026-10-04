use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use super::EditorTemplateRegistry;
use crate::ui::template::{EditorTemplateError, EditorTemplateRuntimeService};
use zircon_runtime::ui::template::UiCompiledDocument;

const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const ENTRY_COUNT: usize = 512;

const MINIMAL_DOCUMENT: &str = r#"
[asset]
kind = "layout"
id = "asset://ui/editor/minimal.zui"
version = 1

[root]
node_id = "root"
kind = "native"
type = "Panel"
"#;

fn compiled_document() -> UiCompiledDocument {
    let service = EditorTemplateRuntimeService;
    let document = service.parse_document_source(MINIMAL_DOCUMENT).unwrap();
    service.compile_document(&document).unwrap()
}

#[test]
fn optimization_batch_20260826bs_editor_template_registry_hash_index_preserves_lookup() {
    let mut registry = EditorTemplateRegistry::default();
    registry
        .register_compiled_document("res://ui/editor/minimal.zui", compiled_document())
        .unwrap();

    let _: &HashMap<String, UiCompiledDocument> = &registry.documents;
    assert!(registry
        .compiled_document("res://ui/editor/minimal.zui")
        .is_ok());
    assert_eq!(
        registry.compiled_document("res://ui/editor/missing.zui"),
        Err(EditorTemplateError::MissingDocument {
            document_id: "res://ui/editor/missing.zui".to_string(),
        })
    );
}

#[test]
fn optimization_batch_20260826bs_editor_template_registry_hash_index_preserves_duplicate_error() {
    let mut registry = EditorTemplateRegistry::default();
    let document_id = "res://ui/editor/duplicate.zui";
    registry
        .register_compiled_document(document_id, compiled_document())
        .unwrap();

    assert_eq!(
        registry.register_compiled_document(document_id, compiled_document()),
        Err(EditorTemplateError::DuplicateDocument {
            document_id: document_id.to_string(),
        })
    );
}

fn run_ordered_workload(entries: &BTreeMap<String, usize>, document_id: &str) -> u128 {
    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(entries.get(document_id));
    }
    started.elapsed().as_nanos().max(1)
}

fn run_hash_workload(entries: &HashMap<String, usize>, document_id: &str) -> u128 {
    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(entries.get(document_id));
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &mut [u128], numerator: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * numerator).div_ceil(100).saturating_sub(1);
    samples[rank]
}

#[test]
#[ignore = "release performance gate; managed validation only"]
fn optimization_batch_20260826bs_editor_template_registry_hash_index_p95() {
    let prefix = "editor-template-registry-shared-prefix/".repeat(20);
    let rows = (0..ENTRY_COUNT)
        .map(|index| (format!("res://ui/{prefix}{index:04}.zui"), index))
        .collect::<Vec<_>>();
    let target = rows.last().unwrap().0.clone();
    let ordered = rows.iter().cloned().collect::<BTreeMap<_, _>>();
    let hashed = rows.into_iter().collect::<HashMap<_, _>>();
    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            ordered_samples.push(run_ordered_workload(&ordered, &target));
            hash_samples.push(run_hash_workload(&hashed, &target));
        } else {
            hash_samples.push(run_hash_workload(&hashed, &target));
            ordered_samples.push(run_ordered_workload(&ordered, &target));
        }
    }

    let ordered_p50 = percentile(&mut ordered_samples.clone(), 50);
    let ordered_p95 = percentile(&mut ordered_samples, 95);
    let hash_p50 = percentile(&mut hash_samples.clone(), 50);
    let hash_p95 = percentile(&mut hash_samples, 95);
    println!(
        "EDITOR01_TEMPLATE_REGISTRY_HASH_INDEX_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95}"
    );
    assert!(
        hash_p95 * 100 <= ordered_p95 * 70,
        "HashMap lookup P95 must be at least 30% below BTreeMap lookup: ordered={ordered_p95}ns hash={hash_p95}ns"
    );
}
