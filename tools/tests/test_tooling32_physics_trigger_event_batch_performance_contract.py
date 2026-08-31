from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_plugins"
    / "physics"
    / "runtime"
    / "src"
    / "backend"
    / "builtin"
    / "trigger"
    / "scan.rs"
)


class PhysicsTriggerEventBatchPerformanceContractTests(unittest.TestCase):
    def test_trigger_event_projection_preallocates_the_pair_upper_bound(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        function = source.split("fn compute_trigger_events", 1)[1].split(
            "fn collect_current_trigger_pairs", 1
        )[0]

        self.assertIn(
            "Vec::with_capacity(current.len().saturating_add(previous.len()))",
            function,
        )
        self.assertNotIn("let mut events = Vec::new()", function)


if __name__ == "__main__":
    unittest.main()
