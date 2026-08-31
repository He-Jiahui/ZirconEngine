from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
BUILDERS = (
    ROOT
    / "zircon_editor/src/ui/layouts/windows/workbench_host_window"
    / "pane_payload_builders"
)


class EditorPanePayloadBuilderPerformanceContractTests(unittest.TestCase):
    def read_builder(self, name: str) -> str:
        return (BUILDERS / name).read_text(encoding="utf-8")

    def test_diagnostic_builders_borrow_runtime_snapshot(self) -> None:
        for name in ("runtime_diagnostics.rs", "performance_timeline.rs"):
            with self.subTest(name=name):
                source = self.read_builder(name)
                self.assertNotIn(".runtime_diagnostics\n        .cloned()", source)
                self.assertNotIn(
                    "let default_diagnostics = RuntimeDiagnosticsSnapshot::default()",
                    source,
                )
                self.assertIn("match context.runtime_diagnostics", source)

    def test_module_plugin_builder_borrows_owner_and_rows(self) -> None:
        source = self.read_builder("module_plugins.rs")
        compact = "".join(source.split())

        self.assertNotIn("context.module_plugins.cloned()", source)
        self.assertNotIn(".row_data(", source)
        self.assertIn("data.plugins.iter()", compact)

    def test_build_export_builder_borrows_rows(self) -> None:
        source = self.read_builder("build_export.rs")
        compact = "".join(source.split())

        self.assertNotIn(".row_data(", source)
        self.assertIn("data.targets.iter()", compact)


if __name__ == "__main__":
    unittest.main()
