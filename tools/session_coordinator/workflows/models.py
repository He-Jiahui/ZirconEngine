from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime

from ..models import (
    WorkflowNodeKind,
    WorkflowNodeState,
    WorkflowState,
)


@dataclass(frozen=True, slots=True)
class WorkflowRunRecord:
    run_id: str
    session_id: str | None
    workflow_key: str
    plan_path: str | None
    topology_hash: str | None
    state: WorkflowState
    status_reason: str | None
    created_at: datetime
    updated_at: datetime
    completed_at: datetime | None


@dataclass(frozen=True, slots=True)
class WorkflowNodeRecord:
    node_id: str
    run_id: str
    node_key: str
    kind: WorkflowNodeKind
    title: str
    stage: str
    state: WorkflowNodeState
    owner_session_id: str | None
    status_reason: str | None
    attempt_count: int
    created_at: datetime
    updated_at: datetime


@dataclass(frozen=True, slots=True)
class WorkflowAttemptRecord:
    attempt_id: str
    node_id: str
    attempt_number: int
    state: WorkflowNodeState
    accepted: bool
    evidence: dict[str, object]
    started_at: datetime
    completed_at: datetime | None
