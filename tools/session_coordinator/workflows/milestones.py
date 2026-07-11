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
        actor: str,
        action_id: str | None,
    ) -> None:
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
        context = self.prepare_context(run_id, paths)
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
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO workflow_validation_bindings(
                       validation_run_id, job_id, run_id, topology_version_id,
                       node_id, session_id, template, source_manifest_hash, paths_json,
                       input_fingerprint,
                       action_id, actor, created_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    validation_run_id,
                    job_id,
                    run_id,
                    context.topology_version_id,
                    milestone["node_id"],
                    session_id,
                    template,
                    source_manifest_hash,
                    json.dumps(paths),
                    fingerprint,
                    action_id,
                    actor,
                    utc_text(),
                ),
            )
        self.import_validation_result(validation_run_id)

    def import_validation_result(self, validation_run_id: str) -> bool:
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT binding.*, validation.exit_code, validation.command_json,
                          validation.stdout_text, validation.stderr_text,
                          validation.completed_at, node.node_key, copy.source_root
                   FROM workflow_validation_bindings binding
                   LEFT JOIN validation_copy_runs validation
                     ON validation.run_id=binding.validation_run_id
                   JOIN validation_copies copy ON copy.job_id=binding.job_id
                   JOIN workflow_nodes node ON node.node_id=binding.node_id
                   WHERE binding.validation_run_id=?""",
                (validation_run_id,),
            ).fetchone()
        if row is None or row["exit_code"] is None or row["imported_at"] is not None:
            return False
        if not row["source_manifest_hash"] or not json.loads(row["paths_json"]):
            return self._reject_validation_binding(row, "validation_binding_legacy_unbound")
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
                "template": row["template"],
                "command": json.loads(row["command_json"]),
                "exitCode": int(row["exit_code"]),
                "completedAt": row["completed_at"],
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
        context = self.prepare_context(run_id, paths)
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
        failures = (
            self.failures.open_related_to_plan(plan_path)
            if self.failures is not None and plan_path
            else []
        )
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
                """SELECT node_id, node_key FROM workflow_nodes
                   WHERE run_id=? AND kind='milestone' ORDER BY node_key""",
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
            context = self.prepare_context(run_id, paths)
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
        self, run_id: str, paths: list[str] | tuple[str, ...]
    ) -> GateContext:
        normalized = tuple(sorted(set(paths), key=str.casefold))
        if not normalized:
            raise CoordinatorError("milestone_paths_empty", "Milestone commit requires paths")
        with self.database.connect() as connection:
            run = connection.execute(
                """SELECT workflow_runs.current_topology_version_id,
                          versions.content_hash, versions.plan_path
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
            if current_plan.content_hash != run["content_hash"]:
                raise CoordinatorError(
                    "workflow_topology_plan_changed",
                    "Plan content changed after the active topology version was imported",
                    details={
                        "activeContentHash": run["content_hash"],
                        "currentContentHash": current_plan.content_hash,
                    },
                )
            failure_rows = [
                dict(row)
                for row in connection.execute(
                    """SELECT lifecycle_key, status, kind, artifact_path
                       FROM failure_nodes
                       WHERE origin_plan=? OR fixing_plan=?
                       ORDER BY lifecycle_key, artifact_path""",
                    (run["plan_path"], run["plan_path"]),
                )
            ]
        baseline = self.baselines.current()
        return GateContext(
            topology_version_id=run["current_topology_version_id"],
            head_commit=self._git("rev-parse", "HEAD"),
            baseline_epoch=baseline.epoch_id,
            manifest_hash=self._manifest_hash(normalized),
            failure_revision=_hash_json(failure_rows),
            plan_content_hash=current_plan.content_hash,
        )

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
        self._require_run_owner(run_id, session_id)
        paths = self._derive_milestone_paths(session_id, run_id, milestone_key)
        context = self.prepare_context(run_id, paths)
        milestone = self._milestone_node(run_id, milestone_key)
        self._record_manifest(
            session_id=session_id,
            run_id=run_id,
            milestone=milestone,
            paths=paths,
            context=context,
            actor=actor,
            action_id=action_id,
        )
        return paths

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
            context = self.prepare_context(run_id, paths)
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
        message: str,
        actor: str,
        action_id: str | None = None,
    ) -> MilestoneCommitResult:
        self._require_run_owner(run_id, session_id)
        notification_module = None
        if self.notifications is not None:
            with self.database.connect() as connection:
                run = connection.execute(
                    "SELECT plan_path FROM workflow_runs WHERE run_id=?", (run_id,)
                ).fetchone()
            if run is None or not run["plan_path"]:
                raise CoordinatorError(
                    "notification_module_unavailable",
                    "Milestone notification requires a workflow run bound to a plan module",
                    details={"runId": run_id},
                )
            notification_module = plan_module_name(str(run["plan_path"]))
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
        context = self.prepare_context(run_id, paths)
        initial = self.gates.evaluate(run_id, milestone_key, context)
        self._require_allowed(initial)

        latest: GateDecision = initial
        milestone = self._milestone_node(run_id, milestone_key)
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
            refreshed_context = self.prepare_context(run_id, paths)
            latest = self.gates.evaluate(run_id, milestone_key, refreshed_context)
            self._require_allowed(latest)

        try:
            result = self.finalize.commit_milestone(
                session_id,
                paths=paths,
                message=message,
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
            assert notification_module is not None
            commit_time = self._git("show", "-s", "--format=%cI", result.commit_sha)
            commit_subject = self._git("show", "-s", "--format=%s", result.commit_sha)
            formatted = self.notifications.format_message(
                module=notification_module,
                summary=f"{milestone_key} 里程碑已通过全部门禁并完成提交",
                commit_time=commit_time,
                shortstat=shortstat or "0 files changed",
                commit_content=f"{result.commit_sha} {commit_subject}",
            )
            try:
                notification = self.notifications.notify_once(
                    commit_sha=result.commit_sha,
                    message=formatted,
                    run_id=run_id,
                    topology_version_id=context.topology_version_id,
                    node_id=milestone["node_id"],
                    action_id=action_id,
                )
            except CoordinatorError as error:
                if error.code != "notification_already_attempted":
                    raise
        return MilestoneCommitResult(result, latest, notification, shortstat)

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

    def close_goal(self, session_id: str, run_id: str) -> dict[str, object]:
        if self.sessions is None or self.leases is None:
            raise CoordinatorError("action_unavailable", "Goal closeout services are unavailable")
        self._require_run_owner(run_id, session_id)
        with self.database.connect() as connection:
            run = connection.execute(
                """SELECT run.current_topology_version_id, version.content_hash,
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
            if current_plan.content_hash != run["content_hash"]:
                raise CoordinatorError(
                    "workflow_topology_plan_changed",
                    "Plan content changed after the active topology version was imported",
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
                       WHERE run_id=? AND status <> 'reconciled'""",
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

    def _milestone_node(self, run_id: str, milestone_key: str):
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT * FROM workflow_nodes
                   WHERE run_id=? AND node_key=? AND kind='milestone'""",
                (run_id, milestone_key),
            ).fetchone()
        if row is None:
            raise CoordinatorError(
                "workflow_milestone_not_found", f"Unknown milestone {milestone_key}"
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
        owned = set(self.leases.owned_paths(session_id)) if self.leases is not None else set()
        if not normalized or not set(normalized) <= owned:
            raise CoordinatorError(
                "milestone_manifest_unleased",
                "Every milestone manifest path must be covered by the executing Session leases",
                details={"paths": list(normalized), "unleased": sorted(set(normalized) - owned)},
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
        for record in sorted(child.glob("*.md"), key=lambda item: item.name.casefold()):
            if record.name.startswith(("failure-", "fixed-")):
                continue
            fields, files = self._plan_output_fields(record)
            if fields.get("Plan") == session.plan_path and fields.get("Milestone") == milestone_key:
                relative = record.relative_to(self.repo_root).as_posix()
                matches.append((relative, files))
        if len(matches) != 1:
            raise CoordinatorError(
                "milestone_manifest_record_ambiguous",
                "Exactly one child-plan record must declare this milestone manifest",
                details={"records": [path for path, _ in matches]},
            )
        record_path, declared = matches[0]
        normalized: set[str] = {record_path}
        for raw in declared:
            candidate = (self.repo_root / raw).resolve()
            if not candidate.is_relative_to(self.repo_root) or candidate == self.repo_root:
                raise CoordinatorError(
                    "milestone_manifest_path_invalid", "Declared milestone path escaped repository"
                )
            normalized.add(candidate.relative_to(self.repo_root).as_posix())
        dirty_attributed = set(self.attributed_changes(session_id))
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
                digest = hashlib.sha256(absolute.read_bytes()).hexdigest()
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
