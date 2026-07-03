import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ZIRCON_EXPORT_CLI = REPO_ROOT / "tools/zircon_export/cli.py"
CLI_ARGUMENTS = REPO_ROOT / "tools/zircon_export/cli_arguments.py"
VALIDATE_STAGE = REPO_ROOT / "tools/zircon_export/validate_stage.py"
PACK_STAGE = REPO_ROOT / "tools/zircon_export/pack_stage.py"
PACK_STAGE_PATHS = REPO_ROOT / "tools/zircon_export/pack_stage_paths.py"


class ZirconExportCliOwnerBoundaryTests(unittest.TestCase):
    def test_cli_argument_surface_lives_in_cli_arguments_owner(self):
        self.assertTrue(
            CLI_ARGUMENTS.exists(),
            "CLI argument declarations belong in cli_arguments.py",
        )
        cli_text = ZIRCON_EXPORT_CLI.read_text(encoding="utf-8")
        argument_text = CLI_ARGUMENTS.read_text(encoding="utf-8")

        for function_name in (
            "parse_args",
            "option_present",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                cli_text,
                f"{function_name} belongs in cli_arguments.py",
            )
            self.assertIn(
                f"def {function_name}(",
                argument_text,
            )

        for constant_name in (
            "STAGES",
            "RESUMABLE_STAGES",
            "DEFAULT_OUT",
        ):
            self.assertNotIn(
                f"{constant_name} =",
                cli_text,
                f"{constant_name} belongs in cli_arguments.py",
            )
            self.assertIn(
                f"{constant_name} =",
                argument_text,
            )

        self.assertIn(
            "from .cli_arguments import parse_args",
            cli_text,
            "cli.py should consume parsing through the CLI argument owner",
        )
        self.assertNotIn(
            "from .cli import",
            argument_text,
            "cli_arguments.py must stay independent from the root CLI owner",
        )

    def test_validate_stage_leaf_helpers_live_in_validate_stage_owner(self):
        self.assertTrue(
            VALIDATE_STAGE.exists(),
            "Validate command/report/path leaf helpers belong in validate_stage.py",
        )
        cli_text = ZIRCON_EXPORT_CLI.read_text(encoding="utf-8")
        validate_text = VALIDATE_STAGE.read_text(encoding="utf-8")

        for function_name in (
            "validate_command",
            "validate_preflight_failure_report",
            "resolve_validate_path",
            "resolve_validate_optional_path",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                cli_text,
                f"{function_name} belongs in validate_stage.py",
            )
            self.assertIn(
                f"def {function_name}(",
                validate_text,
            )

        self.assertIn(
            "from .validate_stage import",
            cli_text,
            "cli.py should consume Validate helpers through the Validate stage owner",
        )
        self.assertNotIn(
            "from .cli import",
            validate_text,
            "validate_stage.py must stay independent from the root CLI owner",
        )

    def test_pack_stage_leaf_helpers_live_in_pack_stage_owner(self):
        self.assertTrue(
            PACK_STAGE.exists(),
            "Pack orchestration belongs in pack_stage.py",
        )
        self.assertTrue(
            PACK_STAGE_PATHS.exists(),
            "Pack path and argument helpers belong in pack_stage_paths.py",
        )
        cli_text = ZIRCON_EXPORT_CLI.read_text(encoding="utf-8")
        pack_text = PACK_STAGE.read_text(encoding="utf-8")
        pack_paths_text = PACK_STAGE_PATHS.read_text(encoding="utf-8")

        for function_name in (
            "run_pack",
            "pack_preflight_failure_report",
            "pack_command",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                cli_text,
                f"{function_name} belongs in pack_stage.py",
            )
            self.assertIn(
                f"def {function_name}(",
                pack_text,
            )

        for function_name in (
            "pack_asset_manifest_argument_diagnostic",
            "pack_file_argument_diagnostic",
            "pack_optional_path_argument_diagnostic",
            "pack_asset_manifest_path",
            "pack_output_path",
            "resolve_pack_optional_path",
            "resolve_pack_stage_path",
            "pack_delta_argument_diagnostics",
            "pack_asset_manifest_diagnostic",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                cli_text,
                f"{function_name} belongs in pack_stage_paths.py",
            )
            self.assertNotIn(
                f"def {function_name}(",
                pack_text,
                f"{function_name} should not live in pack_stage.py",
            )
            self.assertIn(
                f"def {function_name}(",
                pack_paths_text,
            )

        self.assertIn(
            "from .pack_stage import run_pack",
            cli_text,
            "cli.py should consume Pack through the Pack stage owner",
        )
        self.assertNotIn(
            "from .pack_stage_paths import",
            cli_text,
            "cli.py should not bypass Pack stage orchestration for path helpers",
        )
        self.assertNotIn(
            "from .cli import",
            pack_text,
            "pack_stage.py must stay independent from the root CLI owner",
        )

    def test_root_cli_stays_below_large_file_budget_after_stage_splits(self):
        cli_lines = ZIRCON_EXPORT_CLI.read_text(encoding="utf-8").splitlines()
        self.assertLess(
            len(cli_lines),
            420,
            "cli.py must remain a root command/pipeline orchestration owner, not a stage helper sink",
        )


if __name__ == "__main__":
    unittest.main()
