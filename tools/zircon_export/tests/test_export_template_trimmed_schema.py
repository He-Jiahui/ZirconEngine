from __future__ import annotations

import shutil
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.export_template import validate_export_template
from tools.zircon_export.tests.export_test_support import VALID_TEMPLATE


class ExportTemplateTrimmedSchemaTests(unittest.TestCase):
    def test_template_rejects_padded_file_sha256(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            manifest = template_dir / "template.toml"
            manifest.write_text(
                manifest.read_text(encoding="utf-8").replace(
                    (
                        'sha256 = "63a26218c731a8b79da125da1e59a6a4e67ac'
                        '2212ce6a2ee3f3016dde237dd97"'
                    ),
                    (
                        'sha256 = " 63a26218c731a8b79da125da1e59a6a4e67ac'
                        '2212ce6a2ee3f3016dde237dd97 "'
                    ),
                    1,
                ),
                encoding="utf-8",
            )

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

        self.assertTrue(report["fatal"], report["diagnostics"])
        self.assertTrue(
            any(
                "template file bin/zircon_runtime.host-placeholder sha256 "
                "must be a non-empty trimmed string"
                in diagnostic
                for diagnostic in report["diagnostics"]
            ),
            report["diagnostics"],
        )


if __name__ == "__main__":
    unittest.main()
