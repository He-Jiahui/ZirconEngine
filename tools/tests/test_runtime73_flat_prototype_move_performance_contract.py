from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
FLAT_NODES = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "ui"
    / "template"
    / "asset"
    / "schema"
    / "flat_nodes.rs"
)
PERFORMANCE_TESTS = FLAT_NODES.parent / "flat_nodes" / "performance_tests.rs"


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    brace = source.index("{", start)
    depth = 0
    for index in range(brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[brace + 1 : index]
    raise AssertionError(f"unterminated Rust function: {signature}")


class Runtime73FlatPrototypeMovePerformanceContract(unittest.TestCase):
    def test_flat_prototype_materialization_moves_owned_node_fields(self) -> None:
        source = FLAT_NODES.read_text(encoding="utf-8")
        materialize = function_body(source, "fn into_raw_prototype(self)")
        materialize_nodes = function_body(source, "fn materialize_prototype_nodes(")
        project = function_body(source, "fn into_node_prototype(")

        self.assertIn("materialize_prototype_nodes(&self.asset.id, self.nodes", materialize)
        self.assertIn("for (node_id, flat_node) in flat_nodes", materialize_nodes)
        self.assertIn("flat_node.into_node_prototype(", materialize_nodes)
        self.assertIn("fn into_node_prototype(\n        self,", source)
        self.assertIn(".children\n            .into_iter()", project)
        self.assertNotIn(".clone()", project)

    def test_reachability_traversal_borrows_node_ids(self) -> None:
        source = FLAT_NODES.read_text(encoding="utf-8")
        validation = function_body(source, "fn validate_reachable_prototype_root<'a>(")

        self.assertIn("enum PrototypeVisitFrame<'a>", source)
        self.assertIn("PrototypeVisitFrame::Enter(root)", validation)
        self.assertIn("PrototypeVisitFrame::Enter(child.child.as_str())", validation)
        self.assertIn(
            "vec![PrototypeVisitState::Unseen; node_handles.len()]", validation
        )
        self.assertNotIn("HashSet", source)
        self.assertNotIn("node_id.clone()", validation)
        self.assertNotIn("child.child.clone()", validation)

    def test_flat_prototype_benchmark_reports_latency_and_clone_owners(self) -> None:
        source = FLAT_NODES.read_text(encoding="utf-8")
        benchmark = PERFORMANCE_TESTS.read_text(encoding="utf-8")

        self.assertIn("#[cfg(test)]\nmod performance_tests;", source)
        self.assertIn("#[ignore = \"performance acceptance benchmark\"]", benchmark)
        self.assertIn("RUNTIME73_FLAT_PROTOTYPE_MOVE_PERF", benchmark)
        self.assertIn("legacy_owned_clones=", benchmark)
        self.assertIn("optimized_owned_clones=", benchmark)
        self.assertIn("legacy_p95_ns=", benchmark)
        self.assertIn("optimized_p95_ns=", benchmark)
        self.assertIn("optimized_p95_ns * 2 <= legacy_p95_ns", benchmark)
        self.assertIn("RUNTIME73_FLAT_PROTOTYPE_VALIDATION_PERF", benchmark)
        self.assertIn("legacy_node_id_clones=", benchmark)
        self.assertIn("optimized_node_id_clones=0", benchmark)
        self.assertIn(
            "optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(60)",
            benchmark,
        )

    def test_flat_prototype_benchmarks_use_four_paired_warmups(self) -> None:
        benchmark = PERFORMANCE_TESTS.read_text(encoding="utf-8")
        materialization = function_body(
            benchmark, "fn flat_prototype_owned_field_move_performance_acceptance()"
        )
        validation = function_body(
            benchmark, "fn flat_prototype_borrowed_validation_performance_acceptance()"
        )

        self.assertIn("const BENCHMARK_WARMUP_PAIRS: usize = 4;", benchmark)
        self.assertIn("for _ in 0..BENCHMARK_WARMUP_PAIRS", materialization)
        self.assertIn("for _ in 0..BENCHMARK_WARMUP_PAIRS", validation)


if __name__ == "__main__":
    unittest.main()
