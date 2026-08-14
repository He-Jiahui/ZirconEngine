from __future__ import annotations

import os
import sys
import threading
import uuid
from typing import Any, Callable, Mapping

from ...models import CoordinatorError, SessionStatus
from .models import (
    ActionKind,
    ActionParameters,
    ActionSpec,
    BenchmarkGrantIssueParameters,
    SessionParameters,
    GoalCloseoutParameters,
    MilestoneCommitParameters,
    MilestoneParameters,
    LifecycleParameters,
    NativePluginBenchmarkProfile,
    ValidationCancelParameters,
    ValidationStartParameters,
    ValidationTemplate,
    MilestoneReconciliationParameters,
    TopologyRefreshParameters,
)
from ...supervision.models import LifecycleKind


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
        topology_importer=None,
        milestones=None,
        lifecycle=None,
        git_finalize=None,
        codex_wake=None,
        benchmark_grants=None,
    ):
        self.sessions = sessions
        self.leases = leases
        self.patches = patches
        self.failures = failures
        self.workspace_copy = workspace_copy
        self.workflows = workflows
        self.topology_importer = topology_importer
        self.milestones = milestones
        self.lifecycle = lifecycle
        self.git_finalize = git_finalize
        self.codex_wake = codex_wake
        self.benchmark_grants = benchmark_grants
        self._deferred_completion: Callable[..., None] | None = None

    def set_deferred_completion(self, callback: Callable[..., None]) -> None:
        self._deferred_completion = callback

    def execute(
        self,
        spec: ActionSpec,
        parameters: ActionParameters,
        *,
        resource_snapshot: Mapping[str, object],
        action_id: str | None = None,
        actor: str | None = None,
    ) -> dict[str, object]:
        kind = spec.kind
        if kind is ActionKind.SESSION_HEARTBEAT:
            value = self._session(parameters)
            return {
                "session": self.sessions.heartbeat(value.session_id).to_dict(),
                "leases": {"renewed": self.leases.heartbeat(value.session_id)},
            }
        if kind is ActionKind.SESSION_ACTIVATE:
            value = self._session(parameters)
            try:
                session = self.sessions.get(value.session_id)
            except CoordinatorError as error:
                if error.code != "session_not_found" or value.maintenance_session_id is None:
                    raise
                self._require_writable_maintenance_sessions((value.maintenance_session_id,))
                session = self.sessions.register(
                    session_id=value.session_id,
                    display_name=value.display_name,
                    plan_path=value.plan_path,
                    write_scope=value.write_scope,
                )
            else:
                if (
                    value.display_name is not None
                    or value.plan_path is not None
                    or value.write_scope
                    or value.maintenance_session_id is not None
                ):
                    raise CoordinatorError(
                        "session_bootstrap_target_exists",
                        "Scoped bootstrap may only create a previously unknown Session",
                    )
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
        if kind is ActionKind.BENCHMARK_GRANT_ISSUE:
            if (
                self.workspace_copy is None
                or self.milestones is None
                or self.benchmark_grants is None
            ):
                raise CoordinatorError(
                    "action_unavailable", "Benchmark grant services are unavailable"
                )
            value = self._typed(parameters, BenchmarkGrantIssueParameters)
            paths = self._milestone_validation_paths(
                session_id=value.session_id,
                run_id=value.run_id,
                milestone_id=value.milestone_id,
                actor=actor,
                action_id=action_id,
            )
            candidate = self.benchmark_grants.select_candidate(
                source_session_id=value.source_session_id,
                target_session_id=value.session_id,
            )
            copy_scoped_hash = self.workspace_copy.scoped_manifest_hash(
                candidate.job_id, paths
            )
            current_scoped_hash = self.milestones.current_milestone_manifest_hash(
                session_id=value.session_id,
                run_id=value.run_id,
                milestone_key=value.milestone_id,
                paths=paths,
            )
            if copy_scoped_hash != current_scoped_hash:
                raise CoordinatorError(
                    "validation_copy_manifest_stale",
                    "Existing benchmark copy does not match the current milestone manifest",
                    details={
                        "copyManifestHash": copy_scoped_hash,
                        "currentManifestHash": current_scoped_hash,
                    },
                )
            validation = ValidationStartParameters(
                value.session_id,
                ValidationTemplate.NATIVE_PLUGIN_BENCHMARK,
                value.run_id,
                value.milestone_id,
                value.benchmark_name,
                value.cargo_profile,
            )
            grant = self.benchmark_grants.issue(
                candidate=candidate,
                target_session_id=value.session_id,
                run_id=value.run_id,
                milestone_id=value.milestone_id,
                benchmark_name=value.benchmark_name.value,
                cargo_profile=value.cargo_profile.value,
                command=self._validation_command(validation),
                scoped_manifest_hash=current_scoped_hash,
            )
            return {"benchmarkGrant": grant.to_dict()}
        if kind is ActionKind.VALIDATION_START:
            if self.workspace_copy is None:
                raise CoordinatorError("action_unavailable", "Validation-copy service is unavailable")
            value = self._typed(parameters, ValidationStartParameters)
            if self.milestones is None:
                raise CoordinatorError("action_unavailable", "Milestone service is unavailable")
            paths = self._milestone_validation_paths(
                session_id=value.session_id,
                run_id=value.run_id,
                milestone_id=value.milestone_id,
                actor=actor,
                action_id=action_id,
            )
            if action_id is None or self._deferred_completion is None:
                return self._start_validation(value, paths, actor=actor, action_id=action_id)
            return {
                "deferred": True,
                "validation": {"status": "materializing"},
                "_start": lambda: self._start_validation_thread(
                    action_id, value, paths, actor=actor
                ),
            }
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
            value = self._typed(parameters, TopologyRefreshParameters)
            if self.topology_importer is None:
                raise CoordinatorError("action_unavailable", "Workflow store is unavailable")
            executor_session_id = value.executor_session_id or value.session_id
            session = self.sessions.get(executor_session_id)
            if not session.plan_path:
                raise CoordinatorError("session_plan_missing", "Session has no numbered plan")
            imported = self.topology_importer.import_plan(
                executor_session_id,
                session.plan_path,
                activate_candidate=value.run_id is None,
            )
            review = None
            prepared = None
            if value.run_id is not None:
                if self.milestones is None:
                    raise CoordinatorError("action_unavailable", "Milestone service is unavailable")
                review = self.milestones.submit_review(
                    session_id=executor_session_id,
                    run_id=value.run_id,
                    milestone_key=value.milestone_id or "",
                    reviewer_session_id=value.session_id,
                    reviewer_actor=actor or "controlled-action",
                    critical_count=value.critical_count or 0,
                    important_count=value.important_count or 0,
                    summary=value.summary or "",
                    action_id=action_id,
                )
            elif value.milestone_id is not None:
                if self.milestones is None:
                    raise CoordinatorError("action_unavailable", "Milestone service is unavailable")
                prepared = self.milestones.prepare_milestone(
                    session_id=executor_session_id,
                    run_id=imported.run_id,
                    milestone_key=value.milestone_id,
                    actor=actor or "controlled-action",
                    action_id=action_id,
                )
            gates = self.milestones.refresh_gates(
                session_id=executor_session_id,
                run_id=imported.run_id,
                actor=actor or "controlled-action",
                action_id=action_id,
            ) if self.milestones is not None else {}
            return {
                "sessionId": value.session_id,
                "runId": imported.run_id,
                "refreshed": True,
                "topologyVersionId": imported.topology_version_id,
                "versionNumber": imported.version_number,
                "activated": imported.activated,
                "review": review,
                "prepared": prepared,
                "gates": gates,
            }
        if kind in {
            ActionKind.SERVICE_DRAIN,
            ActionKind.SERVICE_RESUME,
            ActionKind.SERVICE_ROLLOVER,
            ActionKind.SERVICE_STOP,
            ActionKind.SERVICE_RESTART,
            ActionKind.SERVICE_FORCE_STOP,
        }:
            if self.lifecycle is None or action_id is None:
                raise CoordinatorError("action_unavailable", "Lifecycle service is unavailable")
            value = self._typed(parameters, LifecycleParameters)
            if value.maintenance_session_ids:
                if kind is not ActionKind.SERVICE_DRAIN:
                    raise CoordinatorError(
                        "action_parameters_invalid",
                        "maintenanceSessionIds are valid only for a controlled drain",
                    )
                self._require_writable_maintenance_sessions(value.maintenance_session_ids)
            if value.maintenance_session_id is not None:
                if kind is not ActionKind.SERVICE_RESUME or not value.release_maintenance_hold:
                    raise CoordinatorError(
                        "action_parameters_invalid",
                        "maintenanceSessionId is valid only for an explicit maintenance release",
                    )
                self._require_writable_maintenance_sessions((value.maintenance_session_id,))
            if (
                value.gpu_reservation_session_id is not None
                or value.release_maintenance_hold
                or value.maintenance_hold_action_id is not None
            ):
                if kind is not ActionKind.SERVICE_RESUME:
                    raise CoordinatorError(
                        "action_parameters_invalid",
                        "GPU reservation and maintenance-release proof are only valid when resuming service mutations",
                    )
                if value.gpu_reservation_session_id is not None and self.sessions is None:
                    raise CoordinatorError("action_unavailable", "Session service is unavailable")
                if value.gpu_reservation_session_id is not None:
                    self.sessions.get(value.gpu_reservation_session_id)
            result = self.lifecycle.request(
                LifecycleKind(kind.value),
                action_id=action_id,
                actor=actor or "controlled-action",
                timeout_seconds=value.timeout_seconds,
                release_maintenance_hold=value.release_maintenance_hold,
                maintenance_hold_action_id=value.maintenance_hold_action_id,
            )
            if value.gpu_reservation_session_id is not None:
                result["gpuReservationSessionId"] = value.gpu_reservation_session_id
            return result
        if kind is ActionKind.MAINTENANCE_CLEANUP:
            if self.git_finalize is None or action_id is None:
                raise CoordinatorError("action_unavailable", "Git index cleanup is unavailable")
            return {
                "indexCleanup": self.git_finalize.cleanup_shared_index(
                    f"action:{action_id}"
                )
            }

        if kind is ActionKind.MILESTONE_COMMIT:
            if self.milestones is None:
                raise CoordinatorError("action_unavailable", "Milestone service is unavailable")
            value = self._typed(parameters, MilestoneCommitParameters)
            paths = self.milestones.milestone_paths(value.run_id, value.milestone_id)
            if not paths:
                paths = self.milestones.attributed_changes(value.session_id)
            result = self.milestones.commit(
                session_id=value.session_id,
                run_id=value.run_id,
                milestone_key=value.milestone_id,
                paths=paths,
                summary=value.summary,
                actor=actor or "controlled-action",
                action_id=action_id,
            )
            return {
                "commitSha": result.finalize.commit_sha,
                "message": result.finalize.message,
                "paths": list(paths),
                "shortstat": result.shortstat,
                "notification": (
                    {
                        "attemptId": result.notification.notification_attempt_id,
                        "status": result.notification.status,
                    }
                    if result.notification
                    else None
                ),
            }
        if kind is ActionKind.MILESTONE_RECONCILE:
            if self.milestones is None:
                raise CoordinatorError("action_unavailable", "Milestone service is unavailable")
            value = self._typed(parameters, MilestoneReconciliationParameters)
            return {
                "reconciliation": self.milestones.reconcile_accepted_milestones(
                    source_run_id=value.source_run_id,
                    target_run_id=value.target_run_id,
                    milestone_keys=value.milestone_ids,
                    actor=actor or "controlled-action",
                    action_id=action_id,
                )
            }
        if kind is ActionKind.SESSION_COMPLETE:
            if self.milestones is None:
                raise CoordinatorError("action_unavailable", "Milestone service is unavailable")
            value = self._typed(parameters, GoalCloseoutParameters)
            return self.milestones.close_goal(value.session_id, value.run_id)
        if kind is ActionKind.CODEX_RECONCILE:
            if self.codex_wake is None:
                raise CoordinatorError("action_unavailable", "Codex sync worker is unavailable")
            self.codex_wake("controlled")
            return {"queued": True, "trigger": "controlled"}
        raise CoordinatorError("action_executor_missing", "Action has no M3 executor")

    def _require_writable_maintenance_sessions(self, session_ids: tuple[str, ...]) -> None:
        if self.sessions is None:
            raise CoordinatorError("action_unavailable", "Session service is unavailable")
        for session_id in session_ids:
            session = self.sessions.get(session_id)
            if session.status not in {SessionStatus.ACTIVE, SessionStatus.RESOLVING_FAILURE}:
                raise CoordinatorError(
                    "maintenance_session_not_writable",
                    "Maintenance scope requires active or failure-resolving Sessions",
                    details={"sessionId": session_id, "status": session.status.value},
                )

    def cancel(
        self, kind: ActionKind, action_id: str, *, actor: str, reason: str
    ) -> dict[str, object]:
        if kind in {
            ActionKind.SERVICE_STOP,
            ActionKind.SERVICE_RESTART,
        }:
            if self.lifecycle is None:
                raise CoordinatorError(
                    "lifecycle_unavailable", "Lifecycle service is unavailable"
                )
            return self.lifecycle.cancel(action_id, actor=actor, reason=reason)
        raise CoordinatorError("action_not_cancellable", f"Action {kind.value} is executing")

    def _start_validation(
        self,
        value: ValidationStartParameters,
        paths: tuple[str, ...],
        *,
        actor: str | None,
        action_id: str | None,
    ) -> dict[str, object]:
        if self.workspace_copy is None or self.milestones is None:
            raise CoordinatorError("action_unavailable", "Validation services are unavailable")
        command = self._validation_command(value)
        if value.template is ValidationTemplate.NATIVE_PLUGIN_BENCHMARK:
            if self.benchmark_grants is None:
                raise CoordinatorError(
                    "action_unavailable", "Benchmark grant service is unavailable"
                )
            assert value.benchmark_name is not None
            assert value.cargo_profile is not None
            grant = self.benchmark_grants.acquire(
                target_session_id=value.session_id,
                run_id=value.run_id,
                milestone_id=value.milestone_id,
                benchmark_name=value.benchmark_name.value,
                cargo_profile=value.cargo_profile.value,
                command=command,
            )
            launched = False
            binding_created = False
            validation_run_id = uuid.uuid4().hex
            try:
                benchmark_environment = self._benchmark_environment(value, grant)
                scoped_manifest_hash = self.workspace_copy.scoped_manifest_hash(
                    grant.job_id, paths
                )
                if scoped_manifest_hash != grant.scoped_manifest_hash:
                    raise CoordinatorError(
                        "validation_copy_manifest_stale",
                        "Granted benchmark copy no longer matches its milestone manifest",
                        details={
                            "copyManifestHash": scoped_manifest_hash,
                            "grantManifestHash": grant.scoped_manifest_hash,
                        },
                    )
                self.milestones.bind_validation(
                    session_id=value.session_id,
                    run_id=value.run_id,
                    milestone_key=value.milestone_id,
                    validation_run_id=validation_run_id,
                    job_id=grant.job_id,
                    template=value.template.value,
                    source_manifest_hash=scoped_manifest_hash,
                    copy_input_manifest_hash=grant.input_manifest_hash,
                    benchmark_name=value.benchmark_name.value,
                    cargo_profile=value.cargo_profile.value,
                    benchmark_grant_id=grant.grant_id,
                    actor=actor or "controlled-action",
                    action_id=action_id,
                )
                binding_created = True
                started = self.workspace_copy.start(
                    value.session_id,
                    grant.job_id,
                    command=command,
                    run_id=validation_run_id,
                    benchmark_grant_id=grant.grant_id,
                    environment=benchmark_environment,
                )
                launched = True
                self.milestones.record_validation_process_identity(
                    validation_run_id,
                    root_pid=int(started["pid"]),
                    process_creation_time=str(started["processCreationTime"]),
                )
            except BaseException as error:
                if not launched:
                    error_code = (
                        error.code
                        if isinstance(error, CoordinatorError)
                        else "benchmark_validation_launch_failed"
                    )
                    if binding_created:
                        try:
                            self.milestones.reject_validation_launch(
                                validation_run_id, error_code=error_code
                            )
                        except Exception:
                            pass
                    self.benchmark_grants.deny(grant.grant_id, error_code=error_code)
                raise
            return {
                "copy": {
                    "jobId": grant.job_id,
                    "inputManifestHash": grant.input_manifest_hash,
                },
                "validation": started,
                "benchmarkIdentity": {
                    "rootPid": started["pid"],
                    "rootProcessCreationTime": started["processCreationTime"],
                    "runId": started["runId"],
                    "sourceManifestHash": grant.input_manifest_hash,
                    "milestoneManifestHash": scoped_manifest_hash,
                    "cargoProfile": value.cargo_profile.value,
                    "benchmarkName": value.benchmark_name.value,
                    "grantId": grant.grant_id,
                },
            }
        if value.template is ValidationTemplate.RUNTIME14_RUST_FOCUSED:
            record = self.workspace_copy.materialize_cargo(
                value.session_id,
                command=command,
                overlay_paths=paths,
                discover_external_sources=True,
            )
        else:
            record = self.workspace_copy.materialize_validation(
                value.session_id,
                dependency_roots=self._validation_dependency_roots(value.template),
                overlay_paths=paths,
            )
        validation_run_id = uuid.uuid4().hex
        source_manifest_hash = self.workspace_copy.scoped_manifest_hash(
            record.job_id, paths
        )
        self.milestones.bind_validation(
            session_id=value.session_id,
            run_id=value.run_id,
            milestone_key=value.milestone_id,
            validation_run_id=validation_run_id,
            job_id=record.job_id,
            template=value.template.value,
            source_manifest_hash=source_manifest_hash,
            actor=actor or "controlled-action",
            action_id=action_id,
        )
        started = self.workspace_copy.start(
            value.session_id,
            record.job_id,
            command=command,
            run_id=validation_run_id,
        )
        return {"copy": record.to_dict(), "validation": started}

    def _milestone_validation_paths(
        self,
        *,
        session_id: str,
        run_id: str,
        milestone_id: str,
        actor: str | None,
        action_id: str | None,
    ) -> tuple[str, ...]:
        paths = self.milestones.milestone_paths(run_id, milestone_id)
        if not paths:
            paths = self.milestones.bind_manifest(
                session_id=session_id,
                run_id=run_id,
                milestone_key=milestone_id,
                actor=actor or "controlled-action",
                action_id=action_id,
            )
        paths = tuple(paths)
        unattributed = sorted(
            set(paths) - set(self.milestones.attributed_changes(session_id)),
            key=str.casefold,
        )
        if unattributed:
            raise CoordinatorError(
                "milestone_manifest_not_attributed",
                "The immutable milestone manifest no longer belongs to this Session",
                details={"paths": unattributed},
            )
        return paths

    def _start_validation_thread(
        self,
        action_id: str,
        value: ValidationStartParameters,
        paths: tuple[str, ...],
        *,
        actor: str | None,
    ) -> None:
        callback = self._deferred_completion
        if callback is None:
            return

        def worker() -> None:
            try:
                result = self._start_validation(value, paths, actor=actor, action_id=action_id)
            except CoordinatorError as error:
                callback(action_id, value, error_code=error.code)
            except Exception:
                callback(action_id, value, error_code="action_execution_failed")
            else:
                callback(action_id, value, result=result)

        threading.Thread(
            target=worker,
            name=f"zircon-validation-materialize-{action_id[:8]}",
            daemon=True,
        ).start()

    @staticmethod
    def _session(parameters: ActionParameters) -> SessionParameters:
        return ActionExecutor._typed(parameters, SessionParameters)

    @staticmethod
    def _typed(parameters: ActionParameters, expected: type[Any]):
        if not isinstance(parameters, expected):
            raise CoordinatorError("action_parameters_invalid", "Typed action parameters mismatch")
        return parameters

    @staticmethod
    def _validation_command(
        value: ValidationStartParameters | ValidationTemplate,
    ) -> tuple[str, ...]:
        template = (
            value.template if isinstance(value, ValidationStartParameters) else value
        )
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
        if template is ValidationTemplate.RUNTIME14_RUST_FOCUSED:
            return (
                "cargo",
                "+1.94.1",
                "test",
                "-p",
                "zircon_runtime",
                "--lib",
                "runtime_14_module_family_mirror_docs_match_structure_audit_counts",
                "--locked",
                "--jobs",
                "1",
                "--",
                "--nocapture",
                "--test-threads=1",
            )
        if template is ValidationTemplate.NATIVE_PLUGIN_BENCHMARK:
            if not isinstance(value, ValidationStartParameters):
                raise CoordinatorError(
                    "action_parameters_invalid",
                    "Native plugin benchmark command requires typed validation parameters",
                )
            if value.benchmark_name is None or value.cargo_profile is None:
                raise CoordinatorError(
                    "action_parameters_invalid",
                    "Native plugin benchmark requires a name and optimized Cargo profile",
                )
            profile_arguments = (
                ("--release",)
                if value.cargo_profile is NativePluginBenchmarkProfile.RELEASE
                else ("--profile", "profiling")
            )
            return (
                "cargo",
                "+1.94.1",
                "test",
                "-p",
                "zircon_runtime",
                "--lib",
                *profile_arguments,
                "--locked",
                "--jobs",
                "1",
                value.benchmark_name.value,
                "--",
                "--ignored",
                "--exact",
                "--nocapture",
                "--test-threads=1",
            )
        raise CoordinatorError("action_validation_template_unknown", "Unknown validation template")

    @staticmethod
    def _benchmark_environment(
        value: ValidationStartParameters, record: object
    ) -> dict[str, str] | None:
        if value.template is not ValidationTemplate.NATIVE_PLUGIN_BENCHMARK:
            return None
        if value.cargo_profile is None:
            raise CoordinatorError(
                "action_parameters_invalid",
                "Native plugin benchmark Cargo profile is missing",
            )
        manifest = getattr(record, "input_manifest_hash", None)
        if manifest is None:
            raise CoordinatorError(
                "validation_benchmark_manifest_missing",
                "Materialized benchmark input manifest is missing",
            )
        if (
            not isinstance(manifest, str)
            or len(manifest) != 64
            or any(character not in "0123456789abcdef" for character in manifest)
        ):
            raise CoordinatorError(
                "validation_benchmark_manifest_invalid",
                "Materialized benchmark input manifest is invalid",
            )
        return {
            "ZR_BENCHMARK_SOURCE_MANIFEST": manifest,
            "ZR_BENCHMARK_CARGO_PROFILE": value.cargo_profile.value,
        }

    @staticmethod
    def _validation_dependency_roots(template: ValidationTemplate) -> tuple[str, ...]:
        if template is ValidationTemplate.COORDINATOR_ACTIONS:
            return ("tools/session_coordinator",)
        if template is ValidationTemplate.WEB_CHECK:
            return ("tools/session_coordinator/web",)
        raise CoordinatorError("action_validation_template_unknown", "Unknown validation template")
