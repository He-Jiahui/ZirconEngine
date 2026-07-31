from __future__ import annotations

import subprocess
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from sqlite3 import Row

from .baselines import BaselineService, WorkspaceChange, hash_file
from .database import Database
from .leases import LeaseService
from .models import CoordinatorError, parse_utc, utc_now


_EXECUTABLE_SESSION_STATUSES = frozenset(
    {"registered", "active", "waiting_lease", "resolving_failure", "waiting_validation", "finalizing"}
)


@dataclass(frozen=True, slots=True)
class OwnershipMatrixEntry:
    path: str
    kind: str
    current_hash: str | None
    owner_session_id: str | None
    owner_status: str | None
    lease_session_id: str | None
    state: str
    blocking_reasons: tuple[str, ...]

    def to_dict(self) -> dict[str, object]:
        return {
            "path": self.path,
            "kind": self.kind,
            "currentHash": self.current_hash,
            "ownerSessionId": self.owner_session_id,
            "ownerStatus": self.owner_status,
            "leaseSessionId": self.lease_session_id,
            "state": self.state,
            "blockingReasons": list(self.blocking_reasons),
        }


@dataclass(frozen=True, slots=True)
class OwnershipCandidate:
    session_id: str
    paths: tuple[str, ...]

    def to_dict(self) -> dict[str, object]:
        return {"sessionId": self.session_id, "paths": list(self.paths)}


@dataclass(frozen=True, slots=True)
class OwnershipMatrix:
    baseline_epoch: int
    entries: tuple[OwnershipMatrixEntry, ...]
    candidates: tuple[OwnershipCandidate, ...]

    def to_dict(self) -> dict[str, object]:
        return {
            "baselineEpoch": self.baseline_epoch,
            "entries": [entry.to_dict() for entry in self.entries],
            "candidates": [candidate.to_dict() for candidate in self.candidates],
        }


class OwnershipMatrixService:
    """Derive exact main-integration candidates without assigning missing ownership."""

    def __init__(
        self,
        database: Database,
        baselines: BaselineService,
        leases: LeaseService,
    ) -> None:
        self.database = database
        self.baselines = baselines
        self.leases = leases

    def build(
        self,
        *,
        prefix: str | None = None,
        now: datetime | None = None,
    ) -> OwnershipMatrix:
        current_time = now or utc_now()
        prefix_key = self._prefix_key(prefix)
        if prefix_key is None:
            baseline = self.baselines.current()
            baseline_epoch = baseline.epoch_id
            filtered = tuple(self.baselines.diff())
        else:
            baseline_epoch, filtered = self._changes_for_prefix(prefix_key)
        with self.database.connect() as connection:
            attributions = self._attributions(connection)
            leases = self._leases(connection, current_time)
        entries = tuple(
            self._entry(change, baseline_epoch, attributions.get(change.path.casefold()), leases)
            for change in filtered
        )
        candidate_paths: dict[str, list[str]] = {}
        for entry in entries:
            if entry.state == "integration_ready" and entry.owner_session_id is not None:
                candidate_paths.setdefault(entry.owner_session_id, []).append(entry.path)
        candidates = tuple(
            OwnershipCandidate(session_id, tuple(sorted(paths, key=str.casefold)))
            for session_id, paths in sorted(candidate_paths.items(), key=lambda item: item[0])
        )
        return OwnershipMatrix(baseline_epoch, entries, candidates)

    def _prefix_key(self, prefix: str | None) -> str | None:
        if prefix is None:
            return None
        return self.leases.path_policy.normalize(prefix).key

    def _changes_for_prefix(self, prefix_key: str) -> tuple[int, tuple[WorkspaceChange, ...]]:
        """Compare one subtree without loading the complete retained manifest into Python."""
        with self.database.connect() as connection:
            epoch = connection.execute("SELECT MAX(epoch_id) FROM baseline_epochs").fetchone()[0]
            if epoch is None:
                raise CoordinatorError(
                    "baseline_not_initialized",
                    "Ownership matrix requires an initialized baseline",
                )
            rows = connection.execute(
                """
                SELECT manifest.key AS path, manifest.value AS content_hash
                FROM baseline_epochs, json_each(baseline_epochs.manifest_json) AS manifest
                WHERE baseline_epochs.epoch_id=?
                  AND (manifest.key=? OR manifest.key LIKE ?)
                """,
                (int(epoch), prefix_key, prefix_key + "/%"),
            ).fetchall()
        baseline = {str(row["path"]): str(row["content_hash"]) for row in rows}
        paths = set(baseline)
        result = subprocess.run(
            ["git", "ls-files", "--others", "--exclude-standard", "--", prefix_key],
            cwd=self.baselines.repo_root,
            check=True,
            capture_output=True,
            text=True,
        )
        paths.update(line.strip() for line in result.stdout.splitlines() if line.strip())
        changes: list[WorkspaceChange] = []
        for path in sorted(paths, key=str.casefold):
            current_hash = hash_file(Path(self.baselines.repo_root) / path)
            baseline_hash = baseline.get(path)
            if current_hash == baseline_hash:
                continue
            kind = (
                "added"
                if baseline_hash is None
                else "deleted"
                if current_hash is None
                else "modified"
            )
            changes.append(WorkspaceChange(path, kind, baseline_hash, current_hash))
        return int(epoch), tuple(changes)

    @staticmethod
    def _attributions(connection) -> dict[str, Row]:
        rows = connection.execute(
            """
            SELECT attributions.path_key, attributions.session_id, attributions.baseline_epoch,
                   attributions.content_hash, sessions.status AS owner_status
            FROM attributions
            JOIN sessions ON sessions.session_id=attributions.session_id
            """
        ).fetchall()
        return {str(row["path_key"]): row for row in rows}

    @staticmethod
    def _leases(connection, current_time: datetime) -> tuple[Row, ...]:
        rows = connection.execute(
            """
            SELECT path_key, session_id, expires_at
            FROM leases ORDER BY path_key
            """
        ).fetchall()
        return tuple(row for row in rows if current_time <= parse_utc(str(row["expires_at"])))

    @staticmethod
    def _entry(
        change: WorkspaceChange,
        baseline_epoch: int,
        attribution: Row | None,
        leases: tuple[Row, ...],
    ) -> OwnershipMatrixEntry:
        path_key = change.path.casefold()
        owner = str(attribution["session_id"]) if attribution is not None else None
        owner_status = str(attribution["owner_status"]) if attribution is not None else None
        matching_leases = tuple(
            row
            for row in leases
            if path_key == str(row["path_key"])
            or path_key.startswith(str(row["path_key"]) + "/")
        )
        lease_owner = str(matching_leases[0]["session_id"]) if matching_leases else None
        reasons: list[str] = []
        if change.kind == "deleted":
            reasons.append("deletion_requires_explicit_candidate")
        if attribution is None:
            reasons.append("attribution_missing")
        else:
            if attribution["content_hash"] != change.current_hash:
                reasons.append("attribution_hash_stale")
            if int(attribution["baseline_epoch"]) != baseline_epoch:
                reasons.append("attribution_baseline_stale")
            if owner_status not in _EXECUTABLE_SESSION_STATUSES:
                reasons.append("owner_not_executable")
            if not any(str(row["session_id"]) == owner for row in matching_leases):
                reasons.append(
                    "lease_owner_mismatch" if matching_leases else "live_lease_missing"
                )
        return OwnershipMatrixEntry(
            change.path,
            change.kind,
            change.current_hash,
            owner,
            owner_status,
            lease_owner,
            "integration_ready" if not reasons else "unowned",
            tuple(reasons),
        )
