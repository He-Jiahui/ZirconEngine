"""Reference-only joint protocol, native authority and author consumption tests.

Native collectors below are isolated fixtures; no compiler or live controller
is launched. Source CAS values and artifact bytes are generated fixture inputs.
"""
from __future__ import annotations

from copy import deepcopy
import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock, patch

from tools.jenkins.cli import dispatch
from tools.jenkins.contracts import JenkinsError, digest, file_digest
from tools.jenkins.resources import canonical_build_root
from tools.jenkins.source import _consumer_key, register_sealed_consumer, repository_id
from tools.jenkins.state import State
from tools.jenkins.validation import TrustedReceiptAuthority, closure_manifest
from tools.jenkins.workflow.execution import ExecutionEngine
from tools.jenkins.workflow.group_protocol import handle, resolve_joint_checkout
from tools.jenkins.workflow.groups import MAX_GROUP_SESSIONS, SessionGroups
from tools.jenkins.workflow.joint import CONSUMPTION_DOMAIN

BUILD_ROOT = "E:/cargo-targets"


class JointGroupProtocolTests(unittest.TestCase):
    def setUp(self):
        self.approved = canonical_build_root(BUILD_ROOT)
        self.temporary = tempfile.TemporaryDirectory(prefix="joint-protocol-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.state = State(self.root / "state.sqlite3")
        self.repo = repository_id(self.root)
        self.driver = digest("fixture-driver")
        self.objects = self.approved.namespace() / "inputs" / "objects"
        self.objects.mkdir(parents=True, exist_ok=True)
        self.groups = SessionGroups(self.state, strict=True, repo_root=self.root)
        self.authors = [self.author(name, "src/" + name + ".rs") for name in "abc"]
        self.current_version = None

    def author(self, name, path, *, dependencies=None, features=None, external=None, owned=None):
        entries = []
        for relative, content in {path: self.root.name + name, **(dependencies or {})}.items():
            data = content.encode()
            checksum = hashlib.sha256(data).hexdigest()
            obj = self.objects / checksum
            existed = obj.exists()
            obj.write_bytes(data)
            if not existed:
                self.addCleanup(obj.unlink, missing_ok=True)
            entries.append({"path": relative, "status": "present", "sha256": checksum, "objectDigest": checksum})
        manifest = {"schemaVersion": 1, "repositoryId": self.repo, "baseHead": "a" * 40,
                    "entries": sorted(entries, key=lambda value: value["path"]),
                    "external": external or [], "declaredDependencies": sorted(dependencies or {})}
        if external:
            manifest["preparationKind"] = "current-source"
        source = digest(manifest)
        scope = owned or [path.casefold()]
        coverage = {"sourceDigest": source, "profile": "dev", "features": features or [],
                    "requiredTests": ["test_" + name]}
        coverage["coverageDigest"] = digest(coverage)
        sealed = {"status": "sealed", "repositoryId": self.repo, "repositoryRoot": str(self.root),
                  "sourceDigest": source, "sourceManifest": manifest, "manifest": manifest,
                  "coverage": coverage, "coverageDigest": coverage["coverageDigest"],
                  "ownedPaths": scope, "objectRoot": str(self.approved.namespace()), "buildRoot": BUILD_ROOT,
                  "toolchainDigest": digest("toolchain"), "lockDigest": digest("lock"),
                  "compilerIdentity": "fixture-rustc", "wrapperDigest": digest("wrappers"),
                  "owner": "attempt-" + name, "sessionId": "session-" + name, "claims": [], "patchOperationRefs": []}
        self.state.put("sealed_input", source, sealed)
        identity = {"repositoryId": self.repo, "sessionId": "session-" + name, "requestId": "request-" + name,
                    "attemptId": "attempt-" + name, "generation": 1, "sourceInputDigest": source,
                    "coverageDigest": coverage["coverageDigest"], "stageImplementationDigest": self.driver}
        extra_scope = ["external/" + item["name"] + "/" + entry["path"]
                       for item in external or [] for entry in item["manifest"]["entries"]]
        grant = self.state.authorize_session(self.repo, identity["sessionId"], "fixture-user", [*scope, *extra_scope],
            ["validation"], {"source": "user", "task": "isolated joint fixture"})
        self.state.submit_request({**identity, "sealedInputRef": source, "ownedPaths": scope,
                                  "buildRoot": BUILD_ROOT, "authorizationSnapshot": grant["snapshotDigest"]})
        register_sealed_consumer(self.state, repository_id_value=self.repo, session_id=identity["sessionId"],
                                owner=identity["attemptId"], source_digest=source, manifest_digest=source)
        return {"identity": identity, "sealedInputRef": source}

    def prepare(self, authors=None):
        selected = self.authors if authors is None else authors
        result = handle("prepare-group", {"identity": selected[0]["identity"], "sessions": selected, "mode": "joint"},
                        self.state, self.root)
        self.group_id, self.current_version = result["groupId"], result["version"]
        return result

    def aggregator(self, prepared, *, scope=None):
        identity = {"repositoryId": self.repo, "sessionId": "union-session", "requestId": "union-request",
            "attemptId": "union-attempt", "generation": 1, "sourceInputDigest": prepared["unionSourceDigest"],
            "coverageDigest": prepared["unionCoverageDigest"], "stageImplementationDigest": self.driver}
        paths = sorted(path for item in prepared["sessions"] for path in item["paths"])
        grant = self.state.authorize_session(self.repo, identity["sessionId"], "fixture-user", scope or paths,
            ["validation"], {"source": "user", "task": "isolated union fixture"})
        self.state.submit_request({**identity, "ownedPaths": paths, "sealedInputRef": identity["sourceInputDigest"],
                                  "buildRoot": BUILD_ROOT, "jointGroupId": self.group_id,
                                  "authorizationSnapshot": grant["snapshotDigest"]})
        self.union_identity = identity
        return identity

    def call(self, action, identity=None, **fields):
        identity = identity or self.union_identity
        result = handle(action, {"identity": identity, "groupId": self.group_id,
            "expectedVersion": self.current_version, **fields}, self.state, self.root)
        self.current_version = result["version"]
        return result

    def seal(self, prepared=None):
        prepared = prepared or self.prepare()
        self.aggregator(prepared)
        result = self.call("seal-joint")
        manifest_file = self.approved.namespace() / "inputs" / (result["sealedInputRef"] + ".json")
        self.addCleanup(manifest_file.unlink, missing_ok=True)
        return result

    def accept_union(self, required_stages=None, recipe_values=None):
        """Use existing execution/publication/receipt authorities end to end."""
        self.seal()
        source, coverage = (self.union_identity[key] for key in ("sourceInputDigest", "coverageDigest"))
        preparation = digest({"fixture": self.root.name})
        driver_binding = {"digest": self.driver, "generation": "1", "runtimeOperationId": "fixture-" + self.root.name}
        recipe = {"kind": "cargo", "buildRoot": BUILD_ROOT, "preparationKey": preparation,
            "preparationRoot": str(self.approved.namespace() / "preparations" / preparation / "inputs" / "source"),
            "resourceEstimates": {"cpu": 1, "memoryBytes": 256, "diskBytes": 64},
            "phases": ["compile", "list", "test"], "requiredTestStages": ["unit_test", "integration_test"],
            "profile": "dev", "features": [], "requiredTests": ["test_a", "test_b", "test_c"],
            "toolchainDigest": digest("toolchain"), "lockDigest": digest("lock"),
            "compilerIdentity": "fixture-rustc", "resourceProfile": "default",
            "commands": {phase: ["fixture-cargo", "test", phase] for phase in ("compile", "list", "test")}}
        recipe.update(recipe_values or {})
        recipe_ref = digest(recipe)
        self.state.put("resource_policy", "default", {"cpuBudget": 2, "memoryBudget": 1024, "diskBudget": 2048})
        self.state.put("recipe_plan", recipe_ref, {"recipe": recipe, "recipeDigest": recipe_ref,
            "sourceRef": source, "coverageRef": coverage, "driver": driver_binding, "generation": "1"})
        flow_id = ":".join(self.union_identity[key] for key in ("sessionId", "requestId", "attemptId"))
        flow = {"identity": self.union_identity, "repositoryId": self.repo, "sourceDigest": source, "coverageDigest": coverage,
            "driverDigest": self.driver, "generation": 1, "buildRoot": BUILD_ROOT, "status": "ready",
            "requiredStages": ["build", "unit_test", "integration_test"],
            "stageRecipeDigests": {stage: recipe_ref for stage in ("build", "unit_test", "integration_test")},
            "stageReceipts": {}, "dagDigest": digest(["build", "unit_test"]), "requiredOutputs": [],
            "requiredOutputsByStage": {stage: ["stdout.log", "stderr.log"] for stage in ("build", "unit_test", "integration_test")},
            "stages": [{"name": name, "status": "pending"} for name in ("build", "unit_test", "integration_test", "acceptance")]}
        if required_stages is not None:
            flow["requiredStages"] = required_stages
        self.state.put("workflow", flow_id, flow)
        engine = ExecutionEngine(self.state, repo_root=self.root, build_root=BUILD_ROOT, resource_manager=Mock())
        claimed = engine.claim(source=source, recipe=recipe_ref, driver=driver_binding, consumer_id=flow_id)
        self.execution_id = eid = claimed["executionId"]
        row = self.state.get("workflow", flow_id)
        self.state.put("workflow", flow_id, {**row["payload"], "executionId": eid})
        row = self.state.get("execution", eid)
        output_base = self.root / "receipts"
        self.state.put("execution", eid, {**row["payload"], "status": "running", "phaseOutputRoot": str(output_base)})
        authority = TrustedReceiptAuthority(self.state, repo_root=self.root)
        phases, receipts = [], {}
        for index, phase in enumerate(("compile", "list", "test")):
            native, birth = "native-" + phase, "birth-" + phase
            stage = {"compile": "build", "list": "discovery", "test": "unit_test"}[phase]
            proof = {"complete": True, "processExitCode": 0, "stdoutEof": True, "stderrEof": True, "childrenGone": True}
            self.state.put("native_job", native, {"nativeJobId": native, "status": "terminal", "owner": eid,
                "executionId": eid, "birthToken": birth, "pid": 13000 + index, "completeProof": proof})
            record = {"phase": phase, "stage": stage, "nativeJobId": native, "terminalProof": proof,
                      "commandDigest": digest(recipe["commands"][phase])}
            if phase == "list":
                record["discovery"] = {"testCount": 3, "source": "cargo-test-list", "tests": ["test_a", "test_b", "test_c"]}
            else:
                output = output_base / phase
                output.mkdir(parents=True)
                for stream in ("stdout", "stderr"):
                    (output / (stream + ".log")).write_text(self.root.name + phase + stream, encoding="utf-8")
                for stage_name in (["build"] if phase == "compile" else ["unit_test", "integration_test"]):
                    attempt = f"{eid}:compile" if phase == "compile" else f"{eid}:test:{stage_name}"
                    authority.register_attempt({"executionId": attempt, "stage": stage_name, "sourceDigest": source,
                        "recipeDigest": recipe_ref, "coverageDigest": coverage, "driverDigest": self.driver,
                        "nativeJobId": native, "birthToken": birth, "phase": phase, "outputRoot": str(output),
                        "buildRoot": str(output), "requiredOutputs": ["stdout.log", "stderr.log"]})
                    authority.record_terminal(attempt, {**proof, "nativeJobId": native, "birthToken": birth}, closure_manifest(output))
                    receipts[stage_name] = attempt
            phases.append(record)
        self.state.put("execution_host", eid, {"status": "terminal", "phaseRecords": phases, "stageReceipts": receipts})
        self.assertEqual("passed", engine.observe(eid)["status"])
        row = self.state.get("workflow", flow_id)
        self.state.put("workflow", flow_id, {**row["payload"], "stageReceipts": receipts})
        accepted = authority.accept_flow({"flowId": flow_id})
        self.acceptance_ref, self.flow_id = accepted["acceptanceId"], flow_id
        return self.call("accept-joint", receiptRef=self.acceptance_ref, flowId=flow_id)

    def reject(self, code, action, identity=None, **fields):
        before = deepcopy(self.groups.record(self.group_id))
        with self.assertRaises(JenkinsError) as raised:
            self.call(action, identity, **fields)
        self.assertEqual(code, raised.exception.code)
        self.assertEqual(before, self.groups.record(self.group_id))

    def test_three_authors_draft_seal_immutable_and_no_source_objects_copied(self):
        prepared = self.prepare()
        self.assertEqual("draft", prepared["sourceStatus"])
        union = self.state.get("joint_input", prepared["unionSourceDigest"])["payload"]
        self.assertEqual(3, len(union["authors"]))
        selected_objects = {entry["objectDigest"] for entry in union["sourceManifest"]["entries"]}
        object_inventory = {checksum: file_digest(self.objects / checksum) for checksum in selected_objects}
        result = self.seal(prepared)
        self.assertEqual("sealed", result["status"])
        self.assertEqual("prepared", result["jointPreparationStatus"])
        replay = self.call("seal-joint", expectedVersion=prepared["version"])
        self.assertEqual(result, replay)
        self.assertEqual(object_inventory, {checksum: file_digest(self.objects / checksum) for checksum in selected_objects})
        self.assertEqual([], self.state.list(CONSUMPTION_DOMAIN))
        self.assertEqual([], self.state.list("acceptance"))

    def test_actual_collector_acceptance_has_separate_per_author_consumption(self):
        ready = self.accept_union()
        self.assertEqual("ready", ready["status"])
        self.assertEqual([], ready["acceptedConsumers"])
        original = [self.state.get_request(self.repo, item["identity"]["sessionId"], item["identity"]["requestId"])
                    for item in self.authors]
        refs = []
        for item in self.authors:
            result = self.call("consume-joint", item["identity"])
            consumed = result["consumption"]
            refs.append(result["receiptRef"])
            self.assertEqual("joint-union", consumed["acceptanceScope"])
            self.assertFalse(consumed["standaloneSourceAccepted"])
            self.assertEqual(item["sealedInputRef"], consumed["authorSourceDigest"])
            self.assertEqual(self.union_identity["sourceInputDigest"], consumed["unionSourceDigest"])
            self.assertEqual(3, len(consumed["unionAcceptance"]["nativeEvidence"]))
            replay = self.call("consume-joint", item["identity"], expectedVersion=ready["version"])
            self.assertEqual(result["receiptRef"], replay["receiptRef"])
        self.assertEqual(3, len(set(refs)))
        self.assertEqual(3, len(self.state.list(CONSUMPTION_DOMAIN)))
        after = [self.state.get_request(self.repo, item["identity"]["sessionId"], item["identity"]["requestId"])
                 for item in self.authors]
        self.assertEqual(original, after)
        self.assertEqual({"session-a", "session-b", "session-c"}, set(self.groups.observe(self.group_id).accepted_consumers))

    def test_cancel_one_author_keeps_union_and_other_consumers(self):
        self.accept_union()
        frozen = deepcopy(self.state.get("joint_input", self.union_identity["sourceInputDigest"]))
        self.call("cancel-group", self.authors[0]["identity"])
        self.reject("group_consumer_cancelled", "consume-joint", self.authors[0]["identity"])
        self.call("consume-joint", self.authors[1]["identity"])
        self.call("consume-joint", self.authors[2]["identity"])
        self.assertEqual(frozen, self.state.get("joint_input", self.union_identity["sourceInputDigest"]))
        self.assertEqual(["session-b", "session-c"], list(self.groups.observe(self.group_id).accepted_consumers))

    def test_cancelled_registered_request_and_late_generation_cannot_consume(self):
        self.accept_union()
        identity = self.authors[0]["identity"]
        self.state.transition_request(self.repo, identity["sessionId"], identity["requestId"], 1, "pending", "cancelled")
        self.reject("request_cancelled", "consume-joint", identity)
        self.reject("group_author_mismatch", "consume-joint", {**self.authors[1]["identity"], "generation": 2})
        self.assertEqual([], self.state.list(CONSUMPTION_DOMAIN))

    def test_collectors_and_original_native_eof_are_rechecked(self):
        self.accept_union()
        row = self.state.get("native_job", "native-list")
        proof = {**row["payload"]["completeProof"], "stderrEof": False}
        self.state.put("native_job", row["key"], {**row["payload"], "completeProof": proof})
        self.reject("joint_native_untrusted", "consume-joint", self.authors[0]["identity"])
        self.assertEqual([], self.state.list(CONSUMPTION_DOMAIN))

    def test_changed_managed_output_is_rejected_before_author_consumption(self):
        self.accept_union()
        (self.root / "receipts" / "compile" / "stdout.log").write_text("foreign output", encoding="utf-8")
        self.reject("output_closure_changed", "consume-joint", self.authors[0]["identity"])

    def test_changed_default_features_cannot_accept_the_original_author_coverage(self):
        with self.assertRaises(JenkinsError) as raised:
            self.accept_union(recipe_values={"commands": {phase: ["fixture-cargo", "test", "--no-default-features", phase]
                                                          for phase in ("compile", "list", "test")}})
        self.assertEqual("joint_recipe_mismatch", raised.exception.code)
        self.assertEqual("pending-joint-validation", self.groups.observe(self.group_id).status)
        self.assertEqual([], self.state.list(CONSUMPTION_DOMAIN))

    def test_union_compiler_identity_cannot_replace_compatible_author_compilers(self):
        with self.assertRaises(JenkinsError) as raised:
            self.accept_union(recipe_values={"compilerIdentity": "foreign-compiler"})
        self.assertEqual("joint_recipe_mismatch", raised.exception.code)
        self.assertEqual([], self.state.list(CONSUMPTION_DOMAIN))

    def test_original_native_command_must_match_the_union_recipe(self):
        self.accept_union()
        row = self.state.get("execution", self.execution_id)
        phases = deepcopy(row["payload"]["phaseRecords"])
        phases[1]["commandDigest"] = digest(["foreign-command"])
        self.state.put("execution", row["key"], {**row["payload"], "phaseRecords": phases})
        self.reject("joint_native_untrusted", "consume-joint", self.authors[0]["identity"])

    def test_frozen_union_coverage_cannot_be_mutated_under_an_existing_digest(self):
        self.accept_union()
        row = self.state.get("joint_input", self.union_identity["sourceInputDigest"])
        coverage = {**row["payload"]["coverage"], "requiredTests": []}
        self.state.put("joint_input", row["key"], {**row["payload"], "coverage": coverage})
        self.reject("joint_coverage_corrupt", "consume-joint", self.authors[0]["identity"])

    def test_published_union_object_is_revalidated_by_the_receipt_authority(self):
        self.accept_union()
        item = self.state.get("artifact_bundle", self.execution_id)["payload"]["files"][0]
        obj = self.approved.namespace() / "artifacts" / "artifacts" / "objects" / item["digest"]
        original = obj.read_bytes()
        self.addCleanup(obj.write_bytes, original)
        obj.write_bytes(b"foreign artifact")
        self.reject("workflow_artifact_changed", "consume-joint", self.authors[0]["identity"])

    def test_invalid_partial_joint_acceptance_does_not_change_group(self):
        self.seal()
        self.state.put("acceptance", "fake", {"status": "accepted", "sourceDigest": self.union_identity["sourceInputDigest"]})
        self.reject("fabricated_acceptance", "accept-joint", receiptRef="fake", flowId="fake-flow")
        self.reject("joint_acceptance_pending", "consume-joint", self.authors[0]["identity"])

    def test_prefix_case_conflict_and_dependency_disagreement_rejected(self):
        first = self.author("directory", "Src/child.rs", owned=["src"])
        second = self.author("child", "src/child.rs")
        for selected, code in [([first, second], "group_path_conflict"),
            ([self.author("dep-a", "src/d.rs", dependencies={"Cargo.toml": "first"}),
              self.author("dep-b", "src/e.rs", dependencies={"Cargo.toml": "second"})], "joint_dependency_conflict")]:
            with self.subTest(code=code), self.assertRaises(JenkinsError) as raised:
                self.prepare(selected)
            self.assertEqual(code, raised.exception.code)

    def test_duplicate_authors_capacity_and_features_are_bounded(self):
        for selected, code in [([self.authors[0], self.authors[0]], "group_session_duplicate"),
                               ([self.authors[0]], "group_size_invalid"),
                               ([self.authors[0]] * (MAX_GROUP_SESSIONS + 1), "group_input_invalid"),
                               ([self.authors[0], self.author("feature", "src/f.rs", features=["new"])],
                                "group_identity_conflict")]:
            with self.subTest(code=code), self.assertRaises(JenkinsError) as raised:
                self.prepare(selected)
            self.assertEqual(code, raised.exception.code)

    def test_wrong_original_request_or_sealed_consumer_is_rejected(self):
        wrong = deepcopy(self.authors)
        wrong[0]["identity"]["requestId"] = "unregistered"
        with self.assertRaises(JenkinsError) as raised:
            self.prepare(wrong)
        self.assertEqual("request_not_registered", raised.exception.code)
        identity = self.authors[1]["identity"]
        key = _consumer_key(self.repo, identity["sessionId"], identity["attemptId"], identity["sourceInputDigest"])
        row = self.state.get("sealed_consumer", key)
        self.state.put("sealed_consumer", key, {**row["payload"], "status": "released"})
        with self.assertRaises(JenkinsError) as raised:
            self.prepare()
        self.assertEqual("sealed_consumer_missing", raised.exception.code)

    def test_scope_grant_cannot_be_replaced_by_caller_paths(self):
        prepared = self.prepare()
        self.aggregator(prepared, scope=["src/a.rs"])
        self.reject("path_not_authorized", "seal-joint", ownedPaths=["src/a.rs"])

    def test_metadata_failure_retains_exact_seal_and_replay_finishes_preparation(self):
        prepared = self.prepare()
        self.aggregator(prepared)
        with patch("tools.jenkins.patches._ensure_patch_metadata", side_effect=JenkinsError("fixture-metadata-error", "failure")):
            with self.assertRaises(JenkinsError):
                self.call("seal-joint")
        group = self.groups.record(self.group_id)
        self.assertEqual("metadata-pending", group["payload"]["jointPreparationStatus"])
        self.assertEqual("fixture-metadata-error", group["payload"]["jointPreparationError"]["reasonCode"])
        self.assertIsNotNone(self.state.get("sealed_input", prepared["unionSourceDigest"]))
        self.current_version = group["version"]
        result = self.call("seal-joint")
        self.assertEqual("prepared", result["jointPreparationStatus"])
        manifest_file = self.approved.namespace() / "inputs" / (prepared["unionSourceDigest"] + ".json")
        self.addCleanup(manifest_file.unlink, missing_ok=True)

    def test_public_cas_rejects_stale_mutation_but_exact_cancel_replay_is_idempotent(self):
        self.prepare()
        version = self.current_version
        self.call("cancel-group", self.authors[0]["identity"])
        self.reject("group_version_conflict", "cancel-group", self.authors[1]["identity"], expectedVersion=version)
        replay = self.call("cancel-group", self.authors[0]["identity"], expectedVersion=version)
        self.assertEqual(["session-a"], replay["cancelledConsumers"])

    def test_new_operations_use_runtime_fence_historical_query_cancel_do_not(self):
        prepared = self.prepare()
        deployment = self.root / ".jenkins" / "state" / "deployment"
        deployment.mkdir(parents=True)
        (deployment / "driver.json").write_text("{}", encoding="utf-8")
        binding = {"digest": digest("new-driver"), "runtimeOperationId": "new-runtime"}
        with patch("tools.jenkins.deployment.driver.active_driver_binding", return_value=binding):
            with self.assertRaises(JenkinsError) as raised:
                self.prepare()
            self.assertEqual("driver_context_mismatch", raised.exception.code)
            observed = handle("observe-group", {"groupId": self.group_id}, self.state, self.root)
            self.assertEqual(prepared["groupId"], observed["groupId"])
            self.call("cancel-group", self.authors[0]["identity"])

    def test_authoritative_but_partial_workflow_cannot_accept_union(self):
        with self.assertRaises(JenkinsError) as raised:
            self.accept_union(required_stages=["build", "integration_test"])
        self.assertEqual("joint_coverage_incomplete", raised.exception.code)
        self.assertEqual("pending-joint-validation", self.groups.observe(self.group_id).status)
        self.assertEqual([], self.state.list(CONSUMPTION_DOMAIN))

    def test_registered_collector_replacement_is_rejected(self):
        self.accept_union()
        row = self.state.get("validation_receipt", self.execution_id + ":compile")
        self.state.put("validation_receipt", row["key"], {**row["payload"], "driverDigest": digest("foreign")})
        self.reject("joint_collector_untrusted", "consume-joint", self.authors[0]["identity"])

    def test_author_revocation_is_rechecked_at_seal_and_consume(self):
        self.accept_union()
        self.state.revoke_session(self.repo, self.authors[0]["identity"]["sessionId"])
        self.reject("action_not_authorized", "consume-joint", self.authors[0]["identity"])
        self.call("consume-joint", self.authors[1]["identity"])

    def test_union_checkout_resolver_binds_aggregator_and_all_authors(self):
        self.seal()
        request = self.state.get_request(self.repo, self.union_identity["sessionId"], self.union_identity["requestId"])
        intent = request["payload"]
        with self.state.transaction() as connection:
            root, proof = resolve_joint_checkout(self.state, connection, self.root, intent, intent,
                self.union_identity, self.union_identity["sourceInputDigest"])
        self.assertEqual(self.root, root)
        self.assertEqual(3, len(proof["originalAuthors"]))
        self.assertEqual(self.group_id, proof["jointGroupId"])
        with self.state.transaction() as connection, self.assertRaises(JenkinsError) as raised:
            resolve_joint_checkout(self.state, connection, self.root, {**intent, "jointGroupId": digest("foreign")},
                intent, self.union_identity, self.union_identity["sourceInputDigest"])
        self.assertEqual("joint_checkout_mismatch", raised.exception.code)

    def test_public_union_consumer_intake_and_flow_use_original_authority(self):
        self.seal()
        source = self.union_identity["sourceInputDigest"]
        intake = {"identity": self.union_identity, "owner": self.union_identity["attemptId"],
                  "sourceDigest": source, "manifestDigest": source, "jointGroupId": self.group_id,
                  "driverDigest": self.driver, "runtimeOperationId": "fixture-runtime"}
        runtime = {"digest": self.driver, "runtimeOperationId": "fixture-runtime"}
        with patch("tools.jenkins.source.consumer_intake.active_driver_binding", return_value=runtime):
            registered = dispatch("candidate", "register-consumer", intake, self.state, self.root)
        self.assertEqual("registered", registered["status"])
        key = _consumer_key(self.repo, self.union_identity["sessionId"], self.union_identity["attemptId"], source)
        proof = self.state.get("sealed_consumer_intake", key)["payload"]["sourceCheckout"]
        self.assertEqual(self.group_id, proof["jointGroupId"])
        self.assertEqual(3, len(proof["originalAuthors"]))
        with patch("tools.jenkins.workflow.handler._trusted_driver", return_value=self.driver):
            flow = dispatch("flow", "register-flow", {"identity": self.union_identity, "sealedInputRef": source,
                            "jointGroupId": self.group_id}, self.state, self.root)
        self.assertEqual("pending", flow["status"])
        flow_id = ":".join(self.union_identity[name] for name in ("sessionId", "requestId", "attemptId"))
        contract = self.state.get("workflow", flow_id)["payload"]
        self.assertEqual(source, contract["sourceDigest"])
        self.assertEqual(3, len(contract["sourceManifest"]["authorInputs"]))
        self.assertEqual("joint-input", contract["sourceManifest"]["preparationKind"])
        foreign = {**intake, "jointGroupId": digest("foreign")}
        with patch("tools.jenkins.source.consumer_intake.active_driver_binding", return_value=runtime), self.assertRaises(JenkinsError) as raised:
            dispatch("candidate", "register-consumer", foreign, self.state, self.root)
        self.assertEqual("joint_checkout_mismatch", raised.exception.code)

    def test_external_sources_with_same_name_must_share_full_identity(self):
        dependency_a = self.author("dependency-a", "lib.rs")
        dependency_b = self.author("dependency-b", "lib.rs")
        external = []
        for author in (dependency_a, dependency_b):
            manifest = self.state.get("sealed_input", author["sealedInputRef"])["payload"]["sourceManifest"]
            external.append({"kind": "sealed-external-source", "name": "zr_vm",
                "repositoryRoot": str(self.root / "fixture-sibling"), "manifest": manifest, "sourceDigest": digest(manifest)})
        first = self.author("external-a", "src/ext-a.rs", external=[external[0]])
        second = self.author("external-b", "src/ext-b.rs", external=[external[1]])
        with self.assertRaises(JenkinsError) as raised:
            self.prepare([first, second])
        self.assertEqual("joint_external_conflict", raised.exception.code)
        shared = self.author("external-c", "src/ext-c.rs", external=[external[0]])
        result = self.prepare([first, shared])
        joint = self.state.get("joint_input", result["unionSourceDigest"])["payload"]
        self.assertEqual([external[0]], joint["sourceManifest"]["external"])

    def test_legacy_inactive_group_is_observed_without_rewrite(self):
        old = digest("old-fixture")
        row = self.state.put("workflow_session_group", old, {"mode": "joint", "status": "cancelled",
            "sessions": [{"sessionId": "legacy", "generation": 1}], "jointManifestDigest": digest("old-source")})
        observed = handle("observe-group", {"groupId": old}, self.state, self.root)
        self.assertEqual("cancelled", observed["status"])
        self.assertEqual(row, self.state.get("workflow_session_group", old))


if __name__ == "__main__":
    unittest.main()
