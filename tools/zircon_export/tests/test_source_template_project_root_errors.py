from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import (
    _run_source_template_quiet,
    _source_template_args,
    _source_template_validate_report,
    json_dumps,
    json_loads,
)


class SourceTemplateProjectRootErrorTests(unittest.TestCase):
    def test_source_template_stage_rejects_project_root_create_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(_source_template_validate_report()),
                encoding="utf-8",
            )
            out = root / "out"
            project_dir = (out / "stages" / "source_template" / "project").resolve()
            original_mkdir = Path.mkdir

            def mkdir_or_fail(path: Path, *args: object, **kwargs: object) -> None:
                if path.resolve() == project_dir:
                    raise OSError("simulated project root create failure")
                original_mkdir(path, *args, **kwargs)

            with mock.patch.object(Path, "mkdir", mkdir_or_fail):
                exit_code = _run_source_template_quiet(
                    _source_template_args(
                        out=out,
                        validate_report=validate_report,
                        dry_run=False,
                    )
                )

            report = json_loads(
                (out / "stages" / "source_template" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["generated_files"], [])
            self.assertTrue(
                any(
                    "SourceTemplate generated project" in diagnostic
                    and "could not be created" in diagnostic
                    and "simulated project root create failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
