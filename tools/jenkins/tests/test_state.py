from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.jenkins.contracts import JenkinsError, digest, execution_key
from tools.jenkins.state import State


class StateTests(unittest.TestCase):
    def setUp(self):
        parent = Path(__file__).resolve().parents[3] / ".jenkins/state/test-runs"
        parent.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=parent)
        self.addCleanup(self.temp.cleanup)
        self.state = State(Path(self.temp.name) / "coordination.sqlite3")

    def test_request_id_is_idempotent_and_payload_immutable(self):
        request = {"sessionId": "s1", "requestId": "r1", "repositoryId": "repo",
                   "ownedPaths": [], "requestedActions": []}
        first = self.state.submit_request(request)
        self.assertEqual(first, self.state.submit_request(request))
        with self.assertRaisesRegex(JenkinsError, "payload"):
            self.state.submit_request({**request, "ownedPaths": ["changed.rs"]})

    def test_record_compare_and_swap_rejects_stale_writer(self):
        record = self.state.put("pool", "p", {"writer": None})
        self.state.put("pool", "p", {"writer": "one"}, expected_version=record["version"])
        with self.assertRaises(JenkinsError):
            self.state.put("pool", "p", {"writer": "two"}, expected_version=record["version"])
        self.assertEqual("one", self.state.get("pool", "p")["payload"]["writer"])

    def test_transaction_rolls_back_resource_and_writer_together(self):
        with self.assertRaises(ValueError):
            with self.state.transaction() as connection:
                self.state.put("pool", "p", {"writer": "one"}, connection=connection)
                self.state.put("reservation", "r", {"bytes": 100}, connection=connection)
                raise ValueError("abort")
        self.assertIsNone(self.state.get("pool", "p"))
        self.assertIsNone(self.state.get("reservation", "r"))

    def test_operation_intent_is_immutable_and_transition_is_conditional(self):
        self.state.put_operation("deployment", "op", {"root": "one"})
        with self.assertRaises(JenkinsError):
            self.state.put_operation("deployment", "op", {"root": "two"})
        self.state.transition_operation("op", "prepared", "running", {"pid": 1})
        with self.assertRaises(JenkinsError):
            self.state.transition_operation("op", "prepared", "complete", {})

    def test_session_scope_does_not_grant_commit(self):
        self.state.authorize_session("repo", "s", "local-user", ["tools/jenkins"],
                                     ["implementation", "validation"],
                                     {"source": "user", "task": "Jenkins implementation"})
        self.state.check_authorization("repo", "s", "validation", ["tools/jenkins/cli.py"])
        with self.assertRaises(JenkinsError):
            self.state.check_authorization("repo", "s", "commit", ["tools/jenkins/cli.py"])
        with self.assertRaises(JenkinsError):
            self.state.check_authorization("repo", "s", "implementation", ["tools/other.py"])

    def test_execution_identity_is_shared_by_different_sessions(self):
        key = execution_key(digest("source"), {"kind": "test"}, {"package": "p"}, digest("driver"), "v1")
        self.assertEqual(key, execution_key(digest("source"), {"kind": "test"}, {"package": "p"}, digest("driver"), "v1"))

    def test_request_transition_is_generation_bound_and_terminal_is_immutable(self):
        self.state.submit_request({"repositoryId": "repo", "sessionId": "s", "requestId": "r"})
        self.state.transition_request("repo", "s", "r", 1, "pending", "running")
        with self.assertRaises(JenkinsError):
            self.state.transition_request("repo", "s", "r", 2, "running", "accepted")
        receipt = self.state.put("acceptance", "accept", {"status": "accepted"})
        result = self.state.transition_request("repo", "s", "r", 1, "running", "accepted",
                                               {"receiptRef": receipt["key"]})
        self.assertEqual("accepted", result["status"])
        self.assertEqual(result, self.state.transition_request("repo", "s", "r", 1, "running", "accepted",
                                                               {"receiptRef": "accept"}))
        with self.assertRaises(JenkinsError):
            self.state.transition_request("repo", "s", "r", 1, "accepted", "running")
        with self.assertRaises(JenkinsError):
            self.state.transition_request("repo", "s", "r", 1, "running", "accepted",
                                          {"receiptRef": "changed"})

    def test_request_status_and_evidence_roll_back_together(self):
        self.state.submit_request({"repositoryId": "repo", "sessionId": "s", "requestId": "r"})
        with self.assertRaises(ValueError):
            with self.state.transaction() as connection:
                self.state.transition_request("repo", "s", "r", 1, "pending", "running",
                                              {"flowId": "flow"}, connection=connection)
                raise ValueError("simulate failed workflow write")
        self.assertEqual("pending", self.state.get_request("repo", "s", "r")["status"])
        self.assertIsNone(self.state.get("request_result", digest(["repo", "s", "r"])))

    def test_absolute_authorization_path_cannot_become_relative(self):
        with self.assertRaises(JenkinsError):
            self.state.authorize_session("repo", "absolute", "user", ["/tools/jenkins"],
                                         ["implementation"], {"source": "user", "task": "implementation"})


if __name__ == "__main__":
    unittest.main()
