from __future__ import annotations

import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.pipeline_report_platform_bundle_file_evidence import (
    platform_bundle_file_sha256,
)


class Tooling03PlatformBundleFileEvidenceMmapPerformanceContractTests(
    unittest.TestCase
):
    def test_file_evidence_hashing_does_not_materialize_the_whole_file(self) -> None:
        payload = bytes(range(256)) * 32
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "artifact.bin"
            path.write_bytes(payload)
            diagnostics: list[str] = []
            with mock.patch.object(
                Path,
                "read_bytes",
                side_effect=AssertionError("file evidence copied the whole file"),
            ):
                digest = platform_bundle_file_sha256(
                    path,
                    diagnostics,
                    "artifact",
                )

        self.assertEqual(diagnostics, [])
        self.assertEqual(digest, hashlib.sha256(payload).hexdigest())

    def test_file_evidence_hashing_preserves_empty_file_digest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "empty.bin"
            path.write_bytes(b"")
            diagnostics: list[str] = []
            digest = platform_bundle_file_sha256(path, diagnostics, "artifact")

        self.assertEqual(diagnostics, [])
        self.assertEqual(digest, hashlib.sha256().hexdigest())

    def test_file_evidence_hashing_reports_file_open_errors(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "artifact.bin"
            path.write_bytes(b"artifact")
            diagnostics: list[str] = []
            with mock.patch.object(
                Path,
                "open",
                side_effect=OSError("simulated evidence read failure"),
            ):
                digest = platform_bundle_file_sha256(
                    path,
                    diagnostics,
                    "artifact",
                )

        self.assertIsNone(digest)
        self.assertEqual(len(diagnostics), 1)
        self.assertIn("could not be read", diagnostics[0])
        self.assertIn("simulated evidence read failure", diagnostics[0])


if __name__ == "__main__":
    unittest.main()
