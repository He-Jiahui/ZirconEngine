from __future__ import annotations

import hashlib
import json
import re
import subprocess
import uuid
from dataclasses import dataclass
from pathlib import Path

from ..baselines import BaselineService
from ..database import Database
from ..failures import WORKFLOW_NODE_ID
from ..git_finalize import FinalizeResult, GitFinalizeService
from ..models import CoordinatorError, SessionStatus, WorkflowArtifactKind, WorkflowNodeState, utc_text
from ..notifications import NotificationAttemptRecord, WeComNotificationService
from .artifacts import WorkflowArtifactStore
from .gates import GateContext, GateDecision, GateEvidenceStore, MilestoneGateEvaluator
from .store import WorkflowStore
from .topology import TopologyParser


@dataclass(frozen=True, slots=True)
class MilestoneCommitResult:
    finalize: FinalizeResult
    gate: GateDecision
    notification: NotificationAttemptRecord | None
    shortstat: str


_MODULE_NAME = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")


def plan_module_name(plan_path: str) -> str:
    """Return the safe plan-folder name used only by external notifications."""
    normalized_plan = plan_path.replace("\\", "/").rstrip("/")
    parts = [part for part in normalized_plan.split("/") if part]
    if len(parts) < 2:
        raise CoordinatorError(
            "notification_module_unavailable",
            "Milestone notification requires a plan path inside a module folder",
            details={"planPath": plan_path},
        )
    module_name = parts[-2].strip()
    if not _MODULE_NAME.fullmatch(module_name):
        raise CoordinatorError(
            "notification_module_invalid",
            "Plan module folder cannot form a safe notification prefix",
            details={"planPath": plan_path, "module": module_name},
        )
    return module_name


class MilestoneWorkflowService:
    """Join accepted workflow evidence to the authoritative scoped finalizer."""

    def __init__(
        self,
        database: Database,
        repo_root: str | Path,
        baselines: BaselineService,
        finalize: GitFinalizeService,
        notifications: WeComNotificationService | None,
        *,
        sessions=None,
        leases=None,
        failures=None,
    ):
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        self.baselines = baselines
        self.finalize = finalize
        self.notifications = notifications
        self.gates = MilestoneGateEvaluator(database)
        self.evidence = GateEvidenceStore(database)
        self.workflows = WorkflowStore(database)
        self.artifacts = WorkflowArtifactStore(database)
        self.sessions = sessions
        self.leases = leases
        self.failures = failures

    def current_milestone_manifest_hash(
        self,
        *,
        session_id: str,
        run_id: str,
        milestone_key: str,
        paths: tuple[str, ...] | list[str],
    ) -> str:
        self._require_run_owner(run_id, session_id)
        normalized_paths = tuple(paths)
        if normalized_paths != tuple(self.milestone_paths(run_id, milestone_key)):
            raise CoordinatorError(
                "milestone_manifest_paths_changed",
                "Requested paths do not match the immutable milestone manifest",
            )
        return self.prepare_context(
            run_id,
            normalized_paths,
            failure_workflow_node_keys=self._failure_node_keys(
                run_id, milestone_key
            ),
        ).manifest_hash

    def bind_validation(
        self,
        *,
        session_id: str,
        run_id: str,
        milestone_key: str,
        validation_run_id: str,
        job_id: str,
        template: str,
        source_manifest_hash: str,
        copy_input_manifest_hash: str | None = None,
        benchmark_name: str | None = None,
        cargo_profile: str | None = None,
        benchmark_grant_id: str | None = None,
        actor: str,
        action_id: str | None,
    ) -> None:
        benchmark_values = (
            copy_input_manifest_hash,
            benchmark_name,
            cargo_profile,
            benchmark_grant_id,
        )
        if template == "native-plugin-benchmark":
            if any(value is None for value in benchmark_values):
                raise CoordinatorError(
                    "validation_benchmark_binding_incomplete",
                    "Native benchmark validation requires both manifests and grant identity",
                )
            if (
                len(str(copy_input_manifest_hash)) != 64
                or any(
                    character not in "0123456789abcdef"
                    for character in str(copy_input_manifest_hash)
                )
                or cargo_profile not in {"release", "profiling"}
                or not str(benchmark_name).strip()
                or not str(benchmark_grant_id).strip()
            ):
                raise CoordinatorError(
                    "validation_benchmark_binding_invalid",
                    "Native benchmark validation identity is malformed",
                )
        elif any(value is not None for value in benchmark_values):
            raise CoordinatorError(
                "validation_benchmark_binding_unexpected",
                "Benchmark identity is valid only for the native benchmark template",
            )
        self._require_run_owner(run_id, session_id)
        paths = self.milestone_paths(run_id, milestone_key)
        if not paths:
            paths = self.bind_manifest(
                session_id=session_id,
                run_id=run_id,
                milestone_key=milestone_key,
                actor=actor,
                action_id=action_id,
            )
        failure_node_keys = self._failure_node_keys(run_id, milestone_key)
        context = self.prepare_context(
            run_id,
            paths,
            failure_workflow_node_keys=failure_node_keys,
        )
        if source_manifest_hash != context.manifest_hash:
            raise CoordinatorError(
                "validation_copy_manifest_stale",
                "Validation copy does not match the current milestone manifest",
                details={
                    "copyManifestHash": source_manifest_hash,
                    "currentManifestHash": context.manifest_hash,
                },
            )
        milestone = self._milestone_node(run_id, milestone_key)
        fingerprint = self.gates.input_fingerprint(run_id, milestone_key, context)
        legacy_template = (
            template if template in {"coordinator-actions", "web-check"} else "coordinator-actions"
        )
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO workflow_validation_bindings(
                       validation_run_id, job_id, run_id, topology_version_id,
                       node_id, session_id, template, source_manifest_hash, paths_json,
                       input_fingerprint, copy_input_manifest_hash, benchmark_name,
                       cargo_profile, benchmark_grant_id,
                       action_id, actor, created_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    validation_run_id,
                    job_id,
                    run_id,
                    context.topology_version_id,
                    milestone["node_id"],
                    session_id,
                    legacy_template,
                    source_manifest_hash,
                    json.dumps(paths),
                    fingerprint,
                    copy_input_manifest_hash,
                    benchmark_name,
                    cargo_profile,
                    benchmark_grant_id,
                    action_id,
                    actor,
                    utc_text(),
                ),
            )
            connection.execute(
                """INSERT INTO workflow_validation_template_bindings(
                       validation_run_id, template
                   ) VALUES (?, ?)""",
                (validation_run_id, template),
            )
        self.import_validation_result(validation_run_id)

    def record_validation_process_identity(
        self,
        validation_run_id: str,
        *,
        root_pid: int,
        process_creation_time: str,
    ) -> None:
        if not isinstance(root_pid, int) or root_pid <= 0:
            raise CoordinatorError(
                "validation_benchmark_root_pid_invalid",
                "Native benchmark root process ID must be positive",
            )
        if not isinstance(process_creation_time, str) or not process_creation_time:
            raise CoordinatorError(
                "validation_benchmark_process_creation_time_invalid",
                "Native benchmark process creation time must be recorded",
            )
        with self.database.transaction() as connection:
            cursor = connection.execute(
                """UPDATE workflow_validation_bindings
                   SET root_pid=?, root_process_creation_time=?
                   WHERE validation_run_id=?
                     AND benchmark_grant_id IS NOT NULL
                     AND copy_input_manifest_hash IS NOT NULL
                     AND root_pid IS NULL
                     AND root_process_creation_time IS NULL""",
                (root_pid, process_creation_time, validation_run_id),
            )
            if cursor.rowcount == 1:
                return
            existing = connection.execute(
                """SELECT root_pid, root_process_creation_time
                   FROM workflow_validation_bindings
                   WHERE validation_run_id=? AND benchmark_grant_id IS NOT NULL""",
                (validation_run_id,),
            ).fetchone()
            if (
                existing is None
                or existing["root_pid"] != root_pid
                or existing["root_process_creation_time"] != process_creation_time
            ):
                raise CoordinatorError(
                    "validation_benchmark_process_identity_unavailable",
                    "Benchmark validation binding is missing or has another root process",
                )

    def reject_validation_launch(
        self, validation_run_id: str, *, error_code: str
    ) -> bool:
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT * FROM workflow_validation_bindings
                   WHERE validation_run_id=? AND imported_at IS NULL""",
                (validation_run_id,),
            ).fetchone()
        if row is None:
            return False
        return self._reject_validation_binding(row, error_code)

    def import_validation_result(self, validation_run_id: str) -> bool:
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT binding.*, template_binding.template AS exact_template,
                          validation.exit_code, validation.command_json,
                          validation.stdout_text, validation.stderr_text,
                          validation.completed_at, node.node_key, copy.source_root,
                          copy.input_manifest_hash AS current_copy_input_manifest_hash
                   FROM workflow_validation_bindings binding
                   LEFT JOIN validation_copy_runs validation
                     ON validation.run_id=binding.validation_run_id
                   LEFT JOIN workflow_validation_template_bindings template_binding
                     ON template_binding.validation_run_id=binding.validation_run_id
                   JOIN validation_copies copy ON copy.job_id=binding.job_id
                   JOIN workflow_nodes node ON node.node_id=binding.node_id
                   WHERE binding.validation_run_id=?""",
                (validation_run_id,),
            ).fetchone()
        if row is None or row["exit_code"] is None or row["imported_at"] is not None:
            return False
        if not row["source_manifest_hash"] or not json.loads(row["paths_json"]):
            return self._reject_validation_binding(row, "validation_binding_legacy_unbound")
        if (
            row["copy_input_manifest_hash"] is not None
            and row["current_copy_input_manifest_hash"]
            != row["copy_input_manifest_hash"]
        ):
            return self._reject_validation_binding(
                row, "validation_copy_input_manifest_changed"
            )
        current_copy_hash = self._manifest_hash_at(
            Path(row["source_root"]), tuple(json.loads(row["paths_json"]))
        )
        if current_copy_hash != row["source_manifest_hash"]:
            return self._reject_validation_binding(row, "validation_copy_manifest_changed")
        accepted = int(row["exit_code"]) == 0
        self.evidence.record_gate(
            run_id=row["run_id"],
            topology_version_id=row["topology_version_id"],
            node_id=row["node_id"],
            gate_kind="validation",
            decision="accepted" if accepted else "rejected",
            decision_code=(
                "managed_validation_succeeded" if accepted else "managed_validation_failed"
            ),
            input_fingerprint=row["input_fingerprint"],
            actor=row["actor"],
            action_id=row["action_id"],
            source_revision=validation_run_id,
            payload={
                "validationRunId": validation_run_id,
                "jobId": row["job_id"],
                "template": row["exact_template"] or row["template"],
                "command": json.loads(row["command_json"]),
                "exitCode": int(row["exit_code"]),
                "completedAt": row["completed_at"],
                **(
                    {
                        "benchmarkIdentity": {
                            "sourceManifestHash": row["copy_input_manifest_hash"],
                            "milestoneManifestHash": row["source_manifest_hash"],
                            "benchmarkName": row["benchmark_name"],
                            "cargoProfile": row["cargo_profile"],
                            "grantId": row["benchmark_grant_id"],
                            "rootPid": row["root_pid"],
                            "rootProcessCreationTime": row["root_process_creation_time"],
                        }
                    }
                    if row["benchmark_grant_id"] is not None
                    else {}
                ),
            },
        )
        with self.database.transaction() as connection:
            connection.execute(
                """UPDATE workflow_validation_bindings
                   SET imported_at=?, terminal_status=?, terminal_code=?
                   WHERE validation_run_id=? AND imported_at IS NULL""",
                (
                    utc_text(),
                    "accepted" if accepted else "rejected",
                    "managed_validation_succeeded" if accepted else "managed_validation_failed",
                    validation_run_id,
                ),
            )
            connection.execute(
                "UPDATE workflow_runs SET updated_at=? WHERE run_id=?",
                (utc_text(), row["run_id"]),
            )
        return True

    def recover_validation_results(self) -> tuple[str, ...]:
        with self.database.connect() as connection:
            run_ids = [
                row[0]
                for row in connection.execute(
                    """SELECT binding.validation_run_id
                       FROM workflow_validation_bindings binding
                       JOIN validation_copy_runs validation
                         ON validation.run_id=binding.validation_run_id
                       WHERE binding.imported_at IS NULL ORDER BY validation.completed_at"""
                )
            ]
        recovered: list[str] = []
        for run_id in run_ids:
            try:
                if self.import_validation_result(run_id):
                    recovered.append(run_id)
            except Exception:
                continue
        return tuple(recovered)

    def _reject_validation_binding(self, row, code: str) -> bool:
        self.evidence.record_gate(
            run_id=row["run_id"],
            topology_version_id=row["topology_version_id"],
            node_id=row["node_id"],
            gate_kind="validation",
            decision="rejected",
            decision_code=code,
            input_fingerprint=row["input_fingerprint"],
            actor=row["actor"],
            action_id=row["action_id"],
            source_revision=row["validation_run_id"],
            payload={"validationRunId": row["validation_run_id"], "jobId": row["job_id"]},
        )
        with self.database.transaction() as connection:
            connection.execute(
                """UPDATE workflow_validation_bindings
                   SET imported_at=?, terminal_status='rejected', terminal_code=?
                   WHERE validation_run_id=? AND imported_at IS NULL""",
                (utc_text(), code, row["validation_run_id"]),
            )
        return True

    def submit_review(
        self,
        *,
        session_id: str,
        run_id: str,
        milestone_key: str,
        reviewer_session_id: str,
        reviewer_actor: str,
        critical_count: int,
        important_count: int,
        summary: str,
        action_id: str | None,
    ) -> dict[str, object]:
        self._require_run_owner(run_id, session_id)
        if reviewer_session_id == session_id:
            raise CoordinatorError(
                "workflow_review_not_independent",
                "Review actor must differ from the executing Session",
            )
        if self.sessions is None:
            raise CoordinatorError("action_unavailable", "Session service is unavailable")
        self.sessions.get(reviewer_session_id)
        paths = self.milestone_paths(run_id, milestone_key)
        if not paths:
            paths = self.bind_manifest(
                session_id=session_id,
                run_id=run_id,
                milestone_key=milestone_key,
                actor=reviewer_actor,
                action_id=action_id,
            )
        failure_node_keys = self._failure_node_keys(run_id, milestone_key)
        context = self.prepare_context(
            run_id,
            paths,
            failure_workflow_node_keys=failure_node_keys,
        )
        milestone = self._milestone_node(run_id, milestone_key)
        fingerprint = self.gates.input_fingerprint(run_id, milestone_key, context)
        review = self.evidence.record_review(
            run_id=run_id,
            topology_version_id=context.topology_version_id,
            reviewer=reviewer_session_id,
            executor=session_id,
            critical_count=critical_count,
            important_count=important_count,
            summary=summary,
            input_fingerprint=fingerprint,
            node_id=milestone["node_id"],
        )
        self.evidence.record_gate(
            run_id=run_id,
            topology_version_id=context.topology_version_id,
            node_id=milestone["node_id"],
            gate_kind="review",
            decision=review.verdict,
            decision_code=(
                "independent_review_accepted"
                if review.verdict == "accepted"
                else "independent_review_findings_open"
            ),
            input_fingerprint=fingerprint,
            actor=reviewer_actor,
            action_id=action_id,
            payload={
                "reviewId": review.review_id,
                "reviewerSessionId": reviewer_session_id,
                "reviewerActor": reviewer_actor,
                "criticalCount": review.critical_count,
                "importantCount": review.important_count,
            },
        )
        return {
            "reviewId": review.review_id,
            "verdict": review.verdict,
            "criticalCount": review.critical_count,
            "importantCount": review.important_count,
            "inputFingerprint": fingerprint,
        }

    def refresh_gates(
        self,
        *,
        session_id: str,
        run_id: str,
        actor: str,
        action_id: str | None,
    ) -> dict[str, object]:
        """Derive non-validation gates from authoritative repository/service state."""
        self._require_run_owner(run_id, session_id)
        if self.failures is not None:
            self.failures.import_repository()
        session = self.sessions.get(session_id) if self.sessions is not None else None
        plan_path = session.plan_path if session is not None else None
        owned = set(self.leases.owned_paths(session_id)) if self.leases is not None else set()
        owner = TopologyParser(self.repo_root).plans.resolve_owner(plan_path or "")
        child = self.repo_root / owner.child_dir
        output_files = tuple(
            sorted(
                (
                    path.relative_to(self.repo_root).as_posix()
                    for path in child.glob("*.md")
                    if not path.name.startswith(("failure-", "fixed-"))
                ),
                key=str.casefold,
            )
        ) if child.is_dir() else ()
        refreshed: dict[str, dict[str, str]] = {}
        with self.database.connect() as connection:
            milestones = connection.execute(
                """SELECT node_id, node_key, kind FROM workflow_nodes
                   WHERE run_id=? AND kind IN ('milestone', 'slice') ORDER BY node_key""",
                (run_id,),
            ).fetchall()
        for milestone in milestones:
            paths = self.milestone_paths(run_id, milestone["node_key"])
            if not paths:
                refreshed[milestone["node_key"]] = {
                    "commit_manifest": "rejected",
                    "plan_output": "rejected",
                    "review": "rejected",
                    "failure_audit": "rejected",
                }
                continue
            failure_node_keys = self._failure_node_keys(run_id, milestone["node_key"])
            failures = self.open_failures_for_milestone(
                run_id=run_id,
                milestone_key=milestone["node_key"],
                paths=paths,
            )
            context = self.prepare_context(
                run_id,
                paths,
                failure_workflow_node_keys=failure_node_keys,
            )
            fingerprint = self.gates.input_fingerprint(
                run_id, milestone["node_key"], context
            )
            plan_records = tuple(
                path
                for path in output_files
                if self._valid_plan_output(
                    self.repo_root / path,
                    plan_path or "",
                    milestone["node_key"],
                )
            )
            manifest_ok = bool(paths) and set(paths) <= owned
            decisions = {
                "failure_audit": (
                    not failures,
                    "failure_graph_clear" if not failures else "failure_graph_open",
                ),
                "plan_output": (
                    bool(plan_records),
                    "plan_output_present" if plan_records else "plan_output_missing",
                ),
                "commit_manifest": (
                    manifest_ok,
                    "commit_manifest_owned" if manifest_ok else "commit_manifest_unleased",
                ),
            }
            with self.database.connect() as connection:
                review = connection.execute(
                    """SELECT review_id, verdict FROM workflow_review_evidence
                       WHERE run_id=? AND topology_version_id=? AND node_id=?
                         AND input_fingerprint=?
                       ORDER BY created_at DESC, review_id DESC LIMIT 1""",
                    (
                        run_id,
                        context.topology_version_id,
                        milestone["node_id"],
                        fingerprint,
                    ),
                ).fetchone()
            decisions["review"] = (
                review is not None and review["verdict"] == "accepted",
                "independent_review_accepted"
                if review is not None and review["verdict"] == "accepted"
                else "independent_review_missing_or_rejected",
            )
            refreshed[milestone["node_key"]] = {}
            for gate_kind, (accepted, code) in decisions.items():
                self.evidence.record_gate(
                    run_id=run_id,
                    topology_version_id=context.topology_version_id,
                    node_id=milestone["node_id"],
                    gate_kind=gate_kind,
                    decision="accepted" if accepted else "rejected",
                    decision_code=code,
                    input_fingerprint=fingerprint,
                    actor=actor,
                    action_id=action_id,
                    applicable_failure_ids=tuple(
                        item.lifecycle_key for item in failures
                    ),
                    payload={"paths": list(paths), "planRecords": list(plan_records)},
                )
                refreshed[milestone["node_key"]][gate_kind] = (
                    "accepted" if accepted else "rejected"
                )
        return {"refreshed": True, "milestones": refreshed}

    def prepare_context(
        self,
        run_id: str,
        paths: list[str] | tuple[str, ...],
        *,
        failure_workflow_node_keys: tuple[str, ...],
    ) -> GateContext:
        normalized = tuple(sorted(set(paths), key=str.casefold))
        if not normalized:
            raise CoordinatorError("milestone_paths_empty", "Milestone commit requires paths")
        if (
            not isinstance(failure_workflow_node_keys, tuple)
            or not failure_workflow_node_keys
            or any(
                not isinstance(node_key, str)
                or WORKFLOW_NODE_ID.fullmatch(node_key) is None
                for node_key in failure_workflow_node_keys
            )
        ):
            raise CoordinatorError(
                "milestone_failure_scope_invalid",
                "Milestone context requires explicit workflow node Failure scope",
            )
        with self.database.connect() as connection:
            run = connection.execute(
                """SELECT workflow_runs.current_topology_version_id,
                          versions.topology_hash, versions.plan_path
                   FROM workflow_runs
                   LEFT JOIN workflow_topology_versions versions
                     ON versions.topology_version_id=workflow_runs.current_topology_version_id
                   WHERE workflow_runs.run_id=?""",
                (run_id,),
            ).fetchone()
            if run is None or run["current_topology_version_id"] is None:
                raise CoordinatorError(
                    "workflow_topology_not_active", "Workflow has no active topology version"
                )
            current_plan = TopologyParser(self.repo_root).parse(run["plan_path"])
            if current_plan.topology_hash != run["topology_hash"]:
                raise CoordinatorError(
                    "workflow_topology_plan_changed",
                    "Plan topology changed after the active topology version was imported",
                    details={
                        "activeTopologyHash": run["topology_hash"],
                        "currentTopologyHash": current_plan.topology_hash,
                    },
                )
            plan_path = run["plan_path"]
        if self.failures is not None:
            open_failures = self.failures.open_for_manifest(
                plan_path,
                failure_workflow_node_keys,
                normalized,
            )
            applicable, deferrals = self._apply_failure_deferrals(
                run_id,
                failure_workflow_node_keys,
                open_failures,
            )
            failure_rows = [
                {
                    "lifecycle_key": item.lifecycle_key,
                    "status": item.status,
                    "kind": item.kind,
                    "artifact_path": item.artifact_path,
                }
                for item in applicable
            ] + list(deferrals)
        else:
            placeholders = ", ".join("?" for _ in failure_workflow_node_keys)
            with self.database.connect() as connection:
                failure_rows = [
                    dict(row)
                    for row in connection.execute(
                        f"""SELECT lifecycle_key, status, kind, artifact_path
                            FROM failure_nodes
                            WHERE fixing_plan=?
                               OR (
                                 origin_plan=?
                                 AND (
                                   origin_workflow_node IS NULL
                                   OR origin_workflow_node IN ({placeholders})
                                 )
                               )
                            ORDER BY lifecycle_key, artifact_path""",
                        (
                            plan_path,
                            plan_path,
                            *failure_workflow_node_keys,
                        ),
                    )
                ]
        baseline = self.baselines.current()
        return GateContext(
            topology_version_id=run["current_topology_version_id"],
            head_commit=self._git("rev-parse", "HEAD"),
            baseline_epoch=baseline.epoch_id,
            manifest_hash=self._manifest_hash(normalized),
            failure_revision=_hash_json(failure_rows),
            plan_topology_hash=current_plan.topology_hash,
        )

    def defer_failure(
        self,
        *,
        session_id: str,
        source_milestone_key: str,
        target_milestone_key: str,
        failure_lifecycle_key: str,
        actor: str,
        action_id: str | None,
    ) -> dict[str, object]:
        """Persist one strict-successor Failure deferral bound to plan topology.

        The durable identity deliberately omits a consuming workflow run id. A
        topology import changes the semantic hash and therefore makes the old
        decision inapplicable without mutating or deleting its audit record.
        """
        with self.database.transaction() as connection:
            session = connection.execute(
                "SELECT plan_path FROM sessions WHERE session_id=?",
                (session_id,),
            ).fetchone()
            run = connection.execute(
                """SELECT run.run_id, version.plan_path, version.topology_hash,
                          run.current_topology_version_id
                   FROM workflow_runs run
                   JOIN workflow_topology_versions version
                     ON version.topology_version_id=run.current_topology_version_id
                   WHERE run.session_id=?
                   ORDER BY run.updated_at DESC, run.run_id DESC LIMIT 1""",
                (session_id,),
            ).fetchone()
            if run is None or session is None:
                raise CoordinatorError(
                    "workflow_run_owner_mismatch",
                    "Failure deferral requires the executor Session's active workflow",
                )
            plan_path = str(run["plan_path"])
            if session["plan_path"] != plan_path:
                raise CoordinatorError(
                    "milestone_failure_deferral_plan_mismatch",
                    "Failure deferral must use the executor Session's numbered plan",
                )
            nodes = connection.execute(
                """SELECT node_id, node_key FROM workflow_nodes
                   WHERE run_id=? AND node_key IN (?, ?) AND kind='milestone'""",
                (
                    run["run_id"],
                    source_milestone_key,
                    target_milestone_key,
                ),
            ).fetchall()
            by_key = {str(row["node_key"]): str(row["node_id"]) for row in nodes}
            if set(by_key) != {source_milestone_key, target_milestone_key}:
                raise CoordinatorError(
                    "milestone_failure_deferral_target_invalid",
                    "Failure deferral requires two current-topology milestones",
                )
            reachable = connection.execute(
                """WITH RECURSIVE successors(node_id) AS (
                       SELECT edge.to_node_id
                       FROM workflow_edges edge
                       WHERE edge.run_id=? AND edge.from_node_id=?
                       UNION
                       SELECT edge.to_node_id
                       FROM workflow_edges edge
                       JOIN successors prior ON prior.node_id=edge.from_node_id
                       WHERE edge.run_id=?
                   )
                   SELECT 1 FROM successors WHERE node_id=? LIMIT 1""",
                (
                    run["run_id"],
                    by_key[source_milestone_key],
                    run["run_id"],
                    by_key[target_milestone_key],
                ),
            ).fetchone()
            if reachable is None:
                raise CoordinatorError(
                    "milestone_failure_deferral_target_invalid",
                    "Failure deferral target must be a strict reachable successor",
                )
            failure = connection.execute(
                """SELECT lifecycle_key FROM failure_nodes
                   WHERE lifecycle_key=? AND kind='failure' AND status='open'
                     AND fixing_plan=?""",
                (failure_lifecycle_key, plan_path),
            ).fetchone()
            if failure is None:
                raise CoordinatorError(
                    "milestone_failure_deferral_failure_invalid",
                    "Failure deferral requires an open lifecycle owned by the executor plan",
                )
            existing = connection.execute(
                """SELECT * FROM workflow_failure_deferrals
                   WHERE session_id=? AND plan_path=? AND topology_hash=?
                     AND source_milestone_key=? AND failure_lifecycle_key=?""",
                (
                    session_id,
                    plan_path,
                    run["topology_hash"],
                    source_milestone_key,
                    failure_lifecycle_key,
                ),
            ).fetchone()
            if existing is not None:
                if existing["target_milestone_key"] != target_milestone_key:
                    raise CoordinatorError(
                        "milestone_failure_deferral_conflict",
                        "Failure is already deferred to another milestone in this topology",
                    )
                row = existing
            else:
                deferral_id = uuid.uuid4().hex
                connection.execute(
                    """INSERT INTO workflow_failure_deferrals(
                           deferral_id, session_id, plan_path, topology_hash,
                           source_milestone_key, target_milestone_key,
                           failure_lifecycle_key, actor, action_id, created_at
                       ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                    (
                        deferral_id,
                        session_id,
                        plan_path,
                        run["topology_hash"],
                        source_milestone_key,
                        target_milestone_key,
                        failure_lifecycle_key,
                        actor,
                        action_id,
                        utc_text(),
                    ),
                )
                row = connection.execute(
                    "SELECT * FROM workflow_failure_deferrals WHERE deferral_id=?",
                    (deferral_id,),
                ).fetchone()
        return {
            "deferralId": row["deferral_id"],
            "sourceMilestoneId": row["source_milestone_key"],
            "targetMilestoneId": row["target_milestone_key"],
            "failureLifecycleKey": row["failure_lifecycle_key"],
            "topologyHash": row["topology_hash"],
        }

    def open_failures_for_milestone(
        self,
        *,
        run_id: str,
        milestone_key: str,
        paths: tuple[str, ...] | list[str],
    ):
        if self.failures is None:
            return []
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT version.plan_path
                   FROM workflow_runs run
                   JOIN workflow_topology_versions version
                     ON version.topology_version_id=run.current_topology_version_id
                   WHERE run.run_id=?""",
                (run_id,),
            ).fetchone()
        if row is None:
            raise CoordinatorError(
                "workflow_topology_not_active", "Workflow has no active topology version"
            )
        keys = self._failure_node_keys(run_id, milestone_key)
        failures = self.failures.open_for_manifest(str(row["plan_path"]), keys, paths)
        applicable, _ = self._apply_failure_deferrals(run_id, keys, failures)
        return applicable

    def _apply_failure_deferrals(
        self,
        run_id: str,
        failure_node_keys: tuple[str, ...],
        failures,
    ):
        placeholders = ", ".join("?" for _ in failure_node_keys)
        with self.database.connect() as connection:
            run = connection.execute(
                """SELECT run.session_id, version.plan_path, version.topology_hash
                   FROM workflow_runs run
                   JOIN workflow_topology_versions version
                     ON version.topology_version_id=run.current_topology_version_id
                   WHERE run.run_id=?""",
                (run_id,),
            ).fetchone()
        if run is None:
            rows = ()
        else:
            current_topology_hash = TopologyParser(self.repo_root).parse(
                run["plan_path"]
            ).topology_hash
            with self.database.connect() as connection:
                rows = connection.execute(
                    f"""SELECT * FROM workflow_failure_deferrals
                        WHERE session_id=? AND plan_path=? AND topology_hash=?
                          AND source_milestone_key IN ({placeholders})
                        ORDER BY failure_lifecycle_key""",
                    (
                        run["session_id"],
                        run["plan_path"],
                        current_topology_hash,
                        *failure_node_keys,
                    ),
                ).fetchall()
        deferred = {str(row["failure_lifecycle_key"]) for row in rows}
        applicable = [item for item in failures if item.lifecycle_key not in deferred]
        evidence = tuple(
            {
                "lifecycle_key": row["failure_lifecycle_key"],
                "status": "deferred",
                "kind": "deferral",
                "artifact_path": row["target_milestone_key"],
            }
            for row in rows
        )
        return applicable, evidence

    def attributed_changes(self, session_id: str) -> tuple[str, ...]:
        with self.database.connect() as connection:
            attributed = {
                row[0]
                for row in connection.execute(
                    "SELECT display_path FROM attributions WHERE session_id=?",
                    (session_id,),
                )
            }
        tracked = self._git_paths(
            "diff", "--name-only", "-z", "HEAD", "--"
        )
        untracked = self._git_paths(
            "ls-files", "--others", "--exclude-standard", "-z", "--"
        )
        changed_paths = set(tracked) | set(untracked)
        return tuple(sorted(attributed & changed_paths, key=str.casefold))

    def owned_scope_dirty(self, session_id: str) -> tuple[str, ...]:
        session = self.sessions.get(session_id) if self.sessions is not None else None
        scopes = set(session.write_scope if session is not None else ())
        if self.leases is not None:
            scopes.update(self.leases.owned_paths(session_id))
        with self.database.connect() as connection:
            scopes.update(
                row[0]
                for row in connection.execute(
                    "SELECT display_path FROM attributions WHERE session_id=?", (session_id,)
                )
            )
        tracked = set(self._git_paths("diff", "--name-only", "-z", "HEAD", "--"))
        untracked = set(self._git_paths("ls-files", "--others", "--exclude-standard", "-z", "--"))
        dirty = tracked | untracked

        def covered(path: str) -> bool:
            folded = path.casefold()
            for raw in scopes:
                scope = raw.replace("\\", "/").strip("/").casefold()
                if scope and (folded == scope or folded.startswith(scope + "/")):
                    return True
            return False

        return tuple(sorted((path for path in dirty if covered(path)), key=str.casefold))

    def milestone_paths(self, run_id: str, milestone_key: str) -> tuple[str, ...]:
        milestone = self._milestone_node(run_id, milestone_key)
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT manifest.paths_json
                   FROM workflow_milestone_manifests manifest
                   JOIN workflow_runs run ON run.run_id=manifest.run_id
                   WHERE manifest.run_id=? AND manifest.node_id=?
                     AND manifest.topology_version_id=run.current_topology_version_id
                   ORDER BY manifest.created_at DESC, manifest.manifest_id DESC LIMIT 1""",
                (run_id, milestone["node_id"]),
            ).fetchone()
        return tuple(json.loads(row["paths_json"])) if row is not None else ()

    def bind_manifest(
        self,
        *,
        session_id: str,
        run_id: str,
        milestone_key: str,
        actor: str,
        action_id: str | None,
    ) -> tuple[str, ...]:
        prepared = self.prepare_milestone(
            session_id=session_id,
            run_id=run_id,
            milestone_key=milestone_key,
            actor=actor,
            action_id=action_id,
        )
        return tuple(prepared["paths"])

    def prepare_milestone(
        self,
        *,
        session_id: str,
        run_id: str,
        milestone_key: str,
        actor: str,
        action_id: str | None,
    ) -> dict[str, object]:
        """Bind and expose one exact current-version milestone manifest."""
        self._require_run_owner(run_id, session_id)
        paths = self._derive_milestone_paths(session_id, run_id, milestone_key)
        failure_node_keys = self._failure_node_keys(run_id, milestone_key)
        context = self.prepare_context(
            run_id,
            paths,
            failure_workflow_node_keys=failure_node_keys,
        )
        milestone = self._milestone_node(run_id, milestone_key)
        manifest_id = self._record_manifest(
            session_id=session_id,
            run_id=run_id,
            milestone=milestone,
            paths=paths,
            context=context,
            actor=actor,
            action_id=action_id,
        )
        return {
            "milestoneId": milestone_key,
            "nodeId": str(milestone["node_id"]),
            "topologyVersionId": context.topology_version_id,
            "manifestId": manifest_id,
            "manifestHash": context.manifest_hash,
            "paths": list(paths),
        }

    def live_eligibility(self, run_id: str, milestone_key: str) -> dict[str, object]:
        paths = self.milestone_paths(run_id, milestone_key)
        if not paths:
            return {
                "eligible": False,
                "code": "milestone_manifest_missing",
                "missing": ["commit_manifest"],
                "rejected": [],
                "fingerprintConsistent": True,
                "independentReviewAccepted": False,
            }
        try:
            failure_node_keys = self._failure_node_keys(run_id, milestone_key)
            context = self.prepare_context(
                run_id,
                paths,
                failure_workflow_node_keys=failure_node_keys,
            )
            decision = self.gates.evaluate(run_id, milestone_key, context)
        except CoordinatorError as error:
            return {
                "eligible": False,
                "code": error.code,
                "missing": [],
                "rejected": [],
                "fingerprintConsistent": False,
                "independentReviewAccepted": False,
            }
        return {
            "eligible": decision.allowed,
            "code": decision.code,
            "missing": list(decision.required_evidence),
            "rejected": list(decision.blocking_node_ids),
            "fingerprintConsistent": decision.code != "milestone_gate_stale_evidence",
            "independentReviewAccepted": "review" not in decision.required_evidence,
        }

    def commit(
        self,
        *,
        session_id: str,
        run_id: str,
        milestone_key: str,
        paths: list[str] | tuple[str, ...],
        summary: str,
        actor: str,
        action_id: str | None = None,
    ) -> MilestoneCommitResult:
        self._require_run_owner(run_id, session_id)
        with self.database.connect() as connection:
            run = connection.execute(
                "SELECT plan_path FROM workflow_runs WHERE run_id=?", (run_id,)
            ).fetchone()
        if run is None or not run["plan_path"]:
            raise CoordinatorError(
                "milestone_commit_context_unavailable",
                "Milestone commit requires a workflow run bound to a plan module",
                details={"runId": run_id},
            )
        plan_path = str(run["plan_path"])
        module = plan_module_name(plan_path)
        bound_paths = self.milestone_paths(run_id, milestone_key)
        if bound_paths:
            if tuple(sorted(paths, key=str.casefold)) != tuple(sorted(bound_paths, key=str.casefold)):
                raise CoordinatorError(
                    "milestone_manifest_scope_changed",
                    "Commit paths differ from the milestone-bound manifest",
                    details={"boundPaths": list(bound_paths), "requestedPaths": list(paths)},
                )
            paths = bound_paths
        else:
            raise CoordinatorError(
                "milestone_manifest_missing",
                "Milestone commit requires an explicit service-bound manifest",
            )
        failure_workflow_node_keys = self._failure_node_keys(
            run_id, milestone_key
        )
        context = self.prepare_context(
            run_id,
            paths,
            failure_workflow_node_keys=failure_workflow_node_keys,
        )
        initial = self.gates.evaluate(run_id, milestone_key, context)
        self._require_allowed(initial)

        latest: GateDecision = initial
        milestone = self._milestone_node(run_id, milestone_key)
        message = self._commit_subject(module, paths, summary)
        intent_id = uuid.uuid4().hex
        now = utc_text()
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO workflow_commit_intents(
                       intent_id, run_id, topology_version_id, node_id,
                       session_id, action_id, actor, gate_fingerprint,
                       paths_json, message, status, created_at, updated_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'prepared', ?, ?)""",
                (
                    intent_id,
                    run_id,
                    context.topology_version_id,
                    milestone["node_id"],
                    session_id,
                    action_id,
                    actor,
                    initial.input_fingerprint,
                    json.dumps(sorted(paths, key=str.casefold)),
                    message,
                    now,
                    now,
                ),
            )

        def guard() -> None:
            nonlocal latest
            refreshed_context = self.prepare_context(
                run_id,
                paths,
                failure_workflow_node_keys=failure_workflow_node_keys,
            )
            latest = self.gates.evaluate(run_id, milestone_key, refreshed_context)
            self._require_allowed(latest)

        try:
            result = self.finalize.commit_milestone(
                session_id,
                paths=paths,
                message=message,
                failure_workflow_node_keys=failure_workflow_node_keys,
                precommit_guard=guard,
                request_id=intent_id,
            )
        except BaseException:
            recovered = self._reconcile_intent(intent_id)
            if recovered is None:
                with self.database.transaction() as connection:
                    connection.execute(
                        """UPDATE workflow_commit_intents
                           SET status='failed', error_text='finalize failed before ref update', updated_at=?
                           WHERE intent_id=? AND status='prepared'""",
                        (utc_text(), intent_id),
                    )
                raise
            result = recovered
        shortstat = self._git("show", "--shortstat", "--format=", result.commit_sha).strip()
        self._reconcile_intent(
            intent_id,
            result=result,
            shortstat=shortstat,
            gate_fingerprint=latest.input_fingerprint,
        )

        notification = None
        if self.notifications is not None:
            try:
                commit_time = self._git("show", "-s", "--format=%cI", result.commit_sha)
                commit_subject = self._git("show", "-s", "--format=%s", result.commit_sha)
                formatted = self.notifications.format_message(
                    module=module,
                    summary=f"{milestone_key} · {milestone['title']}：{summary}",
                    commit_time=commit_time,
                    shortstat=shortstat or "0 files changed",
                    commit_content=f"{result.commit_sha} {commit_subject}",
                )
                notification = self.notifications.notify_once(
                    commit_sha=result.commit_sha,
                    message=formatted,
                    run_id=run_id,
                    topology_version_id=context.topology_version_id,
                    node_id=milestone["node_id"],
                    action_id=action_id,
                )
            except Exception as error:
                notification = self.notifications.record_post_commit_failure(
                    commit_sha=result.commit_sha,
                    error=error,
                    run_id=run_id,
                    topology_version_id=context.topology_version_id,
                    node_id=milestone["node_id"],
                    action_id=action_id,
                )
        return MilestoneCommitResult(result, latest, notification, shortstat)

    @staticmethod
    def _commit_subject(module: str, paths: list[str] | tuple[str, ...], summary: str) -> str:
        value = summary.strip()
        conventional = re.fullmatch(
            r"[a-z]+(?:\([^)]+\))?!?: (?P<description>.+)", value
        )
        description = conventional.group("description") if conventional else value
        normalized = re.sub(r"\s+", " ", description).casefold().strip(".。")
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
            not value
            or len(value) > 120
            or "\r" in value
            or "\n" in value
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
        if conventional:
            return value
        lowered = tuple(path.replace("\\", "/").casefold() for path in paths)
        has_code = any(
            not path.startswith("docs/")
            and not path.startswith("tools/")
            and "/tests/" not in path
            and not path.startswith("tests/")
            for path in lowered
        )
        has_tests = any("/tests/" in path or path.startswith("tests/") for path in lowered)
        has_scripts = any(path.startswith("tools/") for path in lowered)
        has_docs = any(path.startswith("docs/") for path in lowered)
        kind = "feat" if has_code else "test" if has_tests else "chore" if has_scripts else "docs" if has_docs else "chore"
        subject = f"{kind}({module}): {value}"
        if len(subject) > 160:
            raise CoordinatorError(
                "milestone_commit_summary_invalid",
                "Milestone commit subject exceeds 160 characters after contextual scope is added",
            )
        return subject

    def recover_pending_commits(self) -> tuple[str, ...]:
        with self.database.connect() as connection:
            intent_ids = [
                row[0]
                for row in connection.execute(
                    """SELECT intent_id FROM workflow_commit_intents
                       WHERE status IN ('prepared', 'committed') ORDER BY created_at"""
                )
            ]
        recovered: list[str] = []
        for intent_id in intent_ids:
            if self._reconcile_intent(intent_id) is not None:
                recovered.append(intent_id)
        return tuple(recovered)

    def _reconcile_intent(
        self,
        intent_id: str,
        *,
        result: FinalizeResult | None = None,
        shortstat: str | None = None,
        gate_fingerprint: str | None = None,
    ) -> FinalizeResult | None:
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT intent.*, finalize.commit_sha AS finalized_sha,
                          finalize.ref_updated_sha, finalize.message AS finalized_message,
                          finalize.categories_json, finalize.untracked_json
                   FROM workflow_commit_intents intent
                   LEFT JOIN finalize_requests finalize
                     ON finalize.request_id=intent.intent_id
                   WHERE intent.intent_id=?""",
                (intent_id,),
            ).fetchone()
        if row is None:
            raise KeyError(intent_id)
        if result is None and (row["finalized_sha"] or row["ref_updated_sha"]):
            result = self.finalize.reconcile_request(intent_id)
            if result is None:
                return None
        commit_sha = result.commit_sha if result is not None else None
        if not commit_sha:
            return None
        message = result.message if result is not None else row["finalized_message"]
        if not message:
            return None
        if result is None:
            return None
        shortstat = shortstat if shortstat is not None else self._git(
            "show", "--shortstat", "--format=", str(commit_sha)
        ).strip()
        evidence = {
            "commitSha": str(commit_sha),
            "shortstat": shortstat,
            "baselineEpoch": self.baselines.current().epoch_id,
            "actor": row["actor"],
            "actionId": row["action_id"],
            "gateFingerprint": gate_fingerprint or row["gate_fingerprint"],
            "intentId": intent_id,
        }
        content = (str(message) + "\n" + shortstat).encode("utf-8")
        now = utc_text()
        with self.database.transaction() as connection:
            current = connection.execute(
                "SELECT status FROM workflow_commit_intents WHERE intent_id=?",
                (intent_id,),
            ).fetchone()
            if current["status"] == "reconciled":
                return result
            attempt_number = int(
                connection.execute(
                    "SELECT COALESCE(MAX(attempt_number), 0) + 1 FROM workflow_attempts WHERE node_id=?",
                    (row["node_id"],),
                ).fetchone()[0]
            )
            attempt_id = uuid.uuid4().hex
            connection.execute(
                """INSERT INTO workflow_attempts(
                       attempt_id, run_id, node_id, attempt_number, state,
                       accepted, evidence_json, started_at, completed_at
                   ) VALUES (?, ?, ?, ?, 'succeeded', 1, ?, ?, ?)""",
                (
                    attempt_id,
                    row["run_id"],
                    row["node_id"],
                    attempt_number,
                    json.dumps(evidence, sort_keys=True),
                    now,
                    now,
                ),
            )
            connection.execute(
                """UPDATE workflow_nodes SET state='succeeded', attempt_count=?, updated_at=?
                   WHERE node_id=?""",
                (attempt_number, now, row["node_id"]),
            )
            connection.execute(
                """INSERT INTO workflow_artifacts(
                       artifact_id, run_id, node_id, attempt_id, artifact_kind,
                       display_name, content_hash, byte_count, metadata_json, created_at
                   ) VALUES (?, ?, ?, ?, 'commit', ?, ?, ?, ?, ?)""",
                (
                    uuid.uuid4().hex,
                    row["run_id"],
                    row["node_id"],
                    attempt_id,
                    str(commit_sha),
                    hashlib.sha256(content).hexdigest(),
                    len(content),
                    json.dumps({"commitSha": str(commit_sha), "intentId": intent_id}),
                    now,
                ),
            )
            connection.execute(
                """UPDATE workflow_commit_intents
                   SET status='reconciled', commit_sha=?, error_text=NULL, updated_at=?
                   WHERE intent_id=?""",
                (str(commit_sha), now, intent_id),
            )
            connection.execute(
                "UPDATE workflow_runs SET updated_at=? WHERE run_id=?",
                (now, row["run_id"]),
            )
        return result

    def reconcile_accepted_milestones(
        self,
        *,
        source_run_id: str,
        target_run_id: str,
        milestone_keys: tuple[str, ...],
        actor: str,
        action_id: str | None,
    ) -> dict[str, object]:
        """Import accepted immutable milestone evidence between equal plan topologies.

        This is deliberately evidence-only: it never stages files or recreates a
        historical commit.  The source manifest is reconstructed from the accepted
        commit before its record is copied into the target run.
        """
        normalized_keys = tuple(dict.fromkeys(key.strip().upper() for key in milestone_keys))
        if (
            not normalized_keys
            or any(not re.fullmatch(r"M[1-9]\d*", key) for key in normalized_keys)
            or source_run_id == target_run_id
        ):
            raise CoordinatorError(
                "workflow_reconcile_parameters_invalid",
                "Reconciliation requires distinct runs and one or more milestone keys",
            )
        audit_id = action_id or uuid.uuid4().hex
        with self.database.connect() as connection:
            runs = {
                row["run_id"]: row
                for row in connection.execute(
                    """SELECT run.run_id, run.session_id, run.plan_path, run.topology_hash,
                              run.state, run.updated_at, run.current_topology_version_id,
                              version.topology_json
                       FROM workflow_runs run
                       JOIN workflow_topology_versions version
                         ON version.topology_version_id=run.current_topology_version_id
                       WHERE run.run_id IN (?, ?)""",
                    (source_run_id, target_run_id),
                )
            }
            if len(runs) != 2:
                raise CoordinatorError(
                    "workflow_reconcile_run_not_found",
                    "Both source and target workflow runs must have an active topology",
                )
            source_run = runs[source_run_id]
            target_run = runs[target_run_id]
            if (
                str(source_run["plan_path"]).replace("\\", "/")
                != str(target_run["plan_path"]).replace("\\", "/")
                or not source_run["topology_hash"]
                or source_run["topology_hash"] != target_run["topology_hash"]
            ):
                raise CoordinatorError(
                    "workflow_reconcile_topology_mismatch",
                    "Accepted evidence can only move between identical plan topology hashes",
                )
            if str(target_run["state"]) not in {"active", "stale"}:
                raise CoordinatorError(
                    "workflow_reconcile_target_terminal",
                    "Accepted evidence can only be imported into an active or stale workflow run",
                )
            source_topology = json.loads(str(source_run["topology_json"]))
            target_topology = json.loads(str(target_run["topology_json"]))
            source_topology.pop("content_hash", None)
            target_topology.pop("content_hash", None)
            if source_topology != target_topology:
                raise CoordinatorError(
                    "workflow_reconcile_topology_content_mismatch",
                    "Topology payloads differ despite matching topology hashes",
                )
            source_records = [
                self._reconciliation_source_record(
                    connection,
                    source_run,
                    target_run,
                    milestone_key,
                )
                for milestone_key in normalized_keys
            ]
            accepted_target_milestones = {
                str(row["node_key"])
                for row in connection.execute(
                    """SELECT node.node_key
                       FROM workflow_nodes AS node
                       WHERE node.run_id=?
                         AND EXISTS (
                             SELECT 1 FROM workflow_attempts AS attempt
                             WHERE attempt.node_id=node.node_id AND attempt.accepted=1
                         )""",
                    (target_run_id,),
                )
            }
            unresolved_dependencies = self._reconciliation_unaccepted_dependencies(
                source_records, accepted_target_milestones
            )
            if unresolved_dependencies:
                raise CoordinatorError(
                    "workflow_reconcile_dependency_unaccepted",
                    "Accepted evidence cannot skip an unaccepted milestone dependency",
                    details={"dependencies": unresolved_dependencies},
                )

        open_failure_paths: tuple[str, ...] = ()
        plan_path = str(source_run["plan_path"])
        if self.failures is not None:
            self.failures.import_repository()
            open_failures = self.failures.open_related_to_plan(plan_path)
            open_failure_paths = tuple(
                sorted(
                    {str(item.artifact_path) for item in open_failures},
                    key=str.casefold,
                )
            )

        head = self._git("rev-parse", "HEAD")
        for record in source_records:
            commit_sha = str(record["commit_sha"])
            if self._git("rev-parse", f"{commit_sha}^{{commit}}") != commit_sha:
                raise CoordinatorError(
                    "workflow_reconcile_commit_invalid",
                    "Accepted evidence references a non-canonical commit SHA",
                    details={"milestone": record["milestone_key"], "commitSha": commit_sha},
                )
            ancestor = subprocess.run(
                ["git", "merge-base", "--is-ancestor", commit_sha, head],
                cwd=self.repo_root,
                capture_output=True,
            )
            if ancestor.returncode != 0:
                raise CoordinatorError(
                    "workflow_reconcile_commit_not_ancestor",
                    "Accepted evidence commit is not an ancestor of the current HEAD",
                    details={"milestone": record["milestone_key"], "commitSha": commit_sha},
                )
            actual_manifest_hash = self._manifest_hash_from_commit(
                commit_sha, tuple(record["paths"])
            )
            if actual_manifest_hash != record["manifest_hash"]:
                raise CoordinatorError(
                    "workflow_reconcile_manifest_content_mismatch",
                    "Accepted manifest does not match the historical commit content",
                    details={"milestone": record["milestone_key"]},
                )

        now = utc_text()
        copied: list[dict[str, object]] = []
        with self.database.transaction() as connection:
            for record in source_records:
                target_node = connection.execute(
                    """SELECT * FROM workflow_nodes
                       WHERE run_id=? AND node_key=? AND kind='milestone'""",
                    (target_run_id, record["milestone_key"]),
                ).fetchone()
                if target_node is None:
                    raise CoordinatorError(
                        "workflow_reconcile_target_milestone_missing",
                        "Target topology no longer contains the requested milestone",
                        details={"milestone": record["milestone_key"]},
                    )
                if (
                    str(target_node["title"]) != str(record["source_title"])
                    or str(target_node["stage"]) != str(record["source_stage"])
                ):
                    raise CoordinatorError(
                        "workflow_reconcile_milestone_identity_mismatch",
                        "Source and target milestone identities differ",
                        details={"milestone": record["milestone_key"]},
                    )
                accepted = connection.execute(
                    """SELECT attempt_id FROM workflow_attempts
                       WHERE node_id=? AND accepted=1 ORDER BY attempt_number DESC LIMIT 1""",
                    (target_node["node_id"],),
                ).fetchone()
                if accepted is not None:
                    raise CoordinatorError(
                        "workflow_reconcile_target_already_accepted",
                        "Target milestone already has accepted evidence",
                        details={"milestone": record["milestone_key"]},
                    )
                existing_manifest = connection.execute(
                    """SELECT manifest_hash, paths_json FROM workflow_milestone_manifests
                       WHERE run_id=? AND topology_version_id=? AND node_id=?""",
                    (
                        target_run_id,
                        target_run["current_topology_version_id"],
                        target_node["node_id"],
                    ),
                ).fetchone()
                if existing_manifest is not None and (
                    str(existing_manifest["manifest_hash"]) != str(record["manifest_hash"])
                    or str(existing_manifest["paths_json"]) != str(record["paths_json"])
                ):
                    raise CoordinatorError(
                        "workflow_reconcile_target_manifest_mismatch",
                        "Target already bound a different immutable manifest",
                        details={"milestone": record["milestone_key"]},
                    )
                if existing_manifest is None:
                    connection.execute(
                        """INSERT INTO workflow_milestone_manifests(
                               manifest_id, run_id, topology_version_id, node_id, session_id,
                               paths_json, manifest_hash, actor, action_id, created_at
                           ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                        (
                            uuid.uuid4().hex,
                            target_run_id,
                            target_run["current_topology_version_id"],
                            target_node["node_id"],
                            target_run["session_id"],
                            record["paths_json"],
                            record["manifest_hash"],
                            actor,
                            audit_id,
                            now,
                        ),
                    )
                target_intent_id = uuid.uuid4().hex
                connection.execute(
                    """INSERT INTO workflow_commit_intents(
                           intent_id, run_id, topology_version_id, node_id, session_id,
                           action_id, actor, gate_fingerprint, paths_json, message, status,
                           commit_sha, created_at, updated_at
                       ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'reconciled', ?, ?, ?)""",
                    (
                        target_intent_id,
                        target_run_id,
                        target_run["current_topology_version_id"],
                        target_node["node_id"],
                        target_run["session_id"],
                        audit_id,
                        actor,
                        record["gate_fingerprint"],
                        record["paths_json"],
                        record["message"],
                        record["commit_sha"],
                        now,
                        now,
                    ),
                )
                attempt_number = int(
                    connection.execute(
                        "SELECT COALESCE(MAX(attempt_number), 0) + 1 FROM workflow_attempts WHERE node_id=?",
                        (target_node["node_id"],),
                    ).fetchone()[0]
                )
                target_attempt_id = uuid.uuid4().hex
                evidence = dict(record["evidence"])
                evidence.update(
                    {
                        # Future reconciliations resolve the intent inside this target run.
                        "intentId": target_intent_id,
                        "reconciledFromRunId": source_run_id,
                        "sourceAttemptId": record["source_attempt_id"],
                        "sourceIntentId": record["source_intent_id"],
                        "sourceManifestHash": record["manifest_hash"],
                        "reconciliationActionId": audit_id,
                        "openFailurePathsAtReconciliation": list(open_failure_paths),
                    }
                )
                if record["legacy_evidence_intent_id"] is not None:
                    evidence["legacyEvidenceIntentId"] = record["legacy_evidence_intent_id"]
                connection.execute(
                    """INSERT INTO workflow_attempts(
                           attempt_id, run_id, node_id, attempt_number, state, accepted,
                           evidence_json, started_at, completed_at
                       ) VALUES (?, ?, ?, ?, 'succeeded', 1, ?, ?, ?)""",
                    (
                        target_attempt_id,
                        target_run_id,
                        target_node["node_id"],
                        attempt_number,
                        json.dumps(evidence, sort_keys=True),
                        now,
                        now,
                    ),
                )
                connection.execute(
                    """UPDATE workflow_nodes SET state='succeeded', attempt_count=?, updated_at=?
                       WHERE node_id=?""",
                    (attempt_number, now, target_node["node_id"]),
                )
                shortstat = str(record["evidence"].get("shortstat") or self._git(
                    "show", "--shortstat", "--format=", str(record["commit_sha"])
                ).strip())
                artifact_content = (str(record["message"]) + "\n" + shortstat).encode("utf-8")
                connection.execute(
                    """INSERT INTO workflow_artifacts(
                           artifact_id, run_id, node_id, attempt_id, artifact_kind,
                           display_name, content_hash, byte_count, metadata_json, created_at
                       ) VALUES (?, ?, ?, ?, 'commit', ?, ?, ?, ?, ?)""",
                    (
                        uuid.uuid4().hex,
                        target_run_id,
                        target_node["node_id"],
                        target_attempt_id,
                        record["commit_sha"],
                        hashlib.sha256(artifact_content).hexdigest(),
                        len(artifact_content),
                        json.dumps(
                            {
                                "commitSha": record["commit_sha"],
                                "intentId": target_intent_id,
                                "reconciledFromRunId": source_run_id,
                                "sourceIntentId": record["source_intent_id"],
                            },
                            sort_keys=True,
                        ),
                        now,
                    ),
                )
                copied.append(
                    {
                        "milestoneId": record["milestone_key"],
                        "commitSha": record["commit_sha"],
                        "manifestHash": record["manifest_hash"],
                        "attemptId": target_attempt_id,
                        "intentId": target_intent_id,
                        "state": "succeeded",
                    }
                )
            connection.execute(
                "UPDATE workflow_runs SET updated_at=? WHERE run_id=?",
                (now, target_run_id),
            )
            connection.execute(
                """INSERT INTO events(session_id, event_type, payload_json, created_at)
                   VALUES (?, 'workflow.milestones_reconciled', ?, ?)""",
                (
                    target_run["session_id"],
                    json.dumps(
                        {
                            "actionId": audit_id,
                            "sourceRunId": source_run_id,
                            "targetRunId": target_run_id,
                            "milestones": [item["milestoneId"] for item in copied],
                            "openFailurePaths": list(open_failure_paths),
                        },
                        sort_keys=True,
                    ),
                    now,
                ),
            )
        return {
            "auditId": audit_id,
            "sourceRunId": source_run_id,
            "targetRunId": target_run_id,
            "nodes": copied,
            "openFailurePaths": list(open_failure_paths),
        }

    def _reconciliation_source_record(self, connection, source_run, target_run, milestone_key: str) -> dict[str, object]:
        source_node = connection.execute(
            """SELECT * FROM workflow_nodes
               WHERE run_id=? AND node_key=? AND kind='milestone'""",
            (source_run["run_id"], milestone_key),
        ).fetchone()
        target_node = connection.execute(
            """SELECT node_id FROM workflow_nodes
               WHERE run_id=? AND node_key=? AND kind='milestone'""",
            (target_run["run_id"], milestone_key),
        ).fetchone()
        if source_node is None or target_node is None:
            raise CoordinatorError(
                "workflow_reconcile_milestone_missing",
                "Both equal topologies must contain every requested milestone",
                details={"milestone": milestone_key},
            )
        attempt = connection.execute(
            """SELECT * FROM workflow_attempts
               WHERE run_id=? AND node_id=? AND state='succeeded' AND accepted=1
               ORDER BY attempt_number DESC LIMIT 1""",
            (source_run["run_id"], source_node["node_id"]),
        ).fetchone()
        if attempt is None:
            raise CoordinatorError(
                "workflow_reconcile_source_evidence_missing",
                "Source milestone requires accepted attempt and immutable manifest",
                details={"milestone": milestone_key},
            )
        try:
            evidence = json.loads(str(attempt["evidence_json"]))
        except json.JSONDecodeError as error:
            raise CoordinatorError(
                "workflow_reconcile_source_evidence_invalid",
                "Source accepted attempt evidence is not valid JSON",
                details={"milestone": milestone_key},
            ) from error
        if not isinstance(evidence, dict):
            raise CoordinatorError(
                "workflow_reconcile_source_evidence_invalid",
                "Source accepted attempt evidence must be an object",
                details={"milestone": milestone_key},
            )
        evidence_intent_id = str(evidence.get("intentId") or "")
        commit_sha = str(evidence.get("commitSha") or "")
        intent = connection.execute(
            """SELECT * FROM workflow_commit_intents
               WHERE intent_id=? AND run_id=? AND node_id=? AND status='reconciled'""",
            (evidence_intent_id, source_run["run_id"], source_node["node_id"]),
        ).fetchone()
        intent_id = evidence_intent_id
        legacy_evidence_intent_id: str | None = None
        is_legacy_reconciliation = bool(
            evidence.get("reconciledFromRunId")
            and evidence_intent_id
            and evidence.get("sourceIntentId") == evidence_intent_id
        )
        if intent is None and is_legacy_reconciliation and commit_sha:
            candidates = connection.execute(
                """SELECT * FROM workflow_commit_intents
                   WHERE run_id=? AND node_id=? AND status='reconciled' AND commit_sha=?
                   ORDER BY created_at, intent_id""",
                (source_run["run_id"], source_node["node_id"], commit_sha),
            ).fetchall()
            if len(candidates) > 1:
                raise CoordinatorError(
                    "workflow_reconcile_source_commit_ambiguous",
                    "Legacy accepted evidence matches multiple reconciled local commit intents",
                    details={
                        "milestone": milestone_key,
                        "candidateIntentIds": [str(item["intent_id"]) for item in candidates],
                    },
                )
            if len(candidates) == 1:
                intent = candidates[0]
                intent_id = str(intent["intent_id"])
                legacy_evidence_intent_id = evidence_intent_id
        if intent is None or not commit_sha or str(intent["commit_sha"] or "") != commit_sha:
            raise CoordinatorError(
                "workflow_reconcile_source_commit_missing",
                "Source accepted attempt is not backed by a reconciled commit intent",
                details={"milestone": milestone_key},
            )
        manifest = connection.execute(
            """SELECT * FROM workflow_milestone_manifests
               WHERE run_id=? AND topology_version_id=? AND node_id=?
               ORDER BY created_at DESC, manifest_id DESC LIMIT 1""",
            (
                source_run["run_id"],
                intent["topology_version_id"],
                source_node["node_id"],
            ),
        ).fetchone()
        source_evidence_topology = connection.execute(
            """SELECT topology_json FROM workflow_topology_versions
               WHERE topology_version_id=? AND run_id=?""",
            (intent["topology_version_id"], source_run["run_id"]),
        ).fetchone()
        if manifest is None or source_evidence_topology is None:
            raise CoordinatorError(
                "workflow_reconcile_source_evidence_missing",
                "Source milestone requires accepted attempt and immutable manifest",
                details={"milestone": milestone_key},
            )
        source_title, source_dependencies = self._reconciliation_milestone_identity(
            source_evidence_topology["topology_json"], milestone_key
        )
        target_title, target_dependencies = self._reconciliation_milestone_identity(
            target_run["topology_json"], milestone_key
        )
        if source_title != target_title or any(
            dependency not in target_dependencies for dependency in source_dependencies
        ):
            raise CoordinatorError(
                "workflow_reconcile_historical_milestone_identity_mismatch",
                "Accepted milestone identity changed after its historical topology version",
                details={"milestone": milestone_key},
            )
        paths = tuple(json.loads(str(manifest["paths_json"])))
        normalized_paths = tuple(sorted(set(paths), key=str.casefold))
        if not normalized_paths or tuple(paths) != normalized_paths or str(intent["paths_json"]) != str(manifest["paths_json"]):
            raise CoordinatorError(
                "workflow_reconcile_manifest_invalid",
                "Source manifest paths are not canonical or do not match its commit intent",
                details={"milestone": milestone_key},
            )
        return {
            "milestone_key": milestone_key,
            "source_title": source_title,
            "source_stage": source_node["stage"],
            "dependencies": source_dependencies,
            "source_attempt_id": attempt["attempt_id"],
            "source_intent_id": intent_id,
            "legacy_evidence_intent_id": legacy_evidence_intent_id,
            "commit_sha": commit_sha,
            "manifest_hash": manifest["manifest_hash"],
            "paths_json": manifest["paths_json"],
            "paths": normalized_paths,
            "gate_fingerprint": intent["gate_fingerprint"],
            "message": intent["message"],
            "evidence": evidence,
        }

    @staticmethod
    def _reconciliation_milestone_identity(
        topology_json: object, milestone_key: str
    ) -> tuple[str, tuple[str, ...]]:
        """Return the immutable identity of one milestone from a topology version."""
        try:
            topology = json.loads(str(topology_json))
            milestones = topology["milestones"]
        except (json.JSONDecodeError, KeyError, TypeError) as error:
            raise CoordinatorError(
                "workflow_reconcile_historical_topology_invalid",
                "Historical topology evidence is not valid",
                details={"milestone": milestone_key},
            ) from error
        if not isinstance(milestones, list):
            raise CoordinatorError(
                "workflow_reconcile_historical_topology_invalid",
                "Historical topology milestones are not valid",
                details={"milestone": milestone_key},
            )
        milestone = next(
            (
                item
                for item in milestones
                if isinstance(item, dict) and item.get("node_id") == milestone_key
            ),
            None,
        )
        if not isinstance(milestone, dict):
            raise CoordinatorError(
                "workflow_reconcile_historical_milestone_missing",
                "Historical topology does not contain the requested milestone",
                details={"milestone": milestone_key},
            )
        title = milestone.get("title")
        dependencies = milestone.get("depends_on")
        if (
            not isinstance(title, str)
            or not isinstance(dependencies, list)
            or any(not isinstance(item, str) for item in dependencies)
        ):
            raise CoordinatorError(
                "workflow_reconcile_historical_topology_invalid",
                "Historical milestone identity is not valid",
                details={"milestone": milestone_key},
            )
        return title, tuple(dependencies)

    @staticmethod
    def _reconciliation_unaccepted_dependencies(
        source_records: list[dict[str, object]], accepted_target_milestones: set[str]
    ) -> dict[str, list[str]]:
        """Return dependencies that the requested import would otherwise skip."""
        requested = {str(record["milestone_key"]) for record in source_records}
        unresolved: dict[str, list[str]] = {}
        for record in source_records:
            missing = sorted(
                (
                    dependency
                    for dependency in tuple(record["dependencies"])
                    if dependency not in requested
                    and dependency not in accepted_target_milestones
                ),
                key=str.casefold,
            )
            if missing:
                unresolved[str(record["milestone_key"])] = missing
        return unresolved

    def _manifest_hash_from_commit(self, commit_sha: str, paths: tuple[str, ...]) -> str:
        manifest: list[dict[str, object]] = []
        for path in paths:
            object_name = f"{commit_sha}:{path}"
            result = subprocess.run(
                ["git", "cat-file", "blob", object_name],
                cwd=self.repo_root,
                capture_output=True,
            )
            if result.returncode == 0:
                manifest.append(
                    {"path": path, "kind": "file", "blob": hashlib.sha256(result.stdout).hexdigest()}
                )
                continue
            parent = subprocess.run(
                ["git", "cat-file", "-e", f"{commit_sha}^:{path}"],
                cwd=self.repo_root,
                capture_output=True,
            )
            if parent.returncode != 0:
                raise CoordinatorError(
                    "workflow_reconcile_manifest_path_missing",
                    "Manifest path is absent from both the accepted commit and its parent",
                    details={"commitSha": commit_sha, "path": path},
                )
            manifest.append({"path": path, "kind": "deletion", "blob": None})
        return _hash_json(manifest)

    def close_goal(self, session_id: str, run_id: str) -> dict[str, object]:
        if self.sessions is None or self.leases is None:
            raise CoordinatorError("action_unavailable", "Goal closeout services are unavailable")
        self._require_run_owner(run_id, session_id)
        with self.database.connect() as connection:
            run = connection.execute(
                """SELECT run.current_topology_version_id, version.topology_hash,
                          version.plan_path
                   FROM workflow_runs run
                   LEFT JOIN workflow_topology_versions version
                     ON version.topology_version_id=run.current_topology_version_id
                   WHERE run.run_id=?""",
                (run_id,),
            ).fetchone()
            if run is None or run["current_topology_version_id"] is None:
                raise CoordinatorError(
                    "workflow_topology_not_active",
                    "Goal closeout requires an active topology version",
                )
            current_plan = TopologyParser(self.repo_root).parse(run["plan_path"])
            if current_plan.topology_hash != run["topology_hash"]:
                raise CoordinatorError(
                    "workflow_topology_plan_changed",
                    "Plan topology changed after the active topology version was imported",
                    details={
                        "activeTopologyHash": run["topology_hash"],
                        "currentTopologyHash": current_plan.topology_hash,
                    },
                )
            milestone_count = int(
                connection.execute(
                    "SELECT COUNT(*) FROM workflow_nodes WHERE run_id=? AND kind='milestone'",
                    (run_id,),
                ).fetchone()[0]
            )
            if milestone_count == 0:
                raise CoordinatorError(
                    "workflow_goal_empty", "Goal closeout requires at least one milestone"
                )
            incomplete = [
                row["node_key"]
                for row in connection.execute(
                    """SELECT node.node_key,
                              COALESCE((
                                  SELECT attempt.state FROM workflow_attempts attempt
                                  WHERE attempt.node_id=node.node_id AND attempt.accepted=1
                                  ORDER BY attempt.attempt_number DESC LIMIT 1
                              ), node.state) AS current_state
                       FROM workflow_nodes node
                       WHERE node.run_id=? AND node.kind='milestone'
                         AND COALESCE((
                             SELECT attempt.state FROM workflow_attempts attempt
                             WHERE attempt.node_id=node.node_id AND attempt.accepted=1
                             ORDER BY attempt.attempt_number DESC LIMIT 1
                         ), node.state) <> 'succeeded'""",
                    (run_id,),
                )
            ]
            pending_patches = int(
                connection.execute(
                    """SELECT COUNT(*) FROM patches
                       WHERE session_id=? AND status IN ('queued', 'applying', 'needs_rebase')""",
                    (session_id,),
                ).fetchone()[0]
            )
            if pending_patches:
                raise CoordinatorError(
                    "workflow_goal_pending_patches",
                    "Session still has pending delayed patches",
                    details={"count": pending_patches},
                )
            unreconciled = int(
                connection.execute(
                    """SELECT COUNT(*) FROM workflow_commit_intents
                       WHERE run_id=? AND status IN ('prepared', 'committed')""",
                    (run_id,),
                ).fetchone()[0]
            )
            if unreconciled:
                raise CoordinatorError(
                    "workflow_goal_commit_reconciliation_pending",
                    "Milestone commit reconciliation is incomplete",
                    details={"count": unreconciled},
                )
        if incomplete:
            raise CoordinatorError(
                "workflow_goal_incomplete",
                "All milestones must succeed before Goal closeout",
                details={"milestones": incomplete},
            )
        dirty = self.owned_scope_dirty(session_id)
        if dirty:
            raise CoordinatorError(
                "workflow_goal_owned_scope_dirty",
                "Session-owned changes remain after the last milestone commit",
                details={"paths": list(dirty)},
            )
        session = self.sessions.get(session_id)
        if self.failures is not None and session.plan_path:
            self.failures.import_repository()
            open_failures = self.failures.open_related_to_plan(session.plan_path)
            if open_failures:
                raise CoordinatorError(
                    "workflow_goal_open_failure",
                    "Applicable Failure handoffs remain open",
                    details={"paths": [item.artifact_path for item in open_failures]},
                )
        head = self._git("rev-parse", "HEAD")
        now = utc_text()
        with self.database.transaction() as connection:
            session_row = connection.execute(
                "SELECT * FROM sessions WHERE session_id=?", (session_id,)
            ).fetchone()
            if session_row is None or session_row["status"] not in {
                SessionStatus.ACTIVE.value,
                SessionStatus.WAITING_VALIDATION.value,
            }:
                raise CoordinatorError(
                    "workflow_goal_session_changed",
                    "Session state changed before Goal closeout",
                )
            goal = connection.execute(
                "SELECT * FROM workflow_nodes WHERE run_id=? AND node_key='goal'",
                (run_id,),
            ).fetchone()
            attempt_number = int(
                connection.execute(
                    "SELECT COALESCE(MAX(attempt_number), 0) + 1 FROM workflow_attempts WHERE node_id=?",
                    (goal["node_id"],),
                ).fetchone()[0]
            )
            connection.execute(
                """INSERT INTO workflow_attempts(
                       attempt_id, run_id, node_id, attempt_number, state,
                       accepted, evidence_json, started_at, completed_at
                   ) VALUES (?, ?, ?, ?, 'succeeded', 1, ?, ?, ?)""",
                (
                    uuid.uuid4().hex,
                    run_id,
                    goal["node_id"],
                    attempt_number,
                    json.dumps({"reason": "all milestones accepted", "head": head}),
                    now,
                    now,
                ),
            )
            connection.execute(
                """UPDATE workflow_nodes
                   SET state='succeeded', attempt_count=?, updated_at=?
                   WHERE node_id=?""",
                (attempt_number, now, goal["node_id"]),
            )
            released = connection.execute(
                "DELETE FROM leases WHERE session_id=?", (session_id,)
            ).rowcount
            reason = f"Goal finalized at {head}"
            connection.execute(
                """UPDATE sessions
                   SET status='completed', status_reason=?, updated_at=?,
                       last_heartbeat_at=?, completed_at=?
                   WHERE session_id=?""",
                (reason, now, now, now, session_id),
            )
            connection.execute(
                """UPDATE workflow_runs
                   SET state='succeeded', status_reason=?, updated_at=?, completed_at=?
                   WHERE run_id=?""",
                (reason, now, now, run_id),
            )
            connection.execute(
                """INSERT INTO events(session_id, event_type, payload_json, created_at)
                   VALUES (?, 'session.status_changed', ?, ?)""",
                (
                    session_id,
                    json.dumps(
                        {
                            "from": session_row["status"],
                            "to": SessionStatus.COMPLETED.value,
                            "reason": reason,
                        }
                    ),
                    now,
                ),
            )
        completed = self.sessions.get(session_id)
        return {
            "session": completed.to_dict(),
            "releasedLeases": released,
            "head": head,
        }

    @staticmethod
    def _require_allowed(decision: GateDecision) -> None:
        if not decision.allowed:
            raise CoordinatorError(
                decision.code,
                "Milestone gate is not accepted",
                details={
                    "blockingNodeIds": list(decision.blocking_node_ids),
                    "applicableFailureIds": list(decision.applicable_failure_ids),
                    "requiredEvidence": list(decision.required_evidence),
                },
            )

    def _failure_node_keys(self, run_id: str, milestone_key: str) -> tuple[str, ...]:
        """Return one slice key or a parent milestone plus its direct slice keys."""
        milestone = self._milestone_node(run_id, milestone_key)
        keys = {milestone_key}
        if milestone["kind"] == "milestone":
            with self.database.connect() as connection:
                keys.update(
                    row[0]
                    for row in connection.execute(
                        """SELECT source.node_key
                           FROM workflow_edges edge
                           JOIN workflow_nodes source
                             ON source.node_id=edge.from_node_id
                           WHERE edge.run_id=? AND edge.to_node_id=?
                             AND source.kind='slice'""",
                        (run_id, milestone["node_id"]),
                    )
                )
        return tuple(sorted(keys, key=str.casefold))

    def _milestone_node(self, run_id: str, milestone_key: str):
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT * FROM workflow_nodes
                   WHERE run_id=? AND node_key=? AND kind IN ('milestone', 'slice')""",
                (run_id, milestone_key),
            ).fetchone()
        if row is None:
            raise CoordinatorError(
                "workflow_milestone_not_found",
                f"Unknown milestone or slice {milestone_key}",
            )
        return row

    def _record_manifest(
        self,
        *,
        session_id: str,
        run_id: str,
        milestone,
        paths: tuple[str, ...] | list[str],
        context: GateContext,
        actor: str,
        action_id: str | None,
    ) -> str:
        normalized = tuple(sorted(set(paths), key=str.casefold))
        if not normalized or self.leases is None:
            raise CoordinatorError(
                "milestone_manifest_unleased",
                "Every milestone manifest path must be covered by the executing Session leases",
                details={"paths": list(normalized)},
            )
        self.leases.require_owned_live(
            session_id,
            normalized,
            error_code="milestone_manifest_unleased",
            message="Every milestone manifest path must be covered by the executing Session leases",
        )
        with self.database.connect() as connection:
            existing = connection.execute(
                """SELECT * FROM workflow_milestone_manifests
                   WHERE run_id=? AND topology_version_id=? AND node_id=?""",
                (run_id, context.topology_version_id, milestone["node_id"]),
            ).fetchone()
        if existing is not None:
            existing_paths = tuple(json.loads(existing["paths_json"]))
            if existing_paths != normalized or existing["manifest_hash"] != context.manifest_hash:
                raise CoordinatorError(
                    "milestone_manifest_already_bound",
                    "Milestone manifest is immutable after its first binding",
                    details={"paths": list(existing_paths)},
                )
            return str(existing["manifest_id"])
        manifest_id = uuid.uuid4().hex
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO workflow_milestone_manifests(
                       manifest_id, run_id, topology_version_id, node_id,
                       session_id, paths_json, manifest_hash, actor, action_id, created_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    manifest_id,
                    run_id,
                    context.topology_version_id,
                    milestone["node_id"],
                    session_id,
                    json.dumps(normalized),
                    context.manifest_hash,
                    actor,
                    action_id,
                    utc_text(),
                ),
            )
        return manifest_id

    def _derive_milestone_paths(
        self, session_id: str, run_id: str, milestone_key: str
    ) -> tuple[str, ...]:
        session = self.sessions.get(session_id) if self.sessions is not None else None
        if session is None or not session.plan_path:
            raise CoordinatorError("session_plan_missing", "Session has no numbered plan")
        owner = TopologyParser(self.repo_root).plans.resolve_owner(session.plan_path)
        child = self.repo_root / owner.child_dir
        matches: list[tuple[str, tuple[str, ...]]] = []
        dirty_attributed = set(self.attributed_changes(session_id))
        for record in sorted(child.glob("*.md"), key=lambda item: item.name.casefold()):
            if record.name.startswith(("failure-", "fixed-")):
                continue
            fields, files = self._plan_output_fields(record)
            if fields.get("Plan") == session.plan_path and fields.get("Milestone") == milestone_key:
                relative = record.relative_to(self.repo_root).as_posix()
                matches.append((relative, files))
        # A milestone can retain immutable historical evidence beside a fresh current-source
        # attestation. Only a record changed and attributed by this executor may bind this run.
        current_matches = [item for item in matches if item[0] in dirty_attributed]
        selected = current_matches if len(matches) > 1 else matches
        if len(selected) != 1:
            raise CoordinatorError(
                "milestone_manifest_record_ambiguous",
                "Exactly one current attributed child-plan record must declare this milestone manifest",
                details={
                    "records": [path for path, _ in matches],
                    "attributedRecords": [path for path, _ in current_matches],
                },
            )
        record_path, declared = selected[0]
        normalized: set[str] = {record_path}
        for raw in declared:
            candidate = (self.repo_root / raw).resolve()
            if not candidate.is_relative_to(self.repo_root) or candidate == self.repo_root:
                raise CoordinatorError(
                    "milestone_manifest_path_invalid", "Declared milestone path escaped repository"
                )
            normalized.add(candidate.relative_to(self.repo_root).as_posix())
        if not normalized <= dirty_attributed:
            raise CoordinatorError(
                "milestone_manifest_not_attributed",
                "Declared milestone files must be current attributed changes",
                details={"paths": sorted(normalized - dirty_attributed)},
            )
        return tuple(sorted(normalized, key=str.casefold))

    @staticmethod
    def _valid_plan_output(path: Path, plan_path: str, milestone_key: str) -> bool:
        text = path.read_text(encoding="utf-8", errors="replace")
        fields, files = MilestoneWorkflowService._plan_output_fields(path)
        headings = {line.strip().casefold() for line in text.splitlines() if line.startswith("## ")}
        return (
            fields.get("Plan") == plan_path
            and fields.get("Milestone") == milestone_key
            and fields.get("Status", "").casefold() in {"completed", "accepted"}
            and bool(files)
            and {
                "## scope delivered",
                "## fresh testing evidence",
                "## review",
            } <= headings
        )

    @staticmethod
    def _plan_output_fields(path: Path) -> tuple[dict[str, str], tuple[str, ...]]:
        text = path.read_text(encoding="utf-8", errors="replace")
        values: dict[str, list[str]] = {}
        for line in text.splitlines():
            match = re.fullmatch(r"(Plan|Milestone|Status|Files):\s*(.*?)\s*", line)
            if match:
                values.setdefault(match.group(1), []).append(match.group(2))
        if any(len(items) != 1 for items in values.values()):
            return {}, ()
        fields = {key: items[0] for key, items in values.items()}
        try:
            files_value = json.loads(fields.get("Files", "[]"))
        except json.JSONDecodeError:
            return fields, ()
        if not isinstance(files_value, list) or any(
            not isinstance(item, str) or not item.strip() for item in files_value
        ):
            return fields, ()
        return fields, tuple(dict.fromkeys(item.replace("\\", "/") for item in files_value))

    def _require_run_owner(self, run_id: str, session_id: str) -> None:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT session_id FROM workflow_runs WHERE run_id=?", (run_id,)
            ).fetchone()
        if row is None or row["session_id"] != session_id:
            raise CoordinatorError(
                "workflow_run_session_mismatch",
                "Workflow run does not belong to the requested Session",
            )

    def _manifest_hash(self, paths: tuple[str, ...]) -> str:
        return self._manifest_hash_at(self.repo_root, paths)

    @staticmethod
    def _manifest_hash_at(root: Path, paths: tuple[str, ...]) -> str:
        manifest: list[dict[str, object]] = []
        for path in paths:
            absolute = root / path
            if absolute.is_file():
                with absolute.open("rb") as source:
                    digest = hashlib.file_digest(source, "sha256").hexdigest()
                manifest.append({"path": path, "kind": "file", "blob": digest})
            elif absolute.is_dir():
                raise CoordinatorError(
                    "milestone_manifest_directory",
                    "Milestone manifest paths must name files, not directories",
                    details={"path": path},
                )
            else:
                manifest.append({"path": path, "kind": "deletion", "blob": None})
        return _hash_json(manifest)

    def _git(self, *arguments: str) -> str:
        return subprocess.run(
            ["git", *arguments],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()

    def _git_paths(self, *arguments: str) -> tuple[str, ...]:
        raw = subprocess.run(
            ["git", *arguments],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
        ).stdout
        return tuple(
            item.decode("utf-8", errors="surrogateescape").replace("\\", "/")
            for item in raw.split(b"\0")
            if item
        )


def _hash_json(value: object) -> str:
    return hashlib.sha256(
        json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()
