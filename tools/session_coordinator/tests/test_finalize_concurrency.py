from __future__ import annotations

import tempfile
import threading
import unittest
from pathlib import Path

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.git_finalize import GitFinalizeService
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class FinalizeConcurrencyTests(unittest.TestCase):
    def test_only_one_finalize_owner_can_hold_git_mutex(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            database = Database(config.database_path)
            migrate(database)
            sessions = SessionService(database, repo)
            sessions.register(session_id="session-a")
            sessions.register(session_id="session-b")
            service = GitFinalizeService(
                database, repo, BaselineService(database, repo), sessions
            )
            start = threading.Barrier(2)
            acquired = threading.Event()
            release = threading.Event()
            outcomes: list[str] = []

            def contender(session_id: str) -> None:
                start.wait()
                try:
                    with service.git_mutex(session_id):
                        outcomes.append("acquired")
                        acquired.set()
                        release.wait(timeout=2)
                except CoordinatorError as error:
                    outcomes.append(error.code)
                    acquired.wait(timeout=2)
                    release.set()

            threads = [
                threading.Thread(target=contender, args=("session-a",)),
                threading.Thread(target=contender, args=("session-b",)),
            ]
            for thread in threads:
                thread.start()
            for thread in threads:
                thread.join(timeout=5)

            self.assertEqual(1, outcomes.count("acquired"))
            self.assertEqual(1, outcomes.count("git_mutex_occupied"))
            self.assertTrue(all(not thread.is_alive() for thread in threads))


if __name__ == "__main__":
    unittest.main()
