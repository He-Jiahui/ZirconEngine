from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from pathlib import Path

from tools.session_coordinator.database import Database
from tools.session_coordinator.models import utc_text


@dataclass(frozen=True, slots=True)
class ControlLoadShape:
    sessions: int = 200
    workflows: int = 100
    nodes: int = 5_000
    events: int = 100_000
    artifacts: int = 10_000
    log_bytes: int = 500 * 1024 * 1024


class ControlLoadFixture:
    """Deterministically seeds only temporary coordinator databases and artifacts."""

    def __init__(self, database: Database, artifact_root: Path, shape: ControlLoadShape | None = None):
        self.database = database
        self.artifact_root = artifact_root.resolve()
        self.shape = shape or ControlLoadShape()
        if self.shape.nodes % self.shape.workflows:
            raise ValueError("nodes must divide evenly across workflows")
        if self.shape.artifacts % self.shape.workflows:
            raise ValueError("artifacts must divide evenly across workflows")

    def seed(self) -> None:
        now = utc_text()
        nodes_per_workflow = self.shape.nodes // self.shape.workflows
        artifacts_per_workflow = self.shape.artifacts // self.shape.workflows
        sessions = [
            (
                self.session_id(index),
                f"Load Session {index:03d}",
                f"docs/plans/load/{index:03d}-plan.md",
                "active",
                "load fixture",
                "0" * 40,
                json.dumps([f"load/{index:03d}"]),
                now,
                now,
                now,
            )
            for index in range(self.shape.sessions)
        ]
        runs = [
            (
                self.run_id(index),
                self.session_id(index),
                f"load-workflow-{index:03d}",
                f"docs/plans/load/{index:03d}-plan.md",
                hashlib.sha256(f"topology:{index}".encode()).hexdigest(),
                "active",
                "load fixture",
                now,
                now,
            )
            for index in range(self.shape.workflows)
        ]
        nodes = []
        attempts = []
        artifacts = []
        for workflow in range(self.shape.workflows):
            run_id = self.run_id(workflow)
            session_id = self.session_id(workflow)
            for offset in range(nodes_per_workflow):
                node_id = self.node_id(workflow, offset)
                state = "succeeded" if offset % 5 else "running"
                nodes.append(
                    (
                        node_id,
                        run_id,
                        f"M{offset + 1}",
                        "slice",
                        f"Load node {workflow:03d}/{offset:03d}",
                        "implementation",
                        state,
                        session_id,
                        1,
                        now,
                        now,
                    )
                )
                attempts.append(
                    (
                        f"attempt-{workflow:03d}-{offset:03d}",
                        run_id,
                        node_id,
                        1,
                        state,
                        json.dumps({"fixture": True}),
                        now,
                        now if state == "succeeded" else None,
                    )
                )
            for offset in range(artifacts_per_workflow):
                artifact_id = f"artifact-{workflow:03d}-{offset:03d}"
                artifacts.append(
                    (
                        artifact_id,
                        run_id,
                        "report",
                        f"Artifact {workflow:03d}/{offset:03d}",
                        None,
                        hashlib.sha256(artifact_id.encode()).hexdigest(),
                        0,
                        "{}",
                        now,
                    )
                )
        log_path = self.artifact_root / "load-500mb.log"
        self.artifact_root.mkdir(parents=True, exist_ok=True)
        with log_path.open("wb") as stream:
            stream.truncate(self.shape.log_bytes)
        artifacts[0] = (
            artifacts[0][0],
            artifacts[0][1],
            "log",
            "500 MB load log",
            log_path.name,
            hashlib.sha256(b"sparse-load-log").hexdigest(),
            self.shape.log_bytes,
            "{}",
            now,
        )
        with self.database.transaction() as connection:
            connection.executemany(
                """INSERT INTO sessions(
                       session_id, display_name, plan_path, status, status_reason,
                       base_head, write_scope_json, created_at, updated_at, last_heartbeat_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                sessions,
            )
            connection.executemany(
                """INSERT INTO workflow_runs(
                       run_id, session_id, workflow_key, plan_path, topology_hash,
                       state, status_reason, created_at, updated_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                runs,
            )
            connection.executemany(
                """INSERT INTO workflow_nodes(
                       node_id, run_id, node_key, kind, title, stage, state,
                       owner_session_id, attempt_count, created_at, updated_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                nodes,
            )
            connection.executemany(
                """INSERT INTO workflow_attempts(
                       attempt_id, run_id, node_id, attempt_number, state, accepted,
                       evidence_json, started_at, completed_at
                   ) VALUES (?, ?, ?, ?, ?, 1, ?, ?, ?)""",
                attempts,
            )
            connection.executemany(
                """INSERT INTO workflow_artifacts(
                       artifact_id, run_id, artifact_kind, display_name, storage_path,
                       content_hash, byte_count, metadata_json, created_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                artifacts,
            )
            batch_size = 5_000
            for start in range(0, self.shape.events, batch_size):
                stop = min(start + batch_size, self.shape.events)
                connection.executemany(
                    """INSERT INTO events(session_id, event_type, payload_json, created_at)
                       VALUES (?, 'load.fixture', ?, ?)""",
                    (
                        (
                            self.session_id(index % self.shape.sessions),
                            json.dumps({"sequence": index}),
                            now,
                        )
                        for index in range(start, stop)
                    ),
                )

    @staticmethod
    def session_id(index: int) -> str:
        return f"load-session-{index:03d}"

    @staticmethod
    def run_id(index: int) -> str:
        return f"load-run-{index:03d}"

    @staticmethod
    def node_id(workflow: int, offset: int) -> str:
        return f"load-node-{workflow:03d}-{offset:03d}"
