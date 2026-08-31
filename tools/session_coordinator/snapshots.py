from __future__ import annotations

import json
import os
import re
import sqlite3
import threading
import zlib
from dataclasses import dataclass
from pathlib import Path

from .baselines import hash_bytes, hash_file
from .database import Database
from .models import CoordinatorError, utc_text
from .processes import process_is_alive


_OBJECT_TEMPORARY_NAME = re.compile(
    r"^(?P<suffix>[0-9a-f]{62})\.tmp-(?P<pid>[1-9][0-9]*)-(?P<thread>[0-9]+)$"
)


class ObjectStore:
    def __init__(self, database: Database, root: str | Path):
        self.database = database
        self.root = Path(root)

    def put(
        self, content: bytes, *, connection: sqlite3.Connection | None = None
    ) -> str:
        if connection is None:
            with self.database.transaction() as owned_connection:
                return self._put(content, owned_connection)
        return self._put(content, connection)

    def _put(self, content: bytes, connection: sqlite3.Connection) -> str:
        object_hash = hash_bytes(content)
        target = self._path(object_hash)
        try:
            existing = target.read_bytes()
            valid = hash_bytes(zlib.decompress(existing)) == object_hash
        except (FileNotFoundError, OSError, zlib.error):
            existing = b""
            valid = False
        if valid:
            compressed_byte_count = len(existing)
        else:
            compressed = zlib.compress(content, level=6)
            compressed_byte_count = len(compressed)
            target.parent.mkdir(parents=True, exist_ok=True)
            temporary = target.with_suffix(
                f".tmp-{os.getpid()}-{threading.get_ident()}"
            )
            temporary.write_bytes(compressed)
            try:
                os.replace(temporary, target)
            finally:
                temporary.unlink(missing_ok=True)
        connection.execute(
            """
            INSERT INTO objects(object_hash, byte_count, compressed_byte_count, created_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(object_hash) DO UPDATE SET
                byte_count=excluded.byte_count,
                compressed_byte_count=excluded.compressed_byte_count
            """,
            (object_hash, len(content), compressed_byte_count, utc_text()),
        )
        return object_hash

    def get(self, object_hash: str) -> bytes:
        target = self.path_for_hash(object_hash)
        try:
            content = zlib.decompress(target.read_bytes())
        except (OSError, zlib.error) as error:
            raise CoordinatorError("object_unavailable", f"Object {object_hash} is unavailable") from error
        if hash_bytes(content) != object_hash:
            raise CoordinatorError("object_corrupt", f"Object {object_hash} failed hash verification")
        return content

    def path_for_hash(self, object_hash: str) -> Path:
        """Return the validated on-disk path for retention and integrity tooling."""
        if len(object_hash) != 64 or any(character not in "0123456789abcdef" for character in object_hash):
            raise ValueError("object hash must be a lowercase SHA-256 value")
        return self.root / object_hash[:2] / object_hash[2:]

    _path = path_for_hash

    def reconcile_orphan_files(self) -> int:
        """Remove crash residue published before its database row committed."""
        removed = 0
        self.root.mkdir(parents=True, exist_ok=True)
        with self.database.transaction() as connection:
            known = {
                str(row["object_hash"])
                for row in connection.execute("SELECT object_hash FROM objects")
            }
            for prefix in tuple(self.root.iterdir()):
                if (
                    prefix.is_symlink()
                    or not prefix.is_dir()
                    or len(prefix.name) != 2
                    or any(character not in "0123456789abcdef" for character in prefix.name)
                ):
                    continue
                for candidate in tuple(prefix.iterdir()):
                    temporary = _OBJECT_TEMPORARY_NAME.fullmatch(candidate.name)
                    if temporary is not None:
                        if (
                            not candidate.is_symlink()
                            and candidate.is_file()
                            and not process_is_alive(int(temporary.group("pid")))
                        ):
                            candidate.unlink()
                            removed += 1
                        continue
                    object_hash = prefix.name + candidate.name
                    if (
                        candidate.is_symlink()
                        or not candidate.is_file()
                        or len(object_hash) != 64
                        or any(
                            character not in "0123456789abcdef"
                            for character in object_hash
                        )
                        or object_hash in known
                    ):
                        continue
                    candidate.unlink()
                    removed += 1
                try:
                    prefix.rmdir()
                except OSError:
                    pass
        return removed


@dataclass(frozen=True, slots=True)
class SnapshotRecord:
    snapshot_id: int
    session_id: str
    baseline_epoch: int | None
    purpose: str
    manifest: dict[str, str | None]


@dataclass(frozen=True, slots=True)
class RestorePreview:
    path: str
    snapshot_hash: str | None
    current_hash: str | None
    would_change: bool


class SnapshotService:
    def __init__(self, database: Database, repo_root: str | Path, object_store: ObjectStore):
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        self.object_store = object_store

    def create(
        self,
        *,
        session_id: str,
        paths: list[str] | tuple[str, ...],
        baseline_epoch: int | None,
        purpose: str,
    ) -> SnapshotRecord:
        if not purpose.strip():
            raise ValueError("snapshot purpose cannot be empty")
        contents: list[tuple[str, bytes | None]] = []
        for value in paths:
            display_path, absolute_path = self._resolve(value)
            contents.append(
                (display_path, absolute_path.read_bytes() if absolute_path.is_file() else None)
            )
        with self.database.transaction() as connection:
            manifest = {
                display_path: self.object_store.put(content, connection=connection)
                if content is not None
                else None
                for display_path, content in contents
            }
            cursor = connection.execute(
                """
                INSERT INTO snapshots(session_id, baseline_epoch, manifest_json, purpose, created_at)
                VALUES (?, ?, ?, ?, ?)
                """,
                (
                    session_id,
                    baseline_epoch,
                    json.dumps(manifest, sort_keys=True),
                    purpose,
                    utc_text(),
                ),
            )
            snapshot_id = int(cursor.lastrowid)
        return self.get(snapshot_id)

    def get(self, snapshot_id: int) -> SnapshotRecord:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM snapshots WHERE snapshot_id = ?", (snapshot_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError("snapshot_not_found", f"Unknown snapshot {snapshot_id}")
        return SnapshotRecord(
            snapshot_id=int(row["snapshot_id"]),
            session_id=row["session_id"],
            baseline_epoch=row["baseline_epoch"],
            purpose=row["purpose"],
            manifest=json.loads(row["manifest_json"]),
        )

    def restore_preview(self, snapshot_id: int) -> list[RestorePreview]:
        snapshot = self.get(snapshot_id)
        previews: list[RestorePreview] = []
        for path, snapshot_hash in snapshot.manifest.items():
            _, absolute_path = self._resolve(path)
            current_hash = hash_file(absolute_path)
            previews.append(
                RestorePreview(
                    path=path,
                    snapshot_hash=snapshot_hash,
                    current_hash=current_hash,
                    would_change=snapshot_hash != current_hash,
                )
            )
        return previews

    def _resolve(self, value: str) -> tuple[str, Path]:
        absolute = (self.repo_root / value).resolve()
        try:
            relative = absolute.relative_to(self.repo_root)
        except ValueError as error:
            raise CoordinatorError("path_outside_repo", f"Path is outside repository: {value}") from error
        return relative.as_posix(), absolute
