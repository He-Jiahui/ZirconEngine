from __future__ import annotations

import json
import sqlite3
import uuid
from dataclasses import dataclass
from pathlib import Path

from ..database import Database
from ..models import CoordinatorError, utc_text
from .store import WorkflowStore
from .topology import TopologyParser, WorkflowTopology


@dataclass(frozen=True, slots=True)
class TopologyImportResult:
    topology_version_id: str
    run_id: str
    version_number: int
    content_hash: str
    activated: bool


class TopologyImporter:
    """Persist immutable plan versions and activate only the first run graph."""

    def __init__(self, database: Database, repo_root: str | Path):
        self.database = database
        self.parser = TopologyParser(repo_root)
        self.store = WorkflowStore(database)

    def import_plan(
        self,
        session_id: str,
        plan_path: str,
        *,
        activate_candidate: bool = False,
    ) -> TopologyImportResult:
        topology = self.parser.parse(plan_path)
        run = self.store.ensure_session_run(session_id, topology.plan_path)
        with self.database.transaction() as connection:
            current = connection.execute(
                """SELECT run.topology_hash, run.current_topology_version_id,
                          version.content_hash AS current_content_hash
                   FROM workflow_runs AS run
                   LEFT JOIN workflow_topology_versions AS version
                     ON version.topology_version_id=run.current_topology_version_id
                   WHERE run.run_id=?""",
                (run.run_id,),
            ).fetchone()
            # Ordinary plan prose, status tables, and Failure links must not
            # split the active graph from its already-bound manifest evidence.
            # The current semantic topology is the identity boundary; content
            # remains audit metadata on that same version.
            if current["topology_hash"] == topology.topology_hash:
                existing = connection.execute(
                    "SELECT * FROM workflow_topology_versions WHERE topology_version_id=?",
                    (current["current_topology_version_id"],),
                ).fetchone()
                if existing is None:
                    raise CoordinatorError(
                        "workflow_topology_not_active",
                        "Workflow topology hash exists without its active version",
                    )
                return self._result(
                    existing,
                    activated=True,
                    observed_content_hash=topology.content_hash,
                )
            existing = connection.execute(
                """SELECT * FROM workflow_topology_versions
                   WHERE run_id=? AND content_hash=?
                   LIMIT 1""",
                (run.run_id, topology.content_hash),
            ).fetchone()
            if existing is not None:
                activated = current["current_topology_version_id"] == existing["topology_version_id"]
                if activate_candidate and not activated:
                    self._activate_candidate(
                        connection, run.run_id, session_id, topology, existing["topology_version_id"]
                    )
                    activated = True
                return self._result(
                    existing,
                    activated=activated,
                )
            version_number = int(
                connection.execute(
                    "SELECT COALESCE(MAX(version_number), 0) + 1 FROM workflow_topology_versions WHERE run_id=?",
                    (run.run_id,),
                ).fetchone()[0]
            )
            version_id = uuid.uuid4().hex
            supersedes = connection.execute(
                """SELECT topology_version_id FROM workflow_topology_versions
                   WHERE run_id=? ORDER BY version_number DESC LIMIT 1""",
                (run.run_id,),
            ).fetchone()
            connection.execute(
                """INSERT INTO workflow_topology_versions(
                       topology_version_id, run_id, version_number, plan_path,
                       plan_id, schema_version, source_kind, content_hash,
                       topology_hash, topology_json, supersedes_id, created_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    version_id,
                    run.run_id,
                    version_number,
                    topology.plan_path,
                    topology.plan_id,
                    topology.schema_version,
                    topology.source,
                    topology.content_hash,
                    topology.topology_hash,
                    topology.canonical_json(),
                    supersedes[0] if supersedes else None,
                    utc_text(),
                ),
            )
            activated = current["current_topology_version_id"] is None
            if activated:
                self._activate(connection, run.run_id, session_id, topology)
            elif activate_candidate:
                self._activate_candidate(connection, run.run_id, session_id, topology, version_id)
                activated = True
            inserted = connection.execute(
                "SELECT * FROM workflow_topology_versions WHERE topology_version_id=?",
                (version_id,),
            ).fetchone()
        return self._result(inserted, activated=activated)

    @classmethod
    def _activate_candidate(
        cls,
        connection: sqlite3.Connection,
        run_id: str,
        session_id: str,
        topology: WorkflowTopology,
        version_id: str,
    ) -> None:
        current = connection.execute(
            """SELECT run.topology_hash, version.topology_json
               FROM workflow_runs AS run
               LEFT JOIN workflow_topology_versions AS version
                 ON version.topology_version_id=run.current_topology_version_id
               WHERE run.run_id=?""",
            (run_id,),
        ).fetchone()
        if current["topology_hash"] == topology.topology_hash:
            connection.execute(
                """UPDATE workflow_runs
                   SET current_topology_version_id=?, updated_at=? WHERE run_id=?""",
                (version_id, utc_text(), run_id),
            )
            return
        progressed = int(connection.execute(
            """SELECT
                 (SELECT COUNT(*) FROM workflow_attempts attempt
                  JOIN workflow_nodes node ON node.node_id=attempt.node_id
                  WHERE node.run_id=? AND node.kind <> 'goal') +
                 (SELECT COUNT(*) FROM workflow_gate_evidence WHERE run_id=?) +
                 (SELECT COUNT(*) FROM workflow_review_evidence WHERE run_id=?) +
                 (SELECT COUNT(*) FROM workflow_milestone_manifests WHERE run_id=?) +
                 (SELECT COUNT(*) FROM workflow_validation_bindings WHERE run_id=?) +
                 (SELECT COUNT(*) FROM workflow_commit_intents WHERE run_id=?) +
                 (SELECT COUNT(*) FROM workflow_artifacts WHERE run_id=?)""",
            (run_id,) * 7,
        ).fetchone()[0])
        if progressed:
            if cls._can_append_milestones(current["topology_json"], topology) and not cls._has_live_attempts(
                connection, run_id
            ):
                cls._activate_append_only(
                    connection, run_id, session_id, topology, version_id
                )
                return
            raise CoordinatorError(
                "workflow_topology_activation_requires_pristine_run",
                "A structural plan revision cannot replace a workflow graph with accepted history",
                details={"runId": run_id, "attemptCount": progressed},
            )
        connection.execute("DELETE FROM workflow_edges WHERE run_id=?", (run_id,))
        connection.execute(
            "DELETE FROM workflow_nodes WHERE run_id=? AND node_key <> 'goal'", (run_id,)
        )
        cls._activate(connection, run_id, session_id, topology)

    @staticmethod
    def _can_append_milestones(
        previous_json: str | None, topology: WorkflowTopology
    ) -> bool:
        """Accept only a milestone tail; historic nodes and slices remain immutable."""
        if not previous_json:
            return False
        try:
            previous = json.loads(previous_json)
        except (TypeError, json.JSONDecodeError):
            return False
        if not isinstance(previous, dict):
            return False
        if any(
            previous.get(key) != value
            for key, value in (
                ("schema", topology.schema_version),
                ("workflow_id", topology.workflow_id),
                ("goal", topology.goal),
                ("plan_path", topology.plan_path),
                ("plan_id", topology.plan_id),
                ("source", topology.source),
            )
        ):
            return False
        old_milestones = previous.get("milestones")
        old_slices = previous.get("slices")
        if not isinstance(old_milestones, list) or not isinstance(old_slices, list):
            return False
        candidate_slices = [
            {
                "node_id": item.node_id,
                "title": item.title,
                "depends_on": list(item.depends_on),
                "milestone_id": item.milestone_id,
            }
            for item in topology.slices
        ]
        if old_slices != candidate_slices:
            return False
        old_by_id = {
            item.get("node_id"): item
            for item in old_milestones
            if isinstance(item, dict) and isinstance(item.get("node_id"), str)
        }
        if len(old_by_id) != len(old_milestones):
            return False
        candidate_by_id = {
            item.node_id: {
                "node_id": item.node_id,
                "title": item.title,
                "depends_on": list(item.depends_on),
                "milestone_id": item.milestone_id,
            }
            for item in topology.milestones
        }
        if not set(old_by_id) < set(candidate_by_id):
            return False
        return all(candidate_by_id[node_id] == prior for node_id, prior in old_by_id.items())

    @staticmethod
    def _has_live_attempts(connection: sqlite3.Connection, run_id: str) -> bool:
        """Appending is safe only between attempts; never rewrite an active gate graph."""
        return bool(connection.execute(
            """SELECT 1
               FROM workflow_attempts AS attempt
               JOIN workflow_nodes AS node ON node.node_id=attempt.node_id
               WHERE node.run_id=?
                 AND node.kind <> 'goal'
                 AND attempt.state IN ('pending', 'ready', 'running', 'waiting_external')
               LIMIT 1""",
            (run_id,),
        ).fetchone())

    @staticmethod
    def _activate_append_only(
        connection: sqlite3.Connection,
        run_id: str,
        session_id: str,
        topology: WorkflowTopology,
        version_id: str,
    ) -> None:
        """Advance only the new tail while keeping historical node identity and evidence."""
        existing = {
            row["node_key"]: row["node_id"]
            for row in connection.execute(
                "SELECT node_id, node_key FROM workflow_nodes WHERE run_id=?", (run_id,)
            )
        }
        now = utc_text()
        additions = [item for item in topology.milestones if item.node_id not in existing]
        for item in additions:
            existing[item.node_id] = f"{run_id}:{item.node_id}"
        connection.executemany(
            """INSERT INTO workflow_nodes(
                   node_id, run_id, node_key, kind, title, stage, state,
                   owner_session_id, created_at, updated_at
               ) VALUES (?, ?, ?, 'milestone', ?, 'milestone', 'pending', ?, ?, ?)""",
            (
                (
                    existing[item.node_id],
                    run_id,
                    item.node_id,
                    item.title,
                    session_id,
                    now,
                    now,
                )
                for item in additions
            ),
        )
        connection.executemany(
            """INSERT INTO workflow_edges(run_id, from_node_id, to_node_id, edge_kind)
               VALUES (?, ?, ?, 'depends_on')""",
            (
                (run_id, existing[dependency], existing[item.node_id])
                for item in additions
                for dependency in item.depends_on
            ),
        )
        connection.execute(
            """UPDATE workflow_runs
               SET topology_hash=?, current_topology_version_id=?, updated_at=?
               WHERE run_id=?""",
            (topology.topology_hash, version_id, now, run_id),
        )

    @staticmethod
    def _activate(
        connection: sqlite3.Connection,
        run_id: str,
        session_id: str,
        topology: WorkflowTopology,
    ) -> None:
        now = utc_text()
        node_ids: dict[str, str] = {}
        for item in topology.milestones:
            node_ids[item.node_id] = f"{run_id}:{item.node_id}"
        for item in topology.slices:
            node_ids[item.node_id] = f"{run_id}:{item.node_id}"
        connection.executemany(
            """INSERT INTO workflow_nodes(
                   node_id, run_id, node_key, kind, title, stage, state,
                   owner_session_id, created_at, updated_at
               ) VALUES (?, ?, ?, 'milestone', ?, 'milestone', 'pending', ?, ?, ?)""",
            (
                (
                    node_ids[item.node_id],
                    run_id,
                    item.node_id,
                    item.title,
                    session_id,
                    now,
                    now,
                )
                for item in topology.milestones
            ),
        )
        connection.executemany(
            """INSERT INTO workflow_nodes(
                   node_id, run_id, node_key, kind, title, stage, state,
                   owner_session_id, created_at, updated_at
               ) VALUES (?, ?, ?, 'slice', ?, ?, 'pending', ?, ?, ?)""",
            (
                (
                    node_ids[item.node_id],
                    run_id,
                    item.node_id,
                    item.title,
                    item.milestone_id or "slice",
                    session_id,
                    now,
                    now,
                )
                for item in topology.slices
            ),
        )
        connection.executemany(
            """INSERT INTO workflow_edges(
                   run_id, from_node_id, to_node_id, edge_kind
               ) VALUES (?, ?, ?, 'depends_on')""",
            (
                (run_id, node_ids[dependency], node_ids[item.node_id])
                for item in topology.milestones
                for dependency in item.depends_on
            ),
        )
        connection.executemany(
            """INSERT INTO workflow_edges(
                   run_id, from_node_id, to_node_id, edge_kind
               ) VALUES (?, ?, ?, 'depends_on')""",
            (
                (run_id, node_ids[item.node_id], node_ids[item.milestone_id or ""])
                for item in topology.slices
            ),
        )
        connection.execute(
            """UPDATE workflow_runs
               SET topology_hash=?, current_topology_version_id=(
                       SELECT topology_version_id FROM workflow_topology_versions
                       WHERE run_id=? AND content_hash=?
                   ), updated_at=?
               WHERE run_id=?""",
            (topology.topology_hash, run_id, topology.content_hash, now, run_id),
        )

    @staticmethod
    def _result(
        row: sqlite3.Row,
        *,
        activated: bool,
        observed_content_hash: str | None = None,
    ) -> TopologyImportResult:
        return TopologyImportResult(
            topology_version_id=row["topology_version_id"],
            run_id=row["run_id"],
            version_number=int(row["version_number"]),
            content_hash=observed_content_hash or row["content_hash"],
            activated=activated,
        )
