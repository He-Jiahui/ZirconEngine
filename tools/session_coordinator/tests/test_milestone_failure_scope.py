from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.failures import FailureGraphService
from tools.session_coordinator.git_finalize import GitFinalizeService
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, SessionStatus
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.failure_fixture import FailureGraphFixture
from tools.session_coordinator.tests.helpers import init_repo


class MilestoneFailureScopeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        self.fixture = FailureGraphFixture(self.repo)
        self.current_plan = self.fixture.add_plan(
            "docs/plans/runtime/01-feature.md"
        )
        self.other_plan = self.fixture.add_plan(
            "docs/plans/runtime/02-fixer.md"
        )
        subprocess.run(
            ["git", "add", "--", "docs/plans/runtime"],
            cwd=self.repo,
            check=True,
        )
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add failure scope plans"],
            cwd=self.repo,
            check=True,
        )

        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.sessions.register(
            session_id="session-a",
            plan_path=self.current_plan.path.relative_to(self.repo).as_posix(),
        )
        self.sessions.set_status("session-a", SessionStatus.ACTIVE)
        self.baselines = BaselineService(self.database, self.repo)
        self.baselines.initialize()
        self.leases = LeaseService(
            self.database,
            PathPolicy(self.repo),
            ttl_seconds=config.lease_ttl_seconds,
            grace_seconds=config.lease_grace_seconds,
        )
        self.failures = FailureGraphService(self.database, self.repo)
        self.finalize = GitFinalizeService(
            self.database,
            self.repo,
            self.baselines,
            self.sessions,
            failures=self.failures,
        )
        self.failure_counter = 0

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def _add_failure(
        self,
        *,
        origin_workflow_node: str | None,
        current_is_fixer: bool = False,
    ) -> Path:
        self.failure_counter += 1
        origin = self.other_plan if current_is_fixer else self.current_plan
        fixing = self.current_plan if current_is_fixer else self.other_plan
        artifact = self.fixture.add_handoff(
            origin,
            fixing,
            f"scope-{self.failure_counter}",
        )
        if origin_workflow_node is not None:
            artifact.write_text(
                artifact.read_text(encoding="utf-8").replace(
                    "summary_slug:",
                    f"origin_workflow_node: {origin_workflow_node}\nsummary_slug:",
                ),
                encoding="utf-8",
            )
        return artifact

    def _prepare_owned_change(self, name: str) -> str:
        path = f"src/{name}.py"
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(f"value = {name!r}\n", encoding="utf-8")
        self.assertTrue(self.leases.acquire("session-a", [path]).acquired)
        self.baselines.attribute("session-a", [path])
        return path

    def _commit(self, path: str, node_keys: tuple[str, ...]):
        return self._commit_paths((path,), node_keys)

    def _commit_paths(self, paths: tuple[str, ...], node_keys: tuple[str, ...]):
        return self.finalize.commit_milestone(
            "session-a",
            paths=list(paths),
            message="fix(runtime): enforce milestone failure scope",
            failure_workflow_node_keys=node_keys,
        )

    def _head(self) -> str:
        return subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()

    def _staged(self) -> str:
        return subprocess.run(
            ["git", "diff", "--cached", "--name-only"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout

    def test_unrelated_origin_slice_failure_does_not_block_milestone_commit(
        self,
    ) -> None:
        self._add_failure(origin_workflow_node="M1.2")
        path = self._prepare_owned_change("slice_one")

        result = self._commit(path, ("M1.1",))

        self.assertEqual(result.commit_sha, self._head())

    def test_milestone_commit_rejects_empty_failure_scope(self) -> None:
        path = self._prepare_owned_change("empty_scope")

        with self.assertRaises(CoordinatorError) as rejected:
            self._commit(path, ())

        self.assertEqual(
            "milestone_failure_scope_invalid",
            rejected.exception.code,
        )

    def test_milestone_commit_rejects_invalid_failure_scope(self) -> None:
        path = self._prepare_owned_change("invalid_scope")

        with self.assertRaises(CoordinatorError) as rejected:
            self._commit(path, ("M1.0",))

        self.assertEqual(
            "milestone_failure_scope_invalid",
            rejected.exception.code,
        )
        self.assertEqual("", self._staged())

    def test_own_origin_slice_failure_blocks_before_staging(self) -> None:
        failure = self._add_failure(origin_workflow_node="M1.1")
        path = self._prepare_owned_change("own_slice")
        head_before = self._head()

        with self.assertRaises(CoordinatorError) as rejected:
            self._commit(path, ("M1.1",))

        self.assertEqual("finalize_open_failure", rejected.exception.code)
        self.assertEqual([failure.relative_to(self.repo).as_posix()], rejected.exception.details["paths"])
        self.assertEqual(head_before, self._head())
        self.assertEqual("", self._staged())

    def test_parent_scope_aggregates_child_slice_failures(self) -> None:
        self._add_failure(origin_workflow_node="M1.2")
        path = self._prepare_owned_change("parent")

        with self.assertRaises(CoordinatorError) as rejected:
            self._commit(path, ("M1", "M1.1", "M1.2"))

        self.assertEqual("finalize_open_failure", rejected.exception.code)

    def test_legacy_failure_remains_plan_wide_for_milestone_commit(self) -> None:
        self._add_failure(origin_workflow_node=None)
        path = self._prepare_owned_change("legacy")

        with self.assertRaises(CoordinatorError) as rejected:
            self._commit(path, ("M9.9",))

        self.assertEqual("finalize_open_failure", rejected.exception.code)

    def test_fixing_plan_failure_remains_priority_for_every_milestone(self) -> None:
        self._add_failure(
            origin_workflow_node="M8.2",
            current_is_fixer=True,
        )
        path = self._prepare_owned_change("fixing")

        with self.assertRaises(CoordinatorError) as rejected:
            self._commit(path, ("M1.1",))

        self.assertEqual("finalize_open_failure", rejected.exception.code)

    def test_fixed_return_manifest_excludes_unrelated_fixer_failures(self) -> None:
        fixed = self.fixture.add_handoff(
            self.other_plan,
            self.current_plan,
            "completed-return",
            kind="fixed",
        )
        unrelated = self.fixture.add_handoff(
            self.other_plan,
            self.current_plan,
            "still-open",
        )
        fixed_path = fixed.relative_to(self.repo).as_posix()
        path = self._prepare_owned_change("fixed_return")
        self.assertTrue(self.leases.acquire("session-a", [fixed_path]).acquired)
        self.baselines.attribute("session-a", [fixed_path])

        result = self._commit_paths((path, fixed_path), ("M1.1",))

        self.assertEqual(result.commit_sha, self._head())
        self.assertEqual(
            [unrelated.relative_to(self.repo).as_posix()],
            [node.artifact_path for node in self.failures.open_for_plan(self.current_plan.path)],
        )

    def test_fixed_return_manifest_keeps_applicable_origin_failure_blocking(self) -> None:
        fixed = self.fixture.add_handoff(
            self.other_plan,
            self.current_plan,
            "completed-return",
            kind="fixed",
        )
        applicable = self._add_failure(origin_workflow_node="M1.1")
        fixed_path = fixed.relative_to(self.repo).as_posix()
        path = self._prepare_owned_change("fixed_return_blocked")
        self.assertTrue(self.leases.acquire("session-a", [fixed_path]).acquired)
        self.baselines.attribute("session-a", [fixed_path])

        with self.assertRaises(CoordinatorError) as rejected:
            self._commit_paths((path, fixed_path), ("M1.1",))

        self.assertEqual("finalize_open_failure", rejected.exception.code)
        self.assertEqual(
            [applicable.relative_to(self.repo).as_posix()],
            rejected.exception.details["paths"],
        )

    def test_invalid_origin_node_is_diagnostic_and_falls_back_plan_wide(self) -> None:
        artifact = self._add_failure(origin_workflow_node="M1.0")

        audit = self.failures.import_repository()
        open_failures = self.failures.open_related_to_workflow_nodes(
            self.current_plan.path,
            ("M9.9",),
        )

        imported = next(
            node
            for node in audit.nodes
            if node.artifact_path == artifact.relative_to(self.repo).as_posix()
        )
        self.assertIsNone(imported.origin_workflow_node)
        self.assertIn(
            "invalid_origin_workflow_node",
            {diagnostic.code for diagnostic in audit.diagnostics},
        )
        self.assertEqual(
            [artifact.relative_to(self.repo).as_posix()],
            [node.artifact_path for node in open_failures],
        )

    def test_second_locked_guard_rejects_new_own_failure_before_update_ref(self) -> None:
        path = self._prepare_owned_change("late_own")
        head_before = self._head()
        original = self.finalize._require_milestone_failure_acceptance
        calls = 0

        def inject_after_first_guard(session, node_keys, manifest_paths):
            nonlocal calls
            calls += 1
            original(session, node_keys, manifest_paths)
            if calls == 1:
                self._add_failure(origin_workflow_node="M1.1")

        with mock.patch.object(
            self.finalize,
            "_require_milestone_failure_acceptance",
            side_effect=inject_after_first_guard,
        ):
            with self.assertRaises(CoordinatorError) as rejected:
                self._commit(path, ("M1.1",))

        self.assertEqual("finalize_open_failure", rejected.exception.code)
        self.assertEqual(2, calls)
        self.assertEqual(head_before, self._head())

    def test_second_locked_guard_keeps_unrelated_failure_out_of_scope(self) -> None:
        path = self._prepare_owned_change("late_sibling")
        original = self.finalize._require_milestone_failure_acceptance
        calls = 0

        def inject_after_first_guard(session, node_keys, manifest_paths):
            nonlocal calls
            calls += 1
            original(session, node_keys, manifest_paths)
            if calls == 1:
                self._add_failure(origin_workflow_node="M1.2")

        with mock.patch.object(
            self.finalize,
            "_require_milestone_failure_acceptance",
            side_effect=inject_after_first_guard,
        ):
            result = self._commit(path, ("M1.1",))

        self.assertEqual(2, calls)
        self.assertEqual(result.commit_sha, self._head())

    def test_explicit_finalize_stays_plan_wide(self) -> None:
        failure = self._add_failure(origin_workflow_node="M1.2")
        path = self._prepare_owned_change("explicit")
        self.sessions.set_status("session-a", SessionStatus.COMPLETED)

        with self.assertRaises(CoordinatorError) as rejected:
            self.finalize.preview(
                "session-a",
                paths=[path],
                message="fix(runtime): retain explicit plan-wide failure gate",
            )

        self.assertEqual("finalize_open_failure", rejected.exception.code)
        self.assertEqual([failure.relative_to(self.repo).as_posix()], rejected.exception.details["paths"])


if __name__ == "__main__":
    unittest.main()
