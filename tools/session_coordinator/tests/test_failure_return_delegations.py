from __future__ import annotations

import json
import tempfile
import unittest
from datetime import date
from pathlib import Path

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.failure_return_delegations import (
    FailureReturnDelegationService,
)
from tools.session_coordinator.failures import FailureGraphService, FailureResolution
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, SessionStatus, utc_text
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.failure_fixture import FailureGraphFixture
from tools.session_coordinator.tests.helpers import init_repo


class FailureReturnDelegationServiceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        fixture = FailureGraphFixture(self.repo)
        self.origin = fixture.add_plan("docs/plans/runtime/04-runtime.md")
        self.fixing = fixture.add_plan("docs/plans/tooling/01-tooling.md")
        failure = fixture.add_handoff(self.origin, self.fixing, "delegated-fixed")
        failure.write_text(
            failure.read_text(encoding="utf-8").replace(
                "summary_slug:", "plan_link_mode: child_record_only\nsummary_slug:"
            ),
            encoding="utf-8",
        )
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.sessions.register(
            session_id="origin-owner",
            plan_path=self.origin.path.relative_to(self.repo).as_posix(),
        )
        self.sessions.register(
            session_id="fixer",
            plan_path=self.fixing.path.relative_to(self.repo).as_posix(),
        )
        self.sessions.register(session_id="foreign-owner")
        self.sessions.set_status("origin-owner", SessionStatus.ACTIVE)
        self.sessions.set_status("fixer", SessionStatus.ACTIVE)
        self.sessions.set_status("fixer", SessionStatus.RESOLVING_FAILURE)
        self.sessions.set_status("foreign-owner", SessionStatus.ACTIVE)
        self.baselines = BaselineService(self.database, self.repo)
        self.baselines.initialize()
        self.leases = LeaseService(
            self.database,
            PathPolicy(self.repo),
            ttl_seconds=config.lease_ttl_seconds,
            grace_seconds=config.lease_grace_seconds,
        )
        self.failures = FailureGraphService(self.database, self.repo)
        open_node = self.failures.import_repository().nodes[0]
        destination = self.failures.return_fixed(
            open_node.lifecycle_key,
            FailureResolution(
                root_cause="origin attribution was not consumable",
                architecture_fix="bind an exact delegated proof",
                validation="delegation tests pass",
                return_summary="closeout may consume the proof",
            ),
            resolved_at=date(2026, 8, 23),
        )
        self.destination = destination.relative_to(self.repo).as_posix()
        fixed_node = self.failures.import_repository().nodes[0]
        self.lifecycle_key = fixed_node.lifecycle_key
        self.leases.acquire(
            "origin-owner", [self.origin.child.relative_to(self.repo).as_posix()]
        )
        self.baselines.attribute("origin-owner", [self.destination])
        with self.database.transaction() as connection:
            cursor = connection.execute(
                """INSERT INTO events(session_id, event_type, payload_json, created_at)
                   VALUES (?, 'failure.return_origin_destination_authorized', ?, ?)""",
                (
                    "fixer",
                    json.dumps(
                        {
                            "lifecycleKey": self.lifecycle_key,
                            "destination": self.destination,
                            "originOwnerSessionId": "origin-owner",
                            "originPlan": self.origin.path.relative_to(self.repo).as_posix(),
                        },
                        sort_keys=True,
                    ),
                    utc_text(),
                ),
            )
            self.authorization_event_id = int(cursor.lastrowid)
        self.service = FailureReturnDelegationService(
            self.database, self.repo, self.baselines
        )

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def _prepare(self):
        return self.service.prepare_proofs(
            fixing_session_id="fixer",
            lifecycle_keys=(self.lifecycle_key,),
            manifest_paths=(self.destination,),
        )

    def test_prepare_upgrades_legacy_authorization_to_exact_durable_proof(self) -> None:
        proofs = self._prepare()

        self.assertEqual(1, len(proofs))
        proof = proofs[0]
        self.assertEqual("origin-owner", proof.origin_session_id)
        self.assertEqual(self.destination, proof.destination_path)
        self.assertEqual(self.authorization_event_id, proof.authorization_event_id)
        self.assertEqual(self.baselines.current().epoch_id, proof.baseline_epoch)
        self.service.require_for_commit(
            fixing_session_id="fixer",
            closeout_id="closeout-a",
            input_fingerprint="a" * 64,
            lifecycle_keys=(self.lifecycle_key,),
            manifest_paths=(self.destination,),
            proofs=proofs,
        )

    def test_content_drift_after_prepare_fails_closed(self) -> None:
        proofs = self._prepare()
        (self.repo / self.destination).write_text("tampered\n", encoding="utf-8")

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.require_for_commit(
                fixing_session_id="fixer",
                closeout_id="closeout-a",
                input_fingerprint="a" * 64,
                lifecycle_keys=(self.lifecycle_key,),
                manifest_paths=(self.destination,),
                proofs=proofs,
            )

        self.assertEqual(
            "failure_closeout_delegation_content_drift", rejected.exception.code
        )

    def test_origin_lease_reacquisition_by_foreign_session_fails_closed(self) -> None:
        proofs = self._prepare()
        self.leases.release(
            "origin-owner", [self.origin.child.relative_to(self.repo).as_posix()]
        )
        self.leases.acquire("foreign-owner", [self.destination])

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.require_for_commit(
                fixing_session_id="fixer",
                closeout_id="closeout-a",
                input_fingerprint="a" * 64,
                lifecycle_keys=(self.lifecycle_key,),
                manifest_paths=(self.destination,),
                proofs=proofs,
            )

        self.assertEqual(
            "failure_closeout_delegation_origin_lease_missing",
            rejected.exception.code,
        )

    def test_consumed_proof_cannot_replay(self) -> None:
        proofs = self._prepare()
        self.service.consume(
            fixing_session_id="fixer",
            closeout_id="closeout-a",
            input_fingerprint="a" * 64,
            lifecycle_keys=(self.lifecycle_key,),
            manifest_paths=(self.destination,),
            proofs=proofs,
            commit_sha="b" * 40,
        )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.require_for_commit(
                fixing_session_id="fixer",
                closeout_id="closeout-b",
                input_fingerprint="c" * 64,
                lifecycle_keys=(self.lifecycle_key,),
                manifest_paths=(self.destination,),
                proofs=proofs,
            )

        self.assertEqual("failure_closeout_delegation_replayed", rejected.exception.code)


if __name__ == "__main__":
    unittest.main()
