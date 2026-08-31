from __future__ import annotations

import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.export_template_manifest import template_file_manifest


def manifest_for(path: str, payload: bytes) -> dict[str, object]:
    return {
        "files": [
            {
                "path": path,
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
        ]
    }


class Tooling03ExportTemplateManifestMmapPerformanceContractTests(
    unittest.TestCase
):
    def test_manifest_hashing_does_not_materialize_the_whole_file(self) -> None:
        payload = bytes(range(256)) * 32
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "host.bin").write_bytes(payload)
            diagnostics: list[str] = []
            with mock.patch.object(
                Path,
                "read_bytes",
                side_effect=AssertionError("manifest hashing copied the whole file"),
            ):
                files = template_file_manifest(
                    root,
                    manifest_for("host.bin", payload),
                    diagnostics,
                )

        self.assertEqual(diagnostics, [])
        self.assertEqual(len(files), 1)
        self.assertEqual(files[0]["sha256"], hashlib.sha256(payload).hexdigest())

    def test_manifest_hashing_preserves_empty_file_digest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "empty.bin").write_bytes(b"")
            diagnostics: list[str] = []
            files = template_file_manifest(
                root,
                manifest_for("empty.bin", b""),
                diagnostics,
            )

        self.assertEqual(diagnostics, [])
        self.assertEqual(len(files), 1)

    def test_manifest_hashing_reports_file_open_errors(self) -> None:
        payload = b"template-host"
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "host.bin").write_bytes(payload)
            diagnostics: list[str] = []
            with mock.patch.object(
                Path,
                "open",
                side_effect=OSError("simulated manifest read failure"),
            ):
                files = template_file_manifest(
                    root,
                    manifest_for("host.bin", payload),
                    diagnostics,
                )

        self.assertEqual(files, [])
        self.assertEqual(len(diagnostics), 1)
        self.assertIn("could not be read", diagnostics[0])
        self.assertIn("simulated manifest read failure", diagnostics[0])


if __name__ == "__main__":
    unittest.main()
