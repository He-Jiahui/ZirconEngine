from __future__ import annotations

import json
import os
import shutil
import sqlite3
import threading
import uuid
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Callable

from .cargo_jobs import (
    ACTIVE_CARGO_STATUSES,
    CargoCleanupPolicy,
    CargoCleanupStatus,
    CargoJobStatus,
    CargoJobService,
    target_identity,
    targets_overlap,
)
from .database import Database
from .models import CoordinatorError, utc_text
from .snapshots import ObjectStore


@dataclass(frozen=True, slots=True)
class CleanupDenial:
    path: str
    code: str
    message: str


@dataclass(frozen=True, slots=True)
class CleanupPlan:
    plan_id: str
    candidates: tuple[str, ...]
    denied: tuple[CleanupDenial, ...]
    generated_at: datetime
    free_bytes_by_root: tuple[tuple[str, int], ...]
    pressure_roots: tuple[str, ...]
    older_than_hours: int


@dataclass(frozen=True, slots=True)
class CleanupResult:
    deleted: tuple[str, ...]
    denied: tuple[CleanupDenial, ...]


@dataclass(frozen=True, slots=True)
class RetentionPlan:
    plan_id: str
    snapshot_ids: tuple[int, ...]
    object_hashes: tuple[str, ...]
    created_at: datetime

    def to_dict(self) -> dict[str, object]:
        return {
            "plan_id": self.plan_id,
            "snapshot_ids": list(self.snapshot_ids),
            "object_hashes": list(self.object_hashes),
            "created_at": self.created_at.isoformat(),
        }


@dataclass(frozen=True, slots=True)
class RetentionResult:
    plan_id: str
    deleted_snapshot_ids: tuple[int, ...]
    deleted_object_hashes: tuple[str, ...]

    def to_dict(self) -> dict[str, object]:
        return {
            "plan_id": self.plan_id,
            "deleted_snapshot_ids": list(self.deleted_snapshot_ids),
            "deleted_object_hashes": list(self.deleted_object_hashes),
        }


class RetentionService:
    """Plan and atomically retire expired snapshot/object references."""

    def __init__(
        self,
        database: Database,
        object_store: ObjectStore,
        *,
        completed_days: int = 14,
        archived_days: int = 30,
    ):
        self.database = database
        self.object_store = object_store
        self.completed_days = completed_days
        self.archived_days = archived_days

    def plan(self, *, now: datetime | None = None) -> RetentionPlan:
        current_time = now or datetime.now(UTC)
        snapshot_ids, object_hashes = self._candidates(current_time)
        identity = json.dumps(
            {"snapshot_ids": snapshot_ids, "object_hashes": object_hashes},
            sort_keys=True,
            separators=(",", ":"),
        )
        import hashlib

        plan_id = f"object-gc-{hashlib.sha256(identity.encode('utf-8')).hexdigest()[:24]}"
        plan = RetentionPlan(plan_id, snapshot_ids, object_hashes, current_time)
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO object_gc_plans(
                    plan_id, snapshot_ids_json, object_hashes_json, status, created_at
                ) VALUES (?, ?, ?, 'planned', ?)
                ON CONFLICT(plan_id) DO UPDATE SET
                    snapshot_ids_json = excluded.snapshot_ids_json,
                    object_hashes_json = excluded.object_hashes_json,
                    status = CASE
                        WHEN object_gc_plans.status = 'failed' THEN 'planned'
                        ELSE object_gc_plans.status
                    END,
                    created_at = CASE
                        WHEN object_gc_plans.status IN ('planned', 'failed') THEN excluded.created_at
                        ELSE object_gc_plans.created_at
                    END,
                    error_text = CASE
                        WHEN object_gc_plans.status = 'failed' THEN NULL
                        ELSE object_gc_plans.error_text
                    END
                """,
                (
                    plan.plan_id,
                    json.dumps(plan.snapshot_ids),
                    json.dumps(plan.object_hashes),
                    plan.created_at.isoformat(),
                ),
            )
        return plan

    def get_plan(self, plan_id: str) -> RetentionPlan:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM object_gc_plans WHERE plan_id = ?", (plan_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError("object_gc_plan_not_found", f"Unknown object GC plan {plan_id}")
        return RetentionPlan(
            row["plan_id"],
            tuple(int(item) for item in json.loads(row["snapshot_ids_json"])),
            tuple(str(item) for item in json.loads(row["object_hashes_json"])),
            datetime.fromisoformat(row["created_at"]),
        )

    def apply(self, plan: RetentionPlan, *, now: datetime | None = None) -> RetentionResult:
        current_time = now or datetime.now(UTC)
        trash_root = self.object_store.root.parent / "gc-trash" / plan.plan_id
        moved: list[tuple[Path, Path]] = []
        try:
            # One SQLite writer transaction serializes candidate revalidation,
            # object moves, snapshot/object deletion, and concurrent snapshot put.
            # A creator either commits before this recheck and keeps the object,
            # or waits and recreates the object after collection completes.
            with self.database.transaction() as connection:
                current_snapshots, current_objects = self._candidates_with_connection(
                    connection, current_time
                )
                if (
                    current_snapshots != plan.snapshot_ids
                    or current_objects != plan.object_hashes
                ):
                    raise CoordinatorError(
                        "object_gc_plan_stale",
                        "Object GC candidates changed after planning",
                    )
                row = connection.execute(
                    "SELECT * FROM object_gc_plans WHERE plan_id = ?", (plan.plan_id,)
                ).fetchone()
                if row is None:
                    raise CoordinatorError(
                        "object_gc_plan_not_found", f"Unknown object GC plan {plan.plan_id}"
                    )
                if row["status"] != "planned":
                    raise CoordinatorError(
                        "object_gc_plan_not_pending",
                        f"Object GC plan {plan.plan_id} is {row['status']}",
                    )
                if (
                    tuple(json.loads(row["snapshot_ids_json"])) != plan.snapshot_ids
                    or tuple(json.loads(row["object_hashes_json"])) != plan.object_hashes
                ):
                    raise CoordinatorError(
                        "object_gc_plan_tampered", "Object GC plan was modified"
                    )
                connection.execute(
                    "UPDATE object_gc_plans SET status = 'applying' WHERE plan_id = ?",
                    (plan.plan_id,),
                )
                for object_hash in plan.object_hashes:
                    source = self.object_store.path_for_hash(object_hash)
                    if not source.is_file():
                        raise CoordinatorError(
                            "object_gc_source_missing",
                            f"Object file disappeared: {object_hash}",
                        )
                    destination = trash_root / object_hash
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    os.replace(source, destination)
                    moved.append((source, destination))
                if plan.snapshot_ids:
                    placeholders = ",".join("?" for _ in plan.snapshot_ids)
                    connection.execute(
                        f"DELETE FROM snapshots WHERE snapshot_id IN ({placeholders})",
                        plan.snapshot_ids,
                    )
                if plan.object_hashes:
                    placeholders = ",".join("?" for _ in plan.object_hashes)
                    connection.execute(
                        f"DELETE FROM objects WHERE object_hash IN ({placeholders})",
                        plan.object_hashes,
                    )
                connection.execute(
                    """
                    UPDATE object_gc_plans
                    SET status = 'applied', applied_at = ?, error_text = NULL
                    WHERE plan_id = ?
                    """,
                    (utc_text(current_time), plan.plan_id),
                )
                connection.execute(
                    "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                    (
                        "retention.objects_collected",
                        json.dumps(
                            {
                                "plan_id": plan.plan_id,
                                "snapshot_count": len(plan.snapshot_ids),
                                "object_count": len(plan.object_hashes),
                            },
                            sort_keys=True,
                        ),
                        utc_text(current_time),
                    ),
                )
            shutil.rmtree(trash_root, ignore_errors=True)
        except BaseException as error:
            for source, destination in reversed(moved):
                if destination.exists() and not source.exists():
                    source.parent.mkdir(parents=True, exist_ok=True)
                    os.replace(destination, source)
            with self.database.transaction() as connection:
                connection.execute(
                    """
                    UPDATE object_gc_plans SET status = 'failed', error_text = ?
                    WHERE plan_id = ? AND status IN ('planned', 'applying')
                    """,
                    (str(error), plan.plan_id),
                )
            raise
        return RetentionResult(plan.plan_id, plan.snapshot_ids, plan.object_hashes)

    def recover_interrupted(self) -> tuple[str, ...]:
        """Restore pre-commit GC quarantine and discard post-commit residue."""
        trash_parent = self.object_store.root.parent / "gc-trash"
        if not trash_parent.is_dir():
            return ()
        recovered: list[str] = []
        for plan_root in sorted(trash_parent.iterdir(), key=lambda item: item.name):
            if not plan_root.is_dir():
                continue
            with self.database.connect() as connection:
                row = connection.execute(
                    "SELECT status FROM object_gc_plans WHERE plan_id = ?",
                    (plan_root.name,),
                ).fetchone()
            if row is None:
                continue
            if row["status"] == "applied":
                shutil.rmtree(plan_root, ignore_errors=True)
                continue
            if row["status"] not in {"planned", "applying", "failed"}:
                continue
            for quarantined in sorted(plan_root.iterdir(), key=lambda item: item.name):
                if not quarantined.is_file():
                    continue
                target = self.object_store.path_for_hash(quarantined.name)
                target.parent.mkdir(parents=True, exist_ok=True)
                if target.exists():
                    if target.read_bytes() != quarantined.read_bytes():
                        raise CoordinatorError(
                            "object_gc_recovery_conflict",
                            f"Object recovery target differs: {quarantined.name}",
                        )
                    quarantined.unlink()
                else:
                    os.replace(quarantined, target)
            shutil.rmtree(plan_root, ignore_errors=True)
            with self.database.transaction() as connection:
                connection.execute(
                    """
                    UPDATE object_gc_plans
                    SET status = 'failed', error_text = ?
                    WHERE plan_id = ? AND status IN ('planned', 'applying')
                    """,
                    ("recovered interrupted object GC quarantine", plan_root.name),
                )
            recovered.append(plan_root.name)
        return tuple(recovered)

    def _candidates(self, now: datetime) -> tuple[tuple[int, ...], tuple[str, ...]]:
        with self.database.connect() as connection:
            return self._candidates_with_connection(connection, now)

    def _candidates_with_connection(
        self, connection: sqlite3.Connection, now: datetime
    ) -> tuple[tuple[int, ...], tuple[str, ...]]:
        completed_cutoff = now - timedelta(days=self.completed_days)
        archived_cutoff = now - timedelta(days=self.archived_days)
        rows = connection.execute(
                """
                SELECT snapshots.snapshot_id, snapshots.manifest_json,
                       snapshots.created_at AS snapshot_created_at,
                       sessions.status, sessions.completed_at, sessions.archived_at
                FROM snapshots
                JOIN sessions ON sessions.session_id = snapshots.session_id
                ORDER BY snapshots.snapshot_id
                """
        ).fetchall()
        candidate_ids: list[int] = []
        retained_hashes: set[str] = set()
        for row in rows:
            manifest = json.loads(row["manifest_json"])
            hashes = {value for value in manifest.values() if value}
            status = row["status"]
            snapshot_created_at = datetime.fromisoformat(row["snapshot_created_at"])
            expired = False
            if status == "archived" and row["archived_at"]:
                expired = (
                    datetime.fromisoformat(row["archived_at"]) <= archived_cutoff
                    and snapshot_created_at <= archived_cutoff
                )
            elif status in {"completed", "cancelled"} and row["completed_at"]:
                expired = (
                    datetime.fromisoformat(row["completed_at"]) <= completed_cutoff
                    and snapshot_created_at <= completed_cutoff
                )
            if expired:
                candidate_ids.append(int(row["snapshot_id"]))
            else:
                retained_hashes.update(hashes)
        patch_rows = connection.execute(
                """
                SELECT patch_object_hash, base_objects_json, current_objects_json
                FROM patches
                """
        ).fetchall()
        for row in patch_rows:
            retained_hashes.add(row["patch_object_hash"])
            for column in ("base_objects_json", "current_objects_json"):
                if row[column]:
                    retained_hashes.update(
                        value for value in json.loads(row[column]).values() if value
                    )
        object_hashes = [
            row["object_hash"]
            for row in connection.execute(
                "SELECT object_hash FROM objects ORDER BY object_hash"
            ).fetchall()
            if row["object_hash"] not in retained_hashes
        ]
        return tuple(candidate_ids), tuple(object_hashes)


class CleanupService:
    def __init__(
        self,
        database: Database,
        cargo_jobs: CargoJobService,
        *,
        process_alive: Callable[[int], bool],
        free_space: Callable[[Path], int] | None = None,
        pressure_threshold_bytes: int = 50 * 1024**3,
    ):
        self.database = database
        self.cargo_jobs = cargo_jobs
        self.process_alive = process_alive
        self.free_space = free_space or (
            lambda path: shutil.disk_usage(path.anchor or path.parent).free
        )
        self.pressure_threshold_bytes = pressure_threshold_bytes
        self._async_cleanup_lock = threading.Lock()
        self._async_cleanup_requested = threading.Event()

    def recover_reservations(self) -> int:
        with self.database.transaction() as connection:
            count = int(
                connection.execute("SELECT COUNT(*) FROM cleanup_reservations").fetchone()[0]
            )
            if count:
                connection.execute("DELETE FROM cleanup_reservations")
                connection.execute(
                    "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                    (
                        "cleanup.reservations_recovered",
                        json.dumps({"count": count}, sort_keys=True),
                        utc_text(),
                    ),
                )
            connection.execute(
                "UPDATE cleanup_plans SET status = 'planned' WHERE status = 'applying'"
            )
        return count

    def cleanup_job_now(self, job_id: str) -> CleanupResult:
        """Delete a non-reusable lane immediately after ownership has ended."""
        job = self.cargo_jobs.get(job_id)
        if job.cleanup_policy is not CargoCleanupPolicy.DELETE_ON_RELEASE:
            return CleanupResult((), ())
        if job.status not in {
            CargoJobStatus.RELEASED,
            CargoJobStatus.ORPHANED,
        }:
            denial = CleanupDenial(
                job.target_dir,
                "cargo_job_active",
                f"Cargo job {job.job_id} is {job.status.value}",
            )
            return CleanupResult((), (denial,))
        target = self.cargo_jobs.target_policy.validate(job.target_dir)
        key = target_identity(target)
        with self.database.transaction() as connection:
            rows = connection.execute(
                "SELECT * FROM cargo_jobs WHERE status IN ('leased', 'running')"
            ).fetchall()
            active = [
                self.cargo_jobs._from_row(row)
                for row in rows
                if targets_overlap(key, row["target_key"])
            ]
            live = [item for item in active if item.pid and self.process_alive(item.pid)]
            if live or active:
                denial = CleanupDenial(
                    str(target),
                    "active_process" if live else "active_lease",
                    "Cargo lane became active before immediate cleanup",
                )
                return CleanupResult((), (denial,))
            if connection.execute(
                "SELECT 1 FROM cleanup_reservations WHERE target_key=?", (key,)
            ).fetchone() is not None:
                denial = CleanupDenial(
                    str(target),
                    "cleanup_already_reserved",
                    "Cargo lane is already reserved for cleanup",
                )
                return CleanupResult((), (denial,))
            connection.execute(
                "INSERT INTO cleanup_reservations(target_key, target_dir, reserved_at) VALUES (?, ?, ?)",
                (key, str(target), utc_text()),
            )
        error: OSError | None = None
        deleted: tuple[str, ...] = ()
        try:
            if target.exists():
                shutil.rmtree(target)
            deleted = (str(target),)
        except OSError as caught:
            error = caught
        finally:
            with self.database.transaction() as connection:
                connection.execute(
                    "DELETE FROM cleanup_reservations WHERE target_key=?", (key,)
                )
                connection.execute(
                    """
                    UPDATE cargo_jobs
                    SET cleanup_status=?, cleanup_error=?
                    WHERE job_id=?
                    """,
                    (
                        CargoCleanupStatus.DELETED.value
                        if error is None
                        else CargoCleanupStatus.FAILED.value,
                        str(error) if error else None,
                        job_id,
                    ),
                )
                connection.execute(
                    "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                    (
                        "cleanup.ephemeral_lane_deleted"
                        if error is None
                        else "cleanup.ephemeral_lane_failed",
                        json.dumps(
                            {"job_id": job_id, "target_dir": str(target), "error": str(error) if error else None},
                            sort_keys=True,
                        ),
                        utc_text(),
                    ),
                )
        denied = (
            (CleanupDenial(str(target), "cleanup_failed", str(error)),)
            if error is not None
            else ()
        )
        return CleanupResult(deleted, denied)

    def schedule_pending_cleanup(self) -> bool:
        """Start prompt cleanup after the command lock has been released."""
        self._async_cleanup_requested.set()
        if not self._async_cleanup_lock.acquire(blocking=False):
            return False

        def worker() -> None:
            try:
                while True:
                    self._async_cleanup_requested.clear()
                    self.retry_pending_jobs()
                    self.evict_idle_pools_under_pressure()
                    if not self._async_cleanup_requested.is_set():
                        break
            finally:
                self._async_cleanup_lock.release()
                # A release can arrive after the loop's final check but before the
                # worker drops its lock. Hand that request to a fresh worker.
                if self._async_cleanup_requested.is_set():
                    self.schedule_pending_cleanup()

        threading.Thread(
            target=worker,
            name="zircon-cargo-ephemeral-cleanup",
            daemon=True,
        ).start()
        return True

    def retry_pending_jobs(self) -> tuple[str, ...]:
        with self.database.connect() as connection:
            job_ids = tuple(
                row["job_id"]
                for row in connection.execute(
                    """
                    SELECT job_id FROM cargo_jobs
                    WHERE cleanup_policy='delete_on_release'
                      AND cleanup_status IN ('pending', 'failed')
                      AND status IN ('released', 'orphaned')
                    ORDER BY released_at, finished_at, created_at
                    """
                )
            )
        deleted: list[str] = []
        for job_id in job_ids:
            result = self.cleanup_job_now(job_id)
            if result.deleted:
                deleted.append(job_id)
        return tuple(deleted)

    def evict_idle_pools_under_pressure(self) -> CleanupResult:
        """Evict the least-recently-used idle reusable pools until each root recovers."""
        deleted: list[str] = []
        denied: list[CleanupDenial] = []
        with self.database.connect() as connection:
            rows = connection.execute(
                """
                SELECT * FROM cargo_jobs
                WHERE reuse_key IS NOT NULL AND cleanup_status='retained'
                ORDER BY COALESCE(released_at, finished_at, last_heartbeat_at), created_at
                """
            ).fetchall()
        jobs = [self.cargo_jobs._from_row(row) for row in rows]

        for root in self.cargo_jobs.target_policy.roots:
            available_bytes = self.free_space(root)
            if available_bytes > self.pressure_threshold_bytes:
                continue
            groups: dict[str, list] = {}
            for job in jobs:
                target = Path(job.target_dir)
                if target == root or not target.is_relative_to(root):
                    continue
                groups.setdefault(target_identity(target), []).append(job)

            idle_groups: list[tuple[datetime, str, list]] = []
            for target_key, target_jobs in groups.items():
                target_text = target_jobs[-1].target_dir
                live = [
                    job for job in target_jobs if job.pid and self.process_alive(job.pid)
                ]
                if live:
                    denied.append(
                        CleanupDenial(
                            target_text,
                            "active_process",
                            f"PID {live[0].pid} is still alive",
                        )
                    )
                    continue
                active = [
                    job for job in target_jobs if job.status.value in ACTIVE_CARGO_STATUSES
                ]
                if active:
                    denied.append(
                        CleanupDenial(
                            target_text,
                            "active_lease",
                            f"Cargo pool is owned by job {active[0].job_id}",
                        )
                    )
                    continue
                reference_time = max(
                    job.released_at or job.finished_at or job.last_heartbeat_at
                    for job in target_jobs
                )
                idle_groups.append((reference_time, target_key, target_jobs))

            for _reference_time, target_key, target_jobs in sorted(
                idle_groups, key=lambda item: (item[0], item[1])
            ):
                if available_bytes > self.pressure_threshold_bytes:
                    break
                target = self.cargo_jobs.target_policy.validate(target_jobs[-1].target_dir)
                with self.database.transaction() as connection:
                    current_rows = connection.execute(
                        """
                        SELECT job_id, pid, status FROM cargo_jobs
                        WHERE target_key=?
                        """,
                        (target_key,),
                    ).fetchall()
                    live = next(
                        (
                            row
                            for row in current_rows
                            if row["pid"] and self.process_alive(int(row["pid"]))
                        ),
                        None,
                    )
                    if live is not None:
                        denied.append(
                            CleanupDenial(
                                str(target),
                                "active_process",
                                f"PID {live['pid']} became active before cleanup",
                            )
                        )
                        continue
                    active = next(
                        (
                            row
                            for row in current_rows
                            if row["status"] in ACTIVE_CARGO_STATUSES
                        ),
                        None,
                    )
                    if active is not None:
                        denied.append(
                            CleanupDenial(
                                str(target),
                                "active_lease",
                                f"Cargo pool became owned by job {active['job_id']}",
                            )
                        )
                        continue
                    reservation = connection.execute(
                        "SELECT 1 FROM cleanup_reservations WHERE target_key=?",
                        (target_key,),
                    ).fetchone()
                    if reservation is not None:
                        denied.append(
                            CleanupDenial(
                                str(target),
                                "cleanup_already_reserved",
                                "Cargo pool is already reserved for cleanup",
                            )
                        )
                        continue
                    connection.execute(
                        "INSERT INTO cleanup_reservations(target_key, target_dir, reserved_at) "
                        "VALUES (?, ?, ?)",
                        (target_key, str(target), utc_text()),
                    )
                error: OSError | None = None
                try:
                    if target.exists():
                        shutil.rmtree(target)
                except OSError as caught:
                    error = caught
                finally:
                    with self.database.transaction() as connection:
                        connection.execute(
                            "DELETE FROM cleanup_reservations WHERE target_key=?",
                            (target_key,),
                        )
                        connection.execute(
                            """
                            UPDATE cargo_jobs
                            SET cleanup_status=?, cleanup_error=?
                            WHERE target_key=?
                            """,
                            (
                                CargoCleanupStatus.DELETED.value
                                if error is None
                                else CargoCleanupStatus.FAILED.value,
                                str(error) if error else None,
                                target_key,
                            ),
                        )
                if error is None:
                    deleted.append(str(target))
                else:
                    denied.append(CleanupDenial(str(target), "cleanup_failed", str(error)))
                available_bytes = self.free_space(root)
        return CleanupResult(tuple(deleted), tuple(denied))

    def plan(
        self,
        *,
        now: datetime | None = None,
        older_than_hours: int = 2,
    ) -> CleanupPlan:
        self._validate_retention(older_than_hours)
        current_time = now or datetime.now(UTC)
        cutoff = current_time - timedelta(hours=older_than_hours)
        free_bytes_by_root = tuple(
            (str(root), int(self.free_space(root)))
            for root in self.cargo_jobs.target_policy.roots
        )
        pressure_roots = tuple(
            root
            for root, free_bytes in free_bytes_by_root
            if free_bytes <= self.pressure_threshold_bytes
        )
        candidates: list[str] = []
        denied: list[CleanupDenial] = []
        jobs_by_target: dict[str, list] = {}
        for job in self.cargo_jobs.list():
            jobs_by_target.setdefault(target_identity(job.target_dir), []).append(job)
        for jobs in jobs_by_target.values():
            target = jobs[-1].target_dir
            if all(
                job.cleanup_policy is CargoCleanupPolicy.DELETE_ON_RELEASE
                for job in jobs
            ):
                continue
            try:
                self.cargo_jobs.target_policy.validate(target)
            except CoordinatorError as error:
                denied.append(CleanupDenial(target, error.code, error.message))
                continue
            if not Path(target).exists():
                continue
            live = [job for job in jobs if job.pid and self.process_alive(job.pid)]
            if live:
                denied.append(
                    CleanupDenial(
                        target,
                        "active_process",
                        f"PID {live[0].pid} is still alive",
                    )
                )
                continue
            active = [job for job in jobs if job.status.value in ACTIVE_CARGO_STATUSES]
            if active:
                denied.append(
                    CleanupDenial(
                        target,
                        "active_lease",
                        f"Cargo job {active[0].job_id} is {active[0].status.value}",
                    )
                )
                continue
            reference_time = max(
                job.released_at or job.finished_at or job.last_heartbeat_at
                for job in jobs
            )
            if reference_time > cutoff:
                denied.append(
                    CleanupDenial(target, "retention_active", "Cargo lane is inside retention window")
                )
                continue
            candidates.append(target)
        plan = CleanupPlan(
            plan_id=uuid.uuid4().hex,
            candidates=tuple(dict.fromkeys(candidates)),
            denied=tuple(denied),
            generated_at=current_time,
            free_bytes_by_root=free_bytes_by_root,
            pressure_roots=pressure_roots,
            older_than_hours=older_than_hours,
        )
        with self.database.transaction() as connection:
            connection.execute(
                "DELETE FROM cleanup_plans WHERE status != 'applying' AND generated_at < ?",
                ((current_time - timedelta(days=7)).isoformat(),),
            )
            connection.execute(
                """
                INSERT INTO cleanup_plans(
                    plan_id, generated_at, older_than_hours, candidates_json, status
                ) VALUES (?, ?, ?, ?, 'planned')
                """,
                (
                    plan.plan_id,
                    plan.generated_at.isoformat(),
                    plan.older_than_hours,
                    json.dumps(plan.candidates),
                ),
            )
        return plan

    def get_plan(self, plan_id: str) -> CleanupPlan:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM cleanup_plans WHERE plan_id = ?", (plan_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError("cleanup_plan_not_found", f"Unknown cleanup plan {plan_id}")
        return CleanupPlan(
            plan_id=row["plan_id"],
            candidates=tuple(json.loads(row["candidates_json"])),
            denied=(),
            generated_at=datetime.fromisoformat(row["generated_at"]),
            free_bytes_by_root=(),
            pressure_roots=(),
            older_than_hours=int(row["older_than_hours"]),
        )

    def apply(
        self,
        plan: CleanupPlan,
        *,
        now: datetime | None = None,
        max_plan_age_minutes: int = 30,
    ) -> CleanupResult:
        current_time = now or datetime.now(UTC)
        with self.database.transaction() as connection:
            stored = connection.execute(
                "SELECT * FROM cleanup_plans WHERE plan_id = ?", (plan.plan_id,)
            ).fetchone()
            if stored is None:
                raise CoordinatorError(
                    "cleanup_plan_not_found", f"Unknown cleanup plan {plan.plan_id}"
                )
            if stored["status"] != "planned":
                raise CoordinatorError(
                    "cleanup_plan_not_pending",
                    f"Cleanup plan {plan.plan_id} is {stored['status']}",
                )
            stored_candidates = tuple(json.loads(stored["candidates_json"]))
            if (
                stored_candidates != plan.candidates
                or int(stored["older_than_hours"]) != plan.older_than_hours
            ):
                raise CoordinatorError(
                    "cleanup_plan_tampered", "Cleanup candidate snapshot does not match stored plan"
                )
            generated_at = datetime.fromisoformat(stored["generated_at"])
            if current_time - generated_at > timedelta(minutes=max_plan_age_minutes):
                raise CoordinatorError(
                    "cleanup_plan_expired", f"Cleanup plan {plan.plan_id} has expired"
                )
            connection.execute(
                "UPDATE cleanup_plans SET status = 'applying' WHERE plan_id = ?",
                (plan.plan_id,),
            )
        cutoff = current_time - timedelta(hours=plan.older_than_hours)
        deleted: list[str] = []
        denied: list[CleanupDenial] = []
        for target_text in plan.candidates:
            try:
                target = self.cargo_jobs.target_policy.validate(target_text)
            except CoordinatorError as error:
                denied.append(CleanupDenial(target_text, error.code, error.message))
                continue
            key = target_identity(target)
            reserved = False
            with self.database.transaction() as connection:
                rows = connection.execute(
                    "SELECT * FROM cargo_jobs",
                ).fetchall()
                all_jobs = [self.cargo_jobs._from_row(row) for row in rows]
                matching_jobs = [
                    job for job in all_jobs if target_identity(job.target_dir) == key
                ]
                if not matching_jobs:
                    denied.append(
                        CleanupDenial(
                            target_text,
                            "untracked_target",
                            "Cargo lane has no coordinator job history",
                        )
                    )
                    continue
                active = [
                    job
                    for job in all_jobs
                    if job.status.value in ACTIVE_CARGO_STATUSES
                    and targets_overlap(key, target_identity(job.target_dir))
                ]
                live = [
                    job
                    for job in all_jobs
                    if job.pid
                    and targets_overlap(key, target_identity(job.target_dir))
                    and self.process_alive(job.pid)
                ]
                if live:
                    denied.append(
                        CleanupDenial(
                            target_text,
                            "active_process",
                            "Process became active after cleanup planning",
                        )
                    )
                    continue
                if active:
                    denied.append(
                        CleanupDenial(
                            target_text,
                            "active_lease",
                            "Lease became active after cleanup planning",
                        )
                    )
                    continue
                newest_reference = max(
                    (
                        job.released_at or job.finished_at or job.last_heartbeat_at
                        for job in matching_jobs
                    ),
                    default=None,
                )
                if newest_reference is not None and newest_reference > cutoff:
                    denied.append(
                        CleanupDenial(
                            target_text,
                            "retention_active",
                            "Cargo lane re-entered the retention window after cleanup planning",
                        )
                    )
                    continue
                existing_reservation = connection.execute(
                    "SELECT target_dir FROM cleanup_reservations WHERE target_key = ?",
                    (key,),
                ).fetchone()
                if existing_reservation is not None:
                    denied.append(
                        CleanupDenial(
                            target_text,
                            "cleanup_already_reserved",
                            "Cargo lane is already reserved by another cleanup",
                        )
                    )
                    continue
                connection.execute(
                    "INSERT INTO cleanup_reservations(target_key, target_dir, reserved_at) VALUES (?, ?, ?)",
                    (key, str(target), utc_text()),
                )
                reserved = True
            if not reserved:
                continue
            deletion_error: OSError | None = None
            try:
                if not target.exists():
                    denied.append(
                        CleanupDenial(target_text, "target_missing", "Cargo lane no longer exists")
                    )
                else:
                    shutil.rmtree(target)
                    deleted.append(target_text)
            except OSError as error:
                deletion_error = error
                denied.append(CleanupDenial(target_text, "cleanup_failed", str(error)))
            finally:
                with self.database.transaction() as connection:
                    connection.execute(
                        "DELETE FROM cleanup_reservations WHERE target_key = ?",
                        (key,),
                    )
                    if deletion_error is None and target_text in deleted:
                        connection.execute(
                            """
                            UPDATE cargo_jobs SET cleanup_status='deleted', cleanup_error=NULL
                            WHERE target_key=?
                            """,
                            (key,),
                        )
                    connection.execute(
                        "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                        (
                            "cleanup.cargo_lane_deleted"
                            if deletion_error is None and target_text in deleted
                            else "cleanup.cargo_lane_retained",
                            json.dumps(
                                {
                                    "target_dir": target_text,
                                    "error": str(deletion_error) if deletion_error else None,
                                },
                                sort_keys=True,
                            ),
                            utc_text(),
                        ),
                    )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cleanup_plans SET status = 'applied', applied_at = ? WHERE plan_id = ?",
                (utc_text(), plan.plan_id),
            )
        return CleanupResult(tuple(deleted), tuple(denied))

    @staticmethod
    def _validate_retention(older_than_hours: int) -> None:
        if older_than_hours <= 0:
            raise ValueError("older_than_hours must be positive")
