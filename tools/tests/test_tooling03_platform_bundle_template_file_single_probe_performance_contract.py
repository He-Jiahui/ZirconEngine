from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.platform_bundle_template_files_materialize import (
    materialize_platform_bundle_template_files,
)


class Tooling03PlatformBundleTemplateFileSingleProbePerformanceContractTests(
    unittest.TestCase
):
    def test_valid_template_file_uses_one_metadata_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            template_dir = root / "template"
            source = template_dir / "config.json"
            source.parent.mkdir(parents=True)
            source.write_text("{}", encoding="utf-8")
            observed_source = mock.Mock(wraps=source)
            observed_source.__str__ = mock.Mock(return_value=str(source))
            observed_source.__truediv__ = mock.Mock(return_value=observed_source)
            with mock.patch(
                "tools.zircon_export.platform_bundle_template_files_materialize.Path",
                return_value=observed_source,
            ), mock.patch(
                "tools.zircon_export.platform_bundle_template_files_materialize.copy_platform_bundle_template_file",
                return_value=True,
            ):
                fatal, copied = materialize_platform_bundle_template_files(
                    bundle_root=root / "bundle",
                    template_report={
                        "template_dir": str(template_dir),
                        "files": [{"path": "config.json"}],
                    },
                    host_executable=None,
                    host_destination=None,
                    diagnostics=[],
                )

        self.assertFalse(fatal)
        self.assertEqual(len(copied), 1)
        probes = (
            observed_source.exists.call_count
            + observed_source.is_file.call_count
            + observed_source.stat.call_count
        )
        self.assertEqual(probes, 1)

    def test_template_file_inspection_error_is_not_reported_as_missing(self) -> None:
        source = Path("template") / "config.json"
        diagnostics: list[str] = []
        observed_source = mock.Mock(wraps=source)
        observed_source.__str__ = mock.Mock(return_value=str(source))
        observed_source.__truediv__ = mock.Mock(return_value=observed_source)
        observed_source.stat.side_effect = PermissionError(
            "simulated permission failure"
        )
        with mock.patch(
            "tools.zircon_export.platform_bundle_template_files_materialize.Path",
            return_value=observed_source,
        ):
            fatal, copied = materialize_platform_bundle_template_files(
                bundle_root=Path("bundle"),
                template_report={
                    "template_dir": "template",
                    "files": [{"path": "config.json"}],
                },
                host_executable=None,
                host_destination=None,
                diagnostics=diagnostics,
            )

        self.assertTrue(fatal)
        self.assertEqual(copied, [])
        self.assertEqual(
            diagnostics,
            [
                f"template file {source} could not be inspected during bundle copy: "
                "simulated permission failure"
            ],
        )


if __name__ == "__main__":
    unittest.main()
