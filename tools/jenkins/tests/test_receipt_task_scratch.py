"""Scratch receipts retain original collector evidence; no Cargo is launched."""
from copy import deepcopy
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.processes.execution_host import ExecutionHost
from tools.jenkins.state import State
from tools.jenkins.validation import (
    TrustedReceiptAuthority, closure_manifest, issue_acceptance_receipt,
    validate_acceptance_receipt, validate_producer_receipt,
)


class ReceiptTaskScratchTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="receipt-scratch-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.state = State(self.root / "state.sqlite3")
        self.authority = TrustedReceiptAuthority(self.state)
        self.output = self.root / "output"
        self.output.mkdir()
        (self.output / "artifact.bin").write_bytes(b"managed output")
        self.closure = closure_manifest(self.output)
        self.eid, self.attempt_id = "owned-execution", "owned-execution:compile"
        self.scratch = {"scratchGeneration": digest(["scratch", str(self.root)]),
                        "effectiveEnvironment": {name: str(self.root / "scratch")
                                                 for name in ("TEMP", "TMP", "TMPDIR")}}
        self.environment_digest = digest({**self.scratch["effectiveEnvironment"], "SEALED_FLAG": "yes"})
        self.manifest = {"executionId": self.attempt_id, "stage": "build", "phase": "compile",
                         "sourceDigest": "a" * 64, "recipeDigest": "b" * 64,
                         "coverageDigest": "c" * 64, "driverDigest": "d" * 64,
                         "nativeJobId": "phase-job", "birthToken": "123", "outputRoot": str(self.output),
                         "requiredOutputs": ["artifact.bin"], "taskScratch": deepcopy(self.scratch),
                         "effectiveEnvironmentDigest": self.environment_digest}
        self.proof = self.native_proof(42, "123")
        self.state.put("native_job", "phase-job", {
            "nativeJobId": "phase-job", "owner": self.eid, "executionId": self.eid,
            "role": "phase", "status": "terminal", "pid": 42, "birthToken": "123",
            "executable": "python.exe", "terminalProof": self.proof, "completeProof": self.proof,
            "guardNativeJobId": "guard-job"})
        guard_proof = self.native_proof(41, "122")
        self.state.put("native_job", "guard-job", {
            "nativeJobId": "guard-job", "owner": self.eid, "executionId": self.eid,
            "role": "execution_host_guard", "status": "terminal", "pid": 41, "birthToken": "122",
            "executable": "python.exe", "terminalProof": guard_proof, "completeProof": guard_proof})
        self.state.put("execution_launch", self.eid, {"guardNativeJobId": "guard-job",
            "launcherIdentity": {"pid": 41, "creationTime": "122"}})
        self.state.put("execution_host", self.eid, {"executionId": self.eid, "operationId": "runtime-1",
            "hostPid": 41, "hostBirthToken": "122", "generation": "1",
            "taskScratch": deepcopy(self.scratch), "phaseRecords": []})
        self.state.put("execution", self.eid, {"operationId": "runtime-1", "generation": "1",
            "taskScratch": {"scratchGeneration": self.scratch["scratchGeneration"],
                            "effectiveEnvironmentDigest": digest(self.scratch["effectiveEnvironment"])}})
        self.state.put("execution_scratch", self.eid, {
            "executionId": self.eid, "status": "owned", **deepcopy(self.scratch)})

    @staticmethod
    def native_proof(pid, birth):
        return {"identity": {"pid": pid, "creationTime": birth, "executable": "python.exe",
                             "commandDigest": ""}, "complete": True, "processExitCode": 0,
                "activeProcesses": 0, "stdoutEof": True, "stderrEof": True, "childrenGone": True}

    def update(self, domain, key, **changes):
        row = self.state.get(domain, key)
        return self.state.put(domain, key, {**row["payload"], **deepcopy(changes)}, expected_version=row["version"])

    def collect(self, *, publish=True):
        self.authority.register_attempt(self.manifest)
        proof = {**self.proof, "nativeJobId": "phase-job", "birthToken": "123"}
        receipt = self.authority.record_terminal(self.attempt_id, proof, self.closure)
        if publish:
            phase = {"phase": "compile", "nativeJobId": "phase-job", "terminalProof": self.proof,
                     "taskScratch": deepcopy(self.scratch), "effectiveEnvironmentDigest": self.environment_digest,
                     "stageReceipts": {"build": self.attempt_id}, "receipts": {"build": receipt}}
            self.update("execution_host", self.eid, phaseRecords=[phase])
            self.update("execution", self.eid, phaseRecords=[phase])
        return receipt

    def test_collector_and_acceptance_preserve_both_distinct_environment_digests(self):
        receipt = self.collect()
        self.assertEqual(self.scratch, receipt["taskScratch"])
        self.assertEqual(self.environment_digest, receipt["effectiveEnvironmentDigest"])
        self.assertNotEqual(digest(self.scratch["effectiveEnvironment"]), receipt["effectiveEnvironmentDigest"])
        accepted = self.authority.accept(self.attempt_id, {"stage": "build"})
        self.assertEqual(self.scratch, accepted["taskScratch"])
        self.assertEqual(self.environment_digest, accepted["effectiveEnvironmentDigest"])
        validate_acceptance_receipt(accepted, producer_receipt=receipt, output_root=self.output)

    def test_genuine_old_receipts_remain_compatible_when_both_fields_are_absent(self):
        for name in ("taskScratch", "effectiveEnvironmentDigest"):
            self.manifest.pop(name)
        for domain in ("execution_host", "execution"):
            row = self.state.get(domain, self.eid)
            value = row["payload"]
            value.pop("taskScratch")
            self.state.put(domain, self.eid, value, expected_version=row["version"])
        receipt = self.collect(publish=False)
        accepted = self.authority.accept(self.attempt_id, {})
        self.assertNotIn("taskScratch", receipt)
        self.assertNotIn("effectiveEnvironmentDigest", accepted)
        validate_acceptance_receipt(accepted, producer_receipt=receipt)

    def test_registration_rejects_partial_or_malformed_new_bindings(self):
        variants = []
        for name in ("taskScratch", "effectiveEnvironmentDigest"):
            value = deepcopy(self.manifest)
            value.pop(name)
            variants.append(value)
        for scratch in (None, {}, {"scratchGeneration": "x", "effectiveEnvironment": {}},
                        {**self.scratch, "extra": True},
                        {**self.scratch, "effectiveEnvironment": {"TEMP": 123, "TMP": "x", "TMPDIR": "x"}},
                        {**self.scratch, "effectiveEnvironment": {**self.scratch["effectiveEnvironment"], "PATH": "x"}}):
            variants.append({**self.manifest, "taskScratch": scratch})
        variants.append({**self.manifest, "effectiveEnvironmentDigest": "not a digest"})
        for manifest in variants:
            with self.subTest(binding=manifest.get("taskScratch")):
                with self.assertRaises(JenkinsError):
                    self.authority.register_attempt(manifest)
                self.assertIsNone(self.state.get("validation_attempt", self.attempt_id))

    def test_receipt_dictionary_cannot_replace_or_omit_original_manifest_binding(self):
        receipt = self.collect()
        variants = []
        for name in ("taskScratch", "effectiveEnvironmentDigest"):
            forged = deepcopy(receipt)
            forged.pop(name)
            variants.append(forged)
        variants.extend([{**receipt, "effectiveEnvironmentDigest": "e" * 64},
                         {**receipt, "taskScratch": {**self.scratch, "scratchGeneration": "f" * 64}}])
        for forged in variants:
            with self.subTest(field=forged.get("effectiveEnvironmentDigest")):
                self.update("validation_attempt", self.attempt_id, receipt=forged)
                self.state.put("validation_receipt", self.attempt_id, forged)
                with self.assertRaises(JenkinsError):
                    self.authority.accept(self.attempt_id, {})
                self.assertEqual([], self.state.list("acceptance"))

    def test_omitting_both_manifest_and_receipt_fields_cannot_downgrade_new_native_evidence(self):
        receipt = self.collect()
        row = self.state.get("validation_attempt", self.attempt_id)
        stripped = row["payload"]
        for value in (stripped, receipt):
            for name in ("taskScratch", "effectiveEnvironmentDigest"):
                value.pop(name)
        stripped["receipt"] = receipt
        self.state.put("validation_attempt", self.attempt_id, stripped, expected_version=row["version"])
        with self.assertRaises(JenkinsError) as caught:
            self.authority.get(self.attempt_id)
        self.assertEqual("receipt_scratch_mismatch", caught.exception.code)

    def test_collection_requires_original_execution_scratch_and_native_identity(self):
        mutations = [("execution_scratch", self.eid, {"scratchGeneration": "e" * 64}),
                     ("execution_scratch", self.eid, {"executionId": "foreign"}),
                     ("execution_host", self.eid, {"operationId": "foreign-runtime"}),
                     ("execution_host", self.eid, {"hostBirthToken": "foreign-birth"}),
                     ("execution_host", self.eid, {"taskScratch": {**self.scratch, "scratchGeneration": "e" * 64}}),
                     ("execution", self.eid, {"taskScratch": {"scratchGeneration": self.scratch["scratchGeneration"],
                                                              "effectiveEnvironmentDigest": "e" * 64}}),
                     ("native_job", "phase-job", {"birthToken": "foreign-birth"}),
                     ("native_job", "phase-job", {"guardNativeJobId": "foreign-guard"})]
        self.authority.register_attempt(self.manifest)
        for domain, key, changes in mutations:
            with self.subTest(domain=domain, changes=list(changes)):
                original = self.state.get(domain, key)["payload"]
                self.update(domain, key, **changes)
                with self.assertRaises(JenkinsError):
                    self.authority.record_terminal(self.attempt_id,
                        {**self.proof, "nativeJobId": "phase-job", "birthToken": "123"}, self.closure)
                self.assertIsNone(self.state.get("validation_receipt", self.attempt_id))
                self.state.put(domain, key, original)

    def test_complete_new_dictionary_requires_real_scratch_registration(self):
        self.state.delete("execution_scratch", self.eid)
        with self.assertRaises(JenkinsError) as caught:
            self.collect(publish=False)
        self.assertEqual("receipt_scratch_unproven", caught.exception.code)
        self.assertIsNone(self.state.get("validation_receipt", self.attempt_id))

    def test_collection_precedes_phase_publication_but_acceptance_waits_for_phase_and_guard(self):
        self.collect(publish=False)
        with self.assertRaises(JenkinsError) as caught:
            self.authority.get(self.attempt_id)
        self.assertEqual("receipt_scratch_pending", caught.exception.code)
        self.assertTrue(caught.exception.retryable)
        self.collect()
        guard = self.state.get("native_job", "guard-job")["payload"]
        self.update("native_job", "guard-job", status="running", completeProof=None)
        with self.assertRaises(JenkinsError) as caught:
            self.authority.accept(self.attempt_id, {})
        self.assertEqual("receipt_scratch_pending", caught.exception.code)
        self.assertEqual([], self.state.list("acceptance"))
        self.state.put("native_job", "guard-job", guard)
        self.assertEqual("accepted", self.authority.accept(self.attempt_id, {})["status"])

    def test_phase_and_execution_publication_cannot_change_sealed_environment_or_proof(self):
        self.collect()
        original = self.state.get("execution_host", self.eid)["payload"]["phaseRecords"]
        changes = [{"effectiveEnvironmentDigest": "e" * 64},
                   {"taskScratch": {**self.scratch, "effectiveEnvironment":
                       {name: str(self.root / "foreign") for name in ("TEMP", "TMP", "TMPDIR")}}},
                   {"terminalProof": self.native_proof(42, "foreign-birth")}, {"phase": "test"},
                   {"stageReceipts": {"build": "foreign-attempt"}}]
        for change in changes:
            with self.subTest(change=list(change)):
                self.update("execution_host", self.eid, phaseRecords=[{**original[0], **change}])
                with self.assertRaises(JenkinsError):
                    self.authority.get(self.attempt_id)
        self.update("execution_host", self.eid, phaseRecords=original)
        self.update("execution", self.eid, phaseRecords=[{**original[0], "effectiveEnvironmentDigest": "e" * 64}])
        with self.assertRaises(JenkinsError):
            self.authority.get(self.attempt_id)

    def test_redigesting_acceptance_cannot_forge_or_drop_producer_scratch_evidence(self):
        receipt = self.collect()
        accepted = issue_acceptance_receipt(receipt)
        for change in ({"effectiveEnvironmentDigest": "e" * 64}, {"taskScratch": None}):
            forged = {**accepted, **change}
            forged["receiptDigest"] = digest({key: value for key, value in forged.items() if key != "receiptDigest"})
            with self.assertRaises(JenkinsError):
                validate_acceptance_receipt(forged, producer_receipt=receipt)
        stripped = {key: value for key, value in accepted.items()
                    if key not in {"taskScratch", "effectiveEnvironmentDigest", "receiptDigest"}}
        stripped["receiptDigest"] = digest(stripped)
        with self.assertRaises(JenkinsError):
            validate_acceptance_receipt(stripped, producer_receipt=receipt)

    def test_reusing_stored_acceptance_checks_original_scratch_evidence(self):
        self.collect()
        accepted = self.authority.accept(self.attempt_id, {})
        forged = {**accepted, "effectiveEnvironmentDigest": "e" * 64}
        forged["receiptDigest"] = digest({key: value for key, value in forged.items() if key != "receiptDigest"})
        self.state.put("acceptance", accepted["acceptanceId"], forged)
        with self.assertRaises(JenkinsError) as caught:
            self.authority.accept(self.attempt_id, {})
        self.assertEqual("receipt_scratch_mismatch", caught.exception.code)

    def test_historical_flow_preserves_collected_outputs_but_rejects_redigested_scratch(self):
        producer = self.collect()
        contract = {"sourceDigest": "a" * 64, "coverageDigest": "c" * 64, "driverDigest": "d" * 64,
                    "generation": "1", "requiredStages": ["build"],
                    "stageReceipts": {"build": self.attempt_id}, "status": "accepted"}
        acceptance_id = self.authority._flow_acceptance_id("consumer", contract)
        accepted = issue_acceptance_receipt(producer, acceptance_id=acceptance_id)
        accepted.update(stages=["build"], flowIdentity=self.authority._flow_identity(contract))
        accepted["receiptDigest"] = digest({key: value for key, value in accepted.items() if key != "receiptDigest"})
        self.state.put("acceptance", acceptance_id, accepted)
        self.state.put("workflow", "consumer", {**contract, "acceptance": accepted})
        (self.output / "artifact.bin").unlink()
        self.assertEqual(accepted, self.authority.accept_flow({"flowId": "consumer"}))
        for remove in (False, True):
            forged = deepcopy(accepted)
            if remove:
                forged.pop("taskScratch")
                forged.pop("effectiveEnvironmentDigest")
            else:
                forged["effectiveEnvironmentDigest"] = "e" * 64
            forged["receiptDigest"] = digest({key: value for key, value in forged.items() if key != "receiptDigest"})
            self.state.put("acceptance", acceptance_id, forged)
            self.update("workflow", "consumer", acceptance=forged)
            with self.assertRaises(JenkinsError) as caught:
                self.authority.accept_flow({"flowId": "consumer"})
            self.assertEqual("receipt_scratch_mismatch", caught.exception.code)

    def test_real_native_phase_receipt_matches_the_environment_received_by_child(self):
        (self.root / "prepared").mkdir()
        (self.root / "scratch").mkdir()
        command = [sys.executable, "-B", "-c", "import json,os; print(json.dumps({key:os.environ[key] "
                   "for key in ('TEMP','TMP','TMPDIR','ZIRCON_SCRATCH_TEST')}))"]
        recipe = {"kind": "fixture", "phases": ["compile"], "commands": {"compile": command},
                  "preparationRoot": str(self.root / "prepared"),
                  "executionEnvironment": {"ZIRCON_SCRATCH_TEST": "sealed-phase"}}
        recipe_ref = digest(recipe)
        self.state.put("recipe_plan", recipe_ref, {"recipe": recipe, "recipeDigest": recipe_ref})
        self.state.put("sealed_input", "a" * 64, {"status": "sealed", "preparationRoot": str(self.root / "prepared")})
        self.state.delete("execution_host", self.eid)
        self.state.delete("execution_launch", self.eid)
        self.update("execution", self.eid, status="running", recipeRef=recipe_ref, recipeDigest=recipe_ref,
                    sealedInputRef="a" * 64, sourceDigest="a" * 64, coverageDigest="c" * 64,
                    driverDigest="d" * 64, phaseOutputRoot=str(self.root / "phases"),
                    phaseLogRoot=str(self.root / "logs"))
        host = ExecutionHost(self.state, self.eid, operation_id="runtime-1", generation="1")
        trusted_recipe = host._trusted_recipe

        def scratch_recipe(payload):
            actual, sealed = trusted_recipe(payload)
            return {**actual, "_taskScratch": deepcopy(self.scratch)}, sealed

        # The scratch allocator is covered separately; the child, registry,
        # manifest, collector, phase publication and receipt are all real.
        with patch.object(host, "_trusted_recipe", side_effect=scratch_recipe):
            result = host.serve_phases()
        phase = result["phaseRecords"][0]
        receipt = self.authority.get(self.attempt_id)
        received = json.loads(Path(phase["stdoutPath"]).read_text(encoding="utf-8").strip())
        self.assertEqual({**self.scratch["effectiveEnvironment"], "ZIRCON_SCRATCH_TEST": "sealed-phase"}, received)
        effective = {**os.environ, **recipe["executionEnvironment"], **self.scratch["effectiveEnvironment"]}
        self.assertEqual(digest(effective), receipt["effectiveEnvironmentDigest"])
        self.assertEqual(self.scratch, receipt["taskScratch"])
        self.assertEqual(phase["effectiveEnvironmentDigest"], receipt["effectiveEnvironmentDigest"])
        self.assertTrue(receipt["terminalProof"]["complete"])
        accepted = self.authority.accept(self.attempt_id, {"stage": "build"})
        validate_acceptance_receipt(accepted, producer_receipt=receipt)


if __name__ == "__main__":
    unittest.main()
