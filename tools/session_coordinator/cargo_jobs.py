from __future__ import annotations

import hashlib
import json
import shutil
import uuid
from dataclasses import dataclass
from datetime import datetime
from enum import StrEnum
from pathlib import Path
from typing import Callable

from .database import Database
from .models import CoordinatorError, parse_utc, utc_now, utc_text
from .processes import process_is_alive


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

    def to_dict(self) -> dict[str, object]:
        return {
            "job_id": self.job_id,
            "session_id": self.session_id,
            "lane_kind": self.lane_kind.value,
            "target_dir": self.target_dir,
            "status": self.status.value,
            "dry_run": self.dry_run,
            "pid": self.pid,
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
    ):
        self.database = database
        self.target_policy = target_policy
        self.repo_root = Path(repo_root).resolve() if repo_root is not None else Path.cwd().resolve()
        self.free_space = free_space or (
            lambda path: shutil.disk_usage(path.anchor or path.parent).free
        )
        self.process_alive = process_alive or process_is_alive

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
            if connection.execute(
                "SELECT 1 FROM sessions WHERE session_id = ?", (session_id,)
            ).fetchone() is None:
                raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")
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
                    reserved_candidate = connection.execute(
                        "SELECT 1 FROM cleanup_reservations WHERE target_key=?",
                        (candidate_key,),
                    ).fetchone()
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
            reservation = connection.execute(
                "SELECT target_dir FROM cleanup_reservations WHERE target_key = ?",
                (target_key,),
            ).fetchone()
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
        if not dry_run:
            try:
                target.mkdir(parents=True, exist_ok=True)
            except BaseException:
                with self.database.transaction() as connection:
                    connection.execute("DELETE FROM cargo_jobs WHERE job_id = ?", (job_id,))
                raise
        return self.get(job_id)

    def start(
        self,
        job_id: str,
        *,
        session_id: str,
        pid: int,
        command: list[str] | tuple[str, ...],
    ) -> CargoJob:
        if pid <= 0:
            raise ValueError("Cargo process PID must be positive")
        now = utc_text()
        with self.database.transaction() as connection:
            self._require_status(
                connection, job_id, {CargoJobStatus.LEASED}, session_id=session_id
            )
            connection.execute(
                """
                UPDATE cargo_jobs
                SET status = ?, pid = ?, command_json = ?, started_at = ?, last_heartbeat_at = ?
                WHERE job_id = ?
                """,
                (
                    CargoJobStatus.RUNNING.value,
                    pid,
                    json.dumps(tuple(command)),
                    now,
                    now,
                    job_id,
                ),
            )
        return self.get(job_id)

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
        with self.database.transaction() as connection:
            self._require_status(
                connection, job_id, {CargoJobStatus.RUNNING}, session_id=session_id
            )
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
        return self.get(job_id)

    def release(self, job_id: str, *, session_id: str) -> CargoJob:
        now = utc_text()
        with self.database.transaction() as connection:
            row = connection.execute(
                "SELECT status, session_id FROM cargo_jobs WHERE job_id = ?", (job_id,)
            ).fetchone()
            if row is None:
                raise CoordinatorError("cargo_job_not_found", f"Unknown Cargo job {job_id}")
            self._require_owner(row, job_id, session_id)
            if row["status"] == CargoJobStatus.RUNNING.value:
                raise CoordinatorError("cargo_job_running", "A running Cargo job cannot be released")
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
        return self.get(job_id)

    def _compatibility_fingerprint(self, compatibility_json: str) -> str:
        digest = hashlib.sha256()
        digest.update(target_identity(self.repo_root).encode("utf-8"))
        digest.update(b"\0")
        digest.update(compatibility_json.encode("utf-8"))
        return digest.hexdigest()

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
                SELECT job_id, pid, status, last_heartbeat_at
                FROM cargo_jobs WHERE status IN ('leased', 'running')
                """
            ).fetchall()
            for row in rows:
                pid = row["pid"]
                if pid is not None and self.process_alive(int(pid)):
                    continue
                if (
                    row["status"] == CargoJobStatus.LEASED.value
                    and pid is None
                    and (current_time - parse_utc(row["last_heartbeat_at"])).total_seconds()
                    <= leased_timeout_seconds
                ):
                    continue
                connection.execute(
                    """
                    UPDATE cargo_jobs
                    SET status = ?, finished_at = ?, last_heartbeat_at = ?
                    WHERE job_id = ? AND status IN ('leased', 'running')
                    """,
                    (CargoJobStatus.ORPHANED.value, now_text, now_text, row["job_id"]),
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
        )
