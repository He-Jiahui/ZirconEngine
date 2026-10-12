"""Public reference intake on isolated metadata and synthetic CAS fixtures."""
from __future__ import annotations

import copy
import hashlib
import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.cli import dispatch
from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.resources import ApprovedBuildRoot
from tools.jenkins.source import get_sealed_manifest, register_sealed_consumer, repository_id
from tools.jenkins.state import State


class SealedConsumerIntakeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.context = self.root
        self.state = State(self.root / "fixture.sqlite3")
        self.repo = repository_id(self.root)
        self.build_root = self.root / "physical-build-fixture"
        self.approved = ApprovedBuildRoot(self.build_root)
        self.object_root = self.approved.namespace()
        self.source = self.root / "src/lib.rs"
        self.source.parent.mkdir()
        self.bytes = b"pub fn fixture() {}\n"
        self.source.write_bytes(self.bytes)
        checksum = hashlib.sha256(self.bytes).hexdigest()
        self.obj = self.object_root / "inputs/objects" / checksum
        self.obj.parent.mkdir(parents=True)
        self.obj.write_bytes(self.bytes)
        self.manifest = {"schemaVersion": 1, "repositoryId": self.repo, "baseHead": None,
                         "entries": [{"path": "src/lib.rs", "status": "present", "sha256": checksum,
                                      "objectDigest": checksum, "size": len(self.bytes)}],
                         "external": [], "declaredDependencies": []}
        self.source_ref = digest(self.manifest)
        coverage = {"template": "module_unit", "sourceDigest": self.source_ref}
        self.coverage_ref = digest(coverage)
        coverage["coverageDigest"] = self.coverage_ref
        self.sealed = {"status": "sealed", "repositoryId": self.repo, "repositoryRoot": str(self.root),
                       "sourceDigest": self.source_ref, "sourceManifest": self.manifest, "manifest": self.manifest,
                       "coverage": coverage, "coverageDigest": self.coverage_ref,
                       "owner": "original-attempt", "sessionId": "original-session",
                       "ownedPaths": ["src/lib.rs"], "objectRoot": str(self.object_root),
                       "buildRoot": str(self.build_root), "claims": [], "patchOperationRefs": []}
        self.state.put("sealed_input", self.source_ref, self.sealed)
        self.sessions = set()
        # Only the Windows physical-root adapter is replaced by a bounded
        # fixture adapter. Request, State, grant, manifest and CAS checks run.
        self.patches = ExitStack()
        self.addCleanup(self.patches.close)
        self.patches.enter_context(patch("tools.jenkins.source.consumer_intake.canonical_build_root",
                                         side_effect=self._build))
        self.patches.enter_context(patch("tools.jenkins.source.consumer_intake.physical_path_under",
                                         side_effect=self._physical))

    def _build(self, value):
        if not isinstance(value, str) or Path(value) != self.build_root:
            raise JenkinsError("build_root_not_approved", "Fixture build root differs")
        return self.approved

    def _physical(self, approved, value, *, allow_missing=True):
        selected = Path(value)
        if approved != self.approved or ".." in selected.parts or not selected.is_relative_to(self.build_root):
            raise JenkinsError("namespace_escape", "Fixture path escapes its physical build root")
        for part in (selected, *selected.parents):
            if part.is_symlink():
                raise JenkinsError("build_root_reparse_point", "Fixture path is an alias")
        if not allow_missing and not selected.exists():
            raise JenkinsError("physical_identity_unavailable", "Fixture object path is missing")
        return selected

    def _request(self, *, session="consumer-session", attempt="consumer-attempt", request="consumer-request",
                 paths=None, grant=True, identity_values=None, request_values=None, public=False):
        selected_paths = ["src/lib.rs"] if paths is None else paths
        if grant and session not in self.sessions:
            self.state.authorize_session(self.repo, session, "fixture-user", selected_paths, ["validation"],
                                         {"source": "user", "task": "sealed-consumer-fixture"})
            self.sessions.add(session)
        identity = {"repositoryId": self.repo, "sessionId": session, "requestId": request,
                    "attemptId": attempt, "generation": 1, "sourceInputDigest": self.source_ref,
                    "coverageDigest": self.coverage_ref, "stageImplementationDigest": "c" * 64}
        identity.update(identity_values or {})
        intent = {**identity, "ownedPaths": selected_paths, "buildRoot": str(self.build_root),
                  "sealedInputRef": identity["sourceInputDigest"]}
        if grant:
            intent["authorizationSnapshot"] = self.state.check_authorization(
                self.repo, session, "validation")["snapshotDigest"]
        intent.update(request_values or {})
        if grant and (not request_values or public):
            dispatch("request", "submit", intent, self.state, self.context)
        else:
            self.state.submit_request(intent)
        return {"identity": identity, "owner": attempt, "sourceDigest": identity["sourceInputDigest"],
                "manifestDigest": identity["sourceInputDigest"], "ownedPaths": []}

    def _register(self, payload):
        return dispatch("candidate", "register-consumer", payload, self.state, self.context)

    def _records(self):
        domains = ("sealed_consumer", "sealed_consumer_intake", "sealed_input", "path_claims",
                   "sealed_cargo_evidence", "candidate", "workflow", "execution", "cancellation",
                   "acceptance_checkout")
        return {domain: self.state.list(domain) for domain in domains}

    def _reject(self, payload, code=None):
        before = self._records()
        source, obj = self.source.read_bytes(), self.obj.read_bytes() if self.obj.exists() else None
        with self.assertRaises(JenkinsError) as caught:
            self._register(payload)
        if code:
            self.assertEqual(code, caught.exception.code)
        self.assertEqual(before, self._records())
        self.assertEqual(source, self.source.read_bytes())
        self.assertEqual(obj, self.obj.read_bytes() if self.obj.exists() else None)

    def _sealed_update(self, values):
        self.state.put("sealed_input", self.source_ref, {**copy.deepcopy(self.sealed), **values})

    def _formal_runtime(self):
        marker = self.context / ".jenkins/state/deployment/driver.json"
        marker.parent.mkdir(parents=True, exist_ok=True)
        marker.write_text("{}", encoding="utf-8")
        self.active = {"digest": "d" * 64, "runtimeOperationId": "current-driver-D"}
        self.runtime_mock = self.patches.enter_context(patch(
            "tools.jenkins.source.consumer_intake.active_driver_binding", return_value=self.active))

    def _steps_payload(self, *, request="steps-request", attempt="steps-attempt", session="steps-session"):
        values = {"driverDigest": self.active["digest"], "runtimeOperationId": self.active["runtimeOperationId"]}
        if hasattr(self, "checkout_ref"):
            values.update(checkoutBindingRef=self.checkout_ref, repositoryRoot=str(self.root))
        payload = self._request(session=session, request=request, attempt=attempt, public=True,
                                identity_values={"stageImplementationDigest": self.active["digest"]},
                                request_values=values)
        return {**payload, **values}

    def _register_flow(self, payload):
        flow_payload = {"identity": payload["identity"], "sealedInputRef": self.source_ref,
                        "ownedPaths": ["src/lib.rs"], "buildRoot": str(self.build_root),
                        "driverDigest": self.active["digest"],
                        "runtimeOperationId": self.active["runtimeOperationId"],
                        "checkoutBindingRef": self.checkout_ref, "repositoryRoot": str(self.root)}
        with patch("tools.jenkins.workflow.handler._trusted_driver", return_value=self.active["digest"]), \
                patch("tools.jenkins.resources.canonical_build_root", side_effect=self._build):
            return dispatch("flow", "register-flow", flow_payload, self.state, self.context)

    def _historical_checkout(self):
        from tools.jenkins.patch_context import register_acceptance_checkout
        self.root = self.context / ".jenkins/tmp/acceptance/original"
        self.source = self.root / "src/lib.rs"
        self.source.parent.mkdir(parents=True)
        self.source.write_bytes(self.bytes)
        self.repo = repository_id(self.root)
        self.author = {"repositoryId": self.repo, "sessionId": "original-session", "requestId": "original-request",
                       "attemptId": "original-attempt", "generation": 1, "stageImplementationDigest": "b" * 64}
        self.state.authorize_session(self.repo, self.author["sessionId"], "original-user", ["src"],
                                     ["implementation", "validation", "deployment"],
                                     {"source": "user", "task": "original synthetic acceptance checkout"})
        old_runtime = {"digest": "b" * 64, "runtimeOperationId": "historical-driver-B"}
        with patch("tools.jenkins.patch_context.active_driver_binding", return_value=old_runtime):
            binding = register_acceptance_checkout(self.state, self.context,
                         {"identity": self.author, "checkoutRoot": str(self.root), "ownedPaths": ["src"]})
        self.checkout_ref = binding["checkoutBindingRef"]
        self.manifest = {**self.manifest, "repositoryId": self.repo}
        self.source_ref = digest(self.manifest)
        coverage = {"template": "module_unit", "sourceDigest": self.source_ref}
        self.coverage_ref = digest(coverage)
        coverage["coverageDigest"] = self.coverage_ref
        self.patch_ref = "historical-complete-patch"
        self.sealed = {**self.sealed, "repositoryId": self.repo, "repositoryRoot": str(self.root),
                       "sourceDigest": self.source_ref, "sourceManifest": self.manifest, "manifest": self.manifest,
                       "coverage": coverage, "coverageDigest": self.coverage_ref,
                       "patchOperationRefs": [self.patch_ref]}
        self.state.put("sealed_input", self.source_ref, self.sealed)
        after = self.manifest["entries"][0]["sha256"]
        before = hashlib.sha256(b"pub fn fixture_before() {}\n").hexdigest()
        operation = {"owner": self.author["attemptId"], "paths": ["src/lib.rs"],
                     "beforeHashes": {"src/lib.rs": before}, "patchDigest": digest({"src/lib.rs": self.bytes.decode()}),
                     "repositoryRoot": str(self.root), "objectRoot": str(self.object_root),
                     "authorization": {**self.author, "owner": self.author["attemptId"]},
                     "beforeObjects": {"src/lib.rs": before}, "expectedAfterHashes": {"src/lib.rs": after}}
        self.state.put_operation("source_patch", self.patch_ref, operation)
        self.state.transition_operation(self.patch_ref, "prepared", "complete",
                                         {"beforeHashes": {"src/lib.rs": before}, "afterHashes": {"src/lib.rs": after}})
        register_sealed_consumer(self.state, repository_id_value=self.repo, session_id=self.author["sessionId"],
                                 owner=self.author["attemptId"], source_digest=self.source_ref,
                                 manifest_digest=self.source_ref)
        self._formal_runtime()

    def test_public_cli_shares_exact_input_across_three_bindings_without_execution(self):
        payloads = [self._request(request="request-1", attempt="attempt-1"),
                    self._request(request="request-2", attempt="attempt-2"),
                    self._request(session="other-session", request="request-3", attempt="attempt-3")]
        before = self._records()
        with ExitStack() as guard:
            for name in ("claim_paths", "apply_owned_patch", "seal_candidate", "materialize_manifest",
                         "collect_cargo_evidence"):
                guard.enter_context(patch("tools.jenkins.source." + name, side_effect=AssertionError(name)))
            guard.enter_context(patch("subprocess.run", side_effect=AssertionError("native execution")))
            guard.enter_context(patch("subprocess.Popen", side_effect=AssertionError("native execution")))
            guard.enter_context(patch("shutil.copy", side_effect=AssertionError("source copy")))
            guard.enter_context(patch("shutil.copy2", side_effect=AssertionError("source copy")))
            guard.enter_context(patch("shutil.copyfile", side_effect=AssertionError("source copy")))
            guard.enter_context(patch("shutil.copytree", side_effect=AssertionError("source copy")))
            guard.enter_context(patch.object(Path, "write_bytes", side_effect=AssertionError("source write")))
            guard.enter_context(patch.object(Path, "write_text", side_effect=AssertionError("source write")))
            results = [self._register(payload) for payload in payloads]
        self.assertEqual(["registered"] * 3, [result["status"] for result in results])
        self.assertEqual(3, len(self.state.list("sealed_consumer")))
        self.assertEqual(3, len(self.state.list("sealed_consumer_intake")))
        self.assertEqual(self.bytes, self.source.read_bytes())
        self.assertEqual(self.bytes, self.obj.read_bytes())
        after = self._records()
        for domain in before:
            if domain not in {"sealed_consumer", "sealed_consumer_intake"}:
                self.assertEqual(before[domain], after[domain])
        for payload in payloads:
            identity = payload["identity"]
            manifest = get_sealed_manifest(self.state, repository_id_value=self.repo,
                                           session_id=identity["sessionId"], owner=identity["attemptId"],
                                           source_digest=self.source_ref)
            self.assertEqual(self.source_ref, manifest.source_digest)
            with self.assertRaises(JenkinsError) as caught:
                self.state.check_authorization(self.repo, identity["sessionId"], "implementation", ["src/lib.rs"])
            self.assertEqual("action_not_authorized", caught.exception.code)

    def test_identical_retry_preserves_versions_and_legacy_helper_payload(self):
        payload = self._request()
        identity = payload["identity"]
        legacy = register_sealed_consumer(self.state, repository_id_value=self.repo,
                                          session_id=identity["sessionId"], owner=identity["attemptId"],
                                          source_digest=self.source_ref, manifest_digest=self.source_ref)
        first = self._register(payload)
        before = self._records()
        again = self._register(copy.deepcopy(payload))
        self.assertFalse(first["reused"])
        self.assertTrue(again["reused"])
        self.assertEqual(legacy, again["consumer"])
        self.assertEqual(before, self._records())
        self.assertEqual(legacy, register_sealed_consumer(self.state, repository_id_value=self.repo,
                         session_id=identity["sessionId"], owner=identity["attemptId"],
                         source_digest=self.source_ref, manifest_digest=self.source_ref))

    def test_missing_validation_grant_is_rejected_before_consumer_rows(self):
        self._reject(self._request(grant=False), "action_not_authorized")

    def test_empty_caller_paths_cannot_bypass_sealed_scope_grant(self):
        payload = self._request(paths=["unrelated"])
        self.assertEqual([], payload["ownedPaths"])
        self._reject(payload, "path_not_authorized")

    def test_authorization_revoked_after_cli_check_is_rechecked_in_transaction(self):
        payload = self._request()
        def revoke_after_boundary(domain, action, selected, root):
            self.state.revoke_session(self.repo, payload["identity"]["sessionId"])
            return selected
        with patch("tools.jenkins.cli.configured_build_payload", side_effect=revoke_after_boundary):
            self._reject(payload, "action_not_authorized")

    def test_request_must_exist_at_exact_generation_attempt_and_identity(self):
        payload = self._request()
        cases = [("requestId", "missing-request", "request_not_registered"),
                 ("generation", 2, "request_generation_mismatch"),
                 ("attemptId", "different-attempt", "request_identity_mismatch"),
                 ("coverageDigest", "d" * 64, "request_identity_mismatch"),
                 ("stageImplementationDigest", "e" * 64, "request_identity_mismatch")]
        for field, value, code in cases:
            with self.subTest(field=field):
                altered = copy.deepcopy(payload)
                altered["identity"][field] = value
                if field == "attemptId":
                    altered["owner"] = value
                self._reject(altered, code)
        altered = copy.deepcopy(payload)
        altered["identity"]["generation"] = True
        self._reject(altered, "generation_invalid")

    def test_missing_or_conflicting_source_manifest_owner_and_control_identity(self):
        payload = self._request()
        for field in ("owner", "sourceDigest", "manifestDigest"):
            with self.subTest(missing=field):
                altered = copy.deepcopy(payload)
                del altered[field]
                self._reject(altered, "invalid_identity" if field == "owner" else "invalid_digest")
        for values, code in (({"owner": "other-owner"}, "consumer_owner_mismatch"),
                             ({"sourceDigest": "a" * 64}, "sealed_input_digest_mismatch"),
                             ({"manifestDigest": "a" * 64}, "sealed_input_digest_mismatch"),
                             ({"manifestDigest": "A" * 64}, "invalid_digest"),
                             ({"sealedInputRef": "a" * 64}, "sealed_input_digest_mismatch"),
                             ({"sessionId": "other-session"}, "identity_conflict")):
            with self.subTest(values=values):
                self._reject({**payload, **values}, code)

    def test_registered_source_coverage_scope_and_authorization_cannot_change(self):
        for index, values in enumerate(({"sourceInputDigest": "a" * 64},
                                        {"coverageDigest": "a" * 64},
                                        {"ownedPaths": ["other.rs"]},
                                        {"authorizationSnapshot": "a" * 64})):
            with self.subTest(values=values):
                payload = self._request(request="request-" + str(index), attempt="attempt-" + str(index),
                                        request_values=values)
                self._reject(payload)
        payload = self._request(request="crossed-source", identity_values={"sourceInputDigest": "a" * 64})
        self._reject(payload, "sealed_input_missing")
        payload = self._request(request="crossed-coverage", identity_values={"coverageDigest": "a" * 64})
        self._reject(payload, "coverage_identity_mismatch")

    def test_caller_cannot_replace_manifest_paths_roots_or_prepare_source(self):
        payload = self._request()
        for values, code in (({"manifest": self.manifest}, "consumer_input_invalid"),
                             ({"sourceManifest": self.manifest}, "consumer_input_invalid"),
                             ({"claims": []}, "consumer_input_invalid"),
                             ({"patch": {}}, "consumer_input_invalid"),
                             ({"ownedPaths": None}, "consumer_input_invalid"),
                             ({"ownedPaths": ["other.rs"]}, "path_not_authorized"),
                             ({"buildRoot": str(self.root / "other-root")}, "build_root_not_approved"),
                             ({"objectRoot": str(self.root / "other-objects")}, "namespace_escape")):
            with self.subTest(values=values):
                self._reject({**payload, **values}, code)

    def test_authorized_alternate_path_cannot_replace_the_sealed_owned_scope(self):
        payload = self._request(paths=["src"], request_values={"ownedPaths": ["src/lib.rs"]})
        self._reject({**payload, "ownedPaths": ["src/other.rs"]}, "consumer_identity_conflict")

    def test_missing_corrupt_or_wrong_repository_sealed_records_leave_no_bindings(self):
        payload = self._request()
        changed_manifest = copy.deepcopy(self.manifest)
        changed_manifest["entries"][0]["sha256"] = "a" * 64
        cases = [({"status": "pending"}, "sealed_input_missing"),
                 ({"sourceDigest": "a" * 64}, "sealed_input_corrupt"),
                 ({"sourceManifest": changed_manifest}, "sealed_input_corrupt"),
                 ({"manifest": changed_manifest}, "sealed_input_corrupt"),
                 ({"repositoryId": "other-repository"}, "sealed_input_repository_mismatch"),
                 ({"repositoryRoot": str(self.root / "other-repository")}, "sealed_repository_mismatch"),
                 ({"coverageDigest": "a" * 64}, "coverage_identity_mismatch"),
                 ({"ownedPaths": []}, "sealed_scope_missing"),
                 ({"objectRoot": str(self.root / "other-store")}, "namespace_escape"),
                 ({"buildRoot": str(self.root / "other-build")}, "build_root_not_approved")]
        for values, code in cases:
            with self.subTest(values=values):
                self._sealed_update(values)
                self._reject(payload, code)
        self.state.delete("sealed_input", self.source_ref)
        self._reject(payload, "sealed_input_missing")

    def test_physical_repository_alias_is_rejected(self):
        payload = self._request()
        with patch("tools.jenkins.source.consumer_intake._plain",
                   side_effect=JenkinsError("driver_alias_rejected", "fixture alias")):
            self._reject(payload, "driver_alias_rejected")

    def test_matching_source_at_another_repository_cannot_be_attached(self):
        payload = self._request()
        other = self.root / "other-repository"
        other.mkdir()
        before = self._records()
        with self.assertRaises(JenkinsError) as caught:
            dispatch("candidate", "register-consumer", payload, self.state, other)
        self.assertEqual("source_repository_mismatch", caught.exception.code)
        self.assertEqual(before, self._records())

    def test_corrupt_or_missing_cas_object_is_rejected_before_registration(self):
        payload = self._request()
        self.obj.write_bytes(b"tampered sealed object")
        self._reject(payload, "input_object_missing")
        self.obj.unlink()
        self._reject(payload, "physical_identity_unavailable")

    def test_request_cancellation_preserves_historical_consumer_references(self):
        payload = self._request()
        self._register(payload)
        identity = payload["identity"]
        dispatch("request", "cancel", {key: identity[key] for key in ("repositoryId", "sessionId", "requestId")},
                 self.state, self.root)
        self._reject(payload, "request_cancelled")
        self.assertEqual("active", self.state.list("sealed_consumer")[0]["payload"]["status"])

    def test_cancelled_request_and_released_consumer_cannot_be_reactivated(self):
        payload = self._request()
        identity = payload["identity"]
        self.state.transition_request(self.repo, identity["sessionId"], identity["requestId"], 1,
                                      "pending", "cancelled")
        self._reject(payload, "request_cancelled")
        other = self._request(request="active-request", attempt="active-attempt")
        first = self._register(other)["consumer"]
        self.state.put("sealed_consumer", first["key"], {**first["payload"], "status": "released"})
        self._reject(other, "consumer_identity_conflict")

    def test_changed_public_binding_cannot_reuse_consumer_key(self):
        payload = self._request()
        payload["operationId"] = "consumer-operation"
        self._register(payload)
        self._reject({**payload, "operationId": "another-operation"}, "consumer_identity_conflict")
        other_request = self._request(request="another-request")
        self._reject(other_request, "consumer_identity_conflict")

    def test_intake_write_failure_rolls_back_consumer_insert(self):
        payload = self._request()
        original = self.state.put
        def fail_intake(domain, key, value, **kwargs):
            if domain == "sealed_consumer_intake":
                raise JenkinsError("fixture_write_failure", "Inject intake write failure")
            return original(domain, key, value, **kwargs)
        with patch.object(self.state, "put", side_effect=fail_intake):
            self._reject(payload, "fixture_write_failure")
        self.assertEqual([], self.state.list("sealed_consumer"))
        self.assertEqual([], self.state.list("sealed_consumer_intake"))

    def test_actual_steps_payload_binds_top_level_driver_without_inner_duplication(self):
        self._formal_runtime()
        payload = self._steps_payload()
        self.assertNotIn("driverDigest", payload["identity"])
        result = self._register(payload)
        self.assertEqual("registered", result["status"])
        stored = self.state.list("sealed_consumer_intake")[0]["payload"]
        self.assertEqual("current-driver-D", stored["runtimeOperationId"])
        self.assertEqual("d" * 64, stored["driverDigest"])
        self.runtime_mock.assert_called_with(self.context)

    def test_formal_steps_require_current_driver_and_runtime(self):
        self._formal_runtime()
        payload = self._steps_payload()
        for values, code in (({"runtimeOperationId": "historical-driver-B"}, "consumer_runtime_mismatch"),
                             ({"driverDigest": "b" * 64}, "driver_context_mismatch")):
            with self.subTest(values=values):
                self._reject({**payload, **values}, code)
        for missing in ("runtimeOperationId", "driverDigest"):
            with self.subTest(missing=missing):
                altered = copy.deepcopy(payload)
                del altered[missing]
                self._reject(altered, "invalid_identity" if missing == "runtimeOperationId" else "invalid_digest")
        wrong_stage = self._request(request="old-stage", attempt="old-attempt",
                         identity_values={"stageImplementationDigest": "b" * 64})
        self._reject({**wrong_stage, "driverDigest": "d" * 64, "runtimeOperationId": "current-driver-D"},
                     "driver_context_mismatch")

    def test_formal_missing_active_binding_cannot_accept_caller_runtime(self):
        self._formal_runtime()
        payload = self._steps_payload()
        self.runtime_mock.side_effect = JenkinsError("active_driver_unavailable", "fixture unavailable")
        self._reject(payload, "active_driver_unavailable")

    def test_registered_runtime_pins_cannot_be_replaced_by_steps(self):
        self._formal_runtime()
        for index, values in enumerate(({"runtimeOperationId": "historical-driver-B"}, {"driverDigest": "b" * 64})):
            with self.subTest(values=values):
                payload = self._request(request="registered-runtime-" + str(index), attempt="runtime-attempt-" + str(index),
                                        identity_values={"stageImplementationDigest": "d" * 64}, request_values=values)
                self._reject({**payload, "runtimeOperationId": "current-driver-D", "driverDigest": "d" * 64},
                             "request_identity_mismatch")

    def test_unregistered_repository_root_cannot_select_another_checkout(self):
        payload = self._request()
        self._reject({**payload, "repositoryRoot": str(self.root / "unregistered")}, "consumer_checkout_mismatch")

    def test_historical_child_serves_three_current_validation_consumers_without_source_copies(self):
        self._historical_checkout()
        # Historical author authority is retained after revocation; a new
        # consumer uses only its own live validation scope under the new driver.
        self.state.revoke_session(self.repo, self.author["sessionId"])
        payloads = [self._steps_payload(request="child-request-1", attempt="child-attempt-1"),
                    self._steps_payload(request="child-request-2", attempt="child-attempt-2"),
                    self._steps_payload(session="child-other-session", request="child-request-3", attempt="child-attempt-3")]
        before = self._records()
        journal = self.state.get_operation(self.patch_ref)
        with ExitStack() as guard:
            for name in ("claim_paths", "apply_owned_patch", "seal_candidate", "materialize_manifest", "collect_cargo_evidence"):
                guard.enter_context(patch("tools.jenkins.source." + name, side_effect=AssertionError(name)))
            guard.enter_context(patch("tools.jenkins.patch_context.register_acceptance_checkout",
                                      side_effect=AssertionError("checkout registration")))
            guard.enter_context(patch("tools.jenkins.patch_context.active_driver_binding",
                                      side_effect=AssertionError("historical runtime adoption")))
            guard.enter_context(patch("subprocess.run", side_effect=AssertionError("native execution")))
            guard.enter_context(patch("subprocess.Popen", side_effect=AssertionError("native execution")))
            guard.enter_context(patch("shutil.copy", side_effect=AssertionError("source copy")))
            guard.enter_context(patch("shutil.copy2", side_effect=AssertionError("source copy")))
            guard.enter_context(patch("shutil.copyfile", side_effect=AssertionError("source copy")))
            guard.enter_context(patch("shutil.copytree", side_effect=AssertionError("source copy")))
            guard.enter_context(patch.object(Path, "write_bytes", side_effect=AssertionError("source write")))
            guard.enter_context(patch.object(Path, "write_text", side_effect=AssertionError("source write")))
            results = [self._register(payload) for payload in payloads]
        self.assertEqual(["registered"] * 3, [result["status"] for result in results])
        self.assertEqual(4, len(self.state.list("sealed_consumer")))
        self.assertEqual(3, len(self.state.list("sealed_consumer_intake")))
        self.assertEqual([self.root], list((self.context / ".jenkins/tmp/acceptance").iterdir()))
        self.assertEqual(self.bytes, self.source.read_bytes())
        self.assertEqual(self.bytes, self.obj.read_bytes())
        self.assertEqual(journal, self.state.get_operation(self.patch_ref))
        after = self._records()
        for domain in before:
            if domain not in {"sealed_consumer", "sealed_consumer_intake"}:
                self.assertEqual(before[domain], after[domain])
        for payload in payloads:
            identity = payload["identity"]
            with self.assertRaises(JenkinsError) as caught:
                self.state.check_authorization(self.repo, identity["sessionId"], "implementation", ["src/lib.rs"])
            self.assertEqual("action_not_authorized", caught.exception.code)
        for row in self.state.list("sealed_consumer_intake"):
            intake = row["payload"]
            self.assertEqual(self.checkout_ref, intake["sourceCheckout"]["checkoutBindingRef"])
            self.assertEqual("b" * 64, intake["sourceCheckout"]["originalIdentity"]["stageImplementationDigest"])
            self.assertEqual("d" * 64, intake["driverDigest"])

    def test_historical_child_requires_request_and_payload_original_binding_reference(self):
        self._historical_checkout()
        payload = self._steps_payload()
        self._reject({**payload, "checkoutBindingRef": "a" * 64}, "consumer_checkout_mismatch")
        missing = copy.deepcopy(payload)
        del missing["checkoutBindingRef"]
        self._reject(missing, "invalid_digest")
        absent = self._request(request="unbound-child", attempt="unbound-child-attempt",
                         identity_values={"stageImplementationDigest": "d" * 64},
                         request_values={"repositoryRoot": str(self.root), "driverDigest": "d" * 64,
                                         "runtimeOperationId": "current-driver-D"}, public=True)
        self._reject({**absent, "checkoutBindingRef": self.checkout_ref, "driverDigest": "d" * 64,
                      "runtimeOperationId": "current-driver-D"}, "consumer_checkout_mismatch")
        self._reject({**payload, "repositoryRoot": str(self.context / "other")}, "consumer_checkout_mismatch")

    def test_historical_child_public_intake_then_flow_preserves_source_and_cas_authority(self):
        self._historical_checkout()
        self.state.revoke_session(self.repo, self.author["sessionId"])
        payload = self._steps_payload()
        original = self._records()
        journal = self.state.get_operation(self.patch_ref)
        with ExitStack() as guard:
            for name in ("claim_paths", "apply_owned_patch", "seal_candidate", "materialize_manifest", "collect_cargo_evidence"):
                guard.enter_context(patch("tools.jenkins.source." + name, side_effect=AssertionError(name)))
            guard.enter_context(patch("tools.jenkins.patch_context.register_acceptance_checkout",
                                      side_effect=AssertionError("checkout registration")))
            guard.enter_context(patch("tools.jenkins.patch_context.active_driver_binding",
                                      side_effect=AssertionError("historical runtime adoption")))
            guard.enter_context(patch("subprocess.run", side_effect=AssertionError("native execution")))
            guard.enter_context(patch("subprocess.Popen", side_effect=AssertionError("native execution")))
            guard.enter_context(patch("shutil.copytree", side_effect=AssertionError("source copy")))
            guard.enter_context(patch.object(Path, "write_bytes", side_effect=AssertionError("source write")))
            guard.enter_context(patch.object(Path, "write_text", side_effect=AssertionError("source write")))
            intake = self._register(payload)
            result = self._register_flow(payload)
        self.assertEqual("registered", intake["status"])
        self.assertEqual("pending", result["status"])
        rows = self.state.list("workflow")
        self.assertEqual(1, len(rows))
        flow = rows[0]["payload"]
        self.assertEqual(payload["identity"], flow["identity"])
        self.assertEqual(str(self.root), flow["repoRoot"])
        self.assertEqual(self.repo, flow["repositoryId"])
        self.assertEqual(str(self.object_root), flow["objectRoot"])
        self.assertEqual(str(self.build_root), flow["buildRoot"])
        self.assertEqual(self.source_ref, flow["sourceDigest"])
        self.assertEqual(self.coverage_ref, flow["coverageDigest"])
        self.assertEqual(self.manifest, flow["sourceManifest"])
        self.assertEqual(self.obj.name, flow["sourceManifest"]["entries"][0]["objectDigest"])
        self.assertEqual([self.patch_ref], flow["patchOperationRefs"])
        self.assertEqual("d" * 64, flow["driverDigest"])
        self.assertEqual("b" * 64, self.state.get("acceptance_checkout", self.checkout_ref)["payload"]["driverDigest"])
        self.assertEqual(journal, self.state.get_operation(self.patch_ref))
        for domain, records in original.items():
            if domain not in {"workflow", "sealed_consumer", "sealed_consumer_intake"}:
                self.assertEqual(records, self.state.list(domain))
        self.assertEqual(self.bytes, self.source.read_bytes())
        self.assertEqual(self.bytes, self.obj.read_bytes())
        self.assertEqual([self.root], list((self.context / ".jenkins/tmp/acceptance").iterdir()))
        with self.assertRaises(JenkinsError) as caught:
            self.state.check_authorization(self.repo, payload["identity"]["sessionId"], "implementation", ["src/lib.rs"])
        self.assertEqual("action_not_authorized", caught.exception.code)

    def test_historical_binding_tampered_after_intake_denies_public_flow_before_write(self):
        self._historical_checkout()
        self.state.revoke_session(self.repo, self.author["sessionId"])
        payload = self._steps_payload()
        self.assertEqual("registered", self._register(payload)["status"])
        binding = self.state.get("acceptance_checkout", self.checkout_ref)
        self.state.put("acceptance_checkout", self.checkout_ref,
                       {**binding["payload"], "ownedPaths": ["unrelated"]}, expected_version=binding["version"])
        before = self._records()
        journal = self.state.get_operation(self.patch_ref)
        with self.assertRaises(JenkinsError) as caught:
            self._register_flow(payload)
        self.assertEqual("consumer_checkout_mismatch", caught.exception.code)
        self.assertEqual(before, self._records())
        self.assertEqual([], self.state.list("workflow"))
        self.assertEqual(journal, self.state.get_operation(self.patch_ref))
        self.assertEqual(self.bytes, self.source.read_bytes())
        self.assertEqual(self.bytes, self.obj.read_bytes())

    def test_historical_child_rejects_corrupt_binding_identity_context_and_scope(self):
        self._historical_checkout()
        payload = self._steps_payload()
        original = self.state.get("acceptance_checkout", self.checkout_ref)["payload"]
        changes = [{"contextRoot": str(self.context / "foreign-context")},
                   {"checkoutRoot": str(self.context / "unregistered")},
                   {"identity": {**self.author, "sessionId": "foreign-author"}},
                   {"identity": {**self.author, "repositoryId": repository_id(self.context)}},
                   {"ownedPaths": ["unrelated"]}, {"driverDigest": "a" * 64},
                   {"identity": {**self.author, "stageImplementationDigest": "d" * 64}, "driverDigest": "d" * 64},
                   {"productRepositoryAcceptance": True}]
        for values in changes:
            with self.subTest(values=values):
                self.state.put("acceptance_checkout", self.checkout_ref, {**original, **values})
                self._reject(payload, "consumer_checkout_mismatch")
        self.state.delete("acceptance_checkout", self.checkout_ref)
        self._reject(payload, "consumer_checkout_mismatch")

    def test_historical_child_cannot_reuse_another_authors_registered_binding(self):
        self._historical_checkout()
        original = self.state.get("acceptance_checkout", self.checkout_ref)["payload"]
        foreign_author = {**self.author, "sessionId": "different-original-session", "requestId": "different-original-request",
                          "attemptId": "different-original-attempt"}
        foreign_ref = digest({key: foreign_author[key] for key in
                              ("repositoryId", "sessionId", "requestId", "attemptId", "generation")})
        self.state.put("acceptance_checkout", foreign_ref, {**original, "identity": foreign_author})
        payload = self._request(request="cross-author-request", attempt="cross-author-attempt", public=True,
                         identity_values={"stageImplementationDigest": "d" * 64},
                         request_values={"checkoutBindingRef": foreign_ref, "repositoryRoot": str(self.root),
                                         "runtimeOperationId": "current-driver-D", "driverDigest": "d" * 64})
        self._reject({**payload, "checkoutBindingRef": foreign_ref, "runtimeOperationId": "current-driver-D",
                      "driverDigest": "d" * 64}, "consumer_checkout_mismatch")

    def test_historical_child_rejects_wrong_sealed_author_root_or_missing_journals(self):
        self._historical_checkout()
        payload = self._steps_payload()
        for values in ({"sessionId": "foreign-author"}, {"owner": "foreign-attempt"},
                       {"repositoryId": repository_id(self.context)}, {"repositoryRoot": str(self.context)},
                       {"patchOperationRefs": []}, {"patchOperationRefs": ["missing-complete-journal"]}):
            with self.subTest(values=values):
                self._sealed_update(values)
                self._reject(payload, "consumer_checkout_mismatch")

    def test_historical_child_rejects_incomplete_or_foreign_source_journal(self):
        self._historical_checkout()
        payload = self._steps_payload()
        original = self.state.get_operation(self.patch_ref)
        for values in ({"status": "prepared"}, {"kind": "foreign-operation"},
                       {"payload": {**original["payload"], "owner": "foreign-attempt"}},
                       {"payload": {**original["payload"], "repositoryRoot": str(self.context)}},
                       {"result": {**original["result"], "afterHashes": {"src/lib.rs": "a" * 64}}}):
            with self.subTest(values=values):
                altered = {**original, **values}
                if "payload" in values:
                    altered["payloadDigest"] = digest(altered["payload"])
                original_get = self.state.get_operation
                def journal_lookup(operation_id, *, connection=None):
                    return altered if operation_id == self.patch_ref else original_get(operation_id, connection=connection)
                with patch.object(self.state, "get_operation", side_effect=journal_lookup):
                    self._reject(payload, "consumer_checkout_mismatch")

    def test_historical_child_still_requires_current_consumer_scope_runtime_and_uncancelled_request(self):
        self._historical_checkout()
        payload = self._steps_payload()
        self._reject({**payload, "runtimeOperationId": "historical-driver-B"}, "consumer_runtime_mismatch")
        denied = self._request(session="ungranted-child", request="ungranted-child", attempt="ungranted-child",
                              paths=["unrelated"], identity_values={"stageImplementationDigest": "d" * 64},
                              request_values={"checkoutBindingRef": self.checkout_ref, "repositoryRoot": str(self.root),
                                              "runtimeOperationId": "current-driver-D", "driverDigest": "d" * 64}, public=True)
        self._reject({**denied, "checkoutBindingRef": self.checkout_ref, "runtimeOperationId": "current-driver-D",
                      "driverDigest": "d" * 64}, "path_not_authorized")
        identity = payload["identity"]
        dispatch("request", "cancel", {key: identity[key] for key in ("repositoryId", "sessionId", "requestId")},
                 self.state, self.context)
        self._reject(payload, "request_cancelled")

    def test_historical_child_revalidates_manifest_cas_before_adding_consumer(self):
        self._historical_checkout()
        payload = self._steps_payload()
        changed = {**self.manifest, "baseHead": "changed-base"}
        self._sealed_update({"sourceManifest": changed, "manifest": changed})
        self._reject(payload, "sealed_input_corrupt")
        self._sealed_update({})
        self.obj.write_bytes(b"tampered historical object")
        self._reject(payload, "input_object_missing")


if __name__ == "__main__":
    unittest.main()
