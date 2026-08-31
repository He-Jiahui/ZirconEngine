from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from sqlite3 import Connection
from typing import Mapping, Sequence

from .database import Database
from .models import CoordinatorError, utc_text


@dataclass(frozen=True)
class SpawnObservation:
    pid: int
    creation_time: str | None
    root_kind: str
    live_pids: tuple[int, ...]


def persist_spawn_authorization(
    connection: Connection,
    *,
    job_id: str,
    session_id: str,
    command: Sequence[str],
    authorized_at: str,
    reservation_id: str | None,
) -> None:
    """Consume one exact launch permit before any child process exists."""

    cursor = connection.execute(
        """
        UPDATE cargo_jobs
        SET status='running', pid=NULL, root_process_creation_time=NULL,
            root_process_kind='supervisor', command_json=?, started_at=?,
            last_heartbeat_at=?, process_tree_observed_at=?,
            process_tree_live_pids_json='[]', process_tree_exited_at=?
        WHERE job_id=? AND session_id=? AND status='leased' AND pid IS NULL
        """,
        (
            json.dumps(tuple(command)),
            authorized_at,
            authorized_at,
            authorized_at,
            authorized_at,
            job_id,
            session_id,
        ),
    )
    if cursor.rowcount != 1:
        raise CoordinatorError(
            "cargo_start_authorization_race",
            "Cargo job changed before launch authorization was persisted",
            details={"jobId": job_id},
        )
    if reservation_id is not None:
        cursor = connection.execute(
            """
            UPDATE cargo_lane_reservations
            SET status='running', started_at=?
            WHERE reservation_id=? AND job_id=? AND status='leased'
            """,
            (authorized_at, reservation_id, job_id),
        )
        if cursor.rowcount != 1:
            raise CoordinatorError(
                "cargo_start_authorization_reservation_race",
                "Cargo reservation changed before launch authorization was persisted",
                details={"jobId": job_id, "reservationId": reservation_id},
            )
    _record_event(
        connection,
        session_id,
        "cargo.start_authorized",
        {"jobId": job_id, "rootIsSupervisor": True},
        authorized_at,
    )


def persist_authorized_spawn_observation(
    connection: Connection,
    *,
    job_id: str,
    session_id: str,
    command: Sequence[str],
    observation: SpawnObservation,
    observed_at: str,
) -> None:
    """Bind one spawned process to its exact persisted authorization."""

    row = connection.execute(
        "SELECT * FROM cargo_jobs WHERE job_id=?", (job_id,)
    ).fetchone()
    if row is None:
        raise CoordinatorError("cargo_job_not_found", f"Unknown Cargo job {job_id}")
    if row["session_id"] != session_id:
        raise CoordinatorError(
            "cargo_job_owner_mismatch", f"Cargo job {job_id} belongs to another Session"
        )
    if (
        row["status"] != "running"
        or row["pid"] is not None
        or tuple(json.loads(row["command_json"])) != tuple(command)
        or row["root_process_kind"] != observation.root_kind
    ):
        raise CoordinatorError(
            "cargo_start_authorization_mismatch",
            "Spawned Cargo process does not match its durable launch authorization",
            details={"jobId": job_id, "pid": observation.pid},
        )
    cursor = connection.execute(
        """
        UPDATE cargo_jobs
        SET pid=?, root_process_creation_time=?, last_heartbeat_at=?,
            process_tree_observed_at=?, process_tree_live_pids_json=?,
            process_tree_exited_at=CASE WHEN ? THEN NULL ELSE ? END
        WHERE job_id=? AND status='running' AND pid IS NULL
        """,
        (
            observation.pid,
            observation.creation_time,
            observed_at,
            observed_at,
            json.dumps(observation.live_pids),
            1 if observation.live_pids else 0,
            observed_at,
            job_id,
        ),
    )
    if cursor.rowcount != 1:
        raise CoordinatorError(
            "cargo_start_authorization_race",
            "Cargo launch authorization changed before process registration",
            details={"jobId": job_id, "pid": observation.pid},
        )
    _record_event(
        connection,
        session_id,
        "cargo.start_accepted",
        {"jobId": job_id, "pid": observation.pid, "rootIsSupervisor": True},
        observed_at,
    )


def persist_authorized_managed_run(
    connection: Connection,
    *,
    run_id: str,
    job_id: str,
    session_id: str,
    command: Sequence[str],
    environment: Mapping[str, str],
    stdout_path: Path,
    stderr_path: Path,
    started_at: str,
    observed_at: str,
    observation: SpawnObservation,
) -> None:
    """Atomically bind the suspended root and its durable run projection."""

    persist_authorized_spawn_observation(
        connection,
        job_id=job_id,
        session_id=session_id,
        command=command,
        observation=observation,
        observed_at=observed_at,
    )
    connection.execute(
        """
        INSERT INTO cargo_job_runs(
            run_id, job_id, session_id, command_json, environment_json, status,
            stdout_path, stderr_path, error_code, started_at
        ) VALUES (?, ?, ?, ?, ?, 'running', ?, ?,
                  'cargo_run_suspended_before_resume', ?)
        """,
        (
            run_id,
            job_id,
            session_id,
            json.dumps(tuple(command)),
            json.dumps(dict(environment), sort_keys=True),
            str(stdout_path),
            str(stderr_path),
            started_at,
        ),
    )


def mark_managed_run_resumed(
    connection: Connection,
    *,
    run_id: str,
    job_id: str,
    session_id: str,
) -> None:
    cursor = connection.execute(
        """
        UPDATE cargo_job_runs
        SET error_code=NULL
        WHERE run_id=? AND job_id=? AND session_id=? AND status='running'
          AND error_code='cargo_run_suspended_before_resume'
        """,
        (run_id, job_id, session_id),
    )
    if cursor.rowcount != 1:
        raise CoordinatorError(
            "cargo_run_resume_registration_changed",
            "Cargo run changed while recording its resume decision",
            details={"runId": run_id, "jobId": job_id},
        )


def rollback_spawn_authorization(
    connection: Connection,
    *,
    job_id: str,
    session_id: str,
    command: Sequence[str],
    rolled_back_at: str,
) -> None:
    """Restore a permit only while no process identity has been registered."""

    row = connection.execute(
        "SELECT * FROM cargo_jobs WHERE job_id=?", (job_id,)
    ).fetchone()
    if row is None:
        raise CoordinatorError("cargo_job_not_found", f"Unknown Cargo job {job_id}")
    if row["session_id"] != session_id:
        raise CoordinatorError(
            "cargo_job_owner_mismatch", f"Cargo job {job_id} belongs to another Session"
        )
    if (
        row["status"] != "running"
        or row["pid"] is not None
        or tuple(json.loads(row["command_json"])) != tuple(command)
    ):
        raise CoordinatorError(
            "cargo_start_authorization_rollback_conflict",
            "Cargo launch authorization cannot be rolled back after process registration",
            details={"jobId": job_id},
        )
    cursor = connection.execute(
        """
        UPDATE cargo_jobs
        SET status='leased', pid=NULL, root_process_creation_time=NULL,
            root_process_kind='cargo', command_json='[]', started_at=NULL,
            last_heartbeat_at=?, process_tree_observed_at=NULL,
            process_tree_live_pids_json='[]', process_tree_exited_at=NULL
        WHERE job_id=? AND status='running' AND pid IS NULL
        """,
        (rolled_back_at, job_id),
    )
    if cursor.rowcount != 1:
        raise CoordinatorError(
            "cargo_start_authorization_rollback_race",
            "Cargo launch authorization changed during rollback",
            details={"jobId": job_id},
        )
    connection.execute(
        """
        UPDATE cargo_lane_reservations
        SET status='leased', started_at=NULL
        WHERE job_id=? AND status='running'
        """,
        (job_id,),
    )
    _record_event(
        connection,
        session_id,
        "cargo.start_authorization_rolled_back",
        {"jobId": job_id},
        rolled_back_at,
    )


def _record_event(
    connection: Connection,
    session_id: str,
    event_type: str,
    payload: Mapping[str, object],
    created_at: str,
) -> None:
    connection.execute(
        "INSERT INTO events(session_id, event_type, payload_json, created_at) VALUES (?, ?, ?, ?)",
        (session_id, event_type, json.dumps(dict(payload), sort_keys=True), created_at),
    )


def persist_cleanup_unproven_spawn(
    database: Database,
    *,
    run_id: str,
    job_id: str,
    session_id: str,
    command: Sequence[str],
    environment: Mapping[str, str],
    stdout_path: Path,
    stderr_path: Path,
    started_at: str,
    observation: SpawnObservation,
    rejection_code: str,
) -> None:
    """Fail closed around a spawned process whose termination is unproven."""

    now = utc_text()
    command_json = json.dumps(tuple(command))
    with database.transaction() as connection:
        job = connection.execute(
            "SELECT * FROM cargo_jobs WHERE job_id=?", (job_id,)
        ).fetchone()
        if job is None:
            raise CoordinatorError("cargo_job_not_found", f"Unknown Cargo job {job_id}")
        if job["session_id"] != session_id:
            raise CoordinatorError(
                "cargo_job_owner_mismatch",
                f"Cargo job {job_id} belongs to another Session",
            )
        if job["status"] in {"leased", "running"} and job["pid"] is None:
            connection.execute(
                """
                UPDATE cargo_jobs
                SET status='running', pid=?, root_process_creation_time=?,
                    root_process_kind=?, command_json=?, started_at=COALESCE(started_at, ?),
                    last_heartbeat_at=?, process_tree_observed_at=?,
                    process_tree_live_pids_json=?,
                    process_tree_exited_at=CASE WHEN ? THEN NULL ELSE ? END
                WHERE job_id=? AND status IN ('leased', 'running') AND pid IS NULL
                """,
                (
                    observation.pid,
                    observation.creation_time,
                    observation.root_kind,
                    command_json,
                    started_at,
                    now,
                    now,
                    json.dumps(observation.live_pids),
                    1 if observation.live_pids else 0,
                    now,
                    job_id,
                ),
            )
        elif job["status"] != "running" or job["pid"] != observation.pid:
            raise CoordinatorError(
                "cargo_cleanup_unproven_registration_conflict",
                "Spawned process identity conflicts with the durable Cargo job",
                details={"jobId": job_id, "pid": observation.pid},
            )

        existing_run = connection.execute(
            "SELECT run_id FROM cargo_job_runs WHERE job_id=?", (job_id,)
        ).fetchone()
        if existing_run is None:
            connection.execute(
                """
                INSERT INTO cargo_job_runs(
                    run_id, job_id, session_id, command_json, environment_json, status,
                    stdout_path, stderr_path, started_at
                ) VALUES (?, ?, ?, ?, ?, 'running', ?, ?, ?)
                """,
                (
                    run_id,
                    job_id,
                    session_id,
                    command_json,
                    json.dumps(dict(environment), sort_keys=True),
                    str(stdout_path),
                    str(stderr_path),
                    started_at,
                ),
            )
        elif existing_run["run_id"] != run_id:
            raise CoordinatorError(
                "cargo_cleanup_unproven_run_conflict",
                "Spawned process conflicts with another durable Cargo run",
                details={"jobId": job_id, "runId": existing_run["run_id"]},
            )

        connection.execute(
            """
            UPDATE cargo_lane_reservations
            SET status='running', started_at=COALESCE(started_at, ?)
            WHERE job_id=? AND status='leased'
            """,
            (started_at, job_id),
        )

        connection.execute(
            "INSERT INTO events(session_id, event_type, payload_json, created_at) VALUES (?, ?, ?, ?)",
            (
                session_id,
                "cargo.spawn_cleanup_unproven",
                json.dumps(
                    {
                        "jobId": job_id,
                        "pid": observation.pid,
                        "rejectionCode": rejection_code,
                        "runId": run_id,
                    },
                    sort_keys=True,
                ),
                now,
            ),
        )
