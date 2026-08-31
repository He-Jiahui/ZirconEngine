from __future__ import annotations

import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.pipeline_report_platform_bundle_template_report_semantics import (
    template_report_file_source_hash_diagnostics,
)


def template_file(path: str, payload: bytes) -> dict[str, str]:
    return {
        "path": path,
        "bundle_path": path,
        "sha256": hashlib.sha256(payload).hexdigest(),
    }


class Tooling03PlatformBundleTemplateFileMmapPerformanceContractTests(
    unittest.TestCase
):
    def test_source_hashing_does_not_materialize_the_whole_file_as_bytes(self) -> None:
        payload = bytes(range(256)) * 32
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "host.bin").write_bytes(payload)
            with mock.patch.object(
                Path,
                "read_bytes",
                side_effect=AssertionError("template hashing copied the whole file"),
            ):
                diagnostics = template_report_file_source_hash_diagnostics(
                    "template",
                    {"template_dir": str(root)},
                    [template_file("host.bin", payload)],
                )

        self.assertEqual(diagnostics, [])

    def test_source_hashing_preserves_empty_file_digest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "empty.bin").write_bytes(b"")
            diagnostics = template_report_file_source_hash_diagnostics(
                "template",
                {"template_dir": str(root)},
                [template_file("empty.bin", b"")],
            )

        self.assertEqual(diagnostics, [])

    def test_source_hashing_reports_file_open_errors(self) -> None:
        payload = b"template-host"
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "host.bin").write_bytes(payload)
            with mock.patch.object(
                Path,
                "open",
                side_effect=OSError("simulated template read failure"),
            ):
                diagnostics = template_report_file_source_hash_diagnostics(
                    "template",
                    {"template_dir": str(root)},
                    [template_file("host.bin", payload)],
                )

        self.assertEqual(len(diagnostics), 1)
        self.assertIn("could not be read", diagnostics[0])
        self.assertIn("simulated template read failure", diagnostics[0])


if __name__ == "__main__":
    unittest.main()
