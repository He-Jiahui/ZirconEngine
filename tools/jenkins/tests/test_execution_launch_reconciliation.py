from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
from uuid import uuid4

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.deployment.execution_launch_reconciliation import CompletedLaunchBinding, RecordPin, reconcile_completed_launch
from tools.jenkins.processes import NativeJob, ProcessRegistry
from tools.jenkins.state import State


@unittest.skipUnless(os.name == "nt", "requires Windows nested native Job Objects")
class CompletedLaunchReconciliationTests(unittest.TestCase):
    def setUp(self):
        base = Path.cwd() / ".jenkins/tmp/execution-launch-reconciliation-tests"
        base.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=base)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.state = State(self.root / "state.sqlite3")
        self.binding = CompletedLaunchBinding("e", "op", "1", "d" * 64)
        script = """
import os,sys
from pathlib import Path
from tools.jenkins.state import State
from tools.jenkins.processes import NativeJob, ProcessRegistry
from tools.jenkins.processes.identity import current_identity
s=State(sys.argv[1]); root=Path(sys.argv[2]); registry=ProcessRegistry(s)
ident=current_identity(os.getpid(),executable=sys.executable)
host={'executionId':'e','operationId':'op','generation':'1','hostPid':ident.pid,'hostBirthToken':ident.creation_time,'status':'running','phaseRecords':[]}
s.put('execution_host','e',host)
for phase in ('compile','list','test'):
    job=NativeJob.launch([sys.executable,'-B','-c',"print('phase complete')"],cwd=root,log_dir=root/phase)
    try:
        native=registry.register(job,execution_id='e',owner='e')
        job.wait(timeout_seconds=10)
        terminal=registry.record_terminal(job)
        host['phaseRecords'].append({'phase':phase,'nativeJobId':native.native_job_id,'terminalProof':terminal.payload['completeProof']})
        host.update(nativeJobId=native.native_job_id,pid=job.identity.pid,processBirthToken=job.identity.creation_time,terminalProof=terminal.payload['completeProof'])
        s.put('execution_host','e',host)
    finally:
        job.close()
host['status']='terminal';s.put('execution_host','e',host)
"""
        self.guard = NativeJob.launch((sys.executable, "-B", "-c", script, str(self.state.path), str(self.root)),
            cwd=Path.cwd(), env=dict(os.environ, PYTHONPATH=str(Path.cwd())), log_dir=self.root / "guard",
            suspended=True, job_name="Local\\ZirconJenkinsReconciliationTest-" + uuid4().hex)
        self.addCleanup(self.guard.close)
        registry = ProcessRegistry(self.state)
        native = registry.register(self.guard, execution_id="e", owner="e", role="execution_host_guard")
        self.state.put("execution_launch", "e", {"executionId": "e", "operationId": "op", "generation": "1",
            "driverDigest": self.binding.driver_digest, "status": "started", "launcherIdentity": self.guard.identity.to_dict(),
            "guardNativeJobId": native.native_job_id, "nativeHostUUID": None, "historicalNote": "preserve"})
        # Canonical execution rows identify themselves through the record key;
        # they need not duplicate executionId in their payload.
        self.state.put("execution", "e", {"status": "passed", "operationId": "op", "generation": "1",
            "driverDigest": self.binding.driver_digest, "driver": {"runtimeOperationId": "op", "generation": "1", "digest": self.binding.driver_digest}})
        self.guard.resume()
        self.assertEqual(0, self.guard.wait(timeout_seconds=20))
        terminal = registry.record_terminal(self.guard)
        self.assertTrue(terminal.payload["completeProof"]["complete"])
        self.host = self.state.get("execution_host", "e")["payload"]
        launch = self.state.get("execution_launch", "e")
        self.state.put("execution_launch", "e", {**launch["payload"], "nativeHostUUID": self.host["nativeJobId"],
            "nativeHostIdentity": {"hostPid": self.host["hostPid"], "hostBirthToken": self.host["hostBirthToken"]}},
            expected_version=launch["version"])
        self.native_id = native.native_job_id

    def _pins(self):
        return {"binding": self.binding,
                "expected_launch": RecordPin.from_row(self.state.get("execution_launch", "e")),
                "expected_execution": RecordPin.from_row(self.state.get("execution", "e")),
                "expected_host": RecordPin.from_row(self.state.get("execution_host", "e")),
                "expected_native_jobs": {r["key"]: RecordPin.from_row(r) for r in self.state.list("native_job")}}

    def _assert_no_finalization(self, before):
        self.assertEqual(before, self.state.get("execution_launch", "e"))
        self.assertEqual([], self.state.list("execution_launch_reconciliation"))
        self.assertEqual([], self.state.list("execution_abort"))
        self.assertEqual([], self.state.list("artifact"))
        with self.state.connect() as connection:
            self.assertEqual(0, connection.execute("SELECT COUNT(*) FROM events WHERE kind='execution_launch_reconciled'").fetchone()[0])

    def test_real_nested_complete_proofs_repair_only_launch_and_archive_original_bytes(self):
        domains = ("execution", "execution_host", "native_job", "resource_reservation", "pool_writer_hold", "validation_receipt", "artifact")
        self.state.put("resource_reservation", "r", {"status": "active", "owner": "e"})
        self.state.put("pool_writer_hold", "p", {"status": "active", "owner": "e"})
        before = {domain: self.state.list(domain) for domain in domains}
        original = self.state.get("execution_launch", "e")
        result = reconcile_completed_launch(self.state, **self._pins())
        self.assertEqual("exited", result["launch"]["payload"]["status"])
        self.assertEqual(original["version"] + 1, result["launch"]["version"])
        self.assertEqual(0, result["launch"]["payload"]["exitCode"])
        self.assertEqual("preserve", result["launch"]["payload"]["historicalNote"])
        self.assertFalse(result["artifactPublicationAllowed"])
        self.assertEqual(before, {domain: self.state.list(domain) for domain in domains})
        audit = self.state.get("execution_launch_reconciliation", result["auditKey"])["payload"]
        self.assertEqual(digest(audit["archives"]), audit["archivesDigest"])
        self.assertEqual(original, audit["archives"][0]["row"])
        for archive in audit["archives"]:
            self.assertEqual(archive["row"]["payload"], json.loads(archive["rawPayload"]))
            self.assertEqual(digest(archive["row"]["payload"]), archive["payloadDigest"])
            self.assertEqual(hashlib.sha256(archive["rawPayload"].encode("utf-8")).hexdigest(), archive["rawSha256"])
        with self.state.connect() as connection:
            event = connection.execute("SELECT kind,data FROM events WHERE id=?", (result["eventId"],)).fetchone()
        self.assertEqual("execution_launch_reconciled", event[0])
        self.assertEqual(original["version"], json.loads(event[1])["beforeVersion"])

    def test_stale_payload_pin_never_rewrites_a_racing_launch(self):
        pins = self._pins()
        launch = self.state.get("execution_launch", "e")
        self.state.put("execution_launch", "e", {**launch["payload"], "otherWriter": True}, expected_version=launch["version"])
        before = self.state.get("execution_launch", "e")
        with self.assertRaises(JenkinsError) as caught:
            reconcile_completed_launch(self.state, **pins)
        self.assertEqual("state_conflict", caught.exception.code)
        self._assert_no_finalization(before)

    def test_missing_guard_eof_or_phase_proof_cannot_be_inferred_from_process_absence(self):
        for native_id, field, value in ((self.native_id, "stdoutEof", False),
                                       (self.host["nativeJobId"], "stderrEof", False),
                                       (self.native_id, "activeProcesses", 1)):
            with self.subTest(native_id=native_id, field=field):
                row = self.state.get("native_job", native_id)
                original = row["payload"]
                proof = {**original["completeProof"], field: value}
                self.state.put("native_job", native_id, {**original, "completeProof": proof, "terminalProof": proof})
                before = self.state.get("execution_launch", "e")
                with self.assertRaises(JenkinsError) as caught:
                    reconcile_completed_launch(self.state, **self._pins())
                self.assertEqual("execution_launch_reconciliation_unproven", caught.exception.code)
                self._assert_no_finalization(before)
                self.state.put("native_job", native_id, original)

    def test_wrong_host_birth_or_native_owner_fails_without_artifact_publication(self):
        host = self.state.get("execution_host", "e")["payload"]
        self.state.put("execution_host", "e", {**host, "hostBirthToken": "wrong"})
        before = self.state.get("execution_launch", "e")
        with self.assertRaises(JenkinsError):
            reconcile_completed_launch(self.state, **self._pins())
        self._assert_no_finalization(before)
        self.state.put("execution_host", "e", host)
        phase = self.state.get("native_job", host["nativeJobId"])["payload"]
        self.state.put("native_job", host["nativeJobId"], {**phase, "owner": "other"})
        with self.assertRaises(JenkinsError):
            reconcile_completed_launch(self.state, **self._pins())
        self._assert_no_finalization(before)

    def test_audit_or_event_failure_rolls_back_the_entire_repair(self):
        before = self.state.get("execution_launch", "e")
        event = self.state.event
        def fail_reconciliation_event(kind, data, **kwargs):
            if kind == "execution_launch_reconciled":
                raise JenkinsError("state_busy", "test audit event failure", retryable=True)
            return event(kind, data, **kwargs)
        with patch.object(self.state, "event", side_effect=fail_reconciliation_event):
            with self.assertRaises(JenkinsError) as caught:
                reconcile_completed_launch(self.state, **self._pins())
        self.assertEqual("state_busy", caught.exception.code)
        self._assert_no_finalization(before)


if __name__ == "__main__":
    unittest.main()
