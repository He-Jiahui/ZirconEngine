from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum
from typing import ClassVar, Mapping, TypeVar
import re

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
    SERVICE_DRAIN = "service.drain"
    SERVICE_RESUME = "service.resume"
    SERVICE_STOP = "service.stop"
    MILESTONE_COMMIT = "milestone.commit"
    SESSION_COMPLETE = "session.complete"
    SERVICE_RESTART = "service.restart"
    SERVICE_FORCE_STOP = "service.force_stop"
    MAINTENANCE_CLEANUP = "maintenance.cleanup"
    CODEX_RECONCILE = "codex.sessions.reconcile"


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
class LifecycleParameters(ActionParameters):
    timeout_seconds: int
    fields: ClassVar[frozenset[str]] = frozenset({"timeoutSeconds"})

    @classmethod
    def _from_payload(cls, payload: Mapping[str, object]) -> "LifecycleParameters":
        try:
            timeout = int(payload["timeoutSeconds"])
        except (TypeError, ValueError) as error:
            raise CoordinatorError(
                "action_parameters_invalid", "Lifecycle timeout must be an integer"
            ) from error
        if timeout < 1 or timeout > 300:
            raise CoordinatorError(
                "action_parameters_invalid", "Lifecycle timeout must be within 1-300 seconds"
            )
        return cls(timeout)

    def to_payload(self) -> dict[str, object]:
        return {"timeoutSeconds": self.timeout_seconds}


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
    run_id: str
    milestone_id: str
    fields: ClassVar[frozenset[str]] = frozenset(
        {"sessionId", "template", "runId", "milestoneId"}
    )

    @classmethod
    def _from_payload(cls, payload: Mapping[str, object]) -> "ValidationStartParameters":
        session = SessionParameters._from_payload({"sessionId": payload["sessionId"]})
        try:
            template = ValidationTemplate(str(payload["template"]))
        except ValueError as error:
            raise CoordinatorError(
                "action_parameters_invalid", "Unknown server validation template"
            ) from error
        milestone = MilestoneParameters._from_payload(
            {
                "sessionId": session.session_id,
                "runId": payload["runId"],
                "milestoneId": payload["milestoneId"],
            }
        )
        return cls(session.session_id, template, milestone.run_id, milestone.milestone_id)

    def to_payload(self) -> dict[str, object]:
        return {
            "sessionId": self.session_id,
            "template": self.template.value,
            "runId": self.run_id,
            "milestoneId": self.milestone_id,
        }


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
class MilestoneParameters(ActionParameters):
    session_id: str
    run_id: str
    milestone_id: str
    fields: ClassVar[frozenset[str]] = frozenset(
        {"sessionId", "runId", "milestoneId"}
    )

    @classmethod
    def _from_payload(cls, payload: Mapping[str, object]) -> "MilestoneParameters":
        session = SessionParameters._from_payload({"sessionId": payload["sessionId"]})
        run_id = str(payload["runId"]).strip()
        milestone_id = str(payload["milestoneId"]).strip().upper()
        if not run_id or len(run_id) > 128 or not re.fullmatch(r"[a-zA-Z0-9-]+", run_id):
            raise CoordinatorError("action_parameters_invalid", "Workflow run ID is invalid")
        if not re.fullmatch(r"M[1-9]\d*", milestone_id):
            raise CoordinatorError("action_parameters_invalid", "Milestone ID is invalid")
        return cls(session.session_id, run_id, milestone_id)

    def to_payload(self) -> dict[str, object]:
        return {
            "sessionId": self.session_id,
            "runId": self.run_id,
            "milestoneId": self.milestone_id,
        }


@dataclass(frozen=True, slots=True)
class GoalCloseoutParameters(ActionParameters):
    session_id: str
    run_id: str
    fields: ClassVar[frozenset[str]] = frozenset({"sessionId", "runId"})

    @classmethod
    def _from_payload(cls, payload: Mapping[str, object]) -> "GoalCloseoutParameters":
        session = SessionParameters._from_payload({"sessionId": payload["sessionId"]})
        run_id = str(payload["runId"]).strip()
        if not run_id or len(run_id) > 128 or not re.fullmatch(r"[a-zA-Z0-9-]+", run_id):
            raise CoordinatorError("action_parameters_invalid", "Workflow run ID is invalid")
        return cls(session.session_id, run_id)

    def to_payload(self) -> dict[str, object]:
        return {"sessionId": self.session_id, "runId": self.run_id}


@dataclass(frozen=True, slots=True)
class TopologyRefreshParameters(ActionParameters):
    session_id: str
    executor_session_id: str | None = None
    run_id: str | None = None
    milestone_id: str | None = None
    critical_count: int | None = None
    important_count: int | None = None
    summary: str | None = None

    @classmethod
    def parse(cls, payload: Mapping[str, object]) -> "TopologyRefreshParameters":
        keys = set(payload)
        basic = {"sessionId"}
        review = {
            "sessionId", "executorSessionId", "runId", "milestoneId", "criticalCount",
            "importantCount", "summary",
        }
        if keys not in (basic, review):
            raise CoordinatorError(
                "action_parameters_invalid",
                "Topology refresh accepts either a Session or one complete review submission",
            )
        session = SessionParameters._from_payload({"sessionId": payload["sessionId"]})
        if keys == basic:
            return cls(session.session_id)
        executor = SessionParameters._from_payload(
            {"sessionId": payload["executorSessionId"]}
        )
        if executor.session_id == session.session_id:
            raise CoordinatorError(
                "workflow_review_not_independent",
                "Reviewer Session must differ from executor Session",
            )
        milestone = MilestoneParameters._from_payload(
            {
                "sessionId": executor.session_id,
                "runId": payload["runId"],
                "milestoneId": payload["milestoneId"],
            }
        )
        try:
            critical = int(payload["criticalCount"])
            important = int(payload["importantCount"])
        except (TypeError, ValueError) as error:
            raise CoordinatorError(
                "action_parameters_invalid", "Review finding counts must be integers"
            ) from error
        summary = str(payload["summary"]).strip()
        if critical < 0 or important < 0 or not summary or len(summary) > 2_000:
            raise CoordinatorError(
                "action_parameters_invalid", "Review findings or summary are invalid"
            )
        return cls(
            session.session_id,
            executor.session_id,
            milestone.run_id,
            milestone.milestone_id,
            critical,
            important,
            summary,
        )

    def to_payload(self) -> dict[str, object]:
        if self.run_id is None:
            return {"sessionId": self.session_id}
        return {
            "sessionId": self.session_id,
            "executorSessionId": self.executor_session_id or "",
            "runId": self.run_id,
            "milestoneId": self.milestone_id or "",
            "criticalCount": self.critical_count or 0,
            "importantCount": self.important_count or 0,
            "summary": self.summary or "",
        }


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
