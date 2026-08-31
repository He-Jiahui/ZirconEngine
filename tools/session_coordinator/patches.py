from __future__ import annotations

import json
import subprocess
from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path

from .baselines import hash_bytes, hash_file
from .database import Database
from .leases import LeaseService
from .models import CoordinatorError, SessionStatus, utc_text
from .sessions import SessionService
from .snapshots import ObjectStore, SnapshotService


class PatchStatus(StrEnum):
    QUEUED = "queued"
    APPLYING = "applying"
    APPLIED = "applied"
    NEEDS_REBASE = "needs_rebase"
    FAILED = "failed"
    CANCELLED = "cancelled"


@dataclass(frozen=True, slots=True)
class PatchRecord:
    patch_id: int
    session_id: str
    status: PatchStatus
    patch_object_hash: str
    targets: tuple[str, ...]
    base_hashes: dict[str, str | None]
    base_objects: dict[str, str | None]
    current_objects: dict[str, str | None] | None
    error_text: str | None


class PatchService:
    def __init__(
        self,
        database: Database,
        repo_root: str | Path,
        object_store: ObjectStore,
        snapshots: SnapshotService,
        leases: LeaseService,
        sessions: SessionService,
    ):
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        self.object_store = object_store
        self.snapshots = snapshots
        self.leases = leases
        self.sessions = sessions

    def submit(
        self,
        session_id: str,
        patch_text: str,
        targets: list[str] | tuple[str, ...],
    ) -> PatchRecord:
        if not patch_text.strip():
            raise ValueError("patch text cannot be empty")
        normalized = [self.leases.path_policy.normalize(path) for path in targets]
        if not normalized:
            raise ValueError("patch requires at least one explicit target")
        display_paths = tuple(item.display for item in normalized)
        base_contents = {
            item.display: item.absolute.read_bytes() if item.absolute.is_file() else None
            for item in normalized
        }
        base_hashes = {
            path: hash_bytes(content) if content is not None else None
            for path, content in base_contents.items()
        }
        acquisition = self.leases.acquire(session_id, display_paths)
        status = PatchStatus.APPLYING if acquisition.acquired else PatchStatus.QUEUED
        now = utc_text()
        with self.database.transaction() as connection:
            base_objects = {
                path: self.object_store.put(content, connection=connection)
                if content is not None
                else None
                for path, content in base_contents.items()
            }
            patch_hash = self.object_store.put(
                patch_text.encode("utf-8"), connection=connection
            )
            cursor = connection.execute(
                """
                INSERT INTO patches(
                    session_id, patch_object_hash, targets_json, base_hashes_json,
                    base_objects_json, status, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    session_id,
                    patch_hash,
                    json.dumps(display_paths),
                    json.dumps(base_hashes, sort_keys=True),
                    json.dumps(base_objects, sort_keys=True),
                    status.value,
                    now,
                    now,
                ),
            )
            patch_id = int(cursor.lastrowid)
        if not acquisition.acquired:
            self.sessions.set_status(
                session_id,
                SessionStatus.WAITING_LEASE,
                reason=f"patch {patch_id} queued for {', '.join(acquisition.conflicts)}",
            )
            return self.get(patch_id)
        return self._apply(patch_id)

    def get(self, patch_id: int) -> PatchRecord:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM patches WHERE patch_id = ?", (patch_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError("patch_not_found", f"Unknown patch {patch_id}")
        return self._from_row(row)

    def list(
        self,
        *,
        status: PatchStatus | None = None,
        session_id: str | None = None,
    ) -> list[PatchRecord]:
        query = "SELECT * FROM patches"
        clauses: list[str] = []
        parameters: list[object] = []
        if status is not None:
            clauses.append("status = ?")
            parameters.append(status.value)
        if session_id is not None:
            clauses.append("session_id = ?")
            parameters.append(session_id)
        if clauses:
            query += " WHERE " + " AND ".join(clauses)
        query += " ORDER BY created_at, patch_id"
        with self.database.connect() as connection:
            rows = connection.execute(query, tuple(parameters)).fetchall()
        return [self._from_row(row) for row in rows]

    def process_queue(
        self,
        *,
        session_id: str | None = None,
        patch_ids: tuple[int, ...] | None = None,
    ) -> list[PatchRecord]:
        processed: list[PatchRecord] = []
        queued = self.list(status=PatchStatus.QUEUED, session_id=session_id)
        if patch_ids is not None:
            allowed = set(patch_ids)
            queued = [patch for patch in queued if patch.patch_id in allowed]
        for patch in queued:
            try:
                acquisition = self.leases.acquire(patch.session_id, patch.targets)
            except CoordinatorError as error:
                if error.code != "session_not_writable":
                    raise
                # A terminal or stale owner cannot safely apply its queued
                # patch. Keep the durable request for an explicit resume
                # rather than turning an unrelated successful lease release
                # into a client-visible command failure.
                continue
            if not acquisition.acquired:
                continue
            current_hashes = {
                target: hash_file(self.leases.path_policy.normalize(target).absolute)
                for target in patch.targets
            }
            if current_hashes != patch.base_hashes:
                self._update(
                    patch.patch_id,
                    PatchStatus.NEEDS_REBASE,
                    capture_targets=patch.targets,
                    error_text="target content changed after patch was queued",
                )
                self.leases.release(patch.session_id, patch.targets)
                self.sessions.set_status(
                    patch.session_id,
                    SessionStatus.ACTIVE,
                    reason=f"patch {patch.patch_id} requires rebase",
                )
                processed.append(self.get(patch.patch_id))
                continue
            self._update(patch.patch_id, PatchStatus.APPLYING)
            processed.append(self._apply(patch.patch_id))
        return processed

    def _apply(self, patch_id: int) -> PatchRecord:
        patch = self.get(patch_id)
        patch_bytes = self.object_store.get(patch.patch_object_hash)
        epoch = self._current_epoch()
        try:
            current_hashes = {
                target: hash_file(self.leases.path_policy.normalize(target).absolute)
                for target in patch.targets
            }
            if current_hashes != patch.base_hashes:
                self._update(
                    patch.patch_id,
                    PatchStatus.NEEDS_REBASE,
                    capture_targets=patch.targets,
                    error_text="target content changed before patch application",
                )
                self.sessions.set_status(
                    patch.session_id,
                    SessionStatus.ACTIVE,
                    reason=f"patch {patch.patch_id} requires rebase",
                )
                return self.get(patch.patch_id)
            self.snapshots.create(
                session_id=patch.session_id,
                paths=patch.targets,
                baseline_epoch=epoch,
                purpose=f"before patch {patch.patch_id}",
            )
            subprocess.run(
                ["git", "apply", "--check", "-"],
                cwd=self.repo_root,
                input=patch_bytes,
                check=True,
                capture_output=True,
            )
            subprocess.run(
                ["git", "apply", "--whitespace=nowarn", "-"],
                cwd=self.repo_root,
                input=patch_bytes,
                check=True,
                capture_output=True,
            )
            self.snapshots.create(
                session_id=patch.session_id,
                paths=patch.targets,
                baseline_epoch=epoch,
                purpose=f"after patch {patch.patch_id}",
            )
            now = utc_text()
            with self.database.transaction() as connection:
                attribution_rows = []
                for target in patch.targets:
                    normalized = self.leases.path_policy.normalize(target)
                    attribution_rows.append(
                        (
                            normalized.key,
                            normalized.display,
                            patch.session_id,
                            epoch,
                            hash_file(normalized.absolute),
                            now,
                        )
                    )
                connection.executemany(
                    """
                    INSERT INTO attributions(
                        path_key, display_path, session_id, baseline_epoch,
                        content_hash, attributed_at
                    ) VALUES (?, ?, ?, ?, ?, ?)
                    ON CONFLICT(path_key) DO UPDATE SET
                        display_path = excluded.display_path,
                        session_id = excluded.session_id,
                        baseline_epoch = excluded.baseline_epoch,
                        content_hash = excluded.content_hash,
                        attributed_at = excluded.attributed_at
                    """,
                    attribution_rows,
                )
            self._update(patch.patch_id, PatchStatus.APPLIED, applied=True)
            self.sessions.set_status(
                patch.session_id,
                SessionStatus.ACTIVE,
                reason=f"patch {patch.patch_id} applied",
            )
        except subprocess.CalledProcessError as error:
            message = error.stderr.decode("utf-8", errors="replace").strip()
            self._update(patch.patch_id, PatchStatus.FAILED, error_text=message)
        finally:
            self.leases.release(patch.session_id, patch.targets)
        return self.get(patch.patch_id)

    def _capture_objects(
        self, targets: tuple[str, ...], *, connection
    ) -> dict[str, str | None]:
        result: dict[str, str | None] = {}
        for target in targets:
            absolute = self.leases.path_policy.normalize(target).absolute
            result[target] = (
                self.object_store.put(absolute.read_bytes(), connection=connection)
                if absolute.is_file()
                else None
            )
        return result

    def _current_epoch(self) -> int | None:
        with self.database.connect() as connection:
            row = connection.execute("SELECT MAX(epoch_id) FROM baseline_epochs").fetchone()
        return row[0] if row else None

    def _update(
        self,
        patch_id: int,
        status: PatchStatus,
        *,
        current_objects: dict[str, str | None] | None = None,
        capture_targets: tuple[str, ...] | None = None,
        error_text: str | None = None,
        applied: bool = False,
    ) -> None:
        now = utc_text()
        with self.database.transaction() as connection:
            if capture_targets is not None:
                current_objects = self._capture_objects(
                    capture_targets, connection=connection
                )
            connection.execute(
                """
                UPDATE patches
                SET status = ?, current_objects_json = COALESCE(?, current_objects_json),
                    error_text = ?, updated_at = ?, applied_at = CASE WHEN ? THEN ? ELSE applied_at END
                WHERE patch_id = ?
                """,
                (
                    status.value,
                    json.dumps(current_objects, sort_keys=True) if current_objects is not None else None,
                    error_text,
                    now,
                    1 if applied else 0,
                    now,
                    patch_id,
                ),
            )

    @staticmethod
    def _from_row(row) -> PatchRecord:
        return PatchRecord(
            patch_id=int(row["patch_id"]),
            session_id=row["session_id"],
            status=PatchStatus(row["status"]),
            patch_object_hash=row["patch_object_hash"],
            targets=tuple(json.loads(row["targets_json"])),
            base_hashes=json.loads(row["base_hashes_json"]),
            base_objects=json.loads(row["base_objects_json"]),
            current_objects=(
                json.loads(row["current_objects_json"])
                if row["current_objects_json"] is not None
                else None
            ),
            error_text=row["error_text"],
        )
