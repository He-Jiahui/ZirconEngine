from __future__ import annotations

import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.native_dynamic_payload_file_manifest import (
    native_dynamic_package_payload_file_manifest,
)


class Tooling03NativeDynamicPayloadStreamingHashPerformanceContractTests(
    unittest.TestCase
):
    def test_payload_manifest_does_not_materialize_files_as_python_bytes(self) -> None:
        payload = b"zircon-native-payload" * 4096
        with tempfile.TemporaryDirectory() as temp_dir:
            package_dir = Path(temp_dir) / "animation"
            native_dir = package_dir / "native"
            native_dir.mkdir(parents=True)
            artifact = native_dir / "zircon_plugin_animation.dll"
            artifact.write_bytes(payload)

            with mock.patch.object(
                Path,
                "read_bytes",
                side_effect=AssertionError(
                    "payload manifests must not allocate whole-file Python bytes"
                ),
            ):
                manifest = native_dynamic_package_payload_file_manifest(package_dir)

        self.assertEqual(
            manifest,
            [
                {
                    "path": "native/zircon_plugin_animation.dll",
                    "bytes": len(payload),
                    "sha256": hashlib.sha256(payload).hexdigest(),
                }
            ],
        )

    def test_payload_manifest_preserves_empty_file_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            package_dir = Path(temp_dir) / "animation"
            package_dir.mkdir()
            (package_dir / "empty.bin").touch()

            manifest = native_dynamic_package_payload_file_manifest(package_dir)

        self.assertEqual(
            manifest,
            [
                {
                    "path": "empty.bin",
                    "bytes": 0,
                    "sha256": hashlib.sha256().hexdigest(),
                }
            ],
        )


if __name__ == "__main__":
    unittest.main()
