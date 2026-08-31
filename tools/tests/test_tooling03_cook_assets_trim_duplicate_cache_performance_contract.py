from __future__ import annotations

import unittest
from pathlib import Path


SCRIPT = (
    Path(__file__).resolve().parents[1]
    / "zircon_export"
    / "pipeline_report_cook_assets_trim_evidence.py"
)


class CookAssetsTrimDuplicateCachePerformanceContractTests(unittest.TestCase):
    def test_sorts_duplicate_asset_paths_once(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn(
            "ordered_duplicate_assets = sorted(set(duplicate_assets))",
            source,
        )
        self.assertEqual(1, source.count("sorted(set(duplicate_assets))"))
        self.assertIn('"duplicate_assets": ordered_duplicate_assets', source)


if __name__ == "__main__":
    unittest.main()
