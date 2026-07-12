from __future__ import annotations

import json
import os
import re
import time
import uuid
from dataclasses import dataclass
from datetime import datetime
from enum import StrEnum
from pathlib import Path
from typing import Any


TRIGGER_SCHEMA_VERSION = 1
MAX_TRIGGER_BYTES = 4096
MAX_PENDING_TRIGGERS = 1024

_SAFE_ID = re.compile(r"^[A-Za-z0-9._:-]{1,160}$")
_SAFE_METADATA = re.compile(r"^[A-Za-z0-9][A-Za-z0-9 ._:-]{0,255}$")
_REPOSITORY_KEY = re.compile(r"^[a-f0-9]{64}$")


class CodexHookEvent(StrEnum):
    SESSION_START = "session_start"
    USER_PROMPT_SUBMIT = "user_prompt_submit"
    STOP = "stop"
    SUBAGENT_START = "subagent_start"
    SUBAGENT_STOP = "subagent_stop"

    @property
    def codex_name(self) -> str:
        return {
            self.SESSION_START: "SessionStart",
            self.USER_PROMPT_SUBMIT: "UserPromptSubmit",
            self.STOP: "Stop",
            self.SUBAGENT_START: "SubagentStart",
            self.SUBAGENT_STOP: "SubagentStop",
        }[self]

    @classmethod
    def from_codex_name(cls, value: str) -> "CodexHookEvent | None":
        return next((event for event in cls if event.codex_name == value), None)


@dataclass(frozen=True, slots=True)
class CodexTrigger:
    event: CodexHookEvent
    session_id: str
    cwd: str
    created_at: str
    turn_id: str | None = None
    source: str | None = None
    model: str | None = None
    permission_mode: str | None = None
    agent_id: str | None = None
    agent_type: str | None = None


@dataclass(frozen=True, slots=True)
class CodexSpoolItem:
    path: Path
    trigger: CodexTrigger


class CodexTriggerSpool:
    """Repository-scoped, privacy-bounded handoff queue for lifecycle Hooks."""

    def __init__(
        self,
        base_root: str | Path,
        repository_key: str,
        *,
        max_pending: int = MAX_PENDING_TRIGGERS,
    ):
        if not _REPOSITORY_KEY.fullmatch(repository_key):
            raise ValueError("repository key must be a lowercase SHA-256 digest")
        if max_pending <= 0 or max_pending > MAX_PENDING_TRIGGERS:
            raise ValueError("pending trigger cap is outside the supported range")
        self.base_root = Path(base_root).resolve()
        self.repository_key = repository_key
        self.repository_root = (self.base_root / repository_key).resolve()
        self.pending_root = self.repository_root / "pending"
        self.quarantine_root = self.repository_root / "quarantine"
        self.max_pending = max_pending
        if not self.repository_root.is_relative_to(self.base_root):
            raise ValueError("repository spool escaped its managed base")

    def enqueue(self, trigger: CodexTrigger) -> Path:
        payload = self._payload(trigger)
        encoded = json.dumps(
            payload, sort_keys=True, separators=(",", ":")
        ).encode("utf-8")
        if len(encoded) > MAX_TRIGGER_BYTES:
            raise ValueError("sanitized Codex trigger exceeds the spool limit")
        self.pending_root.mkdir(parents=True, exist_ok=True)
        nonce = uuid.uuid4().hex
        temporary = self.pending_root / f".tmp-{nonce}"
        destination = self.pending_root / f"{time.time_ns():020d}-{nonce}.json"
        try:
            with temporary.open("xb") as handle:
                handle.write(encoded)
                handle.flush()
                os.fsync(handle.fileno())
            os.replace(temporary, destination)
        finally:
            temporary.unlink(missing_ok=True)
        self._enforce_cap()
        return destination

    def pending_count(self) -> int:
        if not self.pending_root.exists():
            return 0
        try:
            return sum(1 for path in self.pending_root.glob("*.json") if path.is_file())
        except OSError:
            return 0

    def validated_pending(self) -> tuple[CodexSpoolItem, ...]:
        if not self.pending_root.exists():
            return ()
        items: list[CodexSpoolItem] = []
        for path in self._ordered_pending():
            try:
                if path.stat().st_size > MAX_TRIGGER_BYTES:
                    raise ValueError("trigger is oversized")
                payload = json.loads(path.read_text(encoding="utf-8"))
                trigger = self._trigger(payload)
            except (OSError, UnicodeError, json.JSONDecodeError, TypeError, ValueError):
                self._quarantine(path)
                continue
            items.append(CodexSpoolItem(path, trigger))
        return tuple(items)

    def acknowledge_committed(
        self, items: tuple[CodexSpoolItem, ...], *, run_id: str
    ) -> None:
        if _SAFE_ID.fullmatch(run_id) is None:
            raise ValueError("a committed reconciliation run id is required")
        for item in items:
            resolved = item.path.resolve(strict=False)
            if resolved.parent != self.pending_root or resolved.suffix != ".json":
                raise ValueError("trigger acknowledgement escaped the pending spool")
        for item in items:
            item.path.unlink(missing_ok=True)

    def _payload(self, trigger: CodexTrigger) -> dict[str, object]:
        self._validate_trigger(trigger)
        return {
            "agentId": trigger.agent_id,
            "agentType": trigger.agent_type,
            "createdAt": trigger.created_at,
            "cwd": trigger.cwd,
            "eventName": trigger.event.value,
            "model": trigger.model,
            "permissionMode": trigger.permission_mode,
            "repositoryKey": self.repository_key,
            "schemaVersion": TRIGGER_SCHEMA_VERSION,
            "sessionId": trigger.session_id,
            "source": trigger.source,
            "turnId": trigger.turn_id,
        }

    def _trigger(self, payload: Any) -> CodexTrigger:
        if not isinstance(payload, dict) or set(payload) != {
            "agentId",
            "agentType",
            "createdAt",
            "cwd",
            "eventName",
            "model",
            "permissionMode",
            "repositoryKey",
            "schemaVersion",
            "sessionId",
            "source",
            "turnId",
        }:
            raise ValueError("trigger schema is not exact")
        if payload["schemaVersion"] != TRIGGER_SCHEMA_VERSION:
            raise ValueError("trigger schema version is unsupported")
        if payload["repositoryKey"] != self.repository_key:
            raise ValueError("trigger belongs to another repository")
        trigger = CodexTrigger(
            event=CodexHookEvent(str(payload["eventName"])),
            session_id=str(payload["sessionId"]),
            cwd=str(payload["cwd"]),
            created_at=str(payload["createdAt"]),
            turn_id=self._optional_string(payload["turnId"]),
            source=self._optional_string(payload["source"]),
            model=self._optional_string(payload["model"]),
            permission_mode=self._optional_string(payload["permissionMode"]),
            agent_id=self._optional_string(payload["agentId"]),
            agent_type=self._optional_string(payload["agentType"]),
        )
        self._validate_trigger(trigger)
        return trigger

    @staticmethod
    def _optional_string(value: object) -> str | None:
        if value is None:
            return None
        if not isinstance(value, str):
            raise ValueError("optional trigger field must be text")
        return value

    @staticmethod
    def _validate_trigger(trigger: CodexTrigger) -> None:
        if _SAFE_ID.fullmatch(trigger.session_id) is None:
            raise ValueError("session id is invalid")
        if not trigger.cwd or not Path(trigger.cwd).is_absolute() or len(trigger.cwd) > 4096 or any(
            ord(character) < 32 for character in trigger.cwd
        ):
            raise ValueError("trigger cwd is invalid")
        if not trigger.created_at or len(trigger.created_at) > 64:
            raise ValueError("trigger timestamp is invalid")
        try:
            timestamp = datetime.fromisoformat(trigger.created_at.replace("Z", "+00:00"))
        except ValueError as error:
            raise ValueError("trigger timestamp is invalid") from error
        if timestamp.tzinfo is None:
            raise ValueError("trigger timestamp must include a timezone")
        for value in (trigger.turn_id, trigger.agent_id):
            if value is not None and _SAFE_ID.fullmatch(value) is None:
                raise ValueError("trigger identifier is invalid")
        for value in (
            trigger.source,
            trigger.model,
            trigger.permission_mode,
            trigger.agent_type,
        ):
            if value is not None and _SAFE_METADATA.fullmatch(value) is None:
                raise ValueError("trigger metadata is invalid")
        if trigger.permission_mode not in (
            None,
            "default",
            "acceptEdits",
            "plan",
            "dontAsk",
            "bypassPermissions",
        ):
            raise ValueError("permission mode is unsupported")
        subagent = trigger.event in (
            CodexHookEvent.SUBAGENT_START,
            CodexHookEvent.SUBAGENT_STOP,
        )
        if trigger.event is CodexHookEvent.SESSION_START:
            if trigger.source not in ("startup", "resume", "clear", "compact"):
                raise ValueError("SessionStart source is unsupported")
            if trigger.turn_id is not None:
                raise ValueError("SessionStart must not carry a turn id")
        elif trigger.source is not None or trigger.turn_id is None:
            raise ValueError("turn event fields are inconsistent")
        if subagent != (trigger.agent_id is not None and trigger.agent_type is not None):
            raise ValueError("subagent fields are inconsistent")

    def _ordered_pending(self) -> tuple[Path, ...]:
        try:
            paths = tuple(path for path in self.pending_root.glob("*.json") if path.is_file())
        except OSError:
            return ()
        return tuple(sorted(paths, key=lambda path: path.name))

    def _enforce_cap(self) -> None:
        paths = self._ordered_pending()
        for path in paths[: max(0, len(paths) - self.max_pending)]:
            path.unlink(missing_ok=True)

    def _quarantine(self, path: Path) -> None:
        try:
            if path.resolve(strict=False).parent != self.pending_root:
                return
            self.quarantine_root.mkdir(parents=True, exist_ok=True)
            destination = self.quarantine_root / f"invalid-{uuid.uuid4().hex}.json"
            os.replace(path, destination)
        except OSError:
            return
