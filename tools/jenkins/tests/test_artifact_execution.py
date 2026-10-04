import hashlib
import tempfile
import unittest
from pathlib import Path

from tools.jenkins.artifacts import (attach_execution_outputs,
    find_reusable_outputs, publish_execution_outputs, release_execution_outputs)
from tools.jenkins.contracts import digest
from tools.jenkins.state import State


class ArtifactExecutionTests(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp())
        self.build = self.root / "build"
        self.output = self.build / "run" / "target"
        self.output.mkdir(parents=True)
        (self.output / "zircon.exe").write_bytes(b"stable binary")
        self.state = State(self.root / "state.db")
        self.execution = "exec-artifact-1"
        self.key = "prep-full-input-1"
        self.identity = {k: hashlib.sha256(k.encode()).hexdigest()
                         for k in ("sourceDigest", "recipeDigest", "coverageDigest", "driverDigest")}
        self.state.put("execution", self.execution,
                       {"status": "validated", "executionKey": self.key,
                        "identity": self.identity})
        native = "native-artifact-1"
        self.state.put("native_job", native,
                       {"nativeJobId": native, "status": "terminal", "birthToken": "b1",
                        "completeProof": {"complete": True, "processExitCode": 0,
                                           "stdoutEof": True, "stderrEof": True,
                                           "childrenGone": True}})
        path = "zircon.exe"
        entry = {"path": path, "sha256": hashlib.sha256(b"stable binary").hexdigest(),
                 "size": len(b"stable binary")}
        closure = {"root": str(self.output), "files": [entry], "digest": digest([entry])}
        receipt = {"schemaVersion": 1, "kind": "jenkins.managed.producer-receipt",
                   "status": "passed", "executionId": self.execution, "stage": "build",
                   **self.identity, "producer": {"managed": True, "name": "zircon-jenkins"},
                   "terminalProof": {"nativeJobId": native, "birthToken": "b1"},
                   "outputClosure": closure}
        self.state.put("validation_attempt", self.execution,
                       {**receipt, "status": "passed"})
        self.state.put("validation_receipt", self.execution, receipt)
        self.receipts = {"build": self.execution}

    def test_publish_attach_release_and_exact_reuse(self):
        bundle = publish_execution_outputs(self.state, self.build, self.execution,
                                           self.key, self.receipts)
        self.assertEqual(bundle["status"], "published")
        self.assertEqual(bundle["files"][0]["path"], "zircon.exe")
        consumer = attach_execution_outputs(self.state, self.build, self.execution, "consumer-a")
        self.assertEqual(len(consumer["refs"]), 1)
        self.assertEqual(find_reusable_outputs(self.state, execution_key=self.key,
                                               identity=self.identity)["bundleDigest"],
                         bundle["bundleDigest"])
        released = release_execution_outputs(self.state, self.build, self.execution, "consumer-a")
        self.assertEqual(released["released"], 1)
        self.assertEqual(attach_execution_outputs(self.state, self.build, self.execution,
                                                  "consumer-b")["refs"][0]["owner"], "consumer-b")

    def test_corrupt_cas_is_rejected_on_attach(self):
        publish_execution_outputs(self.state, self.build, self.execution, self.key, self.receipts)
        obj = next((self.build / "zircon-jenkins" / "artifacts").rglob("objects/*"))
        obj.write_bytes(b"tampered")
        with self.assertRaises(Exception) as error:
            attach_execution_outputs(self.state, self.build, self.execution, "consumer")
        self.assertEqual(getattr(error.exception, "code", ""), "artifact_digest_mismatch")

    def test_corrupt_cas_is_rejected_by_reuse_query(self):
        bundle = publish_execution_outputs(self.state, self.build, self.execution,
                                           self.key, self.receipts)
        obj = next((self.build / "zircon-jenkins" / "artifacts").rglob("objects/*"))
        obj.write_bytes(b"tampered")
        with self.assertRaises(Exception) as error:
            find_reusable_outputs(self.state, execution_key=self.key,
                                  identity=self.identity, build_root=self.build)
        self.assertEqual(getattr(error.exception, "code", ""), "artifact_digest_mismatch")

    def test_structured_execution_key_matches_its_persisted_identity(self):
        structured = {"source": "sealed-source", "recipe": "managed-test",
                      "preparation": "stable-pool", "driver": "sealed-driver"}
        row = self.state.get("execution", self.execution)
        self.state.put("execution", self.execution,
                       {**row["payload"], "executionKey": structured},
                       expected_version=row["version"])
        bundle = publish_execution_outputs(self.state, self.build, self.execution,
                                           structured, self.receipts)
        self.assertEqual(bundle["executionKey"], digest(structured))
        with self.assertRaises(Exception) as error:
            publish_execution_outputs(self.state, self.build, self.execution,
                                      {**structured, "source": "other"}, self.receipts)
        self.assertEqual(error.exception.code, "execution_identity_mismatch")

    def test_unvalidated_execution_cannot_publish(self):
        self.state.put("execution", self.execution, {"status": "running",
                       "executionKey": self.key, "identity": self.identity})
        with self.assertRaises(Exception) as error:
            publish_execution_outputs(self.state, self.build, self.execution,
                                      self.key, self.receipts)
        self.assertEqual(getattr(error.exception, "code", ""), "execution_not_validated")


if __name__ == "__main__":
    unittest.main()
