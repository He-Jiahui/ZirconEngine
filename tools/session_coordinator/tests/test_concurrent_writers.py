from __future__ import annotations

import tempfile
import threading
import unittest
from pathlib import Path

from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class ConcurrentWriterTests(unittest.TestCase):
    def test_twenty_races_always_have_exactly_one_owner(self) -> None:
        for iteration in range(20):
            with self.subTest(iteration=iteration), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                repo = init_repo(root / "repo")
                config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
                database = Database(config.database_path)
                migrate(database)
                sessions = SessionService(database, repo)
                sessions.register(session_id="session-a")
                sessions.register(session_id="session-b")
                barrier = threading.Barrier(2)
                outcomes: list[bool] = []
                outcome_lock = threading.Lock()

                def claim(session_id: str) -> None:
                    service = LeaseService(database, PathPolicy(repo), ttl_seconds=300, grace_seconds=120)
                    barrier.wait(timeout=5)
                    result = service.acquire(session_id, ["README.md"])
                    with outcome_lock:
                        outcomes.append(result.acquired)

                threads = [
                    threading.Thread(target=claim, args=("session-a",)),
                    threading.Thread(target=claim, args=("session-b",)),
                ]
                for thread in threads:
                    thread.start()
                for thread in threads:
                    thread.join(timeout=5)

                self.assertEqual(2, len(outcomes))
                self.assertEqual(1, sum(outcomes))


if __name__ == "__main__":
    unittest.main()
