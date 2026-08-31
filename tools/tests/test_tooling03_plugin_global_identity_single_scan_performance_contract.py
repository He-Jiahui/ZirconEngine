from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tools.zircon_export import plugin_validate


class PluginGlobalIdentitySingleScanPerformanceContractTests(unittest.TestCase):
    def test_all_mode_reads_each_manifest_once_for_both_global_identities(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifests = []
            for index in range(128):
                manifest_path = root / f"plugin-{index:03d}" / "plugin.toml"
                manifest_path.parent.mkdir()
                manifest_path.write_text("id = 'fixture'\n", encoding="utf-8")
                manifests.append(manifest_path)

            reads = 0

            def read_manifest(path: Path, diagnostics: list[str]) -> dict[str, object]:
                nonlocal reads
                reads += 1
                return {
                    "id": path.parent.name,
                    "options": [{"key": "shared.option"}],
                    "asset_importers": [{"id": "shared.importer"}],
                }

            diagnostics: list[str] = []
            with mock.patch.object(plugin_validate, "read_toml", read_manifest):
                plugin_validate.validate_plugin_global_identities(root, diagnostics)

            self.assertEqual(len(manifests), reads)
            self.assertTrue(any("options key shared.option" in item for item in diagnostics))
            self.assertTrue(
                any("asset_importers id shared.importer" in item for item in diagnostics)
            )

    def test_all_mode_uses_the_combined_scan(self) -> None:
        source = Path(plugin_validate.__file__).read_text(encoding="utf-8")
        body = source[source.index("def plugin_validate_all_targets(") :]

        self.assertIn("validate_plugin_global_identities(plugin_root, diagnostics)", body)
        self.assertNotIn("validate_plugin_option_global_keys(plugin_root, diagnostics)", body)
        self.assertNotIn(
            "validate_plugin_asset_importer_global_ids(plugin_root, diagnostics)", body
        )


if __name__ == "__main__":
    unittest.main()
