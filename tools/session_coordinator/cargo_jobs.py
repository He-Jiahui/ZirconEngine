from __future__ import annotations

import hashlib
import json
import shutil
import uuid
from dataclasses import dataclass
from datetime import datetime, timedelta
from enum import StrEnum
from pathlib import Path
from typing import Callable

from .cargo_reservations import (
    expire_invalid_pending_cpu_reservations,
    reconcile_terminal_finished_cpu_reservations,
    require_executable_cargo_session,
)
from .database import Database
from .models import CoordinatorError, parse_utc, utc_now, utc_text
from .processes import (
    live_cargo_process_tree_pids,
    live_process_tree_pids,
    process_creation_time as read_process_creation_time,
    process_is_alive,
)


class CargoLaneKind(StrEnum):
    CHECK = "check"
    TEST = "test"
    WORKSPACE = "workspace"
    GPU = "gpu"


class CargoJobStatus(StrEnum):
    LEASED = "leased"
    RUNNING = "running"
    SUCCEEDED = "succeeded"
    FAILED = "failed"
    RELEASED = "released"
    ORPHANED = "orphaned"


class CargoProcessRootKind(StrEnum):
    CARGO = "cargo"
    SUPERVISOR = "supervisor"


class CargoCleanupPolicy(StrEnum):
    RETAINED = "retained"
    DELETE_ON_RELEASE = "delete_on_release"


class CargoCleanupStatus(StrEnum):
    RETAINED = "retained"
    PENDING = "pending"
    DELETED = "deleted"
    FAILED = "failed"


ACTIVE_CARGO_STATUSES = (CargoJobStatus.LEASED.value, CargoJobStatus.RUNNING.value)
ALLOWED_TARGET_ROOT_NAMES = frozenset({"cargo-targets", "targets", "zirconbuilds"})


def target_identity(value: str | Path) -> str:
    return str(value).replace("/", "\\").casefold()


def targets_overlap(left: str, right: str) -> bool:
    left_key = left.rstrip("\\")
    right_key = right.rstrip("\\")
    return (
        left_key == right_key
        or left_key.startswith(right_key + "\\")
        or right_key.startswith(left_key + "\\")
    )


def overlapping_cleanup_reservation(connection, target_key: str):
    """Return the first cleanup reservation whose tree intersects the target."""
    rows = connection.execute(
        "SELECT target_key, target_dir FROM cleanup_reservations ORDER BY reserved_at"
    ).fetchall()
    return next(
        (row for row in rows if targets_overlap(target_key, row["target_key"])),
        None,
    )


@dataclass(frozen=True, slots=True)
class CargoCompatibility:
    platform: str
    toolchain: str
    target_architecture: str
    workspace: str
    build_config: str

    def canonical(self) -> dict[str, str]:
        values = {
            "platform": self.platform.strip().casefold(),
            "toolchain": self.toolchain.strip(),
            "target_architecture": self.target_architecture.strip().casefold(),
            "workspace": self.workspace.strip().replace("\\", "/"),
            "build_config": self.build_config.strip(),
        }
        if values["platform"] not in {"windows", "wsl"}:
            raise CoordinatorError(
                "invalid_cargo_compatibility",
                "Cargo compatibility platform must be windows or wsl",
            )
        for name, value in values.items():
            if not value or len(value) > 4096 or any(char in value for char in ("\0", "\r", "\n")):
                raise CoordinatorError(
                    "invalid_cargo_compatibility",
                    f"Cargo compatibility field {name} is empty or invalid",
                )
        workspace_parts = tuple(part for part in values["workspace"].split("/") if part)
        if (
            not workspace_parts
            or values["workspace"].startswith("/")
            or ":" in workspace_parts[0]
            or any(part in {".", ".."} for part in workspace_parts)
        ):
            raise CoordinatorError(
                "invalid_cargo_compatibility",
                "Cargo compatibility workspace must be a repository-relative path",
            )
        values["workspace"] = "/".join(workspace_parts)
        return values


@dataclass(frozen=True, slots=True)
class CargoJob:
    job_id: str
    session_id: str
    lane_kind: CargoLaneKind
    target_dir: str
    status: CargoJobStatus
    dry_run: bool
    pid: int | None
    command: tuple[str, ...]
    exit_code: int | None
    created_at: datetime
    last_heartbeat_at: datetime
    started_at: datetime | None
    finished_at: datetime | None
    released_at: datetime | None
    reuse_key: str | None
    compatibility_json: str | None
    compatibility_key: str | None
    reuse_profile: str | None
    cleanup_policy: CargoCleanupPolicy
    cleanup_status: CargoCleanupStatus
    reused_from_job_id: str | None
    cleanup_error: str | None
    process_tree_observed_at: datetime | None
    live_process_pids: tuple[int, ...]
    process_tree_exited_at: datetime | None
    root_process_creation_time: str | None
    root_process_kind: CargoProcessRootKind

    def to_dict(self) -> dict[str, object]:
        return {
            "job_id": self.job_id,
            "session_id": self.session_id,
            "lane_kind": self.lane_kind.value,
            "target_dir": self.target_dir,
            "status": self.status.value,
            "dry_run": self.dry_run,
            "pid": self.pid,
            "root_process_creation_time": self.root_process_creation_time,
            "root_process_kind": self.root_process_kind.value,
            "command": list(self.command),
            "exit_code": self.exit_code,
            "created_at": self.created_at.isoformat(),
            "last_heartbeat_at": self.last_heartbeat_at.isoformat(),
            "started_at": self.started_at.isoformat() if self.started_at else None,
            "finished_at": self.finished_at.isoformat() if self.finished_at else None,
            "released_at": self.released_at.isoformat() if self.released_at else None,
            "reuse_key": self.reuse_key,
            "compatibility": (
                json.loads(self.compatibility_json) if self.compatibility_json else None
            ),
            "compatibility_key": self.compatibility_key,
            "reuse_profile": self.reuse_profile,
            "cleanup_policy": self.cleanup_policy.value,
            "cleanup_status": self.cleanup_status.value,
            "reused_from_job_id": self.reused_from_job_id,
            "cleanup_error": self.cleanup_error,
            "process_tree_observed_at": (
                self.process_tree_observed_at.isoformat()
                if self.process_tree_observed_at
                else None
            ),
            "live_process_pids": list(self.live_process_pids),
            "process_tree_exited_at": (
                self.process_tree_exited_at.isoformat()
                if self.process_tree_exited_at
                else None
            ),
        }


class TargetPathPolicy:
    def __init__(self, roots: list[str | Path] | tuple[str | Path, ...]):
        resolved: list[Path] = []
        for value in roots:
            root = Path(value).resolve()
            if root.name.casefold() not in ALLOWED_TARGET_ROOT_NAMES:
                raise CoordinatorError(
                    "invalid_target_root",
                    "Managed target root must end in cargo-targets, targets, or ZirconBuilds: "
                    f"{root}",
                )
            resolved.append(root)
        if not resolved:
            raise CoordinatorError("target_root_unavailable", "No managed target root is configured")
        self.roots = tuple(dict.fromkeys(resolved))

    def validate(self, value: str | Path) -> Path:
        candidate = Path(value).resolve()
        for root in self.roots:
            if candidate != root and candidate.is_relative_to(root):
                return candidate
        raise CoordinatorError(
            "cargo_target_not_managed",
            "Cargo target must be below a configured D/E/F cargo-targets, targets, "
            f"or ZirconBuilds root: {candidate}",
        )

    def choose_root(self, free_space: Callable[[Path], int]) -> Path:
        available = [
            root
            for root in self.roots
            if root.parent.exists() or (root.anchor and Path(root.anchor).exists())
        ]
        if not available:
            raise CoordinatorError("target_root_unavailable", "No configured target drive is available")
        return max(available, key=free_space)


class CargoJobService:
    def __init__(
        self,
        database: Database,
        target_policy: TargetPathPolicy,
        *,
        repo_root: str | Path | None = None,
        free_space: Callable[[Path], int] | None = None,
        process_alive: Callable[[int], bool] | None = None,
        process_tree_pids: Callable[[int], tuple[int, ...]] | None = None,
        supervisor_cargo_pids: Callable[[int], tuple[int, ...]] | None = None,
        process_creation_time: Callable[[int], str] | None = None,
    ):
        self.database = database
        self.target_policy = target_policy
        self.repo_root = Path(repo_root).resolve() if repo_root is not None else Path.cwd().resolve()
        self.free_space = free_space or (
            lambda path: shutil.disk_usage(path.anchor or path.parent).free
        )
        self.process_alive = process_alive or process_is_alive
        self.process_tree_pids = process_tree_pids or (
            live_process_tree_pids
            if process_alive is None
            else lambda pid: (pid,) if self.process_alive(pid) else ()
        )
        self.supervisor_cargo_pids = supervisor_cargo_pids or (
            live_cargo_process_tree_pids
            if process_alive is None
            else self.process_tree_pids
        )
        self.process_creation_time = process_creation_time or read_process_creation_time

    def reserve_cpu(
        self,
        session_id: str,
        *,
        compatibility: CargoCompatibility,
        command: list[str] | tuple[str, ...],
        ttl_seconds: int = 900,
    ) -> dict[str, object]:
        """Reserve the next CPU Cargo lane for one exact managed command."""
        if not 30 <= ttl_seconds <= 3600:
            raise CoordinatorError(
                "cargo_reservation_ttl_invalid",
                "CPU lane reservation TTL must be between 30 and 3600 seconds",
            )
        command_tuple = tuple(str(part) for part in command if str(part))
        if not command_tuple:
            raise CoordinatorError("cargo_reservation_command_empty", "Reservation command cannot be empty")
        canonical = compatibility.canonical()
        compatibility_json = json.dumps(canonical, sort_keys=True, separators=(",", ":"))
        compatibility_key = self._compatibility_fingerprint(compatibility_json)
        command_fingerprint = self._command_fingerprint(command_tuple)
        now = utc_now()
        now_text = utc_text(now)
        expires_at = utc_text(now + timedelta(seconds=ttl_seconds))
        reservation_id = uuid.uuid4().hex
        with self.database.transaction() as connection:
            require_executable_cargo_session(connection, session_id)
            expire_invalid_pending_cpu_reservations(connection, now=now_text)
            reconcile_terminal_finished_cpu_reservations(connection, now=now_text)
            existing = connection.execute(
                """
                SELECT * FROM cargo_lane_reservations
                WHERE lane_scope='cpu' AND status IN ('pending', 'leased', 'running', 'finished')
                ORDER BY created_at LIMIT 1
                """
            ).fetchone()
            if existing is not None:
                if (
                    existing["session_id"] == session_id
                    and existing["compatibility_key"] == compatibility_key
                    and existing["command_fingerprint"] == command_fingerprint
                ):
                    return self._reservation_dict(existing)
                raise CoordinatorError(
                    "cargo_cpu_lane_reserved",
                    "The next managed CPU lane is reserved for another exact job",
                    details={"sessionId": existing["session_id"], "reservationId": existing["reservation_id"]},
                )
            connection.execute(
                """
                INSERT INTO cargo_lane_reservations(
                    reservation_id, session_id, lane_scope, compatibility_key,
                    compatibility_json, command_fingerprint, status, created_at, expires_at
                ) VALUES (?, ?, 'cpu', ?, ?, ?, 'pending', ?, ?)
                """,
                (
                    reservation_id,
                    session_id,
                    compatibility_key,
                    compatibility_json,
                    command_fingerprint,
                    now_text,
                    expires_at,
                ),
            )
            row = connection.execute(
                "SELECT * FROM cargo_lane_reservations WHERE reservation_id=?", (reservation_id,)
            ).fetchone()
        return self._reservation_dict(row)

    def release_cpu_reservation(
        self, reservation_id: str, *, session_id: str
    ) -> dict[str, object]:
        """Release a completed or unstarted CPU reservation for the next queued Session."""
        now = utc_text()
        with self.database.transaction() as connection:
            expire_invalid_pending_cpu_reservations(connection, now=now)
            row = connection.execute(
                "SELECT * FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation_id,),
            ).fetchone()
            if row is None:
                raise CoordinatorError(
                    "cargo_cpu_reservation_not_found",
                    f"Unknown CPU reservation {reservation_id}",
                )
            if row["session_id"] != session_id:
                raise CoordinatorError(
                    "cargo_cpu_reservation_owner_mismatch",
                    f"CPU reservation {reservation_id} belongs to Session {row['session_id']}",
                )
            if row["status"] == "running":
                raise CoordinatorError(
                    "cargo_cpu_reservation_running",
                    "A running CPU reservation cannot be handed off",
                    details={"reservationId": reservation_id, "jobId": row["job_id"]},
                )
            if row["status"] in {"released", "expired"}:
                return self._reservation_dict(row)
            connection.execute(
                """
                UPDATE cargo_lane_reservations
                SET status='released', completed_at=COALESCE(completed_at, ?)
                WHERE reservation_id=?
                """,
                (now, reservation_id),
            )
            row = connection.execute(
                "SELECT * FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation_id,),
            ).fetchone()
        return self._reservation_dict(row)

    def renew_cpu_reservation(
        self,
        reservation_id: str,
        *,
        session_id: str,
        ttl_seconds: int = 900,
    ) -> dict[str, object]:
        """Extend one pending CPU reservation without changing its FIFO identity."""
        if not 30 <= ttl_seconds <= 3600:
            raise CoordinatorError(
                "cargo_reservation_ttl_invalid",
                "CPU lane reservation TTL must be between 30 and 3600 seconds",
            )
        now = utc_now()
        now_text = utc_text(now)
        expires_at = utc_text(now + timedelta(seconds=ttl_seconds))
        with self.database.transaction() as connection:
            require_executable_cargo_session(connection, session_id)
            expire_invalid_pending_cpu_reservations(connection, now=now_text)
            row = connection.execute(
                "SELECT * FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation_id,),
            ).fetchone()
            if row is None:
                raise CoordinatorError(
                    "cargo_cpu_reservation_not_found",
                    f"Unknown CPU reservation {reservation_id}",
                )
            if row["session_id"] != session_id:
                raise CoordinatorError(
                    "cargo_cpu_reservation_owner_mismatch",
                    f"CPU reservation {reservation_id} belongs to Session {row['session_id']}",
                )
            if row["status"] != "pending":
                raise CoordinatorError(
                    "cargo_cpu_reservation_not_pending",
                    "Only a pending CPU reservation can be renewed",
                    details={"reservationId": reservation_id, "status": row["status"]},
                )
            connection.execute(
                "UPDATE cargo_lane_reservations SET expires_at=? WHERE reservation_id=?",
                (expires_at, reservation_id),
            )
            row = connection.execute(
                "SELECT * FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation_id,),
            ).fetchone()
        return self._reservation_dict(row)

    def acquire(
        self,
        session_id: str,
        lane_kind: CargoLaneKind,
        *,
        requested_target: str | Path | None = None,
        dry_run: bool = False,
        owner_pid: int | None = None,
        ephemeral: bool = False,
        compatibility: CargoCompatibility | None = None,
    ) -> CargoJob:
        if not isinstance(lane_kind, CargoLaneKind):
            raise ValueError("lane_kind must be a CargoLaneKind")
        if ephemeral and compatibility is not None:
            raise CoordinatorError(
                "cargo_compatibility_conflict",
                "An ephemeral Cargo job cannot also request a reusable compatibility pool",
            )
        job_id = uuid.uuid4().hex
        canonical_compatibility = (
            compatibility.canonical() if compatibility is not None else None
        )
        compatibility_json = (
            json.dumps(canonical_compatibility, sort_keys=True, separators=(",", ":"))
            if canonical_compatibility is not None
            else None
        )
        reuse_key = (
            self._compatibility_fingerprint(compatibility_json)
            if compatibility_json is not None
            else None
        )
        ephemeral = ephemeral or compatibility_json is None
        cleanup_policy = (
            CargoCleanupPolicy.DELETE_ON_RELEASE if ephemeral else CargoCleanupPolicy.RETAINED
        )
        requested = self.target_policy.validate(requested_target) if requested_target else None
        target: Path | None = requested
        reused_from_job_id: str | None = None
        now = utc_text()
        with self.database.transaction() as connection:
            require_executable_cargo_session(connection, session_id)
            self._require_gpu_reservation(connection, session_id, lane_kind)
            self._require_gpu_lane_available(connection, lane_kind)
            cpu_reservation = self._require_cpu_reservation(
                connection,
                session_id=session_id,
                lane_kind=lane_kind,
                compatibility_key=reuse_key,
                now=now,
            )
            if reuse_key is not None:
                active_pool = connection.execute(
                    """
                    SELECT job_id, target_dir FROM cargo_jobs
                    WHERE reuse_key=? AND status IN ('leased', 'running')
                    ORDER BY created_at DESC LIMIT 1
                    """,
                    (reuse_key,),
                ).fetchone()
                if active_pool is not None:
                    raise CoordinatorError(
                        "cargo_reuse_pool_busy",
                        f"Compatible Cargo pool is owned by job {active_pool['job_id']}",
                    )
                reusable_rows = connection.execute(
                    """
                    SELECT job_id, target_dir FROM cargo_jobs
                    WHERE reuse_key=? AND cleanup_status='retained'
                      AND status IN ('released', 'orphaned')
                    ORDER BY released_at DESC, created_at DESC
                    """,
                    (reuse_key,),
                ).fetchall()
                reusable = next(
                    (
                        row
                        for row in reusable_rows
                        if self.target_policy.validate(row["target_dir"]).is_dir()
                    ),
                    None,
                )
                missing_job_ids = [
                    row["job_id"]
                    for row in reusable_rows
                    if not self.target_policy.validate(row["target_dir"]).is_dir()
                ]
                if missing_job_ids:
                    connection.executemany(
                        """
                        UPDATE cargo_jobs
                        SET cleanup_status='deleted', cleanup_error=NULL
                        WHERE job_id=? AND cleanup_status='retained'
                        """,
                        ((missing_job_id,) for missing_job_id in missing_job_ids),
                    )
                if reusable is not None:
                    candidate = self.target_policy.validate(reusable["target_dir"])
                    candidate_key = target_identity(candidate)
                    duplicate_job_ids = [
                        row["job_id"]
                        for row in reusable_rows
                        if row["job_id"] != reusable["job_id"]
                        and self.target_policy.validate(row["target_dir"]).is_dir()
                        and target_identity(row["target_dir"]) != candidate_key
                    ]
                    if duplicate_job_ids:
                        # Historical or manually imported rows can predate the single-pool
                        # invariant. Keep the newest existing directory authoritative and
                        # send every other retained directory through prompt cleanup.
                        connection.executemany(
                            """
                            UPDATE cargo_jobs
                            SET cleanup_policy='delete_on_release', cleanup_status='pending',
                                cleanup_error=NULL
                            WHERE job_id=? AND cleanup_status='retained'
                            """,
                            ((duplicate_job_id,) for duplicate_job_id in duplicate_job_ids),
                        )
                    reserved_candidate = overlapping_cleanup_reservation(
                        connection, candidate_key
                    )
                    if reserved_candidate is not None:
                        raise CoordinatorError(
                            "cargo_lane_cleanup_reserved",
                            f"Compatible Cargo pool is reserved for cleanup: {candidate}",
                        )
                    if requested is not None and target_identity(requested) != candidate_key:
                        raise CoordinatorError(
                            "cargo_reuse_target_mismatch",
                            "The requested target differs from the existing primary pool for this "
                            "compatibility key",
                        )
                    target = candidate
                    reused_from_job_id = reusable["job_id"]
            if target is None:
                root = self.target_policy.choose_root(self.free_space)
                if reuse_key is not None:
                    target = self.target_policy.validate(
                        root / "zircon-engine" / "pool" / reuse_key
                    )
                else:
                    target = self.target_policy.validate(
                        root / "zircon-engine" / "ephemeral" / lane_kind.value / job_id
                    )
            target_key = target_identity(target)
            reservation = overlapping_cleanup_reservation(connection, target_key)
            if reservation is not None:
                raise CoordinatorError(
                    "cargo_lane_cleanup_reserved",
                    f"Cargo target is reserved for cleanup: {reservation['target_dir']}",
                )
            active_rows = connection.execute(
                """
                SELECT job_id, target_key FROM cargo_jobs
                WHERE status IN ('leased', 'running')
                """
            ).fetchall()
            occupied = next(
                (row for row in active_rows if targets_overlap(target_key, row["target_key"])),
                None,
            )
            if occupied is not None:
                raise CoordinatorError(
                    "cargo_reuse_pool_busy" if reuse_key is not None else "cargo_lane_occupied",
                    f"Cargo target is already owned by job {occupied['job_id']}",
                )
            tree_blocker = self._live_tree_target_blocker(connection, target_key)
            if tree_blocker is not None:
                job, live_pids = tree_blocker
                raise self._process_tree_alive_error(job, live_pids)
            connection.execute(
                """
                INSERT INTO cargo_jobs(
                    job_id, session_id, lane_kind, target_dir, status, dry_run,
                    created_at, last_heartbeat_at, target_key, pid, reuse_key,
                    compatibility_json, compatibility_key, reuse_profile, cleanup_policy,
                    cleanup_status, reused_from_job_id
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    job_id,
                    session_id,
                    lane_kind.value,
                    str(target),
                    CargoJobStatus.LEASED.value,
                    1 if dry_run else 0,
                    now,
                    now,
                    target_key,
                    owner_pid,
                    reuse_key,
                    compatibility_json,
                    reuse_key,
                    compatibility_json,
                    cleanup_policy.value,
                    (
                        CargoCleanupStatus.PENDING.value
                        if ephemeral
                        else CargoCleanupStatus.RETAINED.value
                    ),
                    reused_from_job_id,
                ),
            )
            if cpu_reservation is not None:
                connection.execute(
                    """
                    UPDATE cargo_lane_reservations
                    SET job_id=?, status='leased'
                    WHERE reservation_id=? AND status='pending'
                    """,
                    (job_id, cpu_reservation["reservation_id"]),
                )
        if not dry_run:
            try:
                target.mkdir(parents=True, exist_ok=True)
            except BaseException:
                with self.database.transaction() as connection:
                    connection.execute("DELETE FROM cargo_jobs WHERE job_id = ?", (job_id,))
                raise
        return self.get(job_id)

    def _require_cpu_reservation(
        self,
        connection,
        *,
        session_id: str,
        lane_kind: CargoLaneKind,
        compatibility_key: str | None,
        now: str,
    ):
        if lane_kind is CargoLaneKind.GPU:
            return None
        expire_invalid_pending_cpu_reservations(connection, now=now)
        reconcile_terminal_finished_cpu_reservations(connection, now=now)
        reservation = connection.execute(
            """
            SELECT * FROM cargo_lane_reservations
            WHERE lane_scope='cpu' AND status IN ('pending', 'leased', 'running', 'finished')
            ORDER BY created_at LIMIT 1
            """
        ).fetchone()
        if reservation is None:
            return None
        if reservation["session_id"] != session_id:
            raise CoordinatorError(
                "cargo_cpu_lane_reserved",
                "The next managed CPU lane is reserved for another Session",
                details={"sessionId": reservation["session_id"], "reservationId": reservation["reservation_id"]},
            )
        if reservation["status"] != "pending":
            raise CoordinatorError(
                "cargo_cpu_reservation_consumed",
                "The CPU reservation already owns a Cargo job",
                details={"reservationId": reservation["reservation_id"], "jobId": reservation["job_id"]},
            )
        if compatibility_key != reservation["compatibility_key"]:
            raise CoordinatorError(
                "cargo_cpu_reservation_compatibility_mismatch",
                "The reserved CPU job requires its exact compatibility pool",
                details={"reservationId": reservation["reservation_id"]},
            )
        return reservation

    @staticmethod
    def _require_gpu_reservation(connection, session_id: str, lane_kind: CargoLaneKind) -> None:
        """Honor the one-shot GPU lane reservation carried by the latest resume action.

        The action record is already durable and auditable.  The reservation is
        consumed only after its nominated Session starts and reaches a terminal
        state; a lease that is released before launch remains retryable and
        cannot lose FIFO priority to another Session.
        """
        if lane_kind is not CargoLaneKind.GPU:
            return
        action = connection.execute(
            """
            SELECT parameters_json, completed_at
            FROM action_requests
            WHERE action_kind='service.resume' AND status='succeeded'
              AND completed_at IS NOT NULL
            ORDER BY completed_at DESC, action_id DESC
            LIMIT 1
            """
        ).fetchone()
        if action is None:
            return
        try:
            parameters = json.loads(action["parameters_json"])
        except (TypeError, json.JSONDecodeError):
            return
        reserved_session_id = parameters.get("gpuReservationSessionId")
        if not isinstance(reserved_session_id, str) or not reserved_session_id:
            return
        active = connection.execute(
            """
            SELECT job_id FROM cargo_jobs
            WHERE session_id=? AND lane_kind='gpu' AND created_at>=?
              AND status IN ('leased', 'running')
            LIMIT 1
            """,
            (reserved_session_id, action["completed_at"]),
        ).fetchone()
        if active is not None:
            if session_id == reserved_session_id:
                return
            raise CoordinatorError(
                "cargo_gpu_lane_reserved",
                "The next managed GPU lane is reserved for another Session",
                details={"sessionId": reserved_session_id},
            )
        terminal_started = connection.execute(
            """
            SELECT 1 FROM cargo_jobs
            WHERE session_id=? AND lane_kind='gpu' AND created_at>=?
              AND started_at IS NOT NULL
              AND status NOT IN ('leased', 'running')
            LIMIT 1
            """,
            (reserved_session_id, action["completed_at"]),
        ).fetchone()
        if terminal_started is not None or session_id == reserved_session_id:
            return
        raise CoordinatorError(
            "cargo_gpu_lane_reserved",
            "The next managed GPU lane is reserved for another Session",
            details={"sessionId": reserved_session_id},
        )

    @staticmethod
    def _require_gpu_lane_available(connection, lane_kind: CargoLaneKind) -> None:
        if lane_kind is not CargoLaneKind.GPU:
            return
        occupied = connection.execute(
            """SELECT job_id, session_id, status, target_dir FROM cargo_jobs
               WHERE lane_kind='gpu' AND status IN ('leased', 'running')
               ORDER BY created_at, job_id LIMIT 1"""
        ).fetchone()
        if occupied is None:
            return
        raise CoordinatorError(
            "cargo_gpu_lane_occupied",
            "The single managed GPU lane is occupied",
            details={
                "jobId": occupied["job_id"],
                "sessionId": occupied["session_id"],
                "status": occupied["status"],
                "targetDir": occupied["target_dir"],
            },
        )

    def audit_active_gpu_jobs(self) -> tuple[CargoJob, ...]:
        """Return the startup-visible leases that occupy the single GPU lane."""
        with self.database.connect() as connection:
            rows = connection.execute(
                """SELECT * FROM cargo_jobs
                   WHERE lane_kind='gpu' AND status IN ('leased', 'running')
                   ORDER BY created_at, job_id"""
            ).fetchall()
        return tuple(self._from_row(row) for row in rows)

    def start(
        self,
        job_id: str,
        *,
        session_id: str,
        pid: int,
        command: list[str] | tuple[str, ...],
        root_is_supervisor: bool = False,
    ) -> CargoJob:
        if pid <= 0:
            raise ValueError("Cargo process PID must be positive")
        now = utc_text()
        root_process_creation_time = self._read_process_creation_time(pid)
        root_process_kind = (
            CargoProcessRootKind.SUPERVISOR
            if root_is_supervisor
            else CargoProcessRootKind.CARGO
        )
        try:
            with self.database.transaction() as connection:
                self._require_status(
                    connection, job_id, {CargoJobStatus.LEASED}, session_id=session_id
                )
                reservation = connection.execute(
                    "SELECT * FROM cargo_lane_reservations WHERE job_id=?", (job_id,)
                ).fetchone()
                if reservation is not None:
                    expire_invalid_pending_cpu_reservations(connection, now=now)
                    if reservation["status"] != "leased":
                        raise CoordinatorError(
                            "cargo_cpu_reservation_expired",
                            "The CPU reservation is no longer active",
                            details={"reservationId": reservation["reservation_id"]},
                        )
                    if reservation["command_fingerprint"] != self._command_fingerprint(tuple(command)):
                        raise CoordinatorError(
                            "cargo_cpu_reservation_command_mismatch",
                            "The reserved CPU job must start its exact approved command",
                            details={"reservationId": reservation["reservation_id"]},
                        )
                connection.execute(
                    """
                    UPDATE cargo_jobs
                    SET status = ?, pid = ?, root_process_creation_time = ?, root_process_kind = ?,
                        command_json = ?, started_at = ?, last_heartbeat_at = ?
                        , process_tree_observed_at = NULL, process_tree_live_pids_json = '[]',
                        process_tree_exited_at = NULL
                    WHERE job_id = ?
                    """,
                    (
                        CargoJobStatus.RUNNING.value,
                        pid,
                        root_process_creation_time,
                        root_process_kind.value,
                        json.dumps(tuple(command)),
                        now,
                        now,
                        job_id,
                    ),
                )
                if reservation is not None:
                    connection.execute(
                        """
                        UPDATE cargo_lane_reservations
                        SET status='running', started_at=?
                        WHERE reservation_id=? AND status='leased'
                        """,
                        (now, reservation["reservation_id"]),
                    )
                self._record_event(
                    connection,
                    session_id,
                    "cargo.start_accepted",
                    {
                        "jobId": job_id,
                        "pid": pid,
                        "rootIsSupervisor": root_is_supervisor,
                    },
                )
        except CoordinatorError as error:
            with self.database.transaction() as connection:
                self._record_event(
                    connection,
                    session_id,
                    "cargo.start_rejected",
                    {
                        "jobId": job_id,
                        "pid": pid,
                        "rootIsSupervisor": root_is_supervisor,
                        "code": error.code,
                    },
                )
            raise
        return self.get(job_id)

    @staticmethod
    def _record_event(
        connection,
        session_id: str,
        event_type: str,
        payload: dict[str, object],
    ) -> None:
        connection.execute(
            "INSERT INTO events(session_id, event_type, payload_json, created_at) VALUES (?, ?, ?, ?)",
            (session_id, event_type, json.dumps(payload, sort_keys=True), utc_text()),
        )

    def heartbeat(self, job_id: str, *, session_id: str) -> CargoJob:
        with self.database.transaction() as connection:
            self._require_status(
                connection,
                job_id,
                {CargoJobStatus.LEASED, CargoJobStatus.RUNNING},
                session_id=session_id,
            )
            connection.execute(
                "UPDATE cargo_jobs SET last_heartbeat_at = ? WHERE job_id = ?",
                (utc_text(), job_id),
            )
        return self.get(job_id)

    def finish(self, job_id: str, *, session_id: str, exit_code: int) -> CargoJob:
        now = utc_text()
        job = self.get(job_id)
        live_pids = self._live_process_pids(job, include_supervisor_root=False)
        blocked = False
        with self.database.transaction() as connection:
            self._require_status(
                connection,
                job_id,
                {CargoJobStatus.RUNNING, CargoJobStatus.ORPHANED},
                session_id=session_id,
            )
            self._record_process_tree_observation(connection, job_id, live_pids, now)
            if live_pids:
                blocked = True
            else:
                connection.execute(
                    """
                    UPDATE cargo_jobs
                    SET status = ?, exit_code = ?, finished_at = ?, last_heartbeat_at = ?
                    WHERE job_id = ?
                    """,
                    (
                        CargoJobStatus.SUCCEEDED.value if exit_code == 0 else CargoJobStatus.FAILED.value,
                        exit_code,
                        now,
                        now,
                        job_id,
                    ),
                )
                connection.execute(
                    """
                    UPDATE cargo_lane_reservations
                    SET status='finished', completed_at=?
                    WHERE job_id=? AND status='running'
                    """,
                    (now, job_id),
                )
        if blocked:
            raise self._process_tree_alive_error(job, live_pids)
        return self.get(job_id)

    def release(self, job_id: str, *, session_id: str) -> CargoJob:
        now = utc_text()
        job = self.get(job_id)
        live_pids = self._live_process_pids(job, include_supervisor_root=False)
        blocked = False
        with self.database.transaction() as connection:
            row = connection.execute(
                "SELECT status, session_id FROM cargo_jobs WHERE job_id = ?", (job_id,)
            ).fetchone()
            if row is None:
                raise CoordinatorError("cargo_job_not_found", f"Unknown Cargo job {job_id}")
            self._require_owner(row, job_id, session_id)
            if row["status"] == CargoJobStatus.RUNNING.value:
                raise CoordinatorError("cargo_job_running", "A running Cargo job cannot be released")
            self._record_process_tree_observation(connection, job_id, live_pids, now)
            if live_pids:
                blocked = True
            else:
                connection.execute(
                    """
                    UPDATE cargo_jobs
                    SET status = ?, released_at = ?, last_heartbeat_at = ?,
                        cleanup_status = CASE
                            WHEN cleanup_policy='delete_on_release' THEN 'pending'
                            ELSE cleanup_status
                        END,
                        cleanup_error = NULL
                    WHERE job_id = ?
                    """,
                    (CargoJobStatus.RELEASED.value, now, now, job_id),
                )
                connection.execute(
                    """
                    UPDATE cargo_lane_reservations
                    SET status='released', completed_at=COALESCE(completed_at, ?)
                    WHERE job_id=? AND status IN ('leased', 'finished')
                    """,
                    (now, job_id),
                )
        if blocked:
            raise self._process_tree_alive_error(job, live_pids)
        return self.get(job_id)

    def _live_process_pids(
        self, job: CargoJob, *, include_supervisor_root: bool = True
    ) -> tuple[int, ...]:
        if job.pid is None:
            return ()
        if job.root_process_creation_time is None:
            if job.status not in {
                CargoJobStatus.LEASED,
                CargoJobStatus.RUNNING,
            }:
                return ()
        else:
            observed_creation_time = self._read_process_creation_time(job.pid)
            if (
                observed_creation_time is not None
                and observed_creation_time != job.root_process_creation_time
            ):
                return ()
        process_tree = (
            self.supervisor_cargo_pids
            if (
                not include_supervisor_root
                and job.root_process_kind is CargoProcessRootKind.SUPERVISOR
            )
            else self.process_tree_pids
        )
        live_pids = {int(value) for value in process_tree(job.pid) if int(value) > 0}
        if (
            not include_supervisor_root
            and job.root_process_kind is CargoProcessRootKind.SUPERVISOR
        ):
            live_pids.discard(job.pid)
        return tuple(sorted(live_pids))

    def _read_process_creation_time(self, pid: int) -> str | None:
        try:
            return self.process_creation_time(pid)
        except (OSError, ValueError):
            return None

    def _live_tree_target_blocker(self, connection, target_key: str):
        rows = connection.execute(
            """
            SELECT * FROM cargo_jobs
            WHERE pid IS NOT NULL AND status NOT IN ('leased', 'running')
            ORDER BY created_at DESC, job_id DESC
            """
        ).fetchall()
        for row in rows:
            if not targets_overlap(target_key, row["target_key"]):
                continue
            job = self._from_row(row)
            live_pids = self._live_process_pids(job, include_supervisor_root=False)
            self._record_process_tree_observation(connection, job.job_id, live_pids, utc_text())
            if live_pids:
                return job, live_pids
        return None

    @staticmethod
    def _record_process_tree_observation(connection, job_id: str, live_pids: tuple[int, ...], now: str) -> None:
        connection.execute(
            """
            UPDATE cargo_jobs
            SET process_tree_observed_at = ?, process_tree_live_pids_json = ?,
                process_tree_exited_at = CASE WHEN ? THEN NULL ELSE ? END
            WHERE job_id = ?
            """,
            (now, json.dumps(live_pids), 1 if live_pids else 0, now, job_id),
        )

    @staticmethod
    def _process_tree_alive_error(job: CargoJob, live_pids: tuple[int, ...]) -> CoordinatorError:
        return CoordinatorError(
            "cargo_process_tree_alive",
            "Cargo process tree still owns the target; wait for its recorded descendants to exit",
            details={
                "jobId": job.job_id,
                "targetDir": job.target_dir,
                "rootPid": job.pid,
                "livePids": list(live_pids),
            },
        )

    def _compatibility_fingerprint(self, compatibility_json: str) -> str:
        digest = hashlib.sha256()
        digest.update(target_identity(self.repo_root).encode("utf-8"))
        digest.update(b"\0")
        digest.update(compatibility_json.encode("utf-8"))
        return digest.hexdigest()

    @staticmethod
    def _command_fingerprint(command: tuple[str, ...]) -> str:
        digest = hashlib.sha256()
        digest.update(json.dumps(command, ensure_ascii=False, separators=(",", ":")).encode("utf-8"))
        return digest.hexdigest()

    @staticmethod
    def _reservation_dict(row) -> dict[str, object]:
        return {
            "reservationId": row["reservation_id"],
            "sessionId": row["session_id"],
            "laneScope": row["lane_scope"],
            "compatibilityKey": row["compatibility_key"],
            "compatibility": (
                json.loads(row["compatibility_json"])
                if row["compatibility_json"]
                else None
            ),
            "jobId": row["job_id"],
            "status": row["status"],
            "createdAt": row["created_at"],
            "expiresAt": row["expires_at"],
            "startedAt": row["started_at"],
            "completedAt": row["completed_at"],
        }

    def get(self, job_id: str) -> CargoJob:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM cargo_jobs WHERE job_id = ?", (job_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError("cargo_job_not_found", f"Unknown Cargo job {job_id}")
        return self._from_row(row)

    def list(self) -> list[CargoJob]:
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT * FROM cargo_jobs ORDER BY created_at, job_id"
            ).fetchall()
        return [self._from_row(row) for row in rows]

    def reconcile_orphans(
        self,
        *,
        now: datetime | None = None,
        leased_timeout_seconds: int = 300,
    ) -> tuple[CargoJob, ...]:
        orphaned_ids: list[str] = []
        current_time = now or utc_now()
        now_text = utc_text(current_time)
        with self.database.transaction() as connection:
            rows = connection.execute(
                """
                SELECT *
                FROM cargo_jobs WHERE status IN ('leased', 'running')
                """
            ).fetchall()
            for row in rows:
                job = self._from_row(row)
                live_pids = self._live_process_pids(job)
                self._record_process_tree_observation(
                    connection, job.job_id, live_pids, now_text
                )
                if live_pids:
                    continue
                if (
                    row["status"] == CargoJobStatus.LEASED.value
                    and job.pid is None
                    and (current_time - parse_utc(row["last_heartbeat_at"])).total_seconds()
                    <= leased_timeout_seconds
                ):
                    continue
                cursor = connection.execute(
                    """
                    UPDATE cargo_jobs
                    SET status = ?, finished_at = ?, last_heartbeat_at = ?
                    WHERE job_id = ? AND status IN ('leased', 'running')
                    """,
                    (CargoJobStatus.ORPHANED.value, now_text, now_text, row["job_id"]),
                )
                if cursor.rowcount != 1:
                    continue
                connection.execute(
                    """
                    UPDATE cargo_lane_reservations
                    SET status='expired', completed_at=COALESCE(completed_at, ?)
                    WHERE job_id=? AND status IN ('leased', 'running')
                    """,
                    (now_text, row["job_id"]),
                )
                orphaned_ids.append(row["job_id"])
        return tuple(self.get(job_id) for job_id in orphaned_ids)

    @staticmethod
    def _require_status(
        connection,
        job_id: str,
        allowed: set[CargoJobStatus],
        *,
        session_id: str,
    ):
        row = connection.execute(
            "SELECT * FROM cargo_jobs WHERE job_id = ?", (job_id,)
        ).fetchone()
        if row is None:
            raise CoordinatorError("cargo_job_not_found", f"Unknown Cargo job {job_id}")
        CargoJobService._require_owner(row, job_id, session_id)
        current = CargoJobStatus(row["status"])
        if current not in allowed:
            raise CoordinatorError(
                "invalid_cargo_job_status",
                f"Cargo job {job_id} is {current.value}; expected {sorted(item.value for item in allowed)}",
            )
        return row

    @staticmethod
    def _require_owner(row, job_id: str, session_id: str) -> None:
        if row["session_id"] != session_id:
            raise CoordinatorError(
                "cargo_job_owner_mismatch",
                f"Cargo job {job_id} belongs to Session {row['session_id']}",
            )

    @staticmethod
    def _from_row(row) -> CargoJob:
        def parsed(value):
            return parse_utc(value) if value else None

        return CargoJob(
            job_id=row["job_id"],
            session_id=row["session_id"],
            lane_kind=CargoLaneKind(row["lane_kind"]),
            target_dir=row["target_dir"],
            status=CargoJobStatus(row["status"]),
            dry_run=bool(row["dry_run"]),
            pid=row["pid"],
            command=tuple(json.loads(row["command_json"])),
            exit_code=row["exit_code"],
            created_at=parse_utc(row["created_at"]),
            last_heartbeat_at=parse_utc(row["last_heartbeat_at"]),
            started_at=parsed(row["started_at"]),
            finished_at=parsed(row["finished_at"]),
            released_at=parsed(row["released_at"]),
            reuse_key=row["reuse_key"],
            compatibility_json=row["compatibility_json"],
            compatibility_key=row["compatibility_key"],
            reuse_profile=row["reuse_profile"],
            cleanup_policy=CargoCleanupPolicy(row["cleanup_policy"]),
            cleanup_status=CargoCleanupStatus(row["cleanup_status"]),
            reused_from_job_id=row["reused_from_job_id"],
            cleanup_error=row["cleanup_error"],
            process_tree_observed_at=parsed(row["process_tree_observed_at"]),
            live_process_pids=tuple(json.loads(row["process_tree_live_pids_json"])),
            process_tree_exited_at=parsed(row["process_tree_exited_at"]),
            root_process_creation_time=row["root_process_creation_time"],
            root_process_kind=CargoProcessRootKind(row["root_process_kind"]),
        )
