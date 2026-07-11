from __future__ import annotations

from .baselines import (
    BaselineService,
    PreparedWorkspaceScan,
    WorkspaceChange,
    WorkspaceScanResult,
)


class WorkspaceWatcher:
    """Runs one deterministic workspace observation tick.

    The long-lived service schedules this method; keeping the scan itself
    synchronous makes restart and test behavior deterministic.
    """

    def __init__(self, baselines: BaselineService):
        self.baselines = baselines

    def scan_once(self) -> list[WorkspaceChange]:
        return list(self.apply_scan(self.prepare_scan()).changes)

    def prepare_scan(self) -> PreparedWorkspaceScan:
        return self.baselines.prepare_scan()

    def apply_scan(self, observation: PreparedWorkspaceScan) -> WorkspaceScanResult:
        return self.baselines.apply_scan(observation)
