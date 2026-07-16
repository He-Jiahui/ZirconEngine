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


def expire_unstarted_cpu_reservations_for_session(
    connection, session_id: str, *, completed_at: str
) -> int:
    """Terminalize only FIFO claims that never acquired a nominated job."""
    cursor = connection.execute(
        """
        UPDATE cargo_lane_reservations
        SET status='expired', completed_at=COALESCE(completed_at, ?)
        WHERE session_id=? AND lane_scope='cpu' AND status='pending' AND job_id IS NULL
        """,
        (completed_at, session_id),
    )
    return cursor.rowcount


def expire_invalid_pending_cpu_reservations(connection, *, now: str) -> None:
    """Advance FIFO past stale owners or elapsed unconsumed absolute TTLs."""
    executable = tuple(sorted(EXECUTABLE_CARGO_SESSION_STATUSES))
    placeholders = ", ".join("?" for _ in executable)
    connection.execute(
        f"""
        UPDATE cargo_lane_reservations
        SET status='expired', completed_at=COALESCE(completed_at, ?)
        WHERE lane_scope='cpu' AND status='pending' AND job_id IS NULL
          AND NOT EXISTS (
              SELECT 1 FROM sessions
              WHERE sessions.session_id=cargo_lane_reservations.session_id
                AND sessions.status IN ({placeholders})
          )
        """,
        (now, *executable),
    )
    connection.execute(
        """
        UPDATE cargo_lane_reservations
        SET status='expired', completed_at=COALESCE(completed_at, ?)
        WHERE lane_scope='cpu' AND status='pending' AND job_id IS NULL
          AND expires_at<=?
        """,
        (now, now),
    )


def reconcile_terminal_finished_cpu_reservations(connection, *, now: str) -> int:
    """Release legacy FIFO rows only after their job is terminal and owner is non-executable."""
    executable = tuple(sorted(EXECUTABLE_CARGO_SESSION_STATUSES))
    placeholders = ", ".join("?" for _ in executable)
    cursor = connection.execute(
        f"""
        UPDATE cargo_lane_reservations
        SET status='released', completed_at=COALESCE(completed_at, ?)
        WHERE lane_scope='cpu' AND status='finished'
          AND job_id IN (
              SELECT jobs.job_id
              FROM cargo_jobs AS jobs
              JOIN sessions AS owners ON owners.session_id=jobs.session_id
              WHERE jobs.status='released'
                AND jobs.process_tree_live_pids_json='[]'
                AND owners.status NOT IN ({placeholders})
          )
        """,
        (now, *executable),
    )
    return cursor.rowcount
