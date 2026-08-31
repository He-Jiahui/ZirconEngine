from __future__ import annotations

import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.pipeline_report_platform_bundle_template import (
    platform_bundle_file_sha256,
)


class Tooling03PlatformBundleTemplateStreamingHashPerformanceContractTests(
    unittest.TestCase
):
    def test_template_hash_does_not_materialize_the_file_as_python_bytes(self) -> None:
        payload = b"zircon-platform-template" * 4096
        with tempfile.TemporaryDirectory() as temp_dir:
            template_file = Path(temp_dir) / "zircon_runtime.exe"
            template_file.write_bytes(payload)
            diagnostics: list[str] = []

            with mock.patch.object(
                Path,
                "read_bytes",
                side_effect=AssertionError(
                    "template hashes must not allocate whole-file Python bytes"
                ),
            ):
                digest = platform_bundle_file_sha256(
                    template_file,
                    diagnostics,
                    "template file",
                )

        self.assertEqual(diagnostics, [])
        self.assertEqual(digest, hashlib.sha256(payload).hexdigest())

    def test_template_hash_preserves_empty_file_digest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_file = Path(temp_dir) / "empty.bin"
            template_file.touch()
            diagnostics: list[str] = []

            digest = platform_bundle_file_sha256(
                template_file,
                diagnostics,
                "template file",
            )

        self.assertEqual(diagnostics, [])
        self.assertEqual(digest, hashlib.sha256().hexdigest())


if __name__ == "__main__":
    unittest.main()
