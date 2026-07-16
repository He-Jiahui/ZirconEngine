from __future__ import annotations

import json
import sqlite3


class WorkflowProjectionService:
    """Builds browser-safe workflow projections from one caller-owned transaction."""

    def workflow_summaries(
        self,
        connection: sqlite3.Connection,
        *,
        terminal_history_limit: int | None = None,
    ) -> list[dict[str, object]]:
        parameters: tuple[object, ...] = ()
        run_filter = ""
        if terminal_history_limit is not None:
            run_filter = """
            WHERE run.state NOT IN ('archived', 'stale', 'succeeded', 'cancelled')
               OR run.run_id IN (
                    SELECT run_id FROM workflow_runs
                    WHERE state IN ('archived', 'stale', 'succeeded', 'cancelled')
                    ORDER BY updated_at DESC, run_id DESC LIMIT ?
               )
            """
            parameters = (terminal_history_limit,)
        rows = connection.execute(
            f"""
            SELECT run.*,
                   COUNT(node.node_id) AS node_count,
                   SUM(CASE WHEN COALESCE(current.state, node.state) = 'succeeded'
                            THEN 1 ELSE 0 END) AS succeeded_count,
                   SUM(CASE WHEN COALESCE(current.state, node.state) = 'failed'
                            THEN 1 ELSE 0 END) AS failed_count
            FROM workflow_runs run
            LEFT JOIN workflow_nodes node ON node.run_id = run.run_id
            LEFT JOIN workflow_attempts current
              ON current.node_id = node.node_id
             AND current.accepted = 1
             AND current.attempt_number = (
                 SELECT MAX(latest.attempt_number)
                 FROM workflow_attempts latest
                 WHERE latest.node_id = node.node_id AND latest.accepted = 1
             )
            {run_filter}
            GROUP BY run.run_id
            ORDER BY run.updated_at DESC, run.run_id
            """,
            parameters,
        ).fetchall()
        return [
            {
                "runId": row["run_id"],
                "sessionId": row["session_id"],
                "workflowKey": row["workflow_key"],
                "planPath": row["plan_path"],
                "topologyHash": row["topology_hash"],
                "state": row["state"],
                "statusReason": row["status_reason"],
                "nodeCount": int(row["node_count"]),
                "succeededCount": int(row["succeeded_count"] or 0),
                "failedCount": int(row["failed_count"] or 0),
                "updatedAt": row["updated_at"],
            }
            for row in rows
        ]

    def workflow_detail(
        self, connection: sqlite3.Connection, run_id: str
    ) -> dict[str, object]:
        run = connection.execute(
            "SELECT * FROM workflow_runs WHERE run_id = ?", (run_id,)
        ).fetchone()
        if run is None:
            raise KeyError(run_id)
        node_rows = connection.execute(
            "SELECT * FROM workflow_nodes WHERE run_id = ? ORDER BY stage, node_key",
            (run_id,),
        ).fetchall()
        nodes: list[dict[str, object]] = []
        for node in node_rows:
            attempts = connection.execute(
                """
                SELECT * FROM workflow_attempts
                WHERE node_id = ? ORDER BY attempt_number
                """,
                (node["node_id"],),
            ).fetchall()
            attempt_history = [self._attempt_projection(attempt) for attempt in attempts]
            accepted = [attempt for attempt in attempts if bool(attempt["accepted"])]
            current = self._attempt_projection(accepted[-1]) if accepted else None
            eligibility = (
                self._commit_eligibility(
                    connection,
                    run_id,
                    node["node_id"],
                    run["current_topology_version_id"],
                )
                if node["kind"] == "milestone"
                else None
            )
            nodes.append(
                {
                    "nodeId": node["node_id"],
                    "nodeKey": node["node_key"],
                    "kind": node["kind"],
                    "title": node["title"],
                    "stage": node["stage"],
                    "state": current["state"] if current else node["state"],
                    "ownerSessionId": node["owner_session_id"],
                    "statusReason": node["status_reason"],
                    "currentAttempt": current,
                    "attemptHistory": attempt_history,
                    "commitEligibility": eligibility,
                }
            )
        edges = [
            {
                "fromNodeId": row["from_node_id"],
                "toNodeId": row["to_node_id"],
                "kind": row["edge_kind"],
            }
            for row in connection.execute(
                "SELECT * FROM workflow_edges WHERE run_id = ? ORDER BY from_node_id, to_node_id",
                (run_id,),
            )
        ]
        artifacts = []
        for row in connection.execute(
            """SELECT artifact_id, node_id, attempt_id, artifact_kind, display_name,
                      content_hash, byte_count, metadata_json, created_at
               FROM workflow_artifacts WHERE run_id = ? ORDER BY created_at, artifact_id""",
            (run_id,),
        ):
            artifacts.append(
                {
                    "artifactId": row["artifact_id"],
                    "nodeId": row["node_id"],
                    "attemptId": row["attempt_id"],
                    "kind": row["artifact_kind"],
                    "displayName": row["display_name"],
                    "contentHash": row["content_hash"],
                    "byteCount": row["byte_count"],
                    "metadata": json.loads(row["metadata_json"]),
                    "createdAt": row["created_at"],
                }
            )
        topology_versions = [
            {
                "topologyVersionId": row["topology_version_id"],
                "versionNumber": int(row["version_number"]),
                "schemaVersion": int(row["schema_version"]),
                "sourceKind": row["source_kind"],
                "contentHash": row["content_hash"],
                "topologyHash": row["topology_hash"],
                "supersedesId": row["supersedes_id"],
                "active": row["topology_version_id"] == run["current_topology_version_id"],
                "createdAt": row["created_at"],
            }
            for row in connection.execute(
                """SELECT * FROM workflow_topology_versions
                   WHERE run_id=? ORDER BY version_number""",
                (run_id,),
            )
        ]
        gates = [
            {
                "evidenceId": row["evidence_id"],
                "topologyVersionId": row["topology_version_id"],
                "nodeId": row["node_id"],
                "attemptId": row["attempt_id"],
                "kind": row["gate_kind"],
                "decision": row["decision"],
                "code": row["decision_code"],
                "inputFingerprint": row["input_fingerprint"],
                "blockingNodeIds": json.loads(row["blocking_node_ids_json"]),
                "applicableFailureIds": json.loads(row["applicable_failure_ids_json"]),
                "requiredEvidence": json.loads(row["required_evidence_json"]),
                "createdAt": row["created_at"],
            }
            for row in connection.execute(
                """SELECT * FROM workflow_gate_evidence
                   WHERE run_id=? ORDER BY created_at, evidence_id""",
                (run_id,),
            )
        ]
        reviews = [
            {
                "reviewId": row["review_id"],
                "topologyVersionId": row["topology_version_id"],
                "nodeId": row["node_id"],
                "attemptId": row["attempt_id"],
                "reviewer": row["reviewer"],
                "executor": row["executor"],
                "verdict": row["verdict"],
                "criticalCount": int(row["critical_count"]),
                "importantCount": int(row["important_count"]),
                "summary": row["summary"],
                "createdAt": row["created_at"],
            }
            for row in connection.execute(
                """SELECT * FROM workflow_review_evidence
                   WHERE run_id=? ORDER BY created_at, review_id""",
                (run_id,),
            )
        ]
        notifications = [
            {
                "attemptId": row["notification_attempt_id"],
                "commitSha": row["commit_sha"],
                "channel": row["channel"],
                "status": row["status"],
                "attemptedAt": row["attempted_at"],
                "completedAt": row["completed_at"],
                "exitCode": row["exit_code"],
                "providerErrcode": row["provider_errcode"],
                "sanitizedError": row["sanitized_error"],
                "retryAllowed": False,
            }
            for row in connection.execute(
                """SELECT * FROM notification_attempts
                   WHERE run_id=? ORDER BY attempted_at, notification_attempt_id""",
                (run_id,),
            )
        ]
        return {
            "runId": run["run_id"],
            "sessionId": run["session_id"],
            "workflowKey": run["workflow_key"],
            "planPath": run["plan_path"],
            "topologyHash": run["topology_hash"],
            "state": run["state"],
            "statusReason": run["status_reason"],
            "nodes": nodes,
            "edges": edges,
            "artifacts": artifacts,
            "topologyVersions": topology_versions,
            "gates": gates,
            "reviews": reviews,
            "notifications": notifications,
        }

    @staticmethod
    def _commit_eligibility(
        connection: sqlite3.Connection,
        run_id: str,
        node_id: str,
        topology_version_id: str | None,
    ) -> dict[str, object]:
        required = {
            "validation", "review", "failure_audit", "plan_output", "commit_manifest"
        }
        if topology_version_id is None:
            return {"eligible": False, "code": "workflow_topology_not_active", "missing": sorted(required)}
        rows = connection.execute(
            """SELECT evidence.* FROM workflow_gate_evidence evidence
               WHERE evidence.run_id=? AND evidence.topology_version_id=?
                 AND evidence.node_id=? AND evidence.rowid=(
                    SELECT latest.rowid FROM workflow_gate_evidence latest
                    WHERE latest.run_id=evidence.run_id
                      AND latest.topology_version_id=evidence.topology_version_id
                      AND latest.node_id=evidence.node_id
                      AND latest.gate_kind=evidence.gate_kind
                    ORDER BY latest.created_at DESC, latest.rowid DESC LIMIT 1
                 )""",
            (run_id, topology_version_id, node_id),
        ).fetchall()
        latest = {row["gate_kind"]: row for row in rows}
        missing = sorted(required - set(latest))
        rejected = sorted(
            kind for kind in required & set(latest) if latest[kind]["decision"] != "accepted"
        )
        fingerprints = {
            latest[kind]["input_fingerprint"] for kind in required & set(latest)
        }
        review_ok = False
        if "review" in latest:
            review_ok = connection.execute(
                """SELECT 1 FROM workflow_review_evidence
                   WHERE run_id=? AND topology_version_id=? AND node_id=?
                     AND input_fingerprint=? AND verdict='accepted' LIMIT 1""",
                (
                    run_id,
                    topology_version_id,
                    node_id,
                    latest["review"]["input_fingerprint"],
                ),
            ).fetchone() is not None
        eligible = not missing and not rejected and len(fingerprints) == 1 and review_ok
        code = "database_evidence_ready" if eligible else "database_evidence_not_ready"
        return {
            "eligible": eligible,
            "code": code,
            "missing": missing,
            "rejected": rejected,
            "fingerprintConsistent": len(fingerprints) <= 1,
            "independentReviewAccepted": review_ok,
        }

    @staticmethod
    def _attempt_projection(row: sqlite3.Row) -> dict[str, object]:
        return {
            "attemptId": row["attempt_id"],
            "attemptNumber": int(row["attempt_number"]),
            "state": row["state"],
            "accepted": bool(row["accepted"]),
            "evidence": json.loads(row["evidence_json"]),
            "startedAt": row["started_at"],
            "completedAt": row["completed_at"],
        }
