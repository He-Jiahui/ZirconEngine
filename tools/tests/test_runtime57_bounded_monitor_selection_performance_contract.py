from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MONITOR = (
    ROOT
    / "zircon_app/src/entry/runtime_entry_app/window_attributes/monitor.rs"
)
BUILDER = (
    ROOT
    / "zircon_app/src/entry/runtime_entry_app/window_attributes/builder.rs"
)


class Runtime57BoundedMonitorSelectionPerformanceContractTests(unittest.TestCase):
    def test_window_builder_passes_only_descriptor_monitor_demand(self) -> None:
        source = BUILDER.read_text(encoding="utf-8")
        normalized = " ".join(source.split())

        self.assertIn(
            "WindowMonitorContext::for_event_loop(event_loop, descriptor.position, "
            "descriptor.mode)",
            normalized,
        )

    def test_monitor_context_uses_two_fixed_index_slots(self) -> None:
        source = MONITOR.read_text(encoding="utf-8")
        production = source.split("#[cfg(test)]", 1)[0]
        normalized = " ".join(production.split())

        self.assertIn("const INDEXED_MONITOR_SELECTION_CAPACITY: usize = 2;", production)
        self.assertIn(
            "indexed_monitors: [Option<(usize, MonitorHandle)>; "
            "INDEXED_MONITOR_SELECTION_CAPACITY]",
            normalized,
        )
        self.assertIn("requested_monitor_indices(position, mode)", production)
        self.assertIn("event_loop.available_monitors().enumerate()", production)
        self.assertNotIn("Vec<MonitorHandle>", production)
        self.assertNotIn("collect::<Vec", production)

    def test_index_demand_dedup_and_non_index_behavior_have_rust_coverage(self) -> None:
        source = MONITOR.read_text(encoding="utf-8")

        self.assertIn(
            "fn monitor_index_demand_keeps_two_distinct_descriptor_indices()",
            source,
        )
        self.assertIn(
            "fn monitor_index_demand_deduplicates_and_ignores_non_index_selections()",
            source,
        )


if __name__ == "__main__":
    unittest.main()
