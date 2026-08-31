from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "examples/woc/native/apps/woc_server/src/fixed_tick_driver.rs"


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


class WocServerMovementTransferPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.advance = function_body(cls.source, "advance")

    def test_advance_transfers_the_validated_vector_without_batch_round_trips(self) -> None:
        self.assertIn("canonicalize_pending_movement", self.advance)
        self.assertNotIn("MovementFrameBatch::new", self.advance)
        self.assertNotIn(".frames().to_vec()", self.advance)
        self.assertNotIn("diagnostic.movement_frames.clone()", self.advance)
        self.assertIn("tick_with_movement(commands, movement_frames)", self.advance)

    def test_fault_diagnostics_keep_exactly_one_movement_copy(self) -> None:
        self.assertEqual(self.advance.count("movement_frames.clone()"), 1)
        self.assertIn("last_failed_input = Some(diagnostic)", self.advance)

    def test_canonicalization_preserves_the_protocol_bound(self) -> None:
        body = function_body(self.source, "canonicalize_pending_movement")
        self.assertIn("MAX_MOVEMENT_FRAMES_PER_TICK", body)
        self.assertIn("MovementInputError::TooManyFrames", body)
        self.assertIn("sort_by_key", body)

    def test_release_gate_emits_raw_paired_samples(self) -> None:
        self.assertIn("WOC_APP05_MOVEMENT_TRANSFER_PERF", self.source)
        self.assertIn("sample_pairs=21", self.source)
        self.assertIn("percentile_method=nearest_rank", self.source)
        self.assertIn("legacy_p50_ns", self.source)
        self.assertIn("transferred_p50_ns", self.source)
        self.assertIn("p50_reduction_percent", self.source)
        self.assertIn("p95_reduction_percent", self.source)
        self.assertIn("p50_reduction_percent >= THRESHOLD_PERCENT", self.source)
        self.assertIn("p95_reduction_percent >= THRESHOLD_PERCENT", self.source)

    def test_release_gate_proves_legacy_and_transferred_order_parity(self) -> None:
        benchmark = function_body(
            self.source,
            "woc_app05_movement_transfer_release_benchmark_evidence",
        )
        self.assertIn("MovementFrameBatch::new(fixture.clone())", benchmark)
        self.assertIn("canonicalize_pending_movement(&mut transferred)", benchmark)
        self.assertIn("assert_eq!(legacy.frames(), transferred.as_slice())", benchmark)


if __name__ == "__main__":
    unittest.main()
