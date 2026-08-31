from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

import tools.zircon_export.platform_bundle_template_files_materialize as materialize


class Tooling03PlatformBundleTemplateRootProjectionCachePerformanceContractTests(
    unittest.TestCase
):
    def test_template_root_is_projected_once_for_many_files(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            template_dir = root / "template"
            template_dir.mkdir()
            files = []
            for index in range(64):
                name = f"file-{index}.json"
                template_dir.joinpath(name).write_text("{}", encoding="utf-8")
                files.append({"path": name})
            real_path = Path
            root_projections = 0

            def observed_path(value: object) -> Path:
                nonlocal root_projections
                if value == str(template_dir):
                    root_projections += 1
                return real_path(value)

            with mock.patch.object(materialize, "Path", new=observed_path), mock.patch.object(
                materialize,
                "copy_platform_bundle_template_file",
                return_value=True,
            ):
                fatal, copied = materialize.materialize_platform_bundle_template_files(
                    bundle_root=root / "bundle",
                    template_report={
                        "template_dir": str(template_dir),
                        "files": files,
                    },
                    host_executable=None,
                    host_destination=None,
                    diagnostics=[],
                )

        self.assertFalse(fatal)
        self.assertEqual(len(copied), 64)
        self.assertEqual(root_projections, 1)


if __name__ == "__main__":
    unittest.main()
