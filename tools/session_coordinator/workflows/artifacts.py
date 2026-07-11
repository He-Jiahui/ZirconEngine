from __future__ import annotations

import hashlib
import json
import uuid
from dataclasses import dataclass

from ..database import Database
from ..models import WorkflowArtifactKind, utc_text


@dataclass(frozen=True, slots=True)
class WorkflowArtifactRecord:
    artifact_id: str
    content_hash: str
    byte_count: int


class WorkflowArtifactStore:
    """Persist metadata for immutable evidence; large content stays out of SQLite."""

    def __init__(self, database: Database):
        self.database = database

    def record_bytes(
        self,
        *,
        run_id: str,
        node_id: str | None,
        attempt_id: str | None,
        kind: WorkflowArtifactKind,
        display_name: str,
        content: bytes,
        storage_path: str | None = None,
        metadata: dict[str, object] | None = None,
    ) -> WorkflowArtifactRecord:
        artifact_id = uuid.uuid4().hex
        digest = hashlib.sha256(content).hexdigest()
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO workflow_artifacts(
                       artifact_id, run_id, node_id, attempt_id, artifact_kind,
                       display_name, storage_path, content_hash, byte_count,
                       metadata_json, created_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    artifact_id,
                    run_id,
                    node_id,
                    attempt_id,
                    kind.value,
                    display_name,
                    storage_path,
                    digest,
                    len(content),
                    json.dumps(metadata or {}, sort_keys=True),
                    utc_text(),
                ),
            )
        return WorkflowArtifactRecord(artifact_id, digest, len(content))
