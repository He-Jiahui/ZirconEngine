from __future__ import annotations

from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from enum import StrEnum
from typing import Any


def utc_now() -> datetime:
    return datetime.now(UTC)


def utc_text(value: datetime | None = None) -> str:
    return (value or utc_now()).isoformat()


def parse_utc(value: str) -> datetime:
    parsed = datetime.fromisoformat(value)
    return parsed if parsed.tzinfo is not None else parsed.replace(tzinfo=UTC)


class SessionStatus(StrEnum):
    REGISTERED = "registered"
    ACTIVE = "active"
    WAITING_LEASE = "waiting_lease"
    RESOLVING_FAILURE = "resolving_failure"
    WAITING_VALIDATION = "waiting_validation"
    FINALIZING = "finalizing"
    COMPLETED = "completed"
    STALE = "stale"
    ARCHIVED = "archived"
    CANCELLED = "cancelled"


ALLOWED_STATUS_TRANSITIONS: dict[SessionStatus, frozenset[SessionStatus]] = {
    SessionStatus.REGISTERED: frozenset(
        {SessionStatus.ACTIVE, SessionStatus.RESOLVING_FAILURE, SessionStatus.CANCELLED}
    ),
    SessionStatus.ACTIVE: frozenset(
        {
            SessionStatus.WAITING_LEASE,
            SessionStatus.RESOLVING_FAILURE,
            SessionStatus.WAITING_VALIDATION,
            SessionStatus.FINALIZING,
            SessionStatus.COMPLETED,
            SessionStatus.STALE,
            SessionStatus.CANCELLED,
        }
    ),
    SessionStatus.WAITING_LEASE: frozenset(
        {SessionStatus.ACTIVE, SessionStatus.STALE, SessionStatus.CANCELLED}
    ),
    SessionStatus.RESOLVING_FAILURE: frozenset(
        {SessionStatus.ACTIVE, SessionStatus.WAITING_VALIDATION, SessionStatus.STALE, SessionStatus.CANCELLED}
    ),
    SessionStatus.WAITING_VALIDATION: frozenset(
        {SessionStatus.ACTIVE, SessionStatus.FINALIZING, SessionStatus.COMPLETED, SessionStatus.STALE}
    ),
    SessionStatus.FINALIZING: frozenset(
        {SessionStatus.ACTIVE, SessionStatus.COMPLETED, SessionStatus.STALE}
    ),
    SessionStatus.COMPLETED: frozenset(
        {SessionStatus.ACTIVE, SessionStatus.FINALIZING, SessionStatus.ARCHIVED}
    ),
    SessionStatus.STALE: frozenset(
        {SessionStatus.ACTIVE, SessionStatus.ARCHIVED, SessionStatus.CANCELLED}
    ),
    SessionStatus.ARCHIVED: frozenset(),
    SessionStatus.CANCELLED: frozenset({SessionStatus.ARCHIVED}),
}


class CoordinatorError(RuntimeError):
    def __init__(self, code: str, message: str, *, details: dict[str, Any] | None = None):
        super().__init__(message)
        self.code = code
        self.message = message
        self.details = details or {}

    def to_dict(self) -> dict[str, Any]:
        return {"code": self.code, "message": self.message, "details": self.details}


class InvalidStatusTransition(CoordinatorError):
    def __init__(self, current: SessionStatus, requested: SessionStatus):
        super().__init__(
            "invalid_status_transition",
            f"Session status cannot transition from {current.value} to {requested.value}",
            details={"current": current.value, "requested": requested.value},
        )


@dataclass(frozen=True, slots=True)
class SessionRecord:
    session_id: str
    status: SessionStatus
    display_name: str | None
    plan_path: str | None
    write_scope: tuple[str, ...]
    status_reason: str | None
    base_head: str
    baseline_epoch: int | None
    created_at: datetime
    updated_at: datetime
    last_heartbeat_at: datetime

    def to_dict(self) -> dict[str, Any]:
        result = asdict(self)
        result["status"] = self.status.value
        result["write_scope"] = list(self.write_scope)
        for key in ("created_at", "updated_at", "last_heartbeat_at"):
            result[key] = result[key].isoformat()
        return result
