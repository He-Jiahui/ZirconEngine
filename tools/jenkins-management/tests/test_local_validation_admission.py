"""Local requests enter validation without a separately published user grant."""
from __future__ import annotations
from concurrent.futures import ThreadPoolExecutor
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tools.dev import jenkins_validate as client
from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.source import repository_id
from tools.jenkins.state import State

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("automatic_local_admission_tests", ROOT / "tools/jenkins-management/source_preparation.py")
helper = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(helper)
RUNNER_SPEC = importlib.util.spec_from_file_location("legacy_validation_admission_runner", ROOT / "tools/jenkins-management/runner.py")
runner = importlib.util.module_from_spec(RUNNER_SPEC)
RUNNER_SPEC.loader.exec_module(runner)


class LocalValidationAdmissionTests(unittest.TestCase):
    def setUp(self):
        parent = Path("E:/cargo-targets/zircon-local/tmp/local-validation-admission-tests")
        parent.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(prefix="admit-", dir=parent)
        self.addCleanup(temporary.cleanup)
        self.repo = Path(temporary.name)
        (self.repo / "src").mkdir()
        (self.repo / "src/lib.rs").write_bytes(b"after\n")
        (self.repo / "Cargo.toml").write_bytes(b"[workspace]\n")
        self.state = State(self.repo / ".jenkins/state/coordination.sqlite3")
        self.repository = repository_id(self.repo)
        self.principal = "windows:S-1-5-21-123-456-789-1001"

    def payload(self, session="first", request="request"):
        return {"identity": {"repositoryId": self.repository, "sessionId": session,
                "requestId": request, "attemptId": request + "-1", "generation": 1,
                "stageImplementationDigest": "a" * 64},
                "ownedPaths": ["src/lib.rs"], "beforeHashes": {"src/lib.rs": "b" * 64},
                "afterHashes": {"src/lib.rs": hashlib.sha256(b"after\n").hexdigest()},
                "declaredDependencies": ["Cargo.toml"],
                "dependencyHashes": {"Cargo.toml": hashlib.sha256(b"[workspace]\n").hexdigest()},
                "inputMode": "already-applied"}

    def admit(self, value=None, runtime="runtime-one"):
        return helper.admit_ordinary_context(self.state, self.repo, value or self.payload(),
            principal=self.principal, driver_digest="a" * 64, runtime_operation_id=runtime)

    def test_two_unregistered_sessions_receive_automatic_context_and_no_publish_rights(self):
        first, second = self.admit(), self.admit(self.payload("second"))
        self.assertEqual(self.principal, first["principal"])
        self.assertEqual(["implementation", "validation"], first["actions"])
        self.assertNotEqual(first["snapshotDigest"], second["snapshotDigest"])
        for action in ("commit", "push", "deployment", "gc", "notify"):
            with self.assertRaises(JenkinsError):
                self.state.check_authorization(self.repository, "first", action, ["src/lib.rs"])
        with self.state.connect() as connection:
            row = connection.execute("SELECT evidence FROM authorizations WHERE session_id='first'").fetchone()
        self.assertEqual("automatic-local-request", json.loads(row[0])["source"])

    def test_repeated_and_concurrent_admission_is_idempotent(self):
        with ThreadPoolExecutor(max_workers=4) as readers:
            results = list(readers.map(lambda unused: self.admit(), range(8)))
        self.assertEqual(1, len({value["snapshotDigest"] for value in results}))
        self.assertEqual(1, len(self.state.list("local_request_admission")))

    def test_scope_snapshot_survives_new_top_level_and_runtime_without_expansion(self):
        first = self.admit()
        (self.repo / "new-top.rs").write_bytes(b"new")
        second = self.admit(self.payload(request="next"), runtime="runtime-two")
        self.assertEqual(first, second)
        self.assertNotIn("new-top.rs", second["paths"])
        changed = self.payload(request="new-top")
        changed["ownedPaths"] = ["new-top.rs"]
        with self.assertRaises(JenkinsError) as error:
            self.admit(changed)
        self.assertEqual("path_not_authorized", error.exception.code)
        self.assertEqual(first, self.state.check_authorization(self.repository, "first", "validation", ["src/lib.rs"]))

    def test_preflight_context_allows_final_payload_digest_to_be_recorded_once(self):
        original = self.payload()
        first = helper.admit_ordinary_context(self.state, self.repo, original, principal=self.principal,
            driver_digest="a" * 64, runtime_operation_id="runtime-one", record_request=False)
        self.assertEqual([], self.state.list("local_request_admission"))
        final = {**original, "expectedSourceDigest": "d" * 64}
        second = self.admit(final)
        self.assertEqual(first, second)
        self.assertEqual(1, len(self.state.list("local_request_admission")))

    def test_legacy_validation_only_caller_keeps_scope_and_rejects_implementation(self):
        manual = self.state.authorize_session(self.repository, "legacy-reader", "original-reader", ["src"],
            ["validation"], {"source": "user", "task": "existing validation-only request"})
        def authorization_row():
            with self.state.connect() as connection:
                return tuple(connection.execute(
                    "SELECT * FROM authorizations WHERE repository_id=? AND session_id=?",
                    (self.repository, "legacy-reader")).fetchone())
        before = authorization_row()
        config = {"driverDigest": "a" * 64, "runtimeOperationId": "runtime-one"}
        payload = {"repositoryId": self.repository, "sessionId": "legacy-reader", "requestId": "legacy-request",
            "operationId": "validation", "action": "validate", "ownedPaths": ["src/lib.rs"],
            "sourceView": "checkout", "sourceHashes": {"src/lib.rs": hashlib.sha256(b"after\n").hexdigest()},
            "dependencyHashes": {}, **config}
        with patch.object(runner, "source_helper", return_value=helper), \
                patch.object(helper, "local_request_principal", return_value=self.principal):
            observed = runner.ensure_request_context(config, self.repo, object(), payload, state=self.state)
            self.assertEqual(manual["snapshotDigest"], observed["snapshotDigest"])
            self.assertEqual(["validation"], observed["actions"])
            self.assertEqual(["src"], observed["paths"])
            implementation = {**payload, "operationId": "implementation", "action": "create-branch"}
            with self.assertRaises(JenkinsError) as error:
                runner.ensure_request_context(config, self.repo, object(), implementation, state=self.state)
        self.assertEqual("action_not_authorized", error.exception.code)
        self.assertEqual(before, authorization_row())
        self.assertEqual(1, len(self.state.list("local_request_admission")))

    def test_existing_manual_writer_scope_and_snapshot_are_not_expanded(self):
        manual = self.state.authorize_session(self.repository, "first", "original-writer", ["src"],
            ["implementation", "validation"], {"source": "user", "task": "existing narrow writer"})
        value = self.payload()
        value["declaredDependencies"], value["dependencyHashes"] = [], {}
        observed = self.admit(value)
        self.assertEqual(manual["snapshotDigest"], observed["snapshotDigest"])
        self.assertEqual(["src"], observed["paths"])
        with self.assertRaises(JenkinsError):
            self.admit(self.payload(request="outside"))
        self.assertEqual(manual["snapshotDigest"],
                         self.state.check_authorization(self.repository, "first", "validation", ["src/lib.rs"])["snapshotDigest"])

    def test_bad_paths_hashes_and_changed_request_input_still_fail(self):
        for path in ("../escape", ".git/config", ".jenkins/state/control", ".codex/config.toml"):
            value = self.payload(request="bad-path")
            value["ownedPaths"] = [path]
            with self.subTest(path=path), self.assertRaises(JenkinsError):
                self.admit(value)
        value = self.payload()
        value["afterHashes"]["src/lib.rs"] = "invalid"
        with self.assertRaises(JenkinsError):
            self.admit(value)
        self.admit()
        value = self.payload()
        value["afterHashes"]["src/lib.rs"] = "c" * 64
        with self.assertRaises(JenkinsError) as error:
            self.admit(value)
        self.assertEqual("local_request_conflict", error.exception.code)


class RegistrationContinuationTests(unittest.TestCase):
    def setUp(self):
        parent = Path("E:/cargo-targets/zircon-local/tmp/local-validation-admission-tests")
        parent.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(prefix="lineage-", dir=parent)
        self.addCleanup(temporary.cleanup)
        self.repo = Path(temporary.name)
        (self.repo / "src").mkdir()
        (self.repo / "src/lib.rs").write_bytes(b"after\n")
        self.patch = self.repo / "change.patch"
        self.patch.write_text("--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-before\n+after\n", encoding="utf-8")
        self.driver = {"digest": "a" * 64, "runtimeOperationId": "runtime-one",
                       "manifestFiles": sorted(client.ALREADY_APPLIED_DRIVER_FILES)}
        self.calls = []

    def args(self, action="submit"):
        return client.parser().parse_args([action, "--repository", str(self.repo),
            "--session-id", "original", "--request-id", "original-request", "--attempt-id", "attempt-1",
            "--patch", str(self.patch), "--before-hashes", json.dumps({"src/lib.rs": hashlib.sha256(b"before\n").hexdigest()}),
            "--already-applied", "--base-head", "b" * 40, "--after-hashes",
            json.dumps({"src/lib.rs": hashlib.sha256(b"after\n").hexdigest()}),
            "--dependency-hashes", "{}", "--coverage-template", "module_unit", "--build-root", "E:\\cargo-targets"])

    def execute(self, callback):
        return client.execute(self.args(), call=callback, driver_loader=lambda root: self.driver)

    def test_known_no_post_rejection_continues_same_request_and_preserves_old_bytes(self):
        def denied(root, driver, folder, domain, action, payload, timeout):
            raise client.EntryError("action_not_authorized", "old registration gate")
        with self.assertRaises(client.EntryError):
            self.execute(denied)
        folder = next((self.repo / ".jenkins/state/requests/operator-validation").iterdir())
        original = {name: (folder / name).read_bytes() for name in ("lifecycle.json", "intent.json")}
        self.driver.update(digest="c" * 64, runtimeOperationId="runtime-two")
        def admitted(root, driver, current, domain, action, payload, timeout):
            self.calls.append((domain, action, payload))
            return {"patchRequestRef": "d" * 64} if action == "register" else {"status": "submitted"}
        result = self.execute(admitted)
        self.assertEqual("original-request", result["identity"]["requestId"])
        self.assertNotEqual("attempt-1", result["identity"]["attemptId"])
        self.assertEqual("c" * 64, result["identity"]["stageImplementationDigest"])
        self.assertEqual(original, {name: (folder / name).read_bytes() for name in original})
        self.assertTrue((folder / "registration-continuation.json").is_file())
        self.execute(admitted)
        self.assertEqual([("patch", "register"), ("patch", "submit"), ("patch", "query")],
                         [item[:2] for item in self.calls])

    def test_public_cli_main_runs_after_all_admission_helpers_are_defined(self):
        import ast
        tree = ast.parse(Path(client.__file__).read_text(encoding="utf-8"))
        guards = [node for node in tree.body if isinstance(node, ast.If)
                  and isinstance(node.test, ast.Compare)
                  and isinstance(node.test.left, ast.Name) and node.test.left.id == "__name__"]
        self.assertEqual(1, len(guards))
        self.assertIs(guards[0], tree.body[-1])
        helpers = {node.name: node for node in tree.body if isinstance(node, ast.FunctionDef)}
        for name in ("main", "execute", "admit_local_validation", "registration_continuation"):
            self.assertLess(helpers[name].end_lineno, guards[0].lineno)

    def test_unknown_registration_only_queries_original_without_new_attempt_or_post(self):
        def unknown(root, driver, folder, domain, action, payload, timeout):
            self.calls.append((domain, action, payload))
            if action == "register":
                raise client.EntryError("protocol_delivery_unknown", "interrupted")
            return {"status": "pending"}
        with self.assertRaises(client.EntryError):
            self.execute(unknown)
        self.driver.update(digest="c" * 64, runtimeOperationId="runtime-two")
        self.execute(unknown)
        self.assertEqual([("patch", "register"), ("request", "query")], [item[:2] for item in self.calls])
        self.assertEqual("attempt-1", self.calls[-1][2]["identity"]["attemptId"])
        folder = next((self.repo / ".jenkins/state/requests/operator-validation").iterdir())
        self.assertFalse((folder / "registration-continuation.json").exists())


if __name__ == "__main__":
    unittest.main()
