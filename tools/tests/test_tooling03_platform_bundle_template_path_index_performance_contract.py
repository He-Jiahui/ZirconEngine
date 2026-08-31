from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.pipeline_report_platform_bundle_template_resolution_schema import (
    platform_bundle_template_resolution_schema_diagnostics,
)


def template_resolution(root: Path) -> dict[str, object]:
    template_dir = root / "template"
    return {
        "template_root": str(root),
        "profile": "windows-release",
        "expected_engine_version": "0.1.0",
        "expected_target_platform": "windows-x86_64",
        "fatal": False,
        "diagnostics": [],
        "candidates": [
            {
                "template_dir": str(template_dir),
                "template_id": "windows-template",
                "engine_version": "0.1.0",
                "target_platform": "windows-x86_64",
                "compatible_profiles": ["windows-release"],
                "host_artifact": "precompiled",
                "bundle_format": "directory",
            }
        ],
        "skipped_candidates": [
            {
                "template_dir": str(root / "broken-template"),
                "diagnostics": ["template format_version 999 is not supported"],
            }
        ],
        "template_dir": str(template_dir),
    }


class Tooling03PlatformBundleTemplatePathIndexPerformanceContractTests(
    unittest.TestCase
):
    def test_schema_resolves_each_template_directory_once(self) -> None:
        original_resolve = Path.resolve
        resolve_calls: list[str] = []

        def counted_resolve(path: Path, *args: object, **kwargs: object) -> Path:
            resolve_calls.append(str(path))
            return original_resolve(path, *args, **kwargs)

        with tempfile.TemporaryDirectory() as temp_dir, mock.patch.object(
            Path,
            "resolve",
            counted_resolve,
        ):
            diagnostics = platform_bundle_template_resolution_schema_diagnostics(
                template_resolution(Path(temp_dir)),
                "resolution",
            )

        self.assertEqual(diagnostics, [])
        self.assertEqual(
            len(resolve_calls),
            3,
            "schema path semantics must resolve one root and each entry once",
        )


if __name__ == "__main__":
    unittest.main()
