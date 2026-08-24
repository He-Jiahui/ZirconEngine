from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest import mock

from tools.session_coordinator import migrations
from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, SessionStatus
from tools.session_coordinator.ownership_transfers import OwnershipTransferService
from tools.session_coordinator.server import CoordinatorApplication
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.failure_fixture import FailureGraphFixture
from tools.session_coordinator.tests.helpers import init_repo


class OwnershipTransferTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = TemporaryDirectory()
        self.addCleanup(self.temporary_directory.cleanup)
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        self.database = Database(root / "state" / "coordinator.sqlite3")
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.baselines = BaselineService(self.database, self.repo)
        self.leases = LeaseService(
            self.database,
            PathPolicy(self.repo),
            ttl_seconds=900,
            grace_seconds=120,
        )
        self.service = OwnershipTransferService(
            self.database, self.repo, self.leases, self.sessions
        )

    def _abandoned_change(self) -> Path:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("value = 1\n", encoding="utf-8")
        self.baselines.accept(reason="ownership transfer fixture")
        self.sessions.register(session_id="source")
        self.sessions.register(session_id="target")
        source.write_text("value = 2\n", encoding="utf-8")
        self.assertTrue(self.leases.acquire("source", ["tools/owned.py"]).acquired)
        self.baselines.attribute("source", ["tools/owned.py"])
        self.leases.release("source")
        self.sessions.set_status("source", SessionStatus.STALE, reason="fixture stale")
        return source

    def _archived_clean_path(self) -> Path:
        source = self.repo / "docs" / "accepted.md"
        source.parent.mkdir(parents=True)
        source.write_text("accepted\n", encoding="utf-8")
        self.baselines.accept(reason="archived clean ownership fixture")
        self.sessions.register(session_id="source")
        self.sessions.register(session_id="target")
        self.sessions.set_status("source", SessionStatus.ACTIVE)
        self.assertTrue(self.leases.acquire("source", ["docs/accepted.md"]).acquired)
        self.baselines.attribute("source", ["docs/accepted.md"])
        self.leases.release("source")
        self.sessions.set_status("source", SessionStatus.COMPLETED)
        self.sessions.set_status("source", SessionStatus.ARCHIVED)
        return source

    def test_apply_moves_an_abandoned_exact_path_scope_lease_and_attribution(self) -> None:
        self._abandoned_change()

        preview = self.service.preview(
            target_session_id="target", paths=("tools/owned.py",)
        )
        result = self.service.apply(preview.fingerprint, actor="fixture")
        replay = self.service.apply(preview.fingerprint, actor="fixture")

        self.assertTrue(preview.paths[0].eligible, preview.paths[0].blocking_reasons)
        self.assertFalse(result.already_applied)
        self.assertTrue(replay.already_applied)
        self.assertEqual(("tools/owned.py",), result.paths)
        self.assertIn("tools/owned.py", self.sessions.get("target").write_scope)
        self.assertEqual(["tools/owned.py"], self.leases.owned_paths("target"))
        with self.database.connect() as connection:
            attribution = connection.execute(
                "SELECT session_id FROM attributions WHERE path_key='tools/owned.py'"
            ).fetchone()
            self.assertEqual("target", attribution["session_id"])
            self.assertEqual(
                1,
                connection.execute(
                    "SELECT COUNT(*) FROM ownership_transfers WHERE fingerprint=?",
                    (preview.fingerprint,),
                ).fetchone()[0],
            )

    def test_apply_moves_an_archived_exact_clean_path(self) -> None:
        self._archived_clean_path()

        preview = self.service.preview(
            target_session_id="target", paths=("docs/accepted.md",)
        )
        result = self.service.apply(preview.fingerprint, actor="fixture")

        candidate = preview.paths[0]
        self.assertTrue(candidate.eligible, candidate.blocking_reasons)
        self.assertEqual("archived", candidate.source_status)
        self.assertEqual(candidate.current_hash, candidate.baseline_hash)
        self.assertEqual(candidate.current_hash, candidate.source_content_hash)
        self.assertFalse(result.already_applied)
        self.assertIn("docs/accepted.md", self.sessions.get("target").write_scope)
        self.assertEqual(["docs/accepted.md"], self.leases.owned_paths("target"))
        with self.database.connect() as connection:
            transfer = connection.execute(
                "SELECT path_state, content_hash FROM ownership_transfers WHERE fingerprint=?",
                (preview.fingerprint,),
            ).fetchone()
        self.assertEqual(("existing", candidate.current_hash), tuple(transfer))

    def test_preview_refuses_an_unowned_clean_baseline_path(self) -> None:
        source = self.repo / "docs" / "unowned.md"
        source.parent.mkdir(parents=True)
        source.write_text("unowned\n", encoding="utf-8")
        self.baselines.accept(reason="unowned clean ownership fixture")
        self.sessions.register(session_id="target")

        preview = self.service.preview(
            target_session_id="target", paths=("docs/unowned.md",)
        )

        self.assertFalse(preview.paths[0].eligible)
        self.assertIn("path_matches_baseline", preview.paths[0].blocking_reasons)
        self.assertEqual([], self.leases.owned_paths("target"))

    def test_preview_refuses_an_archived_clean_path_with_stale_attribution(self) -> None:
        self._archived_clean_path()
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE attributions SET content_hash='stale-archived-evidence'
                WHERE path_key='docs/accepted.md'
                """
            )

        preview = self.service.preview(
            target_session_id="target", paths=("docs/accepted.md",)
        )

        self.assertFalse(preview.paths[0].eligible)
        self.assertIn("path_matches_baseline", preview.paths[0].blocking_reasons)
        self.assertEqual([], self.leases.owned_paths("target"))

    def test_preview_refuses_an_executable_source_owner_or_live_foreign_lease(self) -> None:
        source = self._abandoned_change()
        self.sessions.set_status("source", SessionStatus.ACTIVE, reason="fixture resumed")
        self.assertTrue(self.leases.acquire("source", ["tools/owned.py"]).acquired)

        preview = self.service.preview(
            target_session_id="target", paths=("tools/owned.py",)
        )

        self.assertFalse(preview.paths[0].eligible)
        self.assertIn("source_owner_executable", preview.paths[0].blocking_reasons)
        self.assertIn("live_foreign_lease", preview.paths[0].blocking_reasons)
        self.assertEqual("value = 2\n", source.read_text(encoding="utf-8"))
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.apply(preview.fingerprint, actor="fixture")
        self.assertEqual("ownership_transfer_ineligible_paths", rejected.exception.code)

    def test_apply_rejects_when_the_reviewed_blob_changes(self) -> None:
        source = self._abandoned_change()
        preview = self.service.preview(
            target_session_id="target", paths=("tools/owned.py",)
        )
        source.write_text("value = 3\n", encoding="utf-8")

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.apply(preview.fingerprint, actor="fixture")

        self.assertEqual("ownership_transfer_preview_stale", rejected.exception.code)
        self.assertEqual([], self.leases.owned_paths("target"))
        self.assertNotIn("tools/owned.py", self.sessions.get("target").write_scope)

    def test_apply_rejects_when_source_attribution_changes_after_preview(self) -> None:
        self._abandoned_change()
        preview = self.service.preview(
            target_session_id="target", paths=("tools/owned.py",)
        )
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE attributions SET content_hash='different-source-evidence'
                WHERE path_key='tools/owned.py'
                """
            )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.apply(preview.fingerprint, actor="fixture")

        self.assertEqual("ownership_transfer_preview_stale", rejected.exception.code)
        self.assertEqual([], self.leases.owned_paths("target"))

    def test_apply_reserves_an_unowned_missing_future_path(self) -> None:
        self.baselines.accept(reason="future path reservation fixture")
        self.sessions.register(session_id="target")

        preview = self.service.preview(
            target_session_id="target", paths=("docs/generated.md",)
        )
        result = self.service.apply(preview.fingerprint, actor="fixture")
        replay = self.service.apply(preview.fingerprint, actor="fixture")

        self.assertTrue(preview.paths[0].eligible, preview.paths[0].blocking_reasons)
        self.assertEqual("future", preview.paths[0].path_state)
        self.assertFalse(result.already_applied)
        self.assertTrue(replay.already_applied)
        self.assertIn("docs/generated.md", self.sessions.get("target").write_scope)
        self.assertEqual(["docs/generated.md"], self.leases.owned_paths("target"))
        with self.database.connect() as connection:
            attribution = connection.execute(
                "SELECT session_id, content_hash FROM attributions WHERE path_key=?",
                ("docs/generated.md",),
            ).fetchone()
            transfer = connection.execute(
                "SELECT path_state, content_hash FROM ownership_transfers WHERE fingerprint=?",
                (preview.fingerprint,),
            ).fetchone()
        self.assertEqual(("target", None), tuple(attribution))
        self.assertEqual(("future", None), tuple(transfer))

    def test_apply_rejects_when_future_path_materializes_after_preview(self) -> None:
        self.baselines.accept(reason="future path materialization fixture")
        self.sessions.register(session_id="target")
        preview = self.service.preview(
            target_session_id="target", paths=("docs/generated.md",)
        )
        generated = self.repo / "docs" / "generated.md"
        generated.parent.mkdir(parents=True)
        generated.write_text("foreign bytes\n", encoding="utf-8")

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.apply(preview.fingerprint, actor="fixture")

        self.assertTrue(preview.paths[0].eligible, preview.paths[0].blocking_reasons)
        self.assertEqual("ownership_transfer_preview_stale", rejected.exception.code)
        self.assertEqual([], self.leases.owned_paths("target"))
        self.assertNotIn("docs/generated.md", self.sessions.get("target").write_scope)

    def test_preview_refuses_a_missing_future_path_under_a_foreign_lease(self) -> None:
        self.baselines.accept(reason="future path lease fixture")
        self.sessions.register(session_id="source")
        self.sessions.register(session_id="target")
        self.assertTrue(self.leases.acquire("source", ["docs"]).acquired)

        preview = self.service.preview(
            target_session_id="target", paths=("docs/generated.md",)
        )

        self.assertFalse(preview.paths[0].eligible)
        self.assertIn("live_foreign_lease", preview.paths[0].blocking_reasons)

    def test_local_failure_return_uses_audited_future_path_reservations(self) -> None:
        root = Path(self.temporary_directory.name) / "failure-return"
        repo = init_repo(root / "repo")
        fixture = FailureGraphFixture(repo)
        plan = fixture.add_plan("docs/plans/tooling/01-tooling.md")
        failure = fixture.add_handoff(plan, plan, "future-return")
        failure.write_text(
            failure.read_text(encoding="utf-8").replace(
                "summary_slug:",
                "failure_scope: local\n"
                "plan_link_mode: child_record_only\n"
                "summary_slug:",
            ),
            encoding="utf-8",
        )
        application = CoordinatorApplication(
            CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
        )
        application.supervision.mark_healthy()
        application.baselines.initialize()
        node = application.failures.import_repository().nodes[0]
        plan_path = plan.path.relative_to(repo).as_posix()
        source_path = failure.relative_to(repo).as_posix()
        destination_path = (
            plan.child / "fixed-2026-07-16-future-return.md"
        ).relative_to(repo).as_posix()
        receipt_path = (
            plan.child / "2026-07-16-future-return-return.md"
        ).relative_to(repo).as_posix()
        application.sessions.register(
            session_id="owner",
            plan_path=plan_path,
            write_scope=[source_path],
        )
        application.sessions.set_status("owner", SessionStatus.ACTIVE)
        application.sessions.set_status("owner", SessionStatus.RESOLVING_FAILURE)
        self.assertTrue(application.leases.acquire("owner", [source_path]).acquired)
        preview = application.ownership_transfers.preview(
            target_session_id="owner",
            paths=(destination_path, receipt_path),
        )
        application.ownership_transfers.apply(preview.fingerprint, actor="fixture")

        result = application.command(
            "failure.return",
            {
                "session_id": "owner",
                "lifecycle_key": node.lifecycle_key,
                "resolved_at": "2026-07-16",
                "root_cause": "generated paths were absent from immutable scope",
                "architecture_fix": "reserve exact future paths through audited transfer",
                "validation": "future-path and return regressions passed",
                "return_summary": "local lifecycle closeout may continue",
            },
        )

        self.assertEqual(destination_path, result["fixed_artifact"])
        self.assertFalse(failure.exists())
        self.assertTrue((repo / destination_path).is_file())
        self.assertTrue((repo / receipt_path).is_file())
        session = application.sessions.get("owner")
        self.assertIn(destination_path, session.write_scope)
        self.assertIn(receipt_path, session.write_scope)

    def test_schema_51_upgrade_preserves_sessions_and_installs_transfer_audit(self) -> None:
        upgrade_database = Database(
            Path(self.temporary_directory.name) / "upgrade" / "coordinator.sqlite3"
        )
        with mock.patch.object(migrations, "LATEST_SCHEMA_VERSION", 51):
            self.assertEqual(51, migrations.migrate(upgrade_database))
        with upgrade_database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO sessions(
                    session_id, status, status_reason, created_at, updated_at, last_heartbeat_at
                ) VALUES ('schema-51-session', 'active', 'preserve', 'now', 'now', 'now')
                """
            )

        self.assertEqual(
            migrations.LATEST_SCHEMA_VERSION, migrations.migrate(upgrade_database)
        )

        with upgrade_database.connect() as connection:
            tables = {
                row["name"]
                for row in connection.execute(
                    "SELECT name FROM sqlite_master WHERE type='table'"
                )
            }
            session = connection.execute(
                """
                SELECT status, status_reason, session_role, plan_family_key
                FROM sessions WHERE session_id='schema-51-session'
                """
            ).fetchone()
            version = connection.execute("SELECT MAX(version) FROM schema_version").fetchone()[0]

        self.assertEqual(migrations.LATEST_SCHEMA_VERSION, version)
        self.assertTrue({"ownership_transfer_previews", "ownership_transfers"} <= tables)
        self.assertEqual(("active", "preserve", "primary", None), tuple(session))


if __name__ == "__main__":
    unittest.main()
