"""Durable metadata-planning observations shared by copies and ticket receipts."""

from __future__ import annotations

import json
import sqlite3
from contextlib import nullcontext
from typing import Callable, ContextManager, Mapping

from .database import Database
from .models import CoordinatorError


def record_metadata_cache(
    database: Database,
    job_id: str,
    worker_id: str,
    observation: Mapping[str, object],
    mutation_gate: Callable[[], ContextManager[None]] | None = None,
) -> None:
    gate = mutation_gate() if mutation_gate is not None else nullcontext()
    with gate, database.transaction() as connection:
        row = connection.execute(
            "SELECT metadata_cache_json FROM validation_copies "
            "WHERE job_id=? AND status='planned' "
            "AND materialization_phase='closure_planning' "
            "AND materialization_worker_id=?",
            (job_id, worker_id),
        ).fetchone()
        if row is None:
            raise CoordinatorError(
                "validation_copy_materialization_state_lost",
                "Cargo validation copy changed state while metadata was planned",
            )
        try:
            previous = json.loads(str(row[0] or "{}"))
        except (TypeError, ValueError, json.JSONDecodeError):
            previous = {}
        attempts = (
            [dict(item) for item in previous if isinstance(item, Mapping)]
            if isinstance(previous, list)
            else [dict(previous)]
            if isinstance(previous, Mapping) and previous
            else []
        )
        attempts.append(dict(observation))
        connection.execute(
            "UPDATE validation_copies SET metadata_cache_json=? "
            "WHERE job_id=? AND status='planned' "
            "AND materialization_phase='closure_planning' "
            "AND materialization_worker_id=?",
            (json.dumps(attempts, sort_keys=True), job_id, worker_id),
        )


def ticket_metadata_diagnostic(
    connection: sqlite3.Connection, ticket_id: str
) -> dict[str, object]:
    rows = connection.execute(
        "SELECT copy.job_id, copy.metadata_cache_json "
        "FROM validation_ticket_events event JOIN validation_copies copy "
        "ON copy.job_id=json_extract(event.payload_json, '$.jobId') "
        "WHERE event.ticket_id=? AND event.event_type='validation.ticket_copy_linked' "
        "ORDER BY event.event_id",
        (ticket_id,),
    )
    attempts = {}
    for job_id, raw in rows:
        try:
            decoded = json.loads(raw or "{}")
        except (TypeError, ValueError, json.JSONDecodeError):
            decoded = {}
        observations = (
            [item for item in decoded if isinstance(item, Mapping)]
            if isinstance(decoded, list)
            else [decoded]
            if isinstance(decoded, Mapping)
            else []
        )
        for index, observation in enumerate(observations):
            if not observation:
                continue
            attempt_key = job_id if len(observations) == 1 else f"{job_id}:{index}"
            attempts[attempt_key] = {"jobId": job_id, **dict(observation)}
    if not attempts:
        return {}
    values = list(attempts.values())
    return {"metadataCache": values[-1], "metadataCacheAttempts": values}
