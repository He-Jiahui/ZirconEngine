from __future__ import annotations

import hashlib
import os
from pathlib import Path, PurePosixPath
from typing import Mapping


ARCHIVE_ROOT_ENV = "ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT"

RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE = (
    "docs/plans/_archive/zircon_runtime/runtime/15/"
    "2026-07-09-runtime-index-output-records.md"
)

RUNTIME_09_UI_OUTPUT_ARCHIVE = (
    "docs/plans/zircon_runtime/runtime/09/"
    "2026-07-09-ui-subsystem-architecture-output-records.md"
)
RUNTIME_10_DYNAMIC_API_OUTPUT_ARCHIVE = (
    "docs/plans/zircon_runtime/runtime/10/"
    "2026-07-09-dynamic-api-and-interface-convergence-output-records.md"
)

# Fixed historical identities from archive-current-index-02.jsonl. These are
# readback evidence, never current-source Cargo or product acceptance.
HISTORICAL_IDENTITIES = {
    RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE: (
        1642499,
        "c535d29fb4810580d7d361bc50d7ed29b5fa2dfcb8151a7fd5e92644e3b5997c",
    ),
    RUNTIME_09_UI_OUTPUT_ARCHIVE: (
        33828,
        "7e2b375c3289647586d26288cf2f05a972322fe8631182561914d2ce0b408bc0",
    ),
    RUNTIME_10_DYNAMIC_API_OUTPUT_ARCHIVE: (
        49040,
        "2dfeeb83daf87bfab5da5b4f00db7cdffb192cadedf85846ee07a6d6cb41360f",
    ),
}


class HistoricalArchiveReader:
    """Read only explicitly mounted, identity-checked historical documents."""

    def __init__(
        self,
        archive_root: Path | None = None,
        *,
        identities: Mapping[str, tuple[int, str]] | None = None,
    ) -> None:
        configured = os.environ.get(ARCHIVE_ROOT_ENV) if archive_root is None else archive_root
        self.archive_root = Path(configured) if configured else None
        self.identities = dict(HISTORICAL_IDENTITIES if identities is None else identities)
        self.records: dict[str, dict[str, object]] = {}
        self._texts: dict[str, str | None] = {}

    def read(self, relative_path: str) -> str | None:
        if relative_path in self._texts:
            return self._texts[relative_path]
        expected_length, expected_sha256 = self.identities[relative_path]
        record: dict[str, object] = {
            "path": relative_path,
            "historical_only": True,
            "expected_length": expected_length,
            "expected_sha256": expected_sha256,
            "status": "unavailable",
        }
        self.records[relative_path] = record
        self._texts[relative_path] = None
        part = PurePosixPath(relative_path)
        if (
            part.is_absolute()
            or not part.parts
            or any(p in (".", "..") or ":" in p or "\\" in p for p in part.parts)
        ):
            record["reason"] = "invalid-relative-path"
            return None
        if self.archive_root is None:
            record["reason"] = "archive-root-not-configured"
            return None
        if not self.archive_root.is_absolute():
            record["reason"] = "archive-root-not-absolute"
            return None
        try:
            root = self.archive_root.resolve(strict=True)
            path = root.joinpath(*part.parts).resolve(strict=True)
            if not path.is_relative_to(root):
                record["reason"] = "archive-path-escapes-root"
                return None
            record["archive_path"] = str(path)
            payload = path.read_bytes()
            record["actual_length"] = len(payload)
            record["actual_sha256"] = hashlib.sha256(payload).hexdigest()
            if len(payload) != expected_length:
                record["reason"] = "length-mismatch"
                return None
            if record["actual_sha256"] != expected_sha256:
                record["reason"] = "sha256-mismatch"
                return None
            text = payload.decode("utf-8")
        except (OSError, RuntimeError, UnicodeError) as error:
            record["reason"] = "archive-read-error"
            record["error"] = str(error)
            return None
        record["status"] = "verified"
        self._texts[relative_path] = text
        return text
