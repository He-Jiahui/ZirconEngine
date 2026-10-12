use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::ui::template::{UiAssetLoader, UiDocumentCompiler, UiPrototypeStoreBuilder};

const SAMPLE_PAIRS: usize = 21;
const ITERATIONS_PER_SAMPLE: usize = 96;
const MAP_ENTRY_COUNT: usize = 512;

#[test]
fn optimization_batch_hp_runtime597_native_frame_preserves_child_order_and_attributes() {
    let prototype = UiAssetLoader::load_flat_prototype_toml_str(
        r##"
[asset]
kind = "layout"
id = "asset://ui/tests/runtime597_native_frame.ui"
version = 3

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Panel"
control_id = "Runtime597Root"
children = [{ child = "first" }, { child = "second" }]

[nodes.first]
kind = "native"
type = "Label"
props = { text = "First" }

[nodes.second]
kind = "native"
type = "Label"
props = { text = "Second" }
"##,
    )
    .expect("valid prototype document");
    let mut builder = UiPrototypeStoreBuilder::new();
    let _ = builder.insert(prototype);
    let store = builder.build().expect("prototype store");

    let compiled = UiDocumentCompiler::default()
        .compile_prototype_asset("asset://ui/tests/runtime597_native_frame.ui", &store)
        .expect("compiled prototype");
    let root = &compiled.template_instance().root;

    assert_eq!(root.control_id.as_deref(), Some("Runtime597Root"));
    assert_eq!(root.children.len(), 2);
    assert_eq!(
        root.children[0]
            .attributes
            .get("text")
            .and_then(toml::Value::as_str),
        Some("First")
    );
    assert_eq!(
        root.children[1]
            .attributes
            .get("text")
            .and_then(toml::Value::as_str),
        Some("Second")
    );
}

#[test]
fn optimization_batch_hp_runtime597_frames_consume_owned_nodes_and_tasks() {
    let source = include_str!("../../prototype_instancer.rs");
    let native_push = source
        .split("fn push_native_frame(")
        .nth(1)
        .expect("native frame push")
        .split("fn push_local_component_frame(")
        .next()
        .expect("native frame push body");
    let native_frame = source
        .split("struct PrototypeNativeFrame")
        .nth(1)
        .expect("native frame")
        .split("struct PrototypeComponentFillsFrame")
        .next()
        .expect("native frame fields");
    let component_frame = source
        .split("struct PrototypeComponentFillsFrame")
        .nth(1)
        .expect("component fills frame")
        .split("struct PrototypeComponentRootFrame")
        .next()
        .expect("component fills frame fields");

    assert!(!source.contains("clone_without_slot_fills"));
    assert!(native_push.contains(".widget_type"));
    assert!(native_push.contains(".take()"));
    assert!(source.matches("frames.extend(child_frames)").count() >= 2);
    assert!(!native_frame.contains("child_mounts:"));
    assert!(!component_frame.contains("child_mounts:"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hp_runtime597_prototype_frame_ownership_p95() {
    const MARKER: &str = "RUNTIME597_PROTOTYPE_FRAME_OWNERSHIP_BENCH_V1";
    let task = large_task();
    let node = large_node();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&task, &node, false));
            optimized.push(measure(&task, &node, true));
        } else {
            optimized.push(measure(&task, &node, true));
            legacy.push(measure(&task, &node, false));
        }
    }

    let legacy_p95_ns = percentile_ns(&legacy, 95);
    let optimized_p95_ns = percentile_ns(&optimized, 95);
    let ratio = optimized_p95_ns as f64 / legacy_p95_ns.max(1) as f64;
    eprintln!(
        "{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} ratio={ratio:.4}"
    );
    assert!(
        ratio <= 0.55,
        "{MARKER} expected owned frame ratio <= 0.55, got {ratio:.4}"
    );
}

fn measure(task: &BenchTask, node: &BenchNode, optimized: bool) -> Duration {
    let mut elapsed = Duration::ZERO;
    for _ in 0..ITERATIONS_PER_SAMPLE {
        let task = task.clone();
        let node = node.clone();
        if optimized {
            let start = Instant::now();
            let output = optimized_frame_scaffold(task, node);
            black_box(&output);
            elapsed += start.elapsed();
        } else {
            let start = Instant::now();
            let output = legacy_frame_scaffold(&task, &node);
            black_box(&output);
            elapsed += start.elapsed();
        }
    }
    elapsed
}

fn legacy_frame_scaffold(
    task: &BenchTask,
    node: &BenchNode,
) -> (BenchTask, BenchNode, Vec<usize>, Vec<usize>, Vec<BenchTask>) {
    let frame_task = task.clone();
    let child_mounts = node.children.clone();
    let frame_node = node.clone();
    let frame_child_mounts = child_mounts.clone();
    let child_tasks = child_mounts.iter().rev().map(|_| task.clone()).collect();
    (
        frame_task,
        frame_node,
        frame_child_mounts,
        child_mounts,
        child_tasks,
    )
}

fn optimized_frame_scaffold(
    mut task: BenchTask,
    node: BenchNode,
) -> (BenchTask, BenchNode, Vec<BenchTask>) {
    let child_tasks = node.children.iter().rev().map(|_| task.clone()).collect();
    task.slot_fill = false;
    (task, node, child_tasks)
}

#[derive(Clone)]
struct BenchTask {
    tokens: BTreeMap<String, String>,
    params: BTreeMap<String, String>,
    bindings: BTreeMap<String, String>,
    slot_fill: bool,
}

#[derive(Clone)]
struct BenchNode {
    payload: BTreeMap<String, String>,
    children: Vec<usize>,
}

fn large_task() -> BenchTask {
    BenchTask {
        tokens: string_map("token"),
        params: string_map("param"),
        bindings: string_map("binding"),
        slot_fill: true,
    }
}

fn large_node() -> BenchNode {
    BenchNode {
        payload: string_map("node"),
        children: vec![1],
    }
}

fn string_map(prefix: &str) -> BTreeMap<String, String> {
    (0..MAP_ENTRY_COUNT)
        .map(|index| {
            (
                format!("{prefix}-key-{index:04}"),
                format!("{prefix}-value-{index:04}"),
            )
        })
        .collect()
}

fn percentile_ns(samples: &[Duration], percentile: usize) -> u128 {
    let mut values = samples.iter().map(Duration::as_nanos).collect::<Vec<_>>();
    values.sort_unstable();
    let index = (values.len() - 1) * percentile / 100;
    values[index]
}
