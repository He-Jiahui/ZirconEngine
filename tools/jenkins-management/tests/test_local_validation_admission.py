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

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.source import repository_id
from tools.jenkins.state import State

ROOT = Path(__file__).resolve().parents[3]
CLIENT_SPEC = importlib.util.spec_from_file_location("local_admission_validation_client", ROOT / "tools/dev/jenkins_validate.py")
client = importlib.util.module_from_spec(CLIENT_SPEC)
CLIENT_SPEC.loader.exec_module(client)
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

    def node_sdk_payload(self, action="prepare-node"):
        value = self.payload(request=action)
        value.update(action=action, nodeRecipe={"schemaVersion": 1,
                     "id": "penpot-search-native-fill-tests-r1"})
        for name in helper._NODE_READONLY_SDK_DEPENDENCIES:
            path = self.repo / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"read-only SDK fixture\n")
            value["dependencyHashes"][name] = hashlib.sha256(path.read_bytes()).hexdigest()
        return value

    def test_typed_node_sdk_dependencies_are_recorded_without_dev_authority(self):
        for action in ("prepare-node", "validate-node"):
            with self.subTest(action=action):
                value = self.node_sdk_payload(action)
                before = json.dumps(value, sort_keys=True)
                context = self.admit(value)
                self.assertEqual(before, json.dumps(value, sort_keys=True))
                self.assertNotIn("dev", context["paths"])
                admissions = self.state.list("local_request_admission")
                admission = next(row["payload"] for row in admissions
                                 if row["payload"]["inputDigest"] == digest(value))
                self.assertEqual({name: value["dependencyHashes"][name]
                                  for name in helper._NODE_READONLY_SDK_DEPENDENCIES},
                                 admission["readOnlyDependencyHashes"])
                for name in helper._NODE_READONLY_SDK_DEPENDENCIES:
                    self.assertNotIn(name, admission["requestedPaths"])
                    for permission in ("implementation", "validation"):
                        with self.assertRaises(JenkinsError):
                            self.state.check_authorization(self.repository, "first", permission, [name])

    def test_sdk_readonly_role_cannot_be_used_as_owned_or_authored_scope(self):
        name = sorted(helper._NODE_READONLY_SDK_DEPENDENCIES)[0]
        for field in ("ownedPaths", "declaredDependencies", "sourceHashes", "beforeHashes",
                      "afterHashes", "currentHashes"):
            with self.subTest(field=field):
                value = self.node_sdk_payload()
                value[field] = [name] if field in ("ownedPaths", "declaredDependencies") else {
                    name: value["dependencyHashes"][name]}
                with self.assertRaises(JenkinsError) as error:
                    self.admit(value)
                self.assertEqual("invalid_source_path", error.exception.code)

    def test_sdk_readonly_role_rejects_other_actions_recipes_and_versions(self):
        for action, recipe_id, version in (("validate", "penpot-search-native-fill-tests-r1", 1),
                ("generate", "penpot-search-native-fill-tests-r1", 1),
                ("prepare-node", "penpot-public-canonical-tests-r03", 1),
                ("prepare-node", "penpot-search-native-fill-tests-r1", True),
                ("prepare-node", "penpot-search-native-fill-tests-r1", 2)):
            with self.subTest(action=action, recipe=recipe_id, version=version):
                value = self.node_sdk_payload(action)
                value["nodeRecipe"].update(id=recipe_id, schemaVersion=version)
                with self.assertRaises(JenkinsError) as error:
                    self.admit(value)
                self.assertEqual("invalid_source_path", error.exception.code)

    def test_sdk_readonly_role_rejects_unknown_control_paths_and_alias_spelling(self):
        name = sorted(helper._NODE_READONLY_SDK_DEPENDENCIES)[0]
        for bad in ("dev/penpot/other.cljs", ".git/config", ".jenkins/state/control",
                    ".codex/config.toml", name.replace("/", "\\"), "dev/penpot/../other.cljs"):
            with self.subTest(path=bad):
                value = self.node_sdk_payload()
                value["dependencyHashes"][bad] = "a" * 64
                with self.assertRaises(JenkinsError):
                    self.admit(value)

    def test_sdk_readonly_role_requires_exact_present_digest_and_physical_file(self):
        name = sorted(helper._NODE_READONLY_SDK_DEPENDENCIES)[0]
        for bad in (None, "invalid", "c" * 64):
            with self.subTest(digest=bad):
                value = self.node_sdk_payload()
                value["dependencyHashes"][name] = bad
                with self.assertRaises(JenkinsError):
                    self.admit(value)
        value = self.node_sdk_payload()
        (self.repo / name).unlink()
        with self.assertRaises(JenkinsError) as error:
            self.admit(value)
        self.assertEqual("local_input_changed", error.exception.code)

    def test_sdk_readonly_role_still_checks_plain_paths_before_reading(self):
        value = self.node_sdk_payload()
        from tools.jenkins.deployment.driver import _plain
        sdk = self.repo / sorted(helper._NODE_READONLY_SDK_DEPENDENCIES)[0]
        def reject_sdk_alias(path):
            if path == sdk:
                raise JenkinsError("driver_alias_rejected", "fixture alias")
            return _plain(path)
        with patch("tools.jenkins.deployment.driver._plain", side_effect=reject_sdk_alias):
            with self.assertRaises(JenkinsError) as error:
                self.admit(value)
        self.assertEqual("driver_alias_rejected", error.exception.code)

    def palette_payload(self, request="palette"):
        value = self.payload(request=request)
        value.update(action="validate", sourceView="checkout", verb="test",
                     manifestPath="zircon_plugins/Cargo.toml",
                     packages=["zircon_plugin_animation_graph_editor"], features=[],
                     noDefaultFeatures=True, lib=True, locked=True, profile="test",
                     jobs=1, testThreads=1, tests=[], allFeatures=False, noRun=False, offline=False,
                     testFilter="tests::graph_palette_matches_creatable_editor_capabilities")
        value["dependencyHashes"][".cargo/config.toml"] = None
        return value

    def test_palette_absent_compiler_config_is_pinned_without_hidden_path_authority(self):
        value = self.palette_payload()
        before = json.dumps(value, sort_keys=True)
        context = self.admit(value)
        self.assertEqual(before, json.dumps(value, sort_keys=True))
        self.assertNotIn(".cargo", context["paths"])
        admission = self.state.list("local_request_admission")[0]["payload"]
        self.assertEqual({".cargo/config.toml": None}, admission["readOnlyDependencyHashes"])
        self.assertNotIn(".cargo/config.toml", admission["requestedPaths"])
        for action in ("implementation", "validation"):
            with self.subTest(action=action), self.assertRaises(JenkinsError):
                self.state.check_authorization(self.repository, "first", action, [".cargo/config.toml"])

    def test_palette_present_compiler_config_requires_its_actual_hash(self):
        path = self.repo / ".cargo/config.toml"
        path.parent.mkdir()
        path.write_bytes(b"[build]\njobs=1\n")
        value = self.palette_payload()
        value["dependencyHashes"][".cargo/config.toml"] = hashlib.sha256(path.read_bytes()).hexdigest()
        self.admit(value)
        admission = self.state.list("local_request_admission")[0]["payload"]
        self.assertEqual({".cargo/config.toml": value["dependencyHashes"][".cargo/config.toml"]},
                         admission["readOnlyDependencyHashes"])

    def test_palette_absence_pin_rejects_a_new_compiler_config_before_any_context(self):
        value = self.palette_payload()
        path = self.repo / ".cargo/config.toml"
        path.parent.mkdir()
        path.write_bytes(b"changed\n")
        with self.assertRaises(JenkinsError) as error:
            self.admit(value)
        self.assertEqual("local_input_changed", error.exception.code)
        self.assertEqual([], self.state.list("local_request_admission"))
        with self.state.connect() as connection:
            self.assertIsNone(connection.execute("SELECT * FROM authorizations").fetchone())

    def test_palette_config_rejects_bad_missing_and_stale_hashes(self):
        for checksum in ("bad", "a" * 64):
            with self.subTest(checksum=checksum), self.assertRaises(JenkinsError):
                value = self.palette_payload()
                value["dependencyHashes"][".cargo/config.toml"] = checksum
                self.admit(value)
        path = self.repo / ".cargo/config.toml"
        path.parent.mkdir()
        path.write_bytes(b"changed\n")
        value = self.palette_payload()
        value["dependencyHashes"][".cargo/config.toml"] = "a" * 64
        with self.assertRaises(JenkinsError) as error:
            self.admit(value)
        self.assertEqual("local_input_changed", error.exception.code)

    def test_palette_readonly_config_cannot_be_owned_or_authored(self):
        for field in ("ownedPaths", "declaredDependencies", "sourceHashes", "beforeHashes",
                      "afterHashes", "currentHashes"):
            with self.subTest(field=field), self.assertRaises(JenkinsError) as error:
                value = self.palette_payload()
                value[field] = [".cargo/config.toml"] if field in ("ownedPaths", "declaredDependencies") else {
                    ".cargo/config.toml": None}
                self.admit(value)
            self.assertEqual("invalid_source_path", error.exception.code)

    def test_palette_readonly_config_rejects_wider_recipe_and_nonexact_scalar_types(self):
        changes = {"action": "prepare-node", "sourceView": "worktree", "verb": "check",
                   "manifestPath": "Cargo.toml", "packages": ["zircon_editor"],
                   "features": ["extra"], "noDefaultFeatures": False, "lib": False,
                   "locked": False, "profile": "dev", "jobs": 2, "testThreads": 2,
                   "tests": ["other"], "allFeatures": True, "noRun": True, "offline": True,
                   "target": "x86_64-pc-windows-msvc", "testFilter": "tests::",
                   "formalAcceptance": True, "m8Accepted": True}
        for key, changed in [*changes.items(), ("jobs", True), ("testThreads", True),
                             ("noDefaultFeatures", 1), ("formalAcceptance", None)]:
            with self.subTest(field=key, value=changed), self.assertRaises(JenkinsError) as error:
                value = self.palette_payload()
                value[key] = changed
                self.admit(value)
            self.assertEqual("invalid_source_path", error.exception.code)

    def test_palette_readonly_config_rejects_other_hidden_inputs_and_aliases(self):
        for path in (".cargo/config", ".cargo/other.toml", ".git/config", ".jenkins/state/control",
                     ".codex/config.toml", ".cargo\\config.toml", ".cargo/../config.toml"):
            with self.subTest(path=path), self.assertRaises(JenkinsError):
                value = self.palette_payload()
                value["dependencyHashes"][path] = None
                self.admit(value)

    def test_palette_readonly_config_retains_plain_path_guard(self):
        value = self.palette_payload()
        from tools.jenkins.deployment.driver import _plain
        def reject_config_alias(path):
            if path == self.repo / ".cargo/config.toml":
                raise JenkinsError("driver_alias_rejected", "fixture alias")
            return _plain(path)
        with patch("tools.jenkins.deployment.driver._plain", side_effect=reject_config_alias):
            with self.assertRaises(JenkinsError) as error:
                self.admit(value)
        self.assertEqual("driver_alias_rejected", error.exception.code)


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
