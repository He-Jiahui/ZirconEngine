"""Failed upgrade closeout and keeper handoff, using only synthetic local evidence."""
from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
import unittest
from unittest import mock

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.deployment.execution_activation import support_sources
from tools.tests import test_jenkins_upgrade_desktop_holder as holder_tests
from tools.tests import test_jenkins_execution_upgrade_selection as selection_tests

holder = holder_tests.holder
upgrade = selection_tests.upgrade


class FailedCloseoutTests(unittest.TestCase):
    def setUp(self):
        self.base = holder_tests.UpgradeHolderTests()
        self.base.setUp()
        self.addCleanup(self.base.doCleanups)
        self.fixture = self.base.fixture
        self.selection = self.fixture.selection
        self.manager = self.fixture.manager
        self.manager.base_url = "http://127.0.0.1:1"
        self.database = self.fixture.database
        endpoint = lambda _address, _port, pid: {"address": "127.0.0.1", "port": 1, "pid": pid, "ownerCount": 1}
        self.fixture.stack.enter_context(mock.patch.object(holder, "observe_endpoint_owner", side_effect=endpoint))
        self.fixture.stack.enter_context(mock.patch.object(upgrade, "observe_endpoint_owner", side_effect=endpoint))
        self.base.run_upgrade()
        self.original_selection = self.selection.journal.read_bytes()
        self.original_start = self.selection.start_journal.read_bytes()
        self.selector_bytes = self.selection.selector.read_bytes()
        self.activation_bytes = self.selection.activation.read_bytes()
        self.holds = holder.protected_holds(self.fixture.state)
        self.owner = self.selection._owner(*self.selection._snapshot())
        self.manager._stop_locked.reset_mock()
        self.manager._start_locked.reset_mock()
        root = self.fixture.repo / ".jenkins/tmp/acceptance/m4-fixture"
        root.mkdir(parents=True)
        self.preimage = root / "source.rs"
        self.preimage.write_bytes(b"original synthetic fixture\n")
        before = {"source.rs": upgrade.sha(self.preimage)}
        after = {"source.rs": hashlib.sha256(b"changed synthetic fixture\n").hexdigest()}
        self.identity = {"repositoryId": "fixture-repository", "sessionId": "m4-session", "requestId": "m4-request",
            "attemptId": "m4-attempt", "generation": 1, "stageImplementationDigest": self.fixture.candidate["driverDigest"]}
        intent = {"identity": self.identity, "repositoryRoot": str(root), "beforeHashes": before,
                  "renderedDigest": digest({"source.rs": "changed synthetic fixture\n"}),
                  "unifiedPatch": "synthetic source-owned raw patch", "coverage": {"template": "module_unit"},
                  "declaredDependencies": [], "externalInputs": [], "baseHead": None}
        grant = self.database.authorize_session("fixture-repository", "m4-session", "fixture-user", ["source.rs"],
            ["implementation", "validation"], {"source": "user", "task": "synthetic original M4 authorization"})
        intent["authorizationSnapshot"] = grant["snapshotDigest"]
        descriptor = {"schemaVersion": 2, "kind": "formal-incremental-patch-acceptance", "gate": "m4",
            "nonce": root.name, "fixtureRoot": str(root), "repositoryId": self.identity["repositoryId"],
            "driverBinding": {"status": "ready", "driverDigest": self.fixture.candidate["driverDigest"], "runtimeOperationId": "new-operation"},
            "patches": [{k: intent[k] for k in ("identity", "repositoryRoot", "beforeHashes", "unifiedPatch", "coverage",
                                                "declaredDependencies", "externalInputs", "baseHead")}],
            "ownedPaths": ["source.rs"], "scopeHashes": before,
            "policies": {k: False for k in ("allowCommit", "allowPush", "allowNotify", "runCargoInAdapter", "createGitBaselineInAdapter", "createSourceSnapshots")}}
        descriptor["descriptorDigest"] = digest(descriptor)
        self.descriptor = root / "descriptor.json"
        self.descriptor.write_text(json.dumps(descriptor), encoding="utf-8")
        self.patch_ref = digest(intent)
        self.checkout_key = digest({k: self.identity[k] for k in
            ("repositoryId", "sessionId", "requestId", "attemptId", "generation")})
        self.database.put("acceptance_checkout", self.checkout_key, {
            "schemaVersion": 1, "kind": "original-acceptance-checkout", "identity": self.identity,
            "contextRoot": str(self.fixture.repo), "checkoutRoot": str(root), "ownedPaths": ["source.rs"],
            "runtimeOperationId": "new-operation", "driverDigest": self.identity["stageImplementationDigest"],
            "authorizationSnapshot": grant["snapshotDigest"], "productRepositoryAcceptance": False})
        self.operation_id = "patch-" + self.patch_ref
        self.proof = {"kind": "no-execution-dispatch", "workflowRegistered": False, "patchLockHeld": True}
        compensation_id = digest({"patch": self.operation_id, "recoveryOwner": self.identity["attemptId"]})
        recovery = {"status": "complete", "operations": [compensation_id], "terminationEvidence": self.proof}
        self.database.put("patch_intent", self.patch_ref, intent)
        self.database.put("patch_preparation", self.patch_ref, {"status": "failed", "operationId": self.operation_id,
            "reasonCode": "AttributeError", "recovery": recovery})
        self.database.put_operation("source_patch", self.operation_id, {"owner": self.identity["attemptId"],
            "repositoryRoot": str(root), "beforeHashes": before, "expectedAfterHashes": after,
            "patchDigest": intent["renderedDigest"]}, status="complete")
        self.database.put_operation("source_compensation", compensation_id, {"patchOperationId": self.operation_id,
            "owner": self.identity["attemptId"], "repositoryRoot": str(root), "beforeHashes": before,
            "afterHashes": after, "terminationEvidence": self.proof})
        self.database.transition_operation(compensation_id, "prepared", "complete", {
            "restoredPaths": ["source.rs"], "terminationEvidence": self.proof})
        self.submission_key = "fixture-repository:m4-session:m4-request:flow"
        self.database.put("jenkins_submission", self.submission_key, {
            **{k: self.identity[k] for k in ("repositoryId", "sessionId", "requestId")},
            "parameters": {"patchRequestRef": self.patch_ref, "runtimeOperationId": "new-operation",
                           "stageImplementationDigest": self.identity["stageImplementationDigest"]},
            "delivery": "submitted", "dispatchDigest": "a" * 64, "queueUrl": "http://127.0.0.1:1/queue/item/91/"})
        self.parameters = {"REPOSITORY_ID": "fixture-repository", "SESSION_ID": "m4-session", "REQUEST_ID": "m4-request",
            "ATTEMPT_ID": "m4-attempt", "GENERATION": "1", "PATCH_REQUEST_REF": self.patch_ref,
            "STAGE_IMPLEMENTATION_DIGEST": self.identity["stageImplementationDigest"], "RUNTIME_OPERATION_ID": "new-operation"}
        self.api_values = {
            "job/zircon-flow/29/api/json": {"number": 29, "queueId": 91, "building": False, "result": "FAILURE",
                "actions": [{"parameters": [{"name": k, "value": v} for k, v in self.parameters.items()]}]},
            "api/json": {"jobs": [{"name": "zircon-flow", "builds": [{"building": False}]}]},
            "computer/api/json": {"computer": [{"displayName": "fixture-agent", "numExecutors": 1,
                "executors": [{"idle": True, "currentExecutable": None}], "oneOffExecutors": []}]},
            "queue/api/json": {"items": []}}
        self.manager._request.side_effect = lambda endpoint, **_kwargs: (200, json.dumps(self.api_values[endpoint.split("?")[0]]).encode())
        journal = upgrade.read_json(self.selection.journal)
        self.review_kwargs = {"selection_sha256": upgrade.sha(self.selection.journal),
            "start_sha256": upgrade.sha(self.selection.start_journal), "startup_receipt_sha256": journal["startupReceipt"]["sha256"],
            "driver_digest": self.fixture.candidate["driverDigest"], "runtime_operation_id": "new-operation",
            "patch_ref": self.patch_ref, "build_number": 29, "queue_id": 91}

    def review(self):
        return holder.review_failed_acceptance(self.manager, self.selection, histories=[], **self.review_kwargs)

    def review_file(self):
        value = self.review()
        path = self.fixture.state / "migration-baseline/failed-closeout-review.json"
        path.write_text(json.dumps(value), encoding="utf-8")
        return path, upgrade.sha(path)

    def close(self):
        path, checksum = self.review_file()
        return self.selection.close_failed_acceptance(review_path=path, review_sha256=checksum)

    def assert_lifecycle_untouched(self):
        self.manager._stop_locked.assert_not_called()
        self.manager._start_locked.assert_not_called()
        self.assertEqual(self.selector_bytes, self.selection.selector.read_bytes())
        self.assertEqual(self.activation_bytes, self.selection.activation.read_bytes())
        self.assertEqual(self.holds, holder.protected_holds(self.fixture.state))

    def successor(self, closed):
        self.fixture.source.write_text("print('third synthetic fixture')\n", encoding="utf-8")
        candidate = selection_tests.seal_fixture_driver(self.fixture.repo)
        regression = upgrade.read_json(self.fixture.regression)
        regression["sourceFiles"] = support_sources(self.fixture.repo)
        path = self.fixture.state / "migration-baseline/second-regression.json"
        path.write_text(json.dumps(regression), encoding="utf-8")
        value = self.selection.review(candidate_root=Path(candidate["root"]), regression_path=path,
                                      failed_closeout=closed["failedCloseout"])
        review = self.fixture.state / "migration-baseline/second-upgrade-review.json"
        review.write_text(json.dumps(value), encoding="utf-8")
        return candidate, value["plan"], {"reviewedUpgradePath": str(review), "reviewedUpgradeSha256": upgrade.sha(review)}

    def keeper(self, plan):
        original = self.selection._json_reference(plan["oldDriver"]["executionUpgradeIntent"])
        old_release = {"schemaVersion": 1, "kind": "desktop-execution-upgrade-release", "repository": str(self.fixture.repo),
            "configSha256": upgrade.sha(self.fixture.repo / ".codex/config.toml"),
            "candidateDriverRoot": plan["oldDriver"]["root"], "candidateDriverDigest": plan["oldDriver"]["driverDigest"],
            "reviewedUpgradePath": original["review"]["path"], "reviewedUpgradeSha256": original["review"]["sha256"],
            "expectedPreviousOwner": original["plan"]["owner"]}
        release_path = self.fixture.state / "migration-baseline/original-keeper-release.json"
        release_path.write_text(json.dumps(old_release), encoding="utf-8")
        reference = {"path": str(release_path), "sha256": upgrade.sha(release_path)}
        record = {"holderId": "desktop-upgrade-synthetic", "status": "holding-new-generation-acceptance-pending",
            "automaticRestart": False, "reviewedRelease": reference,
            "ownedGeneration": holder.baseline.owner_ref(self.fixture.operation),
            "restrictionPreflight": {"holderIdentity": {"pid": 9001, "creationTime": "15999",
                "executable": str(self.fixture.python), "commandDigest": ""}}}
        path = self.fixture.state / "desktop-upgrade-holder/desktop-upgrade-synthetic.json"
        path.parent.mkdir()
        path.write_text(json.dumps(record), encoding="utf-8")
        args = (str(self.fixture.python), "-B", str(holder.ENTRY), "--config-sha256", old_release["configSha256"],
                "--release-file", str(release_path), "--release-sha256", reference["sha256"], "--no-tray")
        self.fixture.stack.enter_context(mock.patch.object(holder, "_windows_command_line", return_value=args))
        self.fixture.stack.enter_context(mock.patch.object(holder, "observe_departed_identity",
            side_effect=lambda value: {"pid": value["pid"], "expectedCreationTime": value["creationTime"], "status": "absent"}))
        proof = holder.review_keeper_handoff(self.manager, self.selection, plan, path, upgrade.sha(path))
        return proof, path

    def test_review_is_read_only_and_old_pending_guard_still_rejects_successor(self):
        value = self.review()
        self.assertEqual("failed", value["plan"]["functionalAcceptance"])
        self.assertEqual("complete", value["plan"]["failure"]["recovery"]["status"])
        self.assertEqual(self.original_selection, self.selection.journal.read_bytes())
        self.assertEqual(self.original_start, self.selection.start_journal.read_bytes())
        with self.assertRaises(JenkinsError):
            self.selection.review(candidate_root=Path(self.fixture.candidate["root"]), regression_path=self.fixture.regression)
        self.assert_lifecycle_untouched()

    def test_close_preserves_raw_history_and_never_grants_runtime_or_cargo_acceptance(self):
        closed = self.close()
        closure = self.selection._json_reference(closed["failedCloseout"])
        self.assertEqual(self.original_selection, self.selection._reference(closure["selectionArchive"]))
        self.assertEqual(self.original_start, self.selection._reference(closure["startArchive"]))
        self.assertEqual(upgrade.FAILED_SELECTION, upgrade.read_json(self.selection.journal)["status"])
        self.assertEqual(upgrade.FAILED_START, upgrade.read_json(self.selection.start_journal)["status"])
        self.assertFalse(closed["runtimeAcceptance"])
        self.assertFalse(closed["cargoAcceptance"])
        self.assert_lifecycle_untouched()
        self.selection.verify_failed_acceptance(closed["failedCloseout"])

    def test_failed_preparation_requires_actual_complete_hash_compensation(self):
        self.preimage.write_bytes(b"foreign source after compensation\n")
        with self.assertRaises(JenkinsError) as error:
            self.review()
        self.assertEqual("execution_upgrade_compensation_changed", error.exception.code)
        self.assert_lifecycle_untouched()

    def test_formal_fixture_descriptor_and_original_grant_remain_exact(self):
        descriptor = upgrade.read_json(self.descriptor)
        changed = {**descriptor, "nonce": "foreign-nonce"}
        changed["descriptorDigest"] = digest({k: v for k, v in changed.items() if k != "descriptorDigest"})
        self.descriptor.write_text(json.dumps(changed), encoding="utf-8")
        with self.assertRaises(JenkinsError):
            self.review()
        self.descriptor.write_text(json.dumps(descriptor), encoding="utf-8")
        self.database.revoke_session(self.identity["repositoryId"], self.identity["sessionId"])
        with self.assertRaises(JenkinsError):
            self.review()
        self.assert_lifecycle_untouched()

    def test_pending_unknown_foreign_and_partial_failures_never_close(self):
        for change in ({"status": "prepared"}, {"recovery": {"status": "waiting"}},
                       {"recovery": {"status": "complete", "operations": [], "terminationEvidence": self.proof}}):
            row = self.database.get("patch_preparation", self.patch_ref)
            original = row["payload"]
            self.database.put("patch_preparation", self.patch_ref, {**original, **change}, expected_version=row["version"])
            with self.assertRaises(JenkinsError):
                self.review()
            row = self.database.get("patch_preparation", self.patch_ref)
            self.database.put("patch_preparation", self.patch_ref, original, expected_version=row["version"])
        self.assert_lifecycle_untouched()

    def test_only_exact_original_immutable_checkout_binding_is_allowed(self):
        original = self.database.get("acceptance_checkout", self.checkout_key)["payload"]
        for change in ({"checkoutRoot": str(self.fixture.repo)}, {"authorizationSnapshot": "0" * 64},
                       {"identity": {**self.identity, "attemptId": "foreign-attempt"}},
                       {"ownedPaths": ["other.rs"]}, {"status": "pending"}):
            row = self.database.get("acceptance_checkout", self.checkout_key)
            self.database.put("acceptance_checkout", self.checkout_key, {**original, **change}, expected_version=row["version"])
            with self.assertRaises(JenkinsError):
                self.review()
            row = self.database.get("acceptance_checkout", self.checkout_key)
            self.database.put("acceptance_checkout", self.checkout_key, original, expected_version=row["version"])
        self.database.put("acceptance_checkout", "another-current-checkout", original)
        with self.assertRaises(JenkinsError) as error:
            self.review()
        self.assertEqual("execution_upgrade_current_work", error.exception.code)
        self.assert_lifecycle_untouched()

    def test_missing_optional_location_needs_submitted_201_and_exact_failed_build(self):
        row = self.database.get("jenkins_submission", self.submission_key)
        self.database.put("jenkins_submission", self.submission_key,
            {**row["payload"], "status": 201, "queueUrl": None, "buildRef": None}, expected_version=row["version"])
        self.assertEqual("reviewed-failed-functional-acceptance", self.review()["status"])
        for status in (200, None, 201.0):
            row = self.database.get("jenkins_submission", self.submission_key)
            self.database.put("jenkins_submission", self.submission_key,
                {**row["payload"], "status": status}, expected_version=row["version"])
            with self.assertRaises(JenkinsError):
                self.review()
        row = self.database.get("jenkins_submission", self.submission_key)
        self.database.put("jenkins_submission", self.submission_key,
            {**row["payload"], "status": 201}, expected_version=row["version"])
        self.api_values["job/zircon-flow/29/api/json"]["queueId"] = 92
        with self.assertRaises(JenkinsError):
            self.review()
        self.assert_lifecycle_untouched()

    def test_original_failed_build_identity_queue_and_native_owner_are_required(self):
        for key, value in (("building", True), ("result", "SUCCESS"), ("queueId", 92), ("number", 30)):
            actual = self.api_values["job/zircon-flow/29/api/json"]
            previous = actual[key]
            actual[key] = value
            with self.assertRaises(JenkinsError):
                self.review()
            actual[key] = previous
        self.fixture.operation["creationTime"] = "99999"
        with self.assertRaises(JenkinsError):
            self.review()
        self.assert_lifecycle_untouched()

    def test_current_work_unknown_delivery_and_busy_queue_are_not_historical(self):
        self.api_values["queue/api/json"]["items"] = [{"id": 92}]
        with self.assertRaises(JenkinsError):
            self.review()
        self.api_values["queue/api/json"]["items"] = []
        row = self.database.get("jenkins_submission", self.submission_key)
        self.database.put("jenkins_submission", self.submission_key, {**row["payload"], "delivery": "unknown"}, expected_version=row["version"])
        with self.assertRaises(JenkinsError):
            self.review()
        row = self.database.get("jenkins_submission", self.submission_key)
        self.database.put("jenkins_submission", self.submission_key, {**row["payload"], "delivery": "submitted"}, expected_version=row["version"])
        self.database.put("execution", "current-work", {"operationId": "new-operation", "status": "pending"})
        with self.assertRaises(JenkinsError):
            self.review()
        self.assert_lifecycle_untouched()

    def test_undispatched_current_source_preparation_is_still_active_work(self):
        self.database.put("source_preparation", "current-preparation", {
            "identity": {"stageImplementationDigest": self.fixture.candidate["driverDigest"]}, "status": "preparing"})
        with self.assertRaises(JenkinsError) as error:
            self.review()
        self.assertEqual("execution_upgrade_current_work", error.exception.code)
        self.assert_lifecycle_untouched()

    def test_current_native_receipt_rejects_numeric_coercion_and_missing_identity(self):
        payload = {"status": "terminal", "operationId": "new-operation", "executionId": "metadata-fixture",
            "pid": 9002, "birthToken": "16002", "executable": str(self.fixture.python),
            "completeProof": {"complete": True, "childrenGone": True, "activeProcesses": 0,
                "stdoutEof": True, "stderrEof": True, "processExitCode": 0,
                "identity": {"pid": 9002, "creationTime": "16002", "executable": str(self.fixture.python), "commandDigest": "a" * 64}}}
        self.database.put("native_job", "metadata-fixture", payload)
        self.review()
        variants = []
        for value in (False, 0.0, None):
            changed = copy.deepcopy(payload)
            changed["completeProof"]["activeProcesses"] = value
            variants.append(changed)
        for name, value in (("pid", None), ("pid", False), ("pid", 0), ("birthToken", None),
                            ("birthToken", "0"), ("birthToken", "invalid"), ("executable", "")):
            changed = copy.deepcopy(payload)
            changed[name] = value
            observed_name = "creationTime" if name == "birthToken" else name
            changed["completeProof"]["identity"][observed_name] = value
            variants.append(changed)
        changed = copy.deepcopy(payload)
        changed["completeProof"]["identity"]["commandDigest"] = ""
        variants.append(changed)
        for changed in variants:
            row = self.database.get("native_job", "metadata-fixture")
            self.database.put("native_job", "metadata-fixture", changed, expected_version=row["version"])
            with self.assertRaises(JenkinsError) as error:
                self.review()
            self.assertEqual("execution_upgrade_current_work", error.exception.code)
        self.assert_lifecycle_untouched()

    def test_orphan_unresolved_native_job_cannot_hide_without_runtime_markers(self):
        self.database.put("native_job", "unowned-native", {"executionId": "missing-execution", "owner": "foreign",
            "pid": 9003, "birthToken": "16003", "executable": str(self.fixture.python), "status": "running"})
        with self.assertRaises(JenkinsError) as error:
            self.review()
        self.assertEqual("execution_upgrade_current_work", error.exception.code)
        self.assert_lifecycle_untouched()

    def test_post_review_cas_hold_or_compensation_change_prevents_pointer_writes(self):
        path, checksum = self.review_file()
        self.database.put("resource_reservation", "foreign", {"status": "active", "owner": "foreign"})
        with self.assertRaises(JenkinsError):
            self.selection.close_failed_acceptance(review_path=path, review_sha256=checksum)
        self.assertEqual(self.original_selection, self.selection.journal.read_bytes())
        self.assertEqual(self.original_start, self.selection.start_journal.read_bytes())
        self.manager._stop_locked.assert_not_called()
        self.manager._start_locked.assert_not_called()

    def test_partial_dual_pointer_exchange_requires_exact_explicit_reconciliation(self):
        path, checksum = self.review_file()
        atomic = self.selection._atomic_json
        def fail_start(destination, value):
            if destination == self.selection.start_journal:
                raise OSError("synthetic interrupted start-pointer exchange")
            return atomic(destination, value)
        with mock.patch.object(self.selection, "_atomic_json", side_effect=fail_start), self.assertRaises(OSError):
            self.selection.close_failed_acceptance(review_path=path, review_sha256=checksum)
        receipt = upgrade.read_json(self.selection.journal)["failedCloseout"]
        with self.assertRaises(JenkinsError):
            self.selection.verify_failed_acceptance(receipt)
        closed = self.selection.reconcile_failed_acceptance(receipt)
        self.assertEqual("failed-functional-acceptance-closed", closed["status"])
        self.selection.reconcile_failed_acceptance(receipt)
        self.assert_lifecycle_untouched()

    def test_reconciliation_never_replaces_foreign_or_unknown_pointer(self):
        closed = self.close()
        foreign = {**upgrade.read_json(self.selection.start_journal), "status": "unknown"}
        self.selection.start_journal.write_text(json.dumps(foreign), encoding="utf-8")
        raw = self.selection.start_journal.read_bytes()
        with self.assertRaises(JenkinsError):
            self.selection.reconcile_failed_acceptance(closed["failedCloseout"])
        self.assertEqual(raw, self.selection.start_journal.read_bytes())
        self.assert_lifecycle_untouched()

    def test_superseding_closed_failure_uses_normal_terminal_selection_and_serial_start_cas(self):
        closed = self.close()
        candidate, plan, release = self.successor(closed)
        self.assertEqual(closed, plan["failedPredecessor"])
        with self.assertRaises(JenkinsError):
            self.selection.select(review_path=Path(release["reviewedUpgradePath"]), review_sha256=release["reviewedUpgradeSha256"])
        self.fixture.terminal()
        result = self.selection.select(review_path=Path(release["reviewedUpgradePath"]), review_sha256=release["reviewedUpgradeSha256"])
        self.assertEqual(candidate["driverDigest"], result["candidateDriverDigest"])
        self.assertEqual(closed["failedCloseout"], self.selection._json_reference(upgrade.read_json(self.selection.journal)["intent"])["plan"]["failedPredecessor"]["failedCloseout"])
        self.manager._start_locked.assert_not_called()
        self.assertEqual(closed["startSha256"], upgrade.sha(self.selection.start_journal))

    def test_keeper_review_fills_only_new_authorization_and_never_retires_a_live_runtime(self):
        closed = self.close()
        _, plan, release = self.successor(closed)
        proof, path = self.keeper(plan)
        self.assertEqual("", upgrade.read_json(path)["restrictionPreflight"]["holderIdentity"]["commandDigest"])
        self.assertEqual(64, len(proof["identity"]["commandDigest"]))
        release["keeperHandoff"] = proof
        with mock.patch.object(holder, "_retire_keeper_process") as retire, self.assertRaises(JenkinsError):
            holder.retire_reviewed_keeper(self.manager, self.selection, release, plan, {})
        retire.assert_not_called()
        self.assert_lifecycle_untouched()

    def test_split_stop_retire_then_ordinary_lock_and_successor_start(self):
        closed = self.close()
        candidate, plan, release = self.successor(closed)
        proof, _ = self.keeper(plan)
        release["keeperHandoff"] = proof
        stopped = holder.stop_reviewed_generation(self.manager, self.selection, release, plan, lambda *_args: None)
        self.manager._start_locked.assert_not_called()
        self.assertEqual(self.selector_bytes, self.selection.selector.read_bytes())
        with mock.patch.object(holder, "_retire_keeper_process") as retire:
            retired = holder.retire_reviewed_keeper(self.manager, self.selection, release, plan, stopped)
        retire.assert_called_once_with(proof["identity"])
        self.assertFalse(retired["lockBypassed"])
        self.assertFalse(retired["runtimeAcceptance"])
        self.assertEqual(self.holds, holder.protected_holds(self.fixture.state))
        self.assertEqual(self.selector_bytes, self.selection.selector.read_bytes())
        self.manager._start_locked.assert_not_called()
        def start(_lock):
            result = self.base.start_new(_lock)
            self.fixture.operation.update(operationId="third-operation", generation="third-operation", creationTime="16001")
            self.fixture.host.update(operationId="third-operation", generation="third-operation")
            self.fixture.host_path.write_text(json.dumps(self.fixture.host), encoding="utf-8")
            selected = upgrade.read_json(self.selection.selector)
            selected["runtimeOperationId"] = "third-operation"
            self.selection.selector.write_text(json.dumps(selected), encoding="utf-8")
            return dict(self.fixture.operation)
        self.manager._start_locked.side_effect = start
        with holder.process_lock(self.fixture.state / "desktop-upgrade-holder/owner.lock", reason="desktop_upgrade_holder_already_live"):
            result = holder.perform_upgrade(self.manager, self.selection, release, plan, lambda *_args: None, stopped_evidence=stopped)
        self.assertEqual(candidate["driverDigest"], upgrade.read_json(self.selection.selector)["driverDigest"])
        self.assertEqual("third-operation", result["ownedGeneration"]["operationId"])
        self.manager._stop_locked.assert_called_once()
        self.manager._start_locked.assert_called_once()
        self.assertEqual(closed["failedCloseout"], upgrade.read_json(self.selection.start_journal)["failedPredecessor"]["failedCloseout"])

    def test_keeper_native_birth_argv_foreign_owner_and_unknown_departure_fail_closed(self):
        closed = self.close()
        _, plan, release = self.successor(closed)
        proof, path = self.keeper(plan)
        release["keeperHandoff"] = proof
        record = upgrade.read_json(path)
        changed = copy.deepcopy(record)
        changed["ownedGeneration"]["operationId"] = "foreign"
        path.write_text(json.dumps(changed), encoding="utf-8")
        with mock.patch.object(holder, "_retire_keeper_process") as retire, self.assertRaises(JenkinsError):
            holder.verify_keeper_handoff(self.manager, self.selection, plan, proof)
        retire.assert_not_called()
        path.write_text(json.dumps(record), encoding="utf-8")
        with mock.patch.object(holder, "_windows_command_line", return_value=("foreign-program",)), self.assertRaises(JenkinsError):
            holder.review_keeper_handoff(self.manager, self.selection, plan, path, upgrade.sha(path))
        stopped = holder.stop_reviewed_generation(self.manager, self.selection, release, plan, lambda *_args: None)
        with mock.patch.object(holder, "_retire_keeper_process") as retire, \
             mock.patch.object(holder, "observe_departed_identity", return_value={"status": "unknown"}), self.assertRaises(JenkinsError):
            holder.retire_reviewed_keeper(self.manager, self.selection, release, plan, stopped)
        self.manager._start_locked.assert_not_called()
        self.assertEqual(self.selector_bytes, self.selection.selector.read_bytes())

    def test_keeper_heartbeat_changes_only_three_observation_fields(self):
        closed = self.close()
        _, plan, release = self.successor(closed)
        proof, path = self.keeper(plan)
        release["keeperHandoff"] = proof
        record = upgrade.read_json(path)
        record.update(updatedAt=1234567.0, health={"ready": True, "controller": True}, currentGenerationMatches=True)
        path.write_text(json.dumps(record), encoding="utf-8")
        self.assertNotEqual(proof["record"]["sha256"], upgrade.sha(path))
        holder.verify_keeper_handoff(self.manager, self.selection, plan, proof)
        stopped = holder.stop_reviewed_generation(self.manager, self.selection, release, plan, lambda *_args: None)
        record.update(updatedAt=1234597.0, health={"ready": False, "controller": False}, currentGenerationMatches=True)
        path.write_text(json.dumps(record), encoding="utf-8")
        with mock.patch.object(holder, "_retire_keeper_process") as retire:
            holder.retire_reviewed_keeper(self.manager, self.selection, release, plan, stopped)
        retire.assert_called_once_with(proof["identity"])
        for key, value in (("status", "holding-stop-refused"), ("automaticRestart", True),
                           ("foreignOwnershipField", "not-a-heartbeat")):
            changed = {**record, key: value}
            path.write_text(json.dumps(changed), encoding="utf-8")
            with self.assertRaises(JenkinsError):
                holder.verify_keeper_handoff(self.manager, self.selection, plan, proof, terminal=True)


if __name__ == "__main__":
    unittest.main()
