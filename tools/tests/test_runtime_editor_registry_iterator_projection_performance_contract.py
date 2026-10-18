import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class RuntimeEditorRegistryIteratorProjectionPerformanceContractTests(unittest.TestCase):
    def test_runtime_ui_prototype_store_streams_registry_entries(self):
        source = (
            ROOT / "zircon_runtime/src/dynamic_api/session/runtime_ui.rs"
        ).read_text(encoding="utf-8")
        projection = source.split("fn project_ui_prototype_store(", 1)[1].split(
            "\nfn ui_size", 1
        )[0]
        self.assertIn("project.asset_registry().entries_iter()", projection)
        self.assertNotIn("project.asset_registry().entries()", projection)

    def test_editor_asset_rows_stream_registry_entries_into_reserved_output(self):
        source = (ROOT / "zircon_editor/src/core/asset/index.rs").read_text(
            encoding="utf-8"
        )
        projection = source.split("pub fn rows(", 1)[1].split(
            "\n    pub fn row_by_uuid", 1
        )[0]
        self.assertIn("Vec::with_capacity(self.runtime_registry.len())", projection)
        self.assertIn(".entries_iter()", projection)
        self.assertNotIn("self.runtime_registry\n            .entries()", projection)


if __name__ == "__main__":
    unittest.main()
