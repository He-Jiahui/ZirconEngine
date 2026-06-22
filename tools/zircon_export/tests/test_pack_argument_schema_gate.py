from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.cli import run_pack
from tools.zircon_export.tests.export_test_support import _pack_args


class PackArgumentSchemaGateTests(unittest.TestCase):
    def test_pack_rejects_blank_explicit_path_arguments(self) -> None:
        cases = (
            (
                "asset_manifest",
                {"asset_manifest": "   "},
                "asset_manifest argument must be a non-empty string",
                "asset_manifest=<invalid>",
            ),
            (
                "pack_file",
                {"pack_file": "   "},
                "pack_file argument must be a non-empty string",
                "pack=<invalid>",
            ),
            (
                "previous_pack",
                {
                    "previous_pack": "   ",
                    "delta_pack": "delta.zrpd",
                },
                "previous_pack argument must be a non-empty string",
                "command=<skipped>",
            ),
            (
                "delta_pack",
                {
                    "previous_pack": "previous.zrpack",
                    "delta_pack": "   ",
                },
                "delta_pack argument must be a non-empty string",
                "command=<skipped>",
            ),
            (
                "packer",
                {"packer": "   "},
                "packer argument must be a non-empty string",
                "command=<skipped>",
            ),
            (
                "target_dir",
                {"target_dir": "   "},
                "target_dir argument must be a non-empty string",
                "command=<skipped>",
            ),
        )

        for name, overrides, expected_diagnostic, expected_output in cases:
            with self.subTest(name=name):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    args = _pack_args(out=root / "out", dry_run=True)
                    for field, value in overrides.items():
                        setattr(args, field, value)

                    stdout = io.StringIO()
                    with contextlib.redirect_stdout(stdout):
                        exit_code = run_pack(args)

                    output = stdout.getvalue()
                    self.assertEqual(exit_code, 2)
                    self.assertIn(f"diagnostic={expected_diagnostic}", output)
                    self.assertIn(expected_output, output)
                    self.assertIn("command=<skipped>", output)

    def test_pack_rejects_padded_explicit_path_arguments(self) -> None:
        cases = (
            (
                "asset_manifest",
                {"asset_manifest": " asset_manifest.json "},
                "asset_manifest argument must be a non-empty trimmed string",
                "asset_manifest=<invalid>",
            ),
            (
                "pack_file",
                {"pack_file": " assets.zrpack "},
                "pack_file argument must be a non-empty trimmed string",
                "pack=<invalid>",
            ),
            (
                "previous_pack",
                {
                    "previous_pack": " previous.zrpack ",
                    "delta_pack": "delta.zrpd",
                },
                "previous_pack argument must be a non-empty trimmed string",
                "command=<skipped>",
            ),
            (
                "delta_pack",
                {
                    "previous_pack": "previous.zrpack",
                    "delta_pack": " delta.zrpd ",
                },
                "delta_pack argument must be a non-empty trimmed string",
                "command=<skipped>",
            ),
            (
                "packer",
                {"packer": " packer.exe "},
                "packer argument must be a non-empty trimmed string",
                "command=<skipped>",
            ),
            (
                "target_dir",
                {"target_dir": " target "},
                "target_dir argument must be a non-empty trimmed string",
                "command=<skipped>",
            ),
        )

        for name, overrides, expected_diagnostic, expected_output in cases:
            with self.subTest(name=name):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    args = _pack_args(out=root / "out", dry_run=True)
                    for field, value in overrides.items():
                        setattr(args, field, value)

                    stdout = io.StringIO()
                    with contextlib.redirect_stdout(stdout):
                        exit_code = run_pack(args)

                    output = stdout.getvalue()
                    self.assertEqual(exit_code, 2)
                    self.assertIn(f"diagnostic={expected_diagnostic}", output)
                    self.assertIn(expected_output, output)
                    self.assertIn("command=<skipped>", output)


if __name__ == "__main__":
    unittest.main()
