from pathlib import Path
import json
import hashlib
import os
import sys
import threading
import unittest
from unittest.mock import patch

from tools.jenkins.pilot.contracts import PilotError, RequestIdentity, canonical_json, digest
from tools.jenkins.pilot import execution_dispatch as dispatch
from tools.jenkins.pilot.native.process_identity import process_creation_time, popen_process_creation_time
from tools.jenkins.pilot.native.windows_job_process import create_atomic_kill_on_close_process, terminate_and_close_process_job
from tools.jenkins.pilot.snapshot import load_snapshot
from tools.jenkins.pilot.storage import ManagedStorage, physical_identity
from tools.jenkins.pilot.tests.fixtures import sealed_bundle
from tools.jenkins.pilot.tests.test_native_process import native_directory


@unittest.skipUnless(os.name == "nt", "requires physical Windows managed storage")
class DispatchTests(unittest.TestCase):
    def test_submission_waits_for_transient_keeper_lock_without_duplicate_intent(self):
        held, release = threading.Event(), threading.Event()
        failures = []
        def keeper_hold():
            try:
                with dispatch._runtime_lock(self.storage):
                    held.set()
                    release.wait(5)
            except BaseException as error:
                failures.append(error)
        holder = threading.Thread(target=keeper_hold)
        holder.start()
        self.assertTrue(held.wait(5))
        timer = threading.Timer(0.3, release.set)
        timer.start()
        try:
            bundle = self.storage.path("producer.zip")
            self.storage.atomic_write("producer.zip", self.raw)
            with patch.object(dispatch, "_owner", return_value=self.owner), \
                 patch.object(dispatch, "_driver", return_value=(self.root, {"driverInputHash": self.binding["driverInputHash"]})), \
                 patch.object(dispatch, "_admit_build"), patch.object(dispatch, "_read_receipt", return_value={"fixture": "durable-matching-receipt"}):
                result = dispatch.enqueue_and_wait(root=self.root, repo_root=Path.cwd(), bundle=bundle,
                                                  identity=self.identity, job="zircon-pilot", build_number=1, maximum_seconds=5)
            self.assertEqual(result, {"fixture": "durable-matching-receipt"})
            intents = list(self.storage.path("execution-intents").glob("*.json"))
            self.assertEqual(len(intents), 1)
            self.assertEqual(json.loads(intents[0].read_bytes())["binding"], self.binding)
        finally:
            release.set()
            holder.join(5)
            timer.join(5)
        self.assertFalse(failures)

    def test_keeper_historical_proof_audit_does_not_hold_submission_lock(self):
        owner, relative, raw, value, receipt_path = self.recovered_fixture()
        original = dispatch._recovered_completed_intent
        observed = []
        def audit(*args):
            with dispatch._runtime_lock(self.storage):
                observed.append(True)
            return original(*args)
        with patch.object(dispatch, "_owner", return_value=owner), patch.object(dispatch, "_recovered_completed_intent", side_effect=audit):
            dispatch.service_execution_intents(self.root, Path.cwd(), owner)
        self.assertEqual(observed, [True])

    def test_wait_budget_expires_without_creating_or_rebinding_an_intent(self):
        bundle = self.storage.atomic_write("producer-timeout.zip", self.raw)
        with dispatch._runtime_lock(self.storage), \
             patch.object(dispatch, "_owner", return_value=self.owner), \
             patch.object(dispatch, "_driver", return_value=(self.root, {"driverInputHash": self.binding["driverInputHash"]})), \
             patch.object(dispatch, "_admit_build") as admission:
            with self.assertRaises(PilotError):
                dispatch.enqueue_and_wait(root=self.root, repo_root=Path.cwd(), bundle=bundle,
                                          identity=self.identity, job="zircon-pilot", build_number=1, maximum_seconds=0.1)
            admission.assert_not_called()
        self.assertFalse(self.storage.path("execution-intents").exists())

    def test_expired_admission_does_not_publish_original_intent(self):
        bundle = self.storage.atomic_write("producer-admission-timeout.zip", self.raw)
        clock = [0.0]
        def admission(*args):
            clock[0] = 1.1
        with patch.object(dispatch, "_owner", return_value=self.owner), \
             patch.object(dispatch, "_driver", return_value=(self.root, {"driverInputHash": self.binding["driverInputHash"]})), \
             patch.object(dispatch, "_admit_build", side_effect=admission), \
             patch.object(dispatch.time, "monotonic", side_effect=lambda: clock[0]):
            with self.assertRaises(PilotError):
                dispatch.enqueue_and_wait(root=self.root, repo_root=Path.cwd(), bundle=bundle,
                                          identity=self.identity, job="zircon-pilot", build_number=1, maximum_seconds=1)
        self.assertFalse(self.storage.path("execution-intents").exists())
        self.assertFalse(self.storage.path("execution-inputs").exists())

    def test_expired_conflict_audit_does_not_stage_or_publish_intent(self):
        clock = [0.0]
        def conflict(*args):
            clock[0] = 1.1
        with patch.object(dispatch, "_active_conflict", side_effect=conflict), \
             patch.object(dispatch.time, "monotonic", side_effect=lambda: clock[0]):
            with self.assertRaises(PilotError):
                dispatch._submit(self.storage, self.owner, self.identity, self.binding, self.raw, deadline=1)
        self.assertFalse(self.storage.path("execution-intents").exists())
        self.assertFalse(self.storage.path("execution-inputs").exists())

    def test_expired_source_verification_preserves_staging_without_publishing_intent(self):
        clock = [0.0]
        original = dispatch.load_snapshot
        def verify(*args, **kwargs):
            value = original(*args, **kwargs)
            clock[0] = 1.1
            return value
        with patch.object(dispatch, "load_snapshot", side_effect=verify), \
             patch.object(dispatch.time, "monotonic", side_effect=lambda: clock[0]):
            with self.assertRaises(PilotError):
                dispatch._submit(self.storage, self.owner, self.identity, self.binding, self.raw, deadline=1)
        self.assertFalse(self.storage.path("execution-intents").exists())
        self.assertTrue(self.storage.path(f"execution-inputs/{dispatch._key(self.identity)}/source.zip").is_file())

    def test_keeper_pending_poll_is_nonblocking_when_producer_holds_lock(self):
        self.submit()
        with dispatch._runtime_lock(self.storage), patch.object(dispatch, "_owner", return_value=self.owner), \
             patch.object(dispatch, "_driver", side_effect=AssertionError("busy poll must not attempt a launch")):
            dispatch.service_execution_intents(self.root, Path.cwd(), self.owner)
        value = json.loads(self.storage.path(f"execution-intents/{dispatch._key(self.identity)}.json").read_bytes())
        self.assertEqual(value["status"], "pending")
    def recovered_fixture(self):
        import subprocess
        import uuid
        from tools.jenkins.pilot.governance_recovery import _state_bytes
        self.owner["stateFile"] = str(Path.cwd() / ".codex/state/jenkins-pilot" / ("tests-dispatch-" + uuid.uuid4().hex + ".json"))
        Path(self.owner["stateFile"]).write_bytes(canonical_json({"root": str(self.root), "owner": self.owner}))
        self.addCleanup(Path(self.owner["stateFile"]).unlink, missing_ok=True)
        self.submit()
        host = subprocess.Popen([sys.executable, "-B", "-c", "pass"])
        birth = popen_process_creation_time(host)
        host.wait(15)
        relative = f"execution-intents/{dispatch._key(self.identity)}.json"
        value = json.loads(self.storage.path(relative).read_bytes())
        value.update(status="complete", launch={"pid": host.pid, "creationTime": birth})
        raw = canonical_json(value)
        self.storage.atomic_write(relative, raw)
        receipt_path = dispatch._receipt_path(self.storage, self.identity).relative_to(self.root).as_posix()
        receipt = {"schemaVersion": 2, "identity": self.identity.to_dict(), "job": "zircon-pilot", "buildNumber": 1,
                   "bundleHash": self.binding["bundleHash"], "toolchain": {"driverInputHash": self.binding["driverInputHash"]},
                   "terminalErrors": [], "processTree": {"terminal": True, "scope": "windows_job", "readersFinished": True,
                   "pipeEOF": True, "pipes": {"stdout": {"eof": True, "closed": True}, "stderr": {"eof": True, "closed": True}}},
                   "nativeJobEvidence": {"errors": [], "records": {"afterWait": {"activeProcesses": 0}, "afterClose": {"activeProcesses": 0}}}}
        receipt_raw = canonical_json(receipt)
        self.storage.atomic_write(receipt_path, receipt_raw)
        owner = dict(self.owner, authorityId="b" * 32, previousAuthorityId=self.owner["authorityId"], stateFile="fixture-state")
        owner["recoveryProof"] = f"authority-history/{owner['authorityId']}/recovery-proof.json"
        checkpoint = {"previousOwner": self.owner, "rootIdentity": self.owner["rootIdentity"], "completedIntents": {relative: {
            "intentSha256": hashlib.sha256(raw).hexdigest(), "previousAuthorityId": self.owner["authorityId"],
            "binding": self.binding, "receiptPath": receipt_path, "receiptSha256": hashlib.sha256(receipt_raw).hexdigest(), "launch": value["launch"]}}}
        state_raw, state_identity = _state_bytes(Path(self.owner["stateFile"]))
        checkpoint.update(previousStatePath=self.owner["stateFile"], previousStateIdentity=state_identity,
                          previousStateSha256=hashlib.sha256(state_raw).hexdigest(),
                          previousAuthoritySha256=hashlib.sha256(canonical_json(self.owner)).hexdigest())
        folder = str(Path(owner["recoveryProof"]).parent).replace("\\", "/")
        self.storage.atomic_write(folder + "/previous-owner.json", canonical_json(self.owner))
        self.storage.atomic_write(folder + "/previous-state.json", state_raw)
        proof = {"schemaVersion": 1, "authorityId": owner["authorityId"], "root": str(self.root), "stateFile": owner["stateFile"],
                 "checkpoint": checkpoint, "checkpointSha256": digest(checkpoint), "recoveredKeeperPid": os.getpid(),
                 "recoveredKeeperCreationTime": owner["owner_process_creation_time"]}
        proof_raw = canonical_json(proof)
        owner["recoveryProofSha256"] = hashlib.sha256(proof_raw).hexdigest()
        self.storage.atomic_write(owner["recoveryProof"], proof_raw)
        return owner, relative, raw, value, receipt_path

    def test_recovery_skips_only_original_completed_binding_without_relaunch(self):
        owner, relative, raw, value, receipt_path = self.recovered_fixture()
        with patch.object(dispatch, "_owner", return_value=owner), patch.object(dispatch, "_driver", side_effect=AssertionError("old work must not launch")):
            dispatch.service_execution_intents(self.root, Path.cwd(), owner)
        other = RequestIdentity("test-session", "fresh-request", "test-attempt", 1, self.identity.input_hash, "python-static")
        new = dispatch._submit(self.storage, owner, other, dict(self.binding, identity=other.to_dict(), buildNumber=2), self.raw)
        self.assertEqual(new["authorityId"], owner["authorityId"])
        self.assertEqual(new["status"], "pending")

    def test_recovery_changed_receipt_is_rejected(self):
        owner, relative, raw, value, receipt_path = self.recovered_fixture()
        self.storage.atomic_write(receipt_path, b"changed historical receipt")
        with self.assertRaises((ValueError, OSError)):
            dispatch._recovered_completed_intent(self.storage, owner, relative, raw, value)

    def test_recovery_uncertain_old_intent_cannot_be_skipped(self):
        owner, relative, raw, value, receipt_path = self.recovered_fixture()
        value["status"] = "reserved"
        with self.assertRaises(PilotError):
            dispatch._recovered_completed_intent(self.storage, owner, relative, canonical_json(value), value)
    def setUp(self):
        self.directory = native_directory()
        self.root = self.directory.__enter__()
        self.addCleanup(self.directory.__exit__, None, None, None)
        self.storage = ManagedStorage(self.root)
        self.storage.ensure_layout()
        bundle, input_hash = sealed_bundle(self.root)
        self.raw = bundle.read_bytes()
        self.identity = RequestIdentity("test-session", "test-request", "test-attempt", 1, input_hash, "python-static")
        self.owner = {"schemaVersion": 2, "authorityId": "a" * 32, "root": str(self.root),
                      "rootIdentity": physical_identity(self.root), "status": "active",
                      "owner_pid": os.getpid(), "owner_process_creation_time": process_creation_time(os.getpid())}
        self.binding = {"identity": self.identity.to_dict(), "job": "zircon-pilot", "buildNumber": 1,
                        "bundleHash": load_snapshot(bundle).bundle_hash, "driverInputHash": "d" * 64}

    def submit(self, binding=None):
        with dispatch._runtime_lock(self.storage):
            return dispatch._submit(self.storage, self.owner, self.identity, binding or self.binding, self.raw)

    def test_same_build_binds_original_reserved_intent_without_rewriting(self):
        original = self.submit()
        relative = f"execution-intents/{dispatch._key(self.identity)}.json"
        original["status"] = "reserved"
        self.storage.atomic_write(relative, canonical_json(original))
        before = self.storage.path(relative).read_bytes()
        self.assertEqual(self.submit(), original)
        self.assertEqual(self.storage.path(relative).read_bytes(), before)

    def test_attempt_cannot_bind_a_different_build(self):
        self.submit()
        with self.assertRaises(PilotError):
            self.submit(dict(self.binding, buildNumber=2))

    def test_uncertain_reservation_is_never_dispatched(self):
        value = self.submit()
        value["status"] = "reserved"
        relative = f"execution-intents/{dispatch._key(self.identity)}.json"
        self.storage.atomic_write(relative, canonical_json(value))
        with patch.object(dispatch, "_owner", return_value=self.owner), patch.object(dispatch, "_driver", side_effect=AssertionError("reserved intent must never reach a launcher")):
            dispatch.service_execution_intents(self.root, Path.cwd(), self.owner)
        self.assertEqual(json.loads(self.storage.path(relative).read_bytes()), value)

    def test_conflicting_attempt_cannot_create_a_second_native_worker_intent(self):
        self.submit()
        other = RequestIdentity("test-session", "other-request", "test-attempt", 1, self.identity.input_hash, "python-static")
        with dispatch._runtime_lock(self.storage), self.assertRaises(PilotError):
            dispatch._submit(self.storage, self.owner, other,
                             dict(self.binding, identity=other.to_dict()), self.raw)

    def test_service_rejects_caller_that_is_not_exact_keeper(self):
        self.submit()
        stale = dict(self.owner, owner_process_creation_time="1")
        with patch.object(dispatch, "_owner", return_value=stale), self.assertRaises(PilotError):
            dispatch.service_execution_intents(self.root, Path.cwd(), stale)

    def test_completed_receipt_does_not_allow_second_worker_while_exact_host_lives(self):
        value = self.submit()
        value.update(status="complete", launch={"pid": os.getpid(), "creationTime": process_creation_time(os.getpid())})
        self.storage.atomic_write(f"execution-intents/{dispatch._key(self.identity)}.json", canonical_json(value))
        receipt = {"schemaVersion": 2, "identity": self.identity.to_dict(), "job": "zircon-pilot", "buildNumber": 1,
                   "bundleHash": self.binding["bundleHash"], "toolchain": {"driverInputHash": self.binding["driverInputHash"]}}
        path = dispatch._receipt_path(self.storage, self.identity)
        self.storage.atomic_write(path.relative_to(self.root).as_posix(), canonical_json(receipt))
        other = RequestIdentity("test-session", "other-request", "test-attempt", 1, self.identity.input_hash, "python-static")
        with self.assertRaises(PilotError):
            dispatch._active_conflict(self.storage, self.owner, dispatch._key(other))

    def test_receipt_with_wrong_sealed_content_is_rejected(self):
        receipt = {"schemaVersion": 2, "identity": self.identity.to_dict(), "job": "zircon-pilot", "buildNumber": 1,
                   "bundleHash": "0" * 64, "toolchain": {"driverInputHash": self.binding["driverInputHash"]}}
        path = dispatch._receipt_path(self.storage, self.identity)
        self.storage.atomic_write(path.relative_to(self.root).as_posix(), canonical_json(receipt))
        with self.assertRaises(PilotError):
            dispatch._read_receipt(self.storage, self.identity, "zircon-pilot", 1, self.binding)

    def test_runtime_lock_pins_its_file_against_replacement(self):
        with dispatch._runtime_lock(self.storage):
            lock = self.storage.path("execution-runtime.lock")
            replacement = self.storage.atomic_write("replacement-lock", b"replacement")
            with self.assertRaises(OSError):
                os.replace(replacement, lock)

    def test_live_foreign_build_parameters_cannot_be_admitted(self):
        from tools.jenkins.pilot.client import JenkinsClient
        responses = [{"offline": False}, {"number": 1, "building": True, "actions": [{"parameters": [
            {"name": "SESSION_ID", "value": "foreign"}, {"name": "REQUEST_ID", "value": "foreign"}]}]}]
        manifest = {"controllerUrl": "http://127.0.0.1:12345", "agentName": "zircon-windows-agent"}
        with patch("tools.jenkins.pilot.jenkins_config.read_credentials", return_value={"username": "fixture", "password": "fixture"}), \
             patch.object(JenkinsClient, "api", side_effect=responses), self.assertRaises(PilotError):
            dispatch._admit_build(self.root, "zircon-pilot", 1, self.binding, manifest)

    def test_execution_only_without_a_launched_intent_is_rejected(self):
        self.submit()
        with patch.object(dispatch, "_owner", return_value=self.owner), self.assertRaises(PilotError):
            dispatch.require_execution_worker(root=self.root, repo_root=Path.cwd(), identity=self.identity,
                                              job="zircon-pilot", build_number=1)

    def test_exact_live_host_cannot_authorize_process_outside_its_native_job(self):
        value = self.submit()
        process, job = create_atomic_kill_on_close_process((sys.executable, "-B", "-c", "pass"),
                                                          cwd=self.root, env=dict(os.environ))
        try:
            value.update(status="started", launch={"pid": process.pid, "creationTime": popen_process_creation_time(process),
                "keeperPid": self.owner["owner_pid"], "keeperCreationTime": self.owner["owner_process_creation_time"],
                "keeperJobHandle": job})
            self.storage.atomic_write(f"execution-intents/{dispatch._key(self.identity)}.json", canonical_json(value))
            with patch.object(dispatch, "_owner", return_value=self.owner), \
                 patch.object(dispatch, "_driver", return_value=(self.root, {"driverInputHash": self.binding["driverInputHash"]})), \
                 self.assertRaisesRegex(PilotError, "outside the exact"):
                dispatch.require_execution_worker(root=self.root, repo_root=Path.cwd(), identity=self.identity,
                                                  job="zircon-pilot", build_number=1)
        finally:
            terminate_and_close_process_job(job)
            process.wait(15)
            process.close()


if __name__ == "__main__":
    unittest.main()
