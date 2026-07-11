from __future__ import annotations

from ..models import WorkflowNodeState
from .models import WorkflowAttemptRecord
from .store import WorkflowStore


class WorkflowAttemptService:
    """Small domain facade for append-only workflow attempt history."""

    def __init__(self, store: WorkflowStore):
        self.store = store

    def record(
        self,
        node_id: str,
        state: WorkflowNodeState,
        evidence: dict[str, object],
        *,
        accepted: bool = True,
    ) -> WorkflowAttemptRecord:
        return self.store.append_attempt(
            node_id, state, evidence, accepted=accepted
        )

    def current(self, run_id: str) -> dict[str, WorkflowAttemptRecord]:
        return self.store.current_attempts(run_id)

    def history(self, node_id: str) -> tuple[WorkflowAttemptRecord, ...]:
        return self.store.attempt_history(node_id)
