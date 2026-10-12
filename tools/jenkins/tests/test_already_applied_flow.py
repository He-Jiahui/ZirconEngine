"""Coordinator-owned patch preparation connects to flow and scoped Git adapters."""
from contextlib import contextmanager
from pathlib import Path
import unittest

from tools.jenkins import patches
from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.gitops import handle as git_flow
from tools.jenkins.tests import test_already_applied as fixture_module
from tools.jenkins.workflow.handler import handle as flow


@contextmanager
def fixture(*, commit=False):
    value = fixture_module.AlreadyAppliedTests(methodName="runTest")
    value.setUp()
    try:
        if commit:
            value.identity = {**value.identity, "sessionId": "git-session", "requestId": "git-request", "attemptId": "git-attempt"}
            value.payload["identity"] = value.identity
            value.state.authorize_session(value.repo, "git-session", "user",
                ["src", "Cargo.toml", "Cargo.lock", "data.txt", "build.rs"],
                ["implementation", "validation", "commit"], {"source": "user", "task": "isolated Git integration fixture"})
        yield value
    finally:
        value.doCleanups()


def prepare(value):
    registered = patches.handle("register", value.payload, value.state, value.root)
    return patches.prepare_patch(value.state, value.root, value.control(registered["patchRequestRef"]),
                                 trusted_driver_digest="d" * 64)


def register_flow(value, prepared):
    payload = {"identity": prepared["identity"], "sealedInputRef": prepared["sealedInputRef"],
               "sourcePreparationRef": prepared["sourcePreparationRef"], "buildRoot": value.payload["buildRoot"]}
    flow("register-flow", payload, value.state, value.root)
    key = ":".join(prepared["identity"][name] for name in ("sessionId", "requestId", "attemptId"))
    return payload, key


class AlreadyAppliedFlowTests(unittest.TestCase):
    def test_public_patch_route_produces_typed_stage_without_canonical_write(self):
        with fixture() as value:
            prepared = prepare(value)
            control, key = register_flow(value, prepared)
            composed = flow("compose-flow", control, value.state, value.root)
            self.assertEqual("module_unit", composed["template"])
            self.assertIn("unit_test", composed["stages"])
            stage = flow("run-stage", {**control, "stage": "patch"}, value.state, value.root)
            receipt = stage["receipt"]
            self.assertEqual("coordinator-source-preparation", receipt["lightEvidence"]["tool"])
            self.assertEqual(digest(["source", "prepare-already-applied-patch"]), receipt["lightEvidence"]["commandDigest"])
            self.assertEqual(value.after, (value.root / "src/lib.rs").read_bytes())
            self.assertEqual([], value.state.list("execution"))
            self.assertEqual([], value.state.list("recipe_plan"))
            self.assertEqual([], value.state.get("workflow", key)["payload"]["patchOperationRefs"])

    def test_comment_classification_uses_real_git_preimage(self):
        with fixture() as value:
            value.after = value.before.replace(b"// ", b"// Comment edit: ", 1)
            (value.root / "src/lib.rs").write_bytes(value.after)
            value.payload.update(unifiedPatch=fixture_module.change("src/lib.rs", value.before, value.after),
                                 afterHashes={"src/lib.rs": fixture_module.sha(value.after)},
                                 coverage={"template": "comments_only"})
            prepared = prepare(value)
            control, _ = register_flow(value, prepared)
            composed = flow("compose-flow", control, value.state, value.root)
            self.assertEqual("comments_only", composed["template"])
            self.assertEqual(["patch", "syntax", "format", "acceptance", "commit"], composed["stages"])
            self.assertEqual([], value.state.list("recipe_plan"))

    def test_foreign_preparation_and_raw_writer_receipt_are_rejected(self):
        with fixture() as value:
            prepared = prepare(value)
            control = {"identity": prepared["identity"], "sealedInputRef": prepared["sealedInputRef"],
                       "sourcePreparationRef": "foreign-preparation", "buildRoot": value.payload["buildRoot"]}
            with self.assertRaises(JenkinsError):
                flow("register-flow", control, value.state, value.root)
            self.assertEqual([], value.state.list("workflow"))
            control, _ = register_flow(value, prepared)
            flow("compose-flow", control, value.state, value.root)
            with self.assertRaises(JenkinsError):
                flow("run-stage", {**control, "stage": "patch", "patchOperationRef": "foreign-writer"},
                     value.state, value.root)
            self.assertEqual([], value.state.list("light_attempt"))

    def test_corrupt_dependency_blocks_composition_before_stage_receipts(self):
        with fixture() as value:
            prepared = prepare(value)
            control, _ = register_flow(value, prepared)
            operation = value.require(prepared)
            obj = Path(operation["payload"]["objectRoot"]) / "inputs/objects" / fixture_module.sha(value.data)
            original = obj.read_bytes()
            try:
                obj.write_bytes(b"corrupt closure")
                with self.assertRaises(JenkinsError):
                    flow("compose-flow", control, value.state, value.root)
                self.assertEqual([], value.state.list("light_attempt"))
                self.assertEqual([], value.state.list("recipe_plan"))
            finally:
                obj.write_bytes(original)

    def test_git_adapter_commits_only_prepared_owner_bytes_and_preserves_later_edits(self):
        with fixture(commit=True) as value:
            updated_manifest = value.manifest.replace(b'0.1.0', b'0.2.0')
            (value.root / "Cargo.toml").write_bytes(updated_manifest)
            value.payload.update(
                unifiedPatch=value.payload["unifiedPatch"] + fixture_module.change("Cargo.toml", value.manifest, updated_manifest),
                beforeHashes={"src/lib.rs": fixture_module.sha(value.before), "Cargo.toml": fixture_module.sha(value.manifest)},
                afterHashes={"src/lib.rs": fixture_module.sha(value.after), "Cargo.toml": fixture_module.sha(updated_manifest)},
                declaredDependencies=["Cargo.lock", "data.txt"],
                dependencyHashes={"Cargo.lock": fixture_module.sha(b"version = 4\n"), "data.txt": fixture_module.sha(value.data)})
            prepared = prepare(value)
            _, key = register_flow(value, prepared)
            source = prepared["sealedInputRef"]
            shared = value.state.get("sealed_input", source)
            acceptance = "isolated-accepted-source"
            value.state.put("acceptance", acceptance, {"status": "accepted", "sourceDigest": source,
                "coverageDigest": prepared["identity"]["coverageDigest"]})
            record = value.state.get("workflow", key)
            value.state.put("workflow", key, {**record["payload"], "acceptance": acceptance})
            (value.root / "foreign.txt").write_bytes(b"foreign staged edit")
            value.git("add", "foreign.txt")
            later = b"pub fn later_foreign_edit() {}\n"
            (value.root / "src/lib.rs").write_bytes(later)
            result = git_flow("commit-flow", {"workflowKey": key,
                              "authorization": {"repositoryId": value.repo, "sessionId": "git-session"}},
                              value.state, value.root)
            self.assertEqual("ref_updated", result["status"])
            self.assertEqual(value.after, value.git("show", "HEAD:src/lib.rs"))
            self.assertEqual(updated_manifest, value.git("show", "HEAD:Cargo.toml"))
            self.assertEqual(later, (value.root / "src/lib.rs").read_bytes())
            self.assertIn(b"foreign.txt", value.git("diff", "--cached", "--name-only"))
            self.assertEqual(shared, value.state.get("sealed_input", source))
            reconciled = git_flow("reconcile-flow", {"workflowKey": key, "operationId": result["operation_id"]},
                                  value.state, value.root)
            self.assertEqual("complete", reconciled["status"])
            self.assertEqual(later, (value.root / "src/lib.rs").read_bytes())
            self.assertEqual(shared, value.state.get("sealed_input", source))


if __name__ == "__main__":
    unittest.main()
