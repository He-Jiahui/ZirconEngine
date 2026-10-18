"""Contract and lifecycle checks for the actual-checkout editor product."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("checkout_product_under_test",
                                            ROOT / "tools/jenkins-management/checkout_product.py")
PRODUCT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PRODUCT)


class CheckoutProductTests(unittest.TestCase):
    def payload(self):
        response, query = PRODUCT.contract(ROOT)
        amendment = response["contractAmendment"]
        payload = {"action": "validate", "sourceView": "checkout", "buildRoot": "E:\\cargo-targets",
                   "repositoryId": query["identity"]["repositoryId"],
                   "sessionId": amendment["currentSuccessorAuthorization"]["sessionId"],
                   "requestId": "typed-preview-test", "operationId": "preview-development-test",
                   "driverDigest": amendment["selectedDriverDigest"],
                   "runtimeOperationId": amendment["runtimeOperationId"],
                   "baseHead": "a" * 40, "ownedPaths": ["zircon_app"],
                   "untrackedPaths": ["zircon_app/src/new.rs"],
                   "sourceHashes": {"zircon_app/src/main.rs": "b" * 64,
                                    "zircon_app/src/new.rs": "c" * 64},
                   "expectedEngineSourceDigest": "d" * 64,
                   "externalGitlinks": {name: {"head": "e" * 40, "indexCommit": "e" * 40}
                                       for name in PRODUCT.GITLINKS}}
        payload["checkoutProduct"] = {"schemaVersion": 1, "kind": PRODUCT.KIND,
              "contractRef": PRODUCT.exact_ref(ROOT, PRODUCT.RESPONSE, PRODUCT.RESPONSE_SHA),
              "parentQueryRef": PRODUCT.exact_ref(ROOT, PRODUCT.PARENT_QUERY, PRODUCT.PARENT_QUERY_SHA),
              "logicalIdentity": {name: payload[name] for name in PRODUCT.IDENTITY_FIELDS},
              "sourcePins": {name: copy.deepcopy(payload[name]) for name in PRODUCT.PIN_FIELDS}}
        return payload

    def v2_payload(self):
        payload = self.payload()
        payload["checkoutProduct"]["schemaVersion"] = 2
        payload["expectedEngineSourceDigest"] = None
        payload["externalCapturePolicy"] = "jenkins-admission"
        payload["externalRoots"] = [str(ROOT.parent / "zr_vm")]
        self.sync_pins(payload)
        return payload

    def sync_pins(self, payload):
        payload["checkoutProduct"]["sourcePins"] = {
            name: copy.deepcopy(payload[name]) for name in PRODUCT.PIN_FIELDS}

    def admission_capture(self, *, head="a" * 40, files=None):
        source = {"head": head, "files": files or {
            "zircon_app/src/main.rs": "b" * 64,
            "zircon_app/src/new.rs": "c" * 64}, "diffSha256": "e" * 64}
        source["sourceDigest"] = PRODUCT.sha(PRODUCT.encoded(source))
        before = {"sourceView": "checkout", "sourceRoot": str(ROOT), "source": source,
                  "externalRoots": [], "dependencyCache": {"home": "E:/cargo-targets/cargo-home"}}
        before["sourceDigest"] = PRODUCT.sha(PRODUCT.encoded(before))
        return before

    def refresh_capture(self, before):
        source = before["source"]
        source_body = {name: value for name, value in source.items() if name != "sourceDigest"}
        source["sourceDigest"] = PRODUCT.sha(PRODUCT.encoded(source_body))
        body = {name: value for name, value in before.items() if name != "sourceDigest"}
        before["sourceDigest"] = PRODUCT.sha(PRODUCT.encoded(body))

    def test_v1_requires_a_complete_engine_source_digest(self):
        for missing in (False, True):
            with self.subTest(missing=missing):
                payload = self.payload()
                if missing:
                    del payload["expectedEngineSourceDigest"]
                else:
                    payload["expectedEngineSourceDigest"] = None
                    self.sync_pins(payload)
                with self.assertRaises(ValueError):
                    PRODUCT.validate_product(payload, ROOT)

    def test_v2_admits_explicit_null_only_with_full_jenkins_capture(self):
        payload = self.v2_payload()
        PRODUCT.validate_product(payload, ROOT)
        batch = PRODUCT.validation_batch({**payload, "batch": PRODUCT.batch_payload(payload, ROOT)}, ROOT)
        self.assertEqual(batch["coverage"]["sourceAdmissionPolicy"], "jenkins-admission")
        for missing in ("expectedEngineSourceDigest", "externalCapturePolicy"):
            changed = copy.deepcopy(payload)
            del changed[missing]
            with self.subTest(missing=missing), self.assertRaises(ValueError):
                PRODUCT.validate_product(changed, ROOT)
        stale = copy.deepcopy(payload)
        stale["expectedSourceDigest"] = "f" * 64
        with self.assertRaises(ValueError):
            PRODUCT.validate_product(stale, ROOT)

    def test_v2_binds_captured_head_and_owned_hashes(self):
        payload = self.v2_payload()
        before = self.admission_capture()
        PRODUCT.validate_admission_capture(payload, before, ROOT)
        module_spec = importlib.util.spec_from_file_location("product_hash_guard_runner",
                ROOT / "tools/jenkins-management/runner.py")
        runner = importlib.util.module_from_spec(module_spec)
        module_spec.loader.exec_module(runner)
        manager = object.__new__(runner.Management)
        manager.verify_declared_inputs(before, payload)
        changed_head = copy.deepcopy(before)
        changed_head["source"]["head"] = "f" * 40
        self.refresh_capture(changed_head)
        with self.assertRaisesRegex(ValueError, "baseHead"):
            PRODUCT.validate_admission_capture(payload, changed_head, ROOT)
        changed_file = copy.deepcopy(before)
        changed_file["source"]["files"]["zircon_app/src/main.rs"] = "f" * 64
        self.refresh_capture(changed_file)
        with self.assertRaisesRegex(ValueError, "Declared input hash changed"):
            manager.verify_declared_inputs(changed_file, payload)

    def test_v2_rejects_extra_environment_parent_or_recipe_identity(self):
        payload = self.v2_payload()
        for change in ("environment", "parent", "identity"):
            changed = copy.deepcopy(payload)
            if change == "environment":
                changed["environment"] = {"RUSTFLAGS": "-C opt-level=0"}
            elif change == "parent":
                changed["checkoutProduct"]["parentQueryRef"]["sha256"] = "0" * 64
            else:
                changed["checkoutProduct"]["logicalIdentity"]["operationId"] = "another-operation"
            with self.subTest(change=change), self.assertRaises(ValueError):
                PRODUCT.validate_product(changed, ROOT)
        payload["batch"] = PRODUCT.batch_payload(payload, ROOT)
        payload["batch"]["stepEnvironments"][1]["RUSTFLAGS"] = "-C opt-level=0"
        with self.assertRaises(ValueError):
            PRODUCT.validation_batch(payload, ROOT)

    def test_actual_contract_preserves_parent_query_bytes(self):
        path = ROOT / PRODUCT.PARENT_QUERY
        raw = path.read_bytes()
        response, query = PRODUCT.contract(ROOT)
        self.assertEqual(query["identity"]["requestId"], "mvp-preview-20261012-r13")
        self.assertEqual(response["contractAmendment"]["exactCargoCommands"],
                         [list(command) for command in PRODUCT.COMMANDS])
        self.assertEqual(path.read_bytes(), raw)

    def test_directory_scope_and_explicit_file_hashes_are_valid(self):
        payload = self.payload()
        PRODUCT.validate_product(payload, ROOT)
        self.assertNotIn("zircon_app", payload["sourceHashes"])

    def test_fixed_batch_preserves_both_complete_commands(self):
        payload = self.payload()
        payload["batch"] = PRODUCT.batch_payload(payload, ROOT)
        batch = PRODUCT.validation_batch(payload, ROOT)
        self.assertEqual([["cargo", *row["cargoArguments"]] for row in batch["coverage"]["recipes"]],
                         [list(command) for command in PRODUCT.COMMANDS])
        self.assertEqual([recipe["packages"] for recipe in batch["recipes"]],
                         [["zircon_app"], ["zircon_runtime"]])
        self.assertEqual(batch["stepEnvironments"], [{}, {}])
        self.assertIsNone(batch["reuseValidation"])

    def test_fixed_batch_rejects_missing_flag_and_extra_environment(self):
        for change in ("argv", "environment", "single"):
            with self.subTest(change=change):
                payload = self.payload()
                payload["batch"] = PRODUCT.batch_payload(payload, ROOT)
                if change == "argv":
                    payload["batch"]["recipes"][1]["cargoArguments"].remove("--offline")
                elif change == "environment":
                    payload["batch"]["stepEnvironments"][0]["RUSTFLAGS"] = "-C opt-level=0"
                else:
                    payload["jobs"] = 8
                with self.assertRaises(ValueError):
                    PRODUCT.validation_batch(payload, ROOT)

    def test_declared_source_pins_cannot_change_behind_product(self):
        payload = self.payload()
        payload["sourceHashes"]["zircon_app/src/main.rs"] = "f" * 64
        with self.assertRaises(ValueError):
            PRODUCT.validate_product(payload, ROOT)

    def test_owned_paths_cannot_exceed_active_validation_scope(self):
        payload = self.payload()
        payload["ownedPaths"].append("tools/jenkins-management/runner.py")
        self.sync_pins(payload)
        with self.assertRaises(ValueError):
            PRODUCT.validate_product(payload, ROOT)

    def test_untracked_source_requires_exact_hash_and_owned_scope(self):
        for change in ("missing", "outside"):
            with self.subTest(change=change):
                payload = self.payload()
                if change == "missing":
                    del payload["sourceHashes"]["zircon_app/src/new.rs"]
                else:
                    payload["untrackedPaths"].append("tools/new.py")
                    payload["sourceHashes"]["tools/new.py"] = "1" * 64
                self.sync_pins(payload)
                with self.assertRaises(ValueError):
                    PRODUCT.validate_product(payload, ROOT)
        payload = self.v2_payload()
        payload["sourceHashes"]["zircon_app/src/new.rs"] = None
        self.sync_pins(payload)
        with self.assertRaisesRegex(ValueError, "untracked source file"):
            PRODUCT.validate_product(payload, ROOT)

    def test_seven_external_gitlinks_must_match_index_and_head(self):
        for change in ("missing", "drift"):
            with self.subTest(change=change):
                payload = self.payload()
                name = sorted(PRODUCT.GITLINKS)[0]
                if change == "missing":
                    del payload["externalGitlinks"][name]
                else:
                    payload["externalGitlinks"][name]["head"] = "0" * 40
                self.sync_pins(payload)
                with self.assertRaises(ValueError):
                    PRODUCT.validate_product(payload, ROOT)

    def test_runtime_identity_contract_and_root_cannot_be_substituted(self):
        for field, value in (("runtimeOperationId", "other"), ("driverDigest", "0" * 64),
                             ("buildRoot", "D:\\cargo-targets"), ("sourceView", "worktree"),
                             ("commit", True), ("reuseValidation", {"receiptRef": {}})):
            with self.subTest(field=field):
                payload = self.payload()
                payload[field] = value
                with self.assertRaises(ValueError):
                    PRODUCT.validate_product(payload, ROOT)

    def test_capture_flattens_actual_nested_gitlinks_and_external_sources(self):
        child = {"head": "a" * 40, "sourceDigest": "b" * 64,
                 "files": {"dep.c": "c" * 64}}
        before = {"source": {"files": {"main.rs": "d" * 64, "deleted.rs": None,
                  ".jenkins/pipeline/control.groovy": "f" * 64, "nested/target/output": "f" * 64,
                  "vendor/library": {"gitlinkCommit": "e" * 40, "source": child}}},
                  "externalRoots": [{"path": "E:\\Git\\zr_vm", "source": child}]}
        self.assertEqual(PRODUCT.source_inventory_from_capture(before),
                         {"source/main.rs": "d" * 64, "source/vendor/library/dep.c": "c" * 64,
                          "zr_vm/dep.c": "c" * 64})

    def test_capture_rejects_unknown_root_incomplete_hash_and_alias_path(self):
        for before in (
                {"source": {"files": {"main.rs": "invalid"}}},
                {"source": {"files": {"../outside": "a" * 64}}},
                {"source": {"files": {}}, "externalRoots": [
                    {"path": "E:\\Git\\unknown", "source": {"files": {"file": "a" * 64}}}]}):
            with self.subTest(before=before), self.assertRaises(ValueError):
                PRODUCT.source_inventory_from_capture(before)

    def native_steps(self):
        steps = []
        for index, argv in enumerate(PRODUCT.COMMANDS, 1):
            command = ["E:\\tools\\cargo.exe", *argv[1:]]
            steps.append({"command": command, "cwd": str(ROOT), "complete": True, "childrenGone": True,
                          "stdoutEof": True, "stderrEof": True, "activeProcesses": 0, "processExitCode": 0,
                          "identity": {"pid": index, "creationTime": "123456",
                                       "executable": command[0],
                                       "commandDigest": hashlib.sha256("\0".join(command).encode()).hexdigest()},
                          "nativeEvidence": {"path": "E:\\cargo-targets\\zircon-local\\native.json",
                                             "sha256": "1" * 64}})
        return steps

    def test_both_exact_native_steps_are_required(self):
        PRODUCT.validate_native_steps(self.native_steps(), ROOT)
        with self.assertRaises(ValueError):
            PRODUCT.validate_native_steps(self.native_steps()[:1], ROOT)

    def test_failed_or_unjoined_native_step_cannot_publish(self):
        for field, value in (("stdoutEof", False), ("stderrEof", False), ("childrenGone", False),
                             ("complete", False), ("activeProcesses", 1), ("processExitCode", 1)):
            with self.subTest(field=field):
                steps = self.native_steps()
                steps[0][field] = value
                with self.assertRaises(ValueError):
                    PRODUCT.validate_native_steps(steps, ROOT)

    def test_native_command_and_owner_identity_cannot_be_changed(self):
        for change in ("argv", "cwd", "birth", "digest"):
            with self.subTest(change=change):
                steps = self.native_steps()
                if change == "argv":
                    steps[1]["command"].append("--release")
                elif change == "cwd":
                    steps[0]["cwd"] = "E:\\other-checkout"
                elif change == "birth":
                    steps[0]["identity"]["creationTime"] = "0"
                else:
                    steps[0]["identity"]["commandDigest"] = "0" * 64
                with self.assertRaises(ValueError):
                    PRODUCT.validate_native_steps(steps, ROOT)

    def test_packaging_holds_guard_and_uses_only_owned_native_launcher(self):
        import os
        import tempfile
        from types import SimpleNamespace
        from unittest.mock import Mock
        environment = {"TEMP": os.environ["TEMP"], "TMP": os.environ["TMP"]}
        key = "a" * 64
        guard, launcher = Mock(spec=["assert_stable"]), Mock()
        events = []
        def stage(repo, staging, artifacts, **options):
            events.append("stage")
            self.assertEqual(options["cargo_profile"], "development")
            self.assertEqual(options["environment"], environment)
            self.assertEqual(set(artifacts), {"zircon_editor.exe", "zircon_runtime.dll"})
            self.assertTrue(all(origin.parent.name == "debug" for origin, _ in artifacts.values()))
            builder.subprocess.run(["owned-crt-signature"], env=environment, capture_output=True, check=False)
            return {"build_set": {"schema": "test-build-set"}, "official_crt": {},
                    "output_closure": {"files": [], "digest": "closure"}}
        def publish(staging, final, closure):
            events.append("publish")
            self.assertGreaterEqual(guard.assert_stable.call_count, 2)
            return closure
        builder = SimpleNamespace(stage_editor_bundle=stage, publish_editor_bundle=publish,
                    verify_editor_bundle=lambda path, closure: closure,
                    subprocess=SimpleNamespace(CREATE_NO_WINDOW=0))
        loader = SimpleNamespace(exec_module=lambda module: None)
        spec = SimpleNamespace(name="mock_pinned_editor", loader=loader)
        with tempfile.TemporaryDirectory(prefix="typed-editor-", dir=os.environ["TEMP"]) as td:
            target = Path(td) / "target"
            (target / "debug").mkdir(parents=True)
            for name in ("zircon_editor.exe", "zircon_runtime.dll"):
                (target / "debug" / name).write_bytes(name.encode())
            with patch.object(PRODUCT.importlib.util, "spec_from_file_location", return_value=spec), \
                 patch.object(PRODUCT.importlib.util, "module_from_spec", return_value=builder):
                result = PRODUCT.stage_and_publish(ROOT, target, Path(td) / "out", key,
                             {"source/main.rs": "f" * 64}, self.native_steps(), guard, environment, launcher)
        self.assertEqual(events, ["stage", "publish"])
        self.assertEqual(guard.assert_stable.call_count, 3)
        launcher.assert_called_once_with(["owned-crt-signature"], env=environment,
                                        capture_output=True, check=False)
        self.assertFalse(result["formalAcceptance"])
        self.assertFalse(result["m8Accepted"])

    def test_unstable_guard_prevents_any_packaging_import_or_publication(self):
        from unittest.mock import Mock
        guard = Mock(spec=["assert_stable"])
        guard.assert_stable.side_effect = ValueError("source changed")
        with patch.object(PRODUCT.importlib.util, "spec_from_file_location") as load_builder:
            with self.assertRaisesRegex(ValueError, "source changed"):
                PRODUCT.stage_and_publish(ROOT, Path("E:/cargo-targets/zircon-local/target"),
                    Path("E:/cargo-targets/zircon-local/out"), "a" * 64, {}, self.native_steps(),
                    guard, {}, Mock())
            load_builder.assert_not_called()

    def test_published_product_rejects_actual_bundle_mutation(self):
        import os
        import tempfile
        from tools.jenkins.validation.receipts import closure_manifest
        with tempfile.TemporaryDirectory(prefix="typed-product-closure-", dir=os.environ["TEMP"]) as td:
            bundle = Path(td) / "bundle"
            bundle.mkdir()
            file = bundle / "zircon_editor.exe"
            file.write_bytes(b"original-fixture")
            product = {"kind": PRODUCT.KIND, "buildSet": {}, "bundle": str(bundle),
                       "outputClosure": closure_manifest(bundle, [file]), "officialCrt": {},
                       "formalAcceptance": False, "m8Accepted": False}
            PRODUCT.verify_published_product(ROOT, product, bundle)
            file.write_bytes(b"foreign-mutation")
            with self.assertRaisesRegex(ValueError, "differs"):
                PRODUCT.verify_published_product(ROOT, product, bundle)

    def test_delivery_reconciliation_rejects_missing_packaging_proofs(self):
        from unittest.mock import Mock
        spec = importlib.util.spec_from_file_location("product_delivery_runner",
                ROOT / "tools/jenkins-management/runner.py")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        task = object.__new__(module.Management)
        task.config, task.repo, task.key = {}, ROOT, "a" * 64
        task.payload = {"checkoutProduct": {"kind": PRODUCT.KIND}}
        task.request, task.operation = "preview-test", "preview-operation"
        task.outputs = lambda: Path("E:/cargo-targets/zircon-local/management")
        task.batch_checkpoint = lambda: {"receipt": {"path": "unused"}}
        task.batch_claims = lambda: [{"status": "complete", "id": "one"},
                                   {"status": "complete", "id": "two"}]
        task.batch_step_proof = lambda item: (None, None, None, item["id"])
        guard = {"complete": True}
        saved = {"status": "passed", "request": {}, "checkoutGuardRef": {},
                 "checkoutGuard": guard, "nativeProofRefs": ["one", "two"], "packagingNativeProofs": []}
        reader = {"payload": {"status": "released", "pinsReleased": True,
                  "continuousReadProven": True, "checkoutGuardRef": {}}}
        task.state = Mock()
        task.state.get.return_value = reader
        helper = Mock()
        with patch.object(module, "checkout_product_helper", return_value=helper), \
             patch.object(module, "read_reference", side_effect=[saved, {"payload": task.payload}, guard]), \
             patch.object(module, "validation_inputs_verified", return_value=True):
            with self.assertRaisesRegex(ValueError, "owned CRT verification"):
                task.verify_checkout_product_delivery(saved)
        helper.verify_published_product.assert_not_called()


if __name__ == "__main__":
    unittest.main()
