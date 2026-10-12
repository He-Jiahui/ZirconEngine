from __future__ import annotations

import json
import ctypes
import os
from pathlib import Path
import sys
import tempfile
import time
import unittest
from unittest.mock import Mock

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.processes.guards import abort_evidence, guard_phases_complete
from tools.jenkins.processes.job import NativeJob, TerminationProof
from tools.jenkins.processes.identity import ProcessIdentity
from tools.jenkins.processes.registry import ProcessRegistry
from tools.jenkins.resources import Capacity, ResourceManager
from tools.jenkins.state import State


@unittest.skipUnless(os.name == "nt", "requires Windows nested Job Objects")
class ExecutionGuardTests(unittest.TestCase):
    def test_crashed_owner_keeps_holds_until_nested_phase_proof_exists(self):
        base = Path.cwd() / ".jenkins/tmp/execution-guard-tests"
        base.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=base) as temp:
            root = Path(temp)
            state = State(root / "state.sqlite3")
            script = ("import os,sys,time; from pathlib import Path; "
                      "from tools.jenkins.state import State; "
                      "from tools.jenkins.processes.job import NativeJob; "
                      "from tools.jenkins.processes.registry import ProcessRegistry; "
                      "s=State(sys.argv[1]); "
                      "j=NativeJob.launch([sys.executable,'-B','-c','import time; time.sleep(30)'],"
                      "cwd=sys.argv[2],log_dir=Path(sys.argv[2])/'phase'); "
                      "ProcessRegistry(s).register(j,execution_id='exec',owner='exec'); "
                      "Path(sys.argv[2],'ready').write_text('ready'); "
                      "\nwhile not Path(sys.argv[2],'go').exists(): time.sleep(.01)"
                      "\nos._exit(9)")
            guard = NativeJob.launch([sys.executable, "-B", "-c", script, str(state.path), str(root)],
                                     cwd=Path.cwd(), env=dict(os.environ, PYTHONPATH=str(Path.cwd())),
                                     log_dir=root / "guard", suspended=True,
                                     job_name="Local\\ZirconJenkinsTest-" + root.name)
            registry = ProcessRegistry(state)
            phase_handle = None
            try:
                record = registry.register(guard, execution_id="exec", owner="exec", role="execution_host_guard")
                state.put("execution_launch", "exec", {"guardNativeJobId": record.native_job_id,
                    "launcherIdentity": guard.identity.to_dict()})
                state.put("execution", "exec", {"status": "running"})
                guard.resume()
                deadline = time.monotonic() + 10
                while not (root / "ready").exists() and time.monotonic() < deadline:
                    time.sleep(.01)
                self.assertTrue((root / "ready").is_file())
                phases = [r for r in state.list("native_job") if r["payload"].get("role") == "phase"]
                self.assertEqual(1, len(phases))
                from tools.jenkins.processes import native
                api = native._kernel()
                api.OpenProcess.argtypes = [ctypes.c_uint32, ctypes.c_bool, ctypes.c_uint32]
                api.OpenProcess.restype = ctypes.c_void_p
                phase_handle = api.OpenProcess(0x1000 | 0x100000, False, phases[0]["payload"]["pid"])
                self.assertTrue(phase_handle)
                self.assertEqual(phases[0]["payload"]["birthToken"], native._birth(api, phase_handle))
                (root / "go").write_text("go")
                self.assertEqual(9, guard.wait(timeout_seconds=10))
                terminal = registry.record_terminal(guard)
                self.assertEqual("termination_unproven", terminal.payload["status"])
                self.assertIsNone(terminal.payload["completeProof"])
                self.assertFalse(terminal.payload["terminalProof"]["childrenGone"])
                self.assertEqual(record.native_job_id, phases[0]["payload"]["guardNativeJobId"])
                self.assertNotIn("completeProof", phases[0]["payload"])
                state.put("execution_abort", "exec", {"guardNativeJobId": record.native_job_id,
                    "proofDigest": digest(terminal.payload["terminalProof"]), "outcome": "failed"})
                with self.assertRaises(JenkinsError):
                    abort_evidence(state, "exec")
                state.put("resource_reservation", "reservation", {"owner": "exec", "status": "active"})
                state.put("pool_writer_hold", "pool", {"owner": "exec", "status": "active", "reservationKey": "reservation"})
                with self.assertRaises(JenkinsError):
                    ResourceManager(state, Capacity(1, 1, 1)).release("reservation", writer_key="pool",
                        native_proof_ref=record.native_job_id, execution_id="exec")
                self.assertEqual("active", state.get("resource_reservation", "reservation")["payload"]["status"])
                self.assertEqual("active", state.get("pool_writer_hold", "pool")["payload"]["status"])
                self.assertEqual([], state.list("validation_receipt"))
            finally:
                guard.close()
                if phase_handle:
                    # This test owns the pre-crash handle. Waiting it is only
                    # fixture cleanup and cannot grant production tree proof.
                    try:
                        api.WaitForSingleObject.argtypes = [ctypes.c_void_p, ctypes.c_uint32]
                        api.WaitForSingleObject.restype = ctypes.c_uint32
                        self.assertEqual(0, api.WaitForSingleObject(phase_handle, 10000))
                    finally:
                        api.CloseHandle(phase_handle)


class GuardPhaseProofTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.state = State(Path(self.temp.name) / "state.sqlite3")
        self.identity = ProcessIdentity(401, "123456789", "python.exe", "c" * 64)
        self.proof = TerminationProof(self.identity, True, 9, 0, True, True, True).to_dict()
        self.guard = self.state.put("native_job", "guard", {"nativeJobId": "guard", "executionId": "exec",
            "owner": "exec", "role": "execution_host_guard", "status": "terminal",
            "completeProof": self.proof, "terminalProof": self.proof})
        self.phase_identity = {"pid": 402, "creationTime": "123456790", "executable": "cargo.exe", "commandDigest": "d" * 64}
        self.phase_proof = {**self.proof, "identity": self.phase_identity, "processExitCode": 1}
        self.phase = {"nativeJobId": "phase", "executionId": "exec", "owner": "exec", "role": "phase",
            "guardNativeJobId": "guard", "pid": 402, "birthToken": "123456790", "executable": "cargo.exe",
            "status": "terminal", "completeProof": self.phase_proof, "terminalProof": self.phase_proof}
        self.state.put("native_job", "phase", self.phase)
        self.state.put("execution_launch", "exec", {"guardNativeJobId": "guard"})
        self.state.put("execution_abort", "exec", {"guardNativeJobId": "guard", "proofDigest": digest(self.proof)})
        self.state.put("resource_reservation", "reservation", {"owner": "exec", "status": "active"})
        self.state.put("pool_writer_hold", "pool", {"owner": "exec", "status": "active", "reservationKey": "reservation"})

    def test_complete_phase_tree_allows_failure_only_release(self):
        self.assertTrue(guard_phases_complete(self.state, self.guard))
        evidence = abort_evidence(self.state, "exec")
        self.assertEqual("failed", evidence["outcome"])
        self.assertFalse(evidence["artifactPublicationAllowed"])
        self.assertTrue(ResourceManager(self.state, Capacity(1, 1, 1)).release("reservation", writer_key="pool",
            native_proof_ref="guard", execution_id="exec"))

    def test_legacy_outer_proof_cannot_release_unproven_phase_or_foreign_binding(self):
        variants = [{"status": "running", "completeProof": None}, {"owner": "foreign"},
                    {"executionId": "foreign"}, {"guardNativeJobId": "foreign"}, {"role": "foreign"},
                    {"completeProof": {"complete": True}},
                    {"completeProof": {**self.phase_proof, "identity": self.phase_identity | {"pid": 403}}},
                    {"completeProof": {**self.phase_proof, "childrenGone": False}},
                    {"completeProof": {**self.phase_proof, "stdoutEof": False}},
                    {"completeProof": {**self.phase_proof, "stderrEof": False}},
                    {"completeProof": {**self.phase_proof, "activeProcesses": 1}},
                    {"completeProof": {**self.phase_proof, "processExitCode": None}}]
        for changes in variants:
            with self.subTest(changes=changes):
                self.state.put("native_job", "phase", self.phase | changes)
                self.assertFalse(guard_phases_complete(self.state, self.guard))
                with self.assertRaises(JenkinsError):
                    abort_evidence(self.state, "exec")
                with self.assertRaises(JenkinsError):
                    ResourceManager(self.state, Capacity(1, 1, 1)).release("reservation", writer_key="pool",
                        native_proof_ref="guard", execution_id="exec")
                self.assertEqual("active", self.state.get("resource_reservation", "reservation")["payload"]["status"])
                self.assertEqual("active", self.state.get("pool_writer_hold", "pool")["payload"]["status"])

    def test_registry_downgrades_guard_in_same_transaction(self):
        registry = ProcessRegistry(self.state)
        job = Mock()
        job.proof.return_value = TerminationProof(self.identity, True, 9, 0, True, True, True)
        registry._jobs["guard"] = job
        self.state.put("native_job", "phase", self.phase | {"status": "running", "completeProof": None})
        result = registry.record_terminal(job)
        self.assertEqual("termination_unproven", result.payload["status"])
        self.assertIsNone(result.payload["completeProof"])
        self.assertFalse(result.payload["terminalProof"]["childrenGone"])


if __name__ == "__main__":
    unittest.main()
