from __future__ import annotations

from .models import CoordinatorError


EXECUTABLE_CARGO_SESSION_STATUSES = frozenset(
    {
        "registered",
        "active",
        "waiting_lease",
        "resolving_failure",
        "waiting_validation",
        "finalizing",
    }
)


def require_executable_cargo_session(connection, session_id: str):
    row = connection.execute(
        "SELECT session_id, status FROM sessions WHERE session_id=?", (session_id,)
    ).fetchone()
    if row is None:
        raise CoordinatorError("session_not_found", f"Unknown Session {session_id}")
    if row["status"] not in EXECUTABLE_CARGO_SESSION_STATUSES:
        raise CoordinatorError(
            "cargo_session_not_executable",
            f"Session {session_id} cannot create or renew managed Cargo work while "
            f"{row['status']}",
            details={"sessionId": session_id, "status": row["status"]},
        )
    return row


def expire_unstarted_lane_reservations_for_session(
    connection, session_id: str, *, lane_scope: str, completed_at: str
) -> int:
    """Terminalize only one lane's FIFO claims that never acquired a job."""
    cursor = connection.execute(
        """
        UPDATE cargo_lane_reservations
        SET status='expired', completed_at=COALESCE(completed_at, ?)
        WHERE session_id=? AND lane_scope=? AND status='pending' AND job_id IS NULL
        """,
        (completed_at, session_id, lane_scope),
    )
    return cursor.rowcount


def expire_unstarted_cpu_reservations_for_session(
    connection, session_id: str, *, completed_at: str
) -> int:
    return expire_unstarted_lane_reservations_for_session(
        connection, session_id, lane_scope="cpu", completed_at=completed_at
    )


def expire_unstarted_gpu_reservations_for_session(
    connection, session_id: str, *, completed_at: str
) -> int:
    return expire_unstarted_lane_reservations_for_session(
        connection, session_id, lane_scope="gpu", completed_at=completed_at
    )


def expire_invalid_pending_lane_reservations(
    connection, *, lane_scope: str, now: str
) -> None:
    """Advance one FIFO lane past stale owners or elapsed absolute TTLs."""
    executable = tuple(sorted(EXECUTABLE_CARGO_SESSION_STATUSES))
    placeholders = ", ".join("?" for _ in executable)
    connection.execute(
        f"""
        UPDATE cargo_lane_reservations
        SET status='expired', completed_at=COALESCE(completed_at, ?)
        WHERE lane_scope=? AND status='pending' AND job_id IS NULL
          AND NOT EXISTS (
              SELECT 1 FROM sessions
              WHERE sessions.session_id=cargo_lane_reservations.session_id
                AND sessions.status IN ({placeholders})
          )
        """,
        (now, lane_scope, *executable),
    )
    connection.execute(
        """
        UPDATE cargo_lane_reservations
        SET status='expired', completed_at=COALESCE(completed_at, ?)
        WHERE lane_scope=? AND status='pending' AND job_id IS NULL
          AND expires_at<=?
        """,
        (now, lane_scope, now),
    )


def expire_invalid_pending_cpu_reservations(connection, *, now: str) -> None:
    expire_invalid_pending_lane_reservations(connection, lane_scope="cpu", now=now)


def expire_invalid_pending_gpu_reservations(connection, *, now: str) -> None:
    expire_invalid_pending_lane_reservations(connection, lane_scope="gpu", now=now)


def reconcile_terminal_finished_lane_reservations(
    connection, *, lane_scope: str, now: str
) -> int:
    """Release one lane's terminal rows only after owner recovery is safe."""
    executable = tuple(sorted(EXECUTABLE_CARGO_SESSION_STATUSES))
    placeholders = ", ".join("?" for _ in executable)
    cursor = connection.execute(
        f"""
        UPDATE cargo_lane_reservations
        SET status='released', completed_at=COALESCE(completed_at, ?)
        WHERE lane_scope=? AND status='finished'
          AND job_id IN (
              SELECT jobs.job_id
              FROM cargo_jobs AS jobs
              JOIN sessions AS owners ON owners.session_id=jobs.session_id
              WHERE jobs.status='released'
                AND jobs.process_tree_live_pids_json='[]'
                AND owners.status NOT IN ({placeholders})
          )
        """,
        (now, lane_scope, *executable),
    )
    return cursor.rowcount


def reconcile_terminal_finished_cpu_reservations(connection, *, now: str) -> int:
    return reconcile_terminal_finished_lane_reservations(
        connection, lane_scope="cpu", now=now
    )


def reconcile_terminal_finished_gpu_reservations(connection, *, now: str) -> int:
    return reconcile_terminal_finished_lane_reservations(
        connection, lane_scope="gpu", now=now
    )
