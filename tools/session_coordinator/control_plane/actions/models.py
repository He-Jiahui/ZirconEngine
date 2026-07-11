from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum
from typing import ClassVar, Mapping, TypeVar

from ...models import CoordinatorError, WebControlRole


class ActionRisk(StrEnum):
    GREEN = "green"
    YELLOW = "yellow"
    RED = "red"


class ActionStatus(StrEnum):
    PREVIEWED = "previewed"
    EXECUTING = "executing"
    SUCCEEDED = "succeeded"
    FAILED = "failed"
    CANCELLED = "cancelled"
    EXPIRED = "expired"
    STATE_CHANGED = "state_changed"
    DENIED = "denied"


class ActionKind(StrEnum):
    SESSION_HEARTBEAT = "session.heartbeat"
    SESSION_ACTIVATE = "session.activate"
    LEASE_CLAIM = "lease.claim_own_scope"
    LEASE_RELEASE = "lease.release_own"
    PATCH_PROCESS = "patch.process_own"
    VALIDATION_START = "validation.start"
    VALIDATION_CANCEL = "validation.cancel"
    FAILURE_REFRESH = "failure.refresh"
    TOPOLOGY_REFRESH = "topology.refresh"
    DRAIN_PREVIEW = "service.drain_preview"
    MILESTONE_COMMIT = "milestone.commit"
    SESSION_COMPLETE = "session.complete"
    SERVICE_RESTART = "service.restart"
    MAINTENANCE_CLEANUP = "maintenance.cleanup"


@dataclass(frozen=True, slots=True)
class ActionContext:
    actor: str
    role: WebControlRole
    web_session_id: str | None
    bound_session_id: str | None
    daemon_instance_id: str


TParameters = TypeVar("TParameters", bound="ActionParameters")


@dataclass(frozen=True, slots=True)
class ActionParameters:
    fields: ClassVar[frozenset[str]] = frozenset()

    @classmethod
    def parse(cls: type[TParameters], payload: Mapping[str, object]) -> TParameters:
        if set(payload) != cls.fields:
            raise CoordinatorError(
                "action_parameters_invalid",
                "Action parameters must exactly match the typed catalog contract",
                details={"expected": sorted(cls.fields), "actual": sorted(payload)},
            )
        return cls._from_payload(payload)

    @classmethod
    def _from_payload(cls: type[TParameters], payload: Mapping[str, object]) -> TParameters:
        return cls()  # type: ignore[call-arg,return-value]

    def to_payload(self) -> dict[str, object]:
        return {}


@dataclass(frozen=True, slots=True)
class SessionParameters(ActionParameters):
    session_id: str
    fields: ClassVar[frozenset[str]] = frozenset({"sessionId"})

    @classmethod
    def _from_payload(cls, payload: Mapping[str, object]) -> "SessionParameters":
        session_id = str(payload["sessionId"]).strip()
        if not session_id or len(session_id) > 200:
            raise CoordinatorError("action_parameters_invalid", "Session ID is invalid")
        return cls(session_id)

    def to_payload(self) -> dict[str, object]:
        return {"sessionId": self.session_id}


class ValidationTemplate(StrEnum):
    COORDINATOR_ACTIONS = "coordinator-actions"
    WEB_CHECK = "web-check"


@dataclass(frozen=True, slots=True)
class ValidationStartParameters(ActionParameters):
    session_id: str
    template: ValidationTemplate
    fields: ClassVar[frozenset[str]] = frozenset({"sessionId", "template"})

    @classmethod
    def _from_payload(cls, payload: Mapping[str, object]) -> "ValidationStartParameters":
        session = SessionParameters._from_payload({"sessionId": payload["sessionId"]})
        try:
            template = ValidationTemplate(str(payload["template"]))
        except ValueError as error:
            raise CoordinatorError(
                "action_parameters_invalid", "Unknown server validation template"
            ) from error
        return cls(session.session_id, template)

    def to_payload(self) -> dict[str, object]:
        return {"sessionId": self.session_id, "template": self.template.value}


@dataclass(frozen=True, slots=True)
class ValidationCancelParameters(ActionParameters):
    session_id: str
    job_id: str
    fields: ClassVar[frozenset[str]] = frozenset({"sessionId", "jobId"})

    @classmethod
    def _from_payload(cls, payload: Mapping[str, object]) -> "ValidationCancelParameters":
        session = SessionParameters._from_payload({"sessionId": payload["sessionId"]})
        job_id = str(payload["jobId"]).strip()
        if not job_id or len(job_id) > 128 or not job_id.isalnum():
            raise CoordinatorError("action_parameters_invalid", "Validation job ID is invalid")
        return cls(session.session_id, job_id)

    def to_payload(self) -> dict[str, object]:
        return {"sessionId": self.session_id, "jobId": self.job_id}


@dataclass(frozen=True, slots=True)
class ActionSpec:
    kind: ActionKind
    title: str
    risk: ActionRisk
    required_role: WebControlRole
    parameter_type: type[ActionParameters]
    enabled: bool = True
    session_bound: bool = True
    preview_only: bool = False
    warnings: tuple[str, ...] = ()

    def parse_parameters(self, payload: Mapping[str, object]) -> ActionParameters:
        return self.parameter_type.parse(payload)

    def to_dict(self) -> dict[str, object]:
        return {
            "kind": self.kind.value,
            "title": self.title,
            "risk": self.risk.value,
            "requiredRole": self.required_role.value,
            "enabled": self.enabled,
            "sessionBound": self.session_bound,
            "previewOnly": self.preview_only,
            "warnings": list(self.warnings),
        }


@dataclass(frozen=True, slots=True)
class ActionFingerprint:
    digest: str
    payload: dict[str, object]


@dataclass(frozen=True, slots=True)
class ActionRecord:
    action_id: str
    kind: ActionKind
    risk: ActionRisk
    required_role: WebControlRole
    actor: str
    web_session_id: str | None
    bound_session_id: str | None
    parameters: dict[str, object]
    impact: tuple[str, ...]
    warnings: tuple[str, ...]
    state_fingerprint: str
    status: ActionStatus
    created_at: str
    expires_at: str
    reason: str | None = None
    result: dict[str, object] | None = None
    error_code: str | None = None
    confirmation_phrase: str | None = None

    def to_dict(self) -> dict[str, object]:
        return {
            "actionId": self.action_id,
            "kind": self.kind.value,
            "risk": self.risk.value,
            "requiredRole": self.required_role.value,
            "actor": self.actor,
            "boundSessionId": self.bound_session_id,
            "parameters": self.parameters,
            "impact": list(self.impact),
            "warnings": list(self.warnings),
            "stateFingerprint": self.state_fingerprint,
            "status": self.status.value,
            "createdAt": self.created_at,
            "expiresAt": self.expires_at,
            "reason": self.reason,
            "result": self.result,
            "errorCode": self.error_code,
            "confirmationPhrase": self.confirmation_phrase,
        }
