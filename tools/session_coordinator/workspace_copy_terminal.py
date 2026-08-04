from __future__ import annotations

import json
import subprocess
from contextlib import nullcontext
from dataclasses import dataclass
from typing import Callable, ContextManager

from .database import Database
from .models import CoordinatorError, utc_text

_CAPTURED_STREAM_CHARACTER_LIMIT = 65_536


@dataclass(frozen=True, slots=True)
class ValidationRunEvidence:
    run_id: str
    job_id: str
    command: tuple[str, ...]
    exit_code: int
    stdout: str
    stderr: str

    def to_dict(self) -> dict[str, object]:
        return {
            "run_id": self.run_id,
            "job_id": self.job_id,
            "command": list(self.command),
            "exit_code": self.exit_code,
            "stdout": self.stdout,
            "stderr": self.stderr,
        }


class ValidationCopyTerminalLifecycle:
    def __init__(
        self,
        database: Database,
        mutation_gate: Callable[[], ContextManager[None]] | None,
    ) -> None:
        self._database = database
        self._mutation_gate = mutation_gate

    def collect(self, process: subprocess.Popen[str]) -> tuple[int, str, str]:
        stdout_full, stderr_full = process.communicate()
        if process.returncode is None:
            raise CoordinatorError(
                "validation_copy_process_not_terminal",
                "Validation process communicate returned without a terminal exit code",
            )
        return (
            int(process.returncode),
            self._capture_stream_tail(stdout_full),
            self._capture_stream_tail(stderr_full),
        )

    def latest_for_job(
        self, *, session_id: str, job_id: str
    ) -> ValidationRunEvidence | None:
        with self._database.connect() as connection:
            row = connection.execute(
                """SELECT run_id, job_id, command_json, exit_code,
                          stdout_text, stderr_text
                   FROM validation_copy_runs
                   WHERE session_id = ? AND job_id = ?
                   ORDER BY completed_at DESC, run_id DESC
                   LIMIT 1""",
                (session_id, job_id),
            ).fetchone()
        if row is None:
            return None
        try:
            command = json.loads(str(row["command_json"]))
        except (TypeError, ValueError) as error:
            raise CoordinatorError(
                "validation_copy_terminal_evidence_invalid",
                "Validation-copy terminal evidence contains invalid command JSON",
                details={"runId": str(row["run_id"])},
            ) from error
        if not isinstance(command, list) or not all(
            isinstance(part, str) for part in command
        ):
            raise CoordinatorError(
                "validation_copy_terminal_evidence_invalid",
                "Validation-copy terminal evidence command must be a string array",
                details={"runId": str(row["run_id"])},
            )
        return ValidationRunEvidence(
            run_id=str(row["run_id"]),
            job_id=str(row["job_id"]),
            command=tuple(command),
            exit_code=int(row["exit_code"]),
            stdout=str(row["stdout_text"]),
            stderr=str(row["stderr_text"]),
        )

    def persist(
        self,
        *,
        run_id: str,
        job_id: str,
        session_id: str,
        command: tuple[str, ...],
        exit_code: int,
        stdout: str,
        stderr: str,
        started_at: str,
    ) -> ValidationRunEvidence:
        gate = (
            self._mutation_gate()
            if self._mutation_gate is not None
            else nullcontext()
        )
        with gate, self._database.transaction() as connection:
            connection.execute(
                """INSERT INTO validation_copy_runs(
                       run_id, job_id, session_id, command_json, exit_code,
                       stdout_text, stderr_text, started_at, completed_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    run_id,
                    job_id,
                    session_id,
                    json.dumps(command),
                    exit_code,
                    stdout,
                    stderr,
                    started_at,
                    utc_text(),
                ),
            )
        return ValidationRunEvidence(
            run_id, job_id, command, exit_code, stdout, stderr
        )

    def finalize_success(self, job_id: str) -> None:
        gate = (
            self._mutation_gate()
            if self._mutation_gate is not None
            else nullcontext()
        )
        with gate, self._database.transaction() as connection:
            cursor = connection.execute(
                "UPDATE validation_copies SET status = 'materialized', run_pid = NULL "
                "WHERE job_id = ? AND status = 'running'",
                (job_id,),
            )
            if cursor.rowcount != 1:
                raise CoordinatorError(
                    "validation_copy_terminal_state_changed",
                    "Validation copy changed state before terminal completion",
                )

    def notify_completion(
        self, completion_hook: Callable[[str], None] | None, run_id: str
    ) -> None:
        if completion_hook is None:
            return
        try:
            completion_hook(run_id)
        except BaseException as error:
            raise CoordinatorError(
                "validation_copy_completion_hook_failed",
                "Validation evidence was persisted, but its completion hook failed",
                details={"runId": run_id},
            ) from error

    def preserve_completion_failure(
        self,
        *,
        error: BaseException,
        run_id: str,
        job_id: str,
        session_id: str,
    ) -> None:
        if (
            not isinstance(error, CoordinatorError)
            or error.code != "validation_copy_completion_hook_failed"
        ):
            return
        try:
            self._record_completion_failure(
                run_id=run_id,
                job_id=job_id,
                session_id=session_id,
                error_code=error.code,
            )
        except BaseException:
            # The typed hook error remains authoritative. The caller still
            # suppresses cleanup, preserving the immutable source for recovery.
            return

    def _record_completion_failure(
        self,
        *,
        run_id: str,
        job_id: str,
        session_id: str,
        error_code: str,
    ) -> None:
        gate = (
            self._mutation_gate()
            if self._mutation_gate is not None
            else nullcontext()
        )
        with gate, self._database.transaction() as connection:
            connection.execute(
                "UPDATE validation_copies SET status = 'failed', run_pid = NULL "
                "WHERE job_id = ? AND status = 'running'",
                (job_id,),
            )
            connection.execute(
                "INSERT INTO events(session_id, event_type, payload_json, created_at) "
                "VALUES (?, ?, ?, ?)",
                (
                    session_id,
                    "validation_copy.completion_hook_failed",
                    json.dumps(
                        {
                            "errorCode": error_code,
                            "jobId": job_id,
                            "runId": run_id,
                        },
                        sort_keys=True,
                    ),
                    utc_text(),
                ),
            )

    def restore_after_failure(
        self,
        job_id: str,
        *,
        process_started: bool,
        evidence_persisted: bool,
    ) -> None:
        if evidence_persisted:
            return
        status = "failed" if process_started else "materialized"
        gate = (
            self._mutation_gate()
            if self._mutation_gate is not None
            else nullcontext()
        )
        try:
            with gate, self._database.transaction() as connection:
                connection.execute(
                    "UPDATE validation_copies SET status = ?, run_pid = NULL "
                    "WHERE job_id = ? AND status = 'running'",
                    (status, job_id),
                )
        except BaseException:
            # Keep the original terminal-evidence failure authoritative and
            # preserve the materialized copy for coordinator recovery.
            return

    @staticmethod
    def _capture_stream_tail(value: str | None) -> str:
        return (value or "")[-_CAPTURED_STREAM_CHARACTER_LIMIT:]
