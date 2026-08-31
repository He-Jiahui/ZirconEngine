from __future__ import annotations

import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export import native_signing


class Tooling03NativeSigningStreamingHashPerformanceContractTests(unittest.TestCase):
    def test_file_hash_does_not_materialize_the_artifact_as_python_bytes(self) -> None:
        payload = b"zircon-native-signing" * 4096
        with tempfile.TemporaryDirectory() as temp_dir:
            artifact = Path(temp_dir) / "zircon_plugin_animation.dll"
            artifact.write_bytes(payload)

            with mock.patch.object(
                Path,
                "read_bytes",
                side_effect=AssertionError(
                    "native signing hashes must not allocate whole-file Python bytes"
                ),
            ):
                digest = native_signing.file_sha256(artifact)

        self.assertEqual(digest, hashlib.sha256(payload).hexdigest())

    def test_file_hash_preserves_empty_artifact_digest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            artifact = Path(temp_dir) / "empty.dll"
            artifact.touch()

            digest = native_signing.file_sha256(artifact)

        self.assertEqual(digest, hashlib.sha256().hexdigest())


if __name__ == "__main__":
    unittest.main()
