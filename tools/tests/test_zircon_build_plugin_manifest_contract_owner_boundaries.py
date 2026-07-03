import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ZIRCON_BUILD = REPO_ROOT / "tools/zircon_build.py"
ZIRCON_BUILD_PLUGIN_MANIFEST_CONTRACT = (
    REPO_ROOT / "tools/zircon_build_plugin_manifest_contract.py"
)


class ZirconBuildPluginManifestContractOwnerBoundaryTests(unittest.TestCase):
    def test_plugin_manifest_contract_lives_in_plugin_manifest_contract_owner(
        self,
    ):
        self.assertTrue(
            ZIRCON_BUILD_PLUGIN_MANIFEST_CONTRACT.exists(),
            "plugin distribution.forms and module crate manifest contracts belong in zircon_build_plugin_manifest_contract.py",
        )
        build_text = ZIRCON_BUILD.read_text(encoding="utf-8")
        contract_text = ZIRCON_BUILD_PLUGIN_MANIFEST_CONTRACT.read_text(
            encoding="utf-8"
        )

        self.assertIn(
            "from .zircon_build_plugin_manifest_contract import (",
            build_text,
        )
        self.assertIn(
            "from zircon_build_plugin_manifest_contract import (",
            build_text,
        )
        for constant_name in (
            "PLUGIN_DISTRIBUTION_FORM_DIST",
            "PLUGIN_DISTRIBUTION_FORM_EMBED",
            "PLUGIN_DISTRIBUTION_FORMS",
        ):
            self.assertNotIn(
                f"{constant_name} =",
                build_text,
                f"{constant_name} belongs in zircon_build_plugin_manifest_contract.py",
            )
            self.assertIn(f"{constant_name} =", contract_text)

        for function_name in (
            "distribution_table",
            "require_distribution_forms",
            "normalize_optional_string",
            "collect_module_crate_names",
            "append_module_crate",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                build_text,
                f"{function_name} belongs in zircon_build_plugin_manifest_contract.py",
            )
            self.assertIn(f"def {function_name}(", contract_text)

        self.assertIn("distribution.forms", contract_text)
        self.assertIn("modules", contract_text)
        self.assertIn("optional_features", contract_text)
        self.assertIn("feature_extensions", contract_text)
        self.assertLessEqual(
            len(contract_text.splitlines()),
            150,
            "zircon_build_plugin_manifest_contract.py should stay focused on plugin manifest contracts",
        )

    def test_plugin_manifest_contract_owner_preserves_distribution_and_module_semantics(
        self,
    ):
        from tools.zircon_build_plugin_manifest_contract import (
            PLUGIN_DISTRIBUTION_FORM_DIST,
            collect_module_crate_names,
            distribution_table,
            normalize_optional_string,
            require_distribution_forms,
        )

        self.assertEqual(
            {"forms": ["dist"]},
            distribution_table({"distribution": {"forms": ["dist"]}}),
        )
        self.assertEqual({}, distribution_table({"distribution": "invalid"}))
        self.assertEqual("zircon_plugin_native", normalize_optional_string(" zircon_plugin_native "))
        self.assertIsNone(normalize_optional_string("  "))

        forms = require_distribution_forms(
            Path("plugin.toml"),
            {"forms": [PLUGIN_DISTRIBUTION_FORM_DIST, "DIST", "embed"]},
        )
        self.assertEqual(("dist", "embed"), forms)

        with self.assertRaisesRegex(SystemExit, "distribution.forms\\[2\\] must be one of"):
            require_distribution_forms(
                Path("plugin.toml"),
                {"forms": ["dist", "sidecar"]},
            )

        crate_names = collect_module_crate_names(
            {
                "modules": [
                    {"crate_name": "zircon_plugin_main"},
                    "ignored",
                ],
                "optional_features": [
                    {"modules": [{"crate_name": "zircon_plugin_optional"}]},
                    {"modules": "ignored"},
                ],
                "feature_extensions": [
                    {"modules": [{"crate_name": "zircon_plugin_extension"}]},
                ],
            }
        )
        self.assertEqual(
            [
                "zircon_plugin_main",
                "zircon_plugin_optional",
                "zircon_plugin_extension",
            ],
            crate_names,
        )


if __name__ == "__main__":
    unittest.main()
