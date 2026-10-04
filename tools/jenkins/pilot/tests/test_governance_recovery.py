"""Recovery preserves old authority while refusing live or uncertain identities."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import uuid

from tools.jenkins.pilot.contracts import canonical_json, PilotError
from tools.jenkins.pilot import governance_recovery as recovery
from tools.jenkins.pilot.governance import OWNER_FILE, OWNER_SCHEMA, _write_state, allocate_storage
from tools.jenkins.pilot.native.paths import WorkerStorage
from tools.jenkins.pilot.native.process_identity import popen_process_creation_time, process_creation_time
from tools.jenkins.pilot.storage import physical_identity


class GovernanceRecoveryTests(unittest.TestCase):
    def completed_intent(self, root, owner):
        from tools.jenkins.pilot.contracts import RequestIdentity, digest
        from tools.jenkins.pilot.execution_dispatch import _key
        identity = RequestIdentity("recovery-session", "recovery-request", "recovery-attempt", 1, "1"*64, "python-static")
        binding = {"identity": identity.to_dict(), "job": "zircon-pilot", "buildNumber": 1,
                   "bundleHash": "2"*64, "driverInputHash": "1"*64}
        value = {"schemaVersion": 2, "authorityId": owner["authorityId"], "rootIdentity": owner["rootIdentity"],
                 "binding": binding, "fingerprint": digest(binding), "status": "complete",
                 "launch": {"pid": owner["owner_pid"], "creationTime": owner["owner_process_creation_time"]}}
        receipt = {"schemaVersion": 2, "identity": identity.to_dict(), "job": "zircon-pilot", "buildNumber": 1,
                   "bundleHash": binding["bundleHash"], "toolchain": {"driverInputHash": binding["driverInputHash"]},
                   "terminalErrors": [], "processTree": {"terminal": True, "scope": "windows_job", "readersFinished": True,
                   "pipeEOF": True, "pipes": {"stdout": {"eof": True, "closed": True}, "stderr": {"eof": True, "closed": True}}},
                   "nativeJobEvidence": {"errors": [], "records": {"afterWait": {"activeProcesses": 0}, "afterClose": {"activeProcesses": 0}}}}
        relative = f"execution-intents/{_key(identity)}.json"
        with WorkerStorage(root, create=False) as backend:
            backend.write_bytes(relative, canonical_json(value))
            backend.write_bytes(f"runs/{identity.session_id}/{identity.request_id}/{identity.attempt_id}-g1/receipt.json", canonical_json(receipt))
        return relative, value

    def recover_fixture_epoch(self, repo, root, state, owner_raw):
        checkpoint = recovery.inspect_recovery(repo, root, expected_authority_sha256=hashlib.sha256(owner_raw).hexdigest())
        pending = {"schemaVersion": 1, "authorityId": uuid.uuid4().hex, "root": str(root), "stateFile": str(state),
                   "checkpoint": checkpoint, "checkpointSha256": hashlib.sha256(canonical_json(checkpoint)).hexdigest()}
        state.with_suffix(".pending").write_bytes(canonical_json(pending))
        old = json.loads(owner_raw)
        with patch("tools.jenkins.pilot.governance._serve_owner"), \
             patch.object(recovery.os, "getpid", return_value=old["owner_pid"]), \
             patch.object(recovery.native_process, "process_creation_time", return_value=old["owner_process_creation_time"]):
            recovery.keep_recovered_storage(repo, state)
        return (root / OWNER_FILE).read_bytes()

    def test_two_recovery_epochs_preserve_original_completed_intent_and_ancestry(self):
        repo, root, old_state, new_state, owner_raw = self.fixture()
        relative, intent = self.completed_intent(root, json.loads(owner_raw))
        original_state, original_intent = old_state.read_bytes(), (root / relative).read_bytes()
        with self.guards():
            first = self.recover_fixture_epoch(repo, root, new_state, owner_raw)
            first_state = new_state.read_bytes()
            second_state = new_state.with_name(new_state.stem + "-second.json")
            second = self.recover_fixture_epoch(repo, root, second_state, first)
            checkpoint = recovery.inspect_recovery(repo, root, expected_authority_sha256=hashlib.sha256(second).hexdigest())
        self.assertEqual(checkpoint["completedIntents"][relative]["previousAuthorityId"], intent["authorityId"])
        self.assertEqual(old_state.read_bytes(), original_state)
        self.assertEqual(new_state.read_bytes(), first_state)
        self.assertEqual((root / relative).read_bytes(), original_intent)
        with WorkerStorage(root, create=False) as backend:
            self.assertEqual(len(recovery.preservation_chain(backend, json.loads(second))), 2)
        # Same bytes at a replaced ancestral metadata inode still break preservation.
        replacement = old_state.with_suffix(".replacement")
        replacement.write_bytes(original_state)
        os.replace(replacement, old_state)
        with self.guards(), self.assertRaises(PilotError):
            recovery.inspect_recovery(repo, root, expected_authority_sha256=hashlib.sha256(second).hexdigest())
        old_state.write_bytes(b"{}")
        with self.guards(), self.assertRaises(PilotError):
            recovery.inspect_recovery(repo, root, expected_authority_sha256=hashlib.sha256(second).hexdigest())

    def test_recovery_ancestor_proof_intent_pending_and_live_identity_refuse(self):
        repo, root, _, new_state, owner_raw = self.fixture()
        relative, value = self.completed_intent(root, json.loads(owner_raw))
        with self.guards():
            current_raw = self.recover_fixture_epoch(repo, root, new_state, owner_raw)
            expected = hashlib.sha256(current_raw).hexdigest()
            current = json.loads(current_raw)
            for status in ("pending", "reserved", "started"):
                value["status"] = status
                (root / relative).write_bytes(canonical_json(value))
                with self.assertRaises(PilotError):
                    recovery.inspect_recovery(repo, root, expected_authority_sha256=expected)
            value["status"] = "complete"
            (root / relative).write_bytes(canonical_json(value))
            real_dead = recovery._require_dead
            probes = []
            def ancestor_uncertain(pid, birth):
                probes.append((pid, birth))
                if len(probes) > 1:
                    raise PilotError("exact ancestor death is uncertain")
                return real_dead(pid, birth)
            with patch.object(recovery, "_require_dead", side_effect=ancestor_uncertain):
                with self.assertRaises(PilotError):
                    recovery.inspect_recovery(repo, root, expected_authority_sha256=expected)
            self.assertEqual(len(probes), 2)
            value["binding"]["buildNumber"] = 2
            (root / relative).write_bytes(canonical_json(value))
            with self.assertRaises(PilotError):
                recovery.inspect_recovery(repo, root, expected_authority_sha256=expected)
            (root / current["recoveryProof"]).write_bytes(b"{}")
            with self.assertRaises((PilotError, ValueError, OSError)):
                recovery.inspect_recovery(repo, root, expected_authority_sha256=expected)

    def test_recovery_requires_unchanged_historical_record_in_every_epoch(self):
        from tools.jenkins.pilot.execution_dispatch import _recovered_completed_intent
        from tools.jenkins.pilot.storage import ManagedStorage
        repo, root, _, new_state, owner_raw = self.fixture()
        relative, value = self.completed_intent(root, json.loads(owner_raw))
        with self.guards():
            first = self.recover_fixture_epoch(repo, root, new_state, owner_raw)
            second = self.recover_fixture_epoch(repo, root, new_state.with_name(new_state.stem + "-second.json"), first)
        owner = json.loads(second)
        proof_path = root / owner["recoveryProof"]
        proof = json.loads(proof_path.read_bytes())
        del proof["checkpoint"]["completedIntents"][relative]
        proof["checkpointSha256"] = hashlib.sha256(canonical_json(proof["checkpoint"])).hexdigest()
        proof_raw = canonical_json(proof)
        proof_path.write_bytes(proof_raw)
        owner["recoveryProofSha256"] = hashlib.sha256(proof_raw).hexdigest()
        with self.assertRaises(PilotError):
            _recovered_completed_intent(ManagedStorage(root), owner, relative, canonical_json(value), value)

    def fixture(self):
        key = uuid.uuid4().hex
        root = Path(r"D:\cargo-targets\zircon-jenkins") / ("jenkins-pilot-recovery-test-" + key)
        repo = Path.cwd()
        state = repo / ".codex/state/jenkins-pilot" / ("tests-recovery-old-" + key + ".json")
        new_state = state.with_name("tests-recovery-new-" + key + ".json")
        process = subprocess.Popen([sys.executable, "-B", "-c", "import time;time.sleep(0.2)"], creationflags=subprocess.CREATE_NO_WINDOW)
        birth = popen_process_creation_time(process)
        try:
            process.wait(timeout=60)
        except BaseException:
            process.kill()
            process.wait(timeout=15)
            raise
        with WorkerStorage(root) as backend:
            owner = {"schemaVersion": OWNER_SCHEMA, "authorityId": key, "root": str(root),
                     "rootIdentity": physical_identity(root), "stateFile": str(state),
                     "owner_pid": process.pid, "owner_process_creation_time": birth, "status": "active"}
            raw = canonical_json(owner)
            backend.write_bytes(OWNER_FILE, raw)
            backend.write_bytes("pilot-manifest.json", canonical_json({"driverBundle": str(root / "fixture.zip"),
                "driverInputHash": "1"*64, "driverBundleHash": "2"*64, "driverRoot": str(root / "driver-fixture")}))
            backend.write_bytes("asset-source-manifest.json", b"{}")
            backend.write_bytes("process-state.json", b"{}")
            backend.write_bytes("execution-runtime.lock", b"0")
        state.parent.mkdir(parents=True, exist_ok=True)
        _write_state(state, {"schemaVersion": OWNER_SCHEMA, "root": str(root), "owner": owner})
        return repo, root, state, new_state, raw

    def guards(self):
        from contextlib import ExitStack
        stack = ExitStack()
        stack.enter_context(patch("tools.jenkins.pilot.assets.verify_prepared_assets", return_value="3"*64))
        stack.enter_context(patch("tools.jenkins.pilot.snapshot.load_snapshot", return_value=SimpleNamespace(bundle_hash="2"*64)))
        stack.enter_context(patch("tools.jenkins.pilot.snapshot.verify_materialized", return_value="1"*64))
        return stack

    def test_dead_identity_checkpoint_and_unique_epoch_preserve_original_metadata(self):
        repo, root, old_state, new_state, original_owner = self.fixture()
        original_state = old_state.read_bytes()
        expected = hashlib.sha256(original_owner).hexdigest()
        with self.guards():
            checkpoint = recovery.inspect_recovery(repo, root, expected_authority_sha256=expected)
            pending = {"schemaVersion": 1, "authorityId": uuid.uuid4().hex, "root": str(root),
                       "stateFile": str(new_state), "checkpoint": checkpoint,
                       "checkpointSha256": hashlib.sha256(canonical_json(checkpoint)).hexdigest()}
            new_state.with_suffix(".pending").write_bytes(canonical_json(pending))
            with patch("tools.jenkins.pilot.governance._serve_owner"):
                recovery.keep_recovered_storage(repo, new_state)
        current = json.loads((root / OWNER_FILE).read_bytes())
        self.assertNotEqual(current["authorityId"], json.loads(original_owner)["authorityId"])
        self.assertEqual(old_state.read_bytes(), original_state)
        proof_folder = root / Path(current["recoveryProof"]).parent
        self.assertEqual((proof_folder / "previous-owner.json").read_bytes(), original_owner)
        self.assertEqual((proof_folder / "previous-state.json").read_bytes(), original_state)
        self.assertEqual(hashlib.sha256((root / current["recoveryProof"]).read_bytes()).hexdigest(), current["recoveryProofSha256"])
        self.assertEqual((root / "execution-runtime.lock").read_bytes(), b"0")
        self.assertTrue(new_state.with_suffix(".pending").exists())
        with self.assertRaisesRegex(PilotError, "superseded authority"):
            allocate_storage(repo, old_state, "jenkins-pilot-test")
        self.assertEqual(allocate_storage(repo, new_state, "jenkins-pilot-test")["owner"], current)
        self.assertEqual(old_state.read_bytes(), original_state)

    def test_live_and_ambiguous_native_identity_refuse_recovery(self):
        with self.assertRaises(PilotError):
            recovery._require_dead(os.getpid(), process_creation_time(os.getpid()))
        with patch("tools.jenkins.pilot.governance_recovery.native_process._open", side_effect=OSError("access denied")):
            with self.assertRaises(OSError):
                recovery._require_dead(123, "123")

    def test_changed_authority_state_and_hardlinked_metadata_refuse_checkpoint(self):
        repo, root, state, _, owner_raw = self.fixture()
        expected = hashlib.sha256(owner_raw).hexdigest()
        with self.guards():
            with self.assertRaises(PilotError):
                recovery.inspect_recovery(repo, root, expected_authority_sha256="0"*64)
            original = state.read_bytes()
            state.write_bytes(b"{}")
            with self.assertRaises(PilotError):
                recovery.inspect_recovery(repo, root, expected_authority_sha256=expected)
            state.write_bytes(original)
            os.link(state, state.with_suffix(".alias"))
            with self.assertRaises(PilotError):
                recovery.inspect_recovery(repo, root, expected_authority_sha256=expected)

    def test_pending_old_intent_and_busy_handoff_lock_never_publish_new_authority(self):
        repo, root, state, new_state, owner_raw = self.fixture()
        expected = hashlib.sha256(owner_raw).hexdigest()
        with self.guards():
            checkpoint = recovery.inspect_recovery(repo, root, expected_authority_sha256=expected)
            pending = {"schemaVersion": 1, "authorityId": uuid.uuid4().hex, "root": str(root),
                       "stateFile": str(new_state), "checkpoint": checkpoint,
                       "checkpointSha256": hashlib.sha256(canonical_json(checkpoint)).hexdigest()}
            new_state.with_suffix(".pending").write_bytes(canonical_json(pending))
            from tools.jenkins.pilot.execution_dispatch import _runtime_lock
            from tools.jenkins.pilot.storage import ManagedStorage
            with _runtime_lock(ManagedStorage(root)):
                with self.assertRaises(PilotError):
                    recovery.keep_recovered_storage(repo, new_state)
            with WorkerStorage(root) as backend:
                backend.write_bytes("execution-intents/unknown.json", canonical_json({"status": "pending"}))
            with self.assertRaises(PilotError):
                recovery.inspect_recovery(repo, root, expected_authority_sha256=expected)
        self.assertFalse(new_state.exists())
        self.assertEqual((root / OWNER_FILE).read_bytes(), owner_raw)

    def test_fatal_keeper_evidence_is_written_without_overwriting_existing_state(self):
        from tools.jenkins.pilot import governance
        repo, root, state, _, owner_raw = self.fixture()
        original_state = state.read_bytes()
        with patch.object(governance, "keep_storage", side_effect=RuntimeError("test-fatal-evidence")), \
             patch.object(sys, "argv", ["governance", "--repo-root", str(repo), "--state-file", str(state),
                                        "--prefix", "jenkins-pilot-test", "--root", str(root), "--authority-id", "test"]):
            with self.assertRaises(RuntimeError):
                governance.main()
        receipt = json.loads(state.with_suffix(".fatal.json").read_bytes())
        self.assertEqual(receipt["errorType"], "RuntimeError")
        self.assertEqual(state.read_bytes(), original_state)
        self.assertEqual((root / OWNER_FILE).read_bytes(), owner_raw)

    def test_terminal_receipt_rejects_live_job_missing_eof_or_uncertain_scope(self):
        receipt = {"processTree": {"scope": "windows_job", "terminal": True, "pipeEOF": True,
                                  "readersFinished": True, "pipes": {"stdout": {"eof": True, "closed": True},
                                                                     "stderr": {"eof": True, "closed": True}}},
                   "terminalErrors": [],
                   "nativeJobEvidence": {"errors": [], "records": {"beforeWait": {"activeProcesses": 1},
                                               "afterWait": {"activeProcesses": 0}, "afterClose": {"activeProcesses": 0}}}}
        recovery._terminal_receipt(receipt)
        stderr = receipt["processTree"]["pipes"].pop("stderr")
        with self.assertRaises(PilotError):
            recovery._terminal_receipt(receipt)
        receipt["processTree"]["pipes"]["stderr"] = stderr
        stderr["error"] = None
        with self.assertRaises(PilotError):
            recovery._terminal_receipt(receipt)
        del stderr["error"]
        receipt["nativeJobEvidence"]["records"]["afterClose"]["activeProcesses"] = 1
        with self.assertRaises(PilotError):
            recovery._terminal_receipt(receipt)
        receipt["nativeJobEvidence"]["records"]["afterClose"]["activeProcesses"] = 0
        receipt["processTree"]["pipeEOF"] = False
        with self.assertRaises(PilotError):
            recovery._terminal_receipt(receipt)


if __name__ == "__main__":
    unittest.main()
