from __future__ import annotations

import hashlib
import json
import uuid
from dataclasses import dataclass

from ..database import Database
from ..models import CoordinatorError, utc_text


@dataclass(frozen=True, slots=True)
class GateContext:
    topology_version_id: str
    head_commit: str
    baseline_epoch: int
    manifest_hash: str
    failure_revision: str
    plan_content_hash: str


@dataclass(frozen=True, slots=True)
class GateDecision:
    allowed: bool
    code: str
    blocking_node_ids: tuple[str, ...]
    applicable_failure_ids: tuple[str, ...]
    required_evidence: tuple[str, ...]
    current_attempt_ids: tuple[str, ...]
    input_fingerprint: str


@dataclass(frozen=True, slots=True)
class ReviewEvidenceRecord:
    review_id: str
    verdict: str
    critical_count: int
    important_count: int


class GateEvidenceStore:
    """Append evidence bound to the exact state fingerprint it evaluated."""

    def __init__(self, database: Database):
        self.database = database

    def record_gate(
        self,
        *,
        run_id: str,
        topology_version_id: str,
        gate_kind: str,
        decision: str,
        decision_code: str,
        input_fingerprint: str,
        actor: str,
        node_id: str | None = None,
        attempt_id: str | None = None,
        blocking_node_ids: tuple[str, ...] = (),
        applicable_failure_ids: tuple[str, ...] = (),
        required_evidence: tuple[str, ...] = (),
        payload: dict[str, object] | None = None,
        source_revision: str | None = None,
        action_id: str | None = None,
    ) -> str:
        if decision not in {"accepted", "rejected", "stale"}:
            raise CoordinatorError(
                "workflow_gate_decision_invalid", f"Invalid gate decision: {decision}"
            )
        evidence_id = uuid.uuid4().hex
        material = {
            "runId": run_id,
            "topologyVersionId": topology_version_id,
            "nodeId": node_id,
            "attemptId": attempt_id,
            "gateKind": gate_kind,
            "decision": decision,
            "decisionCode": decision_code,
            "inputFingerprint": input_fingerprint,
            "blockingNodeIds": list(blocking_node_ids),
            "applicableFailureIds": list(applicable_failure_ids),
            "requiredEvidence": list(required_evidence),
            "payload": payload or {},
            "sourceRevision": source_revision,
            "actor": actor,
            "actionId": action_id,
        }
        evidence_hash = _hash_json(material)
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO workflow_gate_evidence(
                       evidence_id, run_id, topology_version_id, node_id,
                       attempt_id, gate_kind, decision, decision_code,
                       input_fingerprint, evidence_hash, blocking_node_ids_json,
                       applicable_failure_ids_json, required_evidence_json,
                       payload_json, source_revision, actor, action_id, created_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    evidence_id,
                    run_id,
                    topology_version_id,
                    node_id,
                    attempt_id,
                    gate_kind,
                    decision,
                    decision_code,
                    input_fingerprint,
                    evidence_hash,
                    json.dumps(blocking_node_ids),
                    json.dumps(applicable_failure_ids),
                    json.dumps(required_evidence),
                    json.dumps(payload or {}, sort_keys=True),
                    source_revision,
                    actor,
                    action_id,
                    utc_text(),
                ),
            )
        return evidence_id

    def record_review(
        self,
        *,
        run_id: str,
        topology_version_id: str,
        reviewer: str,
        executor: str,
        critical_count: int,
        important_count: int,
        summary: str,
        input_fingerprint: str,
        node_id: str | None = None,
        attempt_id: str | None = None,
    ) -> ReviewEvidenceRecord:
        if reviewer == executor:
            raise CoordinatorError(
                "workflow_review_not_independent",
                "Milestone review must be performed by a different executor",
            )
        if critical_count < 0 or important_count < 0:
            raise CoordinatorError(
                "workflow_review_count_invalid", "Review finding counts cannot be negative"
            )
        verdict = (
            "accepted" if critical_count == 0 and important_count == 0 else "rejected"
        )
        review_id = uuid.uuid4().hex
        material = {
            "runId": run_id,
            "topologyVersionId": topology_version_id,
            "nodeId": node_id,
            "attemptId": attempt_id,
            "reviewer": reviewer,
            "executor": executor,
            "verdict": verdict,
            "criticalCount": critical_count,
            "importantCount": important_count,
            "summary": summary,
            "inputFingerprint": input_fingerprint,
        }
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO workflow_review_evidence(
                       review_id, run_id, topology_version_id, node_id,
                       attempt_id, reviewer, executor, verdict, critical_count,
                       important_count, evidence_hash, input_fingerprint,
                       summary, created_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    review_id,
                    run_id,
                    topology_version_id,
                    node_id,
                    attempt_id,
                    reviewer,
                    executor,
                    verdict,
                    critical_count,
                    important_count,
                    _hash_json(material),
                    input_fingerprint,
                    summary,
                    utc_text(),
                ),
            )
        return ReviewEvidenceRecord(
            review_id, verdict, critical_count, important_count
        )


class MilestoneGateEvaluator:
    REQUIRED_GATE_KINDS = (
        "validation",
        "review",
        "failure_audit",
        "plan_output",
        "commit_manifest",
    )

    def __init__(self, database: Database):
        self.database = database

    def input_fingerprint(
        self, run_id: str, milestone_key: str, context: GateContext
    ) -> str:
        with self.database.connect() as connection:
            attempt_ids = self._current_attempt_ids(connection, run_id, milestone_key)
        return _hash_json(
            {
                "runId": run_id,
                "milestoneKey": milestone_key,
                "topologyVersionId": context.topology_version_id,
                "headCommit": context.head_commit,
                "baselineEpoch": context.baseline_epoch,
                "manifestHash": context.manifest_hash,
                "failureRevision": context.failure_revision,
                "planContentHash": context.plan_content_hash,
                "currentAttemptIds": list(attempt_ids),
            }
        )

    def evaluate(
        self, run_id: str, milestone_key: str, context: GateContext
    ) -> GateDecision:
        with self.database.connect() as connection:
            run = connection.execute(
                """SELECT current_topology_version_id FROM workflow_runs
                   WHERE run_id=?""",
                (run_id,),
            ).fetchone()
            if run is None:
                raise KeyError(run_id)
            current_attempt_ids = self._current_attempt_ids(
                connection, run_id, milestone_key
            )
            fingerprint = self.input_fingerprint(run_id, milestone_key, context)
            if run["current_topology_version_id"] != context.topology_version_id:
                return GateDecision(
                    False,
                    "milestone_gate_topology_stale",
                    (),
                    (),
                    ("current_topology",),
                    current_attempt_ids,
                    fingerprint,
                )
            incoming = self._incoming_nodes(connection, run_id, milestone_key)
            milestone = connection.execute(
                "SELECT node_id FROM workflow_nodes WHERE run_id=? AND node_key=? AND kind='milestone'",
                (run_id, milestone_key),
            ).fetchone()
            if milestone is None:
                raise CoordinatorError(
                    "workflow_milestone_not_found", f"Unknown milestone {milestone_key}"
                )
            milestone_node_id = milestone["node_id"]
            blocking = tuple(
                row["node_id"]
                for row in incoming
                if row["current_state"] not in {"succeeded", "skipped"}
            )
            if blocking:
                return GateDecision(
                    False,
                    "milestone_gate_nodes_incomplete",
                    blocking,
                    (),
                    (),
                    current_attempt_ids,
                    fingerprint,
                )
            latest = self._latest_evidence(
                connection, run_id, context.topology_version_id, milestone_node_id
            )
            missing = tuple(kind for kind in self.REQUIRED_GATE_KINDS if kind not in latest)
            if missing:
                return GateDecision(
                    False,
                    "milestone_gate_evidence_missing",
                    (),
                    (),
                    missing,
                    current_attempt_ids,
                    fingerprint,
                )
            stale = tuple(
                kind
                for kind in self.REQUIRED_GATE_KINDS
                if latest[kind]["input_fingerprint"] != fingerprint
            )
            applicable_failures = tuple(
                sorted(
                    {
                        failure_id
                        for row in latest.values()
                        for failure_id in json.loads(row["applicable_failure_ids_json"])
                    }
                )
            )
            if stale:
                return GateDecision(
                    False,
                    "milestone_gate_stale_evidence",
                    (),
                    applicable_failures,
                    stale,
                    current_attempt_ids,
                    fingerprint,
                )
            review = connection.execute(
                """SELECT * FROM workflow_review_evidence
                   WHERE run_id=? AND topology_version_id=? AND node_id=?
                     AND input_fingerprint=? AND verdict='accepted'
                   ORDER BY created_at DESC, review_id DESC LIMIT 1""",
                (run_id, context.topology_version_id, milestone_node_id, fingerprint),
            ).fetchone()
            if review is None:
                return GateDecision(
                    False,
                    "milestone_gate_review_missing",
                    (),
                    applicable_failures,
                    ("independent_review",),
                    current_attempt_ids,
                    fingerprint,
                )
            rejected = tuple(
                kind
                for kind in self.REQUIRED_GATE_KINDS
                if latest[kind]["decision"] != "accepted"
            )
            if rejected:
                return GateDecision(
                    False,
                    "milestone_gate_rejected",
                    (),
                    applicable_failures,
                    rejected,
                    current_attempt_ids,
                    fingerprint,
                )
        return GateDecision(
            True,
            "milestone_gate_allowed",
            (),
            applicable_failures,
            (),
            current_attempt_ids,
            fingerprint,
        )

    @staticmethod
    def _incoming_nodes(connection, run_id: str, milestone_key: str):
        return connection.execute(
            """SELECT source.node_id,
                      COALESCE((
                          SELECT attempt.state FROM workflow_attempts attempt
                          WHERE attempt.node_id=source.node_id AND attempt.accepted=1
                          ORDER BY attempt.attempt_number DESC LIMIT 1
                      ), source.state) AS current_state
               FROM workflow_nodes milestone
               JOIN workflow_edges edge
                 ON edge.run_id=milestone.run_id AND edge.to_node_id=milestone.node_id
               JOIN workflow_nodes source ON source.node_id=edge.from_node_id
               WHERE milestone.run_id=? AND milestone.node_key=?
               ORDER BY source.node_key""",
            (run_id, milestone_key),
        ).fetchall()

    @classmethod
    def _current_attempt_ids(
        cls, connection, run_id: str, milestone_key: str
    ) -> tuple[str, ...]:
        node_ids = [row["node_id"] for row in cls._incoming_nodes(connection, run_id, milestone_key)]
        if not node_ids:
            return ()
        placeholders = ",".join("?" for _ in node_ids)
        rows = connection.execute(
            f"""SELECT attempt_id FROM workflow_attempts current
                 WHERE node_id IN ({placeholders}) AND accepted=1
                   AND attempt_number=(
                       SELECT MAX(latest.attempt_number)
                       FROM workflow_attempts latest
                       WHERE latest.node_id=current.node_id AND latest.accepted=1
                   )
                 ORDER BY node_id""",
            node_ids,
        ).fetchall()
        return tuple(row["attempt_id"] for row in rows)

    @staticmethod
    def _latest_evidence(
        connection, run_id: str, topology_version_id: str, node_id: str
    ):
        rows = connection.execute(
            """SELECT evidence.* FROM workflow_gate_evidence evidence
               WHERE evidence.run_id=? AND evidence.topology_version_id=?
                 AND evidence.node_id=?
                 AND evidence.rowid=(
                     SELECT latest.rowid FROM workflow_gate_evidence latest
                     WHERE latest.run_id=evidence.run_id
                       AND latest.topology_version_id=evidence.topology_version_id
                       AND latest.node_id=evidence.node_id
                       AND latest.gate_kind=evidence.gate_kind
                     ORDER BY latest.created_at DESC, latest.rowid DESC LIMIT 1
                 )
               ORDER BY evidence.evidence_id""",
            (run_id, topology_version_id, node_id),
        ).fetchall()
        return {row["gate_kind"]: row for row in rows}


def _hash_json(value: object) -> str:
    encoded = json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()
