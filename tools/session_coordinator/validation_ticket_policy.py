"""Conservative, durable reuse of failures from identical immutable inputs."""

from __future__ import annotations

import hashlib
import json
import os
import re
import sys
from datetime import timedelta
from pathlib import Path
from typing import Mapping

from .models import utc_now, utc_text


FAILURE_REUSABLE_EVENT = "validation.ticket_failure_reusable"
FAILURE_REUSED_EVENT = "validation.ticket_failure_reused"
_PREPARATION_FAILURES = frozenset({
    "validation_copy_cargo_manifest_invalid",
    "validation_copy_cargo_manifest_path_missing",
    "validation_copy_cargo_target_missing",
    "validation_copy_cargo_target_module_outside_repository",
    "validation_copy_cargo_dependency_roots_invalid",
    "validation_copy_cargo_dependency_root_escape",
    "validation_copy_cargo_dependency_root_too_broad",
    "validation_copy_compile_time_resource_missing",
    "validation_copy_compile_time_resource_outside_repository",
    "validation_copy_compile_time_resource_unresolved",
    "validation_copy_compile_time_source_invalid",
    "validation_copy_compile_time_source_limit",
    "validation_copy_compile_time_source_too_large",
    "validation_copy_compile_time_source_total_too_large",
    "validation_copy_external_mount_conflict",
})
_TRANSIENT = re.compile(
    r"out of memory|not enough memory|memory allocation|no space left|disk full|"
    r"permission denied|access is denied|timed? out|timeout|could not resolve|"
    r"failed to download|failed to get successful HTTP|connection (?:reset|refused)|"
    r"internal compiler error|compiler unexpectedly panicked|signal:|os error|"
    r"process didn't exit successfully|failed to run custom build command|"
    r"linking with .+ failed", re.IGNORECASE,
)
_RUST_ERROR = re.compile(r"^error\[(E[0-9]{4})\]:", re.MULTILINE)
_CARGO_COMPILE_FAILED = re.compile(r"^error: could not compile .+ due to", re.MULTILINE)


def validator_identity() -> str:
    """Identify the implementation loaded at service creation, including helpers."""
    root = Path(__file__).parent
    digest = hashlib.sha256(b"validation-failure-policy-v2\0")
    for path in sorted(root.rglob("*.py")):
        if "tests" in path.relative_to(root).parts:
            continue
        digest.update(path.relative_to(root).as_posix().encode())
        digest.update(b"\0")
        digest.update(path.read_bytes())
    digest.update(sys.version.encode())
    digest.update(sys.platform.encode())
    return digest.hexdigest()


def execution_environment_identity() -> str:
    # Only a digest is persisted; inherited values can contain credentials.
    payload = json.dumps(sorted(os.environ.items()), separators=(",", ":"))
    return hashlib.sha256(payload.encode()).hexdigest()


def deterministic_failure(
    evidence: Mapping[str, object], *, compiler_identity_verified: bool = True
) -> str | None:
    if evidence.get("failureCacheExcluded") is True:
        return None
    code = str(evidence.get("errorCode") or "")
    if code in _PREPARATION_FAILURES:
        return code
    if not compiler_identity_verified:
        return None
    if evidence.get("phase") != "run" or evidence.get("exitCode") != 101:
        return None
    stderr = str(evidence.get("stderrTail") or "")
    stdout = str(evidence.get("stdoutTail") or "")
    if _TRANSIENT.search(stderr + "\n" + stdout):
        return None
    errors: set[str] = set()
    for line in stdout.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if not isinstance(message, dict) or message.get("reason") != "compiler-message":
            continue
        diagnostic = message.get("message")
        if not isinstance(diagnostic, dict) or diagnostic.get("level") != "error":
            continue
        diagnostic_code = diagnostic.get("code")
        if isinstance(diagnostic_code, dict):
            value = diagnostic_code.get("code")
            if isinstance(value, str) and re.fullmatch(r"E[0-9]{4}", value):
                errors.add(value)
    if _CARGO_COMPILE_FAILED.search(stderr):
        errors.update(_RUST_ERROR.findall(stderr))
        if errors:
            return "rustc:" + ",".join(sorted(errors))
    return None


def compiler_failure_evidence(stdout: str, stderr: str, exit_code: int) -> dict[str, object]:
    """Retain bounded, independently classifiable errors without hiding transients."""
    full = {"phase": "run", "exitCode": exit_code, "stdoutTail": stdout, "stderrTail": stderr}
    reason = deterministic_failure(full)
    compiler_lines = []
    for line in stderr.splitlines():
        if _RUST_ERROR.match(line) or _CARGO_COMPILE_FAILED.match(line):
            compiler_lines.append(line[:512])
            if len(compiler_lines) == 32:
                break
    tail = stderr[-4096:]
    if reason is not None and not deterministic_failure({**full, "stderrTail": tail, "stdoutTail": stdout[-4096:]}):
        # Select actual compiler output, not a caller-supplied cacheable flag.
        summary = next((line[:512] for line in reversed(stderr.splitlines()) if _CARGO_COMPILE_FAILED.match(line)), "")
        tail = "\n".join([*compiler_lines[:6], summary, tail[-2048:]])
    return {
        "stdoutTail": stdout[-4096:], "stderrTail": tail,
        "failureCacheExcluded": reason is None,
    }


class FailureReusePolicy:
    def __init__(self, *, max_entries: int = 1024, retention_days: int = 7):
        if max_entries < 1 or retention_days < 1:
            raise ValueError("failure cache limits must be positive")
        self.max_entries = max_entries
        self.retention_days = retention_days

    def lookup(self, connection, dedupe_key: str) -> tuple[str, str] | None:
        cutoff = utc_text(utc_now() - timedelta(days=self.retention_days))
        # Events preserve the original diagnosis after cache eviction or restart.
        row = connection.execute(
            """
            SELECT t.ticket_id, e.payload_json FROM (
                SELECT ticket_id, payload_json FROM validation_ticket_events
                WHERE event_type=? AND created_at>=?
                ORDER BY event_id DESC LIMIT ?
            ) AS e JOIN validation_tickets t ON t.ticket_id=e.ticket_id
            WHERE t.dedupe_key=? AND t.status='failed'
              AND NOT EXISTS (
                SELECT 1 FROM validation_tickets later
                WHERE later.dedupe_key=t.dedupe_key
                  AND later.status IN ('passed', 'snapshot_stale')
                  AND later.rowid>t.rowid
                  AND NOT EXISTS (
                    SELECT 1 FROM validation_ticket_events alias
                    WHERE alias.ticket_id=later.ticket_id AND alias.event_type=?
                  )
              )
            LIMIT 1
            """,
            (FAILURE_REUSABLE_EVENT, cutoff, self.max_entries, dedupe_key, FAILURE_REUSED_EVENT),
        ).fetchone()
        if row is None:
            return None
        return str(row["ticket_id"]), str(json.loads(row["payload_json"])["reason"])


def submission_details(connection, ticket_id: str) -> dict[str, object]:
    row = connection.execute(
        "SELECT payload_json FROM validation_ticket_events WHERE ticket_id=? "
        "AND event_type='validation.ticket_submitted' ORDER BY event_id LIMIT 1",
        (ticket_id,),
    ).fetchone()
    return json.loads(row[0]) if row is not None else {}


def reuse_blocker(original_id: str, reason: str) -> dict[str, object]:
    return {
        "code": "validation_deterministic_failure_reused",
        "message": "Identical immutable inputs already failed deterministically",
        "originalFailureTicketId": original_id,
        "reuseReason": reason,
        "repairCondition": "Change the inputs or toolchain, update the validator, or supply forceRerunReason",
    }


def terminal_diagnostic(connection, ticket_id: str) -> Mapping[str, object] | None:
    from .validation_copy_diagnostics import ticket_metadata_diagnostic

    metadata = ticket_metadata_diagnostic(connection, ticket_id)
    row = connection.execute(
        "SELECT payload_json FROM validation_ticket_events WHERE ticket_id=? "
        "AND event_type='validation.ticket_status_changed' ORDER BY event_id DESC LIMIT 1",
        (ticket_id,),
    ).fetchone()
    if row is None:
        return metadata or None
    payload = json.loads(row[0])
    if payload.get("to") not in {"passed", "failed", "snapshot_stale"}:
        return metadata or None
    return {**(payload.get("evidence") or {}), **metadata}
