from copy import deepcopy
import hashlib
import unittest

from tools.jenkins.pilot.contracts import PilotError, RequestIdentity, digest
from tools.jenkins.pilot.receipts import verify_receipt


class ReceiptTests(unittest.TestCase):
    def setUp(self):
        self.identity = RequestIdentity("session", "request", "attempt", 1, "a" * 64, "python-static")
        self.command = ["python", "-B", "static.py", "input"]
        self.receipt = {
            "schemaVersion": 2, "acceptance": "jenkins-command-evidence-v2", "identity": self.identity.to_dict(),
            "job": "pilot", "buildNumber": 1, "bundleHash": "b" * 64,
            "command": self.command, "commandHash": digest(self.command),
            "sourceBefore": self.identity.input_hash, "sourceAfter": self.identity.input_hash,
            "exitCode": 0, "outcome": "passed", "toolchain": {"python": "3.14.0", "driverInputHash": "c" * 64},
            "processTree": {"terminal": True, "scope": "windows_job", "pid": 1, "creationTime": "134353440704803594", "readersFinished": True, "pipeEOF": True, "pipes": {"stdout": {"eof": True}, "stderr": {"eof": True}}},
            "terminalErrors": [], "nativeJobEvidence": {"access": "JOB_OBJECT_QUERY", "errors": [], "records": {"afterClose": {"activeProcesses": 0}}},
            "timings": {"prepareSeconds": 0.1, "runSeconds": 0.2},
            "artifacts": [{"path": "output.log", "sha256": hashlib.sha256(b"x").hexdigest(), "size": 1}],
        }

    def verify(self, receipt):
        return verify_receipt(receipt, identity=self.identity, job="pilot", build_number=1,
                              bundle_hash="b" * 64, command=self.command, artifacts={"output.log": b"x"}, driver_input_hash="c" * 64)

    def test_success_accepts_only_bound_sealed_input(self):
        result = self.verify(self.receipt)
        self.assertEqual(result["acceptance"], "jenkins-validation")
        self.assertTrue(result["accepted"])
        self.assertFalse(result["milestoneAcceptance"])
        self.assertFalse(result["migrationAcceptance"])

    def test_missing_changed_or_stale_proof_cannot_pass(self):
        mutations = [
            ("acceptance", "PASS"), ("buildNumber", 2), ("bundleHash", "d" * 64),
            ("sourceAfter", "d" * 64), ("exitCode", 1), ("commandHash", "e" * 64),
            ("outcome", "cancelled"), ("processTree", {"terminal": False, "scope": "windows_job"}),
            ("toolchain", {}), ("artifacts", []),
        ]
        for key, value in mutations:
            altered = deepcopy(self.receipt)
            altered[key] = value
            with self.subTest(key=key), self.assertRaises(PilotError):
                self.verify(altered)
        old = deepcopy(self.receipt)
        old["identity"]["generation"] = 2
        with self.assertRaises(PilotError):
            self.verify(old)
        absent = deepcopy(self.receipt)
        del absent["processTree"]
        with self.assertRaises(PilotError):
            self.verify(absent)

    def test_artifact_bytes_are_required(self):
        with self.assertRaises(PilotError):
            verify_receipt(self.receipt, identity=self.identity, job="pilot", build_number=1,
                           bundle_hash="b" * 64, command=self.command, artifacts={"output.log": b"y"}, driver_input_hash="c" * 64)

    def test_native_unknown_errors_and_incomplete_output_cannot_pass(self):
        mutations = [
            ("schemaVersion", 1),
            ("terminalErrors", [{"stage": "output", "message": "disk full"}]),
            ("nativeJobEvidence", {"access": "JOB_OBJECT_QUERY", "errors": [],
                "records": {"afterClose": {"activeProcesses": False}}}),
            ("nativeJobEvidence", {"access": "JOB_OBJECT_QUERY", "errors": [],
                "records": {"afterClose": {"activeProcesses": 1}}}),
        ]
        for key, value in mutations:
            altered = deepcopy(self.receipt)
            altered[key] = value
            with self.subTest(key=key, value=value), self.assertRaises(PilotError):
                self.verify(altered)
        for key in ("pipeEOF", "readersFinished"):
            altered = deepcopy(self.receipt)
            altered["processTree"][key] = False
            with self.subTest(key=key), self.assertRaises(PilotError):
                self.verify(altered)
        altered = deepcopy(self.receipt)
        altered["toolchain"]["driverInputHash"] = "d" * 64
        with self.assertRaises(PilotError):
            self.verify(altered)


if __name__ == "__main__":
    unittest.main()
