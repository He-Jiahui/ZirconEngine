from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.database import Database
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, SessionStatus
from tools.session_coordinator.ownership_matrix import OwnershipMatrixService
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class OwnershipMatrixTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = TemporaryDirectory()
        self.addCleanup(self.temporary_directory.cleanup)
        self.root = Path(self.temporary_directory.name)
        self.repo = init_repo(self.root / "repo")
        self.database = Database(self.root / "state" / "coordinator.sqlite3")
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.baselines = BaselineService(self.database, self.repo)
        self.leases = LeaseService(
            self.database,
            PathPolicy(self.repo),
            ttl_seconds=900,
            grace_seconds=120,
        )
        self.matrix = OwnershipMatrixService(
            self.database, self.baselines, self.leases
        )

    def _baseline_with_owned_change(self, owner: str = "owner") -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("value = 1\n", encoding="utf-8")
        self.baselines.accept(reason="matrix fixture baseline")
        self.sessions.register(session_id=owner)
        source.write_text("value = 2\n", encoding="utf-8")
        self.leases.acquire(owner, ["tools/owned.py"])
        self.baselines.attribute(owner, ["tools/owned.py"])

    def test_matrix_exposes_only_currently_attributed_live_leased_paths_as_candidates(self) -> None:
        self._baseline_with_owned_change()
        unowned = self.repo / "tools" / "unowned.py"
        unowned.write_text("value = 3\n", encoding="utf-8")

        matrix = self.matrix.build(prefix="tools")

        entries = {entry.path: entry for entry in matrix.entries}
        self.assertEqual("integration_ready", entries["tools/owned.py"].state)
        self.assertEqual("owner", entries["tools/owned.py"].owner_session_id)
        self.assertEqual((), entries["tools/owned.py"].blocking_reasons)
        self.assertEqual("unowned", entries["tools/unowned.py"].state)
        self.assertIn("attribution_missing", entries["tools/unowned.py"].blocking_reasons)
        self.assertEqual(("tools/owned.py",), matrix.candidates[0].paths)

    def test_stale_owner_or_missing_live_lease_cannot_form_an_integration_candidate(self) -> None:
        self._baseline_with_owned_change()
        self.sessions.set_status("owner", SessionStatus.STALE, reason="fixture stale")

        matrix = self.matrix.build(prefix="tools")

        entry = matrix.entries[0]
        self.assertEqual("unowned", entry.state)
        self.assertIn("owner_not_executable", entry.blocking_reasons)
        self.assertEqual((), matrix.candidates)

    def test_prefix_matrix_requires_an_initialized_baseline_with_a_structured_error(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            self.matrix.build(prefix="tools")

        self.assertEqual("baseline_not_initialized", rejected.exception.code)


if __name__ == "__main__":
    unittest.main()
