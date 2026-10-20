"""Fixed product contracts, with generated synthetic bytes and no native build."""
from __future__ import annotations

import hashlib
import io
import json
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import MagicMock, patch

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins import editor_product as product
from tools.jenkins.tests import test_recipe_planning as planning_fixture
from tools.jenkins.processes.execution_host import ExecutionHost
from tools.jenkins.resources.paths import build_root_containing
from tools.jenkins.workflow.planning import RecipePlanner
from tools.jenkins.validation.receipts import TrustedReceiptAuthority


class EditorProductTests(unittest.TestCase):
    def setUp(self):
        self.fixture = planning_fixture.RecipePlanningTests("test_unknown_impact_is_rejected")
        self.fixture.setUp()
        self.addCleanup(self.fixture.tearDown)
        self.root, self.state = self.fixture.root, self.fixture.state
        self.objects = self.root / "cas"
        self.synthetic = {
            "Cargo.toml": b"[workspace]\n", "Cargo.lock": b"# synthetic lock\n",
            "zircon_runtime/assets/fonts/default.font.toml": b'source = "synthetic.font"\n',
            "zircon_runtime/assets/fonts/editor-ui.font.toml": b'source = "synthetic.font"\n',
            "zircon_runtime/assets/fonts/synthetic.font": b"synthetic font bytes",
            "zircon_editor/assets/editor.asset": b"synthetic editor asset",
            "zircon_runtime_interface/src/runtime_build_set/interface_spec_v1.json": b"{}",
            "zircon_runtime_interface/src/runtime_build_set/payload_schema_set_v1.json": b"{}",
            "tools/export/product_build_profiles.toml": b"# synthetic profile",
        }
        self.synthetic.update({path: ("synthetic template: " + path + "\n").encode("utf-8")
                               for path in product.PROJECT_TEMPLATE_INPUTS})
        self.synthetic.update({name + "/Cargo.toml": b"[package]\n" for name in product.PACKAGES})
        self.synthetic.update({name.replace(".", "/") + ".py": b"# synthetic helper\n" for name in product.HELPER_MODULES})
        self.synthetic["tools/build/build_editor.py"] = b"from tools.dev import local_cargo\nfrom tools.build.zircon_build_runtime_manifest import synthetic\n"
        self.manifest = {"schemaVersion": 1, "baseHead": "a" * 40, "external": [],
                         "preparationKind": "current-source", "entries": []}
        for path, data in self.synthetic.items():
            self.entry(path, data)
        self.source = self.fixture.source
        self.template = "editor_product_development"
        self.configure()

    def entry(self, path, data):
        checksum = hashlib.sha256(data).hexdigest()
        obj = self.objects / "inputs/objects" / checksum
        obj.parent.mkdir(parents=True, exist_ok=True)
        obj.write_bytes(data)
        self.manifest["entries"] = [entry for entry in self.manifest["entries"] if entry["path"] != path]
        self.manifest["entries"].append({"path": path, "status": "present", "sha256": checksum,
                                         "size": len(data), "objectDigest": checksum})

    def configure(self):
        coverage = {"sourceDigest": self.source, "coverageDigest": "c" * 64, "template": self.template,
                    "requiredTests": ["focused::passes"], "noDefaultFeatures": True}
        old = self.state.get("sealed_input", self.source)
        self.state.put("sealed_input", self.source, {**old["payload"], "manifest": self.manifest,
                         "sourceManifest": self.manifest, "coverage": coverage, "objectRoot": str(self.objects)},
                       expected_version=old["version"])
        metadata = {"packages": [{"id": name + " 0.1", "name": name,
                       "manifest_path": self.fixture.workspace + "\\" + name + "\\Cargo.toml",
                       "targets": [{"name": name, "kind": ["lib"], "src_path": self.fixture.workspace + "\\" + name + "\\src\\lib.rs"}],
                       "dependencies": []} for name in product.PACKAGES]}
        evidence = self.state.get("sealed_cargo_evidence", self.source)
        self.state.put("sealed_cargo_evidence", self.source, {**evidence["payload"], "cargoMetadata": metadata,
                         "metadataDigest": digest(metadata)}, expected_version=evidence["version"])
        policy = self.state.get("resource_policy", "default")
        estimates = {**policy["payload"]["recipeEstimates"], "editor-product": {"cpu": 4, "memoryBytes": 8 * 1024**3, "diskBytes": 32 * 1024**3}}
        self.state.put("resource_policy", "default", {**policy["payload"], "recipeEstimates": estimates}, expected_version=policy["version"])

    def plan(self, **changes):
        return RecipePlanner(self.state, repo_root=self.root).plan(sealed_input_ref=self.source,
                   driver=self.fixture.driver, changed_paths=["tools/build/build_editor.py"], template=self.template, **changes)

    def test_fixed_product_phases_outputs_and_preparation_identity(self):
        plan = self.plan()
        recipe = plan.recipe
        self.assertEqual(recipe["phases"], list(product.PHASES))
        self.assertEqual(recipe["requiredNativeStages"], list(product.NATIVE_STAGES))
        self.assertEqual(recipe["packages"], list(product.PACKAGES))
        self.assertEqual(recipe["resourceProfile"], "editor-product")
        self.assertEqual(recipe["testCount"], 0)
        self.assertEqual(recipe["requiredTests"], ["focused::passes"])
        self.assertIn("--list", recipe["commands"]["list"])
        self.assertIn("--no-run", recipe["commands"]["compile"])
        self.assertEqual(recipe["commands"]["check"][-1], "check")
        self.assertEqual(recipe["commands"]["editor_exe"][-1], "editor_exe")
        self.assertIn("bundle/zircon_runtime.dll.manifest.json", recipe["requiredOutputsByStage"]["package"])
        self.assertIn("bundle/vcruntime140.dll", recipe["requiredOutputsByStage"]["package"])
        self.assertFalse(recipe["nativeUiAcceptance"])
        self.assertFalse(recipe["shippingAcceptance"])
        self.assertEqual(self.plan().recipe_ref, plan.recipe_ref)

    def test_every_embedded_project_template_is_required_before_execution(self):
        entries = self.manifest["entries"]
        for path in product.PROJECT_TEMPLATE_INPUTS:
            with self.subTest(path=path):
                self.manifest["entries"] = [entry for entry in entries if entry["path"] != path]
                self.configure()
                with self.assertRaises(JenkinsError) as error:
                    self.plan()
                self.assertEqual(error.exception.code, "editor_inputs_missing")
                self.assertIn(path, error.exception.details["paths"])

    def test_deleted_template_cannot_satisfy_the_production_input_contract(self):
        path = "templates/projects/renderable-empty/zircon-project.toml"
        self.manifest["entries"] = [{**entry, "status": "deleted", "sha256": None, "objectDigest": None}
                                    if entry["path"] == path else entry for entry in self.manifest["entries"]]
        self.configure()
        with self.assertRaises(JenkinsError) as error:
            self.plan()
        self.assertEqual(error.exception.code, "editor_inputs_missing")
        self.assertIn(path, error.exception.details["paths"])

    def test_corrupt_template_object_fails_before_native_execution(self):
        path = "templates/projects/renderable-empty/assets/models/cube.obj"
        entry = next(entry for entry in self.manifest["entries"] if entry["path"] == path)
        (self.objects / "inputs/objects" / entry["objectDigest"]).write_bytes(b"corrupt template")
        with self.assertRaises(JenkinsError) as error:
            self.plan()
        self.assertEqual(error.exception.code, "editor_template_corrupt")
        self.assertEqual(error.exception.details["path"], path)

    def test_template_bytes_are_bound_to_the_immutable_product_recipe(self):
        first = self.plan()
        closure = first.recipe["productTemplateClosure"]
        self.assertEqual(len(closure), 17)
        self.assertEqual(first.recipe["productTemplateClosureDigest"], digest(closure))
        path = "templates/projects/renderable-empty/zircon-project.toml"
        replacement = b"synthetic changed project descriptor\n"
        self.entry(path, replacement)
        self.configure()
        second = self.plan()
        self.assertEqual(second.recipe["productTemplateClosure"][path], hashlib.sha256(replacement).hexdigest())
        self.assertNotEqual(first.recipe["productTemplateClosureDigest"], second.recipe["productTemplateClosureDigest"])
        self.assertNotEqual(first.recipe_ref, second.recipe_ref)
        self.assertEqual(first.recipe["preparationKey"], second.recipe["preparationKey"])

    def test_shipping_has_fixed_features_target_and_archive(self):
        development = self.plan()
        self.template = "editor_product_shipping"
        self.configure()
        shipping = self.plan()
        self.assertIn("x86_64-pc-windows-msvc", shipping.recipe["commands"]["compile"])
        self.assertIn("zircon_app/shipping-editor,zircon_runtime/shipping-editor", shipping.recipe["commands"]["test"])
        self.assertIn("bundle.zip", shipping.recipe["requiredOutputsByStage"]["package"])
        self.assertNotEqual(shipping.recipe["preparationKey"], development.recipe["preparationKey"])

    def test_product_rejects_caller_profile_and_missing_trusted_estimate(self):
        with self.assertRaises(JenkinsError):
            self.plan(profile="release")
        with self.assertRaises(JenkinsError):
            self.plan(features=["default"])
        policy = self.state.get("resource_policy", "default")
        estimates = {key: value for key, value in policy["payload"]["recipeEstimates"].items() if key != "editor-product"}
        self.state.put("resource_policy", "default", {**policy["payload"], "recipeEstimates": estimates}, expected_version=policy["version"])
        with self.assertRaises(JenkinsError) as error:
            self.plan()
        self.assertEqual(error.exception.code, "resource_estimate_missing")

    def test_transitive_helpers_missing_or_corrupt_fail_before_execution(self):
        self.entry("tools/dev/local_cargo.py", b"import tools.dev.synthetic_extra\n")
        with self.assertRaises(JenkinsError) as error:
            product.helper_closure(self.manifest, self.objects)
        self.assertEqual(error.exception.code, "editor_helper_missing")
        self.entry("tools/dev/synthetic_extra.py", b"synthetic = 1\n")
        closure = product.helper_closure(self.manifest, self.objects)
        self.assertIn("tools/dev/synthetic_extra.py", closure)
        entry = next(entry for entry in self.manifest["entries"] if entry["path"] == "tools/dev/synthetic_extra.py")
        (self.objects / "inputs/objects" / entry["objectDigest"]).write_bytes(b"changed")
        with self.assertRaises(JenkinsError):
            product.helper_closure(self.manifest, self.objects)

    def test_namespace_packages_and_missing_font_payload(self):
        closure = product.helper_closure(self.manifest, self.objects)
        self.assertNotIn("tools/build/__init__.py", closure)
        self.manifest["entries"] = [entry for entry in self.manifest["entries"] if not entry["path"].endswith("synthetic.font")]
        self.configure()
        with self.assertRaises(JenkinsError) as error:
            self.plan()
        self.assertEqual(error.exception.code, "editor_font_missing")

    def test_product_phases_require_native_job_identity(self):
        authority = TrustedReceiptAuthority(self.state)
        for stage in product.NATIVE_STAGES:
            with self.subTest(stage=stage), self.assertRaises(JenkinsError):
                authority.register_attempt({"executionId": "synthetic:" + stage, "stage": stage})
        host = object.__new__(ExecutionHost)
        self.assertEqual(host._phase_names(self.plan().recipe, None), list(product.PHASES))
        with self.assertRaises(JenkinsError):
            host._phase_names({"kind": "cargo"}, ["editor_exe"])

    def test_product_output_rejects_prior_logs_and_artifacts(self):
        output = self.root / "phase"
        product._fresh_output(output)
        (output / "stdout.log").write_text("host output", encoding="utf-8")
        (output / "stderr.log").write_text("", encoding="utf-8")
        with self.assertRaises(JenkinsError):
            product._fresh_output(output)
        (output / "artifact.json").write_text("{}", encoding="utf-8")
        with self.assertRaises(JenkinsError):
            product._fresh_output(output)

    def test_host_separates_logs_and_product_outputs_and_binds_native_attempt_ids(self):
        recipe = self.plan().recipe
        phases = ["check", "editor_exe", "runtime_dll", "package"]
        recipe = {**recipe, "phases": phases, "requiredOutputGlobsByPhase": {}, "packages": [],
                  "buildRoot": str(build_root_containing(self.root).path)}
        eid = "synthetic-editor-native"
        payload = {"status": "running", "recipeRef": "f" * 64, "sourceDigest": self.source,
                   "coverageDigest": "c" * 64, "driverDigest": "b" * 64,
                   "phaseLogRoot": str(self.root / "logs"), "phaseOutputRoot": str(self.root / "outputs")}
        self.state.put("execution", eid, payload)
        host = object.__new__(ExecutionHost)
        host.state, host.execution_id = self.state, eid
        host.host = SimpleNamespace(to_dict=lambda: {"operationId": "synthetic", "generation": "1"})
        host.registry = MagicMock()
        host.job, host.native_job_id, host.poll_interval = None, None, 0
        proof = {"complete": True, "processExitCode": 0, "stdoutEof": True,
                 "stderrEof": True, "childrenGone": True}
        host.registry.register.return_value = SimpleNamespace(native_job_id="synthetic-job")
        host.registry.record_terminal.return_value = SimpleNamespace(native_job_id="synthetic-job",
            payload={"terminalProof": proof, "completeProof": True})
        attempts = {}
        authority = MagicMock()
        authority.register_attempt.side_effect = lambda attempt: attempts.update({attempt["executionId"]: attempt})
        authority.record_terminal.side_effect = lambda attempt_id, proof, closure: {"executionId": attempt_id, "stage": attempts[attempt_id]["stage"]}

        def native(command, **options):
            phase = command[-1]
            output = Path(options["env"]["ZIRCON_PRODUCT_OUTPUT_ROOT"])
            logs = options["log_dir"]
            self.assertNotEqual(output, logs)
            product._fresh_output(output)
            logs.mkdir(parents=True)
            (logs / "stdout.log").write_text("synthetic native output", encoding="utf-8")
            (logs / "stderr.log").write_text("", encoding="utf-8")
            for name in recipe["requiredOutputsByPhase"][phase]:
                if name in {"stdout.log", "stderr.log"}:
                    continue
                destination = output / name
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(b"synthetic phase artifact")
            return SimpleNamespace(identity=SimpleNamespace(pid=101, creation_time="synthetic-birth"),
                poll=lambda: 0, wait=lambda **kwargs: None, close=lambda: None,
                proof=lambda: SimpleNamespace(complete=True))

        with patch.object(host, "_trusted_recipe", return_value=(recipe, {})), \
                patch("tools.jenkins.editor_product._verify_source") as verify_source, \
                patch("tools.jenkins.processes.execution_host.NativeJob.launch", side_effect=native), \
                patch("tools.jenkins.processes.execution_host.TrustedReceiptAuthority", return_value=authority):
            result = host.serve_phases()
        self.assertEqual(result["status"], "terminal")
        self.assertEqual(set(attempts), {eid + ":" + phase for phase in phases})
        self.assertEqual(verify_source.call_count, 2 * len(phases))
        for phase in phases:
            self.assertTrue((self.root / "outputs" / phase / "stdout.log").is_file())
        self.assertEqual(result["phaseRecords"][1]["stageReceipts"]["editor_exe"], eid + ":editor_exe")

    def test_actual_test_results_are_required_in_addition_to_discovery(self):
        result = product.verify_test_run("test focused::passes ... ok\ntest result: ok. 1 passed; 0 failed;\n", ["focused::passes"])
        self.assertEqual(result["passedCount"], 1)
        for text in ("focused::passes: test\n", "test result: ok. 0 passed; 0 failed;\n",
                     "test unrelated ... ok\ntest result: ok. 1 passed; 0 failed;\n"):
            with self.subTest(text=text), self.assertRaises(JenkinsError):
                product.verify_test_run(text, ["focused::passes"])

    def test_prepared_source_excludes_deletions_and_rejects_extra_files(self):
        source = self.root / "synthetic-prepared"
        source.mkdir()
        data = b"synthetic source input"
        (source / "present.rs").write_bytes(data)
        manifest = {"entries": [{"path": "present.rs", "status": "present", "sha256": hashlib.sha256(data).hexdigest()},
                                 {"path": "deleted.rs", "status": "deleted", "sha256": None}]}
        recipe = {"sourceDigest": digest(manifest), "buildRoot": str(build_root_containing(self.root).path)}
        captured = io.StringIO()
        with patch.object(product, "canonical_build_root", wraps=product.canonical_build_root) as roots, \
                patch.object(product.sys, "stderr", captured):
            product._verify_source(recipe, {"sourceManifest": manifest}, source,
                                   progress_context="existing-source-fixture")
        self.assertEqual(roots.call_count, 2)
        reports = [json.loads(line) for line in captured.getvalue().splitlines()]
        self.assertEqual([row["boundary"] for row in reports], ["start", "external", "complete"])
        self.assertEqual(reports[-1]["verifiedFiles"], 1)
        self.assertEqual(reports[-1]["hashedBytes"], len(data))
        self.assertEqual(reports[-1]["context"], "existing-source-fixture")
        approved = product.canonical_build_root(recipe["buildRoot"])
        changed = SimpleNamespace(path=approved.path, volume_identity=approved.volume_identity + "-changed")
        with patch.object(product, "canonical_build_root", side_effect=[approved, changed]), \
                self.assertRaises(JenkinsError) as root_error:
            product._verify_source(recipe, {"sourceManifest": manifest}, source)
        self.assertEqual(root_error.exception.code, "editor_build_root_changed")
        def cancel_scan():
            raise JenkinsError("execution_verification_cancelled", "fixture zero-ref cancellation")
        with patch.object(product, "canonical_build_root", wraps=product.canonical_build_root) as roots, \
                self.assertRaises(JenkinsError) as cancelled:
            product._verify_source(recipe, {"sourceManifest": manifest}, source, checkpoint=cancel_scan)
        self.assertEqual(cancelled.exception.code, "execution_verification_cancelled")
        self.assertEqual(roots.call_count, 2)
        (source / "deleted.rs").write_bytes(b"unexpected stale deletion")
        with self.assertRaises(JenkinsError):
            product._verify_source(recipe, {"sourceManifest": manifest}, source)


if __name__ == "__main__":
    unittest.main()
