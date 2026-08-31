from __future__ import annotations

import binascii
import struct
import tempfile
import tracemalloc
import unittest
import zlib
from pathlib import Path
from typing import BinaryIO

from tools.validate_render_measurement_evidence import _validate_png


SCRIPT = Path(__file__).resolve().parents[1] / "validate_render_measurement_evidence.py"


class RenderMeasurementArtifactStreamPerformanceContractTests(unittest.TestCase):
    def test_png_crc_validation_is_bounded_streaming_io(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        function = source[source.index("def _validate_png(") : source.index("def main(")]

        self.assertIn('with path.open("rb") as png_file:', function)
        self.assertIn("_PNG_STREAM_CHUNK_SIZE", function)
        self.assertNotIn("path.read_bytes()", function)
        self.assertNotIn("chunk_type + payload", function)

    def test_artifact_metadata_uses_one_stat_per_path(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        function = source[source.index("def _validate_artifact_files(") : source.index("def _validate_png(")]

        self.assertIn("artifact_stat = resolved.stat()", function)
        self.assertIn("stat.S_ISREG(artifact_stat.st_mode)", function)
        self.assertNotIn("resolved.is_file()", function)

    def test_large_png_validation_has_bounded_peak_memory(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            png_path = Path(temporary_directory) / "large-ancillary.png"
            _write_large_ancillary_png(png_path, payload_size=8 * 1024 * 1024)

            tracemalloc.start()
            try:
                _validate_png(png_path, png_path)
                _current_bytes, peak_bytes = tracemalloc.get_traced_memory()
            finally:
                tracemalloc.stop()

        self.assertLess(peak_bytes, 1024 * 1024)


def _write_large_ancillary_png(path: Path, *, payload_size: int) -> None:
    block = b"a" * (64 * 1024)
    ihdr = struct.pack(">IIBBBBB", 1, 1, 8, 6, 0, 0, 0)
    idat = zlib.compress(b"\x00\x00\x00\x00\x00")
    with path.open("wb") as png_file:
        png_file.write(b"\x89PNG\r\n\x1a\n")
        _write_chunk(png_file, b"IHDR", ihdr)
        png_file.write(struct.pack(">I", payload_size))
        png_file.write(b"tEXt")
        crc = binascii.crc32(b"tEXt")
        remaining = payload_size
        while remaining:
            payload = block[: min(remaining, len(block))]
            png_file.write(payload)
            crc = binascii.crc32(payload, crc)
            remaining -= len(payload)
        png_file.write(struct.pack(">I", crc & 0xFFFFFFFF))
        _write_chunk(png_file, b"IDAT", idat)
        _write_chunk(png_file, b"IEND", b"")


def _write_chunk(png_file: BinaryIO, chunk_type: bytes, payload: bytes) -> None:
    png_file.write(struct.pack(">I", len(payload)))
    png_file.write(chunk_type)
    png_file.write(payload)
    png_file.write(
        struct.pack(">I", binascii.crc32(chunk_type + payload) & 0xFFFFFFFF)
    )


if __name__ == "__main__":
    unittest.main()
