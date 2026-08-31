from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
GC_SOURCE = ROOT / "zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/gc.rs"
REPORT_SOURCE = ROOT / "zircon_runtime/src/script/vm/gc_bridge/budget.rs"


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated function: {signature}")


class GcFrameReportingPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        gc_source = GC_SOURCE.read_text(encoding="utf-8")
        report_source = REPORT_SOURCE.read_text(encoding="utf-8")
        cls.gc_step = function_body(
            gc_source,
            "pub fn gc_step(&self, budget: VmGcBudget) -> Result<VmGcStepReport, VmError>",
        )
        cls.from_slots = function_body(
            report_source,
            "pub(crate) fn from_slots(",
        )
        cls.gc_step_compact = " ".join(cls.gc_step.split())
        cls.from_slots_compact = " ".join(cls.from_slots.split())

    def test_gc_step_preallocates_reports_from_pending_slot_count(self) -> None:
        self.assertIn("pending.extend(due_slots);", self.gc_step_compact)
        self.assertIn("let report_capacity = {", self.gc_step_compact)
        self.assertIn("pending.len()", self.gc_step_compact)
        self.assertIn("Vec::with_capacity(report_capacity)", self.gc_step_compact)
        self.assertNotIn("let mut slot_reports = Vec::new();", self.gc_step)

    def test_gc_report_aggregates_all_counters_in_one_pass(self) -> None:
        self.assertEqual(self.from_slots.count(".iter().fold("), 1)
        self.assertIn("pause_micros.saturating_add", self.from_slots_compact)
        self.assertIn("root_count.saturating_add", self.from_slots_compact)
        self.assertIn(
            "cross_boundary_reference_count .saturating_add",
            self.from_slots_compact,
        )


if __name__ == "__main__":
    unittest.main()
