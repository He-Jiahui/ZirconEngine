"""Bounded nested-workspace admission and cache identity; no compiler runs."""
from __future__ import annotations
import importlib.util
import hashlib
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

ROOT = Path(__file__).absolute().parents[3]

def load(name, relative):
    spec = importlib.util.spec_from_file_location(name, ROOT / relative)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

entry = load("plugin_manifest_entry", "tools/jenkins-management/entry.py")
runner = load("plugin_manifest_runner", "tools/jenkins-management/runner.py")
local = load("plugin_manifest_local", "tools/dev/local_cargo.py")

class CaptureMetadataObservationTests(unittest.TestCase):
    def sample(self, *, head="1", names=("a.rs",), obj="2", debug=b"stat-before"):
        return [(head * 40 + "\n").encode(), b"".join(name.encode() + b"\0" for name in names),
                ("100644 " + obj * 40 + " 0\ta.rs\0").encode(), debug]

    def observe(self, samples, *, limit=64 * 1024 * 1024):
        values = iter([raw for sample in samples for raw in sample])
        calls, results = [], []

        def original(root, *arguments, **options):
            value = SimpleNamespace(stdout=next(values), returncode=0)
            calls.append((root, arguments, options))
            results.append(value)
            return value

        observer = entry.CaptureMetadataObserver(original, limit=limit)
        for _sample in samples:
            for command in observer.commands:
                result = observer(ROOT, *command, check=True)
                self.assertIs(result, results[-1])
        self.assertEqual(len(calls), len(samples) * 4)
        self.assertTrue(all(options == {"check": True} for _, _, options in calls))
        return observer

    def test_observer_preserves_git_calls_and_reports_unchanged_pairs_without_attribution(self):
        observer = self.observe([self.sample(), self.sample()])
        data = observer.observation(runner)
        self.assertTrue(data["complete"])
        delta = data["differences"][0]
        self.assertFalse(delta["headChanged"] or delta["indexDebugChanged"])
        self.assertEqual(delta["addedPaths"], [])
        self.assertEqual(delta["removedPaths"], [])
        self.assertEqual(delta["indexEntryChanges"], [])
        self.assertIsNone(delta["attribution"])

    def test_observer_keeps_exact_head_index_membership_and_untruncated_raw_debug(self):
        import base64
        before = self.sample(names=("a.rs", "old.rs"))
        after = self.sample(head="3", obj="4", names=("a.rs", *[f"added-{i}.rs" for i in range(25)]), debug=b"all-stat-bytes\x00\xff")
        data = self.observe([before, after]).observation(runner)
        delta = data["differences"][0]
        self.assertEqual(delta["headBefore"], "1" * 40)
        self.assertEqual(delta["headAfter"], "3" * 40)
        self.assertEqual(len(delta["addedPaths"]), 25)
        self.assertEqual(delta["removedPaths"], ["old.rs"])
        self.assertEqual(delta["indexEntryChanges"][0]["before"]["object"], "2" * 40)
        self.assertEqual(delta["indexEntryChanges"][0]["after"]["object"], "4" * 40)
        self.assertTrue(delta["indexDebugChanged"])
        record = data["roots"][str(ROOT)][-1]["rawReads"][-1]
        self.assertEqual(base64.b64decode(record["stdoutBase64"]), after[-1])
        self.assertEqual(record["sha256"], entry.sha(after[-1]))

    def test_budget_exhaustion_marks_diagnostics_incomplete_without_changing_results(self):
        data = self.observe([self.sample(), self.sample()], limit=1).observation(runner)
        self.assertFalse(data["complete"])
        self.assertEqual(data["roots"], {})
        self.assertEqual(data["differences"], [])

    def test_failed_capture_metadata_is_owned_compressed_and_idempotent(self):
        import gzip
        import io
        import json
        from tools.jenkins.resources import paths
        base = Path(os.environ["ZR_COMMIT_TEST_ROOT"])
        with tempfile.TemporaryDirectory(prefix="capture-diagnostic-", dir=base) as folder:
            root = Path(folder)
            observer = self.observe([self.sample(), self.sample(head="3")])
            payload = {name: name + "-fixture" for name in (
                "repositoryId", "sessionId", "requestId", "operationId", "driverDigest", "runtimeOperationId")}
            error = ValueError("HEAD, full index or complete source membership changed during capture")
            stderr = io.StringIO()
            with patch.object(paths, "canonical_build_root", return_value=SimpleNamespace(path=root)), \
                 patch.object(paths, "physical_path_under", side_effect=lambda approved, value: Path(value)), \
                 patch.object(entry.sys, "stderr", stderr):
                observer.save_failure({"module": runner}, payload, error)
                observer.save_failure({"module": runner}, payload, error)
            artifacts = list(root.rglob("metadata.json.gz"))
            self.assertEqual(len(artifacts), 1)
            data = json.loads(gzip.decompress(artifacts[0].read_bytes()))
            self.assertEqual(data["identity"], payload)
            self.assertEqual(data["originalError"], str(error))
            self.assertFalse(data["validationResubmitted"])
            ownership = json.loads((artifacts[0].parent / "artifacts.json").read_text())
            self.assertEqual(ownership["artifacts"][0]["sha256"], entry.sha(artifacts[0].read_bytes()))
            self.assertFalse(ownership["sourceCopies"])
            self.assertLess(len(stderr.getvalue()), 4096)

    def test_source_capture_rethrows_the_original_failure_and_restores_git_environment(self):
        error = ValueError("HEAD, full index or complete source membership changed during capture")

        class Actor:
            def git(self, *_args, **_options):
                raise AssertionError("No extra Git calls are needed to report a rejected capture")

            def validation_with_cache(self):
                raise error

        context = {"repo": ROOT, "config": {}, "manager": object(), "module": SimpleNamespace(Management=Actor)}
        payload = {"repositoryId": "repo", "sessionId": "original-session", "ownedPaths": ["a.rs"]}
        previous = os.environ.get("GIT_OPTIONAL_LOCKS")
        with patch.object(entry.CaptureMetadataObserver, "save_failure") as save:
            with self.assertRaises(ValueError) as raised:
                entry.source_digest(context, payload)
            self.assertIs(raised.exception, error)
            save.assert_called_once_with(context, payload, error)
        self.assertEqual(os.environ.get("GIT_OPTIONAL_LOCKS"), previous)


class PluginManifestTests(unittest.TestCase):
    def argv(self, manifest="zircon_plugins/Cargo.toml"):
        return ["test", "--manifest-path", manifest, "-p", "zircon_plugin_material_editor_editor",
                "--no-default-features", "--locked", "--lib", "--profile", "test", "--jobs", "1",
                "material_graph_", "--", "--test-threads", "1"]

    def recipe(self, value):
        actor = object.__new__(runner.Management)
        actor.payload = value
        return actor.validation_recipe()

    def test_requested_absolute_cli_preserves_exact_scope_through_server_and_command(self):
        value = entry.parse_cargo(self.argv(str(ROOT / "zircon_plugins/Cargo.toml")), repo=ROOT)
        self.assertEqual(value["manifestPath"], "zircon_plugins/Cargo.toml")
        result = self.recipe(value)
        self.assertEqual(result["packages"], ["zircon_plugin_material_editor_editor"])
        self.assertEqual(result["features"], [])
        self.assertEqual(result["profile"], "test")
        self.assertTrue(result["noDefaultFeatures"])
        self.assertTrue(result["lib"])
        self.assertEqual(result["testFilter"], "material_graph_")
        self.assertEqual(result["testThreads"], 1)
        command = local.prepare_command(result["command"], Path("E:/cargo-targets/zircon-local/manifest-test/target"))
        flag = command.index("--manifest-path")
        self.assertEqual(command[flag + 1], "zircon_plugins/Cargo.toml")
        self.assertEqual(command.count("--manifest-path"), 1)
        self.assertEqual(command[-3:], ["--", "--test-threads", "1"])
        self.assertEqual(command[:command.index("--")].count("material_graph_"), 1)
        self.assertFalse(set(command) & {"--workspace", "--all", "--all-features", "--all-targets"})

    def test_inline_manifest_and_relative_batch_parser_are_canonical(self):
        argv = self.argv()
        argv[1:3] = ["--manifest-path=zircon_plugins/Cargo.toml"]
        self.assertEqual(entry.parse_cargo(argv)["manifestPath"], "zircon_plugins/Cargo.toml")
        self.assertEqual(self.recipe(entry.parse_cargo(self.argv()))["manifestPath"], "zircon_plugins/Cargo.toml")

    def test_cli_rejects_foreign_alias_parent_and_non_catalog_manifests(self):
        for value in ["../Cargo.toml", "zircon_plugins/../Cargo.toml", "./zircon_plugins/Cargo.toml",
                      "zircon_plugins//Cargo.toml", "zircon_plugins/material_editor/editor/Cargo.toml",
                      "zircon_plugins\\Cargo.toml", "ZIRCON_PLUGINS/Cargo.toml", "C:/foreign/Cargo.toml",
                      str(ROOT.parent / "foreign/Cargo.toml"), "zircon_plugins/Cargo.toml\n"]:
            with self.subTest(value=value), self.assertRaises(entry.EntryError):
                entry.parse_cargo(self.argv(value), repo=ROOT)

    def test_repeated_manifest_and_manifest_wildcard_packages_are_rejected(self):
        with self.assertRaises(entry.EntryError):
            argv = self.argv()
            separator = argv.index("--")
            entry.parse_cargo(argv[:separator] + ["--manifest-path", "zircon_plugins/Cargo.toml"] + argv[separator:])
        for extra in [["-p", "zircon_editor"], ["--workspace"], ["--all-features"], ["--all-targets"]]:
            argv = self.argv()
            separator = argv.index("--")
            with self.subTest(extra=extra), self.assertRaises(entry.EntryError):
                entry.parse_cargo(argv[:separator] + extra + argv[separator:])

    def test_raw_json_cannot_select_free_cwd_or_unbounded_manifest(self):
        value = entry.parse_cargo(self.argv())
        for extra in [{"manifestPath": None}, {"manifestPath": []}, {"manifestPath": "../Cargo.toml"},
                      {"manifestPath": str(ROOT / "zircon_plugins/Cargo.toml")}, {"cwd": str(ROOT)},
                      {"workingDirectory": "zircon_plugins"}, {"allFeatures": True},
                      {"packages": ["zircon_plugin_material_editor_editor", "zircon_editor"]}]:
            with self.subTest(extra=extra), self.assertRaises(ValueError):
                self.recipe({**value, **extra})

    def test_plugin_json_recipe_rejects_package_profile_feature_and_harness_widening(self):
        value = entry.parse_cargo(self.argv())
        for extra in [{"packages": ["zircon_editor"]}, {"profile": "release"}, {"profile": "dev"},
                      {"features": ["native-host"]}, {"locked": False}, {"jobs": 2}, {"testThreads": 2},
                      {"noRun": True}, {"bins": True}, {"testFilter": "other_"}, {"testFilter": None},
                      {"exact": True}, {"ignored": True}, {"harnessFilters": ["other_"]},
                      {"lib": False, "tests": ["other"]}]:
            with self.subTest(extra=extra), self.assertRaises(ValueError):
                self.recipe({**value, **extra})

    def test_plugin_cli_uses_the_same_narrow_policy_even_when_general_policy_is_disabled(self):
        for old, new in [("zircon_plugin_material_editor_editor", "zircon_editor"), ("test", "check"),
                         ("material_graph_", "other_"), ("--lib", "--bins")]:
            argv = self.argv()
            argv[argv.index(old)] = new
            with self.subTest(old=old), self.assertRaises(entry.EntryError):
                entry.parse_cargo(argv, enforce_policy=False)

    def test_existing_root_recipes_keep_their_original_shape(self):
        value = entry.parse_cargo(["check", "-p", "zr_math", "--no-default-features", "--lib", "--locked"])
        result = self.recipe(value)
        self.assertNotIn("manifestPath", result)
        self.assertNotIn("--manifest-path", result["command"])
        self.assertEqual(runner.cargo_manifest_environment(ROOT, result), {})

    def test_manifest_must_exist_in_the_actual_admitted_source_root(self):
        base = Path(os.environ["ZR_COMMIT_TEST_ROOT"])
        with tempfile.TemporaryDirectory(prefix="plugin-manifest-", dir=base) as folder:
            root = Path(folder)
            with self.assertRaisesRegex(ValueError, "absent"):
                runner.cargo_manifest_environment(root, {"manifestPath": "zircon_plugins/Cargo.toml"})
            (root / "zircon_plugins").mkdir()
            (root / "zircon_plugins/Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
            self.assertEqual(runner.cargo_manifest_environment(root, {"manifestPath": "zircon_plugins/Cargo.toml"}),
                             {"cargoWorkspaceManifest": "zircon_plugins/Cargo.toml"})
            with patch.object(runner, "plain", side_effect=ValueError("alias/reparse rejected")):
                with self.assertRaisesRegex(ValueError, "alias/reparse"):
                    runner.cargo_manifest_environment(root, {"manifestPath": "zircon_plugins/Cargo.toml"})

    def test_pool_cannot_reuse_identical_package_name_from_another_workspace(self):
        root_recipe = self.recipe(entry.parse_cargo(["check", "-p", "zr_math", "--no-default-features", "--lib"]))
        nested_recipe = {**root_recipe, "manifestPath": "zircon_plugins/Cargo.toml"}
        actor = object.__new__(runner.Management)
        actor.repo_id, actor.payload = "repo", {}
        observed = []
        def key(repo, trusted, environment, *rest):
            observed.append(environment)
            return runner.sha(runner.encoded(environment))
        preparation = SimpleNamespace(preparation_key=key)
        with patch.object(runner, "plain", side_effect=lambda value: Path(value)), \
             patch.object(Path, "is_file", return_value=True), \
             patch.object(Path, "read_bytes", return_value=b"config"):
            root_pool, _ = actor.batch_pool([root_recipe], preparation, {}, {}, ROOT, {})
            nested_pool, _ = actor.batch_pool([nested_recipe], preparation, {}, {}, ROOT, {})
        self.assertNotEqual(root_pool, nested_pool)
        self.assertNotIn("cargoWorkspaceManifest", observed[0])
        self.assertEqual(observed[1]["cargoWorkspaceManifest"], "zircon_plugins/Cargo.toml")


class PrefabManifestTests(unittest.TestCase):
    def argv(self):
        return ["test", "--manifest-path", str(ROOT / "zircon_plugins/Cargo.toml"),
                "-p", "zircon_plugin_prefab_tools_editor", "--no-default-features", "--locked",
                "--lib", "--profile", "test", "--jobs", "1", "prefab_", "--", "--test-threads", "1"]

    def recipe(self, value):
        return PluginManifestTests().recipe(value)

    def test_exact_prefab_cli_matches_receiver_recipe_and_native_arguments(self):
        value = self.recipe(entry.parse_cargo(self.argv(), repo=ROOT))
        self.assertEqual(value["packages"], ["zircon_plugin_prefab_tools_editor"])
        self.assertEqual(value["features"], [])
        self.assertEqual(value["profile"], "test")
        self.assertEqual(value["testFilter"], "prefab_")
        self.assertTrue(value["lib"] and value["locked"] and value["noDefaultFeatures"])
        self.assertEqual(value["tests"], [])
        command = local.prepare_command(value["command"], Path("E:/cargo-targets/zircon-local/manifest-test/target"))
        self.assertEqual(command[command.index("--manifest-path") + 1], "zircon_plugins/Cargo.toml")
        self.assertEqual(command[command.index("--profile") + 1], "test")
        self.assertEqual(command[-3:], ["--", "--test-threads", "1"])
        self.assertEqual(command[:command.index("--")].count("prefab_"), 1)
        self.assertNotIn("--features", command)
        self.assertFalse(set(command) & {"--workspace", "--all-features", "--all-targets", "--tests", "--bins"})

    def test_prefab_receiver_cannot_borrow_another_package_filter_or_widen_scope(self):
        original = entry.parse_cargo(self.argv(), repo=ROOT)
        for change in [{"packages": ["zircon_plugin_material_editor_editor"]},
                       {"packages": ["zircon_plugin_prefab_tools_editor", "zircon_editor"]},
                       {"packages": ["zircon_editor"]}, {"features": ["runtime"]},
                       {"profile": "dev"}, {"profile": "release"}, {"verb": "check"},
                       {"testFilter": "material_graph_"}, {"testFilter": "prefab"}, {"testFilter": None},
                       {"lib": False, "tests": ["other"]}, {"tests": ["other"]},
                       {"noDefaultFeatures": False}, {"allFeatures": True}, {"allTests": True},
                       {"bins": True}, {"locked": False}, {"jobs": 2}, {"testThreads": 2},
                       {"noRun": True}, {"harnessFilters": ["other"]}, {"exact": True}, {"ignored": True},
                       {"nocapture": True}, {"cwd": str(ROOT)}, {"workingDirectory": "zircon_plugins"},
                       {"manifestPath": "zircon_plugins/prefab_tools/editor/Cargo.toml"}]:
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.recipe({**original, **change})

    def test_prefab_cli_catalog_policy_remains_active_without_the_general_policy(self):
        for old, new in [("prefab_", "material_graph_"), ("prefab_", "prefab"),
                         ("zircon_plugin_prefab_tools_editor", "zircon_plugin_ai_editor"),
                         ("--lib", "--bins"), ("--no-default-features", "--all-features")]:
            argv = self.argv()
            argv[argv.index(old)] = new
            with self.subTest(old=old, new=new), self.assertRaises(entry.EntryError):
                entry.parse_cargo(argv, repo=ROOT, enforce_policy=False)


class ParticlesManifestTests(unittest.TestCase):
    def argv(self):
        return ["test", "--manifest-path", str(ROOT / "zircon_plugins/Cargo.toml"),
                "-p", "zircon_plugin_particles_editor", "--no-default-features", "--locked",
                "--lib", "--profile", "test", "--jobs", "1", "source_tests::", "--", "--test-threads", "1"]

    def test_particles_cli_receiver_and_physical_native_arguments_agree(self):
        value = PluginManifestTests().recipe(entry.parse_cargo(self.argv(), repo=ROOT))
        self.assertEqual(value["packages"], ["zircon_plugin_particles_editor"])
        self.assertEqual(value["features"], [])
        self.assertEqual(value["tests"], [])
        self.assertEqual(value["testFilter"], "source_tests::")
        self.assertEqual(value["profile"], "test")
        self.assertEqual((value["jobs"], value["testThreads"]), (1, 1))
        self.assertTrue(value["locked"] and value["lib"] and value["noDefaultFeatures"])
        command = local.prepare_command(value["command"], Path("E:/cargo-targets/zircon-local/particles-route/target"))
        self.assertEqual(command[command.index("--manifest-path") + 1], "zircon_plugins/Cargo.toml")
        self.assertEqual(command[:command.index("--")].count("source_tests::"), 1)
        self.assertEqual(command[-3:], ["--", "--test-threads", "1"])
        self.assertNotIn("--features", command)
        self.assertFalse(set(command) & {"--workspace", "--all-features", "--all-targets", "--tests", "--bins"})

    def test_particles_receiver_rejects_foreign_filters_and_scope_widening(self):
        original = entry.parse_cargo(self.argv(), repo=ROOT)
        for change in [{"packages": ["zircon_plugin_ai_editor"]},
                       {"packages": ["zircon_plugin_particles_editor", "zircon_editor"]},
                       {"packages": ["zircon_editor"]}, {"features": ["runtime"]},
                       {"profile": "dev"}, {"profile": "release"}, {"verb": "check"},
                       {"testFilter": "authoring_tests::"}, {"testFilter": "material_graph_"},
                       {"testFilter": "source_tests"}, {"testFilter": "source_curve_tests::"}, {"testFilter": None},
                       {"lib": False, "tests": ["other"]}, {"noDefaultFeatures": False},
                       {"allFeatures": True}, {"locked": False}, {"jobs": 2}, {"testThreads": 2},
                       {"noRun": True}, {"harnessFilters": ["other"]}, {"exact": True}, {"ignored": True},
                       {"nocapture": True}, {"cwd": str(ROOT)},
                       {"manifestPath": "zircon_plugins/particles/editor/Cargo.toml"}]:
            with self.subTest(change=change), self.assertRaises(ValueError):
                PluginManifestTests().recipe({**original, **change})

    def test_particles_cli_policy_cannot_be_disabled_or_borrow_another_manifest(self):
        for old, new in [("source_tests::", "source_tests"), ("source_tests::", "authoring_tests::"),
                         ("source_tests::", "material_graph_"),
                         ("zircon_plugin_particles_editor", "zircon_plugin_ai_editor"),
                         ("--lib", "--bins"), ("--no-default-features", "--all-features"),
                         (str(ROOT / "zircon_plugins/Cargo.toml"), "zircon_plugins/particles/editor/Cargo.toml"),
                         (str(ROOT / "zircon_plugins/Cargo.toml"), "./zircon_plugins/Cargo.toml")]:
            argv = self.argv()
            argv[argv.index(old)] = new
            with self.subTest(old=old, new=new), self.assertRaises(entry.EntryError):
                entry.parse_cargo(argv, repo=ROOT, enforce_policy=False)

    def test_particles_pool_identity_cannot_reuse_ai_or_root_workspace_outputs(self):
        actor = object.__new__(runner.Management)
        actor.repo_id, actor.payload = "repo", {}
        preparation = SimpleNamespace(preparation_key=lambda *args: runner.sha(runner.encoded(args)))
        particles = PluginManifestTests().recipe(entry.parse_cargo(self.argv(), repo=ROOT))
        ai = PluginManifestTests().recipe(entry.parse_cargo(AiManifestTests().argv(), repo=ROOT))
        root_recipe = {k: v for k, v in particles.items() if k != "manifestPath"}
        with patch.object(runner, "plain", side_effect=lambda value: Path(value)), \
             patch.object(Path, "is_file", return_value=True), \
             patch.object(Path, "read_bytes", return_value=b"config"):
            pools = [actor.batch_pool([recipe], preparation, {}, {}, ROOT, {})[0]
                     for recipe in [particles, ai, root_recipe]]
        self.assertEqual(len(set(pools)), 3)


class AiManifestTests(unittest.TestCase):
    filters = ("authoring_tests::", "gameplay_debugger_tests::", "tests::pie_mirror_", "tests::perception_overlay_")

    def argv(self, selected="authoring_tests::"):
        return ["test", "--manifest-path", str(ROOT / "zircon_plugins/Cargo.toml"),
                "-p", "zircon_plugin_ai_editor", "--no-default-features", "--locked",
                "--lib", "--profile", "test", "--jobs", "1", selected, "--", "--test-threads", "1"]

    def test_four_ai_filters_preserve_receiver_scope_and_physical_native_arguments(self):
        for selected in self.filters:
            with self.subTest(selected=selected):
                value = PluginManifestTests().recipe(entry.parse_cargo(self.argv(selected), repo=ROOT))
                self.assertEqual(value["packages"], ["zircon_plugin_ai_editor"])
                self.assertEqual(value["features"], [])
                self.assertEqual(value["testFilter"], selected)
                self.assertEqual(value["profile"], "test")
                self.assertEqual(value["jobs"], 1)
                self.assertEqual(value["testThreads"], 1)
                self.assertTrue(value["locked"] and value["lib"] and value["noDefaultFeatures"])
                command = local.prepare_command(value["command"], Path("E:/cargo-targets/zircon-local/ai-route/target"))
                self.assertEqual(command[command.index("--manifest-path") + 1], "zircon_plugins/Cargo.toml")
                self.assertEqual(command[:command.index("--")].count(selected), 1)
                self.assertEqual(command[-3:], ["--", "--test-threads", "1"])
                self.assertNotIn("--features", command)
                self.assertFalse(set(command) & {"--workspace", "--all-features", "--all-targets", "--tests", "--bins"})

    def test_ai_raw_receiver_rejects_cross_package_filters_and_scope_widening(self):
        original = entry.parse_cargo(self.argv(), repo=ROOT)
        for change in [{"packages": ["zircon_plugin_particles_editor"]},
                       {"packages": ["zircon_plugin_material_editor_editor"]},
                       {"packages": ["zircon_plugin_ai_editor", "zircon_editor"]},
                       {"features": ["runtime"]}, {"profile": "dev"}, {"profile": "release"},
                       {"testFilter": "material_graph_"}, {"testFilter": "prefab_"},
                       {"testFilter": "authoring_tests"}, {"testFilter": None},
                       {"lib": False, "tests": ["other"]}, {"locked": False}, {"noDefaultFeatures": False},
                       {"jobs": 2}, {"testThreads": 2}, {"noRun": True}, {"harnessFilters": ["other"]},
                       {"exact": True}, {"ignored": True}, {"nocapture": True},
                       {"cwd": str(ROOT)}, {"manifestPath": "zircon_plugins/ai/editor/Cargo.toml"}]:
            with self.subTest(change=change), self.assertRaises(ValueError):
                PluginManifestTests().recipe({**original, **change})

    def test_ai_cli_scope_cannot_be_disabled_for_query_or_borrowed_by_another_package(self):
        for old, new in [("authoring_tests::", "authoring_tests"), ("authoring_tests::", "prefab_"),
                         ("authoring_tests::", "gameplay_debugger_tests"),
                         ("zircon_plugin_ai_editor", "zircon_plugin_particles_editor"),
                         ("--lib", "--bins"), ("--no-default-features", "--all-features")]:
            argv = self.argv()
            argv[argv.index(old)] = new
            with self.subTest(old=old, new=new), self.assertRaises(entry.EntryError):
                entry.parse_cargo(argv, repo=ROOT, enforce_policy=False)


class GltfManifestTests(unittest.TestCase):
    FEATURES = ["runtime", "zircon_runtime/native-host", "zircon_runtime/animation"]
    LIB_FILTER = "source_snapshot_tests::importer_rejects_required_extensions_outside_its_stable_policy"

    def argv(self, *, lib=False):
        target = ["--lib", self.LIB_FILTER] if lib else ["--test", "character_skinning_sources"]
        return ["test", "--manifest-path", str(ROOT / "zircon_plugins/Cargo.toml"),
                "-p", "zircon_plugin_gltf_importer_runtime", "--no-default-features", "--locked",
                "--features", ",".join(self.FEATURES), "--profile", "test", "--jobs", "1",
                *target, "--", "--test-threads", "1"]

    def recipe(self, value):
        return PluginManifestTests().recipe(value)

    def test_both_exact_targets_reach_the_native_command_without_expanding_scope(self):
        for lib in (False, True):
            with self.subTest(lib=lib):
                parsed = entry.parse_cargo(self.argv(lib=lib), repo=ROOT)
                value = self.recipe(parsed)
                self.assertEqual(value["manifestPath"], "zircon_plugins/Cargo.toml")
                self.assertEqual(value["packages"], ["zircon_plugin_gltf_importer_runtime"])
                self.assertEqual(value["features"], self.FEATURES)
                self.assertEqual(value["profile"], "test")
                self.assertEqual(value["lib"], lib)
                self.assertEqual(value["tests"], [] if lib else ["character_skinning_sources"])
                self.assertEqual(value["testFilter"], self.LIB_FILTER if lib else None)
                command = local.prepare_command(value["command"], Path("E:/cargo-targets/zircon-local/manifest-test/target"))
                self.assertEqual(command[command.index("--manifest-path") + 1], "zircon_plugins/Cargo.toml")
                self.assertEqual(command[command.index("--profile") + 1], "test")
                self.assertEqual(command[command.index("--features") + 1], ",".join(self.FEATURES))
                self.assertEqual(command[-3:], ["--", "--test-threads", "1"])
                self.assertIn("--no-default-features", command)
                self.assertIn("--locked", command)
                self.assertFalse(set(command) & {"--workspace", "--all", "--tests", "--bins", "--all-features", "--all-targets"})

    def test_cli_and_receiver_reject_recipe_widening_independently(self):
        for lib in (False, True):
            original = entry.parse_cargo(self.argv(lib=lib), repo=ROOT)
            changes = [{"packages": ["zircon_plugin_ai_editor"]},
                       {"packages": ["zircon_plugin_particles_editor"]},
                       {"packages": ["zircon_plugin_gltf_importer_runtime", "zircon_editor"]},
                       {"profile": "dev"}, {"profile": "release"}, {"verb": "check"},
                       {"noDefaultFeatures": False}, {"locked": False}, {"features": []},
                       {"features": self.FEATURES[:-1]}, {"features": self.FEATURES + ["gpu"]},
                       {"features": self.FEATURES + ["runtime"]}, {"jobs": 2}, {"testThreads": 2},
                       {"noRun": True}, {"bins": True}, {"allTests": True}, {"allFeatures": True},
                       {"harnessFilters": ["other"]}, {"ignored": True}, {"exact": True}, {"nocapture": True},
                       {"tests": ["other"]}, {"testFilter": "source_snapshot_tests::"},
                       {"cwd": str(ROOT / "zircon_plugins")}, {"workingDirectory": str(ROOT)},
                       {"manifestPath": "zircon_plugins/gltf_importer/runtime/Cargo.toml"}]
            for change in changes:
                with self.subTest(lib=lib, change=change), self.assertRaises(ValueError):
                    self.recipe({**original, **change})
            argv = self.argv(lib=lib)
            for old, new in [("test", "build"), ("character_skinning_sources" if not lib else self.LIB_FILTER, "other"),
                             (",".join(self.FEATURES), "runtime"), ("--no-default-features", "--all-features")]:
                changed = argv.copy()
                changed[changed.index(old)] = new
                with self.subTest(lib=lib, old=old), self.assertRaises(entry.EntryError):
                    entry.parse_cargo(changed, repo=ROOT, enforce_policy=False)

    def test_glTF_target_shapes_cannot_be_mixed_or_broadened(self):
        for lib in (False, True):
            value = entry.parse_cargo(self.argv(lib=lib), repo=ROOT)
            for change in [{"lib": not lib}, {"lib": True, "tests": ["character_skinning_sources"]},
                           {"lib": False, "tests": []}, {"tests": ["character_skinning_sources", "other"]}]:
                with self.subTest(lib=lib, change=change), self.assertRaises(ValueError):
                    self.recipe({**value, **change})
        value = entry.parse_cargo(self.argv(), repo=ROOT)
        with self.assertRaises(ValueError):
            self.recipe({**value, "testFilter": self.LIB_FILTER})
        value = entry.parse_cargo(self.argv(lib=True), repo=ROOT)
        with self.assertRaises(ValueError):
            self.recipe({**value, "testFilter": None})

    def test_feature_order_is_semantic_but_duplicate_features_are_not_accepted(self):
        argv = self.argv()
        argv[argv.index(",".join(self.FEATURES))] = ",".join(reversed(self.FEATURES))
        self.assertEqual(set(self.recipe(entry.parse_cargo(argv, repo=ROOT))["features"]), set(self.FEATURES))

    def test_glTF_manifest_aliases_and_foreign_workspaces_still_fail(self):
        for manifest in ["../Cargo.toml", "./zircon_plugins/Cargo.toml", "zircon_plugins//Cargo.toml",
                         "zircon_plugins/../Cargo.toml", "C:/foreign/Cargo.toml",
                         "zircon_plugins/gltf_importer/runtime/Cargo.toml"]:
            argv = self.argv()
            argv[2] = manifest
            with self.subTest(manifest=manifest), self.assertRaises(entry.EntryError):
                entry.parse_cargo(argv, repo=ROOT)

    def test_gltf_targets_share_the_same_graph_pool_but_never_the_root_or_material_pool(self):
        actor = object.__new__(runner.Management)
        actor.repo_id, actor.payload = "repo", {}
        preparation = SimpleNamespace(preparation_key=lambda *args: runner.sha(runner.encoded(args)))
        gltf = [self.recipe(entry.parse_cargo(self.argv(lib=lib), repo=ROOT)) for lib in (False, True)]
        material = self.recipe(entry.parse_cargo(PluginManifestTests().argv()))
        root_recipe = {k: v for k, v in gltf[0].items() if k != "manifestPath"}
        with patch.object(runner, "plain", side_effect=lambda value: Path(value)), \
             patch.object(Path, "is_file", return_value=True), \
             patch.object(Path, "read_bytes", return_value=b"config"):
            pools = [actor.batch_pool([recipe], preparation, {}, {}, ROOT, {})[0]
                     for recipe in [*gltf, material, root_recipe]]
        self.assertEqual(pools[0], pools[1])
        self.assertNotEqual(pools[0], pools[2])
        self.assertNotEqual(pools[0], pools[3])


class PluginWorkspaceInputTests(unittest.TestCase):
    def setUp(self):
        base = Path(os.environ["ZR_COMMIT_TEST_ROOT"])
        self.folder = tempfile.TemporaryDirectory(prefix="plugin-inputs-", dir=base)
        self.addCleanup(self.folder.cleanup)
        self.root = Path(self.folder.name)
        self.write("Cargo.toml", '''[workspace]
members = ["zircon_editor", "zircon_runtime"]
[workspace.package]
version = "0.1.0"
''')
        self.write("zircon_plugins/Cargo.toml", '''[workspace]
members = ["material_editor/editor"]
[workspace.package]
version = "0.1.0"
[workspace.dependencies]
zircon_editor = { path = "../zircon_editor" }
zircon_runtime = { path = "../zircon_runtime", default-features = false, features = ["native-host"] }
''')
        self.write("zircon_plugins/material_editor/editor/Cargo.toml", '''[package]
name = "zircon_plugin_material_editor_editor"
version.workspace = true
[dependencies]
zircon_editor.workspace = true
zircon_runtime.workspace = true
''')
        self.write("zircon_editor/Cargo.toml", '''[package]
name = "zircon_editor"
version.workspace = true
[dependencies]
zircon_runtime = { path = "../zircon_runtime", default-features = false, features = ["target-editor-host"] }
''')
        self.write("zircon_runtime/Cargo.toml", '[package]\nname = "zircon_runtime"\nversion.workspace = true\n')
        self.write("zircon_plugins/Cargo.lock", '''version = 4
[[package]]
name = "zircon_plugin_material_editor_editor"
version = "0.1.0"
''')
        self.recipe = entry.parse_cargo(PluginManifestTests().argv())

    def write(self, name, value):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(value, encoding="utf-8")
        return path

    def capture(self):
        return {"sourceView": "checkout", "sourceRoot": str(self.root), "externalRoots": [],
                "source": {"files": {path.relative_to(self.root).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
                                     for path in self.root.rglob("*") if path.is_file()}}}

    def inputs(self, captured=None):
        return runner.plugin_workspace_inputs(self.root, self.recipe, self.capture() if captured is None else captured)

    def test_inherited_cross_workspace_edges_lock_and_explicit_config_absences_are_bound(self):
        value = self.inputs()
        self.assertEqual(value["selectedLockfile"], str(self.root / "zircon_plugins/Cargo.lock"))
        self.assertEqual(len(value["packageManifests"]), 3)
        editor = value["packageManifests"][str(self.root / "zircon_editor/Cargo.toml")]
        self.assertEqual(editor["workspaceManifest"], str(self.root / "Cargo.toml"))
        self.assertEqual({feature for edge in value["pathDependencyEdges"] for feature in edge["declaredFeatures"]},
                         {"native-host", "target-editor-host"})
        self.assertIn(str(self.root / "zircon_plugins/.cargo/config.toml"), value["files"])
        self.assertIsNone(value["files"][str(self.root / "zircon_plugins/.cargo/config.toml")])
        self.assertFalse(value["cargoResolutionPerformed"])

    def test_nested_lock_is_required_even_when_root_lock_exists(self):
        (self.root / "zircon_plugins/Cargo.lock").unlink()
        self.write("Cargo.lock", 'version = 4\n')
        with self.assertRaisesRegex(ValueError, "absent"):
            self.inputs()

    def test_selected_package_and_workspace_membership_must_match(self):
        path = self.root / "zircon_plugins/material_editor/editor/Cargo.toml"
        path.write_text(path.read_text().replace("zircon_plugin_material_editor_editor", "wrong_package"))
        with self.assertRaisesRegex(ValueError, "mismatch"):
            self.inputs()
        path.write_text(path.read_text().replace("wrong_package", "zircon_plugin_material_editor_editor"))
        path = self.root / "zircon_plugins/Cargo.toml"
        path.write_text(path.read_text().replace('members = ["material_editor/editor"]', 'members = []'))
        with self.assertRaisesRegex(ValueError, "not a member"):
            self.inputs()

    def test_discovery_rejects_changed_or_uncaptured_manifests(self):
        capture = self.capture()
        self.write("zircon_editor/Cargo.toml", '[package]\nname = "changed"\nversion = "0.1.0"\n')
        with self.assertRaisesRegex(ValueError, "captured source"):
            self.inputs(capture)
        capture = self.capture()
        del capture["source"]["files"]["zircon_editor/Cargo.toml"]
        with self.assertRaisesRegex(ValueError, "captured source"):
            self.inputs(capture)

    def test_path_declarations_cannot_escape_to_unregistered_source(self):
        self.write("zircon_editor/Cargo.toml", '''[package]
name = "zircon_editor"
version.workspace = true
[dependencies]
foreign = {path = "../../unregistered"}
''')
        with self.assertRaisesRegex(ValueError, "escapes"):
            self.inputs()

    def test_worktree_and_sealed_source_views_are_not_silently_reinterpreted(self):
        captured = self.capture()
        captured["sourceView"] = "worktree"
        with self.assertRaisesRegex(ValueError, "canonical checkout"):
            self.inputs(captured)

    def test_mixed_batch_cannot_widen_the_plugin_request_to_other_packages(self):
        actor = object.__new__(runner.Management)
        actor.repo = self.root
        actor.payload = {"sourceView": "checkout", "batch": {"recipes": [self.recipe,
            {"verb": "check", "packages": ["zircon_editor"]}]}}
        with self.assertRaisesRegex(ValueError, "one recipe per request"):
            actor.validation_source()

    def test_config_bytes_change_source_digest_but_are_ineffective_below_cwd(self):
        before = self.inputs()
        self.write("zircon_plugins/.cargo/config.toml", '[build]\njobs = 1\n')
        after = self.inputs()
        self.assertNotEqual(before["inputDigest"], after["inputDigest"])
        nested = next(item for item in after["cargoConfiguration"] if "zircon_plugins" in item["path"] and item["path"].endswith("config.toml"))
        self.assertFalse(nested["effectiveFromCanonicalCwd"])

    def test_native_guard_holds_nested_lock_and_cross_workspace_manifests(self):
        guard_module = runner.checkout_guard_helper()
        inputs = self.inputs()
        guard = guard_module.CheckoutReadGuard(self.root, files=inputs["files"], watch_roots=[self.root]).acquire()
        try:
            for name in ("zircon_plugins/Cargo.lock", "zircon_editor/Cargo.toml"):
                with self.subTest(name=name), self.assertRaises(PermissionError):
                    (self.root / name).write_bytes(b"attempted concurrent mutation")
            guard.assert_stable()
            finished = guard.finish()
            self.assertTrue(finished["complete"])
            self.assertTrue(finished["stable"])
        finally:
            guard.close()

    def test_native_guard_detects_new_nested_configuration(self):
        guard_module = runner.checkout_guard_helper()
        inputs = self.inputs()
        guard = guard_module.CheckoutReadGuard(self.root, files=inputs["files"], watch_roots=[self.root]).acquire()
        try:
            self.write("zircon_plugins/.cargo/config.toml", '[build]\njobs = 1\n')
            self.assertTrue(guard.wait_for_change(5))
            with self.assertRaises(guard_module.CheckoutGuardChanged):
                guard.assert_stable()
            finished = guard.finish()
            self.assertTrue(finished["complete"])
            self.assertFalse(finished["stable"])
        finally:
            guard.close()

    def select_gltf(self, *, lib=False):
        path = self.root / "zircon_plugins/Cargo.toml"
        path.write_text(path.read_text().replace('members = ["material_editor/editor"]',
                        'members = ["material_editor/editor", "gltf_importer/runtime"]'), encoding="utf-8")
        self.write("zircon_plugins/gltf_importer/runtime/Cargo.toml", '''[package]
name = "zircon_plugin_gltf_importer_runtime"
version.workspace = true
[features]
runtime = ["dep:zircon_runtime"]
[dependencies]
zircon_runtime = { workspace = true, optional = true }
''')
        path = self.root / "zircon_plugins/Cargo.lock"
        path.write_text(path.read_text() + '''[[package]]
name = "zircon_plugin_gltf_importer_runtime"
version = "0.1.0"
''', encoding="utf-8")
        self.recipe = entry.parse_cargo(GltfManifestTests().argv(lib=lib), repo=ROOT)

    def select_ai(self):
        path = self.root / "zircon_plugins/Cargo.toml"
        path.write_text(path.read_text().replace('members = ["material_editor/editor"]',
                        'members = ["material_editor/editor", "ai/editor", "ai/runtime"]'), encoding="utf-8")
        self.write("zircon_plugins/ai/editor/Cargo.toml", '''[package]
name = "zircon_plugin_ai_editor"
version.workspace = true
[dependencies]
zircon_editor.workspace = true
zircon_runtime.workspace = true
zircon_plugin_ai_runtime = { path = "../runtime" }
''')
        self.write("zircon_plugins/ai/runtime/Cargo.toml", '''[package]
name = "zircon_plugin_ai_runtime"
version.workspace = true
[dependencies]
zircon_runtime.workspace = true
''')
        path = self.root / "zircon_plugins/Cargo.lock"
        path.write_text(path.read_text() + '''[[package]]
name = "zircon_plugin_ai_editor"
version = "0.1.0"
[[package]]
name = "zircon_plugin_ai_runtime"
version = "0.1.0"
''', encoding="utf-8")
        self.recipe = entry.parse_cargo(AiManifestTests().argv(), repo=ROOT)

    def test_ai_closure_recurses_through_runtime_and_binds_both_workspace_host_inputs(self):
        self.select_ai()
        value = self.inputs()
        self.assertEqual(value["selectedPackage"], "zircon_plugin_ai_editor")
        self.assertEqual(value["selectedLockfile"], str(self.root / "zircon_plugins/Cargo.lock"))
        self.assertEqual({p["name"] for p in value["packageManifests"].values()},
                         {"zircon_plugin_ai_editor", "zircon_plugin_ai_runtime", "zircon_editor", "zircon_runtime"})
        self.assertEqual(len(value["workspaceManifests"]), 2)
        self.assertEqual({f for edge in value["pathDependencyEdges"] for f in edge["declaredFeatures"]},
                         {"native-host", "target-editor-host"})
        self.assertIsNone(value["files"][str(self.root / "zircon_plugins/.cargo/config.toml")])
        self.assertFalse(value["cargoResolutionPerformed"])
        for selected in AiManifestTests.filters:
            self.recipe = entry.parse_cargo(AiManifestTests().argv(selected), repo=ROOT)
            self.assertEqual(self.inputs()["inputDigest"], value["inputDigest"])

    def test_ai_member_and_selected_lock_cannot_fall_back_to_another_plugin(self):
        self.select_ai()
        path = self.root / "zircon_plugins/Cargo.toml"
        original = path.read_text()
        path.write_text(original.replace(', "ai/editor"', ''), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "not a member"):
            self.inputs()
        path.write_text(original, encoding="utf-8")
        path = self.root / "zircon_plugins/Cargo.lock"
        path.write_text(path.read_text().replace('name = "zircon_plugin_ai_editor"', 'name = "other"'), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "absent from the plugin lockfile"):
            self.inputs()

    def test_ai_native_and_editor_host_edges_are_each_required(self):
        self.select_ai()
        for relative, feature in [("zircon_plugins/Cargo.toml", "native-host"),
                                  ("zircon_editor/Cargo.toml", "target-editor-host")]:
            path = self.root / relative
            original = path.read_text()
            path.write_text(original.replace('features = ["' + feature + '"]', 'features = []'), encoding="utf-8")
            with self.subTest(feature=feature), self.assertRaisesRegex(ValueError, "native/editor"):
                self.inputs()
            path.write_text(original, encoding="utf-8")

    def test_ai_uncaptured_transitive_manifest_or_added_config_invalidates_source_guard(self):
        self.select_ai()
        path = self.root / "zircon_plugins/ai/runtime/Cargo.toml"
        original = path.read_text()
        captured = self.capture()
        self.write("zircon_plugins/ai/runtime/Cargo.toml", original + '\n[features]\nextra = []\n')
        with self.assertRaisesRegex(ValueError, "captured source"):
            self.inputs(captured)
        path.write_text(original, encoding="utf-8")
        # Config inputs have their own digest/absence binding and native watcher,
        # distinct from the engine source inventory's manifest coverage.
        inputs = self.inputs()
        helper = runner.checkout_guard_helper()
        guard = helper.CheckoutReadGuard(self.root, files=inputs["files"], watch_roots=[self.root]).acquire()
        try:
            self.write("zircon_plugins/.cargo/config.toml", '[build]\njobs = 1\n')
            self.assertTrue(guard.wait_for_change(5))
            with self.assertRaises(helper.CheckoutGuardChanged):
                guard.assert_stable()
            self.assertFalse(guard.finish()["stable"])
        finally:
            guard.close()
        self.assertNotEqual(self.inputs()["inputDigest"], inputs["inputDigest"])

    def select_particles(self):
        path = self.root / "zircon_plugins/Cargo.toml"
        path.write_text(path.read_text().replace('members = ["material_editor/editor"]',
                        'members = ["material_editor/editor", "particles/editor"]'), encoding="utf-8")
        self.write("zircon_plugins/particles/editor/Cargo.toml", '''[package]
name = "zircon_plugin_particles_editor"
version.workspace = true
[dependencies]
zircon_editor.workspace = true
zircon_runtime.workspace = true
''')
        path = self.root / "zircon_plugins/Cargo.lock"
        path.write_text(path.read_text() + '''[[package]]
name = "zircon_plugin_particles_editor"
version = "0.1.0"
''', encoding="utf-8")
        self.recipe = entry.parse_cargo(ParticlesManifestTests().argv(), repo=ROOT)

    def test_particles_closure_binds_its_member_nested_lock_and_both_workspace_host_edges(self):
        self.select_particles()
        value = self.inputs()
        self.assertEqual(value["selectedPackage"], "zircon_plugin_particles_editor")
        self.assertEqual(value["selectedLockfile"], str(self.root / "zircon_plugins/Cargo.lock"))
        self.assertEqual({p["name"] for p in value["packageManifests"].values()},
                         {"zircon_plugin_particles_editor", "zircon_editor", "zircon_runtime"})
        self.assertEqual(len(value["workspaceManifests"]), 2)
        self.assertEqual({f for edge in value["pathDependencyEdges"] for f in edge["declaredFeatures"]},
                         {"native-host", "target-editor-host"})
        self.assertIsNone(value["files"][str(self.root / "zircon_plugins/.cargo/config.toml")])
        self.assertFalse(value["cargoResolutionPerformed"])
        self.assertEqual(value["inputDigest"], runner.sha(runner.encoded({k: v for k, v in value.items()
                                                                        if k != "inputDigest"})))

    def test_particles_member_name_membership_and_lock_cannot_fall_back_to_ai_or_material(self):
        self.select_particles()
        for relative, old, new, message in [
                ("zircon_plugins/particles/editor/Cargo.toml", "zircon_plugin_particles_editor", "zircon_plugin_ai_editor", "mismatch"),
                ("zircon_plugins/Cargo.toml", ', "particles/editor"', '', "not a member"),
                ("zircon_plugins/Cargo.lock", "zircon_plugin_particles_editor", "other", "absent from the plugin lockfile")]:
            path = self.root / relative
            original = path.read_text()
            path.write_text(original.replace(old, new), encoding="utf-8")
            with self.subTest(relative=relative), self.assertRaisesRegex(ValueError, message):
                self.inputs()
            path.write_text(original, encoding="utf-8")

    def test_particles_native_and_editor_host_edges_are_independently_required(self):
        self.select_particles()
        for relative, feature in [("zircon_plugins/Cargo.toml", "native-host"),
                                  ("zircon_editor/Cargo.toml", "target-editor-host")]:
            path = self.root / relative
            original = path.read_text()
            path.write_text(original.replace('features = ["' + feature + '"]', 'features = []'), encoding="utf-8")
            with self.subTest(feature=feature), self.assertRaisesRegex(ValueError, "native/editor"):
                self.inputs()
            path.write_text(original, encoding="utf-8")

    def test_particles_cross_workspace_change_and_added_config_invalidate_the_read_guard(self):
        self.select_particles()
        captured = self.capture()
        original = (self.root / "zircon_editor/Cargo.toml").read_text()
        self.write("zircon_editor/Cargo.toml", '[package]\nname = "foreign_change"\nversion = "0.1.0"\n')
        with self.assertRaisesRegex(ValueError, "captured source"):
            self.inputs(captured)
        # Restore the foreign dependency before checking independently watched config absence.
        self.write("zircon_editor/Cargo.toml", original)
        inputs = self.inputs()
        helper = runner.checkout_guard_helper()
        guard = helper.CheckoutReadGuard(self.root, files=inputs["files"], watch_roots=[self.root]).acquire()
        try:
            self.write("zircon_plugins/.cargo/config.toml", '[build]\njobs = 1\n')
            self.assertTrue(guard.wait_for_change(5))
            with self.assertRaises(helper.CheckoutGuardChanged):
                guard.assert_stable()
            self.assertFalse(guard.finish()["stable"])
        finally:
            guard.close()
        self.assertNotEqual(self.inputs()["inputDigest"], inputs["inputDigest"])

    def select_prefab(self):
        path = self.root / "zircon_plugins/Cargo.toml"
        path.write_text(path.read_text().replace('members = ["material_editor/editor"]',
                        'members = ["material_editor/editor", "prefab_tools/editor"]'), encoding="utf-8")
        self.write("zircon_plugins/prefab_tools/editor/Cargo.toml", '''[package]
name = "zircon_plugin_prefab_tools_editor"
version.workspace = true
[dependencies]
zircon_editor.workspace = true
zircon_runtime.workspace = true
''')
        path = self.root / "zircon_plugins/Cargo.lock"
        path.write_text(path.read_text() + '''[[package]]
name = "zircon_plugin_prefab_tools_editor"
version = "0.1.0"
''', encoding="utf-8")
        self.recipe = entry.parse_cargo(PrefabManifestTests().argv(), repo=ROOT)

    def test_prefab_source_closure_selects_its_own_member_and_both_runtime_host_edges(self):
        self.select_prefab()
        value = self.inputs()
        self.assertEqual(value["selectedPackage"], "zircon_plugin_prefab_tools_editor")
        self.assertEqual(value["selectedLockfile"], str(self.root / "zircon_plugins/Cargo.lock"))
        self.assertEqual({p["name"] for p in value["packageManifests"].values()},
                         {"zircon_plugin_prefab_tools_editor", "zircon_editor", "zircon_runtime"})
        self.assertEqual(len(value["workspaceManifests"]), 2)
        self.assertEqual({f for edge in value["pathDependencyEdges"] for f in edge["declaredFeatures"]},
                         {"native-host", "target-editor-host"})
        self.assertIn(str(self.root / "zircon_plugins/prefab_tools/editor/Cargo.toml"), value["files"])
        self.assertIsNone(value["files"][str(self.root / "zircon_plugins/.cargo/config.toml")])
        self.assertFalse(value["cargoResolutionPerformed"])
        captured = self.capture()
        self.write("zircon_editor/Cargo.toml", '[package]\nname = "foreign_change"\nversion = "0.1.0"\n')
        with self.assertRaisesRegex(ValueError, "captured source"):
            self.inputs(captured)

    def test_prefab_source_member_and_lock_never_fall_back_to_material(self):
        self.select_prefab()
        manifest = self.root / "zircon_plugins/Cargo.toml"
        original = manifest.read_text()
        manifest.write_text(original.replace(', "prefab_tools/editor"', ''), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "not a member"):
            self.inputs()
        manifest.write_text(original, encoding="utf-8")
        lock = self.root / "zircon_plugins/Cargo.lock"
        lock.write_text(lock.read_text().replace("zircon_plugin_prefab_tools_editor", "other"), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "absent from the plugin lockfile"):
            self.inputs()

    def test_prefab_source_guard_requires_native_and_editor_host_edges_independently(self):
        self.select_prefab()
        for relative, feature in [("zircon_plugins/Cargo.toml", "native-host"),
                                  ("zircon_editor/Cargo.toml", "target-editor-host")]:
            path = self.root / relative
            original = path.read_text()
            path.write_text(original.replace('features = ["' + feature + '"]', 'features = []'), encoding="utf-8")
            with self.subTest(feature=feature), self.assertRaisesRegex(ValueError, "native/editor"):
                self.inputs()
            path.write_text(original, encoding="utf-8")

    def test_gltf_closure_starts_at_selected_member_and_binds_cross_workspace_inputs(self):
        self.select_gltf()
        value = self.inputs()
        self.assertEqual(value["selectedPackage"], "zircon_plugin_gltf_importer_runtime")
        self.assertEqual(value["selectedLockfile"], str(self.root / "zircon_plugins/Cargo.lock"))
        self.assertEqual({p["name"] for p in value["packageManifests"].values()},
                         {"zircon_plugin_gltf_importer_runtime", "zircon_runtime"})
        self.assertEqual(len(value["workspaceManifests"]), 2)
        self.assertTrue(any(e["fromPackage"] == value["selectedPackage"] and e["declaredFeatures"] == ["native-host"]
                            for e in value["pathDependencyEdges"]))
        self.assertIsNone(value["files"][str(self.root / "zircon_plugins/.cargo/config.toml")])
        self.assertFalse(value["cargoResolutionPerformed"])
        self.assertEqual(value["inputDigest"], runner.sha(runner.encoded({k: v for k, v in value.items()
                                                                        if k != "inputDigest"})))

    def test_gltf_selected_member_name_membership_and_lock_cannot_fall_back_to_material(self):
        self.select_gltf(lib=True)
        path = self.root / "zircon_plugins/gltf_importer/runtime/Cargo.toml"
        original = path.read_text()
        path.write_text(original.replace("zircon_plugin_gltf_importer_runtime", "wrong_package"), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "mismatch"):
            self.inputs()
        path.write_text(original, encoding="utf-8")
        lock = self.root / "zircon_plugins/Cargo.lock"
        lock_original = lock.read_text()
        lock.write_text(lock_original.replace("zircon_plugin_gltf_importer_runtime", "other"), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "absent from the plugin lockfile"):
            self.inputs()
        lock.write_text(lock_original, encoding="utf-8")
        path = self.root / "zircon_plugins/Cargo.toml"
        path.write_text(path.read_text().replace(', "gltf_importer/runtime"', ''), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "not a member"):
            self.inputs()

    def test_gltf_source_guard_rejects_uncaptured_or_changed_cross_workspace_manifest(self):
        self.select_gltf()
        captured = self.capture()
        runtime = self.root / "zircon_runtime/Cargo.toml"
        runtime.write_text(runtime.read_text() + '\n[features]\nanimation = []\n', encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "captured source"):
            self.inputs(captured)
        captured = self.capture()
        del captured["source"]["files"]["zircon_runtime/Cargo.toml"]
        with self.assertRaisesRegex(ValueError, "captured source"):
            self.inputs(captured)

    def test_gltf_runtime_edge_cannot_silently_lose_inherited_native_feature(self):
        self.select_gltf()
        path = self.root / "zircon_plugins/Cargo.toml"
        path.write_text(path.read_text().replace('features = ["native-host"]', 'features = []'), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "native runtime feature edge"):
            self.inputs()

    def test_gltf_native_guard_keeps_its_lock_and_runtime_manifest_until_finish(self):
        self.select_gltf()
        value = self.inputs()
        helper = runner.checkout_guard_helper()
        guard = helper.CheckoutReadGuard(self.root, files=value["files"], watch_roots=[self.root]).acquire()
        try:
            for relative in ("zircon_plugins/Cargo.lock", "zircon_runtime/Cargo.toml",
                             "zircon_plugins/gltf_importer/runtime/Cargo.toml"):
                with self.subTest(relative=relative), self.assertRaises(PermissionError):
                    (self.root / relative).write_bytes(b"concurrent mutation")
            guard.assert_stable()
            finished = guard.finish()
            self.assertTrue(finished["complete"] and finished["stable"])
        finally:
            guard.close()

if __name__ == "__main__":
    unittest.main()
