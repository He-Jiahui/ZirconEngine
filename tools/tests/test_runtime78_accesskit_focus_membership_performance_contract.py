from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/accessibility/accesskit.rs"
PERFORMANCE_TESTS = SOURCE.parent / "accesskit" / "performance_tests.rs"


def rust_function_body(source: str, name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(name)}\s*[^{{]*{{", source, re.DOTALL)
    if match is None:
        raise AssertionError(f"missing Rust function {name}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated Rust function {name}")
    return source[match.end() : index - 1]


class AccessKitFocusMembershipPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.body = rust_function_body(cls.source, "snapshot_to_accesskit_tree_update")
        cls.focus_body = rust_function_body(cls.source, "accesskit_focus_node_id")

    def test_focus_membership_does_not_build_a_tree_index(self) -> None:
        self.assertNotIn("BTreeSet", self.source)
        self.assertNotIn("collect::<BTreeSet", self.focus_body)
        self.assertRegex(
            self.focus_body,
            r"nodes\s*\.iter\(\)\s*\.any\(\|\(node_id,\s*_\)\|\s*node_id\s*==\s*focused\)",
        )

    def test_focus_falls_back_to_the_resolved_root(self) -> None:
        self.assertIn("snapshot.focused", self.body)
        self.assertIn(".map(accesskit_node_id)", self.focus_body)
        self.assertIn(".unwrap_or(root)", self.focus_body)

    def test_output_nodes_and_synthetic_root_are_preserved(self) -> None:
        self.assertIn("let mut nodes = snapshot", self.body)
        self.assertIn("snapshot.roots.len() > 1", self.body)
        self.assertIn("nodes.push((SYNTHETIC_ROOT_NODE_ID", self.body)
        self.assertIn("Some(TreeUpdate", self.body)
        self.assertIn("nodes,", self.body)

    def test_focus_membership_is_shared_with_the_release_gate(self) -> None:
        self.assertIn("fn accesskit_focus_node_id(", self.source)
        self.assertIn(
            "accesskit_focus_node_id(&nodes, snapshot.focused, root)", self.body
        )

    def test_release_gate_reports_raw_paired_latency_samples(self) -> None:
        benchmark = PERFORMANCE_TESTS.read_text(encoding="utf-8")

        self.assertIn("const BENCHMARK_WARMUP_PAIRS: usize = 4;", benchmark)
        self.assertIn("const BENCHMARK_SAMPLE_PAIRS: usize = 21;", benchmark)
        self.assertIn("RUNTIME78_ACCESSKIT_FOCUS_MEMBERSHIP_PERF", benchmark)
        self.assertIn("legacy_samples_ns={:?}", benchmark)
        self.assertIn("optimized_samples_ns={:?}", benchmark)
        self.assertIn("optimized_p50_ns.saturating_mul(10) <= legacy_p50_ns", benchmark)
        self.assertIn(
            "optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(15)",
            benchmark,
        )


if __name__ == "__main__":
    unittest.main()
