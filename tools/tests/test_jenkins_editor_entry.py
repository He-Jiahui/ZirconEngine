import hashlib
import json
import sqlite3
import tempfile
import unittest
from contextlib import closing
from pathlib import Path
from unittest import mock

from tools.dev import jenkins_editor as entry
from tools.dev.jenkins_validate import EntryError
from tools.jenkins.deployment.job_provisioning import _config


class EditorOperatorLifecycleTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "synthetic-repository"
        self.root.mkdir()
        self.hashes = Path(self.temporary.name) / "hashes.json"
        self.hashes.write_text(json.dumps({"src/lib.rs": "a" * 64}), encoding="utf-8")
        self.driver = {"digest": "b" * 64, "runtimeOperationId": "start-synthetic"}
        self.calls = []

    def args(self, action="submit", *extra):
        values = [action, "--repository", str(self.root), "--session-id", "session",
                  "--request-id", "request", "--build-root", r"D:\cargo-targets"]
        if action not in {"query", "resume-registration"}:
            values += ["--current-hashes", str(self.hashes), "--owned-path", "src/lib.rs",
                       "--untracked-path", "src/lib.rs", "--base-head", "c" * 40,
                       "--required-test", "scene::save_history"]
        return entry.parser().parse_args([*values, *extra])

    def call(self, root, driver, folder, domain, action, payload, timeout):
        self.calls.append((domain, action, payload))
        if action == "register":
            return {"status": "registered", "currentSourceRef": "d" * 64}
        if action == "submit":
            return {"status": "queued", "queueId": 1}
        return {"status": "pending", "currentSourceRef": "d" * 64}

    def execute(self, args, call=None, driver=None):
        return entry.execute(args, call=call or self.call,
                             driver_loader=lambda root: driver or self.driver)

    def immutable_driver(self, *, supports_source_root):
        source = (b"SOURCE_ROOT_CAPABILITY = 'managed-worktree-reference-v1'"
                  if supports_source_root else b"def current_source(): return None")
        contents = {
            "launcher.py": b"# synthetic launcher",
            "pipeline/steps.groovy": b"// synthetic steps",
            "tools/jenkins/current_source.py": source,
        }
        files = {
            name: {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
            for name, data in contents.items()
        }
        driver_digest = entry.digest({"schemaVersion": 1, "files": files})
        driver_root = self.root / ".jenkins/runtime/drivers" / driver_digest
        for name, data in contents.items():
            target = driver_root.joinpath(*name.split("/"))
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        manifest = {"schemaVersion": 1, "driverDigest": driver_digest, "files": files}
        (driver_root / "driver-manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
        return {"digest": driver_digest, "runtimeOperationId": "start-synthetic",
                "launcher": driver_root / "launcher.py"}

    def test_dry_check_does_not_dispatch_or_create_lifecycle(self):
        result = self.execute(self.args("dry-check"))
        self.assertEqual("dry-check", result["status"])
        self.assertEqual([], self.calls)
        self.assertFalse((self.root / ".jenkins").exists())
        self.assertFalse(result["formalAcceptance"])

    def test_large_path_scopes_use_json_lists_without_command_length_growth(self):
        paths = [f"src/module_{index:04d}/component.rs" for index in range(2000)]
        self.hashes.write_text(json.dumps({path: "a" * 64 for path in paths}), encoding="utf-8")
        owned = Path(self.temporary.name) / "owned.json"
        untracked = Path(self.temporary.name) / "untracked.json"
        owned.write_text(json.dumps(paths), encoding="utf-8")
        untracked.write_text(json.dumps(paths), encoding="utf-8")
        args = self.args("dry-check")
        args.owned_path = []
        args.untracked_path = []
        args.owned_paths = str(owned)
        args.untracked_paths = str(untracked)
        result = self.execute(args)
        self.assertEqual(2000, result["selectedFileCount"])
        self.assertEqual(paths, result["ownedPaths"])
        self.assertEqual(paths, result["untrackedPaths"])
        self.assertEqual([], self.calls)
        self.assertFalse((self.root / ".jenkins").exists())

    def test_missing_source_leaf_rejects_a_reparse_ancestor_before_dispatch(self):
        original_lstat = Path.lstat
        ancestor = self.root / "src"

        def lstat(path, *args, **kwargs):
            if path == ancestor:
                return mock.Mock(st_mode=0o040755, st_file_attributes=0x400)
            return original_lstat(path, *args, **kwargs)

        with mock.patch.object(Path, "lstat", lstat), self.assertRaises(EntryError) as caught:
            self.execute(self.args("dry-check"))

        self.assertEqual("path_alias_rejected", caught.exception.code)
        self.assertEqual([], self.calls)
        self.assertFalse((self.root / ".jenkins").exists())

    def test_json_path_lists_reject_conflicts_duplicates_and_non_paths(self):
        values = Path(self.temporary.name) / "paths.json"
        cases = [(["src/lib.rs"], False), (["src/lib.rs", "src/lib.rs"], True),
                 ([123], True), ({"src/lib.rs": True}, True)]
        for content, clear_repeated in cases:
            values.write_text(json.dumps(content), encoding="utf-8")
            args = self.args("dry-check")
            args.owned_paths = str(values)
            if clear_repeated:
                args.owned_path = []
            with self.subTest(content=content, clear_repeated=clear_repeated), self.assertRaises(EntryError):
                self.execute(args)
        self.assertEqual([], self.calls)

    def external_input(self):
        external = self.root.parent / "zr_vm"
        external.mkdir()
        item = {"name": "zr_vm", "repositoryRoot": str(external), "baseHead": "c" * 40,
                "currentHashes": {"binding/src/lib.rs": "a" * 64}, "untrackedPaths": ["binding/src/lib.rs"]}
        metadata = self.root.parent / "external.json"
        metadata.write_text(json.dumps([item]), encoding="utf-8")
        return item, metadata

    def test_external_inputs_are_explicit_dependency_metadata_without_source_copy(self):
        item, metadata = self.external_input()
        args = self.args("dry-check", "--external-inputs", str(metadata))
        payload = entry.intent_payload(args, self.root, {"sessionId": "session"})
        self.assertEqual([item], payload["externalInputs"])
        result = self.execute(args)
        self.assertEqual("dry-check", result["status"])
        self.assertEqual([], self.calls)
        self.assertEqual([], list((self.root.parent / "zr_vm").iterdir()))
        self.assertFalse((self.root / ".jenkins").exists())

    def test_external_inputs_reject_other_roots_control_paths_and_fabricated_fields(self):
        item, metadata = self.external_input()
        cases = [{**item, "name": "other"}, {**item, "repositoryRoot": str(self.root)},
                 {**item, "currentHashes": {".git/config": "a" * 64}},
                 {**item, "ownedPaths": ["binding/src/lib.rs"]},
                 {**item, "untrackedPaths": ["unselected.rs"]}]
        for value in cases:
            metadata.write_text(json.dumps([value]), encoding="utf-8")
            with self.subTest(value=value), self.assertRaises(EntryError):
                self.execute(self.args("dry-check", "--external-inputs", str(metadata)))
        self.assertEqual([], self.calls)

    def test_source_root_is_omitted_by_default_and_added_only_when_explicit(self):
        identity = {"sessionId": "session"}
        legacy = entry.intent_payload(self.args("dry-check"), self.root, identity)
        self.assertNotIn("sourceRoot", legacy)

        worktree = self.root / ".jenkins/worktrees/writer-selected"
        worktree.mkdir(parents=True)
        explicit = entry.intent_payload(
            self.args("dry-check", "--source-root", str(worktree), "--worktree-id", "writer-selected"),
            self.root,
            identity,
        )
        self.assertEqual({**legacy, "sourceRoot": str(worktree.absolute()), "worktreeId": "writer-selected"}, explicit)

    def test_old_driver_without_source_root_marker_rejects_before_protocol_or_intent(self):
        worktree = self.root / ".jenkins/worktrees/writer-legacy"
        worktree.mkdir(parents=True)
        driver = self.immutable_driver(supports_source_root=False)
        args = self.args("submit", "--source-root", str(worktree), "--worktree-id", "writer-legacy")

        with self.assertRaises(EntryError) as caught:
            self.execute(args, driver=driver)

        self.assertEqual("source_root_unsupported", caught.exception.code)
        self.assertEqual([], self.calls)
        folder = self.root / ".jenkins/state/requests/operator-editor" / entry.digest(
            [entry.repository_id(self.root), "session", "request"])
        self.assertFalse((folder / "intent.json").exists())
        self.assertFalse((folder / "lifecycle.json").exists())

    def test_replay_keeps_saved_source_root_and_rejects_replacement_before_call_or_rewrite(self):
        worktree = self.root / ".jenkins/worktrees/writer-saved"
        replacement = self.root / ".jenkins/worktrees/writer-replacement"
        worktree.mkdir(parents=True)
        replacement.mkdir()
        driver = self.immutable_driver(supports_source_root=True)

        submitted = self.execute(self.args("submit", "--source-root", str(worktree), "--worktree-id", "writer-saved"), driver=driver)
        self.assertEqual("submitted", submitted["status"])
        self.assertEqual(str(worktree.absolute()), self.calls[0][2]["sourceRoot"])

        folder = self.root / ".jenkins/state/requests/operator-editor" / entry.digest(
            [entry.repository_id(self.root), "session", "request"])
        intent_path = folder / "intent.json"
        lifecycle_path = folder / "lifecycle.json"
        original_intent = intent_path.read_bytes()
        original_lifecycle = lifecycle_path.read_bytes()

        observed = self.execute(self.args("query", "--source-root", str(worktree), "--worktree-id", "writer-saved"), driver=driver)
        self.assertEqual("pending", observed["status"])
        self.assertEqual(["register", "submit", "query"], [call[1] for call in self.calls])
        self.assertEqual(original_intent, intent_path.read_bytes())
        self.assertEqual(original_lifecycle, lifecycle_path.read_bytes())

        with self.assertRaises(EntryError) as caught:
            self.execute(self.args("query", "--source-root", str(replacement), "--worktree-id", "writer-replacement"), driver=driver)
        self.assertEqual("source_root_binding_conflict", caught.exception.code)
        self.assertEqual(["register", "submit", "query"], [call[1] for call in self.calls])
        self.assertEqual(original_intent, intent_path.read_bytes())
        self.assertEqual(original_lifecycle, lifecycle_path.read_bytes())

    def test_repeated_submit_observes_same_request_without_readmission(self):
        first = self.execute(self.args())
        second = self.execute(self.args())
        self.assertEqual("submitted", first["status"])
        self.assertEqual("pending", second["status"])
        self.assertEqual(["register", "submit", "query"], [x[1] for x in self.calls])
        self.assertEqual(first["identity"], self.calls[-1][2]["identity"])
        self.assertEqual(first["currentSourceRef"], self.calls[-1][2]["currentSourceRef"])

    def test_unknown_submission_queries_original_identity_without_retry(self):
        def interrupted(*args):
            if args[4] == "submit":
                self.calls.append((args[3], args[4], args[5]))
                raise EntryError("protocol_delivery_unknown", "timeout", retryable=True)
            return self.call(*args)
        first = self.execute(self.args(), call=interrupted)
        self.assertEqual("delivery_unknown", first["status"])
        second = self.execute(self.args("query"))
        self.assertEqual("pending", second["status"])
        self.assertEqual(["register", "submit", "query"], [x[1] for x in self.calls])
        self.assertFalse(second["formalAcceptance"])

    def test_unknown_registration_queries_without_fabricating_reference(self):
        def interrupted(*args):
            self.calls.append((args[3], args[4], args[5]))
            raise EntryError("protocol_delivery_unknown", "timeout", retryable=True)
        with self.assertRaises(EntryError):
            self.execute(self.args(), call=interrupted)
        result = self.execute(self.args("query"))
        self.assertEqual("pending", result["status"])
        self.assertEqual(["register", "query"], [x[1] for x in self.calls])
        self.assertNotIn("currentSourceRef", self.calls[-1][2])

    def test_replay_rejects_changed_source_intent_before_any_protocol_call(self):
        self.execute(self.args())
        self.hashes.write_text(json.dumps({"src/lib.rs": "e" * 64}), encoding="utf-8")
        with self.assertRaises(EntryError) as caught:
            self.execute(self.args())
        self.assertEqual("request_input_conflict", caught.exception.code)
        self.assertEqual(["register", "submit"], [x[1] for x in self.calls])

    def test_query_rejects_replacement_driver_runtime_attempt_or_build_root(self):
        self.execute(self.args())
        cases = [
            (self.args("query"), {**self.driver, "digest": "e" * 64}),
            (self.args("query"), {**self.driver, "runtimeOperationId": "start-other"}),
            (self.args("query", "--attempt-id", "other-attempt"), self.driver),
            (self.args("query", "--build-root", r"E:\cargo-targets"), self.driver),
            (self.args("query", "--current-source-ref", "e" * 64), self.driver),
        ]
        for args, driver in cases:
            with self.subTest(args=args, driver=driver), self.assertRaises(EntryError):
                self.execute(args, driver=driver)
        self.assertEqual(["register", "submit"], [x[1] for x in self.calls])

    def saved_query_without_lifecycle(self, *extra):
        result = self.execute(self.args("query", *extra))
        folder = self.root / ".jenkins/state/requests/operator-editor" / entry.digest(
            [entry.repository_id(self.root), "session", "request"])
        path = folder / "current-source-query.json"
        control = {name: result[name] for name in ("identity", "buildRoot", "runtimeOperationId")}
        if "currentSourceRef" in result:
            control["currentSourceRef"] = result["currentSourceRef"]
        entry.save_json(path, control)
        self.calls.clear()
        return path

    def test_query_without_lifecycle_reuses_only_the_same_saved_binding(self):
        for extra in ((), ("--current-source-ref", "d" * 64)):
            with self.subTest(extra=extra):
                self.calls.clear()
                path = self.saved_query_without_lifecycle(*extra)
                original = path.read_bytes()
                result = self.execute(self.args("query", *extra))
                self.assertEqual("pending", result["status"])
                self.assertFalse(result["formalAcceptance"])
                self.assertEqual(["query"], [item[1] for item in self.calls])
                self.assertEqual(json.loads(original), self.calls[0][2])
                self.assertEqual(original, path.read_bytes())
                self.assertFalse((path.parent / "lifecycle.json").exists())
                self.assertFalse((path.parent / "intent.json").exists())
                path.unlink()

    def test_query_without_lifecycle_rejects_changed_binding_before_call_or_overwrite(self):
        path = self.saved_query_without_lifecycle("--current-source-ref", "d" * 64)
        original = path.read_bytes()
        cases = [
            (self.args("query", "--current-source-ref", "d" * 64), {**self.driver, "digest": "e" * 64}),
            (self.args("query", "--current-source-ref", "d" * 64), {**self.driver, "runtimeOperationId": "start-other"}),
            (self.args("query", "--current-source-ref", "d" * 64, "--attempt-id", "other-attempt"), self.driver),
            (self.args("query", "--current-source-ref", "d" * 64, "--build-root", r"E:\cargo-targets"), self.driver),
            (self.args("query", "--current-source-ref", "e" * 64), self.driver),
            (self.args("query"), self.driver),
        ]
        for args, driver in cases:
            with self.subTest(args=args, driver=driver), self.assertRaises(EntryError) as caught:
                self.execute(args, driver=driver)
            self.assertEqual("query_binding_conflict", caught.exception.code)
            self.assertEqual([], self.calls)
            self.assertEqual(original, path.read_bytes())
            self.assertFalse((path.parent / "lifecycle.json").exists())
            self.assertFalse((path.parent / "intent.json").exists())

    def test_query_without_lifecycle_rejects_malformed_saved_control_before_call(self):
        path = self.saved_query_without_lifecycle()
        control = json.loads(path.read_bytes())
        cases = [b"{", b"null", b"[]", b"{}"]
        for generation in (True, 1.0, float("nan")):
            cases.append(json.dumps({**control, "identity": {**control["identity"], "generation": generation}}).encode())
        for content in cases:
            path.write_bytes(content)
            with self.subTest(content=content), self.assertRaises(EntryError):
                self.execute(self.args("query"))
            self.assertEqual([], self.calls)
            self.assertEqual(content, path.read_bytes())
            self.assertFalse((path.parent / "lifecycle.json").exists())
            self.assertFalse((path.parent / "intent.json").exists())

    def test_cargo_scope_rejects_duplicate_tests_or_deleted_untracked_files(self):
        with self.assertRaises(EntryError):
            self.execute(self.args("dry-check", "--required-test", "scene::save_history"))
        self.hashes.write_text(json.dumps({"src/lib.rs": None}), encoding="utf-8")
        with self.assertRaises(EntryError):
            self.execute(self.args("dry-check"))
        self.assertEqual([], self.calls)

    def test_query_before_submission_cannot_claim_acceptance_from_preparation(self):
        def prepared(*args):
            self.calls.append((args[3], args[4], args[5]))
            identity = {**args[5]["identity"], "sourceInputDigest": "e" * 64,
                        "coverageDigest": "f" * 64}
            return {"status": "prepared", "preparation": {"payload": {"result": {"identity": identity}}}}
        result = self.execute(self.args("query"), call=prepared)
        self.assertEqual("pending", result["status"])
        self.assertFalse(result["formalAcceptance"])
        self.assertEqual("state_unavailable", result["flow"]["reasonCode"])
        self.assertEqual(["query"], [x[1] for x in self.calls])

    def unknown_registration(self, *, legacy=False):
        def interrupted(root, driver, folder, domain, action, payload, timeout):
            self.calls.append((domain, action, payload))
            entry.save_json(folder / "current-source-register.json", payload, immutable=True)
            raise EntryError("protocol_delivery_unknown", "timeout", retryable=True)
        with self.assertRaises(EntryError):
            self.execute(self.args("submit", "--timeout", "300"), call=interrupted)
        self.folder = self.root / ".jenkins/state/requests/operator-editor" / entry.digest(
            [entry.repository_id(self.root), "session", "request"])
        self.intent = json.loads((self.folder / "intent.json").read_text(encoding="utf-8"))
        self.prior = json.loads((self.folder / "lifecycle.json").read_text(encoding="utf-8"))
        if legacy:
            self.prior.pop("registrationCallId")
            self.prior.pop("registrationTimeoutSeconds")
            entry.save_json(self.folder / "lifecycle.json", self.prior)
        self.dispatch = self.root.parent / "original-control-intent.json"
        entry.save_json(self.dispatch, {"kind": "synthetic-control-intent", "identity": self.prior["identity"]})
        reference = lambda path: {"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
        self.dispatch_result = self.root.parent / "original-control-result.json"
        entry.save_json(self.dispatch_result, {"exitCode": 2, "dispatchIntentSha256": reference(self.dispatch)["sha256"],
                        "operatorResult": {"reasonCode": "protocol_delivery_unknown", "retryable": True,
                                           "formalAcceptance": False}})
        self.receipt = {"schemaVersion": 1, "kind": "jenkins-editor-registration-terminal",
                        "identity": self.prior["identity"], "runtimeOperationId": self.prior["runtimeOperationId"],
                        "buildRoot": self.prior["buildRoot"], "intentDigest": entry.digest(self.intent),
                        "registrationCallId": self.prior.get("registrationCallId", "original"),
                        "protocolInputSha256": reference(self.folder / "current-source-register.json")["sha256"],
                        "protocolImplementationSha256": entry._protocol_implementation(),
                        "operatorHandle": {"kind": "unified-exec", "id": 42, "status": "terminal", "exitCode": 0},
                        "registration": {"domain": "current-source", "action": "register", "status": "terminal",
                                         "cause": "timeout", "timeoutSeconds": 300, "processReaped": True},
                        "evidence": {"dispatchResult": reference(self.dispatch_result), "dispatchIntent": reference(self.dispatch)}}
        self.receipt_path = self.root.parent / "terminal-control-receipt.json"
        entry.save_json(self.receipt_path, self.receipt)

    def resume_args(self, *extra):
        return self.args("resume-registration", "--registration-terminal-receipt", str(self.receipt_path),
                         "--registration-timeout", "900", *extra)

    def resumed_call(self, root, driver, folder, domain, action, payload, timeout):
        if action == "query":
            self.calls.append((domain, action, payload))
            return {"status": "not_found", "retryable": False, "currentSourceRef": None, "preparation": None}
        return self.call(root, driver, folder, domain, action, payload, timeout)

    def registered_state(self, *, dispatched=False, changed=False):
        recorded = {**self.intent, "schemaVersion": 1, "kind": "current-source", "repositoryRoot": str(self.root),
                    "runtimeOperationId": self.prior["runtimeOperationId"], "authorizationSnapshot": "f" * 64}
        if changed:
            recorded = {**recorded, "baseHead": "e" * 40}
        ref = entry.digest(recorded)
        database = self.root / ".jenkins/state/coordination.sqlite3"
        with closing(sqlite3.connect(database)) as connection, connection:
            connection.execute("CREATE TABLE records (domain TEXT, key TEXT, payload TEXT)")
            connection.execute("INSERT INTO records VALUES (?,?,?)", ("current_source_intent", ref, json.dumps(recorded)))
            slot = entry.digest([self.prior["identity"][name] for name in ("repositoryId", "sessionId", "requestId")])
            connection.execute("INSERT INTO records VALUES (?,?,?)", ("current_source_request", slot, json.dumps({"currentSourceRef": ref})))
            if dispatched:
                key = ":".join(self.prior["identity"][name] for name in ("repositoryId", "sessionId", "requestId")) + ":flow"
                connection.execute("INSERT INTO records VALUES (?,?,?)", ("jenkins_submission", key, '{"delivery":"unknown"}'))
        return ref

    def test_resume_uses_saved_intent_after_fresh_not_found_and_preserves_history(self):
        self.unknown_registration(legacy=True)
        original = {path: path.read_bytes() for path in (self.folder / "intent.json", self.dispatch, self.dispatch_result)}
        with mock.patch.object(entry, "intent_payload", side_effect=AssertionError("must not rebuild immutable intent")):
            result = self.execute(self.resume_args(), call=self.resumed_call)
        self.assertEqual("submitted", result["status"])
        self.assertFalse(result["formalAcceptance"])
        self.assertEqual(["register", "query", "register", "submit"], [x[1] for x in self.calls])
        self.assertEqual(self.intent, self.calls[2][2])
        self.assertEqual(self.prior["identity"], self.calls[1][2]["identity"])
        self.assertEqual(self.prior["runtimeOperationId"], self.calls[1][2]["runtimeOperationId"])
        self.assertNotIn("currentSourceRef", self.calls[1][2])
        for path, content in original.items():
            self.assertEqual(content, path.read_bytes())
        history = list((self.folder / "registration-recovery").glob("*.json"))
        self.assertEqual(2, len(history))
        terminal = next(json.loads(path.read_text(encoding="utf-8")) for path in history if path.name.endswith("-terminal.json"))
        self.assertEqual(self.prior, terminal["previousLifecycle"])

    def test_registration_budget_is_separate_and_query_submit_remain_bounded(self):
        self.unknown_registration()
        budgets = []
        def call(*values):
            budgets.append((values[4], values[6]))
            return self.resumed_call(*values)
        self.execute(self.resume_args("--timeout", "300"), call=call)
        self.assertEqual([("query", 300), ("register", 900), ("submit", 300)], budgets)

    def test_normal_registration_retains_default_budget_and_accepts_explicit_budget(self):
        for timeout in (None, 900):
            with self.subTest(timeout=timeout):
                args = self.args("submit", "--request-id", "request-" + str(timeout))
                args.registration_timeout = timeout
                budgets = []
                def call(*values):
                    budgets.append((values[4], values[6]))
                    return self.call(*values)
                self.execute(args, call=call)
                self.assertEqual([("register", timeout or 60), ("submit", 60)], budgets)

    def test_invalid_or_unbounded_budgets_reject_before_protocol_or_metadata(self):
        for key, values in (("timeout", (0, -1, 301, float("nan"), float("inf"), True)),
                            ("registration_timeout", (0, -1, 901, float("nan"), float("inf"), True))):
            for value in values:
                args = self.args()
                setattr(args, key, value)
                with self.subTest(key=key, value=value), self.assertRaises(EntryError) as caught:
                    self.execute(args)
                self.assertEqual("timeout_invalid", caught.exception.code)
        self.assertFalse((self.root / ".jenkins").exists())
        self.assertEqual([], self.calls)

    def test_resume_rejects_every_replacement_input_before_query(self):
        self.unknown_registration()
        changes = [("--current-hashes", str(self.hashes)), ("--base-head", "e" * 40),
                   ("--profile", "shipping"), ("--required-test", "other::test"),
                   ("--owned-path", "other.rs"), ("--untracked-path", "other.rs"),
                   ("--owned-paths", "other.json"), ("--untracked-paths", "other.json"),
                   ("--external-inputs", "other.json"), ("--current-source-ref", "e" * 64),
                   ("--source-root", str(self.root.parent / "replacement-worktree")), ("--worktree-id", "writer-other")]
        for change in changes:
            with self.subTest(change=change), self.assertRaises(EntryError) as caught:
                self.execute(self.resume_args(*change))
            self.assertEqual("registration_recovery_input_conflict", caught.exception.code)
        self.assertEqual(["register"], [x[1] for x in self.calls])

    def test_resume_rejects_changed_identity_runtime_and_build_root(self):
        self.unknown_registration()
        cases = [(self.resume_args("--attempt-id", "other"), self.driver),
                 (self.resume_args(), {**self.driver, "digest": "e" * 64}),
                 (self.resume_args(), {**self.driver, "runtimeOperationId": "other"}),
                 (self.resume_args("--build-root", r"E:\cargo-targets"), self.driver)]
        for args, driver in cases:
            with self.subTest(args=args, driver=driver), self.assertRaises(EntryError):
                self.execute(args, driver=driver)
        self.assertEqual(["register"], [x[1] for x in self.calls])

    def test_resume_requires_existing_unknown_lifecycle_and_terminal_receipt(self):
        with self.assertRaises(EntryError) as caught:
            self.execute(self.args("resume-registration"))
        self.assertEqual("registration_recovery_missing", caught.exception.code)
        self.unknown_registration()
        with self.assertRaises(EntryError) as caught:
            self.execute(self.args("resume-registration"))
        self.assertEqual("registration_terminal_required", caught.exception.code)
        for phase in ("registering", "dispatching", "delivery_unknown", "submitted", "rejected"):
            entry.save_json(self.folder / "lifecycle.json", {**self.prior, "phase": phase})
            with self.subTest(phase=phase), self.assertRaises(EntryError) as caught:
                self.execute(self.resume_args())
            self.assertEqual("registration_recovery_phase", caught.exception.code)
        self.assertEqual(["register"], [x[1] for x in self.calls])

    def test_resume_rejects_live_unreaped_or_mismatched_terminal_evidence(self):
        self.unknown_registration()
        cases = [ {**self.receipt, "schemaVersion": True},
                  {**self.receipt, "identity": {**self.prior["identity"], "requestId": "other"}},
                  {**self.receipt, "runtimeOperationId": "other"}, {**self.receipt, "buildRoot": r"E:\cargo-targets"},
                  {**self.receipt, "intentDigest": "e" * 64}, {**self.receipt, "protocolInputSha256": "e" * 64},
                  {**self.receipt, "protocolImplementationSha256": "e" * 64}, {**self.receipt, "registrationCallId": "other"},
                  {**self.receipt, "operatorHandle": {**self.receipt["operatorHandle"], "status": "running"}},
                  {**self.receipt, "registration": {**self.receipt["registration"], "processReaped": False}},
                  {**self.receipt, "registration": {**self.receipt["registration"], "cause": "unknown"}},
                  {**self.receipt, "registration": {**self.receipt["registration"], "timeoutSeconds": 900}} ]
        for value in cases:
            entry.save_json(self.receipt_path, value)
            with self.subTest(value=value), self.assertRaises(EntryError):
                self.execute(self.resume_args())
        self.assertEqual(["register"], [x[1] for x in self.calls])

    def test_resume_rejects_tampered_original_evidence_and_saved_intent(self):
        self.unknown_registration()
        self.dispatch_result.write_text('{}', encoding="utf-8")
        with self.assertRaises(EntryError):
            self.execute(self.resume_args())
        self.assertEqual(["register"], [x[1] for x in self.calls])
        entry.save_json(self.folder / "intent.json", {**self.intent, "coverage": {"template": "editor_product_shipping"}})
        with self.assertRaises(EntryError):
            self.execute(self.resume_args())
        self.assertEqual(["register"], [x[1] for x in self.calls])

    def test_resume_refuses_unknown_retryable_and_inconsistent_query_observations(self):
        self.unknown_registration()
        original = (self.folder / "lifecycle.json").read_bytes()
        cases = [{"status": "pending"}, {"status": "not_found"}, {"status": "not_found", "retryable": True},
                 {"status": "not_found", "currentSourceRef": "d" * 64},
                 {"status": "not_found", "preparation": {}},
                 {"status": "not_found", "reasonCode": "delivery_unknown"}]
        for response in cases:
            def call(*values):
                self.calls.append((values[3], values[4], values[5]))
                return response
            with self.subTest(response=response), self.assertRaises(EntryError) as caught:
                self.execute(self.resume_args(), call=call)
            self.assertEqual("registration_reconciliation_unknown", caught.exception.code)
            self.assertEqual(original, (self.folder / "lifecycle.json").read_bytes())
        self.assertEqual(["register", *(["query"] * len(cases))], [x[1] for x in self.calls])

    def test_unknown_query_preserves_original_unknown_without_registering(self):
        self.unknown_registration()
        def interrupted(*values):
            self.calls.append((values[3], values[4], values[5]))
            raise EntryError("protocol_delivery_unknown", "timeout", retryable=True)
        with self.assertRaises(EntryError) as caught:
            self.execute(self.resume_args(), call=interrupted)
        self.assertEqual("registration_reconciliation_unknown", caught.exception.code)
        self.assertEqual(["register", "query"], [x[1] for x in self.calls])
        self.assertTrue(list((self.folder / "registration-recovery").glob("*-query-unknown.json")))

    def test_late_registered_reference_is_reused_without_registering_again(self):
        self.unknown_registration()
        ref = self.registered_state()
        def late(*values):
            if values[4] == "query":
                self.calls.append((values[3], values[4], values[5]))
                return {"status": "registered", "currentSourceRef": ref, "retryable": False,
                        "preparation": None, "request": None}
            return self.call(*values)
        result = self.execute(self.resume_args(), call=late)
        self.assertEqual("submitted", result["status"])
        self.assertEqual(ref, result["currentSourceRef"])
        self.assertEqual(["register", "query", "submit"], [x[1] for x in self.calls])

    def test_late_reference_with_existing_submission_marker_is_observed_without_readmission(self):
        self.unknown_registration()
        ref = self.registered_state(dispatched=True)
        def late(*values):
            self.calls.append((values[3], values[4], values[5]))
            return {"status": "registered", "currentSourceRef": ref, "retryable": False}
        result = self.execute(self.resume_args(), call=late)
        self.assertEqual(ref, result["currentSourceRef"])
        self.assertFalse(result["formalAcceptance"])
        self.assertEqual(["register", "query"], [x[1] for x in self.calls])
        self.execute(self.args("submit"), call=late)
        self.assertEqual(["register", "query", "query"], [x[1] for x in self.calls])

    def test_late_reference_requires_authoritative_exact_original_intent(self):
        self.unknown_registration()
        ref = self.registered_state(changed=True)
        def late(*values):
            self.calls.append((values[3], values[4], values[5]))
            return {"status": "registered", "currentSourceRef": ref, "retryable": False}
        with self.assertRaises(EntryError) as caught:
            self.execute(self.resume_args(), call=late)
        self.assertEqual("registration_reference_mismatch", caught.exception.code)
        self.assertEqual(["register", "query"], [x[1] for x in self.calls])

    def test_late_preparation_or_request_is_only_observed_without_dispatch(self):
        self.unknown_registration()
        ref = self.registered_state()
        for field in ("preparation", "request"):
            entry.save_json(self.folder / "lifecycle.json", self.prior)
            def late(*values):
                self.calls.append((values[3], values[4], values[5]))
                return {"status": "registered", "currentSourceRef": ref, "retryable": False, field: {}}
            result = self.execute(self.resume_args(), call=late)
            self.assertEqual(ref, result["currentSourceRef"])
            self.assertFalse(result["formalAcceptance"])
        self.assertEqual(["register", "query", "query"], [x[1] for x in self.calls])

    def test_second_unknown_registration_cannot_reuse_first_terminal_proof(self):
        self.unknown_registration()
        def interrupted(*values):
            if values[4] == "register":
                self.calls.append((values[3], values[4], values[5]))
                raise EntryError("protocol_delivery_unknown", "timeout", retryable=True)
            return self.resumed_call(*values)
        with self.assertRaises(EntryError):
            self.execute(self.resume_args(), call=interrupted)
        self.assertEqual(["register", "query", "register"], [x[1] for x in self.calls])
        with self.assertRaises(EntryError) as caught:
            self.execute(self.resume_args())
        self.assertEqual("registration_evidence_mismatch", caught.exception.code)
        self.assertEqual(["register", "query", "register"], [x[1] for x in self.calls])


class EditorPipelineParameterTests(unittest.TestCase):
    def test_current_source_references_reach_sandboxed_job_without_defaults(self):
        import xml.etree.ElementTree as ET
        environment = {
            "ZIRCON_REPO_ROOT": "E:/synthetic", "JENKINS_PYTHON": "python.exe",
            "ZIRCON_SEALED_DRIVER": "steps", "ZIRCON_DRIVER_LAUNCHER": "launcher",
            "ZIRCON_DRIVER_DIGEST": "a" * 64, "ZIRCON_REPOSITORY_ID": "repository",
            "ZIRCON_BUILD_ROOT": r"D:\cargo-targets", "ZIRCON_AGENT_LABEL": "agent",
            "ZIRCON_RUNTIME_OPERATION_ID": "start-synthetic",
        }
        config = ET.fromstring(_config("// synthetic pipeline", environment))
        definitions = config.findall("./properties/hudson.model.ParametersDefinitionProperty/parameterDefinitions/*")
        parameters = {x.findtext("name"): x.findtext("defaultValue") or "" for x in definitions}
        self.assertEqual("", parameters["CURRENT_SOURCE_REF"])
        self.assertEqual("", parameters["SOURCE_PREPARATION_REF"])
        self.assertEqual("true", config.findtext("./definition/sandbox"))


if __name__ == "__main__":
    unittest.main()
