from __future__ import annotations

import inspect
import unittest

from tools.session_coordinator.codex_sync.store import CodexSessionStore


class CodexReconcileProjectionCachePerformanceContractTests(unittest.TestCase):
    def test_reconcile_loads_existing_rows_and_bindings_once(self) -> None:
        source = inspect.getsource(CodexSessionStore.reconcile)

        self.assertIn("existing_by_thread_id", source)
        self.assertIn("bound_session_ids", source)
        self.assertNotIn("SELECT * FROM codex_sessions WHERE thread_id=?", source)
        self.assertNotIn(
            "SELECT session_id FROM sessions WHERE session_id=?", source
        )
        self.assertNotIn(
            "SELECT thread_id, missing_scan_count FROM codex_sessions", source
        )


if __name__ == "__main__":
    unittest.main()
