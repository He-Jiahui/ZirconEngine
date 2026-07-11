from __future__ import annotations

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


ACTIVE_CARGO_STATUSES = (CargoJobStatus.LEASED.value, CargoJobStatus.RUNNING.value)


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
        }


class TargetPathPolicy:
    def __init__(self, roots: list[str | Path] | tuple[str | Path, ...]):
        resolved: list[Path] = []
        for value in roots:
            root = Path(value).resolve()
            if root.name.casefold() != "zircon-engine" or root.parent.name.casefold() != "targets":
                raise CoordinatorError(
                    "invalid_target_root",
                    f"Managed target root must end in targets/zircon-engine: {root}",
                )
            resolved.append(root)
        if not resolved:
            raise CoordinatorError("target_root_unavailable", "No managed target root is configured")
        self.roots = tuple(dict.fromkeys(resolved))

    def validate(self, value: str | Path) -> Path:
        candidate = Path(value).resolve()
        for root in self.roots:
            lanes_root = (root / "lanes").resolve()
            if candidate.parent == lanes_root and candidate.name not in ("", ".", ".."):
                return candidate
        raise CoordinatorError(
            "cargo_target_not_managed",
            f"Cargo target must be under a managed targets/zircon-engine/lanes root: {candidate}",
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
        free_space: Callable[[Path], int] | None = None,
        process_alive: Callable[[int], bool] | None = None,
    ):
        self.database = database
        self.target_policy = target_policy
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
    ) -> CargoJob:
        if not isinstance(lane_kind, CargoLaneKind):
            raise ValueError("lane_kind must be a CargoLaneKind")
        job_id = uuid.uuid4().hex
        if requested_target is None:
            root = self.target_policy.choose_root(self.free_space)
            target = self.target_policy.validate(
                root / "lanes" / f"{lane_kind.value}-{job_id}"
            )
        else:
            target = self.target_policy.validate(requested_target)
        target_key = target_identity(target)
        now = utc_text()
        with self.database.transaction() as connection:
            if connection.execute(
                "SELECT 1 FROM sessions WHERE session_id = ?", (session_id,)
            ).fetchone() is None:
                raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")
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
                    "cargo_lane_occupied",
                    f"Cargo target is already owned by job {occupied['job_id']}",
                )
            connection.execute(
                """
                INSERT INTO cargo_jobs(
                    job_id, session_id, lane_kind, target_dir, status, dry_run,
                    created_at, last_heartbeat_at, target_key, pid
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
                ),
            )
        if not dry_run:
            try:
                target.mkdir(parents=True, exist_ok=requested_target is not None)
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
                SET status = ?, released_at = ?, last_heartbeat_at = ?
                WHERE job_id = ?
                """,
                (CargoJobStatus.RELEASED.value, now, now, job_id),
            )
        return self.get(job_id)

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
        )
