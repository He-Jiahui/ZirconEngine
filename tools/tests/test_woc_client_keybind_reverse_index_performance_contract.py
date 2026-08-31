from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "examples/woc/native/apps/woc_client/src/input/keybind/bindings.rs"


def function_body(source: str, name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(name)}\s*\([^)]*\)[^{{]*{{", source)
    if match is None:
        raise AssertionError(f"missing function {name}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated function {name}")
    return source[match.end() : index - 1]


class WocClientKeybindReverseIndexPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_keybinds_owns_prebuilt_reverse_indexes(self) -> None:
        self.assertIn("HashMap", self.source)
        self.assertIn("reverse", self.source)
        self.assertIn("held_reverse", self.source)

    def test_dispatch_methods_use_borrowed_index_queries(self) -> None:
        for name in (
            "action_for_combo",
            "edge_action_for_combo",
            "held_action_for_code",
        ):
            with self.subTest(name=name):
                body = function_body(self.source, name)
                self.assertIn(".get(", body)
                self.assertNotIn("KEYBIND_ACTIONS.iter()", body)
                self.assertNotIn("to_string", body)
                self.assertNotIn("to_owned", body)

    def test_all_mutations_refresh_the_reverse_indexes(self) -> None:
        for name in ("bind", "clear", "reset"):
            with self.subTest(name=name):
                self.assertIn("rebuild_reverse_indexes", function_body(self.source, name))

    def test_release_gate_emits_raw_paired_samples(self) -> None:
        self.assertIn("WOC_APP04_KEYBIND_REVERSE_INDEX_PERF", self.source)
        self.assertIn("sample_pairs=21", self.source)
        self.assertIn("percentile_method=nearest_rank", self.source)
        self.assertIn("legacy_p50_ns", self.source)
        self.assertIn("indexed_p50_ns", self.source)
        self.assertIn("p50_reduction_percent", self.source)
        self.assertIn("p95_reduction_percent", self.source)
        self.assertIn("p50_reduction_percent >= THRESHOLD_PERCENT", self.source)
        self.assertIn("p95_reduction_percent >= THRESHOLD_PERCENT", self.source)
        benchmark = function_body(
            self.source,
            "woc_app04_keybind_reverse_index_release_benchmark_evidence",
        )
        self.assertIn("for _ in 0..4", benchmark)


if __name__ == "__main__":
    unittest.main()
