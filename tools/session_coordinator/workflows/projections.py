from __future__ import annotations

import json
import sqlite3


class WorkflowProjectionService:
    """Builds browser-safe workflow projections from one caller-owned transaction."""

    def workflow_summaries(self, connection: sqlite3.Connection) -> list[dict[str, object]]:
        rows = connection.execute(
            """
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
            GROUP BY run.run_id
            ORDER BY run.updated_at DESC, run.run_id
            """
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
