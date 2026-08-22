from __future__ import annotations

import json
import os
import sqlite3
import threading
import uuid
from collections.abc import Callable, Iterable, Mapping
from dataclasses import dataclass
from pathlib import Path

from .cargo_jobs import target_identity, targets_overlap
from .models import utc_text


@dataclass(frozen=True, slots=True)
class DeletionEvidence:
    deletion_id: str
    trigger: str
    target_key: str
    target_dir: str
    owner_job_id: str
    owner_session_id: str
    before: dict[str, object]
    executor: dict[str, object]


@dataclass(frozen=True, slots=True)
class ValidationCopyOverlap:
    job_id: str
    status: str
    path_kind: str
    path: str

    @property
    def message(self) -> str:
        return (
            f"Validation copy {self.job_id} ({self.status}) owns "
            f"{self.path_kind} {self.path}"
        )


def overlapping_validation_copy(
    connection: sqlite3.Connection,
    target_key: str,
) -> ValidationCopyOverlap | None:
    rows = connection.execute(
        """SELECT job_id, status, job_root, target_root
           FROM validation_copies
           WHERE status <> 'removed'
           ORDER BY created_at, job_id"""
    ).fetchall()
    for row in rows:
        for path_kind in ("job_root", "target_root"):
            path = str(row[path_kind])
            if targets_overlap(target_key, target_identity(path)):
                return ValidationCopyOverlap(
                    job_id=str(row["job_id"]),
                    status=str(row["status"]),
                    path_kind=path_kind,
                    path=path,
                )
    return None


def record_validation_copy_overlap_denial(
    connection: sqlite3.Connection,
    *,
    trigger: str,
    target_key: str,
    target_dir: str,
    overlap: ValidationCopyOverlap,
) -> None:
    _insert_event(
        connection,
        "cleanup.validation_copy_overlap_denied",
        {
            "code": "validation_copy_overlap",
            "trigger": trigger,
            "target_key": target_key,
            "target_dir": target_dir,
            "validation_copy": {
                "job_id": overlap.job_id,
                "status": overlap.status,
                "path_kind": overlap.path_kind,
                "path": overlap.path,
            },
        },
    )


def begin_target_deletion(
    connection: sqlite3.Connection,
    *,
    trigger: str,
    target_key: str,
    target_dir: str,
    owner: Mapping[str, object],
    overlapping_jobs: Iterable[Mapping[str, object]],
    process_alive: Callable[[int], bool],
) -> DeletionEvidence:
    owner_job_id = str(owner["job_id"])
    jobs = tuple(
        _job_state(row, process_alive=process_alive)
        for row in overlapping_jobs
    )
    owner_state = next(
        (item for item in jobs if item["job_id"] == owner_job_id),
        _job_state(owner, process_alive=process_alive),
    )
    before = {
        "owner_job_id": owner_job_id,
        "target_exists": Path(target_dir).exists(),
        "job_status": owner_state["status"],
        "cleanup_status": owner_state["cleanup_status"],
        "pid": owner_state["pid"],
        "process_alive": owner_state["process_alive"],
        "overlapping_jobs": jobs,
    }
    evidence = DeletionEvidence(
        deletion_id=uuid.uuid4().hex,
        trigger=trigger,
        target_key=target_key,
        target_dir=target_dir,
        owner_job_id=owner_job_id,
        owner_session_id=str(owner["session_id"]),
        before=before,
        executor={
            "process_id": os.getpid(),
            "thread_name": threading.current_thread().name,
        },
    )
    _insert_event(
        connection,
        "cleanup.target_deletion_started",
        _payload(evidence, result="reserved", error=None),
    )
    return evidence


def complete_target_deletion(
    connection: sqlite3.Connection,
    evidence: DeletionEvidence,
    *,
    result: str,
    error: str | None,
) -> None:
    _insert_event(
        connection,
        "cleanup.target_deletion_completed",
        _payload(evidence, result=result, error=error),
    )


def interrupted_target_deletions(
    connection: sqlite3.Connection,
    reservations: Iterable[Mapping[str, object]],
) -> tuple[dict[str, object], ...]:
    rows = connection.execute(
        """
        SELECT event_type, payload_json FROM events
        WHERE event_type IN (
            'cleanup.target_deletion_started',
            'cleanup.target_deletion_completed'
        )
        ORDER BY event_id
        """
    ).fetchall()
    started: dict[str, dict[str, object]] = {}
    completed: set[str] = set()
    for row in rows:
        payload = json.loads(row["payload_json"])
        deletion_id = str(payload.get("deletion_id", ""))
        if not deletion_id:
            continue
        if row["event_type"] == "cleanup.target_deletion_started":
            started[deletion_id] = payload
        elif payload.get("result") not in {"failed", "failed_after_restart"}:
            completed.add(deletion_id)

    interrupted: list[dict[str, object]] = []
    for reservation in reservations:
        target_key = str(reservation["target_key"])
        evidence = next(
            (
                payload
                for deletion_id, payload in reversed(tuple(started.items()))
                if deletion_id not in completed
                and payload.get("target_key") == target_key
            ),
            None,
        )
        if evidence is None:
            continue
        interrupted.append(dict(evidence))
    return tuple(interrupted)


def complete_interrupted_target_deletion(
    connection: sqlite3.Connection,
    evidence: Mapping[str, object],
    *,
    result: str,
    error: str | None,
) -> None:
    recovered_payload = dict(evidence)
    recovered_payload.update(
        {
            "result": result,
            "error": error,
            "recovered": True,
            "recovery_executor": {
                "process_id": os.getpid(),
                "thread_name": threading.current_thread().name,
            },
        }
    )
    _insert_event(
        connection,
        "cleanup.target_deletion_completed",
        recovered_payload,
    )


def _job_state(
    row: Mapping[str, object],
    *,
    process_alive: Callable[[int], bool],
) -> dict[str, object]:
    pid_value = row["pid"]
    pid = int(pid_value) if pid_value is not None else None
    return {
        "job_id": str(row["job_id"]),
        "session_id": str(row["session_id"]),
        "status": str(row["status"]),
        "cleanup_status": str(row["cleanup_status"]),
        "pid": pid,
        "process_alive": bool(pid and process_alive(pid)),
    }


def _payload(
    evidence: DeletionEvidence,
    *,
    result: str,
    error: str | None,
) -> dict[str, object]:
    return {
        "deletion_id": evidence.deletion_id,
        "trigger": evidence.trigger,
        "target_key": evidence.target_key,
        "target_dir": evidence.target_dir,
        "owner_job_id": evidence.owner_job_id,
        "owner_session_id": evidence.owner_session_id,
        "before": evidence.before,
        "executor": evidence.executor,
        "result": result,
        "error": error,
    }


def _insert_event(
    connection: sqlite3.Connection,
    event_type: str,
    payload: dict[str, object],
) -> None:
    connection.execute(
        "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
        (event_type, json.dumps(payload, sort_keys=True), utc_text()),
    )
