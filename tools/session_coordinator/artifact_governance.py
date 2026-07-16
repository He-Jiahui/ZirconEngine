from __future__ import annotations

import json
import os
import shutil
from dataclasses import dataclass
from pathlib import Path

from .database import Database
from .models import CoordinatorError, utc_text


@dataclass(frozen=True, slots=True)
class UnmanagedArtifact:
    root: str
    path: str


@dataclass(frozen=True, slots=True)
class UnmanagedArtifactCleanup:
    deleted: tuple[str, ...]
    failed: tuple[UnmanagedArtifact, ...]


class ArtifactGovernanceService:
    """Detect and retire D/E/F artifacts that were not first registered by the coordinator."""

    def __init__(self, database: Database, *, roots: tuple[Path, ...]):
        self.database = database
        self.roots = tuple(dict.fromkeys(root.resolve() for root in roots))

    def scan(self) -> tuple[UnmanagedArtifact, ...]:
        managed = self._managed_paths()
        candidates: list[UnmanagedArtifact] = []
        for root in self.roots:
            if not root.is_dir() or root.is_symlink():
                continue
            candidates.extend(self._scan_children(root, root, managed))
        return tuple(sorted(candidates, key=lambda item: item.path.casefold()))

    def require_clean(self) -> None:
        candidates = self.scan()
        if candidates:
            raise CoordinatorError(
                "unmanaged_artifacts_detected",
                "Coordinator-managed work is blocked until unregistered D/E/F artifacts are removed",
                details={
                    "paths": [candidate.path for candidate in candidates],
                    "managedCargo": list(self._managed_cargo_snapshot()),
                    "cleanupReservations": list(self._cleanup_reservation_snapshot()),
                },
            )

    def cleanup(self, *, max_candidates: int = 1) -> UnmanagedArtifactCleanup:
        if max_candidates < 1:
            raise ValueError("max_candidates must be positive")
        deleted: list[str] = []
        failed: list[UnmanagedArtifact] = []
        for candidate in self.scan()[:max_candidates]:
            path = Path(candidate.path)
            if not path.is_dir() or path.is_symlink() or not self._is_still_unmanaged(path):
                continue
            # Persist intent before the potentially long filesystem operation.
            # A stopped worker then leaves an auditable start record instead of
            # making a completed deletion appear to have happened silently.
            self._record_event("artifact.unmanaged_delete_started", candidate)
            try:
                shutil.rmtree(path)
            except OSError:
                failed.append(candidate)
                continue
            deleted.append(candidate.path)
            self._record_event("artifact.unmanaged_deleted", candidate)
        for candidate in failed:
            self._record_event("artifact.unmanaged_delete_failed", candidate)
        return UnmanagedArtifactCleanup(tuple(deleted), tuple(failed))

    def _is_still_unmanaged(self, path: Path) -> bool:
        normalized = path.resolve()
        return any(candidate.path == str(normalized) for candidate in self.scan())

    def _scan_children(
        self, root: Path, directory: Path, managed: tuple[Path, ...]
    ) -> list[UnmanagedArtifact]:
        candidates: list[UnmanagedArtifact] = []
        try:
            children = tuple(directory.iterdir())
        except OSError:
            return candidates
        for child in children:
            if not child.is_dir() or child.is_symlink():
                continue
            resolved = child.resolve()
            if any(resolved == path for path in managed):
                continue
            if any(path.is_relative_to(resolved) for path in managed):
                candidates.extend(self._scan_children(root, resolved, managed))
                continue
            candidates.append(UnmanagedArtifact(str(root), str(resolved)))
        return candidates

    def _managed_paths(self) -> tuple[Path, ...]:
        paths: set[Path] = set()
        with self.database.connect() as connection:
            # A successful cleanup is terminal evidence that the old target is
            # no longer service-managed.  Retaining every historical path here
            # makes the 30-second guardian resolve thousands of dead entries
            # and silently exempts a later manually recreated directory.
            for row in connection.execute(
                """SELECT target_dir FROM cargo_jobs
                   WHERE cleanup_status <> 'deleted'
                      OR status IN ('leased', 'running')"""
            ):
                self._add_managed_path(paths, row["target_dir"])
            # A cleanup reservation survives after the deleting worker has
            # released SQLite but before its filesystem operation completes.
            # It must remain a managed descendant during that interval: scan
            # may recurse through its parents, but must still detect unrelated
            # sibling directories rather than exempting the whole tree.
            for row in connection.execute(
                "SELECT target_dir FROM cleanup_reservations"
            ):
                self._add_managed_path(paths, row["target_dir"])
            for row in connection.execute("SELECT job_root, target_root FROM validation_copies"):
                self._add_managed_path(paths, row["job_root"])
                self._add_managed_path(paths, row["target_root"])
            for row in connection.execute(
                "SELECT storage_path FROM workflow_artifacts WHERE storage_path IS NOT NULL"
            ):
                self._add_managed_path(paths, row["storage_path"])
        return tuple(paths)

    def _managed_cargo_snapshot(self) -> tuple[dict[str, str], ...]:
        with self.database.connect() as connection:
            rows = connection.execute(
                """SELECT job_id, status, target_dir FROM cargo_jobs
                   WHERE cleanup_status <> 'deleted'
                      OR status IN ('leased', 'running')
                   ORDER BY created_at, job_id
                   LIMIT 100"""
            ).fetchall()
        return tuple(
            {
                "jobId": str(row["job_id"]),
                "status": str(row["status"]),
                "targetDir": str(row["target_dir"]),
            }
            for row in rows
        )

    def _cleanup_reservation_snapshot(self) -> tuple[dict[str, str], ...]:
        with self.database.connect() as connection:
            rows = connection.execute(
                """SELECT target_dir, reserved_at FROM cleanup_reservations
                   ORDER BY reserved_at, target_key LIMIT 100"""
            ).fetchall()
        return tuple(
            {"targetDir": str(row["target_dir"]), "reservedAt": str(row["reserved_at"])}
            for row in rows
        )

    def _add_managed_path(self, paths: set[Path], value: str | None) -> None:
        if not value:
            return
        candidate = Path(value)
        if not candidate.is_absolute():
            return
        try:
            resolved = candidate.resolve()
        except OSError:
            return
        if any(resolved == root or resolved.is_relative_to(root) for root in self.roots):
            paths.add(resolved)

    def _record_event(self, event_type: str, candidate: UnmanagedArtifact) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    event_type,
                    json.dumps({"path": candidate.path, "root": candidate.root}, sort_keys=True),
                    utc_text(),
                ),
            )
