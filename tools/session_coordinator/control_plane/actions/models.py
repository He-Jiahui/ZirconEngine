from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum
from typing import ClassVar, Mapping, TypeVar
import re

from ...models import CoordinatorError, WebControlRole


_COMMIT_NODE_ID = re.compile(r"M[1-9]\d*(?:\.[1-9]\d*)?")


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
    BENCHMARK_GRANT_ISSUE = "validation.benchmark_grant.issue"
    FAILURE_REFRESH = "failure.refresh"
    TOPOLOGY_REFRESH = "topology.refresh"
    DRAIN_PREVIEW = "service.drain_preview"
    SERVICE_DRAIN = "service.drain"
    SERVICE_RESUME = "service.resume"
    SERVICE_ROLLOVER = "service.rollover"
    SERVICE_STOP = "service.stop"
    MILESTONE_COMMIT = "milestone.commit"
    MILESTONE_RECONCILE = "milestone.reconcile_accepted"
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
class MaintenanceCleanupParameters(ActionParameters):
    """Bind shared-index cleanup to the loaded maintenance Session."""

    session_id: str
    fields: ClassVar[frozenset[str]] = frozenset({"sessionId"})

    @classmethod
    def _from_payload(cls, payload: Mapping[str, object]) -> "MaintenanceCleanupParameters":
        session_id = payload.get("sessionId")
        if not isinstance(session_id, str) or not session_id.strip() or len(session_id.strip()) > 200:
            raise CoordinatorError("action_parameters_invalid", "Session ID is invalid")
        return cls(session_id.strip())

    def to_payload(self) -> dict[str, object]:
        return {"sessionId": self.session_id}


@dataclass(frozen=True, slots=True)
class LifecycleParameters(ActionParameters):
    timeout_seconds: int
    gpu_reservation_session_id: str | None = None
    release_maintenance_hold: bool = False
    maintenance_hold_action_id: str | None = None
    maintenance_session_ids: tuple[str, ...] = ()
    maintenance_session_id: str | None = None
    fields: ClassVar[frozenset[str]] = frozenset(
        {
            "timeoutSeconds",
            "gpuReservationSessionId",
            "releaseMaintenanceHold",
            "maintenanceHoldActionId",
            "maintenanceSessionIds",
            "maintenanceSessionId",
        }
    )

    @classmethod
    def parse(cls, payload: Mapping[str, object]) -> "LifecycleParameters":
        required = {"timeoutSeconds"}
        actual = set(payload)
        if not required.issubset(actual) or not actual.issubset(cls.fields):
            raise CoordinatorError(
                "action_parameters_invalid",
                "Lifecycle parameters must contain timeoutSeconds and may include maintenance scope or release proof",
                details={"expected": sorted(cls.fields), "actual": sorted(payload)},
            )
        return cls._from_payload(payload)

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
        reserved_session = payload.get("gpuReservationSessionId")
        if reserved_session is not None and (
            not isinstance(reserved_session, str) or not reserved_session.strip()
        ):
            raise CoordinatorError(
                "action_parameters_invalid",
                "GPU reservation Session must be a non-empty string",
            )
        release_hold = payload.get("releaseMaintenanceHold", False)
        if not isinstance(release_hold, bool):
            raise CoordinatorError(
                "action_parameters_invalid",
                "releaseMaintenanceHold must be a boolean",
            )
        maintenance_hold_action_id = payload.get("maintenanceHoldActionId")
        if maintenance_hold_action_id is not None and (
            not isinstance(maintenance_hold_action_id, str)
            or not maintenance_hold_action_id.strip()
        ):
            raise CoordinatorError(
                "action_parameters_invalid",
                "maintenanceHoldActionId must be a non-empty action ID",
            )
        if release_hold and maintenance_hold_action_id is None:
            raise CoordinatorError(
                "maintenance_hold_release_id_required",
                "Releasing a maintenance hold requires its controlled drain action ID",
            )
        raw_scope = payload.get("maintenanceSessionIds", [])
        if not isinstance(raw_scope, list) or len(raw_scope) > 16:
            raise CoordinatorError(
                "action_parameters_invalid",
                "maintenanceSessionIds must contain at most sixteen Session IDs",
            )
        maintenance_session_ids = tuple(
            session_id.strip() if isinstance(session_id, str) else ""
            for session_id in raw_scope
        )
        if (
            any(not session_id or len(session_id) > 200 for session_id in maintenance_session_ids)
            or len(set(maintenance_session_ids)) != len(maintenance_session_ids)
        ):
            raise CoordinatorError(
                "action_parameters_invalid",
                "maintenanceSessionIds must be unique non-empty Session IDs",
            )
        maintenance_session_id = payload.get("maintenanceSessionId")
        if maintenance_session_id is not None and (
            not isinstance(maintenance_session_id, str)
            or not maintenance_session_id.strip()
            or len(maintenance_session_id.strip()) > 200
        ):
            raise CoordinatorError(
                "action_parameters_invalid",
                "maintenanceSessionId must be a non-empty Session ID",
            )
        if maintenance_session_id is not None and not release_hold:
            raise CoordinatorError(
                "action_parameters_invalid",
                "maintenanceSessionId is valid only for an explicit maintenance release",
            )
        return cls(
            timeout,
            reserved_session.strip() if isinstance(reserved_session, str) else None,
            release_hold,
            (
                maintenance_hold_action_id.strip()
                if isinstance(maintenance_hold_action_id, str)
                else None
            ),
            maintenance_session_ids,
            maintenance_session_id.strip() if isinstance(maintenance_session_id, str) else None,
        )

    def to_payload(self) -> dict[str, object]:
        payload: dict[str, object] = {"timeoutSeconds": self.timeout_seconds}
        if self.gpu_reservation_session_id is not None:
            payload["gpuReservationSessionId"] = self.gpu_reservation_session_id
        if self.release_maintenance_hold:
            payload["releaseMaintenanceHold"] = True
        if self.maintenance_hold_action_id is not None:
            payload["maintenanceHoldActionId"] = self.maintenance_hold_action_id
        if self.maintenance_session_ids:
            payload["maintenanceSessionIds"] = list(self.maintenance_session_ids)
        if self.maintenance_session_id is not None:
            payload["maintenanceSessionId"] = self.maintenance_session_id
        return payload


@dataclass(frozen=True, slots=True)
class SessionParameters(ActionParameters):
    session_id: str
    display_name: str | None = None
    plan_path: str | None = None
    write_scope: tuple[str, ...] = ()
    maintenance_session_id: str | None = None
    fields: ClassVar[frozenset[str]] = frozenset(
        {
            "sessionId",
            "displayName",
            "planPath",
            "writeScope",
            "maintenanceSessionId",
        }
    )

    @classmethod
    def parse(cls, payload: Mapping[str, object]) -> "SessionParameters":
        if "sessionId" not in payload or not set(payload).issubset(cls.fields):
            raise CoordinatorError(
                "action_parameters_invalid",
                "Session parameters require sessionId and may include scoped bootstrap metadata",
                details={"expected": sorted(cls.fields), "actual": sorted(payload)},
            )
        return cls._from_payload(payload)

    @classmethod
    def _from_payload(cls, payload: Mapping[str, object]) -> "SessionParameters":
        session_id = str(payload["sessionId"]).strip()
        if not session_id or len(session_id) > 200:
            raise CoordinatorError("action_parameters_invalid", "Session ID is invalid")
        display_name = payload.get("displayName")
        if display_name is not None and (
            not isinstance(display_name, str) or not display_name.strip() or len(display_name.strip()) > 200
        ):
            raise CoordinatorError("action_parameters_invalid", "displayName is invalid")
        plan_path = payload.get("planPath")
        if plan_path is not None and (
            not isinstance(plan_path, str)
            or not plan_path.strip()
            or len(plan_path.strip()) > 500
            or plan_path.replace("\\", "/").startswith("/")
            or ":" in plan_path
        ):
            raise CoordinatorError("action_parameters_invalid", "planPath must be repository-relative")
        raw_scope = payload.get("writeScope", [])
        if not isinstance(raw_scope, list) or len(raw_scope) > 64:
            raise CoordinatorError(
                "action_parameters_invalid", "writeScope must contain at most sixty-four paths"
            )
        write_scope = tuple(
            path.strip().replace("\\", "/") if isinstance(path, str) else ""
            for path in raw_scope
        )
        if (
            any(not path or len(path) > 500 or path.startswith("/") or ":" in path for path in write_scope)
            or len(set(write_scope)) != len(write_scope)
        ):
            raise CoordinatorError(
                "action_parameters_invalid", "writeScope must contain unique repository-relative paths"
            )
        maintenance_session_id = payload.get("maintenanceSessionId")
        if maintenance_session_id is not None and (
            not isinstance(maintenance_session_id, str)
            or not maintenance_session_id.strip()
            or len(maintenance_session_id.strip()) > 200
        ):
            raise CoordinatorError(
                "action_parameters_invalid", "maintenanceSessionId is invalid"
            )
        if maintenance_session_id is None and (
            display_name is not None or plan_path is not None or write_scope
        ):
            raise CoordinatorError(
                "action_parameters_invalid",
                "Session bootstrap metadata requires maintenanceSessionId",
            )
        return cls(
            session_id,
            display_name.strip() if isinstance(display_name, str) else None,
            plan_path.strip().replace("\\", "/") if isinstance(plan_path, str) else None,
            write_scope,
            maintenance_session_id.strip()
            if isinstance(maintenance_session_id, str)
            else None,
        )

    def to_payload(self) -> dict[str, object]:
        payload: dict[str, object] = {"sessionId": self.session_id}
        if self.display_name is not None:
            payload["displayName"] = self.display_name
        if self.plan_path is not None:
            payload["planPath"] = self.plan_path
        if self.write_scope:
            payload["writeScope"] = list(self.write_scope)
        if self.maintenance_session_id is not None:
            payload["maintenanceSessionId"] = self.maintenance_session_id
        return payload


class ValidationTemplate(StrEnum):
    COORDINATOR_ACTIONS = "coordinator-actions"
    WEB_CHECK = "web-check"
    RUNTIME14_RUST_FOCUSED = "runtime14-rust-focused"
    NATIVE_PLUGIN_BENCHMARK = "native-plugin-benchmark"


class NativePluginBenchmarkProfile(StrEnum):
    RELEASE = "release"
    PROFILING = "profiling"


class NativePluginBenchmarkName(StrEnum):
    NATIVE_CALLBACK_ATOMIC_LEASE_1_THREAD = (
        "native_callback_atomic_lease_1_thread_benchmark"
    )
    NATIVE_CALLBACK_ATOMIC_LEASE_2_THREAD = (
        "native_callback_atomic_lease_2_thread_benchmark"
    )
    NATIVE_CALLBACK_ATOMIC_LEASE_16_THREAD = (
        "native_callback_atomic_lease_16_thread_benchmark"
    )
    NATIVE_CALLBACK_ATOMIC_LEASE_64_THREAD = (
        "native_callback_atomic_lease_64_thread_benchmark"
    )
    NATIVE_HOST_CONTEXT_LOOKUP_1_THREAD = (
        "native_host_context_lookup_1_thread_benchmark"
    )
    NATIVE_HOST_CONTEXT_LOOKUP_16_THREAD = (
        "native_host_context_lookup_16_thread_benchmark"
    )
    NATIVE_REGISTRATION_REPLAY_1_SYSTEM_1_METHOD = (
        "native_registration_replay_1_systems_1_methods_benchmark"
    )
    NATIVE_REGISTRATION_REPLAY_1_SYSTEM_100_METHODS = (
        "native_registration_replay_1_systems_100_methods_benchmark"
    )
    NATIVE_REGISTRATION_REPLAY_100_SYSTEMS_1_METHOD = (
        "native_registration_replay_100_systems_1_methods_benchmark"
    )
    NATIVE_REGISTRATION_REPLAY_100_SYSTEMS_100_METHODS = (
        "native_registration_replay_100_systems_100_methods_benchmark"
    )
    NATIVE_REGISTRATION_REPLAY_1000_SYSTEMS_1_METHOD = (
        "native_registration_replay_1000_systems_1_methods_benchmark"
    )
    NATIVE_REGISTRATION_REPLAY_1000_SYSTEMS_100_METHODS = (
        "native_registration_replay_1000_systems_100_methods_benchmark"
    )
    NATIVE_RUNTIME_BROADCAST_1_PLUGIN = (
        "native_runtime_broadcast_1_plugin_benchmark"
    )
    NATIVE_RUNTIME_BROADCAST_8_PLUGIN = (
        "native_runtime_broadcast_8_plugin_benchmark"
    )
    NATIVE_RUNTIME_BROADCAST_32_PLUGIN = (
        "native_runtime_broadcast_32_plugin_benchmark"
    )


@dataclass(frozen=True, slots=True)
class ValidationStartParameters(ActionParameters):
    session_id: str
    template: ValidationTemplate
    run_id: str
    milestone_id: str
    benchmark_name: NativePluginBenchmarkName | None = None
    cargo_profile: NativePluginBenchmarkProfile | None = None
    fields: ClassVar[frozenset[str]] = frozenset(
        {"sessionId", "template", "runId", "milestoneId"}
    )
    benchmark_fields: ClassVar[frozenset[str]] = fields | frozenset(
        {"benchmarkName", "cargoProfile"}
    )

    @classmethod
    def parse(cls, payload: Mapping[str, object]) -> "ValidationStartParameters":
        expected = (
            cls.benchmark_fields
            if payload.get("template")
            == ValidationTemplate.NATIVE_PLUGIN_BENCHMARK.value
            else cls.fields
        )
        if set(payload) != expected:
            raise CoordinatorError(
                "action_parameters_invalid",
                "Action parameters must exactly match the typed validation contract",
                details={"expected": sorted(expected), "actual": sorted(payload)},
            )
        return cls._from_payload(payload)

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
        benchmark_name = None
        cargo_profile = None
        if template is ValidationTemplate.NATIVE_PLUGIN_BENCHMARK:
            try:
                benchmark_name = NativePluginBenchmarkName(str(payload["benchmarkName"]))
                cargo_profile = NativePluginBenchmarkProfile(str(payload["cargoProfile"]))
            except ValueError as error:
                raise CoordinatorError(
                    "action_parameters_invalid",
                    "Native plugin benchmark name or Cargo profile is not allow-listed",
                ) from error
        return cls(
            session.session_id,
            template,
            milestone.run_id,
            milestone.milestone_id,
            benchmark_name,
            cargo_profile,
        )

    def to_payload(self) -> dict[str, object]:
        payload: dict[str, object] = {
            "sessionId": self.session_id,
            "template": self.template.value,
            "runId": self.run_id,
            "milestoneId": self.milestone_id,
        }
        if self.benchmark_name is not None:
            payload["benchmarkName"] = self.benchmark_name.value
        if self.cargo_profile is not None:
            payload["cargoProfile"] = self.cargo_profile.value
        return payload


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
        if _COMMIT_NODE_ID.fullmatch(milestone_id) is None:
            raise CoordinatorError(
                "action_parameters_invalid", "Milestone or slice ID is invalid"
            )
        return cls(session.session_id, run_id, milestone_id)

    def to_payload(self) -> dict[str, object]:
        return {
            "sessionId": self.session_id,
            "runId": self.run_id,
            "milestoneId": self.milestone_id,
        }


@dataclass(frozen=True, slots=True)
class BenchmarkGrantIssueParameters(ActionParameters):
    session_id: str
    source_session_id: str
    run_id: str
    milestone_id: str
    benchmark_name: NativePluginBenchmarkName
    cargo_profile: NativePluginBenchmarkProfile
    fields: ClassVar[frozenset[str]] = frozenset(
        {
            "sessionId",
            "sourceSessionId",
            "runId",
            "milestoneId",
            "benchmarkName",
            "cargoProfile",
        }
    )

    @classmethod
    def _from_payload(
        cls, payload: Mapping[str, object]
    ) -> "BenchmarkGrantIssueParameters":
        milestone = MilestoneParameters._from_payload(payload)
        source = SessionParameters._from_payload(
            {"sessionId": payload["sourceSessionId"]}
        )
        try:
            benchmark_name = NativePluginBenchmarkName(str(payload["benchmarkName"]))
            cargo_profile = NativePluginBenchmarkProfile(str(payload["cargoProfile"]))
        except ValueError as error:
            raise CoordinatorError(
                "action_parameters_invalid",
                "Native plugin benchmark name or Cargo profile is not allow-listed",
            ) from error
        return cls(
            milestone.session_id,
            source.session_id,
            milestone.run_id,
            milestone.milestone_id,
            benchmark_name,
            cargo_profile,
        )

    def to_payload(self) -> dict[str, object]:
        return {
            "sessionId": self.session_id,
            "sourceSessionId": self.source_session_id,
            "runId": self.run_id,
            "milestoneId": self.milestone_id,
            "benchmarkName": self.benchmark_name.value,
            "cargoProfile": self.cargo_profile.value,
        }


@dataclass(frozen=True, slots=True)
class MilestoneCommitParameters(MilestoneParameters):
    """Require executor-owned change context for every Git milestone commit."""

    summary: str
    fields: ClassVar[frozenset[str]] = frozenset(
        {"sessionId", "runId", "milestoneId", "summary"}
    )

    @classmethod
    def _from_payload(cls, payload: Mapping[str, object]) -> "MilestoneCommitParameters":
        milestone = MilestoneParameters._from_payload(payload)
        summary = str(payload["summary"]).strip()
        normalized = re.sub(r"\s+", " ", summary).casefold().strip(".。")
        generic = {
            "workflow",
            "milestone",
            "complete milestone",
            "completed milestone",
            "finish milestone",
            "done",
            "完成里程碑",
            "里程碑完成",
        }
        if (
            not summary
            or len(summary) > 120
            or "\r" in summary
            or "\n" in summary
            or normalized in generic
            or re.fullmatch(
                r"(?:complete|completed|finish|finished) m[1-9]\d*(?:\.[1-9]\d*)? (?:milestone|slice)",
                normalized,
            )
        ):
            raise CoordinatorError(
                "milestone_commit_summary_invalid",
                "Milestone commit summary must describe the delivered change, not workflow completion",
            )
        return cls(milestone.session_id, milestone.run_id, milestone.milestone_id, summary)

    def to_payload(self) -> dict[str, object]:
        payload = super().to_payload()
        payload["summary"] = self.summary
        return payload


@dataclass(frozen=True, slots=True)
class MilestoneReconciliationParameters(ActionParameters):
    """Identify the two runs and immutable milestone evidence to reconcile."""

    source_run_id: str
    target_run_id: str
    milestone_ids: tuple[str, ...]
    fields: ClassVar[frozenset[str]] = frozenset(
        {"sourceRunId", "targetRunId", "milestoneIds"}
    )

    @classmethod
    def _from_payload(cls, payload: Mapping[str, object]) -> "MilestoneReconciliationParameters":
        source_run_id = str(payload["sourceRunId"]).strip()
        target_run_id = str(payload["targetRunId"]).strip()
        raw_milestones = payload["milestoneIds"]
        if (
            not source_run_id
            or not target_run_id
            or source_run_id == target_run_id
            or len(source_run_id) > 128
            or len(target_run_id) > 128
            or not re.fullmatch(r"[a-zA-Z0-9-]+", source_run_id)
            or not re.fullmatch(r"[a-zA-Z0-9-]+", target_run_id)
            or not isinstance(raw_milestones, list)
            or not raw_milestones
            or len(raw_milestones) > 64
        ):
            raise CoordinatorError(
                "action_parameters_invalid",
                "Milestone reconciliation requires two distinct runs and milestones",
            )
        milestones = tuple(str(item).strip().upper() for item in raw_milestones)
        if (
            any(not re.fullmatch(r"M[1-9]\d*", item) for item in milestones)
            or len(set(milestones)) != len(milestones)
        ):
            raise CoordinatorError(
                "action_parameters_invalid",
                "Milestone reconciliation IDs must be unique canonical milestones",
            )
        return cls(source_run_id, target_run_id, milestones)

    def to_payload(self) -> dict[str, object]:
        return {
            "sourceRunId": self.source_run_id,
            "targetRunId": self.target_run_id,
            "milestoneIds": list(self.milestone_ids),
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
        prepare = {"sessionId", "milestoneId"}
        review = {
            "sessionId", "executorSessionId", "runId", "milestoneId", "criticalCount",
            "importantCount", "summary",
        }
        if keys not in (basic, prepare, review):
            raise CoordinatorError(
                "action_parameters_invalid",
                "Topology refresh accepts a Session, a milestone prepare, or one complete review submission",
            )
        session = SessionParameters._from_payload({"sessionId": payload["sessionId"]})
        if keys == basic:
            return cls(session.session_id)
        if keys == prepare:
            milestone_id = str(payload["milestoneId"]).strip().upper()
            if _COMMIT_NODE_ID.fullmatch(milestone_id) is None:
                raise CoordinatorError(
                    "action_parameters_invalid", "Milestone or slice ID is invalid"
                )
            return cls(session.session_id, milestone_id=milestone_id)
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
            payload: dict[str, object] = {"sessionId": self.session_id}
            if self.milestone_id is not None:
                payload["milestoneId"] = self.milestone_id
            return payload
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
