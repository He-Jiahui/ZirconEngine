from __future__ import annotations

import unittest
from pathlib import Path


SCRIPT = (
    Path(__file__).resolve().parents[1]
    / "plugin_structure_audits"
    / "retired_ui_assets.py"
)


class RetiredUiAssetScanPerformanceContractTests(unittest.TestCase):
    def test_walks_file_names_without_per_entry_stat(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("for directory, _subdirectories, file_names in os.walk(root_path):", source)
        self.assertNotIn('root_path.rglob("*")', source)
        self.assertNotIn("path.is_file()", source)

    def test_filters_names_before_constructing_paths(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        filter_index = source.index("if is_retired_ui_asset_name(file_name)")
        path_index = source.index("Path(directory, file_name)")

        self.assertLess(filter_index, path_index)


if __name__ == "__main__":
    unittest.main()
