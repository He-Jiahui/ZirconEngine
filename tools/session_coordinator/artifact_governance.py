from __future__ import annotations

import json
import os
import re
import stat
import threading
import uuid
from dataclasses import dataclass
from pathlib import Path

from .cargo_jobs import target_identity, targets_overlap
from .artifact_product_staging import (
    ArtifactProductStagingLease,
    ArtifactProductStagingService,
)
from .database import Database
from .models import CoordinatorError, utc_text
from .processes import process_creation_time, process_is_alive
from .windows_tree_delete import filesystem_identity, remove_tree


_FIXTURE_PREFIX = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")
_FIXTURE_LEASE_ID = re.compile(r"^[0-9a-f]{32}$")


@dataclass(frozen=True, slots=True)
class UnmanagedArtifact:
    root: str
    path: str


@dataclass(frozen=True, slots=True)
class UnmanagedArtifactCleanup:
    deleted: tuple[str, ...]
    failed: tuple[UnmanagedArtifact, ...]


@dataclass(frozen=True, slots=True)
class ArtifactFixtureLease:
    lease_id: str
    path: str
    prefix: str
    owner_pid: int
    owner_process_creation_time: str
    status: str
    created_at: str
    released_at: str | None

    def to_dict(self) -> dict[str, object]:
        return {
            "leaseId": self.lease_id,
            "path": self.path,
            "prefix": self.prefix,
            "ownerPid": self.owner_pid,
            "ownerProcessCreationTime": self.owner_process_creation_time,
            "status": self.status,
            "createdAt": self.created_at,
            "releasedAt": self.released_at,
        }


class ArtifactGovernanceService:
    """Detect and retire D/E/F artifacts that were not first registered by the coordinator."""

    def __init__(self, database: Database, *, roots: tuple[Path, ...]):
        self.database = database
        configured_roots: list[Path] = []
        for value in roots:
            root = Path(os.path.abspath(value))
            try:
                resolved = root.resolve(strict=False)
            except OSError as error:
                raise CoordinatorError(
                    "artifact_governance_root_unavailable",
                    f"Artifact governance root cannot be verified: {root}",
                ) from error
            if resolved != root or _existing_reparse_point(root):
                raise CoordinatorError(
                    "artifact_governance_root_reparse",
                    f"Artifact governance root cannot be a filesystem reparse point: {root}",
                )
            configured_roots.append(root)
        self.roots = tuple(dict.fromkeys(configured_roots))
        self._cleanup_lock = threading.Lock()
        self.product_staging = ArtifactProductStagingService(
            database,
            roots=self.roots,
            managed_overlap=self._managed_overlap_in_connection,
        )

    def acquire_product_staging(
        self, purpose: str, *, final_path: str | Path, owner_pid: int
    ) -> ArtifactProductStagingLease:
        return self.product_staging.acquire(
            purpose, final_path=final_path, owner_pid=owner_pid
        )

    def begin_product_staging_publish(
        self, lease_id: str, *, owner_pid: int
    ) -> ArtifactProductStagingLease:
        return self.product_staging.begin_publish(lease_id, owner_pid=owner_pid)

    def complete_product_staging_publish(
        self, lease_id: str, *, owner_pid: int
    ) -> ArtifactProductStagingLease:
        return self.product_staging.complete_publish(lease_id, owner_pid=owner_pid)

    def release_product_staging(
        self, lease_id: str, *, owner_pid: int
    ) -> ArtifactProductStagingLease:
        return self.product_staging.release(lease_id, owner_pid=owner_pid)

    def acquire_fixture(self, prefix: str, *, owner_pid: int) -> ArtifactFixtureLease:
        if not isinstance(prefix, str) or not _FIXTURE_PREFIX.fullmatch(prefix):
            raise CoordinatorError(
                "artifact_fixture_prefix_invalid",
                "Fixture prefix must contain only letters, digits, dots, underscores, or hyphens",
            )
        if (
            isinstance(owner_pid, bool)
            or not isinstance(owner_pid, int)
            or owner_pid <= 0
        ):
            raise CoordinatorError(
                "artifact_fixture_owner_invalid", "Fixture owner PID must be positive"
            )
        owner_identity = self._live_process_creation_time(owner_pid)
        if owner_identity is None:
            raise CoordinatorError(
                "artifact_fixture_owner_not_alive",
                "Fixture owner process is not alive",
                details={"ownerPid": owner_pid},
            )
        root = next(
            (item for item in self.roots if item.name.casefold() == "zirconbuilds"),
            None,
        )
        if root is None:
            raise CoordinatorError(
                "artifact_fixture_root_unavailable",
                "No governed ZirconBuilds root is configured for test fixtures",
            )
        lease_id = uuid.uuid4().hex
        target = root / f"mvp-test-fixtures-{owner_pid}" / f"{prefix}-{lease_id}"
        target_key = target_identity(target)
        created_at = utc_text()
        with self.database.transaction() as connection:
            for row in connection.execute(
                "SELECT target_dir FROM cleanup_reservations"
            ):
                if targets_overlap(target_key, target_identity(str(row["target_dir"]))):
                    raise CoordinatorError(
                        "artifact_fixture_cleanup_reserved",
                        "Fixture path overlaps an active artifact cleanup reservation",
                        details={"path": str(target)},
                    )
            connection.execute(
                """INSERT INTO artifact_fixture_leases(
                       lease_id, target_key, target_dir, prefix, owner_pid,
                       owner_process_creation_time, status, created_at
                   ) VALUES (?, ?, ?, ?, ?, ?, 'active', ?)""",
                (
                    lease_id,
                    target_key,
                    str(target),
                    prefix,
                    owner_pid,
                    owner_identity,
                    created_at,
                ),
            )
            self._insert_fixture_event(
                connection,
                "artifact.fixture_acquired",
                lease_id=lease_id,
                path=str(target),
                owner_pid=owner_pid,
            )
        return ArtifactFixtureLease(
            lease_id,
            str(target),
            prefix,
            owner_pid,
            owner_identity,
            "active",
            created_at,
            None,
        )

    def release_fixture(
        self, lease_id: str, *, owner_pid: int
    ) -> ArtifactFixtureLease:
        if not isinstance(lease_id, str) or not _FIXTURE_LEASE_ID.fullmatch(lease_id):
            raise CoordinatorError(
                "artifact_fixture_lease_invalid",
                "Fixture lease ID must be 32 lowercase hex digits",
            )
        with self.database.transaction() as connection:
            row = connection.execute(
                "SELECT * FROM artifact_fixture_leases WHERE lease_id=?", (lease_id,)
            ).fetchone()
            if row is None:
                raise CoordinatorError(
                    "artifact_fixture_lease_not_found", "Fixture lease does not exist"
                )
            if owner_pid != int(row["owner_pid"]):
                raise CoordinatorError(
                    "artifact_fixture_owner_mismatch",
                    "Fixture lease belongs to another process",
                )
            if row["status"] == "released":
                return self._fixture_lease(row)
            if row["status"] != "active":
                raise CoordinatorError(
                    "artifact_fixture_lease_terminal",
                    "Fixture lease was already recovered by artifact governance",
                )
            if not self._fixture_owner_matches(row):
                raise CoordinatorError(
                    "artifact_fixture_owner_mismatch",
                    "Fixture lease process identity no longer matches its owner",
                )
            path = Path(str(row["target_dir"]))
            try:
                path.lstat()
            except FileNotFoundError:
                pass
            except OSError as error:
                raise CoordinatorError(
                    "artifact_fixture_path_unverifiable",
                    "Fixture path could not be verified before release",
                    details={"path": str(path)},
                ) from error
            else:
                raise CoordinatorError(
                    "artifact_fixture_still_exists",
                    "Fixture directory must be removed before its lease is released",
                    details={"path": str(path)},
                )
            released_at = utc_text()
            connection.execute(
                """UPDATE artifact_fixture_leases
                   SET status='released', released_at=?
                   WHERE lease_id=? AND status='active'""",
                (released_at, lease_id),
            )
            self._insert_fixture_event(
                connection,
                "artifact.fixture_released",
                lease_id=lease_id,
                path=str(path),
                owner_pid=owner_pid,
            )
            values = dict(row)
            values.update(status="released", released_at=released_at)
            return self._fixture_lease(values)

    def scan(self) -> tuple[UnmanagedArtifact, ...]:
        managed = self._managed_paths()
        candidates: list[UnmanagedArtifact] = []
        for root in self.roots:
            if _is_reparse_point(root) or not root.is_dir():
                continue
            candidates.extend(self._scan_children(root, root, managed))
        return tuple(sorted(candidates, key=lambda item: item.path.casefold()))

    def require_clean(self) -> None:
        with self._cleanup_lock:
            self._recover_missing_artifact_reservations()
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

    def _recover_missing_artifact_reservations(self) -> None:
        with self.database.connect() as connection:
            rows = connection.execute(
                """SELECT target_dir FROM cleanup_reservations
                   WHERE reservation_kind='artifact'
                   ORDER BY reserved_at, target_key"""
            ).fetchall()
        for row in rows:
            path = Path(str(row["target_dir"]))
            root = self._root_for(path)
            if root is None:
                continue
            try:
                path.lstat()
            except FileNotFoundError:
                self._complete_candidate(
                    UnmanagedArtifact(root, str(path)), error=None, recovered=True
                )
            except OSError:
                continue

    def cleanup(self, *, max_candidates: int = 1) -> UnmanagedArtifactCleanup:
        if max_candidates < 1:
            raise ValueError("max_candidates must be positive")
        with self._cleanup_lock:
            self.product_staging.recover()
            self._recover_missing_fixture_leases()
            recovered = self._recover_reservations(max_candidates=max_candidates)
            # A retryable reservation failure must not consume the deletion budget;
            # otherwise one locked producer starves every independent candidate.
            processed = len(recovered.deleted)
            if processed >= max_candidates:
                return recovered
            attempted_paths = {
                item.path.casefold() for item in recovered.failed
            } | {path.casefold() for path in recovered.deleted}
            current = self._cleanup(
                max_candidates=max_candidates - processed,
                excluded_paths=attempted_paths,
            )
            return UnmanagedArtifactCleanup(
                recovered.deleted + current.deleted,
                recovered.failed + current.failed,
            )

    def _cleanup(
        self, *, max_candidates: int, excluded_paths: set[str] | None = None
    ) -> UnmanagedArtifactCleanup:
        deleted: list[str] = []
        failed: list[UnmanagedArtifact] = []
        excluded = excluded_paths or set()
        candidates = tuple(
            candidate
            for candidate in self.scan()
            if candidate.path.casefold() not in excluded
        )
        for candidate in candidates[:max_candidates]:
            path = Path(candidate.path)
            identity = self._reserve_candidate(candidate)
            if identity is None:
                continue
            try:
                _remove_candidate_tree(path, expected_identity=identity)
            except BaseException as error:
                self._complete_candidate(candidate, error=error)
                if not isinstance(error, OSError):
                    raise
                failed.append(candidate)
                continue
            deleted.append(candidate.path)
            self._complete_candidate(candidate, error=None)
        return UnmanagedArtifactCleanup(tuple(deleted), tuple(failed))

    def recover_reservations(self) -> UnmanagedArtifactCleanup:
        with self._cleanup_lock:
            self.product_staging.recover()
            self._recover_missing_fixture_leases()
            return self._recover_reservations()

    def _recover_missing_fixture_leases(self) -> None:
        with self.database.connect() as connection:
            rows = connection.execute(
                """SELECT lease_id, target_dir, owner_pid,
                          owner_process_creation_time
                   FROM artifact_fixture_leases
                   WHERE status='active'"""
            ).fetchall()
        missing = []
        for row in rows:
            if self._fixture_owner_matches(row):
                continue
            try:
                Path(str(row["target_dir"])).lstat()
            except FileNotFoundError:
                missing.append(row)
            except OSError:
                continue
        if not missing:
            return
        recovered_at = utc_text()
        with self.database.transaction() as connection:
            for row in missing:
                updated = connection.execute(
                    """UPDATE artifact_fixture_leases
                       SET status='recovered', released_at=?
                       WHERE lease_id=? AND status='active'
                         AND owner_pid=? AND owner_process_creation_time=?""",
                    (
                        recovered_at,
                        row["lease_id"],
                        row["owner_pid"],
                        row["owner_process_creation_time"],
                    ),
                ).rowcount
                if updated:
                    self._insert_fixture_event(
                        connection,
                        "artifact.fixture_recovered",
                        lease_id=str(row["lease_id"]),
                        path=str(row["target_dir"]),
                        owner_pid=int(row["owner_pid"]),
                    )

    def _recover_reservations(
        self, *, max_candidates: int | None = None
    ) -> UnmanagedArtifactCleanup:
        with self.database.connect() as connection:
            rows = connection.execute(
                """SELECT target_dir, filesystem_identity
                   FROM cleanup_reservations
                   WHERE reservation_kind='artifact'
                   ORDER BY reserved_at, target_key"""
            ).fetchall()
        if max_candidates is not None:
            rows = rows[:max_candidates]
        deleted: list[str] = []
        failed: list[UnmanagedArtifact] = []
        for row in rows:
            path = Path(str(row["target_dir"]))
            root = self._root_for(path)
            candidate = UnmanagedArtifact(root or "", str(path))
            if root is None:
                error = OSError("artifact cleanup reservation target is outside its managed root")
                failed.append(candidate)
                self._complete_candidate(candidate, error=error, recovered=True)
                continue
            try:
                path.lstat()
            except FileNotFoundError:
                deleted.append(str(path))
                self._complete_candidate(candidate, error=None, recovered=True)
                continue
            except OSError as error:
                failed.append(candidate)
                self._complete_candidate(candidate, error=error, recovered=True)
                continue
            if not self._is_contained_plain_directory(path):
                error = OSError("artifact cleanup reservation target is a filesystem reparse point")
                failed.append(candidate)
                self._complete_candidate(candidate, error=error, recovered=True)
                continue
            key = target_identity(path)
            with self.database.transaction() as connection:
                managed_overlap = self._managed_overlap_in_connection(
                    connection,
                    key,
                    ignored_artifact_reservation_key=key,
                )
            if managed_overlap:
                error = OSError("artifact cleanup reservation overlaps managed work")
                failed.append(candidate)
                self._complete_candidate(candidate, error=error, recovered=True)
                continue
            try:
                _remove_candidate_tree(
                    path, expected_identity=str(row["filesystem_identity"] or "")
                )
            except OSError as error:
                failed.append(candidate)
                self._complete_candidate(candidate, error=error, recovered=True)
                continue
            deleted.append(str(path))
            self._complete_candidate(candidate, error=None, recovered=True)
        return UnmanagedArtifactCleanup(tuple(deleted), tuple(failed))

    def _is_still_unmanaged(self, path: Path) -> bool:
        normalized = path.resolve()
        return any(candidate.path == str(normalized) for candidate in self.scan())

    def _is_safe_unmanaged_candidate(self, path: Path) -> bool:
        if _is_reparse_point(path) or not path.is_dir():
            return False
        try:
            normalized = path.resolve(strict=True)
        except OSError:
            return False
        if not any(normalized.is_relative_to(root) for root in self.roots):
            return False
        return self._is_still_unmanaged(normalized)

    def _reserve_candidate(self, candidate: UnmanagedArtifact) -> str | None:
        path = Path(candidate.path)
        if not self._is_safe_unmanaged_candidate(path):
            return None
        identity = filesystem_identity(path)
        key = target_identity(path)
        with self.database.transaction() as connection:
            reservation = connection.execute(
                "SELECT * FROM cleanup_reservations WHERE target_key=?", (key,)
            ).fetchone()
            if reservation is not None:
                if reservation["reservation_kind"] != "artifact":
                    return None
                if reservation["filesystem_identity"] != identity:
                    return None
                if self._managed_overlap_in_connection(
                    connection,
                    key,
                    ignored_artifact_reservation_key=key,
                ):
                    return None
            else:
                if self._managed_overlap_in_connection(connection, key):
                    return None
                connection.execute(
                    """INSERT INTO cleanup_reservations(
                           target_key, target_dir, reserved_at,
                           reservation_kind, filesystem_identity
                       ) VALUES (?, ?, ?, 'artifact', ?)""",
                    (key, str(path), utc_text(), identity),
                )
            self._insert_event(
                connection,
                "artifact.unmanaged_delete_started",
                candidate,
                {"filesystemIdentity": identity},
            )
        return identity

    def _managed_overlap_in_connection(
        self,
        connection,
        target_key: str,
        *,
        ignored_artifact_reservation_key: str | None = None,
    ) -> bool:
        managed_values: list[str] = []
        managed_values.extend(
            str(row["target_dir"])
            for row in connection.execute(
                """SELECT target_dir FROM cargo_jobs
                   WHERE cleanup_status <> 'deleted'
                      OR status IN ('leased', 'running')"""
            )
        )
        for row in connection.execute(
            """SELECT target_key, target_dir, reservation_kind
               FROM cleanup_reservations"""
        ):
            if (
                ignored_artifact_reservation_key is not None
                and row["reservation_kind"] == "artifact"
                and row["target_key"] == ignored_artifact_reservation_key
            ):
                continue
            managed_values.append(str(row["target_dir"]))
        for row in connection.execute(
            "SELECT job_root, target_root FROM validation_copies"
        ):
            managed_values.extend((str(row["job_root"]), str(row["target_root"])))
        managed_values.extend(
            str(row["storage_path"])
            for row in connection.execute(
                """SELECT storage_path FROM workflow_artifacts
                   WHERE storage_path IS NOT NULL"""
            )
        )
        for row in connection.execute(
            """SELECT target_dir, owner_pid, owner_process_creation_time
               FROM artifact_fixture_leases WHERE status='active'"""
        ):
            if self._fixture_owner_matches(row):
                managed_values.append(str(row["target_dir"]))
        for row in connection.execute(
            """SELECT staging_dir, final_dir, status
               FROM artifact_product_staging_leases
               WHERE status IN ('active', 'publishing', 'published')"""
        ):
            if row["status"] in {"active", "publishing"}:
                managed_values.append(str(row["staging_dir"]))
            if row["status"] in {"publishing", "published"}:
                managed_values.append(str(row["final_dir"]))
        return any(
            targets_overlap(target_key, target_identity(value)) for value in managed_values
        )

    def _complete_candidate(
        self,
        candidate: UnmanagedArtifact,
        *,
        error: BaseException | None,
        recovered: bool = False,
    ) -> None:
        key = target_identity(candidate.path)
        with self.database.transaction() as connection:
            if error is None:
                connection.execute(
                    """DELETE FROM cleanup_reservations
                       WHERE target_key=? AND reservation_kind='artifact'""",
                    (key,),
                )
                recovered_at = utc_text()
                for row in connection.execute(
                    """SELECT lease_id, target_dir FROM artifact_fixture_leases
                       WHERE status='active'"""
                ).fetchall():
                    if targets_overlap(key, target_identity(str(row["target_dir"]))):
                        connection.execute(
                            """UPDATE artifact_fixture_leases
                               SET status='recovered', released_at=?
                               WHERE lease_id=? AND status='active'""",
                            (recovered_at, row["lease_id"]),
                        )
                        self._insert_fixture_event(
                            connection,
                            "artifact.fixture_recovered",
                            lease_id=str(row["lease_id"]),
                            path=str(row["target_dir"]),
                            owner_pid=None,
                        )
            else:
                connection.execute(
                    """UPDATE cleanup_reservations
                       SET reserved_at=?
                       WHERE target_key=? AND reservation_kind='artifact'""",
                    (utc_text(), key),
                )
            self._insert_event(
                connection,
                (
                    "artifact.unmanaged_deleted"
                    if error is None
                    else "artifact.unmanaged_delete_failed"
                ),
                candidate,
                {"error": str(error) if error else None, "recovered": recovered},
            )

    def _root_for(self, path: Path) -> str | None:
        try:
            normalized = path.resolve(strict=False)
        except OSError:
            return None
        root = next(
            (
                root
                for root in self.roots
                if normalized != root and normalized.is_relative_to(root)
            ),
            None,
        )
        return str(root) if root is not None else None

    def _is_contained_plain_directory(self, path: Path) -> bool:
        if _is_reparse_point(path) or not path.is_dir():
            return False
        return self._root_for(path) is not None

    def _scan_children(
        self, root: Path, directory: Path, managed: tuple[Path, ...]
    ) -> list[UnmanagedArtifact]:
        candidates: list[UnmanagedArtifact] = []
        try:
            children = tuple(directory.iterdir())
        except OSError:
            return candidates
        for child in children:
            if _is_reparse_point(child) or not child.is_dir():
                continue
            resolved = child.resolve()
            if not resolved.is_relative_to(root):
                continue
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
                """SELECT target_dir FROM cleanup_reservations
                   WHERE reservation_kind='cargo'"""
            ):
                self._add_managed_path(paths, row["target_dir"])
            for row in connection.execute("SELECT job_root, target_root FROM validation_copies"):
                self._add_managed_path(paths, row["job_root"])
                self._add_managed_path(paths, row["target_root"])
            for row in connection.execute(
                "SELECT storage_path FROM workflow_artifacts WHERE storage_path IS NOT NULL"
            ):
                self._add_managed_path(paths, row["storage_path"])
            for row in connection.execute(
                """SELECT target_dir, owner_pid, owner_process_creation_time
                   FROM artifact_fixture_leases WHERE status='active'"""
            ):
                if self._fixture_owner_matches(row):
                    self._add_managed_path(paths, row["target_dir"])
            for path in self.product_staging.managed_paths():
                self._add_managed_path(paths, str(path))
        return tuple(paths)

    @staticmethod
    def _fixture_lease(row) -> ArtifactFixtureLease:
        return ArtifactFixtureLease(
            lease_id=str(row["lease_id"]),
            path=str(row["target_dir"]),
            prefix=str(row["prefix"]),
            owner_pid=int(row["owner_pid"]),
            owner_process_creation_time=str(row["owner_process_creation_time"]),
            status=str(row["status"]),
            created_at=str(row["created_at"]),
            released_at=(str(row["released_at"]) if row["released_at"] else None),
        )

    @staticmethod
    def _live_process_creation_time(pid: int) -> str | None:
        if not process_is_alive(pid):
            return None
        try:
            return process_creation_time(pid)
        except (OSError, ValueError):
            return None

    def _fixture_owner_matches(self, row) -> bool:
        identity = self._live_process_creation_time(int(row["owner_pid"]))
        return identity is not None and identity == str(
            row["owner_process_creation_time"]
        )

    @staticmethod
    def _insert_fixture_event(
        connection,
        event_type: str,
        *,
        lease_id: str,
        path: str,
        owner_pid: int | None,
    ) -> None:
        connection.execute(
            "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
            (
                event_type,
                json.dumps(
                    {"leaseId": lease_id, "path": path, "ownerPid": owner_pid},
                    sort_keys=True,
                ),
                utc_text(),
            ),
        )

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

    def _insert_event(
        self,
        connection,
        event_type: str,
        candidate: UnmanagedArtifact,
        extra: dict[str, object] | None = None,
    ) -> None:
        payload = {"path": candidate.path, "root": candidate.root}
        payload.update(extra or {})
        connection.execute(
            "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
            (event_type, json.dumps(payload, sort_keys=True), utc_text()),
        )


def _remove_candidate_tree(path: Path, *, expected_identity: str) -> None:
    remove_tree(path, expected_identity=expected_identity)


def _is_reparse_point(path: Path) -> bool:
    try:
        metadata = path.lstat()
    except OSError:
        return True
    reparse_flag = getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0)
    return path.is_symlink() or bool(
        reparse_flag and metadata.st_file_attributes & reparse_flag
    )


def _existing_reparse_point(path: Path) -> bool:
    try:
        path.lstat()
    except FileNotFoundError:
        return False
    except OSError as error:
        raise CoordinatorError(
            "artifact_governance_root_unavailable",
            f"Artifact governance root cannot be inspected: {path}",
        ) from error
    return _is_reparse_point(path)
