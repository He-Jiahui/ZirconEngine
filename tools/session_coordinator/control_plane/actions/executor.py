from __future__ import annotations

import os
import sys
from typing import Any, Mapping

from ...models import CoordinatorError, SessionStatus
from .models import (
    ActionKind,
    ActionParameters,
    ActionSpec,
    SessionParameters,
    ValidationCancelParameters,
    ValidationStartParameters,
    ValidationTemplate,
)


class ActionExecutor:
    """Dispatches typed catalog records directly to authoritative domain services."""

    def __init__(
        self,
        *,
        sessions,
        leases,
        patches,
        failures,
        workspace_copy,
        workflows,
    ):
        self.sessions = sessions
        self.leases = leases
        self.patches = patches
        self.failures = failures
        self.workspace_copy = workspace_copy
        self.workflows = workflows

    def execute(
        self,
        spec: ActionSpec,
        parameters: ActionParameters,
        *,
        resource_snapshot: Mapping[str, object],
    ) -> dict[str, object]:
        kind = spec.kind
        if kind is ActionKind.SESSION_HEARTBEAT:
            value = self._session(parameters)
            return {"session": self.sessions.heartbeat(value.session_id).to_dict()}
        if kind is ActionKind.SESSION_ACTIVATE:
            value = self._session(parameters)
            session = self.sessions.get(value.session_id)
            if session.status is not SessionStatus.ACTIVE:
                session = self.sessions.set_status(
                    value.session_id,
                    SessionStatus.ACTIVE,
                    reason="controlled action activation",
                )
            return {"session": session.to_dict()}
        if kind is ActionKind.LEASE_CLAIM:
            value = self._session(parameters)
            session = self.sessions.get(value.session_id)
            if not session.write_scope:
                raise CoordinatorError(
                    "action_lease_scope_empty", "Session has no registered write scope"
                )
            acquisition = self.leases.acquire(value.session_id, session.write_scope)
            if not acquisition.acquired:
                raise CoordinatorError(
                    "action_lease_conflict",
                    "One or more Session write-scope paths are leased by another Session",
                    details={"conflicts": list(acquisition.conflicts)},
                )
            return {"lease": {"acquired": True, "paths": list(acquisition.paths)}}
        if kind is ActionKind.LEASE_RELEASE:
            value = self._session(parameters)
            paths = self.leases.owned_paths(value.session_id)
            released = self.leases.release(value.session_id, paths)
            return {"released": released, "paths": paths}
        if kind is ActionKind.PATCH_PROCESS:
            value = self._session(parameters)
            if self.patches is None:
                raise CoordinatorError("action_unavailable", "Patch service is unavailable")
            patch_ids = tuple(
                int(item["patch_id"])
                for item in resource_snapshot.get("patches", [])
                if isinstance(item, dict)
                and item.get("session_id") == value.session_id
                and item.get("status") == "queued"
            )
            records = self.patches.process_queue(
                session_id=value.session_id, patch_ids=patch_ids
            )
            return {
                "patches": [
                    {
                        "patchId": item.patch_id,
                        "sessionId": item.session_id,
                        "status": item.status.value,
                    }
                    for item in records
                ]
            }
        if kind is ActionKind.VALIDATION_START:
            if self.workspace_copy is None:
                raise CoordinatorError("action_unavailable", "Validation-copy service is unavailable")
            value = self._typed(parameters, ValidationStartParameters)
            manifest = self.workspace_copy.validation_manifest(value.session_id)
            record = self.workspace_copy.materialize(
                value.session_id, include_paths=manifest
            )
            started = self.workspace_copy.start(
                value.session_id,
                record.job_id,
                command=self._validation_command(value.template),
            )
            return {"copy": record.to_dict(), "validation": started}
        if kind is ActionKind.VALIDATION_CANCEL:
            if self.workspace_copy is None:
                raise CoordinatorError("action_unavailable", "Validation-copy service is unavailable")
            value = self._typed(parameters, ValidationCancelParameters)
            return {"validation": self.workspace_copy.cancel(value.session_id, value.job_id)}
        if kind is ActionKind.FAILURE_REFRESH:
            self._session(parameters)
            artifacts = resource_snapshot.get("failureArtifacts", [])
            if not isinstance(artifacts, list):
                raise CoordinatorError(
                    "action_resource_snapshot_invalid", "Failure snapshot is invalid"
                )
            audit = self.failures.import_repository(expected_artifacts=artifacts)
            return {
                "failureGraph": {
                    "nodeCount": audit.node_count,
                    "diagnosticCount": len(audit.diagnostics),
                }
            }
        if kind is ActionKind.TOPOLOGY_REFRESH:
            value = self._session(parameters)
            if self.workflows is None:
                raise CoordinatorError("action_unavailable", "Workflow store is unavailable")
            session = self.sessions.get(value.session_id)
            self.workflows.synchronize_sessions([session])
            return {"sessionId": value.session_id, "refreshed": True}
        raise CoordinatorError("action_executor_missing", "Action has no M3 executor")

    @staticmethod
    def _session(parameters: ActionParameters) -> SessionParameters:
        return ActionExecutor._typed(parameters, SessionParameters)

    @staticmethod
    def _typed(parameters: ActionParameters, expected: type[Any]):
        if not isinstance(parameters, expected):
            raise CoordinatorError("action_parameters_invalid", "Typed action parameters mismatch")
        return parameters

    @staticmethod
    def _validation_command(template: ValidationTemplate) -> tuple[str, ...]:
        if template is ValidationTemplate.COORDINATOR_ACTIONS:
            return (
                sys.executable,
                "-m",
                "unittest",
                "tools.session_coordinator.tests.test_action_catalog",
                "tools.session_coordinator.tests.test_action_auth",
                "tools.session_coordinator.tests.test_action_fingerprint",
                "tools.session_coordinator.tests.test_action_execution",
                "tools.session_coordinator.tests.test_action_concurrency",
                "-v",
            )
        if template is ValidationTemplate.WEB_CHECK:
            npm = "npm.cmd" if os.name == "nt" else "npm"
            return (npm, "--prefix", "tools/session_coordinator/web", "run", "check")
        raise CoordinatorError("action_validation_template_unknown", "Unknown validation template")
