from __future__ import annotations

import gc
import hashlib
import tempfile
import tracemalloc
import unittest
from importlib import import_module
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SHADER_DIGEST_SOURCE = (
    REPO_ROOT / "tools/zircon_build_plugin_shader_descriptor_support.py"
)
COOK_REPORT_SOURCE = REPO_ROOT / "tools/zircon_export/pipeline_report_cook_assets.py"
PLUGIN_SIGNATURE_SOURCE = REPO_ROOT / "tools/zircon_export/plugin_build_signature.py"


class StreamingArtifactDigestPerformanceContractTests(unittest.TestCase):
    def test_streaming_digest_matches_sha256_and_reports_byte_length(self) -> None:
        digest_module = import_module("tools.zircon_export.file_digest")
        payload = (b"zircon-streaming-digest\0" * 8193) + b"tail"
        with tempfile.TemporaryDirectory() as temp_dir:
            source = Path(temp_dir) / "artifact.bin"
            source.write_bytes(payload)

            byte_length, digest = digest_module.file_size_and_sha256(source)
            digest_only = digest_module.file_sha256(source)

        self.assertEqual(byte_length, len(payload))
        self.assertEqual(digest, hashlib.sha256(payload).hexdigest())
        self.assertEqual(digest_only, digest)

    def test_streaming_digest_peak_memory_is_independent_of_file_size(self) -> None:
        digest_module = import_module("tools.zircon_export.file_digest")
        file_size = 8 * 1024 * 1024
        with tempfile.TemporaryDirectory() as temp_dir:
            source = Path(temp_dir) / "large-artifact.bin"
            with source.open("wb") as handle:
                handle.truncate(file_size)

            gc.collect()
            tracemalloc.start()
            legacy_digest = hashlib.sha256(source.read_bytes()).hexdigest()
            _, legacy_peak = tracemalloc.get_traced_memory()
            tracemalloc.stop()

            gc.collect()
            tracemalloc.start()
            byte_length, streaming_digest = digest_module.file_size_and_sha256(source)
            _, streaming_peak = tracemalloc.get_traced_memory()
            tracemalloc.stop()

        self.assertEqual(byte_length, file_size)
        self.assertEqual(streaming_digest, legacy_digest)
        self.assertLess(streaming_peak, 1024 * 1024)
        self.assertLess(streaming_peak * 8, legacy_peak)

    def test_shader_module_hash_uses_streaming_digest(self) -> None:
        source = SHADER_DIGEST_SOURCE.read_text(encoding="utf-8")

        self.assertIn("file_sha256(source_path)", source)
        self.assertNotIn("source_path.read_bytes()", source)

    def test_cook_manifest_hash_uses_streaming_digest(self) -> None:
        source = COOK_REPORT_SOURCE.read_text(encoding="utf-8")

        self.assertIn("file_sha256(manifest_path)", source)
        self.assertNotIn("manifest_path.read_bytes()", source)

    def test_plugin_loadable_manifest_uses_single_streaming_pass(self) -> None:
        source = PLUGIN_SIGNATURE_SOURCE.read_text(encoding="utf-8")

        self.assertIn("file_size_and_sha256(file_path)", source)
        self.assertNotIn("file_path.read_bytes()", source)


if __name__ == "__main__":
    unittest.main()
