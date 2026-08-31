from __future__ import annotations

import inspect
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tools.zircon_export import plugin_validate_components
from tools.zircon_export import plugin_validate_event_catalogs
from tools.zircon_export.plugin_validate_common import plugin_manifest_relative_key


class PluginManifestCurrentPathKeyPerformanceContractTests(unittest.TestCase):
    def test_key_normalizes_relative_segments_without_filesystem_resolution(self) -> None:
        root = Path("E:/workspace/plugins")

        self.assertEqual(
            plugin_manifest_relative_key(root, root / "audio" / ".." / "sound" / "plugin.toml"),
            plugin_manifest_relative_key(root, root / "sound" / "plugin.toml"),
        )

    def _assert_index_uses_lexical_candidate_keys(self, function: object) -> None:
        source = inspect.getsource(function)

        self.assertIn("plugin_other_manifest_paths(plugin_root, current_manifest_path)", source)
        self.assertEqual(0, source.count("manifest_path.resolve()"))

    def test_event_catalog_index_avoids_per_manifest_resolve(self) -> None:
        self._assert_index_uses_lexical_candidate_keys(
            plugin_validate_event_catalogs.plugin_validate_event_catalog_namespace_index
        )

    def test_component_index_avoids_per_manifest_resolve(self) -> None:
        self._assert_index_uses_lexical_candidate_keys(
            plugin_validate_components.plugin_validate_component_identity_index
        )


if __name__ == "__main__":
    unittest.main()
