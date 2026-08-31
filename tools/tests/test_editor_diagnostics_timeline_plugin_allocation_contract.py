from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HOTSPOT = ROOT / "zircon_runtime/src/core/runtime/diagnostics/profiling/hotspot.rs"
PANE_CONVERSION = ROOT / "zircon_editor/src/ui/retained_host/ui/pane_data_conversion"


class EditorDiagnosticsTimelinePluginAllocationContractTests(unittest.TestCase):
    def test_hotspot_grouping_borrows_keys_and_selects_percentile(self) -> None:
        source = HOTSPOT.read_text(encoding="utf-8")

        self.assertIn("struct HotspotKey<'a>", source)
        self.assertIn("stream: &'a str", source)
        self.assertIn("HashSet<u64>", source)
        self.assertIn("select_nth_unstable", source)
        self.assertNotIn("self.durations.sort_unstable()", source)

    def test_runtime_status_projection_borrows_payload_text(self) -> None:
        source = (PANE_CONVERSION / "runtime_diagnostics.rs").read_text(
            encoding="utf-8"
        )
        function = source.split("fn runtime_diagnostics_status_lines", 1)[1].split(
            "#[cfg(test)]", 1
        )[0]

        self.assertIn("Vec<&str>", function)
        self.assertNotIn(".cloned()", function)

    def test_plugin_action_projection_borrows_intermediate_fields(self) -> None:
        source = (PANE_CONVERSION / "module_plugins.rs").read_text(encoding="utf-8")
        action_section = source.split("struct ModulePluginRowAction", 1)[1].split(
            "fn module_plugin_node", 1
        )[0]

        self.assertIn("struct ModulePluginRowAction<'a>", source)
        self.assertIn("label: &'a str", source)
        self.assertIn("action_id: &'a str", source)
        self.assertNotIn(".to_string()", action_section)


if __name__ == "__main__":
    unittest.main()
