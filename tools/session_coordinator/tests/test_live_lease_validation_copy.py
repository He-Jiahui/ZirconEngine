from __future__ import annotations

import hashlib
import tempfile
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path
from unittest import mock

from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.server import CoordinatorApplication
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workspace_copy import WorkspaceCopyService


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class LiveLeaseValidationCopyTests(unittest.TestCase):
    def test_untracked_leases_attribute_and_materialize_the_same_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            application.command("session.register", {"session_id": "owner"})
            application.command("session.register", {"session_id": "foreign"})
            application.command("baseline.init", {})

            exact_path = "src/exact.rs"
            derived_path = "src/generated/derived.rs"
            exact = repo / exact_path
            derived = repo / derived_path
            exact.parent.mkdir(parents=True)
            derived.parent.mkdir(parents=True)
            exact.write_text("pub const EXACT: u8 = 1;\n", encoding="utf-8")
            derived.write_text("pub const DERIVED: u8 = 2;\n", encoding="utf-8")

            claimed = application.command(
                "lease.claim",
                {
                    "session_id": "owner",
                    "paths": [exact_path, "src/generated"],
                },
            )["lease"]
            self.assertTrue(claimed["acquired"])
            self.assertFalse(claimed["conflicts"])

            attributed = application.command(
                "baseline.attribute",
                {"session_id": "owner", "paths": [exact_path, derived_path]},
            )
            renewed = application.command(
                "lease.heartbeat", {"session_id": "owner"}
            )
            rejected = application.command(
                "lease.claim",
                {"session_id": "foreign", "paths": [derived_path]},
            )["lease"]

            self.assertEqual("attributed", attributed["status"])
            self.assertEqual(2, renewed["renewed"])
            self.assertFalse(rejected["acquired"])
            self.assertEqual(("src/generated",), rejected["conflicts"])

            target_root = root / "targets"
            target_root.mkdir()
            with mock.patch(
                "tools.session_coordinator.workspace_copy._is_managed_validation_root",
                return_value=True,
            ):
                copies = WorkspaceCopyService(
                    application.database, repo, (target_root,)
                )
            materialized = copies.materialize(
                "owner", include_paths=(exact_path, derived_path)
            )

            self.assertEqual((exact_path, derived_path), materialized.manifest)
            for path in materialized.manifest:
                self.assertEqual(
                    _sha256(repo / path), _sha256(materialized.source_root / path)
                )

    def test_untracked_lease_heartbeat_extends_reclaim_deadline(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(
                repo, state_root=root / "state", port=0
            )
            application = CoordinatorApplication(config)
            application.supervision.mark_healthy()
            application.command("session.register", {"session_id": "owner"})
            application.command("session.register", {"session_id": "foreign"})

            path = "src/new.rs"
            source = repo / path
            source.parent.mkdir()
            source.write_text("pub const NEW: bool = true;\n", encoding="utf-8")
            started = datetime(2026, 8, 3, tzinfo=timezone.utc)
            heartbeat_at = started + timedelta(
                seconds=config.lease_ttl_seconds // 2
            )
            old_deadline = started + timedelta(
                seconds=config.lease_ttl_seconds + config.lease_grace_seconds
            )
            new_deadline = heartbeat_at + timedelta(
                seconds=config.lease_ttl_seconds + config.lease_grace_seconds
            )

            self.assertTrue(
                application.leases.acquire("owner", [path], now=started).acquired
            )
            self.assertEqual(
                1, application.leases.heartbeat("owner", now=heartbeat_at)
            )
            self.assertFalse(
                application.leases.acquire(
                    "foreign", [path], now=old_deadline
                ).acquired
            )
            self.assertTrue(
                application.leases.acquire(
                    "foreign", [path], now=new_deadline + timedelta(seconds=1)
                ).acquired
            )


if __name__ == "__main__":
    unittest.main()
