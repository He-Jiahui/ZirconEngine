"""Bounded-memory file digest helpers for build and export artifacts."""

from __future__ import annotations

import hashlib
from pathlib import Path


FILE_DIGEST_BUFFER_BYTES = 256 * 1024


def file_sha256(path: Path) -> str:
    return file_size_and_sha256(path)[1]


def file_size_and_sha256(path: Path) -> tuple[int, str]:
    digest = hashlib.sha256()
    byte_length = 0
    buffer = bytearray(FILE_DIGEST_BUFFER_BYTES)
    buffer_view = memoryview(buffer)
    with path.open("rb", buffering=0) as source:
        while read_count := source.readinto(buffer):
            byte_length += read_count
            digest.update(buffer_view[:read_count])
    return byte_length, digest.hexdigest()
