from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum


class CodexSourceLocation(StrEnum):
    ACTIVE = "active"
    ARCHIVED = "archived"
    MISSING = "missing"


class CodexSessionState(StrEnum):
    ACTIVE = "active"
    IDLE = "idle"
    ARCHIVED = "archived"
    UNAVAILABLE = "unavailable"


class CodexLifecycleEvent(StrEnum):
    SESSION_META = "session_meta"
    TASK_STARTED = "task_started"
    TASK_COMPLETED = "task_completed"
    TURN_ABORTED = "turn_aborted"
    SESSION_START = "session_start"
    USER_PROMPT_SUBMIT = "user_prompt_submit"
    STOP = "stop"
    SUBAGENT_START = "subagent_start"
    SUBAGENT_STOP = "subagent_stop"
    UNKNOWN = "unknown"


class CodexSyncTrigger(StrEnum):
    STARTUP = "startup"
    PERIODIC = "periodic"
    HOOK = "hook"
    CONTROLLED = "controlled"


@dataclass(frozen=True)
class CodexSourceRevision:
    path: str
    size: int
    mtime_ns: int


@dataclass(frozen=True)
class CodexDiscoveryDiagnostic:
    code: str
    source_path: str


@dataclass(frozen=True)
class CodexDiscoveredSession:
    thread_id: str
    rollout_path: str
    source_location: CodexSourceLocation
    state: CodexSessionState
    cwd: str
    originator: str | None
    cli_version: str | None
    thread_source: str | None
    last_event: CodexLifecycleEvent
    last_turn_id: str | None
    first_seen_at: str
    last_activity_at: str
    source_revision: CodexSourceRevision
    diagnostic_code: str | None = None


@dataclass(frozen=True)
class CodexDiscoveryResult:
    sessions: tuple[CodexDiscoveredSession, ...]
    diagnostics: tuple[CodexDiscoveryDiagnostic, ...]
    membership_complete: bool
    scanned_count: int
    source_revision: str


@dataclass(frozen=True)
class CodexReconcileResult:
    run_id: str
    scanned_count: int
    changed_count: int
    diagnostic_count: int
    unavailable_count: int
