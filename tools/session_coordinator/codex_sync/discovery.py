from __future__ import annotations

import hashlib
import json
import os
import re
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from .models import (
    CodexDiscoveredSession,
    CodexDiscoveryDiagnostic,
    CodexDiscoveryResult,
    CodexLifecycleEvent,
    CodexSessionState,
    CodexSourceLocation,
    CodexSourceRevision,
)

MAX_ROLLOUT_FILES = 10_000
MAX_SESSION_META_BYTES = 1024 * 1024
MAX_TAIL_BYTES = 64 * 1024
MAX_PATH_TEXT = 4096
MAX_ID_TEXT = 160
MAX_METADATA_TEXT = 256

_SAFE_ID = re.compile(r"^[A-Za-z0-9._:-]+$")
_SAFE_METADATA = re.compile(r"^[A-Za-z0-9][A-Za-z0-9 ._:-]*$")
_TAIL_EVENT_MAP = {
    "task_started": CodexLifecycleEvent.TASK_STARTED,
    "task_complete": CodexLifecycleEvent.TASK_COMPLETED,
    "task_completed": CodexLifecycleEvent.TASK_COMPLETED,
    "turn_complete": CodexLifecycleEvent.TASK_COMPLETED,
    "turn_completed": CodexLifecycleEvent.TASK_COMPLETED,
    "turn_aborted": CodexLifecycleEvent.TURN_ABORTED,
    "turn_cancelled": CodexLifecycleEvent.TURN_ABORTED,
}


class CodexSessionDiscovery:
    """Discover bounded Codex rollout metadata without retaining conversation data."""

    def __init__(
        self,
        codex_home: str | Path,
        repo_root: str | Path,
        *,
        max_files: int = MAX_ROLLOUT_FILES,
        max_meta_bytes: int = MAX_SESSION_META_BYTES,
        max_tail_bytes: int = MAX_TAIL_BYTES,
    ):
        self.codex_home = Path(codex_home).resolve()
        self.repo_root = Path(repo_root).resolve()
        self.max_files = max_files
        self.max_meta_bytes = max_meta_bytes
        self.max_tail_bytes = max_tail_bytes

    def discover(self) -> CodexDiscoveryResult:
        candidates: list[tuple[Path, CodexSourceLocation]] = []
        diagnostics: list[CodexDiscoveryDiagnostic] = []
        membership_complete = True
        for root, location in (
            (self.codex_home / "sessions", CodexSourceLocation.ACTIVE),
            (self.codex_home / "archived_sessions", CodexSourceLocation.ARCHIVED),
        ):
            if not root.exists():
                continue
            try:
                for path in root.rglob("rollout-*.jsonl"):
                    if path.is_file():
                        candidates.append((path, location))
                        if len(candidates) > self.max_files:
                            membership_complete = False
                            diagnostics.append(
                                CodexDiscoveryDiagnostic(
                                    code="codex_rollout_limit_exceeded",
                                    source_path=str(root),
                                )
                            )
                            break
            except OSError:
                membership_complete = False
                diagnostics.append(
                    CodexDiscoveryDiagnostic(
                        code="codex_rollout_membership_unreadable",
                        source_path=str(root),
                    )
                )
            if len(candidates) > self.max_files:
                break

        candidates = sorted(candidates, key=lambda item: os.path.normcase(str(item[0])))[: self.max_files]
        by_thread: dict[str, CodexDiscoveredSession] = {}
        for path, location in candidates:
            discovered, diagnostic = self._discover_one(path, location)
            if diagnostic is not None:
                diagnostics.append(diagnostic)
            if discovered is None:
                continue
            previous = by_thread.get(discovered.thread_id)
            if previous is None or self._preferred(discovered, previous):
                by_thread[discovered.thread_id] = discovered

        sessions = tuple(sorted(by_thread.values(), key=lambda item: item.thread_id))
        revision_material = "\n".join(
            f"{item.thread_id}\t{item.source_revision.path}\t"
            f"{item.source_revision.size}\t{item.source_revision.mtime_ns}"
            for item in sessions
        ).encode("utf-8")
        return CodexDiscoveryResult(
            sessions=sessions,
            diagnostics=tuple(diagnostics),
            membership_complete=membership_complete,
            scanned_count=len(candidates),
            source_revision=hashlib.sha256(revision_material).hexdigest(),
        )

    @staticmethod
    def _preferred(candidate: CodexDiscoveredSession, previous: CodexDiscoveredSession) -> bool:
        if candidate.source_location is CodexSourceLocation.ARCHIVED:
            return previous.source_location is not CodexSourceLocation.ARCHIVED
        if previous.source_location is CodexSourceLocation.ARCHIVED:
            return False
        return candidate.source_revision.mtime_ns >= previous.source_revision.mtime_ns

    def _discover_one(
        self, path: Path, location: CodexSourceLocation
    ) -> tuple[CodexDiscoveredSession | None, CodexDiscoveryDiagnostic | None]:
        display_path = str(path)
        try:
            resolved = path.resolve(strict=True)
            if not self._inside(resolved, self.codex_home):
                return None, CodexDiscoveryDiagnostic("codex_rollout_path_outside_home", display_path)
            stat = resolved.stat()
            with resolved.open("rb") as handle:
                raw_meta = handle.readline(self.max_meta_bytes + 1)
            if len(raw_meta) > self.max_meta_bytes or not raw_meta.endswith(b"\n"):
                return None, CodexDiscoveryDiagnostic("codex_session_meta_oversized", display_path)
            meta_record = json.loads(raw_meta.decode("utf-8"))
            meta = self._session_meta(meta_record)
            if meta is None:
                return None, CodexDiscoveryDiagnostic("codex_session_meta_invalid", display_path)
            cwd = Path(meta["cwd"]).resolve(strict=False)
            if not self._inside(cwd, self.repo_root):
                return None, None
            lifecycle, turn_id, event_timestamp = self._latest_lifecycle(resolved, stat.st_size)
        except (OSError, UnicodeError, json.JSONDecodeError, ValueError, TypeError):
            return None, CodexDiscoveryDiagnostic("codex_rollout_unreadable", display_path)

        first_seen_at = self._timestamp(meta.get("timestamp")) or self._mtime_text(stat.st_mtime)
        last_activity_at = event_timestamp or self._mtime_text(stat.st_mtime)
        if location is CodexSourceLocation.ARCHIVED:
            state = CodexSessionState.ARCHIVED
        elif lifecycle is CodexLifecycleEvent.TASK_STARTED:
            state = CodexSessionState.ACTIVE
        else:
            state = CodexSessionState.IDLE
        revision = CodexSourceRevision(
            path=str(resolved),
            size=stat.st_size,
            mtime_ns=stat.st_mtime_ns,
        )
        return (
            CodexDiscoveredSession(
                thread_id=meta["thread_id"],
                rollout_path=str(resolved),
                source_location=location,
                state=state,
                cwd=str(cwd),
                originator=meta.get("originator"),
                cli_version=meta.get("cli_version"),
                thread_source=meta.get("thread_source"),
                last_event=lifecycle,
                last_turn_id=turn_id,
                first_seen_at=first_seen_at,
                last_activity_at=last_activity_at,
                source_revision=revision,
            ),
            None,
        )

    def _session_meta(self, record: Any) -> dict[str, str | None] | None:
        if not isinstance(record, dict) or record.get("type") != "session_meta":
            return None
        payload = record.get("payload")
        if not isinstance(payload, dict):
            return None
        thread_id = self._safe_id(payload.get("session_id") or payload.get("id"))
        cwd = self._safe_path_text(payload.get("cwd"))
        if thread_id is None or cwd is None:
            return None
        return {
            "thread_id": thread_id,
            "cwd": cwd,
            "timestamp": self._safe_text(payload.get("timestamp") or record.get("timestamp"), 64),
            "originator": self._safe_metadata(payload.get("originator")),
            "cli_version": self._safe_metadata(payload.get("cli_version")),
            "thread_source": self._safe_metadata(payload.get("thread_source")),
        }

    def _latest_lifecycle(
        self, path: Path, size: int
    ) -> tuple[CodexLifecycleEvent, str | None, str | None]:
        with path.open("rb") as handle:
            start = max(0, size - self.max_tail_bytes)
            handle.seek(start)
            raw = handle.read(self.max_tail_bytes)
        lines = raw.splitlines()
        if start > 0 and lines:
            lines = lines[1:]
        latest = CodexLifecycleEvent.SESSION_META
        latest_turn: str | None = None
        latest_timestamp: str | None = None
        for raw_line in lines:
            try:
                record = json.loads(raw_line.decode("utf-8"))
            except (UnicodeError, json.JSONDecodeError):
                continue
            if not isinstance(record, dict) or record.get("type") != "event_msg":
                continue
            payload = record.get("payload")
            if not isinstance(payload, dict):
                continue
            mapped = _TAIL_EVENT_MAP.get(payload.get("type"))
            if mapped is None:
                continue
            latest = mapped
            latest_turn = self._safe_id(payload.get("turn_id"))
            latest_timestamp = self._timestamp(record.get("timestamp")) or latest_timestamp
        return latest, latest_turn, latest_timestamp

    @staticmethod
    def _inside(child: Path, parent: Path) -> bool:
        child_text = os.path.normcase(str(child))
        parent_text = os.path.normcase(str(parent))
        try:
            return os.path.commonpath((child_text, parent_text)) == parent_text
        except ValueError:
            return False

    @staticmethod
    def _safe_id(value: Any) -> str | None:
        if not isinstance(value, str) or not value or len(value) > MAX_ID_TEXT:
            return None
        return value if _SAFE_ID.fullmatch(value) else None

    @staticmethod
    def _safe_text(value: Any, limit: int) -> str | None:
        if value is None:
            return None
        if not isinstance(value, str) or not value or len(value) > limit:
            return None
        return value

    @staticmethod
    def _safe_path_text(value: Any) -> str | None:
        if not isinstance(value, str) or not value or len(value) > MAX_PATH_TEXT:
            return None
        if any(ord(character) < 32 for character in value):
            return None
        return value

    @staticmethod
    def _safe_metadata(value: Any) -> str | None:
        if not isinstance(value, str) or not value or len(value) > MAX_METADATA_TEXT:
            return None
        return value if _SAFE_METADATA.fullmatch(value) else None

    @staticmethod
    def _timestamp(value: Any) -> str | None:
        if not isinstance(value, str) or len(value) > 64:
            return None
        try:
            parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
        except ValueError:
            return None
        if parsed.tzinfo is None:
            parsed = parsed.replace(tzinfo=UTC)
        return parsed.astimezone(UTC).isoformat()

    @staticmethod
    def _mtime_text(value: float) -> str:
        return datetime.fromtimestamp(value, tz=UTC).isoformat()
