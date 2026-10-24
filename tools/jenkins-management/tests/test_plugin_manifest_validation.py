"""Bounded nested-workspace admission and cache identity; no compiler runs."""
from __future__ import annotations
import importlib.util
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

if __name__ == "__main__":
    unittest.main()
