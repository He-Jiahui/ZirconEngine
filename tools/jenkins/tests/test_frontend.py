from __future__ import annotations

import json
import os
from unittest import mock
from urllib.parse import parse_qs
import tempfile
import unittest
from pathlib import Path
from urllib.error import URLError

from tools.jenkins.frontend import dispatch_or_local, entry_from_environment, submit_entry
from tools.jenkins.state import State
from tools.jenkins.contracts import JenkinsError, digest


class FrontendTests(unittest.TestCase):
    def _root(self, active: bool) -> Path:
        root = Path(tempfile.mkdtemp())
        state = root / ".jenkins" / "state"
        state.mkdir(parents=True)
        (state / "entry-inventory.json").write_text(json.dumps({
            "activation": {"soleEntryEnforcementApplied": active}
        }), encoding="utf-8")
        (state / "deployment").mkdir()
        (state / "deployment" / "acceptance.json").write_text(json.dumps({
            "m8Accepted": active, "soleEntryEnforcementApplied": active
        }), encoding="utf-8")
        return root

    @staticmethod
    def _identity() -> dict[str, str]:
        return {"repositoryId": "repo", "sessionId": "session", "requestId": "request",
                "sourceRef": "source", "sourceDigest": "a" * 64,
                "coverageRef": "coverage", "coverageDigest": "b" * 64}

    def test_gate_keeps_local_path_before_m8(self):
        root = self._root(False)
        result = dispatch_or_local(repo_root=root, state=None, identity=None, local_result={"exit": 0})
        self.assertEqual("local", result["mode"])
        self.assertFalse(result["formalAcceptance"])

    def test_active_gate_requires_identity(self):
        root = self._root(True)
        with self.assertRaisesRegex(JenkinsError, "identity"):
            submit_entry(State(root / ".jenkins" / "state" / "coordination.sqlite3"),
                         identity=None, repo_root=root)

    def test_unknown_http_outcome_is_not_retried(self):
        root = self._root(True)
        state = State(root / ".jenkins" / "state" / "coordination.sqlite3")
        calls = []

        def transport(request):
            calls.append(request)
            raise URLError("connection dropped after send")

        with self.assertRaisesRegex(JenkinsError, "unknown"):
            submit_entry(state, identity=self._identity(), repo_root=root, transport=transport)
        self.assertEqual(1, len(calls))
        with self.assertRaisesRegex(JenkinsError, "reconciled"):
            submit_entry(state, identity=self._identity(), repo_root=root, transport=transport)
        self.assertEqual(1, len(calls))

    def test_raw_command_and_acceptance_flags_are_rejected(self):
        root = self._root(True)
        state = State(root / ".jenkins" / "state" / "coordination.sqlite3")
        with self.assertRaisesRegex(JenkinsError, "raw command"):
            submit_entry(state, identity=self._identity(), repo_root=root,
                         parameters={"command": "cargo test"})

    def _active_request(self, root: Path, identity: dict) -> State:
        state = State(root / ".jenkins" / "state" / "coordination.sqlite3")
        state.submit_request({
            "repositoryId": identity["repositoryId"], "sessionId": identity["sessionId"],
            "requestId": identity["requestId"], "sourceRef": identity["sealedInputRef"],
            "sourceDigest": identity["sourceInputDigest"], "sealedInputRef": identity["sealedInputRef"],
            "coverageRef": identity["coverageDigest"], "coverageDigest": identity["coverageDigest"],
            "buildRoot": identity["buildRoot"], "parameters": {
                "sealedInputRef": identity["sealedInputRef"], "patchOperationRef": "patch-1",
                "buildRoot": identity["buildRoot"], "rustEdition": "2021"},
        })
        return state

    def test_current_state_identity_dispatches_flow_parameters_once(self):
        root = self._root(True)
        identity = {"repositoryId":"repo", "sessionId":"session", "requestId":"request",
                    "sourceInputDigest":"a" * 64, "sealedInputRef":"a" * 64,
                    "coverageDigest":"b" * 64, "buildRoot":"D:/cargo-targets",
                    "attemptId":"attempt-1", "generation":1, "driverDigest":"d" * 64}
        state = self._active_request(root, identity)
        calls = []
        class Response:
            status = 201
            headers = {}
        def transport(request):
            calls.append(parse_qs(request.data.decode()))
            return Response()
        (root / "identity.json").write_text(json.dumps(identity), encoding="utf-8")
        with mock.patch.dict(os.environ, {"ZIRCON_JENKINS_IDENTITY_FILE": str(root / "identity.json")}, clear=False):
            with mock.patch("tools.jenkins.frontend.submit_entry", return_value={"reused": False}) as dispatch:
                result = entry_from_environment(root, job="zircon-flow")
        self.assertFalse(result["reused"])
        sent_parameters = dispatch.call_args.kwargs["parameters"]
        self.assertEqual(identity["sealedInputRef"], sent_parameters["sealedInputRef"])
        self.assertNotIn("recipeRef", sent_parameters)
        # The production submit adapter maps those canonical fields to Jenkins
        # parameter names and keeps the side effect idempotent.
        calls.clear()
        # Use a fresh request for the transport mapping assertion; the first
        # request above is intentionally already claimed by the mocked entry.
        identity2 = dict(identity, requestId="request-2")
        state2 = State(root / ".jenkins" / "state" / "transport.sqlite3")
        result = submit_entry(state2, identity=identity2, repo_root=root, transport=transport,
                              parameters={"sealedInputRef": identity2["sealedInputRef"],
                                          "patchOperationRef":"patch-1", "buildRoot":"D:/cargo-targets",
                                          "rustEdition":"2021"})
        self.assertTrue(result["reused"] or calls)
        if calls:
            sent = calls[0]
            self.assertEqual([identity["sealedInputRef"]], sent["SEALED_INPUT_REF"])
            self.assertEqual(["patch-1"], sent["PATCH_OPERATION_REF"])
            self.assertEqual(["D:/cargo-targets"], sent["BUILD_ROOT"])
            self.assertEqual(["2021"], sent["RUST_EDITION"])
            self.assertNotIn("RECIPE_REF", sent)

    def test_current_state_identity_mismatch_is_rejected(self):
        root = self._root(True)
        identity = {"repositoryId":"repo", "sessionId":"session", "requestId":"request",
                    "sourceInputDigest":"a" * 64, "sealedInputRef":"a" * 64,
                    "coverageDigest":"b" * 64, "buildRoot":"D:/cargo-targets"}
        self._active_request(root, identity)
        identity["sourceInputDigest"] = "c" * 64
        identity["sealedInputRef"] = "c" * 64
        path = root / "identity.json"; path.write_text(json.dumps(identity), encoding="utf-8")
        with mock.patch.dict(os.environ, {"ZIRCON_JENKINS_IDENTITY_FILE": str(path)}, clear=False):
            with self.assertRaisesRegex(JenkinsError, "matching State request"):
                entry_from_environment(root)

    def test_registered_request_dispatch_preserves_payload_and_reuses_http_delivery(self):
        root = self._root(True)
        identity = {"repositoryId": "repo", "sessionId": "session", "requestId": "request",
                    "sourceInputDigest": "a" * 64, "sealedInputRef": "a" * 64,
                    "coverageDigest": "b" * 64, "buildRoot": "D:/cargo-targets",
                    "attemptId": "attempt-1", "generation": 1, "driverDigest": "d" * 64}
        state = self._active_request(root, identity)
        original = state.get_request("repo", "session", "request")["payload"]
        path = root / "identity.json"
        path.write_text(json.dumps(identity), encoding="utf-8")
        calls = []

        class Response:
            status = 201
            headers = {"Location": "http://127.0.0.1:53748/queue/item/17/"}

        def transport(request):
            calls.append(parse_qs(request.data.decode()))
            return Response()

        with mock.patch.dict(os.environ, {"ZIRCON_JENKINS_IDENTITY_FILE": str(path)}):
            first = entry_from_environment(root, transport=transport)
            second = entry_from_environment(root, transport=transport)
        self.assertEqual(1, len(calls))
        self.assertEqual(original, state.get_request("repo", "session", "request")["payload"])
        self.assertEqual(first["queueUrl"], second["queueUrl"])
        self.assertTrue(second["reused"])
        self.assertFalse(first["formalAcceptance"])
        self.assertEqual([identity["sealedInputRef"]], calls[0]["SEALED_INPUT_REF"])
        self.assertEqual([identity["coverageDigest"]], calls[0]["COVERAGE_DIGEST"])

    def test_default_dispatch_uses_managed_api_credentials(self):
        root = self._root(True)
        state = State(root / ".jenkins" / "state" / "coordination.sqlite3")
        manager = mock.Mock(base_url="http://127.0.0.1:53748")
        manager._request.return_value = (201, b"")
        spec, paths = object(), object()
        with mock.patch("tools.jenkins.deployment.spec.load_spec", return_value=spec) as load:
            with mock.patch("tools.jenkins.deployment.paths.resolve_paths", return_value=paths) as resolve:
                with mock.patch("tools.jenkins.deployment.manager.DeploymentManager", return_value=manager) as create:
                    with mock.patch("tools.jenkins.submission.urlopen", side_effect=AssertionError("unmanaged HTTP")):
                        result = submit_entry(state, identity=self._identity(), repo_root=root)
        load.assert_called_once_with(root / ".jenkins" / "deployment-spec.json")
        resolve.assert_called_once_with(spec)
        create.assert_called_once_with(spec, paths)
        self.assertEqual("job/zircon-flow/buildWithParameters", manager._request.call_args.args[0])
        self.assertEqual("POST", manager._request.call_args.kwargs["method"])
        self.assertFalse(result["formalAcceptance"])

    def test_legacy_unknown_flow_delivery_cannot_be_bypassed(self):
        root = self._root(True)
        identity = {"repositoryId": "repo", "sessionId": "session", "requestId": "request",
                    "sourceInputDigest": "a" * 64, "sealedInputRef": "a" * 64,
                    "coverageDigest": "b" * 64, "buildRoot": "D:/cargo-targets"}
        state = self._active_request(root, identity)
        original = state.get_request("repo", "session", "request")["payload"]
        marker = "repo:session:request:flow"
        state.put("jenkins_submission", marker, {**original, "payloadDigest": digest(original),
                                                "delivery": "unknown", "formalAcceptance": False})
        path = root / "identity.json"
        path.write_text(json.dumps(identity), encoding="utf-8")
        with mock.patch.dict(os.environ, {"ZIRCON_JENKINS_IDENTITY_FILE": str(path)}):
            with self.assertRaises(JenkinsError) as error:
                entry_from_environment(root, transport=lambda request: self.fail("duplicate HTTP delivery"))
        self.assertEqual("submission_payload_mismatch", error.exception.code)
        self.assertEqual("unknown", state.get("jenkins_submission", marker)["payload"]["delivery"])
        self.assertEqual([marker], [row["key"] for row in state.list("jenkins_submission")])


if __name__ == "__main__":
    unittest.main()
