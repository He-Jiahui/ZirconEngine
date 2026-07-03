import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_BUILD = REPO_ROOT / "tools/zircon_export/plugin_build.py"
PLUGIN_BUILD_ASSET_PACK = REPO_ROOT / "tools/zircon_export/plugin_build_asset_pack.py"
PLUGIN_BUILD_COMMAND = REPO_ROOT / "tools/zircon_export/plugin_build_command.py"
PLUGIN_BUILD_PREFLIGHT = REPO_ROOT / "tools/zircon_export/plugin_build_preflight.py"
PLUGIN_COMMAND = REPO_ROOT / "tools/zircon_export/plugin_command.py"
PLUGIN_BUILD_PACKAGE = REPO_ROOT / "tools/zircon_export/plugin_build_package.py"
PLUGIN_BUILD_SIGNATURE = REPO_ROOT / "tools/zircon_export/plugin_build_signature.py"
ZIRCON_EXPORT_CLI = REPO_ROOT / "tools/zircon_export/cli.py"


class PluginBuildOwnerBoundaryTests(unittest.TestCase):
    def test_plugin_subcommand_dispatch_lives_in_plugin_command_owner(self):
        self.assertTrue(
            PLUGIN_COMMAND.exists(),
            "plugin build/validate subcommand dispatch belongs in plugin_command.py",
        )
        command_text = PLUGIN_COMMAND.read_text(encoding="utf-8")
        cli_text = ZIRCON_EXPORT_CLI.read_text(encoding="utf-8")

        self.assertIn("def dispatch_plugin_command", command_text)
        for owner_entry in (
            "parse_plugin_build_args",
            "parse_plugin_validate_args",
            "run_plugin_build",
            "run_plugin_validate",
        ):
            self.assertIn(owner_entry, command_text)
            self.assertNotIn(
                owner_entry,
                cli_text,
                "cli.py should hand plugin subcommands to plugin_command.py",
            )
        self.assertIn("dispatch_plugin_command", cli_text)

    def test_package_materialization_lives_in_package_owner(self):
        self.assertTrue(
            PLUGIN_BUILD_PACKAGE.exists(),
            "plugin build package directory materialization belongs in plugin_build_package.py",
        )
        build_text = PLUGIN_BUILD.read_text(encoding="utf-8")
        package_text = PLUGIN_BUILD_PACKAGE.read_text(encoding="utf-8")

        for function_name in (
            "materialize_plugin_build_package",
            "write_plugin_build_load_manifest",
            "plugin_build_abi_contract",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                build_text,
                f"{function_name} belongs in plugin_build_package.py",
            )
            self.assertIn(
                f"def {function_name}(",
                package_text,
            )

        self.assertIn(
            "from .plugin_build_package import materialize_plugin_build_package",
            build_text,
            "plugin_build.py should consume package materialization through the package owner",
        )
        self.assertNotIn(
            "import shutil",
            build_text,
            "plugin_build.py should not own package directory file-copy materialization",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
        ):
            self.assertNotIn(
                forbidden_import,
                package_text,
                "package materialization owner must stay independent from build and validate entry owners",
            )

    def test_asset_pack_lives_in_asset_pack_owner(self):
        self.assertTrue(
            PLUGIN_BUILD_ASSET_PACK.exists(),
            "plugin build asset pack materialization belongs in plugin_build_asset_pack.py",
        )
        build_text = PLUGIN_BUILD.read_text(encoding="utf-8")
        asset_pack_text = PLUGIN_BUILD_ASSET_PACK.read_text(encoding="utf-8")

        for function_name in (
            "materialize_plugin_asset_pack",
            "plugin_asset_pack_entries",
            "plugin_asset_pack_command",
            "run_plugin_asset_pack_command",
            "plugin_asset_pack_report_is_clean",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                build_text,
                f"{function_name} belongs in plugin_build_asset_pack.py",
            )
            self.assertIn(
                f"def {function_name}(",
                asset_pack_text,
            )

        self.assertIn(
            "from .plugin_validate_distribution_assets import",
            asset_pack_text,
            "asset pack owner should reuse the PluginValidate retired UI asset suffix helpers",
        )
        self.assertIn(
            "from .plugin_validate_distribution_zui_assets import",
            asset_pack_text,
            "asset pack owner should reuse the PluginValidate .zui document helper",
        )
        self.assertIn(
            "validate_plugin_distribution_zui_asset(",
            asset_pack_text,
            "asset pack owner should share PluginValidate .zui asset.kind semantics",
        )
        self.assertIn(
            "plugin_validate_retired_ui_asset_pattern_suffix",
            asset_pack_text,
            "asset pack owner should reuse the PluginValidate retired UI asset pattern suffix helper",
        )
        self.assertIn(
            "plugin_validate_retired_ui_asset_suffix(",
            asset_pack_text,
        )
        self.assertIn(
            "plugin_validate_retired_ui_asset_pattern_suffix(",
            asset_pack_text,
        )
        self.assertNotIn(
            "plugin_validate_retired_ui_asset_suffix(",
            build_text,
            "plugin_build.py must not own asset suffix convergence checks",
        )
        self.assertNotIn(
            "plugin_validate_retired_ui_asset_pattern_suffix(",
            build_text,
            "plugin_build.py must not own asset suffix convergence checks",
        )
        self.assertNotIn(
            "validate_plugin_distribution_zui_asset(",
            build_text,
            "plugin_build.py must not own .zui asset.kind convergence checks",
        )

        package_text = PLUGIN_BUILD_PACKAGE.read_text(encoding="utf-8")
        self.assertIn(
            "from .plugin_build_asset_pack import materialize_plugin_asset_pack",
            package_text,
            "plugin_build_package.py should consume asset pack materialization through the asset pack owner",
        )
        self.assertNotIn(
            "from .plugin_build_asset_pack import",
            build_text,
            "plugin_build.py should consume asset pack materialization through the package owner",
        )
        for moved_import in (
            "import tempfile",
        ):
            self.assertNotIn(
                moved_import,
                build_text,
                "plugin_build.py should not own asset pack temporary-file orchestration",
            )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
        ):
            self.assertNotIn(
                forbidden_import,
                asset_pack_text,
                "asset pack owner must stay independent from build and validate entry owners",
            )

    def test_signature_sidecar_lives_in_signature_owner(self):
        self.assertTrue(
            PLUGIN_BUILD_SIGNATURE.exists(),
            "plugin build signature/hash sidecar logic belongs in plugin_build_signature.py",
        )
        build_text = PLUGIN_BUILD.read_text(encoding="utf-8")
        signature_text = PLUGIN_BUILD_SIGNATURE.read_text(encoding="utf-8")

        for function_name in (
            "plugin_build_signing_audit",
            "write_plugin_build_signature",
            "plugin_build_loadable_file_manifest",
            "plugin_build_signature_template",
            "plugin_build_signing_artifacts",
            "toml_bool",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                build_text,
                f"{function_name} belongs in plugin_build_signature.py",
            )
            self.assertIn(
                f"def {function_name}(",
                signature_text,
            )

        package_text = PLUGIN_BUILD_PACKAGE.read_text(encoding="utf-8")
        self.assertIn(
            "from .plugin_build_signature import",
            package_text,
            "plugin_build_package.py should consume signature sidecar behavior through the signature owner",
        )
        self.assertNotIn(
            "from .plugin_build_signature import",
            build_text,
            "plugin_build.py should consume signature sidecar behavior through the package owner",
        )
        self.assertNotIn(
            "import hashlib",
            build_text,
            "plugin_build.py should not own signature artifact hashing",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
        ):
            self.assertNotIn(
                forbidden_import,
                signature_text,
                "signature sidecar owner must stay independent from build and validate entry owners",
            )

    def test_cargo_command_lives_in_command_owner(self):
        self.assertTrue(
            PLUGIN_BUILD_COMMAND.exists(),
            "plugin build Cargo command semantics belong in plugin_build_command.py",
        )
        build_text = PLUGIN_BUILD.read_text(encoding="utf-8")
        command_text = PLUGIN_BUILD_COMMAND.read_text(encoding="utf-8")

        for function_name in (
            "default_target_dir",
            "plugin_build_features",
            "plugin_build_cargo_command",
            "run_plugin_build_command",
            "shell_join",
            "shell_quote",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                build_text,
                f"{function_name} belongs in plugin_build_command.py",
            )
            self.assertIn(
                f"def {function_name}(",
                command_text,
            )

        self.assertIn(
            "from .plugin_build_command import",
            build_text,
            "plugin_build.py should consume Cargo command behavior through the command owner",
        )
        self.assertNotIn(
            "import subprocess",
            build_text,
            "plugin_build.py should not own Cargo process execution",
        )
        self.assertNotIn(
            "import sys",
            build_text,
            "plugin_build.py should not own shell command quoting",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
        ):
            self.assertNotIn(
                forbidden_import,
                command_text,
                "Cargo command owner must stay independent from build and validate entry owners",
            )
        self.assertLessEqual(
            len(build_text.splitlines()),
            410,
            "plugin_build.py should remain a thin build orchestration owner",
        )
        self.assertLessEqual(
            len(command_text.splitlines()),
            130,
            "plugin_build_command.py should stay focused on command semantics",
        )

    def test_preflight_validation_lives_in_preflight_owner(self):
        self.assertTrue(
            PLUGIN_BUILD_PREFLIGHT.exists(),
            "plugin build distribution/signing preflight belongs in plugin_build_preflight.py",
        )
        build_text = PLUGIN_BUILD.read_text(encoding="utf-8")
        preflight_text = PLUGIN_BUILD_PREFLIGHT.read_text(encoding="utf-8")

        for function_name in (
            "plugin_distribution_dist_crate",
            "plugin_distribution_abi_version",
            "plugin_build_optional_trimmed_string",
            "plugin_build_string_array",
            "plugin_build_failure_report",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                build_text,
                f"{function_name} belongs in plugin_build_preflight.py",
            )
            self.assertIn(
                f"def {function_name}(",
                preflight_text,
            )

        self.assertIn(
            "from .plugin_build_preflight import",
            build_text,
            "plugin_build.py should consume distribution/signing preflight through the preflight owner",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_build_command import",
            "from .plugin_build_package import",
        ):
            self.assertNotIn(
                forbidden_import,
                preflight_text,
                "preflight owner must stay independent from build, validate, command, and package owners",
            )
        self.assertLessEqual(
            len(build_text.splitlines()),
            340,
            "plugin_build.py should keep only argument parsing and build orchestration",
        )
        self.assertLessEqual(
            len(preflight_text.splitlines()),
            120,
            "plugin_build_preflight.py should stay focused on preflight helpers",
        )


if __name__ == "__main__":
    unittest.main()
