"""Accepted flow evidence and the original request must close atomically."""
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tools.jenkins.cli import dispatch
from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.state import State
from tools.jenkins.validation import TrustedReceiptAuthority, closure_manifest


class AcceptanceRequestProjectionTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.state = State(self.root / "state.sqlite3")
        self.authority = TrustedReceiptAuthority(self.state)
        self.identity = {"repositoryId": "repo", "sessionId": "session", "requestId": "request",
                         "attemptId": "attempt", "generation": 1, "sourceInputDigest": "a" * 64,
                         "coverageDigest": "c" * 64, "stageImplementationDigest": "d" * 64}
        self.state.authorize_session("repo", "session", "fixture", ["src/lib.rs"], ["validation"],
                                     {"source": "user", "task": "Exact accepted consumer request closeout fixture"})
        self.request = {**self.identity, "sealedInputRef": "a" * 64, "ownedPaths": ["src/lib.rs"]}
        self.contract = {"identity": self.identity, "repositoryId": "repo", "generation": 1,
                         "sourceDigest": "a" * 64, "coverageDigest": "c" * 64, "driverDigest": "d" * 64,
                         "ownedPaths": ["src/lib.rs"], "status": "ready",
                         "requiredStages": ["patch", "syntax", "format"], "requiredOutputs": [],
                         "stageReceipts": {}, "stageRecipeDigests": {}, "stages": [{"name": "acceptance"}]}
        for index, stage in enumerate(self.contract["requiredStages"]):
            output = self.root / stage
            output.mkdir()
            (output / "evidence.txt").write_text(stage, encoding="utf-8")
            execution = "light-" + stage
            recipe = digest([stage, index])
            self.authority.register_attempt({"executionId": execution, "stage": stage,
                "sourceDigest": "a" * 64, "coverageDigest": "c" * 64, "driverDigest": "d" * 64,
                "recipeDigest": recipe})
            self.state.put("light_attempt", execution,
                           {"status": "registered", "sourceDigest": "a" * 64, "driverDigest": "d" * 64})
            self.authority.record_light_terminal(execution,
                {"status": "passed", "exitCode": 0, "tool": "fixture-parser", "commandDigest": digest(stage)},
                closure_manifest(output))
            self.contract["stageReceipts"][stage] = execution
            self.contract["stageRecipeDigests"][stage] = recipe
        self.state.put("workflow", "flow", self.contract)

    def register_request(self, **changes):
        return self.state.submit_request({**self.request, **changes})

    def accept(self):
        return self.authority.accept_flow({"flowId": "flow", "identity": self.identity})

    def assert_unaccepted(self):
        self.assertEqual("ready", self.state.get("workflow", "flow")["payload"]["status"])
        self.assertEqual([], self.state.list("acceptance"))

    def test_formal_acceptance_closes_original_running_request_atomically(self):
        self.register_request()
        self.state.transition_request("repo", "session", "request", 1, "pending", "running")
        receipt = self.accept()
        request = self.state.get_request("repo", "session", "request")
        self.assertEqual("accepted", request["status"])
        self.assertEqual({"flowId": "flow", "receiptRef": receipt["acceptanceId"]},
                         self.state.get("request_result", digest(["repo", "session", "request"]))["payload"]["result"])
        self.assertEqual(receipt, self.accept())
        self.assertEqual(request, self.state.get_request("repo", "session", "request"))

    def test_another_source_or_attempt_cannot_accept_the_original_request(self):
        for field in ("attemptId", "sourceInputDigest", "coverageDigest", "stageImplementationDigest"):
            with self.subTest(field=field):
                original = self.state.get_request("repo", "session", "request")
                if original is None:
                    self.register_request(**{field: "foreign" if field == "attemptId" else "e" * 64})
                with self.assertRaises(JenkinsError) as raised:
                    self.accept()
                self.assertEqual("workflow_request_mismatch", raised.exception.code)
                self.assert_unaccepted()
                with self.state.transaction() as connection:
                    connection.execute("DELETE FROM requests WHERE repository_id='repo'")

    def test_conflicting_source_alias_cannot_hide_a_mismatch(self):
        self.register_request(sealedInputRef="e" * 64)
        with self.assertRaises(JenkinsError):
            self.accept()
        self.assert_unaccepted()

    def test_late_acceptance_cannot_reverse_a_terminal_failure(self):
        self.register_request()
        self.state.transition_request("repo", "session", "request", 1, "pending", "failed", {"failure": "original"})
        with self.assertRaises(JenkinsError) as raised:
            self.accept()
        self.assertEqual("workflow_request_terminal", raised.exception.code)
        self.assertEqual("failed", self.state.get_request("repo", "session", "request")["status"])
        self.assert_unaccepted()

    def test_cancel_intent_prevents_late_acceptance(self):
        self.register_request()
        dispatch("request", "cancel", {"identity": self.identity}, self.state, self.root)
        with self.assertRaises(JenkinsError):
            self.accept()
        self.assertEqual("cancel_requested", self.state.get_request("repo", "session", "request")["status"])
        self.assert_unaccepted()

    def test_request_write_failure_rolls_back_the_receipt_and_workflow(self):
        self.register_request()
        with patch.object(self.state, "transition_request", side_effect=OSError("fixture durable write failed")):
            with self.assertRaises(OSError):
                self.accept()
        self.assertEqual("pending", self.state.get_request("repo", "session", "request")["status"])
        self.assert_unaccepted()

    def test_historical_observation_is_read_only_and_explicit_reconcile_is_idempotent(self):
        receipt = self.accept()  # Historical flow without its request projection.
        self.register_request()
        self.assertEqual(receipt, self.accept())
        self.assertEqual("pending", self.state.get_request("repo", "session", "request")["status"])
        payload = {"flowId": "flow", "identity": self.identity}
        observed = dispatch("accept", "reconcile-request", payload, self.state, self.root)
        self.assertEqual("accepted", observed["status"])
        self.assertEqual(receipt["acceptanceId"], observed["receiptRef"])
        before = self.state.get("request_result", digest(["repo", "session", "request"]))
        self.assertEqual(observed, dispatch("accept", "reconcile-request", payload, self.state, self.root))
        self.assertEqual(before, self.state.get("request_result", digest(["repo", "session", "request"])))

    def test_reconcile_requires_exact_author_and_an_existing_formal_acceptance(self):
        self.register_request()
        with self.assertRaises(JenkinsError) as raised:
            self.authority.reconcile_request({"flowId": "flow", "identity": self.identity})
        self.assertEqual("workflow_unaccepted", raised.exception.code)
        self.accept()
        with self.assertRaises(JenkinsError):
            self.authority.reconcile_request({"flowId": "flow", "identity": {**self.identity, "attemptId": "other"}})


if __name__ == "__main__":
    unittest.main()
