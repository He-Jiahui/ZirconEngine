import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
COMMANDS = ROOT / "zircon_hub/src/tauri_app/commands.rs"
FOCUS_REFRESH_GATE = (
    ROOT / "zircon_hub/src/tauri_app/commands/focus_refresh_gate.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(function_name)}\s*\(", source)
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class Hub05FocusRefreshGatePerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.commands = COMMANDS.read_text(encoding="utf-8")
        cls.gate = FOCUS_REFRESH_GATE.read_text(encoding="utf-8")

    def test_commands_move_focus_admission_into_the_gate(self) -> None:
        refresh = function_body(
            self.commands,
            "refresh_recent_projects_on_window_focus",
        )

        self.assertIn("mod focus_refresh_gate;", self.commands)
        self.assertIn("focus_refresh_gate: FocusRefreshGate", self.commands)
        self.assertIn("self.focus_refresh_gate.try_enter()", refresh)
        self.assertIn("let _focus_refresh_permit = focus_refresh_permit;", refresh)
        self.assertNotIn(".swap(", refresh)
        self.assertNotIn(".store(false", refresh)

    def test_duplicate_rejection_is_read_fast_and_first_entry_uses_cas(self) -> None:
        enter = function_body(self.gate, "try_enter")
        normalized = " ".join(enter.split())

        self.assertIn("self.pending.load(Ordering::Acquire)", normalized)
        self.assertIn(
            "compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)",
            normalized,
        )
        self.assertLess(normalized.index(".load("), normalized.index(".compare_exchange("))

    def test_permit_drop_releases_pending_and_panic_reentry_is_covered(self) -> None:
        normalized = " ".join(self.gate.split())

        self.assertIn("impl Drop for FocusRefreshPermit", self.gate)
        self.assertIn("self.pending.store(false, Ordering::Release)", normalized)
        self.assertIn("hub05_focus_refresh_gate_rejects_duplicates_until_drop", self.gate)
        self.assertIn("hub05_focus_refresh_gate_releases_after_worker_panic", self.gate)
        self.assertIn("catch_unwind", self.gate)

    def test_release_benchmark_pins_workload_percentiles_and_raw_samples(self) -> None:
        self.assertIn("hub05_focus_refresh_gate_release_benchmark_evidence", self.gate)
        self.assertIn("PERF_RESULT hub05_focus_refresh_gate", self.gate)
        self.assertIn("attempts=2000000", self.gate)
        self.assertIn("sample_pairs=21", self.gate)
        self.assertIn("threshold_percent=30", self.gate)
        self.assertIn("legacy_raw_ns", self.gate)
        self.assertIn("optimized_raw_ns", self.gate)


if __name__ == "__main__":
    unittest.main()
