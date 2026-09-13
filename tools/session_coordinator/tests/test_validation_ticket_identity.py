from __future__ import annotations

import copy
import json
import unittest
from unittest import mock

from tools.session_coordinator import validation_ticket_identity as identity
from tools.session_coordinator.models import CoordinatorError


class ValidationIdentityTests(unittest.TestCase):
    def setUp(self) -> None:
        self.identity = identity.create_identity(
            baseline_epoch=7, base_head="head-1", validator_version="validator-1",
            runtime_identity=None, execution_environment_hash="e" * 64,
        )
        self.inputs = {
            "identity": self.identity, "source_manifest_hash": "a" * 64,
            "command": ["python", "-V"], "toolchain": {"cargo": "not-required"},
            "coverage": {"dependencyRoots": ["tools"]},
        }

    def test_version_one_fixed_vector(self) -> None:
        self.assertEqual(
            "24b5b4c82202f79efe033ca3cde4e929d58222d4ed9081839bdbb042605c872b",
            identity.dedupe_key(**self.inputs),
        )

    def test_each_submission_input_changes_identity(self) -> None:
        original = identity.dedupe_key(**self.inputs)
        for field, changed in (
            ("baselineEpoch", 8), ("baseHead", "head-2"),
            ("validatorVersion", "validator-2"), ("runtimeIdentityHash", "b" * 64),
            ("executionEnvironmentHash", "c" * 64),
        ):
            with self.subTest(field=field):
                value = {**self.inputs, "identity": {**self.identity, field: changed}}
                self.assertNotEqual(original, identity.dedupe_key(**value))
        for field, changed in (
            ("source_manifest_hash", "b" * 64), ("command", ["python", "--version"]),
            ("toolchain", {"python": "changed"}), ("coverage", {"dependencyRoots": ["tests"]}),
        ):
            with self.subTest(field=field):
                self.assertNotEqual(original, identity.dedupe_key(**{**self.inputs, field: changed}))

    def test_runtime_identity_is_hashed_without_persisting_raw_text(self) -> None:
        value = identity.create_identity(
            baseline_epoch=7, base_head="head-1", validator_version="validator-1",
            runtime_identity="abc", execution_environment_hash="e" * 64,
        )
        self.assertEqual(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            value["runtimeIdentityHash"],
        )
        self.assertNotIn("abc", json.dumps(value))

    def test_rejects_ambiguous_or_malformed_identity_fields(self) -> None:
        for field, changed in (
            ("version", True), ("version", 2), ("baselineEpoch", True),
            ("baselineEpoch", -1), ("baseHead", []), ("validatorVersion", ""),
            ("runtimeIdentityHash", "raw-runtime"), ("executionEnvironmentHash", None),
        ):
            with self.subTest(field=field, changed=changed), self.assertRaises(CoordinatorError):
                identity.dedupe_key(**{**self.inputs, "identity": {**self.identity, field: changed}})


class CopyCloseoutIdentityTests(unittest.TestCase):
    def setUp(self) -> None:
        from tools.session_coordinator.tests import test_failure_closeout as closeout_tests

        self.fixture = closeout_tests.FailureCloseoutWorkflowTests()
        self.fixture.setUp()
        self.addCleanup(self.fixture.tearDown)
        self.command, self.key = self.fixture._insert_green_copy_validation()

    def _contract(self):
        return self.fixture.service._load_validation_contract(
            session_id=self.fixture.session_id, job_id="copy-green",
            cargo_run_id="copy-run-green", command=tuple(self.command),
        )

    def _submitted(self):
        with self.fixture.database.connect() as connection:
            return json.loads(connection.execute(
                "SELECT payload_json FROM validation_ticket_events "
                "WHERE ticket_id='copy-run-green' AND event_type='validation.ticket_submitted'",
            ).fetchone()[0])

    def _write_submission(self, value):
        with self.fixture.database.transaction() as connection:
            connection.execute(
                "UPDATE validation_ticket_events SET payload_json=? "
                "WHERE ticket_id='copy-run-green' AND event_type='validation.ticket_submitted'",
                (json.dumps(value),),
            )

    def test_binds_original_submission_after_session_and_process_change(self) -> None:
        before = self._contract()
        with self.fixture.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET baseline_epoch=999, base_head=? WHERE session_id=?",
                ("f" * 40, self.fixture.session_id),
            )
        with mock.patch.dict("os.environ", {"ZIRCON_IDENTITY_TEST": "later-process"}):
            self.assertEqual(before, self._contract())
            prepared = self.fixture.service.prepare(
                session_id=self.fixture.session_id, snapshot_id=self.fixture.snapshot.snapshot_id,
                lifecycle_key=self.fixture.lifecycle_key, validation_command=self.command,
                validation_job_id="copy-green", validation_run_id="copy-run-green",
                executor_thread_id="executor-thread", actor=self.fixture.session_id,
            )
            evidence = self.fixture.service.bind_validation(
                session_id=self.fixture.session_id, closeout_id=prepared.closeout_id,
                job_id="copy-green", cargo_run_id="copy-run-green", actor=self.fixture.session_id,
            )
        self.assertEqual("accepted", evidence.verdict)

    def test_tampering_each_persisted_identity_input_is_rejected(self) -> None:
        original = self._submitted()
        self.assertEqual(self.key, self._contract()["compatibilityKey"])
        for field, changed in (
            ("baselineEpoch", 999), ("baseHead", "b" * 40),
            ("validatorVersion", "tampered"), ("runtimeIdentityHash", "b" * 64),
            ("executionEnvironmentHash", "c" * 64),
        ):
            with self.subTest(field=field):
                value = copy.deepcopy(original)
                value["identity"][field] = changed
                if field in value:
                    value[field] = changed
                self._write_submission(value)
                with self.assertRaises(CoordinatorError):
                    self._contract()
        self._write_submission(original)
        self.assertEqual(self.key, self._contract()["compatibilityKey"])

    def test_missing_and_unknown_identity_require_fresh_validation(self) -> None:
        original = self._submitted()
        variants = [None, {}, {**original["identity"], "version": 2}]
        variants.extend(
            {key: value for key, value in original["identity"].items() if key != missing}
            for missing in original["identity"]
        )
        for value in variants:
            with self.subTest(identity=value):
                self._write_submission({**original, "identity": value})
                with self.assertRaises(CoordinatorError) as rejected:
                    self._contract()
                self.assertEqual("validation_ticket_identity_revalidation_required", rejected.exception.code)
        with self.fixture.database.transaction() as connection:
            connection.execute(
                "DELETE FROM validation_ticket_events WHERE ticket_id='copy-run-green' "
                "AND event_type='validation.ticket_submitted'",
            )
        with self.assertRaises(CoordinatorError) as rejected:
            self._contract()
        self.assertEqual("validation_ticket_identity_revalidation_required", rejected.exception.code)

    def test_ticket_payload_and_submission_metadata_are_still_bound(self) -> None:
        original = self._submitted()
        for field, changed in (
            ("sessionId", "reviewer-b"), ("sourceManifestHash", "c" * 64),
            ("baselineEpoch", 999), ("baseHead", "changed"),
            ("validatorVersion", "changed"), ("runtimeIdentityHash", "c" * 64),
        ):
            with self.subTest(field=field):
                self._write_submission({**original, field: changed})
                with self.assertRaises(CoordinatorError):
                    self._contract()
        self._write_submission(original)
        for column in ("toolchain_json", "coverage_json", "command_json", "source_manifest_json"):
            with self.subTest(column=column), self.fixture.database.transaction() as connection:
                prior = connection.execute(
                    f"SELECT {column} FROM validation_tickets WHERE ticket_id='copy-run-green'",
                ).fetchone()[0]
                connection.execute(
                    f"UPDATE validation_tickets SET {column}=? WHERE ticket_id='copy-run-green'",
                    (json.dumps(["changed"] if column == "command_json" else {"changed": "c" * 64}),),
                )
            with self.assertRaises(CoordinatorError):
                self._contract()
            with self.fixture.database.transaction() as connection:
                connection.execute(
                    f"UPDATE validation_tickets SET {column}=? WHERE ticket_id='copy-run-green'", (prior,),
                )
        self.assertEqual(self.key, self._contract()["compatibilityKey"])

    def test_real_submission_persists_environment_digest_and_invalidates_reuse(self) -> None:
        from tools.session_coordinator.validation_tickets import ValidationTicketService

        service = ValidationTicketService(self.fixture.database, validator_version="test-validator")
        def submit(request_id):
            return service.submit(
                session_id=self.fixture.session_id, request_id=request_id,
                source_manifest={path: self.fixture.snapshot.manifest[path] for path in self.fixture.paths},
                command=self.command, toolchain={"cargo": "not-required"},
                coverage={"dependencyRoots": ["tools/session_coordinator"]},
            ).ticket.ticket_id
        with mock.patch.dict("os.environ", {"ZIRCON_IDENTITY_TEST": "private-first-value"}):
            first = submit("identity-first")
            same = submit("identity-same")
        with mock.patch.dict("os.environ", {"ZIRCON_IDENTITY_TEST": "private-second-value"}):
            different = submit("identity-different")
        self.assertEqual(first, same)
        self.assertNotEqual(first, different)
        with self.fixture.database.connect() as connection:
            payloads = connection.execute(
                "SELECT payload_json FROM validation_ticket_events WHERE ticket_id IN (?, ?) "
                "AND event_type='validation.ticket_submitted'", (first, different),
            ).fetchall()
        for row in payloads:
            self.assertNotIn("private-first-value", row[0])
            self.assertNotIn("private-second-value", row[0])
            self.assertRegex(json.loads(row[0])["identity"]["executionEnvironmentHash"], r"^[0-9a-f]{64}$")
