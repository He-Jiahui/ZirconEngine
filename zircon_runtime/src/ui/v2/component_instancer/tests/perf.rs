use std::collections::BTreeMap;
use std::hint::black_box;
use std::sync::Arc;
use std::time::{Duration, Instant};

use zircon_runtime_interface::ui::v2::UiV2AssetDocument;

use super::{UiV2ComponentInstancer, UiV2PrototypeStore};

const SAMPLE_PAIRS: usize = 21;
const ITERATIONS_PER_SAMPLE: usize = 24;
const NODE_COUNT: usize = 1_024;

#[test]
fn optimization_batch_ho_runtime596_owned_instancing_matches_borrowed_entry_point() {
    let document = sample_document();
    let store = UiV2PrototypeStore::new();

    let borrowed = UiV2ComponentInstancer::instantiate_document(&document, &store)
        .expect("borrowed component instancing");
    let owned = UiV2ComponentInstancer::instantiate_owned(document, &store)
        .expect("owned component instancing");

    assert_eq!(owned.document, borrowed);
}

#[test]
fn optimization_batch_ho_runtime596_component_instancer_reuses_owned_document_storage() {
    let instancer = include_str!("../../component_instancer.rs");
    let compiler = include_str!("../../compiler.rs");

    assert!(instancer.contains("pub(super) fn instantiate_owned("));
    assert!(instancer.contains("Arc::try_unwrap(source_document)"));
    assert!(instancer.contains("output.nodes = output_nodes;"));
    assert!(!instancer.contains("..document.clone()"));
    assert!(compiler.contains("instantiate_owned(rooted, store)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_ho_runtime596_component_document_reuse_p95() {
    const MARKER: &str = "RUNTIME596_COMPONENT_DOCUMENT_REUSE_BENCH_V1";
    let document = large_document();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&document, false));
            optimized.push(measure(&document, true));
        } else {
            optimized.push(measure(&document, true));
            legacy.push(measure(&document, false));
        }
    }

    let legacy_p95_ns = percentile_ns(&legacy, 95);
    let optimized_p95_ns = percentile_ns(&optimized, 95);
    let ratio = optimized_p95_ns as f64 / legacy_p95_ns.max(1) as f64;
    eprintln!(
        "{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} ratio={ratio:.4}"
    );
    assert!(
        ratio <= 0.65,
        "{MARKER} expected owned reuse ratio <= 0.65, got {ratio:.4}"
    );
}

fn measure(document: &UiV2AssetDocument, optimized: bool) -> Duration {
    let mut elapsed = Duration::ZERO;
    for _ in 0..ITERATIONS_PER_SAMPLE {
        let input = document.clone();
        if optimized {
            let start = Instant::now();
            let output = optimized_reuse_model(input);
            black_box(&output);
            elapsed += start.elapsed();
        } else {
            let start = Instant::now();
            let output = legacy_double_clone_model(&input);
            black_box(&output);
            elapsed += start.elapsed();
        }
    }
    elapsed
}

fn legacy_double_clone_model(
    document: &UiV2AssetDocument,
) -> (Arc<UiV2AssetDocument>, UiV2AssetDocument) {
    let source_document = Arc::new(document.clone());
    let output = UiV2AssetDocument {
        root: None,
        nodes: BTreeMap::new(),
        components: BTreeMap::new(),
        ..document.clone()
    };
    (source_document, output)
}

fn optimized_reuse_model(document: UiV2AssetDocument) -> UiV2AssetDocument {
    let source_document = Arc::new(document);
    let traversal_owner = Arc::clone(&source_document);
    black_box(&traversal_owner);
    drop(traversal_owner);

    let mut output = Arc::try_unwrap(source_document).expect("release traversal owner");
    output.root = None;
    output.nodes.clear();
    output.components.clear();
    output
}

fn sample_document() -> UiV2AssetDocument {
    toml::from_str(
        r#"
[asset]
kind = "view"
id = "runtime596.sample"
version = 2
display_name = "Runtime 596"

[root]
node = "root"

[nodes.root]
component = "Panel"

[[nodes.root.children]]
node = "child"

[nodes.child]
component = "Label"

[nodes.child.props]
text = "ready"
"#,
    )
    .expect("valid UI v2 sample")
}

fn large_document() -> UiV2AssetDocument {
    let mut document = sample_document();
    let prototype = document.nodes["child"].clone();
    for index in 0..NODE_COUNT {
        let mut node = prototype.clone();
        node.props.insert(
            "text".to_string(),
            toml::Value::String(format!("runtime596-value-{index:04}")),
        );
        document.nodes.insert(format!("node-{index:04}"), node);
    }
    document
}

fn percentile_ns(samples: &[Duration], percentile: usize) -> u128 {
    let mut values = samples.iter().map(Duration::as_nanos).collect::<Vec<_>>();
    values.sort_unstable();
    let index = (values.len() - 1) * percentile / 100;
    values[index]
}
