from __future__ import annotations

import json
import shutil
import uuid
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Callable

from .cargo_jobs import (
    ACTIVE_CARGO_STATUSES,
    CargoJobService,
    target_identity,
    targets_overlap,
)
from .database import Database
from .models import CoordinatorError, utc_text


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
