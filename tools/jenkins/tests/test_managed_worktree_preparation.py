"""Small source-preparation and lifetime fixtures; no compiler or live Jenkins."""
from __future__ import annotations

from contextlib import ExitStack
import copy
import hashlib
import importlib.util
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from tools.jenkins import managed_worktree as support, managed_worktree_lifetime as lifetime, product_inputs
from tools.jenkins.contracts import JenkinsError, digest, file_digest
from tools.jenkins.resources import canonical_build_root
from tools.jenkins.source import repository_id
from tools.jenkins.state import State
from tools.jenkins.validation.receipts import TrustedReceiptAuthority, closure_manifest

CONTROL = ".codex/compiler/audit.py"
HELPER_PATH = Path(__file__).resolve().parents[2] / "jenkins-management/source_preparation.py"
spec = importlib.util.spec_from_file_location("managed_preparation_fixed_helper", HELPER_PATH)
helper = importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)


class ManagedWorktreePreparationTests(unittest.TestCase):
    def setUp(self):
        area = canonical_build_root(r"E:\cargo-targets").namespace() / "tmp/managed-worktree-tests"
        area.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(prefix="mwt-", dir=area)
        self.addCleanup(temporary.cleanup)
        self.area = Path(temporary.name)
        self.repo, self.cas = self.area / "r", self.area / "cas"
        self.target, self.external = self.repo / ".jenkins/worktrees/w-t", self.area / "zr_vm"
        self.target.mkdir(parents=True)
        self.state = State(self.area / "state.sqlite3")
        self.repo_id, self.base, self.head = repository_id(self.repo), "a" * 40, "b" * 40
        self.desired = {"owned.txt": b"MVP edit\n", "foreign.bin": b"\0\xffforeign",
                        "Cargo.toml": b"[workspace]\n", "rename/new.txt": b"rename\n",
                        "untracked.txt": b"candidate one\n", CONTROL: b"compiler audit\n"}
        self.desired.update({name: b"template\n" for name in product_inputs.PROJECT_TEMPLATE_INPUTS})
        self.before = {"owned.txt": b"base\n", "foreign.bin": b"old\n", "Cargo.toml": b"base cargo\n",
                       "rename/old.txt": b"rename\n", "deleted.txt": b"delete\n"}
        self.write(self.repo, self.desired)
        self.write(self.target, self.before)
        self.write(self.external, {"Cargo.toml": b"[workspace]\n", "src/lib.rs": b"pub fn vm() {}\n"})
        self.links = {"vendor/v" + str(i): {"indexCommit": str(i + 1) * 40, "head": str(i + 1) * 40}
                      for i in range(7)}
        for name in self.links:
            self.write(self.external / name, {"Cargo.toml": b"[package]\n", "src/lib.rs": name.encode()})
        self.tracked = set(self.before)
        self.writer = self.state.authorize_session(self.repo_id, "w", "user", ["owned.txt"],
            ["implementation", "validation"], {"source": "user", "task": "fixture narrow writer"})
        self.record = {"repositoryId": self.repo_id, "sessionId": "w", "status": "ready",
            "path": str(self.target), "baseHead": self.base, "ownedPaths": ["owned.txt"],
            "driverDigest": "d" * 64, "runtimeOperationId": "start-test",
            "writerPrincipal": "user", "writerAuthorizationSnapshot": self.writer["snapshotDigest"]}
        self.state.put("management_worktree", "w-t", self.record)
        self.creator = self.complete(self.payload("create-worktree", "create", "w"),
                                     {"worktreeId": "w-t", "source": self.source(self.target)})
        self.management = SimpleNamespace(state=self.state, repo=self.repo, repo_id=self.repo_id,
            config={"driverDigest": "d" * 64, "runtimeOperationId": "start-test"}, owned=["owned.txt"],
            wt=lambda reference, **kw: (self.record, self.target), git=self.git, source=self.source,
            text=self.text, build_root=lambda: r"E:\cargo-targets", outputs=lambda: self.cas / "management",
            artifact_manifest=self.artifacts)
        stack = ExitStack()
        self.addCleanup(stack.close)
        stack.enter_context(patch.object(product_inputs, "COMPILER_CONTROL_PATHS", frozenset({CONTROL})))
        stack.enter_context(patch.object(support, "COMPILER_CONTROL_PATHS", frozenset({CONTROL})))
        stack.enter_context(patch.object(support, "canonical_build_root",
            return_value=SimpleNamespace(namespace=lambda: self.cas)))
        base_tree = lambda root, base: {name.casefold(): (name, "e" * 40) for name in self.before} if root == self.repo else {}
        stack.enter_context(patch.object(support, "_git_base_tree", side_effect=base_tree))
        stack.enter_context(patch("tools.jenkins.source._git_base_tree", side_effect=base_tree))

    @staticmethod
    def write(root, values):
        for name, data in values.items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)

    def payload(self, action, request, session):
        return {"action": action, "repositoryId": self.repo_id, "sessionId": session,
                "requestId": request, "operationId": request, "worktreeId": "w-t",
                "driverDigest": "d" * 64, "runtimeOperationId": "start-test", "ownedPaths": ["owned.txt"]}

    def use(self, payload):
        self.management.payload = payload
        self.management.session = payload["sessionId"]
        self.management.key = digest({name: payload[name] for name in
                                      ("repositoryId", "sessionId", "requestId", "operationId")})
        self.management.input_digest = digest(payload)

    def complete(self, original, result):
        key = digest({name: original[name] for name in
                      ("repositoryId", "sessionId", "requestId", "operationId")})
        envelope = {name: original[name] for name in
                    ("sessionId", "requestId", "operationId", "driverDigest", "runtimeOperationId")}
        self.state.put("management_operation", key, {"phase": "complete", "action": original["action"],
            "payload": copy.deepcopy(original), "inputDigest": digest(original), **envelope,
            "result": {**result, **envelope, "status": "complete", "operationKey": key}})
        return key

    def source(self, root, *, payload=None):
        if root == self.external:
            files = {"Cargo.toml": file_digest(root / "Cargo.toml"), "src/lib.rs": file_digest(root / "src/lib.rs")}
            for name, info in self.links.items():
                files[name] = {"gitlinkCommit": info["indexCommit"], "source": self.source(root / name)}
            head = "c" * 40
        else:
            files = {path.relative_to(root).as_posix(): file_digest(path)
                     for path in root.rglob("*") if path.is_file() and ".jenkins" not in path.relative_to(root).parts}
            if root == self.target:
                files.update({name: None for name in self.tracked if name not in files})
            head = self.base if root == self.target else self.head if root == self.repo else self.links[root.relative_to(self.external).as_posix()]["head"]
        value = {"head": head, "files": dict(sorted(files.items())), "diffSha256": digest(files)}
        return {**value, "sourceDigest": digest(value)}

    def text(self, root, *args):
        if args[-1] == "--show-toplevel":
            return str(root)
        return self.source(root)["head"]

    def git(self, root, *args, **kwargs):
        if args[:3] == ("add", "--intent-to-add", "--force"):
            self.tracked.update(args[4:])
            return SimpleNamespace(stdout=b"")
        if root == self.external and "--stage" in args:
            raw = b"".join(("160000 " + info["indexCommit"] + " 0\t" + name).encode() + b"\0"
                           for name, info in self.links.items())
        else:
            names = self.desired if root == self.repo else self.tracked if root == self.target else self.source(root)["files"]
            raw = b"".join(name.encode() + b"\0" for name in sorted(names))
        return SimpleNamespace(stdout=raw)

    def artifacts(self, root, paths):
        path = self.cas / "management/reports" / self.management.key / "artifacts.json"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text('{"fixture":true}', encoding="utf-8")
        return {"path": str(path)}

    def inventory(self, request, target_source):
        self.use({**self.payload("inspect-worktree", request, "w"), "targetSourceOperationKey": target_source,
                  "externalRoots": [str(self.external)]})
        result = support.inspect_worktree(self.management, preparation_helper=helper)
        key = self.complete(self.management.payload, result)
        path = Path(result["preparationInputRef"]["path"])
        import json
        self.inputs = json.loads(path.read_bytes())["preparationInput"]
        self.reader = self.state.authorize_session(self.repo_id, "r", "user",
            sorted([*self.inputs["currentHashes"],
                    *["external/zr_vm/" + name for name in self.inputs["externalInputs"][0]["currentHashes"]]]),
            ["validation"], {"source": "user", "task": "fixture complete inspected reader"})
        return key, result["preparationInputRef"]

    def prepare(self, request, target_source):
        inspection, reference = self.inventory("inspect-" + request, target_source)
        self.use({**self.payload("prepare-worktree", request, "r"),
                  "inspectionOperationKey": inspection, "preparationInputRef": reference})
        with lifetime.writer_hold(self.management):
            result = support.prepare_worktree(self.management, preparation_helper=helper)
        key = self.complete(self.management.payload, result)
        return key, result

    def terminal_reader(self, preparation_key):
        identity = {"repositoryId": self.repo_id, "sessionId": "r", "requestId": "flow-one",
                    "attemptId": "attempt-one", "generation": 1, "stageImplementationDigest": "d" * 64}
        intent = {"identity": identity, "worktreeId": "w-t", "sourceRoot": str(self.target),
                  "worktreePreparationRef": preparation_key}
        reference = digest(intent)
        self.state.put("current_source_intent", reference, intent)
        self.state.submit_request({**identity, "currentSourceRef": reference})
        sealed, eid, flow_key = "9" * 64, "execution-one", "flow-one"
        self.state.put("current_source_preparation", reference,
                       {"status": "prepared", "result": {"sealedInputRef": sealed}})
        commands = {"compile": [r"E:\trusted\cargo.exe", "build"]}
        recipe = {"commands": commands, "requiredPhases": ["compile"]}
        recipe_ref, phases = digest(recipe), []
        output = self.area / "terminal-output"
        output.mkdir()
        authority = TrustedReceiptAuthority(self.state, repo_root=self.repo)
        manifest = {"executionId": eid, "stage": "build", "phase": "compile",
            "sourceDigest": sealed, "recipeDigest": recipe_ref, "coverageDigest": "e" * 64,
            "driverDigest": identity["stageImplementationDigest"], "nativeJobId": "compile",
            "birthToken": str(134360700000000000 + 102), "outputRoot": str(output),
            "requiredOutputs": ["compile.log"]}
        authority.register_attempt(manifest)
        for job_id, role, pid, command in [("guard", "execution_host_guard", 101, ["guard"]),
                                           ("compile", "phase", 102, commands["compile"])]:
            identity_native = {"pid": pid, "creationTime": str(134360700000000000 + pid),
                "executable": command[0], "commandDigest": hashlib.sha256("\0".join(command).encode()).hexdigest()}
            proof = {"complete": True, "childrenGone": True, "activeProcesses": 0,
                     "stdoutEof": True, "stderrEof": True, "processExitCode": 0, "identity": identity_native}
            self.state.put("native_job", job_id, {"nativeJobId": job_id, "owner": eid, "executionId": eid, "role": role,
                "guardNativeJobId": "guard", "status": "terminal", "pid": pid,
                "birthToken": identity_native["creationTime"], "executable": command[0],
                "completeProof": proof, "terminalProof": proof})
            if role == "phase":
                phases.append({"phase": "compile", "nativeJobId": job_id, "terminalProof": proof,
                               "commandDigest": digest(command)})
        (output / "compile.log").write_text("fixture compile terminal\n", encoding="utf-8")
        native = self.state.get("native_job", "compile")["payload"]
        receipt = authority.record_terminal(eid,
            {**native["completeProof"], "nativeJobId": "compile", "birthToken": native["birthToken"]},
            closure_manifest(output))
        phases[0].update(stageReceipts={"build": eid}, receipts={"build": receipt})
        self.state.put("recipe_plan", recipe_ref, {"recipe": recipe, "sourceRef": sealed})
        self.state.put("execution", eid, {"status": "passed", "sourceDigest": sealed, "phaseRecords": phases,
            "recipeRef": recipe_ref, "executionKeyDigest": "8" * 64, "consumers": [flow_key], "refs": 1})
        self.state.put("execution_launch", eid, {"status": "exited", "sourceDigest": sealed,
            "recipeRef": recipe_ref, "guardNativeJobId": "guard"})
        self.state.put("execution_host", eid, {"status": "terminal", "phaseRecords": phases})
        self.state.put("execution_consumer", eid + ":" + flow_key,
                       {"executionId": eid, "consumerId": flow_key, "status": "active"})
        self.state.put("execution_binding", digest({"consumer": flow_key, "executionKey": "8" * 64}),
                       {"executionId": eid, "consumerId": flow_key})
        acceptance = authority.accept(eid, {**manifest, "sourceInputDigest": reference,
                                            "generation": identity["generation"]})
        self.state.put("workflow", flow_key, {"identity": identity, "sourceDigest": sealed,
            "status": "accepted", "executionId": eid, "recipeRef": recipe_ref, "acceptance": acceptance})
        self.state.transition_request(self.repo_id, "r", "flow-one", 1, "pending", "accepted",
                                      {"receiptRef": acceptance["acceptanceId"]})
        return reference

    def test_one_worktree_full_gitlinks_seal_terminal_retire_then_remove_prior_added_path(self):
        canonical = self.source(self.repo)
        key, result = self.prepare("prepare-one", self.creator)
        self.assertEqual(canonical, self.source(self.repo))
        self.assertEqual(b"\0\xffforeign", (self.target / "foreign.bin").read_bytes())
        self.assertFalse((self.target / "deleted.txt").exists())
        self.assertFalse((self.target / "rename/old.txt").exists())
        prepared = support.preparation(self.state, key, "w-t")["payload"]
        external = prepared["sourceManifest"]["external"][0]
        flattened = {entry["path"].casefold(): entry["sha256"] for entry in external["manifest"]["entries"]}
        self.assertTrue(all(name + "/src/lib.rs" in flattened for name in self.links))
        (self.external / "vendor/v6/src/lib.rs").write_bytes(b"later foreign edit")
        self.assertEqual(external["manifest"]["entries"], support.prepared_external(
            self.state, {"worktreePreparationRef": key, "worktreeId": "w-t"})[0]["manifest"]["entries"])
        original = {**self.payload("apply-patch", "author-one", "w"), "expectedHead": self.base,
                    "beforeHashes": {"owned.txt": self.inputs["currentHashes"]["owned.txt"]}}
        self.use(original)
        with lifetime.writer_hold(self.management):
            (self.target / "owned.txt").write_bytes(b"actual owned author patch\n")
        after = file_digest(self.target / "owned.txt")
        patch_key = self.complete(original, {"worktreeId": "w-t", "afterHashes": {"owned.txt": after},
                                            "source": self.source(self.target)})
        intent = {"identity": {"repositoryId": self.repo_id, "sessionId": "r",
                               "stageImplementationDigest": "d" * 64},
                  "runtimeOperationId": "start-test", "baseHead": self.base, "worktreeId": "w-t",
                  "worktreePreparationRef": key, "currentHashes": {**self.inputs["currentHashes"], "owned.txt": after},
                  "compilerControlInputs": self.inputs["compilerControlInputs"],
                  "externalInputs": self.inputs["externalInputs"], "ownedPaths": ["owned.txt"],
                  "patchOperationKeys": [patch_key]}
        support.reader_binding(self.state, self.repo, intent, self.reader)
        reference = self.terminal_reader(key)
        self.use(self.payload("prepare-worktree", "blocked", "r"))
        with self.assertRaises(JenkinsError):
            with lifetime.writer_hold(self.management):
                self.fail("Active reader must freeze the target")
        self.use({**self.payload("retire-worktree-reader", "retire", "w"), "currentSourceRef": reference})
        native = self.state.get("native_job", "compile")
        broken = copy.deepcopy(native["payload"])
        broken["completeProof"]["stdoutEof"] = False
        self.state.put("native_job", "compile", broken, expected_version=native["version"])
        with self.assertRaises(JenkinsError):
            lifetime.retire_reader(self.management)
        self.assertFalse(lifetime.reader_retired(self.state, reference))
        current = self.state.get("native_job", "compile")
        self.state.put("native_job", "compile", native["payload"], expected_version=current["version"])
        self.assertTrue(lifetime.retire_reader(self.management)["readerRetired"])
        self.assertTrue(lifetime.reader_retired(self.state, reference))
        self.assertEqual([], lifetime.active_readers(self.state, "w-t", self.target))
        self.assertEqual("released", self.state.get("execution_consumer", "execution-one:flow-one")["payload"]["status"])
        execution = self.state.get("execution", "execution-one")["payload"]
        self.assertEqual(([], 0), (execution["consumers"], execution["refs"]))
        del self.desired["untracked.txt"]
        (self.repo / "untracked.txt").unlink()
        second_key, second = self.prepare("prepare-two", patch_key)
        self.assertIsNone(second["afterHashes"]["untracked.txt"])
        self.assertFalse((self.target / "untracked.txt").exists())
        self.assertEqual(self.target, Path(support.preparation(self.state, second_key, "w-t")["payload"]["path"]))
        self.assertFalse(result["canonicalSourceModified"])

    def test_target_conflict_and_incomplete_native_proof_keep_target_or_reader_blocked(self):
        inspection, reference = self.inventory("inspect-conflict", self.creator)
        (self.target / "owned.txt").write_bytes(b"foreign target")
        before = self.source(self.target)
        self.use({**self.payload("prepare-worktree", "conflict", "r"),
                  "inspectionOperationKey": inspection, "preparationInputRef": reference})
        with self.assertRaises(JenkinsError):
            with lifetime.writer_hold(self.management):
                support.prepare_worktree(self.management, preparation_helper=helper)
        self.assertEqual(before, self.source(self.target))
        self.assertEqual([], self.state.list("managed_worktree_preparation"))
        self.use(self.payload("prepare-worktree", "partial", "r"))
        with self.assertRaisesRegex(RuntimeError, "partial"):
            with lifetime.writer_hold(self.management):
                self.management._worktree_mutation_started = True
                raise RuntimeError("partial")
        with self.assertRaises(JenkinsError):
            lifetime.assert_no_writer(self.state, "w-t")


if __name__ == "__main__":
    unittest.main()
