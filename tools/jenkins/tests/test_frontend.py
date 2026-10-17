from __future__ import annotations

import json
import os
import hashlib
import io
from unittest import mock
from urllib.parse import parse_qs
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from urllib.error import URLError

from tools.jenkins.frontend import dispatch_or_local, entry_from_environment, main as frontend_main, sole_entry_enforced, submit_entry
from tools.jenkins.state import State
from tools.jenkins.contracts import JenkinsError, digest


class FrontendTests(unittest.TestCase):
    def _root(self, active: bool) -> Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = Path(directory.name)
        state = root / ".jenkins" / "state"
        state.mkdir(parents=True)
        (state / "entry-inventory.json").write_text(json.dumps({
            "activation": {"soleEntryEnforcementApplied": active}
        }), encoding="utf-8")
        (state / "deployment").mkdir()
        (state / "deployment" / "acceptance.json").write_text(json.dumps({
            "m8Accepted": active, "soleEntryEnforcementApplied": active
        }), encoding="utf-8")
        # Literal test driver bytes are independent of the product sources.
        content = {"launcher.py": b"pass\n", "pipeline/steps.groovy": b"// fixture\n"}
        files = {name: {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
                 for name, data in content.items()}
        driver_digest = digest({"schemaVersion": 1, "files": files})
        driver = root / ".jenkins/runtime/drivers" / driver_digest
        for name, data in content.items():
            target = driver / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        (driver / "driver-manifest.json").write_text(json.dumps({
            "schemaVersion": 1, "driverDigest": driver_digest, "files": files}), encoding="utf-8")
        python = root / ".jenkins/runtime/python/python.exe"
        python.parent.mkdir(parents=True, exist_ok=True)
        python.write_bytes(b"dedicated Python fixture")
        (state / "deployment/driver.json").write_text(json.dumps({
            "driverDigest": driver_digest, "root": str(driver), "launcher": str(driver / "launcher.py"),
            "steps": str(driver / "pipeline/steps.groovy"), "runtimeOperationId": "start-fixture"}), encoding="utf-8")
        return root

    @staticmethod
    def _identity() -> dict[str, str]:
        return {"repositoryId": "repo", "sessionId": "session", "requestId": "request",
                "sourceRef": "source", "sourceDigest": "a" * 64,
                "coverageRef": "coverage", "coverageDigest": "b" * 64}

    def test_gate_routes_to_ordinary_capability_before_m8(self):
        """A closed M8 gate selects ordinary capability, not a caller-local path.

        The pre-M8 local escape was retired: every entry now goes through the
        ordinary management capability, so an absent identity is refused rather
        than waved through on a caller-supplied result.
        """
        root = self._root(False)
        self.assertFalse(sole_entry_enforced(root))
        with self.assertRaises(JenkinsError) as raised:
            dispatch_or_local(repo_root=root, state=None, identity=None)
        self.assertEqual("ordinary_identity_pending", raised.exception.code)

    def test_ordinary_default_preserves_management_validation_route(self):
        root = self._root(False)
        identity = self._identity()
        with mock.patch("tools.jenkins.frontend._ordinary_dispatch", return_value={"status": "management"}) as dispatch:
            result = submit_entry(None, identity=identity, repo_root=root)
        self.assertEqual({"status": "management"}, result)
        dispatch.assert_called_once_with(root, identity, None)

    def test_ordinary_explicit_flow_requires_functional_route(self):
        root = self._root(False)
        with mock.patch("tools.jenkins.frontend._ordinary_dispatch") as ordinary:
            with mock.patch("tools.jenkins.frontend.submit") as submit:
                with self.assertRaises(JenkinsError) as raised:
                    submit_entry(None, identity=self._identity(), repo_root=root, job="zircon-flow")
        self.assertEqual("functional_scope_required", raised.exception.code)
        ordinary.assert_not_called()
        submit.assert_not_called()

    def test_cli_forwards_functional_only_when_explicit_and_with_exact_fences(self):
        expected_driver = "d" * 64
        expected_runtime = "start-fixture"
        output = io.StringIO()
        with mock.patch("tools.jenkins.frontend.entry_from_environment",
                        side_effect=[{"status": "pending"}, {"status": "submitted"}]) as dispatch:
            with mock.patch("sys.stdout", output):
                ordinary_code = frontend_main(["dispatch", "--job", "zircon-flow"])
                functional_code = frontend_main([
                    "dispatch", "--job", "zircon-flow", "--functional",
                    "--expected-driver-digest", expected_driver,
                    "--expected-runtime-operation-id", expected_runtime])
        self.assertEqual(2, ordinary_code)
        self.assertEqual(0, functional_code)
        ordinary_call, functional_call = dispatch.call_args_list
        self.assertFalse(ordinary_call.kwargs["functional"])
        self.assertEqual("zircon-flow", ordinary_call.kwargs["job"])
        self.assertTrue(functional_call.kwargs["functional"])
        self.assertEqual(expected_driver, functional_call.kwargs["expected_driver_digest"])
        self.assertEqual(expected_runtime, functional_call.kwargs["expected_runtime_operation_id"])

    def test_active_gate_requires_identity(self):
        root = self._root(True)
        with self.assertRaisesRegex(JenkinsError, "identity"):
            submit_entry(State(root / ".jenkins" / "state" / "coordination.sqlite3"),
                         identity=None, repo_root=root)

    def test_unknown_http_outcome_is_not_retried(self):
        root = self._root(True)
        identity = self._identity()
        state = self._active_request(root, identity)
        calls = []

        def transport(request):
            calls.append(request)
            raise URLError("connection dropped after send")

        with self.assertRaisesRegex(JenkinsError, "unknown"):
            submit_entry(state, identity=identity, repo_root=root, transport=transport)
        self.assertEqual(1, len(calls))
        with self.assertRaisesRegex(JenkinsError, "reconciled"):
            submit_entry(state, identity=identity, repo_root=root, transport=transport)
        self.assertEqual(1, len(calls))

    def test_raw_command_and_acceptance_flags_are_rejected(self):
        root = self._root(True)
        identity = self._identity()
        state = self._active_request(root, identity)
        with self.assertRaisesRegex(JenkinsError, "raw command"):
            submit_entry(state, identity=identity, repo_root=root,
                         parameters={"command": "cargo test"})

    def _active_request(self, root: Path, identity: dict) -> State:
        state = State(root / ".jenkins" / "state" / "coordination.sqlite3")
        selector = json.loads((root / ".jenkins/state/deployment/driver.json").read_text(encoding="utf-8"))
        identity.update(attemptId=identity.get("attemptId", "attempt-1"), generation=1,
                        driverDigest=selector["driverDigest"], runtimeOperationId=selector["runtimeOperationId"])
        source_ref = identity.get("sealedInputRef", identity.get("sourceRef"))
        source_digest = identity.get("sourceInputDigest", identity.get("sourceDigest", source_ref))
        coverage_digest = identity["coverageDigest"]
        payload = {
            "repositoryId": identity["repositoryId"], "sessionId": identity["sessionId"],
            "requestId": identity["requestId"], "sourceRef": source_ref,
            "sourceDigest": source_digest, "sealedInputRef": source_ref,
            "coverageRef": identity.get("coverageRef", coverage_digest), "coverageDigest": coverage_digest,
            "attemptId": identity["attemptId"], "generation": identity["generation"],
            "driverDigest": identity["driverDigest"], "runtimeOperationId": identity["runtimeOperationId"],
            "buildRoot": identity.get("buildRoot", "D:/cargo-targets"), "parameters": {
                "sealedInputRef": source_ref, "patchOperationRef": "patch-1",
                "buildRoot": identity.get("buildRoot", "D:/cargo-targets"), "rustEdition": "2021"},
        }
        state.submit_request(payload)
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
        self.assertEqual(identity["sealedInputRef"], sent_parameters["SEALED_INPUT_REF"])
        self.assertNotIn("recipeRef", sent_parameters)
        # The production submit adapter maps those canonical fields to Jenkins
        # parameter names and keeps the side effect idempotent.
        calls.clear()
        # Use a fresh request for the transport mapping assertion; the first
        # request above is intentionally already claimed by the mocked entry.
        identity2 = dict(identity, requestId="request-2")
        state2 = self._active_request(root, identity2)
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
        identity = self._identity()
        state = self._active_request(root, identity)
        manager = mock.Mock(base_url="http://127.0.0.1:53748")
        manager._request_with_headers.return_value = (201, b"", {"Location": "http://127.0.0.1:53748/queue/item/11/"})
        spec, paths = object(), object()
        with mock.patch("tools.jenkins.deployment.spec.load_spec", return_value=spec) as load:
            with mock.patch("tools.jenkins.deployment.paths.resolve_paths", return_value=paths) as resolve:
                with mock.patch("tools.jenkins.deployment.manager.DeploymentManager", return_value=manager) as create:
                    with mock.patch("tools.jenkins.submission.urlopen", side_effect=AssertionError("unmanaged HTTP")):
                        result = submit_entry(state, identity=identity, repo_root=root)
        load.assert_called_once_with(root / ".jenkins" / "deployment-spec.json")
        resolve.assert_called_once_with(spec)
        create.assert_called_once_with(spec, paths)
        self.assertEqual("job/zircon-flow/buildWithParameters", manager._request_with_headers.call_args.args[0])
        self.assertEqual("POST", manager._request_with_headers.call_args.kwargs["method"])
        self.assertEqual("http://127.0.0.1:53748/queue/item/11/", result["queueUrl"])
        manager._request.assert_not_called()
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

    def _current_request(self, root: Path, *, prepared=True, runtime="start-fixture") -> tuple[State, dict]:
        state = State(root / ".jenkins/state/coordination.sqlite3")
        selector = json.loads((root / ".jenkins/state/deployment/driver.json").read_text(encoding="utf-8"))
        identity = {"repositoryId": "repo", "sessionId": "session", "requestId": "current-request",
                    "attemptId": "attempt-1", "generation": 1, "stageImplementationDigest": selector["driverDigest"]}
        authorization = state.authorize_session("repo", "session", "original-author", ["src"], ["validation"],
                                                {"source": "user", "task": "frontend validation fixture"})
        intent = {"schemaVersion": 1, "kind": "current-source", "identity": identity,
                  "repositoryRoot": str(root), "runtimeOperationId": runtime, "buildRoot": "D:/cargo-targets",
                  "currentHashes": {"src/main.rs": "c" * 64}, "externalInputs": [],
                  "authorizationSnapshot": authorization["snapshotDigest"]}
        reference = digest(intent)
        operation = "current-source-" + reference
        state.put("current_source_intent", reference, intent)
        state.put_operation("source_preparation", operation, intent)
        if prepared:
            state.transition_operation(operation, "prepared", "complete", {
                "sourceDigest": "a" * 64, "coverageDigest": "b" * 64, "canonicalSourceModified": False})
        payload = {**identity, "sourceInputDigest": "a" * 64, "sealedInputRef": "a" * 64,
                   "coverageDigest": "b" * 64, "currentSourceRef": reference, "sourcePreparationRef": operation,
                   "buildRoot": "D:/cargo-targets", "driverDigest": selector["driverDigest"],
                   "authorizationSnapshot": authorization["snapshotDigest"]}
        state.submit_request(payload)
        # Normal prepared identities can omit runtime; it belongs to intake and
        # must be derived from that State operation, never from the caller.
        return state, payload

    def _environment(self, root: Path, identity: dict):
        path = root / "identity.json"
        path.write_text(json.dumps(identity), encoding="utf-8")
        return mock.patch.dict(os.environ, {"ZIRCON_JENKINS_IDENTITY_FILE": str(path)})

    def test_current_source_top_level_refs_and_runtime_are_forwarded_from_state(self):
        root = self._root(True)
        state, identity = self._current_request(root)
        original = state.get_request("repo", "session", "current-request")["payload"]
        calls = []
        response = type("Response", (), {"status": 201, "headers": {"Location": "http://localhost/queue/item/31/"}})
        def transport(request):
            calls.append(parse_qs(request.data.decode()))
            return response()
        with self._environment(root, identity):
            first = entry_from_environment(root, transport=transport)
            second = entry_from_environment(root, transport=transport)
        self.assertEqual(1, len(calls))
        for field, value in (("CURRENT_SOURCE_REF", identity["currentSourceRef"]),
                             ("SOURCE_PREPARATION_REF", identity["sourcePreparationRef"]),
                             ("RUNTIME_OPERATION_ID", "start-fixture"), ("ATTEMPT_ID", "attempt-1"),
                             ("STAGE_IMPLEMENTATION_DIGEST", identity["stageImplementationDigest"]), ("GENERATION", "1")):
            self.assertEqual([value], calls[0][field])
        self.assertEqual(identity["currentSourceRef"], first["currentSourceRef"])
        self.assertEqual(identity["sourcePreparationRef"], first["sourcePreparationRef"])
        self.assertEqual("start-fixture", first["runtimeOperationId"])
        self.assertEqual(original, state.get_request("repo", "session", "current-request")["payload"])
        self.assertTrue(second["reused"])
        self.assertFalse(first["formalAcceptance"])
        self.assertNotIn("m8Accepted", first)
        self.assertNotIn("currentSourceRef", calls[0])

    def test_current_source_identity_file_cannot_replace_state_owned_fields(self):
        root = self._root(True)
        state, identity = self._current_request(root)
        for field in ("currentSourceRef", "sourcePreparationRef", "attemptId", "generation", "driverDigest", "runtimeOperationId"):
            with self.subTest(field=field), self._environment(root, {**identity, field: "foreign"}):
                with self.assertRaises(JenkinsError):
                    entry_from_environment(root, transport=lambda _: self.fail("unexpected HTTP"))
        self.assertEqual([], state.list("jenkins_submission"))

    def test_both_parameter_layers_reject_immutable_alias_overrides_before_delivery(self):
        root = self._root(True)
        state, identity = self._current_request(root)
        names = ("repositoryId", "REPOSITORY_ID", "sessionId", "SESSION_ID", "requestId", "REQUEST_ID",
                 "sealedInputRef", "SEALED_INPUT_REF", "sourceInputDigest", "SOURCE_INPUT_DIGEST",
                 "coverageDigest", "COVERAGE_DIGEST", "currentSourceRef", "CURRENT_SOURCE_REF",
                 "sourcePreparationRef", "SOURCE_PREPARATION_REF", "attemptId", "ATTEMPT_ID",
                 "generation", "GENERATION", "driverDigest", "STAGE_IMPLEMENTATION_DIGEST",
                 "runtimeOperationId", "RUNTIME_OPERATION_ID", "buildRoot", "BUILD_ROOT", "PATCH_REQUEST_REF")
        for layer in ("identity-file", "caller"):
            for name in names:
                with self.subTest(layer=layer, name=name):
                    file_identity = {**identity, **({"parameters": {name: "foreign"}} if layer == "identity-file" else {})}
                    caller = {name: "foreign"} if layer == "caller" else None
                    with self._environment(root, file_identity), self.assertRaises(JenkinsError):
                        entry_from_environment(root, parameters=caller, transport=lambda _: self.fail("unexpected HTTP"))
        self.assertEqual([], state.list("jenkins_submission"))

    def test_parameter_alias_conflict_is_rejected_even_when_later_alias_matches_state(self):
        root = self._root(True)
        state, identity = self._current_request(root)
        for parameters in ({"runtimeOperationId": "foreign", "RUNTIME_OPERATION_ID": "start-fixture"},
                           {"RUNTIME_OPERATION_ID": "start-fixture", "runtimeOperationId": "foreign"}):
            with self.subTest(parameters=parameters), self._environment(root, identity), self.assertRaises(JenkinsError):
                entry_from_environment(root, parameters=parameters, transport=lambda _: self.fail("unexpected HTTP"))
        self.assertEqual([], state.list("jenkins_submission"))

    def test_current_source_preparation_must_be_complete_and_owned_by_selected_runtime(self):
        for prepared, runtime in ((False, "start-fixture"), (True, "start-old-runtime")):
            with self.subTest(prepared=prepared, runtime=runtime):
                root = self._root(True)
                state, identity = self._current_request(root, prepared=prepared, runtime=runtime)
                with self._environment(root, identity), self.assertRaises(JenkinsError):
                    entry_from_environment(root, transport=lambda _: self.fail("unexpected HTTP"))
                self.assertEqual([], state.list("jenkins_submission"))

    def test_current_source_ref_must_match_preparation_and_immutable_intent_bytes(self):
        root = self._root(True)
        state, identity = self._current_request(root)
        intent = state.get("current_source_intent", identity["currentSourceRef"])
        state.put("current_source_intent", identity["currentSourceRef"], {**intent["payload"], "runtimeOperationId": "foreign"})
        with self._environment(root, identity), self.assertRaises(JenkinsError) as raised:
            entry_from_environment(root, transport=lambda _: self.fail("unexpected HTTP"))
        self.assertEqual("source_preparation_untrusted", raised.exception.code)
        self.assertEqual([], state.list("jenkins_submission"))

    def test_corrupt_selected_driver_or_redirected_launcher_rejects_before_delivery(self):
        for tamper in ("content", "launcher", "runtime"):
            with self.subTest(tamper=tamper):
                root = self._root(True)
                state, identity = self._current_request(root)
                path = root / ".jenkins/state/deployment/driver.json"
                selector = json.loads(path.read_text(encoding="utf-8"))
                if tamper == "content":
                    Path(selector["launcher"]).write_bytes(b"changed\n")
                elif tamper == "launcher":
                    selector["launcher"] = str(root / "foreign.py")
                    path.write_text(json.dumps(selector), encoding="utf-8")
                else:
                    selector["runtimeOperationId"] = "start-another"
                    path.write_text(json.dumps(selector), encoding="utf-8")
                with self._environment(root, identity), self.assertRaises(JenkinsError):
                    entry_from_environment(root, transport=lambda _: self.fail("unexpected HTTP"))
                self.assertEqual([], state.list("jenkins_submission"))

    def test_inventory_or_acceptance_alone_cannot_activate_sole_entry(self):
        """Both records must agree before callers are routed through the M8 gate.

        A half-activated gate stays open, so the observable evidence is ordinary
        routing rather than a dedicated pre-M8 refusal: the retired
        ``sole_entry_not_active`` error is gone, and an open gate never reaches
        the registered-entry checks that would need coordination State.
        """
        for inventory, acceptance in ((False, True), (True, False)):
            with self.subTest(inventory=inventory, acceptance=acceptance):
                root = self._root(inventory)
                path = root / ".jenkins/state/deployment/acceptance.json"
                path.write_text(json.dumps({"m8Accepted": acceptance, "soleEntryEnforcementApplied": acceptance}), encoding="utf-8")
                self.assertFalse(sole_entry_enforced(root))
                with self.assertRaises(JenkinsError) as raised:
                    entry_from_environment(root, transport=lambda _: self.fail("unexpected HTTP"))
                self.assertTrue(raised.exception.code.startswith("ordinary_"),
                                f"open gate must route to ordinary capability, got {raised.exception.code}")
                with self.assertRaises(JenkinsError) as raised:
                    submit_entry(State(root / ".jenkins/state/coordination.sqlite3"), identity=self._identity(), repo_root=root)
                self.assertTrue(raised.exception.code.startswith("ordinary_"),
                                f"open gate must route to ordinary capability, got {raised.exception.code}")

    def _functional_runtime(self, root: Path):
        from tools.jenkins.deployment.driver import verify_driver
        from tools.jenkins.deployment.manager import DeploymentManager
        selector_path = root / ".jenkins/state/deployment/driver.json"
        selector = json.loads(selector_path.read_text(encoding="utf-8"))
        target = verify_driver(selector["root"], expected_digest=selector["driverDigest"])
        release = {"schemaVersion": 1, "kind": "formal-owned-recovery-upgrade-release", "targetDriver": target}
        release_path = root / ".jenkins/state/deployment/fixture-release.json"
        release_path.write_text(json.dumps(release), encoding="utf-8")
        selector["recoveryRelease"] = {"path": str(release_path), "sha256": hashlib.sha256(release_path.read_bytes()).hexdigest()}
        selector_path.write_text(json.dumps(selector), encoding="utf-8")
        def native(pid, executable):
            return {"pid": pid, "creationTime": str(pid * 10), "executable": str(executable), "commandDigest": "d" * 64}
        operation = {**native(101, root / ".jenkins/runtime/python/python.exe"), "component": "lifecycle-host",
                     "hostPid": 101, "state": "running", "operationId": "start-fixture", "generation": "start-fixture",
                     "homePath": str(root / ".jenkins/jenkins_home"), "controlPlaneOnly": False}
        host = {"hostPid": 101, "status": "running", "operationId": "start-fixture", "generation": "start-fixture",
                "controlPlaneOnly": False, "agentLaunchAttempted": True,
                "controllerIdentity": native(102, root / ".jenkins/runtime/jdk/bin/java.exe"),
                "agentIdentity": native(103, root / ".jenkins/runtime/jdk/bin/java.exe")}
        (root / ".jenkins/state/deployment-operation.json").write_text(json.dumps(operation), encoding="utf-8")
        (root / ".jenkins/state/deployment/host.json").write_text(json.dumps(host), encoding="utf-8")
        manager = mock.Mock(spec=DeploymentManager, base_url="http://127.0.0.1:18080",
                            paths=SimpleNamespace(repo=root, state=root / ".jenkins/state", home=root / ".jenkins/jenkins_home"))
        manager.spec = SimpleNamespace(repository_root=root, controller={"listenAddress": "127.0.0.1", "httpPort": 18080})
        manager.health.return_value = {"controller": True, "agent": True, "plugins": True, "ready": True,
                                       "acceptingRequests": True, "quietingDown": False}
        manager._request_with_headers.return_value = (201, b"", {"Location": "http://127.0.0.1:18080/queue/item/41/"})
        return manager

    @staticmethod
    def _observe_native(expected):
        return {key: expected[key] for key in ("pid", "creationTime", "executable", "commandDigest")}

    def test_explicit_functional_route_proves_runtime_and_keeps_sole_gate_closed(self):
        root = self._root(False)
        state, identity = self._current_request(root)
        manager = self._functional_runtime(root)
        with self._environment(root, identity), mock.patch("tools.jenkins.deployment.live_observations.observe_live_identity", side_effect=self._observe_native) as native:
            with mock.patch("tools.jenkins.deployment.live_observations.observe_endpoint_owner", return_value={"pid": 102}) as endpoint:
                result = entry_from_environment(root, manager=manager, functional=True,
                                                expected_driver_digest=identity["driverDigest"],
                                                expected_runtime_operation_id="start-fixture")
        self.assertEqual(6, native.call_count)
        self.assertEqual(2, endpoint.call_count)
        manager.health.assert_called_once()
        fields = parse_qs(manager._request_with_headers.call_args.kwargs["data"].decode())
        for name in ("ALLOW_COMMIT", "ALLOW_PUSH", "ALLOW_NOTIFY"):
            self.assertEqual(["false"], fields[name])
        self.assertEqual([identity["currentSourceRef"]], fields["CURRENT_SOURCE_REF"])
        self.assertEqual("validation-only", result["functionalScope"])
        self.assertFalse(result["formalAcceptance"])
        self.assertFalse(sole_entry_enforced(root))
        self.assertNotIn("m8Accepted", result)

    def test_functional_route_requires_exact_explicit_fence_and_flow_job(self):
        root = self._root(False)
        state, identity = self._current_request(root)
        manager = self._functional_runtime(root)
        options = {"functional": True, "manager": manager, "expected_driver_digest": identity["driverDigest"],
                   "expected_runtime_operation_id": "start-fixture"}
        for change in ({"expected_driver_digest": None}, {"expected_runtime_operation_id": None},
                       {"expected_driver_digest": "f" * 64}, {"expected_runtime_operation_id": "start-foreign"},
                       {"job": "zircon-execution"}, {"job": "foreign-job"}, {"functional": "true"}):
            with self.subTest(change=change), self.assertRaises(JenkinsError):
                submit_entry(state, identity=identity, repo_root=root, **{**options, **change})
        manager._request_with_headers.assert_not_called()
        self.assertEqual([], state.list("jenkins_submission"))

    def test_functional_route_cannot_enable_commit_notify_or_raw_acceptance(self):
        root = self._root(False)
        state, identity = self._current_request(root)
        manager = self._functional_runtime(root)
        for name in ("ALLOW_COMMIT", "ALLOW_PUSH", "ALLOW_NOTIFY", "allowCommit", "allowNotify", "m8Accepted", "formalAcceptance", "command"):
            with self.subTest(name=name), self.assertRaises(JenkinsError):
                submit_entry(state, identity=identity, parameters={name: "true"}, repo_root=root, manager=manager,
                             functional=True, expected_driver_digest=identity["driverDigest"], expected_runtime_operation_id="start-fixture")
        manager._request_with_headers.assert_not_called()
        self.assertEqual([], state.list("jenkins_submission"))

    def test_functional_route_rejects_revoked_author_validation_grant(self):
        root = self._root(False)
        state, identity = self._current_request(root)
        manager = self._functional_runtime(root)
        state.revoke_session("repo", "session")
        with self.assertRaises(JenkinsError) as raised:
            submit_entry(state, identity=identity, repo_root=root, manager=manager, functional=True,
                         expected_driver_digest=identity["driverDigest"], expected_runtime_operation_id="start-fixture")
        self.assertEqual("action_not_authorized", raised.exception.code)
        manager._request_with_headers.assert_not_called()
        self.assertEqual([], state.list("jenkins_submission"))

    def test_functional_route_rejects_control_only_stale_or_changed_native_owner(self):
        for failure in ("control-only", "birth", "quiet", "owner-changed", "release"):
            with self.subTest(failure=failure):
                root = self._root(False)
                state, identity = self._current_request(root)
                manager = self._functional_runtime(root)
                if failure == "control-only":
                    path = root / ".jenkins/state/deployment-operation.json"
                    operation = json.loads(path.read_text(encoding="utf-8"))
                    operation["controlPlaneOnly"] = True
                    path.write_text(json.dumps(operation), encoding="utf-8")
                elif failure == "quiet":
                    manager.health.return_value = {"ready": True, "quietingDown": True}
                elif failure == "release":
                    (root / ".jenkins/state/deployment/fixture-release.json").write_text("{}", encoding="utf-8")
                count = 0
                def observe(value):
                    nonlocal count
                    count += 1
                    if failure == "birth":
                        raise JenkinsError("live_observation_failed", "fixture process birth changed")
                    observed = self._observe_native(value)
                    if failure == "owner-changed" and count == 4:
                        observed["creationTime"] = "foreign"
                    return observed
                with mock.patch("tools.jenkins.deployment.live_observations.observe_live_identity", side_effect=observe):
                    with mock.patch("tools.jenkins.deployment.live_observations.observe_endpoint_owner", return_value={"pid": 102}):
                        with self.assertRaises(JenkinsError):
                            submit_entry(state, identity=identity, repo_root=root, manager=manager, functional=True,
                                         expected_driver_digest=identity["driverDigest"], expected_runtime_operation_id="start-fixture")
                manager._request_with_headers.assert_not_called()
                self.assertEqual([], state.list("jenkins_submission"))


if __name__ == "__main__":
    unittest.main()
