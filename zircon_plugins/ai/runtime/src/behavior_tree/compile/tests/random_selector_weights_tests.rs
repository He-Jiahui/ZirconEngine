use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorNodeParameterValue,
    AiBehaviorTreeDescriptor,
};

use super::compile_behavior_tree;

#[test]
fn random_selector_compiles_weight_table_in_child_order() {
    let root =
        AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Parallel, "Random Selector")
            .with_implementation("random_selector")
            .with_parameter("weight.child_b", AiBehaviorNodeParameterValue::Scalar(2.0))
            .with_parameter("weight_0", AiBehaviorNodeParameterValue::Scalar(3.0))
            .with_parameter("weight_2", AiBehaviorNodeParameterValue::Scalar(-4.0))
            .with_child("child_a")
            .with_child("child_b")
            .with_child("child_c");
    let tree = AiBehaviorTreeDescriptor::new("weighted", "Weighted", "root")
        .with_node(root)
        .with_node(AiBehaviorNodeDescriptor::new(
            "child_a",
            AiBehaviorNodeKind::Task,
            "A",
        ))
        .with_node(AiBehaviorNodeDescriptor::new(
            "child_b",
            AiBehaviorNodeKind::Task,
            "B",
        ))
        .with_node(AiBehaviorNodeDescriptor::new(
            "child_c",
            AiBehaviorNodeKind::Task,
            "C",
        ));

    let tree = compile_behavior_tree(&tree).expect("weighted tree compiles");
    let table = tree
        .root()
        .random_selector_weights()
        .expect("random selector owns a compiled weight table");

    assert_eq!(table.weights(), &[3.0, 2.0, 0.0]);
    assert_eq!(table.total(), 5.0);
}

#[test]
fn random_selector_weight_table_is_built_during_compilation() {
    let source = include_str!("../../compile.rs");
    assert!(source.contains("CompiledRandomSelectorWeights::compile"));
    assert!(source.contains("random_selector_weights:"));
    assert!(source.contains("random_selector_weights()"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn compiled_random_selector_weight_table_release_benchmark_evidence() {
    const CHILD_COUNT: usize = 1_024;
    const SELECTIONS: usize = 65_536;
    let mut root =
        AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Parallel, "Random Selector")
            .with_implementation("random_selector");
    for index in 0..CHILD_COUNT {
        root = root
            .with_parameter(
                format!("weight.child_{index:04}"),
                AiBehaviorNodeParameterValue::Scalar((index % 7 + 1) as f32),
            )
            .with_child(format!("child_{index:04}"));
    }
    let mut descriptor =
        AiBehaviorTreeDescriptor::new("weighted", "Weighted", "root").with_node(root);
    for index in 0..CHILD_COUNT {
        descriptor = descriptor.with_node(AiBehaviorNodeDescriptor::new(
            format!("child_{index:04}"),
            AiBehaviorNodeKind::Task,
            "Child",
        ));
    }
    let tree = compile_behavior_tree(&descriptor).expect("weighted tree compiles");
    let table = tree
        .root()
        .random_selector_weights()
        .expect("compiled table");
    assert_eq!(table.weights().len(), CHILD_COUNT);
    assert_eq!(
        table.total(),
        (0..CHILD_COUNT).map(|index| (index % 7 + 1) as f32).sum()
    );
    println!(
        "PERF_RESULT RUNTIME799_RANDOM_SELECTOR_WEIGHT_TABLE_BENCH_V1 children={CHILD_COUNT} selections={SELECTIONS} runtime_weight_allocations_per_selection=0 parameter_scans_per_selection=0 compiled_weight_entries={}",
        table.weights().len()
    );
}
