"""Reconstruct validation-ticket phase timings from durable coordinator evidence."""

from __future__ import annotations

import json
import sqlite3
from datetime import UTC, datetime
from typing import Mapping


_TERMINAL = frozenset({"passed", "failed", "snapshot_stale"})
_SUBMITTED = "validation.ticket_submitted"
_STATUS_CHANGED = "validation.ticket_status_changed"
_COPY_LINKED = "validation.ticket_copy_linked"
_RUN_LINKED = "validation.ticket_run_linked"
_FAILURE_REUSED = "validation.ticket_failure_reused"
_CLEANUP = "validation.ticket_cleanup"


def _timestamp(value: object) -> datetime | None:
    if not isinstance(value, str) or not value:
        return None
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
        return parsed.replace(tzinfo=UTC) if parsed.tzinfo is None else parsed
    except (TypeError, ValueError):
        return None


def _elapsed_ms(start: datetime | None, finish: datetime | None) -> int | None:
    if start is None or finish is None:
        return None
    try:
        seconds = (finish - start).total_seconds()
    except TypeError:
        return None
    return max(0, round(seconds * 1000)) if seconds >= 0 else None


def _payload(raw: object) -> dict[str, object]:
    try:
        decoded = json.loads(str(raw))
    except (TypeError, ValueError, json.JSONDecodeError):
        return {}
    return decoded if isinstance(decoded, dict) else {}


def _nonnegative_ms(value: object) -> int | None:
    if isinstance(value, bool) or not isinstance(value, (int, float)) or value < 0:
        return None
    return round(value)


def _table_columns(connection: sqlite3.Connection, table: str) -> frozenset[str]:
    present = connection.execute(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?", (table,)
    ).fetchone()
    if present is None:
        return frozenset()
    return frozenset(str(row[1]) for row in connection.execute(f"PRAGMA table_info({table})"))


def _run_intervals(
    connection: sqlite3.Connection,
    *,
    ticket_id: str,
    job_ids: set[str],
) -> tuple[list[tuple[datetime, datetime]], bool]:
    intervals: list[tuple[datetime, datetime]] = []
    incomplete = False
    for table in ("validation_copy_runs", "cargo_job_runs"):
        columns = _table_columns(connection, table)
        if not {"run_id", "job_id", "started_at", "completed_at"}.issubset(columns):
            continue
        parameters: list[str] = [ticket_id]
        predicates = ["run_id=?"]
        if job_ids:
            placeholders = ",".join("?" for _ in job_ids)
            predicates.append(f"job_id IN ({placeholders})")
            parameters.extend(sorted(job_ids))
        rows = connection.execute(
            f"SELECT started_at,completed_at FROM {table} WHERE " + " OR ".join(predicates),
            parameters,
        )
        for raw_start, raw_finish in rows:
            start = _timestamp(raw_start)
            finish = _timestamp(raw_finish)
            if start is None or finish is None or _elapsed_ms(start, finish) is None:
                incomplete = True
                continue
            intervals.append((start, finish))
    return intervals, incomplete


def _union_ms(intervals: list[tuple[datetime, datetime]]) -> int | None:
    clipped = sorted(intervals)
    if not clipped:
        return 0
    total = 0
    current_start, current_finish = clipped[0]
    for start, finish in clipped[1:]:
        if start <= current_finish:
            current_finish = max(current_finish, finish)
            continue
        elapsed = _elapsed_ms(current_start, current_finish)
        if elapsed is None:
            return None
        total += elapsed
        current_start, current_finish = start, finish
    elapsed = _elapsed_ms(current_start, current_finish)
    return None if elapsed is None else total + elapsed


def _state_intervals(
    status_events: list[tuple[str, dict[str, object], datetime | None]],
    *,
    submitted_at: datetime | None,
    current_status: str,
) -> tuple[
    list[tuple[datetime, datetime]],
    list[tuple[datetime, datetime]],
    str,
] | None:
    if submitted_at is None:
        return None
    queue_intervals: list[tuple[datetime, datetime]] = []
    execution_intervals: list[tuple[datetime, datetime]] = []
    state = "queued"
    state_started = submitted_at
    for _event_type, payload, changed_at in status_events:
        if changed_at is None or _elapsed_ms(state_started, changed_at) is None:
            return None
        if state == "queued":
            queue_intervals.append((state_started, changed_at))
        elif state in {"materializing", "running"}:
            execution_intervals.append((state_started, changed_at))
        state = str(payload.get("to") or state)
        state_started = changed_at
        if state in _TERMINAL:
            break
    if state not in _TERMINAL:
        if current_status not in {"queued", "materializing", "running"} or state != current_status:
            return None
    return queue_intervals, execution_intervals, state


def _clip_to_windows(
    intervals: list[tuple[datetime, datetime]],
    windows: list[tuple[datetime, datetime]],
) -> list[tuple[datetime, datetime]]:
    return [
        (max(start, lower), min(finish, upper))
        for start, finish in intervals
        for lower, upper in windows
        if finish > lower and start < upper
    ]


def ticket_timings(connection: sqlite3.Connection, ticket_id: str) -> dict[str, object]:
    """Return durable phase durations for one validation ticket.

    ``None`` means the persisted timeline does not yet establish the full phase.
    A deterministic failure reuse is the sole zero-duration shortcut because no
    worker execution or cleanup is admitted for that ticket.
    """

    row = connection.execute(
        "SELECT status,created_at,updated_at FROM validation_tickets WHERE ticket_id=?",
        (ticket_id,),
    ).fetchone()
    if row is None:
        raise KeyError(ticket_id)
    status = str(row[0])
    ticket_created = _timestamp(row[1])

    event_columns = _table_columns(connection, "validation_ticket_events")
    if not {"ticket_id", "event_type", "payload_json", "created_at"}.issubset(
        event_columns
    ):
        events: list[tuple[str, dict[str, object], datetime | None]] = []
    else:
        order = "event_id" if "event_id" in event_columns else "created_at"
        events = [
            (str(event_type), _payload(raw_payload), _timestamp(created_at))
            for event_type, raw_payload, created_at in connection.execute(
                "SELECT event_type,payload_json,created_at "
                f"FROM validation_ticket_events WHERE ticket_id=? ORDER BY {order}",
                (ticket_id,),
            )
        ]

    submitted = next((event for event in events if event[0] == _SUBMITTED), None)
    submitted_payload: Mapping[str, object] = submitted[1] if submitted else {}
    submitted_at = submitted[2] if submitted else ticket_created
    admission_ms = _nonnegative_ms(submitted_payload.get("admissionMs"))

    status_events = [event for event in events if event[0] == _STATUS_CHANGED]
    terminal = next(
        (
            event
            for event in status_events
            if str(event[1].get("to") or "") in _TERMINAL
        ),
        None,
    )
    terminal_at = terminal[2] if terminal else None
    terminal_evidence = (
        terminal[1].get("evidence") if terminal and isinstance(terminal[1].get("evidence"), dict) else {}
    )
    failure_reuse = any(event[0] == _FAILURE_REUSED for event in events) or (
        bool(submitted_payload.get("originalFailureTicketId"))
        and str(terminal_evidence.get("phase") or "") == "failure_reuse"
    )
    if failure_reuse:
        return {
            "executionKind": "failure_reuse",
            "queuedMs": 0,
            "preparationMs": 0,
            "cargoMs": 0,
            "cleanupMs": 0,
            "admissionMs": admission_ms,
            "totalMs": admission_ms,
        }

    execution_kind = "executed" if status in _TERMINAL else "pending"
    state_intervals = _state_intervals(
        status_events,
        submitted_at=submitted_at,
        current_status=status,
    )
    if state_intervals is None:
        queued_ms = None
        execution_intervals: list[tuple[datetime, datetime]] = []
        execution_ms = None
    else:
        queue_intervals, execution_intervals, active_state = state_intervals
        queued_ms = None if active_state == "queued" else _union_ms(queue_intervals)
        execution_ms = (
            _union_ms(execution_intervals) if active_state in _TERMINAL else None
        )

    job_ids = {
        str(event[1]["jobId"])
        for event in events
        if event[0] in {_COPY_LINKED, _RUN_LINKED} and event[1].get("jobId")
    }
    intervals, incomplete_run = _run_intervals(
        connection, ticket_id=ticket_id, job_ids=job_ids
    )

    cargo_ms: int | None
    preparation_ms: int | None
    if execution_kind == "pending" or terminal_at is None or execution_ms is None:
        cargo_ms = None
        preparation_ms = None
    else:
        if incomplete_run:
            cargo_ms = None
        elif intervals:
            cargo_ms = _union_ms(_clip_to_windows(intervals, execution_intervals))
        else:
            running_intervals = []
            running_start: datetime | None = None
            for _event_type, payload, changed_at in status_events:
                if changed_at is None:
                    running_intervals = []
                    break
                if running_start is not None:
                    running_intervals.append((running_start, changed_at))
                    running_start = None
                if str(payload.get("to") or "") == "running":
                    running_start = changed_at
            cargo_ms = (
                _union_ms(running_intervals)
                if running_intervals
                else 0
            )
        preparation_ms = (
            max(0, execution_ms - cargo_ms)
            if execution_ms is not None and cargo_ms is not None
            else None
        )

    cleanup_events = [event for event in events if event[0] == _CLEANUP]
    completed_cleanup = next(
        (event for event in reversed(cleanup_events) if event[1].get("completed") is True),
        None,
    )
    if execution_kind == "pending":
        cleanup_ms = None
    elif completed_cleanup is not None:
        cleanup_ms = _nonnegative_ms(completed_cleanup[1].get("elapsedMs"))
    elif job_ids:
        cleanup_ms = None
    else:
        cleanup_ms = 0

    phases = (admission_ms, queued_ms, preparation_ms, cargo_ms, cleanup_ms)
    total_ms = sum(phases) if all(value is not None for value in phases) else None
    build_metrics = terminal_evidence.get("buildMetrics")
    detail = {}
    if isinstance(build_metrics, dict):
        durations = build_metrics.get("timings", {})
        detail = {
            "sourceSyncMs": round(build_metrics.get("source", {}).get("sourceSyncSeconds", 0) * 1000),
            "checkMs": round(durations.get("checkSeconds", 0) * 1000),
            "compileLinkMs": round(durations.get("compileLinkSeconds", 0) * 1000),
            "testExecutionMs": round(durations.get("testExecutionSeconds", 0) * 1000),
            "cacheDelta": build_metrics.get("cacheDelta", {}),
            "disk": build_metrics.get("disk", {}),
        }
    return {
        "executionKind": execution_kind,
        "queuedMs": queued_ms,
        "preparationMs": preparation_ms,
        "cargoMs": cargo_ms,
        "cleanupMs": cleanup_ms,
        "admissionMs": admission_ms,
        "totalMs": total_ms,
        **detail,
    }
