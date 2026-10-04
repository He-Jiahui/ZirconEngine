from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.validation import (
    closure_manifest,
    issue_acceptance_receipt,
    validate_acceptance_receipt,
    validate_producer_receipt,
    TrustedReceiptAuthority,
)
from tools.jenkins.state import State


def producer(stage: str, closure: dict, *, heavy: bool = False) -> dict:
    value = {
        "schemaVersion": 1,
        "kind": "jenkins.managed.producer-receipt",
        "status": "passed",
        "executionId": "execution-1",
        "stage": stage,
        "sourceDigest": "a" * 64,
        "recipeDigest": "b" * 64,
        "coverageDigest": "c" * 64,
        "driverDigest": "d" * 64,
        "producer": {"managed": True, "name": "zircon-jenkins"},
        "outputClosure": closure,
    }
    if heavy:
        value.update({"requiresCompilerProof": True, "terminalProof": {"state": "exited", "exitCode": 0, "pid": 42, "birthToken": "birth", "nativeJobId": "job", "eofObserved": True, "nativeJobActive": False}})
    return value


class ValidationReceiptTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        (self.root / "artifact.bin").write_bytes(b"ok")
        self.closure = closure_manifest(self.root)

    def tearDown(self):
        self.temp.cleanup()

    def test_light_receipt_needs_no_compiler_proof(self):
        receipt = validate_producer_receipt(producer("syntax", self.closure), output_root=self.root)
        self.assertEqual(receipt["stage"], "syntax")

    def test_heavy_receipt_requires_terminal_proof(self):
        value = producer("build", self.closure)
        with self.assertRaises(JenkinsError) as raised:
            validate_producer_receipt(value)
        self.assertEqual(raised.exception.code, "terminal_proof_missing")

    def test_output_mutation_is_rejected(self):
        value = producer("unit_test", self.closure, heavy=True)
        (self.root / "artifact.bin").write_bytes(b"changed")
        with self.assertRaises(JenkinsError) as raised:
            validate_producer_receipt(value, output_root=self.root)
        self.assertEqual(raised.exception.code, "output_closure_changed")

    def test_fabricated_success_is_rejected(self):
        with self.assertRaises(JenkinsError) as raised:
            validate_producer_receipt({"status": "passed", "success": True})
        self.assertEqual(raised.exception.code, "fabricated_receipt")

    def test_acceptance_is_bound_and_tamper_evident(self):
        value = producer("format", self.closure)
        accepted = issue_acceptance_receipt(value, acceptance_id="acceptance-1", output_root=self.root)
        self.assertEqual(validate_acceptance_receipt(accepted, producer_receipt=value, output_root=self.root)["status"], "accepted")
        accepted["stage"] = "build"
        with self.assertRaises(JenkinsError):
            validate_acceptance_receipt(accepted, producer_receipt=value)

    def test_closure_digest_is_content_addressed(self):
        self.assertEqual(self.closure["digest"], digest(self.closure["files"]))

    def test_public_handle_cannot_register_or_record(self):
        from tools.jenkins.validation import handle
        with self.assertRaises(ValueError):
            handle("register", {}, State(self.root / "state.db"))

    def test_state_authority_binds_manifest_and_persists_acceptance(self):
        authority = TrustedReceiptAuthority(State(self.root / "state.db"))
        manifest = {
            "executionId": "execution-1", "stage": "build", "sourceDigest": "a" * 64,
            "recipeDigest": "b" * 64, "coverageDigest": "c" * 64, "driverDigest": "d" * 64,
            "nativeJobId": "job", "birthToken": "birth", "outputRoot": str(self.root),
            "requiredOutputs": ["artifact.bin"],
        }
        authority.register_attempt(manifest)
        authority.state.put("native_job", "job", {"nativeJobId": "job", "birthToken": "birth", "status": "terminal", "pid": 42, "completeProof": {"complete": True, "processExitCode": 0, "stdoutEof": True, "stderrEof": True, "childrenGone": True}})
        with self.assertRaises(JenkinsError):
            authority.register_attempt({**manifest, "coverageDigest": "e" * 64})
        value = producer("build", self.closure, heavy=True)
        authority.record_terminal("execution-1", {**value["terminalProof"], "processExitCode": 0, "stdoutEof": True, "stderrEof": True, "childrenGone": True}, self.closure)
        accepted = authority.accept("execution-1", {"executionId": "execution-1", "stage": "build"})
        self.assertEqual(authority.accept("execution-1", {"executionId": "execution-1", "stage": "build"}), accepted)

    def test_light_terminal_uses_trusted_journal_without_native_job(self):
        authority = TrustedReceiptAuthority(State(self.root / "light.db"))
        manifest = {
            "executionId": "light-1", "stage": "syntax", "sourceDigest": "a" * 64,
            "recipeDigest": "b" * 64, "coverageDigest": "c" * 64, "driverDigest": "d" * 64,
        }
        authority.register_attempt(manifest)
        authority.state.put("light_attempt", "light-1", {"status": "registered", "sourceDigest": "a" * 64, "driverDigest": "d" * 64})
        receipt = authority.record_light_terminal("light-1", {"status": "passed", "exitCode": 0, "tool": "parser", "commandDigest": "e" * 64}, self.closure)
        self.assertEqual(receipt["stage"], "syntax")

    def test_accept_flow_uses_state_contract_and_rejects_caller_stage_cut(self):
        authority = TrustedReceiptAuthority(State(self.root / "flow.db"))
        stages = ("patch", "syntax", "format")
        refs = {}
        for i, stage in enumerate(stages):
            execution = f"flow-{i}"
            authority.register_attempt({"executionId": execution, "stage": stage, "sourceDigest": "a" * 64,
                                        "recipeDigest": f"{i+1:064x}", "coverageDigest": "c" * 64, "driverDigest": "d" * 64})
            authority.state.put("light_attempt", execution, {"status": "registered", "sourceDigest": "a" * 64, "driverDigest": "d" * 64})
            authority.record_light_terminal(execution, {"status": "passed", "exitCode": 0, "tool": "parser", "commandDigest": "e" * 64}, self.closure)
            refs[stage] = execution
        recipes = {stage: f"{i+1:064x}" for i, stage in enumerate(stages)}
        authority.state.put("workflow", "wf-1", {"requiredStages": list(stages), "stageReceipts": refs,
                                                   "stageRecipeDigests": recipes, "requiredOutputs": [], "requiredOutputsByStage": {stage: [] for stage in stages},
                                                   "sourceDigest": "a" * 64, "coverageDigest": "c" * 64, "driverDigest": "d" * 64, "generation": 1})
        with self.assertRaises(JenkinsError):
            authority.accept_flow({"flowId": "wf-1", "requiredStages": ["patch"], "stageReceipts": {"patch": refs["patch"]}})
        accepted = authority.accept_flow({"flowId": "wf-1"})
        self.assertEqual(accepted["status"], "accepted")


if __name__ == "__main__":
    unittest.main()
