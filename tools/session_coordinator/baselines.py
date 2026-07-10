from __future__ import annotations

import hashlib
import json
import subprocess
from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path
from sqlite3 import Row

from .database import Database
from .models import CoordinatorError, utc_text


class BaselineHealth(StrEnum):
    HEALTHY = "healthy"
    DEGRADED = "degraded"


@dataclass(frozen=True, slots=True)
class BaselineEpoch:
    epoch_id: int
    head_commit: str
    index_tree: str
    health: BaselineHealth
    manifest: dict[str, str]
    degraded_reason: str | None


@dataclass(frozen=True, slots=True)
class WorkspaceChange:
    path: str
    kind: str
    baseline_hash: str | None
    current_hash: str | None


def hash_bytes(content: bytes) -> str:
    return hashlib.sha256(content).hexdigest()


def hash_file(path: Path) -> str | None:
    if not path.is_file():
        return None
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


class BaselineService:
    def __init__(self, database: Database, repo_root: str | Path):
        self.database = database
        self.repo_root = Path(repo_root).resolve()

    def initialize(self) -> BaselineEpoch:
        try:
            return self.current()
        except CoordinatorError as error:
            if error.code != "baseline_missing":
                raise
        return self._capture(BaselineHealth.HEALTHY, reason="initial baseline")

    def current(self) -> BaselineEpoch:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM baseline_epochs ORDER BY epoch_id DESC LIMIT 1"
            ).fetchone()
        if row is None:
            raise CoordinatorError("baseline_missing", "No workspace baseline has been initialized")
        return self._from_row(row)

    def refresh_for_head_change(self) -> BaselineEpoch:
        current = self.initialize()
        if current.head_commit == self._git_output("rev-parse", "HEAD"):
            return current
        return self._capture(BaselineHealth.HEALTHY, reason="HEAD changed")

    def scan(self) -> list[WorkspaceChange]:
        baseline = self.initialize()
        current_manifest = self.build_manifest()
        changes = self._unattributed_changes(
            self._compare(baseline.manifest, current_manifest)
        )
        if changes:
            with self.database.transaction() as connection:
                connection.execute(
                    """
                    UPDATE baseline_epochs
                    SET health = ?, degraded_at = ?, degraded_reason = ?
                    WHERE epoch_id = ?
                    """,
                    (
                        BaselineHealth.DEGRADED.value,
                        utc_text(),
                        f"{len(changes)} unaccepted workspace change(s)",
                        baseline.epoch_id,
                    ),
                )
                connection.execute(
                    "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                    (
                        "baseline.degraded",
                        json.dumps({"epoch_id": baseline.epoch_id, "paths": [item.path for item in changes]}),
                        utc_text(),
                    ),
                )
        return changes

    def _unattributed_changes(
        self, changes: list[WorkspaceChange]
    ) -> list[WorkspaceChange]:
        if not changes:
            return []
        path_keys = [change.path.casefold() for change in changes]
        placeholders = ",".join("?" for _ in path_keys)
        with self.database.connect() as connection:
            rows = connection.execute(
                f"SELECT path_key, content_hash FROM attributions WHERE path_key IN ({placeholders})",
                tuple(path_keys),
            ).fetchall()
        attributed_hashes = {row["path_key"]: row["content_hash"] for row in rows}
        return [
            change
            for change in changes
            if change.path.casefold() not in attributed_hashes
            or attributed_hashes[change.path.casefold()] != change.current_hash
        ]

    def diff(self) -> list[WorkspaceChange]:
        baseline = self.initialize()
        return self._compare(baseline.manifest, self.build_manifest())

    def attribute(self, session_id: str, paths: list[str] | tuple[str, ...]) -> None:
        baseline = self.initialize()
        now = utc_text()
        with self.database.transaction() as connection:
            if connection.execute(
                "SELECT 1 FROM sessions WHERE session_id = ?", (session_id,)
            ).fetchone() is None:
                raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")
            for display_path in paths:
                normalized = self._normalize_repo_path(display_path)
                connection.execute(
                    """
                    INSERT INTO attributions(
                        path_key, display_path, session_id, baseline_epoch, content_hash, attributed_at
                    ) VALUES (?, ?, ?, ?, ?, ?)
                    ON CONFLICT(path_key) DO UPDATE SET
                        display_path = excluded.display_path,
                        session_id = excluded.session_id,
                        baseline_epoch = excluded.baseline_epoch,
                        content_hash = excluded.content_hash,
                        attributed_at = excluded.attributed_at
                    """,
                    (
                        normalized.casefold(),
                        normalized,
                        session_id,
                        baseline.epoch_id,
                        hash_file(self.repo_root / normalized),
                        now,
                    ),
                )

    def accept(self, *, reason: str) -> BaselineEpoch:
        if not reason.strip():
            raise ValueError("baseline acceptance requires a reason")
        return self._capture(BaselineHealth.HEALTHY, reason=reason)

    def build_manifest(self) -> dict[str, str]:
        result = subprocess.run(
            ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
        )
        manifest: dict[str, str] = {}
        for raw_path in result.stdout.split(b"\0"):
            if not raw_path:
                continue
            display_path = raw_path.decode("utf-8", errors="surrogateescape").replace("\\", "/")
            if display_path == ".codex/state" or display_path.startswith(".codex/state/"):
                continue
            content_hash = hash_file(self.repo_root / display_path)
            if content_hash is not None:
                manifest[display_path] = content_hash
        return dict(sorted(manifest.items(), key=lambda item: item[0].casefold()))

    def _capture(self, health: BaselineHealth, *, reason: str) -> BaselineEpoch:
        manifest = self.build_manifest()
        now = utc_text()
        with self.database.transaction() as connection:
            cursor = connection.execute(
                """
                INSERT INTO baseline_epochs(
                    head_commit, index_tree, health, manifest_json, created_at, degraded_reason
                ) VALUES (?, ?, ?, ?, ?, ?)
                """,
                (
                    self._git_output("rev-parse", "HEAD"),
                    self._git_output("write-tree"),
                    health.value,
                    json.dumps(manifest, sort_keys=True),
                    now,
                    None if health is BaselineHealth.HEALTHY else reason,
                ),
            )
            epoch_id = int(cursor.lastrowid)
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "baseline.created",
                    json.dumps({"epoch_id": epoch_id, "reason": reason}),
                    now,
                ),
            )
        return self.current()

    def _git_output(self, *arguments: str) -> str:
        result = subprocess.run(
            ["git", *arguments],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            text=True,
        )
        return result.stdout.strip()

    def _normalize_repo_path(self, value: str) -> str:
        candidate = (self.repo_root / value).resolve()
        try:
            relative = candidate.relative_to(self.repo_root)
        except ValueError as error:
            raise CoordinatorError("path_outside_repo", f"Path is outside repository: {value}") from error
        return relative.as_posix()

    @staticmethod
    def _compare(
        baseline: dict[str, str], current: dict[str, str]
    ) -> list[WorkspaceChange]:
        changes: list[WorkspaceChange] = []
        for path in sorted(set(baseline) | set(current), key=str.casefold):
            old_hash = baseline.get(path)
            new_hash = current.get(path)
            if old_hash == new_hash:
                continue
            kind = "added" if old_hash is None else "deleted" if new_hash is None else "modified"
            changes.append(WorkspaceChange(path, kind, old_hash, new_hash))
        return changes

    @staticmethod
    def _from_row(row: Row) -> BaselineEpoch:
        return BaselineEpoch(
            epoch_id=int(row["epoch_id"]),
            head_commit=row["head_commit"],
            index_tree=row["index_tree"],
            health=BaselineHealth(row["health"]),
            manifest=json.loads(row["manifest_json"]),
            degraded_reason=row["degraded_reason"],
        )
