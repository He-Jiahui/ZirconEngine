from __future__ import annotations

import json
import os
import re
import time
import uuid
from dataclasses import dataclass
from datetime import UTC, datetime
from enum import StrEnum
from pathlib import Path
from typing import Any


TRIGGER_SCHEMA_VERSION = 1
MAX_TRIGGER_BYTES = 4096
MAX_PENDING_TRIGGERS = 1024
OVERFLOW_SCHEMA_VERSION = 1
MAX_OVERFLOW_MARKER_BYTES = 2048
HOOK_HEALTH_SCHEMA_VERSION = 1
MAX_HOOK_HEALTH_MARKER_BYTES = 2048

_SAFE_ID = re.compile(r"^[A-Za-z0-9._:-]{1,160}$")
_SAFE_METADATA = re.compile(r"^[A-Za-z0-9][A-Za-z0-9 ._:-]{0,255}$")
_REPOSITORY_KEY = re.compile(r"^[a-f0-9]{64}$")
_HOOK_OUTCOMES = frozenset(("success", "error", "drop"))
_HOOK_ERROR_CODES = frozenset(
    ("hook_execution_failed", "spool_enqueue_failed", "coordinator_signal_failed")
)
_HOOK_DROP_CODES = frozenset(("hook_input_invalid",))


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
        self.overflow_path = self.repository_root / "overflow.json"
        self.hook_health_path = self.repository_root / "hook-health.json"
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
        self._enforce_cap(destination)
        return destination

    def pending_count(self) -> int:
        if not self.pending_root.exists():
            return 0
        try:
            return sum(1 for path in self.pending_root.glob("*.json") if path.is_file())
        except OSError:
            return 0

    def overflow_status(self) -> dict[str, object]:
        """Return bounded historical overflow evidence without trigger data."""
        try:
            payload = self._read_overflow_marker()
        except (OSError, UnicodeError, json.JSONDecodeError, TypeError, ValueError):
            return {"markerStatus": "invalid"}
        if payload is None:
            return {"markerStatus": "absent"}
        return {
            "markerStatus": "valid",
            "firstDetectedAt": payload["firstDetectedAt"],
            "lastDetectedAt": payload["lastDetectedAt"],
            "maxPending": payload["maxPending"],
            "pendingCount": payload["pendingCount"],
        }

    def record_hook_outcome(
        self,
        outcome: str,
        *,
        detected_at: str,
        code: str | None = None,
        pending_persisted: bool,
    ) -> None:
        """Persist bounded Hook health without payload or session identity."""
        _validate_marker_timestamp(detected_at)
        if outcome not in _HOOK_OUTCOMES or not isinstance(pending_persisted, bool):
            raise ValueError("hook health outcome is invalid")
        if outcome == "success":
            if code is not None or not pending_persisted:
                raise ValueError("successful hook health outcome is invalid")
        elif outcome == "error":
            if code not in _HOOK_ERROR_CODES:
                raise ValueError("hook health error code is invalid")
        elif code not in _HOOK_DROP_CODES or pending_persisted:
            raise ValueError("hook health drop code is invalid")

        try:
            existing = self._read_hook_health_marker()
        except (OSError, UnicodeError, json.JSONDecodeError, TypeError, ValueError):
            existing = None
        payload: dict[str, object] = {
            "lastAttemptAt": detected_at,
            "lastOutcome": outcome,
            "lastSuccessAt": None,
            "lastErrorAt": None,
            "lastErrorCode": None,
            "lastDropAt": None,
            "lastDropCode": None,
            "pendingPersisted": pending_persisted,
            "repositoryKey": self.repository_key,
            "schemaVersion": HOOK_HEALTH_SCHEMA_VERSION,
        }
        if existing is not None:
            for field in (
                "lastSuccessAt",
                "lastErrorAt",
                "lastErrorCode",
                "lastDropAt",
                "lastDropCode",
            ):
                payload[field] = existing[field]
        if outcome == "success":
            payload["lastSuccessAt"] = detected_at
        elif outcome == "error":
            payload["lastErrorAt"] = detected_at
            payload["lastErrorCode"] = code
        else:
            payload["lastDropAt"] = detected_at
            payload["lastDropCode"] = code

        encoded = json.dumps(
            payload, sort_keys=True, separators=(",", ":")
        ).encode("utf-8")
        if len(encoded) > MAX_HOOK_HEALTH_MARKER_BYTES:
            raise ValueError("hook health marker is oversized")
        self.repository_root.mkdir(parents=True, exist_ok=True)
        temporary = self.repository_root / f".hook-health-{uuid.uuid4().hex}.tmp"
        try:
            with temporary.open("xb") as handle:
                handle.write(encoded)
                handle.flush()
                os.fsync(handle.fileno())
            os.replace(temporary, self.hook_health_path)
        finally:
            temporary.unlink(missing_ok=True)

    def hook_health_status(self) -> dict[str, object]:
        try:
            payload = self._read_hook_health_marker()
        except (OSError, UnicodeError, json.JSONDecodeError, TypeError, ValueError):
            return {"markerStatus": "invalid"}
        if payload is None:
            return {"markerStatus": "absent"}
        return {
            "markerStatus": "valid",
            "lastAttemptAt": payload["lastAttemptAt"],
            "lastOutcome": payload["lastOutcome"],
            "lastSuccessAt": payload["lastSuccessAt"],
            "lastErrorAt": payload["lastErrorAt"],
            "lastErrorCode": payload["lastErrorCode"],
            "lastDropAt": payload["lastDropAt"],
            "lastDropCode": payload["lastDropCode"],
            "pendingPersisted": payload["pendingPersisted"],
        }

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

    def _enforce_cap(self, destination: Path) -> None:
        paths = self._ordered_pending()
        if len(paths) <= self.max_pending:
            return
        try:
            self._record_overflow(max(0, len(paths) - 1))
        except BaseException:
            destination.unlink(missing_ok=True)
            raise
        destination.unlink(missing_ok=True)
        raise OverflowError("Codex trigger spool is full")

    def _record_overflow(self, pending_count: int) -> None:
        now = datetime.now(tz=UTC).isoformat()
        try:
            existing = self._read_overflow_marker()
        except (OSError, UnicodeError, json.JSONDecodeError, TypeError, ValueError):
            existing = None
        first_detected = now if existing is None else str(existing["firstDetectedAt"])
        payload = {
            "firstDetectedAt": first_detected,
            "lastDetectedAt": now,
            "maxPending": self.max_pending,
            "pendingCount": pending_count,
            "repositoryKey": self.repository_key,
            "schemaVersion": OVERFLOW_SCHEMA_VERSION,
        }
        encoded = json.dumps(
            payload, sort_keys=True, separators=(",", ":")
        ).encode("utf-8")
        self.repository_root.mkdir(parents=True, exist_ok=True)
        temporary = self.repository_root / f".overflow-{uuid.uuid4().hex}.tmp"
        try:
            with temporary.open("xb") as handle:
                handle.write(encoded)
                handle.flush()
                os.fsync(handle.fileno())
            os.replace(temporary, self.overflow_path)
        finally:
            temporary.unlink(missing_ok=True)

    def _read_overflow_marker(self) -> dict[str, object] | None:
        try:
            size = self.overflow_path.stat().st_size
        except FileNotFoundError:
            return None
        if size > MAX_OVERFLOW_MARKER_BYTES:
            raise ValueError("overflow marker is oversized")
        payload = json.loads(self.overflow_path.read_text(encoding="utf-8"))
        if not isinstance(payload, dict) or set(payload) != {
            "firstDetectedAt",
            "lastDetectedAt",
            "maxPending",
            "pendingCount",
            "repositoryKey",
            "schemaVersion",
        }:
            raise ValueError("overflow marker schema is not exact")
        if (
            payload["schemaVersion"] != OVERFLOW_SCHEMA_VERSION
            or payload["repositoryKey"] != self.repository_key
            or not isinstance(payload["maxPending"], int)
            or not 0 < payload["maxPending"] <= MAX_PENDING_TRIGGERS
            or not isinstance(payload["pendingCount"], int)
            or payload["pendingCount"] < 0
        ):
            raise ValueError("overflow marker identity is invalid")
        for field in ("firstDetectedAt", "lastDetectedAt"):
            _validate_marker_timestamp(payload[field])
        return payload

    def _read_hook_health_marker(self) -> dict[str, object] | None:
        try:
            size = self.hook_health_path.stat().st_size
        except FileNotFoundError:
            return None
        if size > MAX_HOOK_HEALTH_MARKER_BYTES:
            raise ValueError("hook health marker is oversized")
        payload = json.loads(self.hook_health_path.read_text(encoding="utf-8"))
        if not isinstance(payload, dict) or set(payload) != {
            "lastAttemptAt",
            "lastOutcome",
            "lastSuccessAt",
            "lastErrorAt",
            "lastErrorCode",
            "lastDropAt",
            "lastDropCode",
            "pendingPersisted",
            "repositoryKey",
            "schemaVersion",
        }:
            raise ValueError("hook health marker schema is not exact")
        if (
            type(payload["schemaVersion"]) is not int
            or payload["schemaVersion"] != HOOK_HEALTH_SCHEMA_VERSION
            or payload["repositoryKey"] != self.repository_key
            or payload["lastOutcome"] not in _HOOK_OUTCOMES
            or not isinstance(payload["pendingPersisted"], bool)
        ):
            raise ValueError("hook health marker identity is invalid")
        _validate_marker_timestamp(payload["lastAttemptAt"])
        for field in ("lastSuccessAt", "lastErrorAt", "lastDropAt"):
            value = payload[field]
            if value is not None:
                _validate_marker_timestamp(value)
        if (payload["lastErrorAt"] is None) != (payload["lastErrorCode"] is None):
            raise ValueError("hook health error evidence is inconsistent")
        if payload["lastErrorCode"] is not None and payload["lastErrorCode"] not in _HOOK_ERROR_CODES:
            raise ValueError("hook health error code is invalid")
        if (payload["lastDropAt"] is None) != (payload["lastDropCode"] is None):
            raise ValueError("hook health drop evidence is inconsistent")
        if payload["lastDropCode"] is not None and payload["lastDropCode"] not in _HOOK_DROP_CODES:
            raise ValueError("hook health drop code is invalid")
        outcome = payload["lastOutcome"]
        if outcome == "success" and (
            payload["lastSuccessAt"] != payload["lastAttemptAt"]
            or not payload["pendingPersisted"]
        ):
            raise ValueError("successful hook health evidence is inconsistent")
        if outcome == "error" and (
            payload["lastErrorAt"] != payload["lastAttemptAt"]
            or payload["lastErrorCode"] is None
        ):
            raise ValueError("failed hook health evidence is inconsistent")
        if outcome == "drop" and (
            payload["lastDropAt"] != payload["lastAttemptAt"]
            or payload["lastDropCode"] is None
            or payload["pendingPersisted"]
        ):
            raise ValueError("dropped hook health evidence is inconsistent")
        return payload

    def _quarantine(self, path: Path) -> None:
        try:
            if path.resolve(strict=False).parent != self.pending_root:
                return
            self.quarantine_root.mkdir(parents=True, exist_ok=True)
            destination = self.quarantine_root / f"invalid-{uuid.uuid4().hex}.json"
            os.replace(path, destination)
        except OSError:
            return


def _validate_marker_timestamp(value: object) -> None:
    if not isinstance(value, str) or len(value) > 64:
        raise ValueError("marker timestamp is invalid")
    timestamp = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if timestamp.tzinfo is None:
        raise ValueError("marker timestamp must include a timezone")
