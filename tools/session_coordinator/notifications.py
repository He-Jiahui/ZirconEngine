from __future__ import annotations

import hashlib
import json
import re
import subprocess
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

from .database import Database
from .models import CoordinatorError, utc_text


_URL = re.compile(r"https://[^\s\"']+", re.IGNORECASE)
_KEY = re.compile(r"(?i)(?:key|webhook|token)\s*[=:]\s*[^\s,;]+")
_MODULE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")


@dataclass(frozen=True, slots=True)
class NotificationAttemptRecord:
    notification_attempt_id: str
    commit_sha: str
    status: str
    attempted_at: str
    completed_at: str | None
    exit_code: int | None
    provider_errcode: str | None
    sanitized_error: str | None


class WeComNotificationService:
    """Reserve then perform exactly one credential-free WeCom script call."""

    def __init__(
        self,
        database: Database,
        *,
        script_path: str | Path | None = None,
        runner: Callable[[list[str]], subprocess.CompletedProcess[str]] | None = None,
    ):
        self.database = database
        self.script_path = Path(script_path or (
            Path.home()
            / ".codex/skills/wecom-push-message/scripts/send-wecom-message.ps1"
        )).resolve()
        self.runner = runner or self._run

    @staticmethod
    def format_message(
        *,
        module: str,
        summary: str,
        commit_time: str,
        shortstat: str,
        commit_content: str,
    ) -> str:
        normalized_module = module.strip()
        if not _MODULE.fullmatch(normalized_module):
            raise CoordinatorError(
                "notification_module_invalid",
                "Notification module must be a safe plan-folder name",
            )
        values = (summary, commit_time, shortstat, commit_content)
        if any(not value.strip() or "\n" in value or "\r" in value for value in values):
            raise CoordinatorError(
                "notification_content_invalid",
                "Notification fields must be non-empty single lines",
            )
        return "\n".join(
            (
                f"核心内容摘要：【{normalized_module}】{summary.strip()}",
                f"提交时间：{commit_time.strip()}",
                f"修改情况统计：{shortstat.strip()}",
                f"提交的commit内容：{commit_content.strip()}",
            )
        )

    def notify_once(
        self,
        *,
        commit_sha: str,
        message: str,
        run_id: str | None = None,
        topology_version_id: str | None = None,
        node_id: str | None = None,
        action_id: str | None = None,
    ) -> NotificationAttemptRecord:
        attempt_id = uuid.uuid4().hex
        attempted_at = utc_text()
        message_hash = hashlib.sha256(message.encode("utf-8")).hexdigest()
        try:
            with self.database.transaction() as connection:
                connection.execute(
                    """INSERT INTO notification_attempts(
                           notification_attempt_id, run_id, topology_version_id,
                           node_id, action_id, commit_sha, channel, status,
                           message_hash, attempted_at
                       ) VALUES (?, ?, ?, ?, ?, ?, 'wecom', 'reserved', ?, ?)""",
                    (
                        attempt_id,
                        run_id,
                        topology_version_id,
                        node_id,
                        action_id,
                        commit_sha,
                        message_hash,
                        attempted_at,
                    ),
                )
        except Exception as error:
            try:
                with self.database.connect() as connection:
                    existing = connection.execute(
                        """SELECT * FROM notification_attempts
                           WHERE commit_sha=? AND channel='wecom'""",
                        (commit_sha,),
                    ).fetchone()
                if existing is not None:
                    return self._record(existing)
            except Exception:
                pass
            return NotificationAttemptRecord(
                attempt_id,
                commit_sha,
                "unknown",
                attempted_at,
                attempted_at,
                None,
                None,
                self._sanitize(str(error)),
            )

        command = [
            "powershell",
            "-NoProfile",
            "-File",
            str(self.script_path),
            "-Message",
            message,
            "-Format",
            "text",
        ]
        try:
            result = self.runner(command)
            status = "succeeded" if result.returncode == 0 else "failed"
            provider_errcode = self._provider_errcode(result.stdout, result.stderr)
            error_text = None if status == "succeeded" else self._sanitize(result.stderr or result.stdout)
            exit_code = int(result.returncode)
        except Exception as error:
            status = "unknown"
            provider_errcode = None
            error_text = self._sanitize(str(error))
            exit_code = None
        completed_at = utc_text()
        try:
            with self.database.transaction() as connection:
                connection.execute(
                    """UPDATE notification_attempts
                       SET status=?, completed_at=?, exit_code=?, provider_errcode=?,
                           sanitized_error=?
                       WHERE notification_attempt_id=? AND status='reserved'""",
                    (
                        status,
                        completed_at,
                        exit_code,
                        provider_errcode,
                        error_text,
                        attempt_id,
                    ),
                )
                row = connection.execute(
                    "SELECT * FROM notification_attempts WHERE notification_attempt_id=?",
                    (attempt_id,),
                ).fetchone()
            return self._record(row)
        except Exception as error:
            return NotificationAttemptRecord(
                attempt_id,
                commit_sha,
                "unknown",
                attempted_at,
                completed_at,
                exit_code,
                provider_errcode,
                self._sanitize(str(error)),
            )

    def record_post_commit_failure(
        self,
        *,
        commit_sha: str,
        error: Exception,
        run_id: str | None = None,
        topology_version_id: str | None = None,
        node_id: str | None = None,
        action_id: str | None = None,
    ) -> NotificationAttemptRecord:
        """Persist a non-delivery outcome without ever re-opening a committed milestone."""
        attempt_id = uuid.uuid4().hex
        completed_at = utc_text()
        sanitized_error = self._sanitize(
            f"post-commit notification preparation failed: {error}"
        )
        try:
            with self.database.transaction() as connection:
                existing = connection.execute(
                    """SELECT * FROM notification_attempts
                       WHERE commit_sha=? AND channel='wecom'""",
                    (commit_sha,),
                ).fetchone()
                if existing is not None:
                    return self._record(existing)
                connection.execute(
                    """INSERT INTO notification_attempts(
                           notification_attempt_id, run_id, topology_version_id,
                           node_id, action_id, commit_sha, channel, status,
                           message_hash, attempted_at, completed_at, sanitized_error
                       ) VALUES (?, ?, ?, ?, ?, ?, 'wecom', 'unknown', ?, ?, ?, ?)""",
                    (
                        attempt_id,
                        run_id,
                        topology_version_id,
                        node_id,
                        action_id,
                        commit_sha,
                        hashlib.sha256(
                            b"post-commit notification preparation failure"
                        ).hexdigest(),
                        completed_at,
                        completed_at,
                        sanitized_error,
                    ),
                )
                row = connection.execute(
                    "SELECT * FROM notification_attempts WHERE notification_attempt_id=?",
                    (attempt_id,),
                ).fetchone()
            return self._record(row)
        except Exception:
            try:
                with self.database.connect() as connection:
                    existing = connection.execute(
                        """SELECT * FROM notification_attempts
                           WHERE commit_sha=? AND channel='wecom'""",
                        (commit_sha,),
                    ).fetchone()
                if existing is not None:
                    return self._record(existing)
            except Exception:
                pass
            return NotificationAttemptRecord(
                attempt_id,
                commit_sha,
                "unknown",
                completed_at,
                completed_at,
                None,
                None,
                sanitized_error,
            )

    def recover_reserved(self) -> tuple[str, ...]:
        """Fail closed after a crash: delivery may have happened, so never retry."""
        now = utc_text()
        with self.database.transaction() as connection:
            rows = connection.execute(
                """SELECT notification_attempt_id FROM notification_attempts
                   WHERE status='reserved' ORDER BY attempted_at"""
            ).fetchall()
            connection.execute(
                """UPDATE notification_attempts
                   SET status='unknown', completed_at=?,
                       sanitized_error='service restarted after delivery reservation'
                   WHERE status='reserved'""",
                (now,),
            )
        return tuple(row[0] for row in rows)

    @staticmethod
    def _run(command: list[str]) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            command, check=False, capture_output=True, text=True, timeout=45
        )

    @staticmethod
    def _provider_errcode(stdout: str | None, stderr: str | None) -> str | None:
        for raw in (stdout or "", stderr or ""):
            for line in reversed(raw.splitlines()):
                try:
                    value = json.loads(line)
                except json.JSONDecodeError:
                    match = re.search(r"errcode[=:](?P<code>-?\d+)", line, re.IGNORECASE)
                    if match:
                        return match.group("code")
                    continue
                if isinstance(value, dict) and "errcode" in value:
                    return str(value["errcode"])
        return None

    @staticmethod
    def _sanitize(value: str) -> str:
        sanitized = _URL.sub("[redacted-url]", value)
        sanitized = _KEY.sub("[redacted-secret]", sanitized)
        return sanitized.strip()[:1000] or "notification command failed"

    @staticmethod
    def _record(row) -> NotificationAttemptRecord:
        return NotificationAttemptRecord(
            notification_attempt_id=row["notification_attempt_id"],
            commit_sha=row["commit_sha"],
            status=row["status"],
            attempted_at=row["attempted_at"],
            completed_at=row["completed_at"],
            exit_code=row["exit_code"],
            provider_errcode=row["provider_errcode"],
            sanitized_error=row["sanitized_error"],
        )
