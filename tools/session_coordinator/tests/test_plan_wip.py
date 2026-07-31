from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, SessionStatus
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class PlanWipAdmissionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = TemporaryDirectory()
        self.addCleanup(self.temporary_directory.cleanup)
        self.root = Path(self.temporary_directory.name)
        self.repo = init_repo(self.root / "repo")
        self.plan_path = "docs/plans/tooling/01-tooling.md"
        plan = self.repo / self.plan_path
        plan.parent.mkdir(parents=True)
        plan.write_text("# Tooling Plan\n", encoding="utf-8")
        self.database = Database(self.root / "state" / "coordinator.sqlite3")
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)

    def test_second_primary_for_the_same_numbered_plan_is_rejected(self) -> None:
        primary = self.sessions.register(
            session_id="primary-a",
            plan_path=self.plan_path,
            write_scope=["tools/owned.py"],
        )

        with self.assertRaises(CoordinatorError) as rejected:
            self.sessions.register(
                session_id="primary-b",
                plan_path=self.plan_path,
                write_scope=["tools/next.py"],
            )

        self.assertEqual("plan_wip_limit_reached", rejected.exception.code)
        self.assertEqual("primary-a", rejected.exception.details["primarySessionId"])
        self.assertEqual("docs/plans/tooling/01", primary.plan_family_key)

    def test_idempotent_registration_and_failure_repair_reuse_the_existing_primary_slot(self) -> None:
        self.sessions.register(
            session_id="primary-a",
            plan_path=self.plan_path,
            write_scope=["tools/owned.py"],
        )
        resumed = self.sessions.register(
            session_id="primary-a",
            plan_path=self.plan_path,
            write_scope=["tools/owned.py"],
            requested_status=SessionStatus.RESOLVING_FAILURE,
        )

        self.assertEqual(SessionStatus.RESOLVING_FAILURE, resumed.status)
        with self.assertRaises(CoordinatorError) as rejected:
            self.sessions.register(
                session_id="repair-micro-session",
                plan_path=self.plan_path,
                write_scope=["tools/repair.py"],
            )
        self.assertEqual("plan_wip_limit_reached", rejected.exception.code)

    def test_one_reviewer_must_be_linked_to_its_primary_and_have_no_write_scope(self) -> None:
        self.sessions.register(
            session_id="primary-a",
            plan_path=self.plan_path,
            write_scope=["tools/owned.py"],
        )
        reviewer = self.sessions.register(
            session_id="reviewer-a",
            plan_path=self.plan_path,
            session_role="reviewer",
            parent_session_id="primary-a",
        )

        self.assertEqual("reviewer", reviewer.session_role)
        self.assertEqual("primary-a", reviewer.parent_session_id)
        with self.assertRaises(CoordinatorError) as second_reviewer:
            self.sessions.register(
                session_id="reviewer-b",
                plan_path=self.plan_path,
                session_role="reviewer",
                parent_session_id="primary-a",
            )
        self.assertEqual("plan_wip_reviewer_limit_reached", second_reviewer.exception.code)
        with self.assertRaises(CoordinatorError) as writer_reviewer:
            self.sessions.register(
                session_id="writer-reviewer",
                plan_path=self.plan_path,
                session_role="reviewer",
                parent_session_id="primary-a",
                write_scope=["tools/forbidden.py"],
            )
        self.assertEqual("plan_wip_reviewer_write_scope_forbidden", writer_reviewer.exception.code)

    def test_non_numbered_legacy_plan_remains_compatible_with_multiple_sessions(self) -> None:
        legacy_plan = "docs/superpowers/plans/legacy.md"
        self.sessions.register(session_id="legacy-a", plan_path=legacy_plan)
        second = self.sessions.register(session_id="legacy-b", plan_path=legacy_plan)

        self.assertIsNone(second.plan_family_key)


if __name__ == "__main__":
    unittest.main()
