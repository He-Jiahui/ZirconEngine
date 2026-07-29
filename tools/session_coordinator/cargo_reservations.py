from __future__ import annotations

import json

from .models import CoordinatorError, utc_text


NORMAL_CPU_RESERVATION_PRIORITY = 1000

ACTIVE_LANE_RESERVATION_STATUSES = ("pending", "leased", "running", "finished")
LANE_FIFO_ORDER = "priority_rank, created_at, reservation_id"


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
) -> int:
    """Advance one FIFO lane past stale owners or elapsed absolute TTLs."""
    candidates = connection.execute(
        """
        SELECT reservations.reservation_id, reservations.session_id,
               reservations.status, reservations.job_id, reservations.expires_at,
               owners.status AS owner_status
        FROM cargo_lane_reservations AS reservations
        LEFT JOIN sessions AS owners ON owners.session_id=reservations.session_id
        WHERE reservations.lane_scope=? AND reservations.status='pending'
          AND reservations.job_id IS NULL
        ORDER BY reservations.created_at, reservations.reservation_id
        """,
        (lane_scope,),
    ).fetchall()
    reconciled = 0
    for candidate in candidates:
        if candidate["owner_status"] not in EXECUTABLE_CARGO_SESSION_STATUSES:
            reason = "owner_not_executable"
        elif candidate["expires_at"] <= now:
            reason = "absolute_ttl_elapsed"
        else:
            continue
        cursor = connection.execute(
            """
            UPDATE cargo_lane_reservations
            SET status='expired', completed_at=COALESCE(completed_at, ?)
            WHERE reservation_id=? AND lane_scope=?
              AND status='pending' AND job_id IS NULL
            """,
            (now, candidate["reservation_id"], lane_scope),
        )
        if cursor.rowcount != 1:
            continue
        _record_reservation_reconciliation(
            connection,
            candidate,
            lane_scope=lane_scope,
            status="expired",
            reason=reason,
            created_at=now,
        )
        reconciled += 1
    return reconciled


def expire_invalid_pending_cpu_reservations(connection, *, now: str) -> int:
    return expire_invalid_pending_lane_reservations(connection, lane_scope="cpu", now=now)


def expire_invalid_pending_gpu_reservations(connection, *, now: str) -> int:
    return expire_invalid_pending_lane_reservations(connection, lane_scope="gpu", now=now)


def reconcile_terminal_finished_lane_reservations(
    connection, *, lane_scope: str, now: str
) -> int:
    """Release one lane's terminal rows only after owner recovery is safe."""
    candidates = connection.execute(
        """
        SELECT reservations.reservation_id, reservations.session_id,
               reservations.status, reservations.job_id,
               owners.status AS owner_status
        FROM cargo_lane_reservations AS reservations
        JOIN cargo_jobs AS jobs ON jobs.job_id=reservations.job_id
        JOIN sessions AS owners ON owners.session_id=jobs.session_id
        WHERE reservations.lane_scope=? AND reservations.status='finished'
          AND jobs.status='released'
          AND jobs.process_tree_live_pids_json='[]'
        ORDER BY reservations.created_at, reservations.reservation_id
        """,
        (lane_scope,),
    ).fetchall()
    reconciled = 0
    for candidate in candidates:
        if candidate["owner_status"] in EXECUTABLE_CARGO_SESSION_STATUSES:
            continue
        cursor = connection.execute(
            """
            UPDATE cargo_lane_reservations
            SET status='released', completed_at=COALESCE(completed_at, ?)
            WHERE reservation_id=? AND lane_scope=? AND status='finished'
            """,
            (now, candidate["reservation_id"], lane_scope),
        )
        if cursor.rowcount != 1:
            continue
        _record_reservation_reconciliation(
            connection,
            candidate,
            lane_scope=lane_scope,
            status="released",
            reason="terminal_job_released_owner_not_executable",
            created_at=now,
        )
        reconciled += 1
    return reconciled


def _record_reservation_reconciliation(
    connection,
    reservation,
    *,
    lane_scope: str,
    status: str,
    reason: str,
    created_at: str,
) -> None:
    connection.execute(
        """
        INSERT INTO events(session_id, event_type, payload_json, created_at)
        VALUES (?, 'cargo.reservation_reconciled', ?, ?)
        """,
        (
            str(reservation["session_id"]),
            json.dumps(
                {
                    "reservationId": reservation["reservation_id"],
                    "sessionId": reservation["session_id"],
                    "laneScope": lane_scope,
                    "previousStatus": reservation["status"],
                    "status": status,
                    "reason": reason,
                    "jobId": reservation["job_id"],
                },
                sort_keys=True,
            ),
            created_at,
        ),
    )


def reconcile_terminal_finished_cpu_reservations(connection, *, now: str) -> int:
    return reconcile_terminal_finished_lane_reservations(
        connection, lane_scope="cpu", now=now
    )


def reconcile_terminal_finished_gpu_reservations(connection, *, now: str) -> int:
    return reconcile_terminal_finished_lane_reservations(
        connection, lane_scope="gpu", now=now
    )


def lane_fifo_head(connection, *, lane_scope: str, execution_mode: str | None = None):
    """Return the durable FIFO head for one lane and optional CPU execution mode."""
    placeholders = ", ".join("?" for _ in ACTIVE_LANE_RESERVATION_STATUSES)
    query = f"""
        SELECT * FROM cargo_lane_reservations
        WHERE lane_scope=? AND status IN ({placeholders})
    """
    parameters: tuple[str, ...] = (lane_scope,)
    parameters += ACTIVE_LANE_RESERVATION_STATUSES
    if execution_mode is not None:
        query += " AND execution_mode=?"
        parameters += (execution_mode,)
    query += f" ORDER BY {LANE_FIFO_ORDER} LIMIT 1"
    return connection.execute(query, parameters).fetchone()


def lane_fifo_predecessor(
    connection, reservation, *, lane_scope: str, execution_mode: str | None = None
):
    """Return the active lane row ahead of ``reservation`` under the bind order."""
    placeholders = ", ".join("?" for _ in ACTIVE_LANE_RESERVATION_STATUSES)
    query = f"""
        SELECT *
        FROM cargo_lane_reservations
        WHERE lane_scope=? AND status IN ({placeholders})
          AND reservation_id<>?
          AND (
                priority_rank < ?
                OR (
                    priority_rank = ?
                    AND (
                        created_at < ?
                        OR (created_at = ? AND reservation_id < ?)
                    )
                )
          )
    """
    parameters = (
        lane_scope,
        *ACTIVE_LANE_RESERVATION_STATUSES,
        reservation["reservation_id"],
        reservation["priority_rank"],
        reservation["priority_rank"],
        reservation["created_at"],
        reservation["created_at"],
        reservation["reservation_id"],
    )
    if execution_mode is not None:
        query += " AND execution_mode=?"
        parameters += (execution_mode,)
    query += f" ORDER BY {LANE_FIFO_ORDER} LIMIT 1"
    return connection.execute(query, parameters).fetchone()


def cpu_warm_fifo_predecessor(connection, reservation):
    """Return the warm CPU reservation ahead of ``reservation`` under the bind order."""
    row = lane_fifo_predecessor(
        connection,
        reservation,
        lane_scope="cpu",
        execution_mode="warm",
    )
    if row is None:
        return None
    return {
        "reservationId": row["reservation_id"],
        "sessionId": row["session_id"],
        "priorityRank": row["priority_rank"],
        "createdAt": row["created_at"],
    }


def failure_priority_yield_barrier(
    connection,
    *,
    session_id: str,
    failure_lifecycle_key: str,
    created_at: str,
    reservation_id: str,
):
    """Return the normal warm reservation that a repeated failure retry must yield to."""
    prior_priority = connection.execute(
        """
        SELECT reservation_id
        FROM cargo_lane_reservations
        WHERE lane_scope='cpu' AND session_id=? AND failure_lifecycle_key=?
          AND priority_rank < ?
          AND (created_at < ? OR (created_at = ? AND reservation_id < ?))
          AND status IN ('leased', 'running', 'finished', 'released', 'expired')
        ORDER BY created_at DESC, reservation_id DESC
        LIMIT 1
        """,
        (
            session_id,
            failure_lifecycle_key,
            NORMAL_CPU_RESERVATION_PRIORITY,
            created_at,
            created_at,
            reservation_id,
        ),
    ).fetchone()
    if prior_priority is None:
        return None
    barrier = connection.execute(
        """
        SELECT reservation_id
        FROM cargo_lane_reservations
        WHERE lane_scope='cpu' AND execution_mode='warm'
          AND priority_rank >= ?
          AND (created_at < ? OR (created_at = ? AND reservation_id < ?))
          AND status IN ('pending', 'leased', 'running', 'finished')
        ORDER BY created_at, reservation_id
        LIMIT 1
        """,
        (
            NORMAL_CPU_RESERVATION_PRIORITY,
            created_at,
            created_at,
            reservation_id,
        ),
    ).fetchone()
    if barrier is None:
        return None
    return {
        "reservation_id": barrier["reservation_id"],
        "prior_priority_reservation_id": prior_priority["reservation_id"],
    }


def reconcile_cpu_fifo_eligibility(connection, *, now: str) -> tuple[int, int, int]:
    """Apply the same CPU FIFO reconciliation before every head observation or bind."""
    expired = expire_invalid_pending_cpu_reservations(connection, now=now)
    released = reconcile_terminal_finished_cpu_reservations(connection, now=now)
    candidates = connection.execute(
        """
        SELECT reservation_id, session_id, failure_lifecycle_key, priority_rank, created_at
        FROM cargo_lane_reservations
        WHERE lane_scope='cpu' AND status='pending'
          AND priority_rank < ? AND failure_lifecycle_key IS NOT NULL
        ORDER BY created_at, reservation_id
        """,
        (NORMAL_CPU_RESERVATION_PRIORITY,),
    ).fetchall()
    yielded = 0
    for candidate in candidates:
        barrier = failure_priority_yield_barrier(
            connection,
            session_id=str(candidate["session_id"]),
            failure_lifecycle_key=str(candidate["failure_lifecycle_key"]),
            created_at=str(candidate["created_at"]),
            reservation_id=str(candidate["reservation_id"]),
        )
        if barrier is None:
            continue
        cursor = connection.execute(
            """
            UPDATE cargo_lane_reservations
            SET priority_rank=?
            WHERE reservation_id=? AND status='pending' AND priority_rank < ?
            """,
            (
                NORMAL_CPU_RESERVATION_PRIORITY,
                candidate["reservation_id"],
                NORMAL_CPU_RESERVATION_PRIORITY,
            ),
        )
        if cursor.rowcount != 1:
            continue
        connection.execute(
            """
            INSERT INTO events(session_id, event_type, payload_json, created_at)
            VALUES (?, ?, ?, ?)
            """,
            (
                str(candidate["session_id"]),
                "cargo.reservation_failure_priority_yielded",
                json.dumps(
                    {
                        "reservationId": candidate["reservation_id"],
                        "barrierReservationId": barrier["reservation_id"],
                        "priorPriorityReservationId": barrier["prior_priority_reservation_id"],
                        "previousPriorityRank": candidate["priority_rank"],
                        "priorityRank": NORMAL_CPU_RESERVATION_PRIORITY,
                    },
                    sort_keys=True,
                ),
                utc_text(),
            ),
        )
        yielded += 1
    return expired, released, yielded
