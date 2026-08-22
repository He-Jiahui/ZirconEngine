from __future__ import annotations

import hashlib
import json
import re
import subprocess
import uuid
from dataclasses import asdict, dataclass
from pathlib import Path

from ..baselines import BaselineService, hash_file
from ..database import Database
from ..failure_return_delegations import (
    FailureReturnDelegationProof,
    FailureReturnDelegationService,
)
from ..failures import FailureGraphService, FailureNode
from ..git_finalize import FinalizeResult, GitFinalizeService
from ..models import CoordinatorError, SessionStatus, utc_text
from ..notifications import NotificationAttemptRecord, WeComNotificationService
from ..snapshots import SnapshotRecord, SnapshotService
from .milestones import plan_module_name


@dataclass(frozen=True, slots=True)
class PreservedFailure:
    lifecycle_key: str
    artifact_path: str
    related_code: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class PreparedFailureCloseout:
    closeout_id: str
    session_id: str
    snapshot_id: int
    lifecycle_key: str
    lifecycle_keys: tuple[str, ...]
    fixing_plan: str
    target_artifact: str
    target_artifacts: tuple[str, ...]
    additional_paths: tuple[str, ...]
    paths: tuple[str, ...]
    return_records: tuple[str, ...]
    validation_command: tuple[str, ...]
    validation_job_id: str
    validation_run_id: str
    validation_compatibility_key: str
    validation_contract_hash: str
    executor_thread_id: str
    delegated_return_proofs: tuple[FailureReturnDelegationProof, ...]
    preserved_open_failures: tuple[PreservedFailure, ...]
    input_fingerprint: str


@dataclass(frozen=True, slots=True)
class FailureCloseoutEvidence:
    evidence_id: str
    verdict: str
    input_fingerprint: str


@dataclass(frozen=True, slots=True)
class FailureCloseoutCommitResult:
    finalize: FinalizeResult
    notification: NotificationAttemptRecord | None
    shortstat: str
    staged_total: int
    preserved_open_failures: tuple[PreservedFailure, ...]


_FRONTMATTER = re.compile(
    r"\A---\r?\n(?P<header>.*?)\r?\n---(?:\r?\n|\Z)", re.DOTALL
)


class FailureCloseoutWorkflowService:
    """Commit one fixed lifecycle while its owner keeps unrelated failures open."""

    PREPARED_EVENT = "failure.closeout.prepared"
    VALIDATION_EVENT = "failure.closeout.validation"
    REVIEW_EVENT = "failure.closeout.review"
    COMMITTED_EVENT = "failure.closeout.committed"
    COMMIT_PREPARED_EVENT = "failure.closeout.commit_prepared"

    def __init__(
        self,
        database: Database,
        repo_root: str | Path,
        baselines: BaselineService,
        finalize: GitFinalizeService,
        snapshots: SnapshotService,
        failures: FailureGraphService,
        notifications: WeComNotificationService | None,
        *,
        sessions,
        leases,
        delegations: FailureReturnDelegationService,
    ):
        self.database = database
        self.repo_root = Path(repo_root).resolve()
        self.baselines = baselines
        self.finalize = finalize
        self.snapshots = snapshots
        self.failures = failures
        self.notifications = notifications
        self.sessions = sessions
        self.leases = leases
        self.delegations = delegations

    def prepare(
        self,
        *,
        session_id: str,
        snapshot_id: int,
        lifecycle_key: str,
        validation_command: list[str] | tuple[str, ...],
        validation_job_id: str,
        validation_run_id: str,
        executor_thread_id: str,
        actor: str,
        action_id: str | None = None,
    ) -> PreparedFailureCloseout:
        return self._prepare_targets(
            session_id=session_id,
            snapshot_id=snapshot_id,
            lifecycle_keys=(lifecycle_key,),
            delivery_records=(),
            validation_command=validation_command,
            validation_job_id=validation_job_id,
            validation_run_id=validation_run_id,
            executor_thread_id=executor_thread_id,
            actor=actor,
            action_id=action_id,
            allow_preserved_overlap=False,
        )

    def prepare_combined(
        self,
        *,
        session_id: str,
        snapshot_id: int,
        lifecycle_keys: list[str] | tuple[str, ...],
        delivery_records: list[str] | tuple[str, ...],
        validation_command: list[str] | tuple[str, ...],
        validation_job_id: str,
        validation_run_id: str,
        executor_thread_id: str,
        actor: str,
        action_id: str | None = None,
    ) -> PreparedFailureCloseout:
        normalized_keys = tuple(
            sorted(
                {str(key).strip() for key in lifecycle_keys if str(key).strip()},
                key=str.casefold,
            )
        )
        if len(normalized_keys) < 2:
            raise CoordinatorError(
                "failure_closeout_combined_targets_invalid",
                "Combined Failure closeout requires at least two distinct lifecycle keys",
            )
        return self._prepare_targets(
            session_id=session_id,
            snapshot_id=snapshot_id,
            lifecycle_keys=normalized_keys,
            delivery_records=delivery_records,
            validation_command=validation_command,
            validation_job_id=validation_job_id,
            validation_run_id=validation_run_id,
            executor_thread_id=executor_thread_id,
            actor=actor,
            action_id=action_id,
            allow_preserved_overlap=False,
        )

    def _prepare_targets(
        self,
        *,
        session_id: str,
        snapshot_id: int,
        lifecycle_keys: tuple[str, ...],
        delivery_records: list[str] | tuple[str, ...],
        validation_command: list[str] | tuple[str, ...],
        validation_job_id: str,
        validation_run_id: str,
        executor_thread_id: str,
        actor: str,
        action_id: str | None,
        allow_preserved_overlap: bool,
    ) -> PreparedFailureCloseout:
        session = self.sessions.get(session_id)
        if session.status is not SessionStatus.RESOLVING_FAILURE:
            raise CoordinatorError(
                "failure_closeout_session_not_resolving",
                "Failure closeout requires a Session in resolving_failure",
            )
        if not session.plan_path:
            raise CoordinatorError(
                "failure_closeout_plan_missing",
                "Failure closeout requires the owner Session plan",
            )
        snapshot = self.snapshots.get(snapshot_id)
        if snapshot.session_id != session_id:
            raise CoordinatorError(
                "failure_closeout_snapshot_owner_mismatch",
                "Failure closeout snapshot belongs to another Session",
            )
        self._require_snapshot_current(snapshot)
        command = tuple(str(item) for item in validation_command)
        if not command:
            raise CoordinatorError(
                "failure_closeout_validation_command_missing",
                "Failure closeout prepare requires the exact managed validation command",
            )
        self._require_codex_task_provenance(executor_thread_id)
        validation_contract = self._load_validation_contract(
            session_id=session_id,
            job_id=validation_job_id,
            cargo_run_id=validation_run_id,
            command=command,
        )
        audit = self.failures.import_repository()
        indexed = {node.lifecycle_key: node for node in audit.nodes}
        targets: list[FailureNode] = []
        for lifecycle_key in lifecycle_keys:
            target = indexed.get(lifecycle_key)
            if target is None or target.kind != "fixed" or target.status != "fixed":
                raise CoordinatorError(
                    "failure_closeout_target_not_fixed",
                    "Failure closeout target must be a fixed lifecycle",
                    details={"lifecycleKey": lifecycle_key},
                )
            if target.fixing_plan != session.plan_path:
                raise CoordinatorError(
                    "failure_closeout_fixing_plan_mismatch",
                    "All Failure closeout targets must share the owner Session fixing plan",
                    details={"lifecycleKey": lifecycle_key},
                )
            self._require_target_diagnostics_clean(audit.diagnostics, target.artifact_path)
            targets.append(target)
        targets_tuple = tuple(targets)
        paths = tuple(sorted(snapshot.manifest, key=str.casefold))
        return_bindings = tuple(
            self._return_records(paths, target) for target in targets_tuple
        )
        returns = tuple(binding[0] for binding in return_bindings)
        source_artifacts = tuple(binding[1] for binding in return_bindings)
        extras = self._typed_delivery_paths(delivery_records, lifecycle_keys)
        required = {
            *extras,
            *returns,
            *source_artifacts,
            *(target.artifact_path for target in targets_tuple),
            *(path for target in targets_tuple for path in target.related_code),
        }
        if set(paths) != required:
            raise CoordinatorError(
                "failure_closeout_manifest_not_exact",
                "Failure closeout snapshot must exactly match all target lifecycles and explicit supporting paths",
                details={
                    "missing": sorted(required - set(paths), key=str.casefold),
                    "extra": sorted(set(paths) - required, key=str.casefold),
                },
            )
        source_not_deleted = sorted(
            (
                path
                for path in source_artifacts
                if snapshot.manifest[path] is not None
            ),
            key=str.casefold,
        )
        if source_not_deleted:
            raise CoordinatorError(
                "failure_closeout_source_artifact_not_deleted",
                "Failure closeout requires each prior child-record artifact as a deletion",
                details={"paths": source_not_deleted},
            )
        preserved = self._preserved_open_failures(
            session.plan_path, excluded_keys=frozenset(lifecycle_keys)
        )
        protected_paths = {
            path
            for failure in preserved
            for path in (failure.artifact_path, *failure.related_code)
        }
        overlap = sorted(set(paths) & protected_paths, key=str.casefold)
        if overlap:
            raise CoordinatorError(
                "failure_closeout_preserved_scope_overlap",
                "Failure closeout manifest overlaps another open Failure lifecycle",
                details={"paths": overlap},
            )
        delegated_return_proofs = self.delegations.prepare_proofs(
            fixing_session_id=session_id,
            lifecycle_keys=lifecycle_keys,
            manifest_paths=paths,
        )
        material = self._material(
            snapshot=snapshot,
            targets=targets_tuple,
            additional_paths=extras,
            paths=paths,
            return_records=returns,
            validation_command=command,
            validation_job_id=validation_job_id,
            validation_run_id=validation_run_id,
            validation_compatibility_key=str(
                validation_contract["compatibilityKey"]
            ),
            validation_contract_hash=self._hash(validation_contract),
            executor_thread_id=executor_thread_id,
            delegated_return_proofs=delegated_return_proofs,
            preserved=preserved,
        )
        closeout_id = uuid.uuid4().hex
        fingerprint = self._hash(material)
        payload = {
            "closeoutId": closeout_id,
            "actor": actor,
            "actionId": action_id,
            "inputFingerprint": fingerprint,
            **material,
        }
        self._append_event(session_id, self.PREPARED_EVENT, payload)
        return self._prepared(payload)

    def bind_validation(
        self,
        *,
        session_id: str,
        closeout_id: str,
        job_id: str,
        cargo_run_id: str,
        actor: str,
        action_id: str | None = None,
    ) -> FailureCloseoutEvidence:
        prepared = self._refresh(session_id, closeout_id)
        if job_id != prepared.validation_job_id or cargo_run_id != prepared.validation_run_id:
            raise CoordinatorError(
                "failure_closeout_validation_contract_mismatch",
                "Managed validation job and run differ from the prepare-bound contract",
            )
        command = self._require_terminal_validation(
            prepared,
            job_id=job_id,
            cargo_run_id=cargo_run_id,
        )
        evidence_id = uuid.uuid4().hex
        self._append_event(
            session_id,
            self.VALIDATION_EVENT,
            {
                "closeoutId": closeout_id,
                "evidenceId": evidence_id,
                "verdict": "accepted",
                "inputFingerprint": prepared.input_fingerprint,
                "jobId": job_id,
                "cargoRunId": cargo_run_id,
                "command": list(command),
                "actor": actor,
                "actionId": action_id,
            },
        )
        return FailureCloseoutEvidence(
            evidence_id, "accepted", prepared.input_fingerprint
        )

    def record_review(
        self,
        *,
        session_id: str,
        closeout_id: str,
        reviewer_session_id: str,
        reviewer_thread_id: str,
        critical_count: int,
        important_count: int,
        moderate_count: int,
        summary: str,
        action_id: str | None = None,
    ) -> FailureCloseoutEvidence:
        prepared = self._refresh(session_id, closeout_id)
        if reviewer_session_id == session_id:
            raise CoordinatorError(
                "failure_closeout_review_not_independent",
                "Failure closeout review must use an independent registered Session",
            )
        if reviewer_thread_id == prepared.executor_thread_id:
            raise CoordinatorError(
                "failure_closeout_review_not_independent",
                "Failure closeout review must come from a different Codex task than prepare",
            )
        self._require_reviewer_provenance(
            reviewer_session_id=reviewer_session_id,
            reviewer_thread_id=reviewer_thread_id,
        )
        reviewer = self.sessions.get(reviewer_session_id)
        if reviewer.status is not SessionStatus.ACTIVE:
            raise CoordinatorError(
                "failure_closeout_reviewer_not_active",
                "Failure closeout review must be submitted by an active reviewer Session",
            )
        with self.database.connect() as connection:
            reviewer_paths = {
                str(row["display_path"])
                for row in connection.execute(
                    "SELECT display_path FROM attributions WHERE session_id=?",
                    (reviewer_session_id,),
                )
            }
        overlap = sorted(reviewer_paths & set(prepared.paths), key=str.casefold)
        if overlap:
            raise CoordinatorError(
                "failure_closeout_reviewer_not_independent",
                "Reviewer Session owns paths in the closeout manifest",
                details={"paths": overlap},
            )
        counts = (critical_count, important_count, moderate_count)
        if any(value < 0 for value in counts):
            raise CoordinatorError(
                "failure_closeout_review_count_invalid",
                "Failure closeout review counts cannot be negative",
            )
        if not summary.strip():
            raise CoordinatorError(
                "failure_closeout_review_summary_missing",
                "Failure closeout review requires a non-empty summary",
            )
        verdict = "accepted" if counts == (0, 0, 0) else "rejected"
        evidence_id = uuid.uuid4().hex
        self._append_event(
            session_id,
            self.REVIEW_EVENT,
            {
                "closeoutId": closeout_id,
                "evidenceId": evidence_id,
                "verdict": verdict,
                "inputFingerprint": prepared.input_fingerprint,
                "reviewerSessionId": reviewer_session_id,
                "reviewerThreadId": reviewer_thread_id,
                "executorSessionId": session_id,
                "criticalCount": critical_count,
                "importantCount": important_count,
                "moderateCount": moderate_count,
                "summary": summary.strip(),
                "actionId": action_id,
            },
        )
        return FailureCloseoutEvidence(
            evidence_id, verdict, prepared.input_fingerprint
        )

    def commit(
        self,
        *,
        session_id: str,
        closeout_id: str,
        summary: str,
        actor: str,
        action_id: str | None = None,
    ) -> FailureCloseoutCommitResult:
        prepared = self._refresh(session_id, closeout_id)
        validation = self._require_accepted_evidence(
            session_id,
            closeout_id,
            prepared.input_fingerprint,
            self.VALIDATION_EVENT,
            "failure_closeout_validation_missing",
        )
        self._require_terminal_validation(
            prepared,
            job_id=str(validation["jobId"]),
            cargo_run_id=str(validation["cargoRunId"]),
        )
        self._require_accepted_evidence(
            session_id,
            closeout_id,
            prepared.input_fingerprint,
            self.REVIEW_EVENT,
            "failure_closeout_review_missing",
        )

        def guard() -> None:
            refreshed = self._refresh(session_id, closeout_id)
            if refreshed.input_fingerprint != prepared.input_fingerprint:
                raise CoordinatorError(
                    "failure_closeout_evidence_stale",
                    "Failure closeout state changed under the Git mutex",
                )
            validation = self._require_accepted_evidence(
                session_id,
                closeout_id,
                refreshed.input_fingerprint,
                self.VALIDATION_EVENT,
                "failure_closeout_validation_missing",
            )
            self._require_terminal_validation(
                refreshed,
                job_id=str(validation["jobId"]),
                cargo_run_id=str(validation["cargoRunId"]),
            )
            self._require_accepted_evidence(
                session_id,
                closeout_id,
                refreshed.input_fingerprint,
                self.REVIEW_EVENT,
                "failure_closeout_review_missing",
            )

        request_id = uuid.uuid4().hex
        self._append_event(
            session_id,
            self.COMMIT_PREPARED_EVENT,
            {
                "closeoutId": closeout_id,
                "requestId": request_id,
                "summary": summary,
                "actor": actor,
                "actionId": action_id,
                "inputFingerprint": prepared.input_fingerprint,
            },
        )
        result = self.finalize.commit_failure_closeouts(
            session_id,
            paths=prepared.paths,
            message=summary,
            lifecycle_keys=prepared.lifecycle_keys,
            precommit_guard=guard,
            delegated_paths=tuple(
                proof.destination_path for proof in prepared.delegated_return_proofs
            ),
            delegation_guard=lambda: self.delegations.require_for_commit(
                fixing_session_id=session_id,
                closeout_id=closeout_id,
                input_fingerprint=prepared.input_fingerprint,
                lifecycle_keys=prepared.lifecycle_keys,
                manifest_paths=prepared.paths,
                proofs=prepared.delegated_return_proofs,
            ),
            delegation_consumer=lambda commit_sha: self.delegations.consume(
                fixing_session_id=session_id,
                closeout_id=closeout_id,
                input_fingerprint=prepared.input_fingerprint,
                lifecycle_keys=prepared.lifecycle_keys,
                manifest_paths=prepared.paths,
                proofs=prepared.delegated_return_proofs,
                commit_sha=commit_sha,
            ),
            request_id=request_id,
        )
        return self._complete_committed(
            prepared,
            result,
            summary=summary,
            actor=actor,
            action_id=action_id,
        )

    def recover_pending_commits(self) -> tuple[str, ...]:
        """Reconcile post-CAS closeouts and finish their durable notification attempt."""
        with self.database.connect() as connection:
            rows = connection.execute(
                """SELECT session_id, payload_json FROM events
                   WHERE event_type=? ORDER BY event_id""",
                (self.COMMIT_PREPARED_EVENT,),
            ).fetchall()
        recovered: list[str] = []
        for row in rows:
            payload = json.loads(row["payload_json"])
            request_id = str(payload["requestId"])
            with self.database.connect() as connection:
                request = connection.execute(
                    "SELECT * FROM finalize_requests WHERE request_id=?",
                    (request_id,),
                ).fetchone()
            if request is None or not (
                request["commit_sha"] or request["ref_updated_sha"]
            ):
                continue
            if request["status"] == "committed" and request["commit_sha"]:
                result = FinalizeResult(
                    request_id,
                    str(request["commit_sha"]),
                    str(request["message"]),
                    {
                        key: tuple(value)
                        for key, value in json.loads(
                            request["categories_json"]
                        ).items()
                    },
                    tuple(json.loads(request["untracked_json"])),
                )
            else:
                result = self.finalize.reconcile_request(request_id)
                if result is None:
                    continue
            prepared_payload = self._event_payload(
                str(row["session_id"]),
                self.PREPARED_EVENT,
                str(payload["closeoutId"]),
            )
            prepared = self._prepared(prepared_payload)
            self.delegations.consume(
                fixing_session_id=prepared.session_id,
                closeout_id=prepared.closeout_id,
                input_fingerprint=prepared.input_fingerprint,
                lifecycle_keys=prepared.lifecycle_keys,
                manifest_paths=prepared.paths,
                proofs=prepared.delegated_return_proofs,
                commit_sha=result.commit_sha,
            )
            committed = self._event_payload(
                prepared.session_id,
                self.COMMITTED_EVENT,
                prepared.closeout_id,
                required=False,
            )
            with self.database.connect() as connection:
                notification = connection.execute(
                    """SELECT 1 FROM notification_attempts
                       WHERE commit_sha=? AND channel='wecom'""",
                    (result.commit_sha,),
                ).fetchone()
            if committed and notification is not None:
                continue
            self._complete_committed(
                prepared,
                result,
                summary=str(payload["summary"]),
                actor=str(payload["actor"]),
                action_id=(
                    str(payload["actionId"])
                    if payload.get("actionId") is not None
                    else None
                ),
            )
            recovered.append(request_id)
        return tuple(recovered)

    def _complete_committed(
        self,
        prepared: PreparedFailureCloseout,
        result: FinalizeResult,
        *,
        summary: str,
        actor: str,
        action_id: str | None,
    ) -> FailureCloseoutCommitResult:
        shortstat = self._git(
            "show", "--shortstat", "--format=", result.commit_sha
        ).strip()
        existing = self._event_payload(
            prepared.session_id,
            self.COMMITTED_EVENT,
            prepared.closeout_id,
            required=False,
        )
        if not existing:
            self._append_event(
                prepared.session_id,
                self.COMMITTED_EVENT,
                {
                    "closeoutId": prepared.closeout_id,
                    "requestId": result.request_id,
                    "commitSha": result.commit_sha,
                    "shortstat": shortstat,
                    "inputFingerprint": prepared.input_fingerprint,
                    "actor": actor,
                    "actionId": action_id,
                    "preservedOpenFailures": [
                        asdict(item) for item in prepared.preserved_open_failures
                    ],
                },
            )
        notification = self._notify(
            prepared,
            result,
            shortstat,
            summary,
            action_id,
        )
        staged_total = len(
            [
                line
                for line in self._git(
                    "diff", "--cached", "--name-only"
                ).splitlines()
                if line
            ]
        )
        return FailureCloseoutCommitResult(
            result,
            notification,
            shortstat,
            staged_total,
            prepared.preserved_open_failures,
        )

    def _refresh(
        self, session_id: str, closeout_id: str
    ) -> PreparedFailureCloseout:
        payload = self._event_payload(session_id, self.PREPARED_EVENT, closeout_id)
        prepared = self._prepared(payload)
        if prepared.session_id != session_id:
            raise CoordinatorError(
                "failure_closeout_owner_mismatch",
                "Failure closeout belongs to another Session",
            )
        session = self.sessions.get(session_id)
        if session.status is not SessionStatus.RESOLVING_FAILURE:
            raise CoordinatorError(
                "failure_closeout_session_not_resolving",
                "Failure closeout owner is no longer resolving_failure",
            )
        snapshot = self.snapshots.get(prepared.snapshot_id)
        self._require_snapshot_current(snapshot)
        audit = self.failures.import_repository()
        indexed = {node.lifecycle_key: node for node in audit.nodes}
        targets: list[FailureNode] = []
        for lifecycle_key in prepared.lifecycle_keys:
            target = indexed.get(lifecycle_key)
            if (
                target is None
                or target.kind != "fixed"
                or target.status != "fixed"
                or target.fixing_plan != prepared.fixing_plan
            ):
                raise CoordinatorError(
                    "failure_closeout_target_not_fixed",
                    "A prepared Failure lifecycle is no longer fixed by the owner plan",
                    details={"lifecycleKey": lifecycle_key},
                )
            self._require_target_diagnostics_clean(audit.diagnostics, target.artifact_path)
            targets.append(target)
        targets_tuple = tuple(targets)
        refreshed_bindings = tuple(
            self._return_records(prepared.paths, target) for target in targets_tuple
        )
        refreshed_returns = tuple(binding[0] for binding in refreshed_bindings)
        if refreshed_returns != prepared.return_records:
            raise CoordinatorError(
                "failure_closeout_return_record_invalid",
                "Prepared Failure return records changed after admission",
            )
        refreshed_sources = tuple(binding[1] for binding in refreshed_bindings)
        if any(
            source not in prepared.paths or snapshot.manifest.get(source) is not None
            for source in refreshed_sources
        ):
            raise CoordinatorError(
                "failure_closeout_source_artifact_not_deleted",
                "A prepared prior Failure artifact is no longer an exact deletion",
                details={"paths": list(refreshed_sources)},
            )
        preserved = self._preserved_open_failures(
            prepared.fixing_plan,
            excluded_keys=frozenset(prepared.lifecycle_keys),
        )
        material = self._material(
            snapshot=snapshot,
            targets=targets_tuple,
            additional_paths=prepared.additional_paths,
            paths=prepared.paths,
            return_records=prepared.return_records,
            validation_command=prepared.validation_command,
            validation_job_id=prepared.validation_job_id,
            validation_run_id=prepared.validation_run_id,
            validation_compatibility_key=prepared.validation_compatibility_key,
            validation_contract_hash=prepared.validation_contract_hash,
            executor_thread_id=prepared.executor_thread_id,
            delegated_return_proofs=self.delegations.prepare_proofs(
                fixing_session_id=session_id,
                lifecycle_keys=prepared.lifecycle_keys,
                manifest_paths=prepared.paths,
            ),
            preserved=preserved,
        )
        if self._hash(material) != prepared.input_fingerprint:
            raise CoordinatorError(
                "failure_closeout_state_changed",
                "Failure closeout snapshot, graph, baseline, or HEAD changed",
            )
        return prepared

    def _material(
        self,
        *,
        snapshot: SnapshotRecord,
        targets: tuple[FailureNode, ...],
        additional_paths: tuple[str, ...],
        paths: tuple[str, ...],
        return_records: tuple[str, ...],
        validation_command: tuple[str, ...],
        validation_job_id: str,
        validation_run_id: str,
        validation_compatibility_key: str,
        validation_contract_hash: str,
        executor_thread_id: str,
        delegated_return_proofs: tuple[FailureReturnDelegationProof, ...],
        preserved: tuple[PreservedFailure, ...],
    ) -> dict[str, object]:
        baseline = self.baselines.current()
        head = self._git("rev-parse", "HEAD")
        if baseline.head_commit != head:
            raise CoordinatorError(
                "failure_closeout_baseline_head_changed",
                "Failure closeout requires the coordinator baseline at current HEAD",
            )
        target = targets[0]
        material: dict[str, object] = {
            "sessionId": snapshot.session_id,
            "snapshotId": snapshot.snapshot_id,
            "baselineEpoch": baseline.epoch_id,
            "headCommit": head,
            "lifecycleKey": target.lifecycle_key,
            "fixingPlan": target.fixing_plan,
            "targetArtifact": target.artifact_path,
            "paths": list(paths),
            "manifestHash": self._hash(snapshot.manifest),
            "returnRecords": list(return_records),
            "validationCommand": list(validation_command),
            "validationJobId": validation_job_id,
            "validationRunId": validation_run_id,
            "validationCompatibilityKey": validation_compatibility_key,
            "validationContractHash": validation_contract_hash,
            "executorThreadId": executor_thread_id,
            "delegatedReturnProofs": [
                proof.to_dict() for proof in delegated_return_proofs
            ],
            "preservedOpenFailures": [asdict(item) for item in preserved],
        }
        if len(targets) > 1 or additional_paths:
            material.update(
                {
                    "lifecycleKeys": [item.lifecycle_key for item in targets],
                    "targetArtifacts": [item.artifact_path for item in targets],
                    "targets": [
                        {
                            "lifecycleKey": item.lifecycle_key,
                            "artifactPath": item.artifact_path,
                            "relatedCode": list(item.related_code),
                            "returnRecord": return_record,
                        }
                        for item, return_record in zip(targets, return_records, strict=True)
                    ],
                    "additionalPaths": list(additional_paths),
                }
            )
        return material

    def _return_records(
        self, paths: tuple[str, ...], target: FailureNode
    ) -> tuple[str, str]:
        matches: list[tuple[str, str]] = []
        expected_source = (
            f"{target.fixing_child_dir}/failure-{target.created_at}-{target.summary_slug}.md"
        )
        for path in paths:
            if not path.casefold().startswith("docs/plans/") or path == target.artifact_path:
                continue
            if not (self.repo_root / path).is_file():
                continue
            metadata = self._frontmatter(self.repo_root / path)
            if metadata.get("record_kind") != "failure_return_status":
                continue
            source_artifact = metadata.get("source_artifact", expected_source).replace(
                "\\", "/"
            )
            if (
                metadata.get("status") == "fixed"
                and metadata.get("summary_slug") == target.summary_slug
                and metadata.get("origin_plan") == target.origin_plan
                and metadata.get("fixing_plan") == target.fixing_plan
                and source_artifact == expected_source
            ):
                matches.append((path, source_artifact))
        if len(matches) != 1:
            raise CoordinatorError(
                "failure_closeout_return_record_invalid",
                "Failure closeout requires exactly one matching fixed return-status record",
                details={"paths": [item[0] for item in matches]},
            )
        return matches[0]

    def _typed_delivery_paths(
        self,
        delivery_records: list[str] | tuple[str, ...],
        lifecycle_keys: tuple[str, ...],
    ) -> tuple[str, ...]:
        records = tuple(
            sorted(
                {
                    str(path).replace("\\", "/")
                    for path in delivery_records
                    if str(path)
                },
                key=str.casefold,
            )
        )
        if not records:
            return ()
        authorized = set(records)
        expected_keys = tuple(sorted(lifecycle_keys, key=str.casefold))
        for record in records:
            metadata = self._frontmatter(self.repo_root / record)
            if metadata.get("record_kind") != "failure_closeout_delivery":
                raise CoordinatorError(
                    "failure_closeout_delivery_record_invalid",
                    "Combined closeout supplemental paths require a typed delivery record",
                    details={"path": record},
                )
            try:
                raw_keys = json.loads(metadata["lifecycle_keys_json"])
                raw_paths = json.loads(metadata["delivery_paths_json"])
                if not isinstance(raw_keys, list) or not isinstance(raw_paths, list):
                    raise TypeError("typed delivery fields must be arrays")
                if not all(isinstance(item, str) and item for item in (*raw_keys, *raw_paths)):
                    raise TypeError("typed delivery arrays must contain non-empty strings")
                record_keys = tuple(
                    sorted(
                        set(raw_keys),
                        key=str.casefold,
                    )
                )
                delivery_paths = {
                    item.replace("\\", "/") for item in raw_paths
                }
            except (KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
                raise CoordinatorError(
                    "failure_closeout_delivery_record_invalid",
                    "Failure closeout delivery record contains invalid typed JSON",
                    details={"path": record},
                ) from error
            if record_keys != expected_keys or any(not path for path in delivery_paths):
                raise CoordinatorError(
                    "failure_closeout_delivery_record_invalid",
                    "Failure closeout delivery record is not bound to the exact targets and paths",
                    details={"path": record},
                )
            authorized.update(delivery_paths)
        return tuple(sorted(authorized, key=str.casefold))

    @staticmethod
    def _frontmatter(path: Path) -> dict[str, str]:
        try:
            text = path.read_text(encoding="utf-8")
        except OSError as error:
            raise CoordinatorError(
                "failure_closeout_return_record_unavailable",
                f"Failure return record is unavailable: {path}",
            ) from error
        match = _FRONTMATTER.match(text)
        if match is None:
            return {}
        result: dict[str, str] = {}
        for line in match.group("header").splitlines():
            key, separator, value = line.partition(":")
            if separator and key and not key[0].isspace():
                result[key.strip()] = value.strip().strip("'\"")
        return result

    def _preserved_open_failures(
        self,
        fixing_plan: str,
        *,
        excluded_keys: frozenset[str] = frozenset(),
    ) -> tuple[PreservedFailure, ...]:
        return tuple(
            PreservedFailure(
                node.lifecycle_key,
                node.artifact_path,
                node.related_code,
            )
            for node in self.failures.open_for_plan(fixing_plan)
            if node.lifecycle_key not in excluded_keys
        )

    @staticmethod
    def _require_target_diagnostics_clean(
        diagnostics,
        target_artifact: str,
    ) -> None:
        target = target_artifact.casefold()
        blocking = [
            diagnostic.message
            for diagnostic in diagnostics
            if target in diagnostic.message.replace("\\", "/").casefold()
            or any(path.casefold() == target for path in diagnostic.paths)
        ]
        if blocking:
            raise CoordinatorError(
                "failure_closeout_target_invalid",
                "The target fixed lifecycle has canonical Markdown diagnostics",
                details={"diagnostics": blocking},
            )

    def _require_snapshot_current(self, snapshot: SnapshotRecord) -> None:
        drift = [
            path
            for path, expected in snapshot.manifest.items()
            if hash_file(self.repo_root / path) != expected
        ]
        if drift:
            raise CoordinatorError(
                "failure_closeout_snapshot_drift",
                "Failure closeout snapshot differs from the current worktree",
                details={"paths": sorted(drift, key=str.casefold)},
            )

    def _require_accepted_evidence(
        self,
        session_id: str,
        closeout_id: str,
        fingerprint: str,
        event_type: str,
        error_code: str,
    ) -> dict[str, object]:
        payload = self._event_payload(
            session_id, event_type, closeout_id, required=False
        )
        if (
            not payload
            or payload.get("verdict") != "accepted"
            or payload.get("inputFingerprint") != fingerprint
        ):
            raise CoordinatorError(
                error_code,
                "Failure closeout requires accepted current-source evidence",
            )
        return payload

    def _require_reviewer_provenance(
        self, *, reviewer_session_id: str, reviewer_thread_id: str
    ) -> None:
        if reviewer_session_id != reviewer_thread_id:
            raise CoordinatorError(
                "failure_closeout_reviewer_provenance_invalid",
                "Reviewer Session must be the calling Codex task identity",
            )
        self._require_codex_task_provenance(
            reviewer_thread_id, bound_session_id=reviewer_session_id
        )

    def _require_codex_task_provenance(
        self, thread_id: str, *, bound_session_id: str | None = None
    ) -> None:
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT source_location, state, cwd, bound_session_id
                   FROM codex_sessions WHERE thread_id=?""",
                (thread_id,),
            ).fetchone()
        try:
            cwd_matches = row is not None and Path(str(row["cwd"])).resolve() == self.repo_root
        except (OSError, ValueError):
            cwd_matches = False
        if (
            row is None
            or (
                bound_session_id is not None
                and row["bound_session_id"] != bound_session_id
            )
            or row["source_location"] != "active"
            or row["state"] not in {"active", "idle"}
            or not cwd_matches
        ):
            raise CoordinatorError(
                "failure_closeout_reviewer_provenance_invalid",
                "Codex task provenance is not live and bound in this repository",
            )

    def _load_validation_contract(
        self,
        *,
        session_id: str,
        job_id: str,
        cargo_run_id: str,
        command: tuple[str, ...],
    ) -> dict[str, object]:
        with self.database.connect() as connection:
            job = connection.execute(
                "SELECT * FROM cargo_jobs WHERE job_id=?", (job_id,)
            ).fetchone()
            run = connection.execute(
                "SELECT * FROM cargo_job_runs WHERE run_id=?", (cargo_run_id,)
            ).fetchone()
        if (
            job is None
            or run is None
            or job["session_id"] != session_id
            or run["session_id"] != session_id
            or run["job_id"] != job_id
        ):
            raise CoordinatorError(
                "failure_closeout_validation_owner_mismatch",
                "Managed validation job and run must belong to the closeout Session",
            )
        try:
            job_command = tuple(str(item) for item in json.loads(job["command_json"] or "[]"))
            run_command = tuple(str(item) for item in json.loads(run["command_json"] or "[]"))
            compatibility = json.loads(job["compatibility_json"] or "{}")
            environment = json.loads(run["environment_json"] or "{}")
        except (TypeError, ValueError, json.JSONDecodeError) as error:
            raise CoordinatorError(
                "failure_closeout_validation_contract_invalid",
                "Managed validation compatibility or environment is not valid JSON",
            ) from error
        if command != job_command or command != run_command:
            raise CoordinatorError(
                "failure_closeout_validation_command_mismatch",
                "Managed validation command differs from the prepare-bound command",
            )
        compatibility_key = str(job["compatibility_key"] or "").strip()
        if not isinstance(compatibility, dict) or not isinstance(environment, dict) or not compatibility_key:
            raise CoordinatorError(
                "failure_closeout_validation_contract_invalid",
                "Managed validation requires structured compatibility, environment, and compatibility key",
            )
        return {
            "jobId": job_id,
            "runId": cargo_run_id,
            "laneKind": str(job["lane_kind"]),
            "targetDir": str(job["target_dir"]),
            "compatibilityKey": compatibility_key,
            "compatibility": compatibility,
            "environment": environment,
        }

    def _require_terminal_validation(
        self,
        prepared: PreparedFailureCloseout,
        *,
        job_id: str,
        cargo_run_id: str,
    ) -> tuple[str, ...]:
        with self.database.connect() as connection:
            job = connection.execute(
                "SELECT * FROM cargo_jobs WHERE job_id=?", (job_id,)
            ).fetchone()
            run = connection.execute(
                "SELECT * FROM cargo_job_runs WHERE run_id=?", (cargo_run_id,)
            ).fetchone()
        if (
            job is None
            or run is None
            or job["session_id"] != prepared.session_id
            or run["session_id"] != prepared.session_id
            or run["job_id"] != job_id
        ):
            raise CoordinatorError(
                "failure_closeout_validation_owner_mismatch",
                "Managed validation job and run must belong to the closeout Session",
            )
        if (
            job["status"] != "released"
            or int(job["exit_code"] if job["exit_code"] is not None else -1) != 0
            or run["status"] != "completed"
            or int(run["exit_code"] if run["exit_code"] is not None else -1) != 0
            or json.loads(job["process_tree_live_pids_json"] or "[]")
            or not job["process_tree_exited_at"]
        ):
            raise CoordinatorError(
                "failure_closeout_validation_not_green",
                "Managed validation must be terminal exit 0 with no live process tree",
            )
        command = prepared.validation_command
        contract = self._load_validation_contract(
            session_id=prepared.session_id,
            job_id=job_id,
            cargo_run_id=cargo_run_id,
            command=command,
        )
        compatibility = contract["compatibility"]
        source_manifest = compatibility.get("source_manifest")
        expected_source = {
            path: digest
            for path, digest in self.snapshots.get(prepared.snapshot_id).manifest.items()
            if not path.casefold().startswith("docs/plans/")
        }
        normalized_source = (
            {str(path): str(digest).casefold() for path, digest in source_manifest.items()}
            if isinstance(source_manifest, dict)
            else {}
        )
        if normalized_source != {
            path: str(digest).casefold() for path, digest in expected_source.items()
        }:
            raise CoordinatorError(
                "failure_closeout_validation_source_drift",
                "Managed validation source manifest differs from the closeout snapshot",
            )
        if (
            contract["compatibilityKey"] != prepared.validation_compatibility_key
            or self._hash(contract) != prepared.validation_contract_hash
        ):
            raise CoordinatorError(
                "failure_closeout_validation_contract_drift",
                "Managed validation compatibility or environment changed after prepare",
            )
        return command

    def _notify(
        self,
        prepared: PreparedFailureCloseout,
        result: FinalizeResult,
        shortstat: str,
        summary: str,
        action_id: str | None,
    ) -> NotificationAttemptRecord | None:
        if self.notifications is None:
            return None
        try:
            commit_time = self._git("show", "-s", "--format=%cI", result.commit_sha)
            subject = self._git("show", "-s", "--format=%s", result.commit_sha)
            message = self.notifications.format_message(
                module=plan_module_name(prepared.fixing_plan),
                summary=f"Failure {Path(prepared.target_artifact).stem}: {summary}",
                commit_time=commit_time,
                shortstat=shortstat or "0 files changed",
                commit_content=f"{result.commit_sha} {subject}",
            )
            return self.notifications.notify_once(
                commit_sha=result.commit_sha,
                message=message,
                action_id=action_id,
            )
        except Exception as error:
            return self.notifications.record_post_commit_failure(
                commit_sha=result.commit_sha,
                error=error,
                action_id=action_id,
            )

    def _event_payload(
        self,
        session_id: str,
        event_type: str,
        closeout_id: str,
        *,
        required: bool = True,
    ) -> dict[str, object]:
        with self.database.connect() as connection:
            rows = connection.execute(
                """SELECT payload_json FROM events
                   WHERE session_id=? AND event_type=? ORDER BY event_id DESC""",
                (session_id, event_type),
            ).fetchall()
        for row in rows:
            payload = json.loads(row["payload_json"])
            if payload.get("closeoutId") == closeout_id:
                return payload
        if required:
            raise CoordinatorError(
                "failure_closeout_not_found",
                f"Unknown Failure closeout {closeout_id}",
            )
        return {}

    def _append_event(
        self, session_id: str, event_type: str, payload: dict[str, object]
    ) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO events(session_id, event_type, payload_json, created_at)
                   VALUES (?, ?, ?, ?)""",
                (session_id, event_type, json.dumps(payload, sort_keys=True), utc_text()),
            )

    @staticmethod
    def _prepared(payload: dict[str, object]) -> PreparedFailureCloseout:
        preserved = tuple(
            PreservedFailure(
                str(item["lifecycle_key"]),
                str(item["artifact_path"]),
                tuple(str(path) for path in item["related_code"]),
            )
            for item in payload["preservedOpenFailures"]
        )
        lifecycle_keys = tuple(
            str(item)
            for item in payload.get("lifecycleKeys", [payload["lifecycleKey"]])
        )
        target_artifacts = tuple(
            str(item)
            for item in payload.get("targetArtifacts", [payload["targetArtifact"]])
        )
        return PreparedFailureCloseout(
            closeout_id=str(payload["closeoutId"]),
            session_id=str(payload["sessionId"]),
            snapshot_id=int(payload["snapshotId"]),
            lifecycle_key=str(payload["lifecycleKey"]),
            lifecycle_keys=lifecycle_keys,
            fixing_plan=str(payload["fixingPlan"]),
            target_artifact=str(payload["targetArtifact"]),
            target_artifacts=target_artifacts,
            additional_paths=tuple(
                str(path) for path in payload.get("additionalPaths", [])
            ),
            paths=tuple(str(path) for path in payload["paths"]),
            return_records=tuple(str(path) for path in payload["returnRecords"]),
            validation_command=tuple(
                str(argument) for argument in payload["validationCommand"]
            ),
            validation_job_id=str(payload["validationJobId"]),
            validation_run_id=str(payload["validationRunId"]),
            validation_compatibility_key=str(payload["validationCompatibilityKey"]),
            validation_contract_hash=str(payload["validationContractHash"]),
            executor_thread_id=str(payload["executorThreadId"]),
            delegated_return_proofs=tuple(
                FailureReturnDelegationProof.from_dict(item)
                for item in payload.get("delegatedReturnProofs", [])
            ),
            preserved_open_failures=preserved,
            input_fingerprint=str(payload["inputFingerprint"]),
        )

    @staticmethod
    def _hash(value: object) -> str:
        return hashlib.sha256(
            json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
        ).hexdigest()

    def _git(self, *arguments: str) -> str:
        result = subprocess.run(
            ["git", *arguments],
            cwd=self.repo_root,
            check=True,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        return result.stdout.strip()
