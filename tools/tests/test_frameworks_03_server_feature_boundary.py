from __future__ import annotations

import sys
import unittest
from pathlib import Path

if sys.version_info >= (3, 11):
    import tomllib
else:
    import tomli as tomllib


REPO_ROOT = Path(__file__).resolve().parents[2]
RUNTIME_MANIFEST = REPO_ROOT / "zircon_runtime" / "Cargo.toml"
RUNTIME_ROOT = REPO_ROOT / "zircon_runtime" / "src" / "lib.rs"
APP_MANIFEST = REPO_ROOT / "zircon_app" / "Cargo.toml"
APP_PLUGIN_GROUPS = REPO_ROOT / "zircon_app" / "src" / "plugins" / "groups.rs"
RUNTIME_CORE_MODULES = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "builtin"
    / "runtime_modules"
    / "core_modules.rs"
)


class Frameworks03ServerFeatureBoundaryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = tomllib.loads(RUNTIME_MANIFEST.read_text(encoding="utf-8"))
        cls.root_source = RUNTIME_ROOT.read_text(encoding="utf-8")
        cls.app_manifest = tomllib.loads(APP_MANIFEST.read_text(encoding="utf-8"))
        cls.app_plugin_groups = APP_PLUGIN_GROUPS.read_text(encoding="utf-8")
        cls.runtime_core_modules = RUNTIME_CORE_MODULES.read_text(encoding="utf-8")

    def test_domain_features_use_current_vocabulary(self) -> None:
        features = self.manifest["features"]
        for feature in (
            "graphics",
            "text",
            "ui",
            "animation",
            "navigation",
            "script",
            "diagnostic-log",
        ):
            self.assertIn(feature, features)

        server = set(features["target-server"])
        self.assertIn("diagnostic-log", server)
        self.assertTrue(
            server.isdisjoint(
                {
                    "graphics",
                    "text",
                    "ui",
                    "animation",
                    "navigation",
                    "script",
                }
            )
        )

    def test_graphics_and_text_dependencies_are_optional(self) -> None:
        dependencies = self.manifest["dependencies"]
        for dependency in (
            "wgpu",
            "naga",
            "raw-window-handle",
            "pollster",
            "glyphon",
            "swash",
            "taffy",
            "ttf-parser",
            "unicode-linebreak",
            "unicode-normalization",
            "unicode-script",
            "unicode-segmentation",
        ):
            entry = dependencies[dependency]
            self.assertIsInstance(entry, dict, dependency)
            self.assertTrue(entry.get("optional"), dependency)

    def test_root_domain_modules_are_cfg_gated(self) -> None:
        required = {
            "animation": "animation",
            "diagnostic_log": "diagnostic-log",
            "dynamic_api": "dynamic-api",
            "graphics": "graphics",
            "navigation": "navigation",
            "render_graph": "graphics",
            "rhi": "graphics",
            "script": "script",
            "ui": "ui",
        }
        for module, feature in required.items():
            self.assertIn(
                f'#[cfg(feature = "{feature}")]\npub mod {module};',
                self.root_source,
                module,
            )
        self.assertIn(
            '#[cfg(feature = "graphics")]\nmod rhi_wgpu;', self.root_source
        )

    def test_app_target_server_does_not_reenable_client_domains(self) -> None:
        features = self.app_manifest["features"]
        server = set(features["target-server"])
        self.assertIn("diagnostic-log", server)
        self.assertTrue(
            server.isdisjoint(
                {
                    "graphics",
                    "text",
                    "ui",
                    "animation",
                    "navigation",
                    "script",
                    "dynamic-api",
                }
            )
        )
        self.assertIn(
            '#[cfg(feature = "graphics")]\n    if include_graphics',
            self.app_plugin_groups,
        )
        self.assertIn(
            '#[cfg(feature = "script")]\n    modules.push', self.app_plugin_groups
        )

    def test_server_runtime_selection_excludes_script_when_script_is_compiled(self) -> None:
        expected_gate = (
            '#[cfg(feature = "script")]\n'
            "    if target != RuntimeTargetMode::ServerRuntime {\n"
            "        modules.push(Arc::new(script::ScriptModule));\n"
            "    }"
        )
        self.assertEqual(self.runtime_core_modules.count(expected_gate), 2)


if __name__ == "__main__":
    unittest.main()
