from __future__ import annotations

import re
from pathlib import Path
from urllib.parse import quote

from ..database import Database
from ..models import CoordinatorError
from .assets import BinaryResponse


_OPAQUE_ID = re.compile(r"^[A-Za-z0-9_-]{1,128}$")
_RANGE = re.compile(r"^bytes=(\d*)-(\d*)$")
_MAX_RANGE_BYTES = 8 * 1024 * 1024
_MAX_DIRECT_BYTES = 16 * 1024 * 1024


class ArtifactDownloadService:
    def __init__(self, database: Database, artifact_root: Path):
        self.database = database
        self.artifact_root = artifact_root.resolve()

    def download(self, artifact_id: str, range_header: str | None) -> BinaryResponse:
        if not _OPAQUE_ID.fullmatch(artifact_id):
            raise CoordinatorError("artifact_not_found", "Artifact was not found")
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT display_name, storage_path FROM workflow_artifacts WHERE artifact_id = ?",
                (artifact_id,),
            ).fetchone()
        if row is None or not row["storage_path"]:
            raise CoordinatorError("artifact_not_found", "Artifact was not found")
        stored = Path(row["storage_path"])
        path = (stored if stored.is_absolute() else self.artifact_root / stored).resolve()
        if not path.is_relative_to(self.artifact_root) or not path.is_file():
            raise CoordinatorError("artifact_not_found", "Artifact was not found")
        size = path.stat().st_size
        try:
            start, end, partial = self._range(size, range_header)
        except CoordinatorError as error:
            if error.code != "invalid_range":
                raise
            return self._invalid_range(size)
        count = max(0, end - start + 1)
        if (partial and count > _MAX_RANGE_BYTES) or (not partial and count > _MAX_DIRECT_BYTES):
            return self._invalid_range(size)
        with path.open("rb") as stream:
            stream.seek(start)
            body = stream.read(count)
        safe_name = quote(str(row["display_name"]), safe="")
        headers = {
            "Content-Type": "application/octet-stream",
            "Content-Disposition": f"attachment; filename*=UTF-8''{safe_name}",
            "Accept-Ranges": "bytes",
            "Cache-Control": "no-store",
            "X-Content-Type-Options": "nosniff",
        }
        if partial:
            headers["Content-Range"] = f"bytes {start}-{end}/{size}"
        return BinaryResponse(206 if partial else 200, body, headers)

    @staticmethod
    def _invalid_range(size: int) -> BinaryResponse:
        return BinaryResponse(
            416,
            b"",
            {
                "Content-Range": f"bytes */{size}",
                "Accept-Ranges": "bytes",
                "Cache-Control": "no-store",
                "X-Content-Type-Options": "nosniff",
            },
        )

    @staticmethod
    def _range(size: int, header: str | None) -> tuple[int, int, bool]:
        if not header:
            return 0, max(0, size - 1), False
        match = _RANGE.fullmatch(header.strip())
        if not match or size == 0:
            raise CoordinatorError("invalid_range", "Artifact byte range is invalid")
        start_text, end_text = match.groups()
        if not start_text and not end_text:
            raise CoordinatorError("invalid_range", "Artifact byte range is invalid")
        if not start_text:
            suffix = int(end_text)
            if suffix <= 0:
                raise CoordinatorError("invalid_range", "Artifact byte range is invalid")
            start, end = max(0, size - suffix), size - 1
        else:
            start = int(start_text)
            end = min(int(end_text), size - 1) if end_text else size - 1
        if start >= size or start > end:
            raise CoordinatorError("invalid_range", "Artifact byte range is unsatisfiable")
        return start, end, True
