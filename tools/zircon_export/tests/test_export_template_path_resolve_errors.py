from __future__ import annotations

import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.export_template import validate_export_template
from tools.zircon_export.export_template_resolution import resolve_export_template_from_root
from tools.zircon_export.tests.export_test_support import VALID_TEMPLATE


class ExportTemplatePathResolveErrorsTests(unittest.TestCase):
    def test_validate_template_rejects_template_directory_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if Path(path) == template_dir:
                    raise OSError("simulated template directory resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                report = validate_export_template(
                    template_dir=template_dir,
                    expected_engine_version="0.1.0",
                    profile="windows-release",
                    expected_target_platform="windows-x86_64",
                )

        self.assertTrue(report["fatal"], report["diagnostics"])
        self.assertTrue(
            any(
                "export template directory" in diagnostic
                and "could not be resolved" in diagnostic
                and "simulated template directory resolve failure" in diagnostic
                for diagnostic in report["diagnostics"]
            ),
            report["diagnostics"],
        )

    def test_template_resolution_rejects_candidate_directory_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_root = Path(temp_dir) / "templates"
            template_dir = template_root / "windows"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if Path(path) == template_dir:
                    raise OSError("simulated candidate directory resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                report = resolve_export_template_from_root(
                    template_root=template_root,
                    profile="windows-release",
                    expected_engine_version="0.1.0",
                    expected_target_platform="windows-x86_64",
                )

        self.assertTrue(report["fatal"], report)
        self.assertIsNone(report["template_dir"])
        self.assertTrue(report["skipped_candidates"], report)
        skipped_diagnostics = report["skipped_candidates"][0]["diagnostics"]
        self.assertTrue(
            any(
                "export template directory" in diagnostic
                and "could not be resolved" in diagnostic
                and "simulated candidate directory resolve failure" in diagnostic
                for diagnostic in skipped_diagnostics
            ),
            skipped_diagnostics,
        )


if __name__ == "__main__":
    unittest.main()
