from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.audit import RolloutAuditService
from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.failures import FailureGraphService
from tools.session_coordinator.legacy import LegacyMigrationService
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.plans import PlanRepository
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class RolloutAuditTests(unittest.TestCase):
    def test_audit_is_deterministic_read_only_and_covers_both_plan_roots(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            repo = init_repo(root / "repo")
            formal = repo / "docs/plans/runtime/01-runtime.md"
            formal.parent.mkdir(parents=True)
            formal.write_text("# Runtime\n", encoding="utf-8")
            legacy_plan = repo / ".codex/plans/legacy.md"
            legacy_plan.parent.mkdir(parents=True)
            legacy_plan.write_text("# Legacy\n", encoding="utf-8")
            (repo / ".codex/sessions").mkdir()
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            database = Database(config.database_path)
            migrate(database)
            sessions = SessionService(database, repo)
            baselines = BaselineService(database, repo)
            baselines.initialize()
            legacy = LegacyMigrationService(database, repo, sessions)
            service = RolloutAuditService(
                database,
                repo,
                sessions=sessions,
                baselines=baselines,
                plans=PlanRepository(repo),
                failures=FailureGraphService(database, repo),
                legacy=legacy,
                target_roots=(root / "drive/targets/zircon-engine",),
            )

            first = service.audit_all().to_dict()
            second = service.audit_all().to_dict()

            self.assertEqual(first, second)
            self.assertEqual("main", first["branch"])
            self.assertEqual(1, first["formal_plan_count"])
            self.assertEqual(1, first["legacy_plan_count"])
            self.assertEqual([], first["invalid_session_statuses"])
            self.assertEqual([], first["unsafe_cleanup_candidates"])

    def test_repo_local_legacy_cargo_target_is_diagnostic_only(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            repo = init_repo(root / "repo")
            target = repo / "target/codex-shared-old"
            target.mkdir(parents=True)
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            database = Database(config.database_path)
            migrate(database)
            sessions = SessionService(database, repo)
            baselines = BaselineService(database, repo)
            baselines.initialize()
            legacy = LegacyMigrationService(database, repo, sessions)
            service = RolloutAuditService(
                database,
                repo,
                sessions=sessions,
                baselines=baselines,
                plans=PlanRepository(repo),
                failures=FailureGraphService(database, repo),
                legacy=legacy,
                target_roots=(),
            )

            audit = service.audit_all()

            self.assertIn("target/codex-shared-old", audit.legacy_cargo_targets)
            self.assertTrue(target.exists())

    def test_codex_legacy_target_reports_lane_root_not_every_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            repo = init_repo(root / "repo")
            lane = repo / ".codex/targets/old-lane"
            artifact = lane / "debug/incremental/object.o"
            artifact.parent.mkdir(parents=True)
            artifact.write_bytes(b"artifact")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            database = Database(config.database_path)
            migrate(database)
            sessions = SessionService(database, repo)
            legacy = LegacyMigrationService(database, repo, sessions)

            diagnostics = legacy.legacy_cargo_diagnostics()

            self.assertEqual((".codex/targets/old-lane",), diagnostics)

    def test_audit_uses_exact_direct_lane_policy_for_recorded_targets(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            repo = init_repo(root / "repo")
            managed = root / "drive/targets/zircon-engine"
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            database = Database(config.database_path)
            migrate(database)
            sessions = SessionService(database, repo)
            baselines = BaselineService(database, repo)
            baselines.initialize()
            legacy = LegacyMigrationService(database, repo, sessions)
            service = RolloutAuditService(
                database,
                repo,
                sessions=sessions,
                baselines=baselines,
                plans=PlanRepository(repo),
                failures=FailureGraphService(database, repo),
                legacy=legacy,
                target_roots=(managed,),
            )

            self.assertTrue(service._target_is_managed(managed / "lanes/direct"))
            self.assertFalse(
                service._target_is_managed(managed / "lanes/direct/nested")
            )
            self.assertFalse(service._target_is_managed(managed / "verify/job"))
            self.assertFalse(service._target_is_managed(managed))


if __name__ == "__main__":
    unittest.main()
