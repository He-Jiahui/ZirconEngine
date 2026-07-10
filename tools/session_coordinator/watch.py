from __future__ import annotations

from .baselines import BaselineService, WorkspaceChange


class WorkspaceWatcher:
    """Runs one deterministic workspace observation tick.

    The long-lived service schedules this method; keeping the scan itself
    synchronous makes restart and test behavior deterministic.
    """

    def __init__(self, baselines: BaselineService):
        self.baselines = baselines

    def scan_once(self) -> list[WorkspaceChange]:
        self.baselines.refresh_for_head_change()
        return self.baselines.scan()
