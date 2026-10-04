from __future__ import annotations

import json
import os
from pathlib import Path
import sys
import tempfile
import unittest

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.processes.guards import abort_evidence
from tools.jenkins.processes.job import NativeJob
from tools.jenkins.processes.registry import ProcessRegistry
from tools.jenkins.resources import Capacity, ResourceManager
from tools.jenkins.state import State


@unittest.skipUnless(os.name == "nt", "requires Windows nested Job Objects")
class ExecutionGuardTests(unittest.TestCase):
    def test_crashed_owner_is_collected_by_outer_job_without_success_receipt(self):
        base = Path.cwd() / ".jenkins/tmp/execution-guard-tests"
        base.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=base) as temp:
            root = Path(temp)
            state = State(root / "state.sqlite3")
            script = ("import os,sys; from pathlib import Path; "
                      "from tools.jenkins.state import State; "
                      "from tools.jenkins.processes.job import NativeJob; "
                      "from tools.jenkins.processes.registry import ProcessRegistry; "
                      "s=State(sys.argv[1]); "
                      "j=NativeJob.launch([sys.executable,'-B','-c','import time; time.sleep(30)'],"
                      "cwd=sys.argv[2],log_dir=Path(sys.argv[2])/'phase'); "
                      "ProcessRegistry(s).register(j,execution_id='exec',owner='exec'); os._exit(9)")
            guard = NativeJob.launch([sys.executable, "-B", "-c", script, str(state.path), str(root)],
                                     cwd=Path.cwd(), env=dict(os.environ, PYTHONPATH=str(Path.cwd())),
                                     log_dir=root / "guard", suspended=True,
                                     job_name="Local\\ZirconJenkinsTest-" + root.name)
            registry = ProcessRegistry(state)
            try:
                record = registry.register(guard, execution_id="exec", owner="exec", role="execution_host_guard")
                state.put("execution_launch", "exec", {"guardNativeJobId": record.native_job_id,
                    "launcherIdentity": guard.identity.to_dict()})
                state.put("execution", "exec", {"status": "running"})
                guard.resume()
                self.assertEqual(9, guard.wait(timeout_seconds=10))
                terminal = registry.record_terminal(guard)
                self.assertTrue(terminal.payload["completeProof"]["complete"])
                phases = [r for r in state.list("native_job") if r["payload"].get("role") == "phase"]
                self.assertEqual(1, len(phases))
                self.assertEqual(record.native_job_id, phases[0]["payload"]["guardNativeJobId"])
                self.assertNotIn("completeProof", phases[0]["payload"])
                state.put("execution_abort", "exec", {"guardNativeJobId": record.native_job_id,
                    "proofDigest": digest(terminal.payload["completeProof"]), "outcome": "failed"})
                evidence = abort_evidence(state, "exec")
                self.assertEqual("failed", evidence["outcome"])
                self.assertFalse(evidence["artifactPublicationAllowed"])
                state.put("resource_reservation", "reservation", {"owner": "exec", "status": "active"})
                state.put("pool_writer_hold", "pool", {"owner": "exec", "status": "active", "reservationKey": "reservation"})
                self.assertTrue(ResourceManager(state, Capacity(1, 1, 1)).release("reservation", writer_key="pool",
                    native_proof_ref=record.native_job_id, execution_id="exec"))
                self.assertEqual([], state.list("validation_receipt"))
                state.put("native_job", "foreign", {"executionId": "exec", "status": "running"})
                with self.assertRaises(JenkinsError):
                    abort_evidence(state, "exec")
            finally:
                guard.close()


if __name__ == "__main__":
    unittest.main()
