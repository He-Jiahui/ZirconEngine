from __future__ import annotations

import importlib.util
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[5]
sys.path.insert(0, str(REPO_ROOT))

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.models import SessionStatus
from tools.session_coordinator.server import RunningCoordinator
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


def load_reader():
    path = Path(__file__).with_name("read-closeout-evidence.py")
    spec = importlib.util.spec_from_file_location("read_closeout_evidence", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ReadCloseoutEvidenceTests(unittest.TestCase):
    def test_reads_current_attributed_dirty_scope_without_mutating_database(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = init_repo(Path(temporary) / "repo")
            plan = repo / "docs/plans/feature/02-feature.md"
            source = repo / "src/feature.py"
            plan.parent.mkdir(parents=True, exist_ok=True)
            source.parent.mkdir(parents=True, exist_ok=True)
            plan.write_text("# plan\n", encoding="utf-8")
            source.write_text("baseline\n", encoding="utf-8")
            subprocess.run(["git", "add", "docs", "src"], cwd=repo, check=True)
            subprocess.run(["git", "commit", "-q", "-m", "test: fixture"], cwd=repo, check=True)

            config = CoordinatorConfig.for_repo(repo)
            database = Database(config.database_path)
            migrate(database)
            baselines = BaselineService(database, repo)
            sessions = SessionService(database, repo)
            baselines.initialize()
            sessions.register(session_id="session-a", plan_path="docs/plans/feature/02-feature.md")
            sessions.set_status("session-a", SessionStatus.ACTIVE)
            leases = LeaseService(
                database,
                PathPolicy(repo),
                ttl_seconds=config.lease_ttl_seconds,
                grace_seconds=config.lease_grace_seconds,
            )
            leases.acquire("session-a", ["src/feature.py"])
            source.write_text("milestone\n", encoding="utf-8")
            baselines.attribute("session-a", ["src/feature.py"])
            subprocess.run(["git", "add", "src/feature.py"], cwd=repo, check=True)
            with RunningCoordinator.start(config):
                database_files = [
                    path
                    for path in (
                        config.database_path,
                        Path(f"{config.database_path}-wal"),
                        Path(f"{config.database_path}-shm"),
                    )
                    if path.exists()
                ]
                before = {path: path.read_bytes() for path in database_files}
                evidence = load_reader().read_evidence(repo, "session-a")
                self.assertEqual(
                    before, {path: path.read_bytes() for path in database_files}
                )

            self.assertEqual("main", evidence["branch"])
            self.assertEqual("read_write", evidence["service_mode"])
            self.assertEqual("active", evidence["session_status"])
            self.assertEqual("docs/plans/feature/02-feature.md", evidence["plan_path"])
            self.assertEqual(["src/feature.py"], evidence["owned_dirty_paths"])
            self.assertEqual(
                evidence["attributed_hashes"], evidence["staged_hashes"]
            )
            self.assertEqual(0, evidence["open_failure_count"])
            self.assertEqual(["src/feature.py"], evidence["leased_paths"])
            self.assertEqual([], evidence["failure_diagnostics"])


if __name__ == "__main__":
    unittest.main()
