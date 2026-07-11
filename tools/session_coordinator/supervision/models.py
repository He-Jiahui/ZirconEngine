from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum

from ..models import SupervisionState


class LifecycleKind(StrEnum):
    DRAIN = "service.drain"
    RESUME = "service.resume"
    STOP = "service.stop"
    RESTART = "service.restart"
    FORCE_STOP = "service.force_stop"


class LifecycleStatus(StrEnum):
    ACCEPTED = "accepted"
    DRAINING = "draining"
    READY = "ready"
    STOPPING = "stopping"
    AWAITING_RESTART = "awaiting_restart"
    SUCCEEDED = "succeeded"
    FAILED = "failed"
    CANCELLED = "cancelled"


class BlockerKind(StrEnum):
    GIT_FINALIZE = "git_finalize"
    CARGO = "cargo"
    PATCH = "patch"
    VALIDATION = "validation"
    CONTROLLED_ACTION = "controlled_action"
    MAINTENANCE = "maintenance"
    LEASE = "lease"


@dataclass(frozen=True, slots=True)
class SupervisionBlocker:
    kind: BlockerKind
    identity: str
    status: str
    blocking: bool = True
    session_id: str | None = None

    def to_dict(self) -> dict[str, object]:
        return {
            "kind": self.kind.value,
            "identity": self.identity,
            "status": self.status,
            "blocking": self.blocking,
            "sessionId": self.session_id,
        }


@dataclass(frozen=True, slots=True)
class SupervisionSnapshot:
    repository_key: str
    state: SupervisionState
    daemon_instance_id: str | None
    process_id: int | None
    process_creation_time: str | None
    explicit_stop: bool
    maintenance_hold: bool
    failure_count: int
    next_retry_at: str | None
    circuit_open_until: str | None
    healthy_since: str | None
    last_reason_code: str | None
    updated_at: str
    blockers: tuple[SupervisionBlocker, ...]

    def to_dict(self) -> dict[str, object]:
        return {
            "repositoryKey": self.repository_key,
            "state": self.state.value,
            "daemonInstanceId": self.daemon_instance_id,
            "processId": self.process_id,
            "processCreationTime": self.process_creation_time,
            "explicitStop": self.explicit_stop,
            "maintenanceHold": self.maintenance_hold,
            "failureCount": self.failure_count,
            "nextRetryAt": self.next_retry_at,
            "circuitOpenUntil": self.circuit_open_until,
            "healthySince": self.healthy_since,
            "lastReasonCode": self.last_reason_code,
            "updatedAt": self.updated_at,
            "busy": any(item.blocking for item in self.blockers),
            "blockers": [item.to_dict() for item in self.blockers],
        }
