from __future__ import annotations

import hashlib
import json
import threading
from collections.abc import Callable
from datetime import UTC, datetime, timedelta
from sqlite3 import Connection
from typing import Any

from .database import Database
from .models import CoordinatorError, utc_text


MAX_COMMAND_RESPONSE_BYTES = 256 * 1024
COMMAND_REQUEST_RETENTION_DAYS = 7
MAX_TERMINAL_COMMAND_REQUESTS = 10_000
EPHEMERAL_REQUEST_RETENTION_DAYS = 1
MAX_EPHEMERAL_COMMAND_REQUESTS = 10_000
INTERNAL_COMMAND_ERROR_MESSAGE = "Coordinator command failed unexpectedly"


def unexpected_command_error(correlation_id: str) -> CoordinatorError:
    return CoordinatorError(
        "internal_error",
        INTERNAL_COMMAND_ERROR_MESSAGE,
        details={"correlationId": correlation_id},
    )


class CommandRequestJournal:
    """Durable idempotency boundary for the legacy command transport."""

    def __init__(self, database: Database):
        self.database = database
        self._deferred_failure_lock = threading.Lock()
        self._deferred_failures: dict[str, CoordinatorError] = {}

    @staticmethod
    def _fingerprint(command: str, arguments: dict[str, Any]) -> str:
        canonical = json.dumps(
            {"command": command, "arguments": arguments},
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        )
        return hashlib.sha256(canonical.encode("utf-8")).hexdigest()

    @staticmethod
    def _validate_request_id(request_id: str) -> str:
        normalized = request_id.strip().lower()
        if len(normalized) != 32 or any(character not in "0123456789abcdef" for character in normalized):
            raise CoordinatorError(
                "command_request_id_invalid",
                "Command request_id must be a 32-character lowercase hexadecimal identifier",
            )
        return normalized

    @staticmethod
    def _validate_retention_class(retention_class: str) -> str:
        if retention_class not in {"durable", "ephemeral"}:
            raise CoordinatorError(
                "command_request_retention_invalid",
                "Command request retention class must be durable or ephemeral",
            )
        return retention_class

    def execute(
        self,
        request_id: str,
        command: str,
        arguments: dict[str, Any],
        callback: Callable[[], dict[str, Any]],
        *,
        retention_class: str = "durable",
    ) -> dict[str, Any]:
        request_id = self._validate_request_id(request_id)
        retention_class = self._validate_retention_class(retention_class)
        fingerprint = self._fingerprint(command, arguments)
        existing = None
        now = utc_text()
        with self.database.transaction() as connection:
            existing = connection.execute(
                "SELECT * FROM command_requests WHERE request_id=?", (request_id,)
            ).fetchone()
            if existing is None:
                connection.execute(
                    """
                    INSERT INTO command_requests(
                        request_id, command, arguments_hash, status, received_at, accepted_at,
                        retention_class
                    ) VALUES (?, ?, ?, 'accepted', ?, ?, ?)
                    """,
                    (request_id, command, fingerprint, now, now, retention_class),
                )
            elif existing["command"] != command or existing["arguments_hash"] != fingerprint:
                raise CoordinatorError(
                    "command_request_conflict",
                    "request_id is already bound to a different command payload",
                    details={"requestId": request_id},
                )

        if existing is not None:
            return self._replay(existing)

        try:
            result = callback()
            if not isinstance(result, dict):
                raise CoordinatorError(
                    "invalid_response", "Coordinator command returned a non-object response"
                )
            response = dict(result)
            response["requestId"] = request_id
            response_json = self._bounded_response_json(response)
            completed_at = utc_text()
            with self.database.transaction() as connection:
                connection.execute(
                    """
                    UPDATE command_requests
                    SET status='completed', completed_at=?, response_json=?
                    WHERE request_id=? AND status='accepted'
                    """,
                    (
                        completed_at,
                        response_json,
                        request_id,
                    ),
                )
            return response
        except CoordinatorError as error:
            self._record_failure(request_id, error)
            raise
        except BaseException as error:
            self._record_failure(
                request_id,
                unexpected_command_error(request_id),
            )
            raise

    def execute_transactional(
        self,
        request_id: str,
        command: str,
        arguments: dict[str, Any],
        callback: Callable[
            [Connection], tuple[dict[str, Any], Callable[[], object] | None]
        ],
        *,
        retention_class: str = "durable",
    ) -> dict[str, Any]:
        """Commit command acceptance and its durable admission result together."""
        request_id = self._validate_request_id(request_id)
        retention_class = self._validate_retention_class(retention_class)
        fingerprint = self._fingerprint(command, arguments)
        now = utc_text()
        existing = None
        response: dict[str, Any] | None = None
        after_commit: Callable[[], object] | None = None
        failure: BaseException | None = None
        with self.database.transaction() as connection:
            existing = connection.execute(
                "SELECT * FROM command_requests WHERE request_id=?", (request_id,)
            ).fetchone()
            if existing is not None:
                if existing["command"] != command or existing["arguments_hash"] != fingerprint:
                    raise CoordinatorError(
                        "command_request_conflict",
                        "request_id is already bound to a different command payload",
                        details={"requestId": request_id},
                    )
            else:
                connection.execute(
                    """
                    INSERT INTO command_requests(
                        request_id, command, arguments_hash, status, received_at, accepted_at,
                        retention_class
                    ) VALUES (?, ?, ?, 'accepted', ?, ?, ?)
                    """,
                    (request_id, command, fingerprint, now, now, retention_class),
                )
                connection.execute("SAVEPOINT command_admission")
                try:
                    result, after_commit = callback(connection)
                    if not isinstance(result, dict):
                        raise CoordinatorError(
                            "invalid_response", "Coordinator command returned a non-object response"
                        )
                    response = dict(result)
                    response["requestId"] = request_id
                    response_json = self._bounded_response_json(response)
                except BaseException as error:
                    connection.execute("ROLLBACK TO command_admission")
                    connection.execute("RELEASE command_admission")
                    issue = (
                        error
                        if isinstance(error, CoordinatorError)
                        else unexpected_command_error(request_id)
                    )
                    connection.execute(
                        """
                        UPDATE command_requests
                        SET status='failed', completed_at=?, error_json=?
                        WHERE request_id=? AND status='accepted'
                        """,
                        (utc_text(), self._bounded_error(issue), request_id),
                    )
                    failure = error
                else:
                    connection.execute("RELEASE command_admission")
                    connection.execute(
                        """
                        UPDATE command_requests
                        SET status='completed', completed_at=?, response_json=?
                        WHERE request_id=? AND status='accepted'
                        """,
                        (
                            utc_text(),
                            response_json,
                            request_id,
                        ),
                    )

        if existing is not None:
            return self._replay(existing)
        if failure is not None:
            raise failure
        if response is None:
            raise CoordinatorError("invalid_response", "Command admission produced no response")
        if after_commit is not None:
            after_commit()
        return response

    def execute_accepted_transactionally(
        self,
        request_id: str,
        command: str,
        arguments: dict[str, Any],
        callback: Callable[
            [Connection], tuple[dict[str, Any], Callable[[], object] | None]
        ],
        *,
        retention_class: str = "durable",
        before_admission: Callable[[], object] | None = None,
    ) -> dict[str, Any]:
        """Expose accepted work, then commit its mutation and terminal result together."""
        request_id = self._validate_request_id(request_id)
        retention_class = self._validate_retention_class(retention_class)
        fingerprint = self._fingerprint(command, arguments)
        now = utc_text()
        existing = None
        with self.database.transaction() as connection:
            existing = connection.execute(
                "SELECT * FROM command_requests WHERE request_id=?", (request_id,)
            ).fetchone()
            if existing is None:
                connection.execute(
                    """
                    INSERT INTO command_requests(
                        request_id, command, arguments_hash, status, received_at, accepted_at,
                        retention_class
                    ) VALUES (?, ?, ?, 'accepted', ?, ?, ?)
                    """,
                    (request_id, command, fingerprint, now, now, retention_class),
                )
            elif existing["command"] != command or existing["arguments_hash"] != fingerprint:
                raise CoordinatorError(
                    "command_request_conflict",
                    "request_id is already bound to a different command payload",
                    details={"requestId": request_id},
                )

        if existing is not None:
            return self._replay(existing)

        response: dict[str, Any] | None = None
        after_commit: Callable[[], object] | None = None
        failure: BaseException | None = None
        terminal = None
        try:
            if before_admission is not None:
                before_admission()
            with self.database.transaction() as connection:
                current = connection.execute(
                    "SELECT * FROM command_requests WHERE request_id=?", (request_id,)
                ).fetchone()
                if current is None:
                    raise CoordinatorError(
                        "command_request_not_found",
                        f"Unknown command request {request_id}",
                        details={"requestId": request_id},
                    )
                if current["status"] != "accepted":
                    terminal = current
                else:
                    connection.execute("SAVEPOINT command_admission")
                    try:
                        result, after_commit = callback(connection)
                        if not isinstance(result, dict):
                            raise CoordinatorError(
                                "invalid_response", "Coordinator command returned a non-object response"
                            )
                        response = dict(result)
                        response["requestId"] = request_id
                        response_json = self._bounded_response_json(response)
                    except BaseException as error:
                        connection.execute("ROLLBACK TO command_admission")
                        connection.execute("RELEASE command_admission")
                        issue = (
                            error
                            if isinstance(error, CoordinatorError)
                            else unexpected_command_error(request_id)
                        )
                        connection.execute(
                            """
                            UPDATE command_requests
                            SET status='failed', completed_at=?, error_json=?
                            WHERE request_id=? AND status='accepted'
                            """,
                            (utc_text(), self._bounded_error(issue), request_id),
                        )
                        failure = error
                    else:
                        connection.execute("RELEASE command_admission")
                        connection.execute(
                            """
                            UPDATE command_requests
                            SET status='completed', completed_at=?, response_json=?
                            WHERE request_id=? AND status='accepted'
                            """,
                            (utc_text(), response_json, request_id),
                        )
        except CoordinatorError as error:
            self._record_or_defer_failure(request_id, error)
            raise
        except BaseException as error:
            self._record_or_defer_failure(
                request_id,
                unexpected_command_error(request_id),
            )
            raise

        if terminal is not None:
            return self._replay(terminal)
        if failure is not None:
            raise failure
        if response is None:
            raise CoordinatorError("invalid_response", "Command admission produced no response")
        if after_commit is not None:
            after_commit()
        return response

    def get(self, request_id: str) -> dict[str, Any]:
        request_id = self._validate_request_id(request_id)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM command_requests WHERE request_id=?", (request_id,)
            ).fetchone()
            start = connection.execute(
                "SELECT * FROM cargo_start_requests WHERE request_id=?", (request_id,)
            ).fetchone()
        if row is None:
            raise CoordinatorError(
                "command_request_not_found",
                f"Unknown command request {request_id}",
                details={"requestId": request_id},
            )
        payload: dict[str, Any] = {"request": self._request_dict(row)}
        if row["response_json"]:
            payload["result"] = json.loads(row["response_json"])
        if row["error_json"]:
            payload["error"] = json.loads(row["error_json"])
        if start is not None:
            payload["start"] = self._start_dict(start)
        return payload

    def reconcile_interrupted(self) -> tuple[str, ...]:
        """Terminalize requests whose handler disappeared before a durable result."""
        completed_at = utc_text()
        with self.database.transaction() as connection:
            request_ids = tuple(
                str(row["request_id"])
                for row in connection.execute(
                    """
                    SELECT request_id FROM command_requests
                    WHERE status='accepted' ORDER BY accepted_at, request_id
                    """
                )
            )
            for request_id in request_ids:
                issue = CoordinatorError(
                    "command_execution_interrupted",
                    "Coordinator restarted before the accepted command produced a durable result",
                    details={"requestId": request_id},
                )
                connection.execute(
                    """
                    UPDATE command_requests
                    SET status='failed', completed_at=?, error_json=?
                    WHERE request_id=? AND status='accepted'
                    """,
                    (completed_at, self._bounded_error(issue), request_id),
                )
        return request_ids

    def retry_deferred_failures(self) -> tuple[str, ...]:
        """Retry terminal writes that failed after a request was accepted.

        Only requests recorded by the failure path are retried. This avoids
        terminalizing a different accepted request whose callback is live while
        preserving startup reconciliation as the process-crash fallback.
        """
        with self._deferred_failure_lock:
            pending = tuple(self._deferred_failures.items())
        settled: list[str] = []
        for request_id, error in pending:
            try:
                self._record_failure(request_id, error)
            except Exception:
                continue
            with self._deferred_failure_lock:
                if self._deferred_failures.get(request_id) is error:
                    del self._deferred_failures[request_id]
                    settled.append(request_id)
        return tuple(settled)

    def prune(
        self,
        *,
        now: datetime | None = None,
        retention_days: int = COMMAND_REQUEST_RETENTION_DAYS,
        max_terminal: int = MAX_TERMINAL_COMMAND_REQUESTS,
        ephemeral_retention_days: int = EPHEMERAL_REQUEST_RETENTION_DAYS,
        max_ephemeral: int = MAX_EPHEMERAL_COMMAND_REQUESTS,
        batch_size: int = 256,
    ) -> int:
        if (
            retention_days < 1
            or max_terminal < 1
            or ephemeral_retention_days < 1
            or max_ephemeral < 1
            or batch_size < 1
        ):
            raise ValueError("Command request retention bounds must be positive")
        current = now or datetime.now(UTC)
        cutoff = utc_text(current - timedelta(days=retention_days))
        ephemeral_cutoff = utc_text(current - timedelta(days=ephemeral_retention_days))
        changed = 0
        with self.database.transaction() as connection:
            remaining = batch_size
            changed += connection.execute(
                """
                DELETE FROM command_requests
                WHERE request_id IN (
                    SELECT candidate.request_id FROM command_requests AS candidate
                    WHERE candidate.retention_class='ephemeral'
                      AND candidate.status IN ('completed', 'failed')
                      AND candidate.completed_at IS NOT NULL
                      AND candidate.completed_at<?
                      AND NOT EXISTS (
                          SELECT 1 FROM cargo_start_requests AS start
                          WHERE start.request_id=candidate.request_id
                      )
                    ORDER BY candidate.completed_at, candidate.request_id
                    LIMIT ?
                )
                """,
                (ephemeral_cutoff, remaining),
            ).rowcount
            remaining = batch_size - changed

            if remaining:
                changed += connection.execute(
                    """
                    DELETE FROM command_requests
                    WHERE request_id IN (
                        SELECT candidate.request_id
                        FROM command_requests AS candidate
                        WHERE candidate.retention_class='ephemeral'
                          AND candidate.status IN ('completed', 'failed')
                          AND candidate.completed_at IS NOT NULL
                          AND NOT EXISTS (
                              SELECT 1 FROM cargo_start_requests AS start
                              WHERE start.request_id=candidate.request_id
                          )
                        ORDER BY candidate.completed_at DESC, candidate.request_id DESC
                        LIMIT ? OFFSET ?
                    )
                    """,
                    (remaining, max_ephemeral),
                ).rowcount
                remaining = batch_size - changed

            compacted_at = utc_text(current)

            def compact(rows) -> None:
                nonlocal changed, remaining
                for row in rows:
                    request_id = str(row["request_id"])
                    if row["status"] == "completed":
                        payload = self._completion_tombstone(request_id, row["response_json"])
                        cursor = connection.execute(
                            """
                            UPDATE command_requests
                            SET response_json=?, payload_compacted_at=?
                            WHERE request_id=? AND payload_compacted_at IS NULL
                            """,
                            (payload, compacted_at, request_id),
                        )
                    else:
                        payload = self._failure_tombstone(request_id, row["error_json"])
                        cursor = connection.execute(
                            """
                            UPDATE command_requests
                            SET error_json=?, payload_compacted_at=?
                            WHERE request_id=? AND payload_compacted_at IS NULL
                            """,
                            (payload, compacted_at, request_id),
                        )
                    changed += cursor.rowcount
                    remaining = batch_size - changed

            if remaining:
                compact(
                    connection.execute(
                        """
                        SELECT * FROM command_requests
                        WHERE retention_class='durable'
                          AND status IN ('completed', 'failed')
                          AND payload_compacted_at IS NULL
                          AND completed_at IS NOT NULL AND completed_at<?
                          AND NOT EXISTS (
                              SELECT 1 FROM cargo_start_requests AS start
                              WHERE start.request_id=command_requests.request_id
                          )
                        ORDER BY completed_at, request_id
                        LIMIT ?
                        """,
                        (cutoff, remaining),
                    ).fetchall()
                )

            if remaining:
                compact(
                    connection.execute(
                        """
                        SELECT * FROM command_requests
                        WHERE retention_class='durable'
                          AND status IN ('completed', 'failed')
                          AND payload_compacted_at IS NULL
                          AND completed_at IS NOT NULL
                          AND NOT EXISTS (
                              SELECT 1 FROM cargo_start_requests AS start
                              WHERE start.request_id=command_requests.request_id
                          )
                        ORDER BY completed_at DESC, request_id DESC
                        LIMIT ? OFFSET ?
                        """,
                        (remaining, max_terminal),
                    ).fetchall()
                )
        return changed

    def _record_or_defer_failure(self, request_id: str, error: CoordinatorError) -> None:
        try:
            self._record_failure(request_id, error)
        except Exception:
            with self._deferred_failure_lock:
                self._deferred_failures[request_id] = error
        else:
            with self._deferred_failure_lock:
                self._deferred_failures.pop(request_id, None)

    def _record_failure(self, request_id: str, error: CoordinatorError) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE command_requests
                SET status='failed', completed_at=?, error_json=?
                WHERE request_id=? AND status='accepted'
                """,
                (utc_text(), self._bounded_error(error), request_id),
            )

    @staticmethod
    def _bounded_response_json(response: dict[str, Any]) -> str:
        serialized = json.dumps(response, ensure_ascii=False, sort_keys=True)
        encoded = serialized.encode("utf-8")
        if len(encoded) <= MAX_COMMAND_RESPONSE_BYTES:
            return serialized
        tombstone: dict[str, Any] = {
            "requestId": response["requestId"],
            "requestStatus": "completed",
            "responseOmitted": True,
            "responseBytes": len(encoded),
            "responseSha256": hashlib.sha256(encoded).hexdigest(),
        }
        return json.dumps(tombstone, sort_keys=True)

    @staticmethod
    def _bounded_error(error: CoordinatorError) -> str:
        serialized = json.dumps(error.to_dict(), ensure_ascii=False, sort_keys=True)
        encoded = serialized.encode("utf-8")
        if len(encoded) <= MAX_COMMAND_RESPONSE_BYTES:
            return serialized
        compact = {
            "code": error.code,
            "message": error.message[:2048],
            "details": {
                "detailsOmitted": True,
                "errorBytes": len(encoded),
                "errorSha256": hashlib.sha256(encoded).hexdigest(),
            },
        }
        return json.dumps(compact, ensure_ascii=False, sort_keys=True)

    @staticmethod
    def _completion_tombstone(request_id: str, response_json: str | None) -> str:
        if response_json:
            try:
                current = json.loads(response_json)
            except (TypeError, ValueError):
                current = None
            if isinstance(current, dict) and (
                current.get("responseExpired") is True or current.get("responseOmitted") is True
            ):
                return response_json
            encoded = response_json.encode("utf-8")
        else:
            encoded = b""
        tombstone = {
            "requestId": request_id,
            "requestStatus": "completed",
            "responseExpired": True,
            "responseBytes": len(encoded),
            "responseSha256": hashlib.sha256(encoded).hexdigest(),
        }
        return json.dumps(tombstone, sort_keys=True)

    @staticmethod
    def _failure_tombstone(request_id: str, error_json: str | None) -> str:
        current: dict[str, Any] = {}
        if error_json:
            try:
                parsed = json.loads(error_json)
            except (TypeError, ValueError):
                parsed = None
            if isinstance(parsed, dict):
                current = parsed
                details = current.get("details")
                if isinstance(details, dict):
                    if details.get("errorExpired") is True:
                        return error_json
                    if (
                        details.get("detailsOmitted") is True
                        and isinstance(details.get("errorSha256"), str)
                        and details["errorSha256"]
                    ):
                        return error_json
        encoded = (error_json or "").encode("utf-8")
        tombstone = {
            "code": str(current.get("code", "command_failed")),
            "message": "Command failure details expired from the bounded request journal",
            "details": {
                "requestId": request_id,
                "errorExpired": True,
                "errorBytes": len(encoded),
                "errorSha256": hashlib.sha256(encoded).hexdigest(),
            },
        }
        return json.dumps(tombstone, sort_keys=True)

    @classmethod
    def _replay(cls, row) -> dict[str, Any]:
        if row["status"] == "completed" and row["response_json"]:
            response = json.loads(row["response_json"])
            if isinstance(response, dict):
                return response
        if row["status"] == "failed" and row["error_json"]:
            issue = json.loads(row["error_json"])
            raise CoordinatorError(
                str(issue.get("code", "command_failed")),
                str(issue.get("message", "Command request failed")),
                details=issue.get("details") or {},
            )
        return {"requestId": row["request_id"], "requestStatus": "accepted"}

    @staticmethod
    def _request_dict(row) -> dict[str, Any]:
        return {
            "requestId": row["request_id"],
            "command": row["command"],
            "status": row["status"],
            "receivedAt": row["received_at"],
            "acceptedAt": row["accepted_at"],
            "completedAt": row["completed_at"],
        }

    @staticmethod
    def _start_dict(row) -> dict[str, Any]:
        return {
            "requestId": row["request_id"],
            "reservationId": row["reservation_id"],
            "jobId": row["job_id"],
            "sessionId": row["session_id"],
            "status": row["status"],
            "acknowledgedAt": row["acknowledged_at"],
            "deadlineAt": row["deadline_at"],
            "runId": row["run_id"],
            "errorCode": row["error_code"],
            "errorMessage": row["error_message"],
            "completedAt": row["completed_at"],
        }
