"""Agent-local policy/preview/receipt tests. All Cargo execution is mocked.

Temporary source/output fixtures are unit boundaries, not physical-storage,
compiler-provenance, process-tree, or product acceptance evidence.
"""
from __future__ import annotations

import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from tools.dev import agent_cargo_policy as policy
from tools.dev import local_cargo


class AgentLocalCargoTests(unittest.TestCase):
    @contextlib.contextmanager
    def source(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory).resolve()
            repo = base / "repo"
            repo.mkdir()
            (repo / "Cargo.toml").write_text('[workspace]\nmembers = ["selected"]\n', encoding="utf-8")
            (repo / "selected").mkdir()
            (repo / "selected/Cargo.toml").write_text('[package]\nname = "selected_package"\nversion = "0.1.0"\n', encoding="utf-8")
            plugins = repo / "zircon_plugins"
            plugins.mkdir()
            (plugins / "Cargo.toml").write_text('[workspace]\nmembers = ["plugin"]\n', encoding="utf-8")
            (plugins / "plugin").mkdir()
            (plugins / "plugin/Cargo.toml").write_text('[package]\nname = "selected_plugin"\nversion = "0.1.0"\n', encoding="utf-8")
            yield repo, base / "cargo-targets"

    @staticmethod
    def argv(verb="check", package="selected_package"):
        return [verb, "-p", package, "--no-default-features"]

    @contextlib.contextmanager
    def mock_storage(self, root):
        # Do not replace the separate real physical-root contract tests.
        with patch.object(local_cargo, "storage_roots", return_value=(root,)), \
                patch.object(local_cargo, "validate_output_path", side_effect=Path), \
                patch.object(local_cargo, "_evidence_output", side_effect=Path):
            yield

    def preview(self, repo, root, mode="--dry-run", extra=(), cargo=None, default_target=False):
        out = io.StringIO()
        target_option = [] if default_target else ["--target-dir", str(root / "zircon-local/unit/target")]
        with self.mock_storage(root), patch.object(Path, "mkdir", side_effect=AssertionError("preview wrote directory")), \
                patch.object(local_cargo.subprocess, "run", side_effect=AssertionError("preview started Cargo")), \
                patch.object(local_cargo, "_run_owned_command", side_effect=AssertionError("preview started native Job")), \
                patch.object(local_cargo, "free_bytes", side_effect=AssertionError("preview did execution preflight")), \
                patch.dict(sys.modules, {"tools.jenkins.frontend": None}), contextlib.redirect_stdout(out):
            code = local_cargo.main(["--repo-root", str(repo), *target_option,
                                     mode, *extra, "--", *(cargo or self.argv())])
        return code, json.loads(out.getvalue())

    def test_preview_binds_actual_exact_package_and_both_manifests(self):
        with self.source() as (repo, root):
            code, plan = self.preview(repo, root)
            self.assertEqual(0, code, plan)
            self.assertEqual("Agent", plan["executionOwner"])
            self.assertEqual("single-exact-package", plan["selectionScope"])
            self.assertEqual(str(repo / "selected/Cargo.toml"), plan["packageManifest"])
            self.assertEqual(str(repo / "Cargo.toml"), plan["workspaceManifest"])
            self.assertEqual(str(repo), plan["cwd"])
            self.assertIn("--locked", plan["cargoArguments"])
            self.assertIn("--no-default-features", plan["cargoArguments"])
            self.assertFalse(plan["commandExecuted"])
            self.assertFalse(plan["formalAcceptance"])
            self.assertNotIn("receipt", plan)
            self.assertFalse(root.exists())

    def test_query_and_deprecated_independent_preview_never_execute(self):
        with self.source() as (repo, root):
            for mode in ("--query-only", "--independent-preview"):
                with self.subTest(mode=mode):
                    code, plan = self.preview(repo, root, mode)
                    self.assertEqual(0, code, plan)
                    self.assertFalse(plan["commandExecuted"])
                    if mode == "--query-only":
                        self.assertTrue(plan["queryOnly"])
                    else:
                        self.assertIn("read-only", plan["deprecatedOptions"][0])

    def test_plugin_package_uses_real_plugin_workspace_without_special_case(self):
        with self.source() as (repo, root):
            code, plan = self.preview(repo, root, cargo=self.argv(package="selected_plugin"))
            self.assertEqual(0, code, plan)
            manifest = str(repo / "zircon_plugins/Cargo.toml")
            self.assertEqual(manifest, plan["workspaceManifest"])
            args = plan["cargoArguments"]
            self.assertEqual(manifest, args[args.index("--manifest-path") + 1])

    def test_wrong_unknown_pattern_and_repeated_package_are_rejected(self):
        with self.source() as (repo, _root):
            bad = [self.argv(package="selected*"), self.argv(package="unknown"),
                   self.argv() + ["-p", "selected_plugin"],
                   self.argv() + ["--manifest-path", "zircon_plugins/Cargo.toml"],
                   ["check", "--no-default-features"], ["check", "-p", "selected_package"]]
            for args in bad:
                with self.subTest(args=args), self.assertRaises(ValueError):
                    policy.admit_cargo(args, repo)

    def test_real_package_manifest_can_be_selected_but_foreign_manifest_cannot(self):
        with self.source() as (repo, _root):
            manifest = repo / "selected/Cargo.toml"
            selected = policy.admit_cargo(self.argv() + ["--manifest-path", str(manifest)], repo)
            self.assertEqual(str(manifest), selected["manifestPath"])
            for value in ("../repo/selected/Cargo.toml", str(repo / "missing/Cargo.toml"), "zircon_plugins/plugin/Cargo.toml"):
                with self.subTest(value=value), self.assertRaises(ValueError):
                    policy.admit_cargo(self.argv() + ["--manifest-path", value], repo)

    def test_duplicate_owner_in_another_workspace_is_ambiguous(self):
        with self.source() as (repo, _root):
            (repo / "zircon_plugins/plugin/Cargo.toml").write_text('[package]\nname = "selected_package"\n', encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "Ambiguous"):
                policy.admit_cargo(self.argv(), repo)

    def test_scope_and_output_overrides_are_fail_closed(self):
        with self.source() as (repo, _root):
            for extra in (["--workspace"], ["--all"], ["--all-features"], ["--all-targets"], ["--tests"],
                          ["--config", "build.target-dir='C:/escape'"], ["--config=build.build-dir='C:/escape'"],
                          ["--target-dir=C:/escape"], ["--out-dir", "C:/escape"],
                          ["--artifact-dir", "C:/escape"], ["--target", "custom.json"]):
                with self.subTest(extra=extra), self.assertRaises(ValueError):
                    policy.admit_cargo(self.argv() + extra, repo)

    def test_test_scope_and_exact_harness_survive_admission(self):
        with self.source() as (repo, _root):
            args = self.argv("test") + ["--lib", "focused_filter", "--", "--exact", "--test-threads", "1"]
            selected = policy.admit_cargo(args, repo)
            result = selected["cargoArguments"]
            self.assertEqual(args[args.index("--"):], result[result.index("--"):])
            self.assertEqual("focused_filter", selected["testFilter"])
            for invalid in (self.argv("test"), self.argv("test") + ["--lib", "--test", "other"],
                            self.argv("test") + ["--lib", "--no-run", "--", "--exact"]):
                with self.assertRaises(ValueError):
                    policy.admit_cargo(invalid, repo)

    def test_retired_identity_flags_reject_even_empty_before_any_source_read(self):
        for flag in ("--jenkins-identity-file", "-JenkinsIdentityFile", "--identity-file", "--jenkins-identity-file="):
            with self.subTest(flag=flag), patch.object(Path, "read_text", side_effect=AssertionError("identity/source read")), \
                    contextlib.redirect_stdout(io.StringIO()) as out:
                self.assertEqual(2, local_cargo.main([flag, "", "--dry-run", "--", *self.argv()]))
                self.assertIn("retired", json.loads(out.getvalue())["message"])

    def test_ambient_jenkins_flags_never_affect_preview_or_child_environment(self):
        inherited = {"ZIRCON_JENKINS_IDENTITY_FILE": "not-read", "JENKINS_URL": "not-contacted", "JENKINS_HOME": "not-read",
                     "RUSTFLAGS": "-Cincremental=C:/escape", "CARGO_PROFILE_DEV_DEBUG": "2", "CARGO_BUILD_BUILD_DIR": "C:/escape"}
        with self.source() as (repo, root), patch.dict(os.environ, inherited):
            code, plan = self.preview(repo, root)
            self.assertEqual(0, code, plan)
            with self.mock_storage(root):
                environment, directories = local_cargo.local_environment(root / "zircon-local/unit/target")
            self.assertTrue(all(path.is_relative_to(root) for path in directories))
            self.assertNotIn("RUSTFLAGS", environment)
            self.assertFalse(any("JENKINS" in key for key in environment))
            self.assertEqual("0", environment["CARGO_PROFILE_DEV_DEBUG"])
            self.assertEqual("false", environment["CARGO_PROFILE_TEST_INCREMENTAL"])
            self.assertNotEqual(environment["CARGO_TARGET_DIR"], environment["CARGO_BUILD_BUILD_DIR"])

    def test_collapsed_output_directories_and_existing_config_are_rejected(self):
        with self.source() as (repo, root), self.mock_storage(root):
            with self.assertRaisesRegex(ValueError, "distinct"):
                local_cargo.local_environment(root / "zircon-local/unit/build")
            environment, _ = local_cargo.local_environment(root / "zircon-local/unit/target")
            (repo / ".cargo").mkdir()
            (repo / ".cargo/config.toml").write_text("must not be interpreted", encoding="utf-8")
            with patch.object(Path, "read_text", side_effect=AssertionError("configuration content was read")):
                with self.assertRaisesRegex(ValueError, "configuration"):
                    local_cargo._configuration_preflight(repo, environment)

    def test_normalized_json_still_meets_literal_native_artifact_contract(self):
        with self.source() as (repo, root), self.mock_storage(root):
            selected = policy.admit_cargo(self.argv("test") + ["--lib", "--no-run", "--message-format=json"], repo)
            command = local_cargo.prepare_command(selected["cargoArguments"], root / "zircon-local/unit/target")
            local_cargo._validate_artifact_evidence(command, 10)
            self.assertEqual(1, command.count("--message-format=json"))
            for invalid in (["cargo", "check", "--message-format=json"],
                            ["cargo", "test", "--lib", "--message-format=json"],
                            ["cargo", "test", "--no-run", "--message-format=json", "--", "--exact"]):
                with self.assertRaises(ValueError):
                    local_cargo._validate_artifact_evidence(invalid, 10)

    def test_native_evidence_parameters_require_complete_triad_before_execution(self):
        with self.source() as (repo, root):
            code, result = self.preview(repo, root, extra=["--evidence-directory", str(root / "zircon-local/evidence")])
            self.assertEqual(2, code)
            self.assertIn("together", result["message"])

    def test_default_main_executes_local_boundary_and_propagates_real_nonzero_code(self):
        with self.source() as (repo, root), self.mock_storage(root), \
                patch.object(local_cargo, "free_bytes", return_value=local_cargo.MIN_FREE_BYTES + 1), \
                patch.object(local_cargo.shutil, "which", return_value=str(root / "installed-cargo")), \
                patch.object(Path, "mkdir") as mkdir, \
                patch.object(local_cargo, "_ordinary_receipt", return_value=7) as run, \
                patch.object(local_cargo, "_run_with_evidence", side_effect=AssertionError("native artifact mode selected")), \
                patch.dict(sys.modules, {"tools.jenkins.frontend": None}):
            code = local_cargo.main(["--repo-root", str(repo), "--target-dir", str(root / "zircon-local/unit/target"),
                                     "--", *self.argv()])
            self.assertEqual(7, code)
            self.assertGreater(mkdir.call_count, 0)
            run.assert_called_once()
            command, cwd, _target, environment, _output, owner = run.call_args.args
            self.assertEqual(repo, cwd)
            self.assertEqual("selected_package", owner["package"])
            self.assertEqual(str(repo / "selected/Cargo.toml"), owner["packageManifest"])
            self.assertIn("--no-default-features", command)
            self.assertEqual("0", environment["CARGO_INCREMENTAL"])

    def test_low_capacity_fails_before_mkdir_and_native_lookup(self):
        with self.source() as (repo, root), self.mock_storage(root), \
                patch.object(local_cargo, "free_bytes", return_value=1) as capacity, \
                patch.object(local_cargo.shutil, "which", side_effect=AssertionError("Cargo lookup before capacity")), \
                patch.object(Path, "mkdir", side_effect=AssertionError("output created on low disk")), \
                contextlib.redirect_stdout(io.StringIO()) as out:
            code = local_cargo.main(["--repo-root", str(repo), "--target-dir", str(root / "zircon-local/unit/target"),
                                     "--", *self.argv()])
            self.assertEqual(2, code)
            capacity.assert_called_once()
            self.assertIn("35 GiB", json.loads(out.getvalue())["message"])

    @staticmethod
    def simple_fact(path):
        data = path.read_bytes()
        return {"path": str(path), "sha256": hashlib.sha256(data).hexdigest(), "byteLength": len(data)}

    def test_ordinary_receipt_records_actual_root_exit_logs_command_and_owner(self):
        with self.source() as (repo, root), self.mock_storage(root):
            target = root / "zircon-local/unit/target"
            output = root / "zircon-local/receipts/owned-fixture"
            selected = policy.admit_cargo(self.argv(), repo)
            command = ["installed-cargo", "check", "-p", "selected_package", "--no-default-features"]
            def actual_boundary(args, **kwargs):
                self.assertEqual(command, args)
                self.assertEqual(repo, kwargs["cwd"])
                kwargs["stdout"].write(b"actual root output\n")
                kwargs["stderr"].write(b"actual root error\n")
                return subprocess.CompletedProcess(args, 7)
            with patch.object(local_cargo.subprocess, "run", side_effect=actual_boundary) as run, \
                    patch.object(local_cargo, "_file_fact", side_effect=self.simple_fact), \
                    contextlib.redirect_stdout(io.StringIO()) as out, contextlib.redirect_stderr(io.StringIO()):
                code = local_cargo._ordinary_receipt(command, repo, target, {"CARGO_BUILD_BUILD_DIR": str(target.parent / "build")}, output, selected)
            self.assertEqual(7, code)
            run.assert_called_once()
            receipt = json.loads((output / "receipt.json").read_text(encoding="utf-8"))
            self.assertEqual(7, receipt["exitCode"])
            self.assertEqual(command, receipt["command"])
            self.assertEqual(selected, receipt["owner"])
            self.assertEqual("command-failed", receipt["status"])
            self.assertTrue(receipt["commandExecuted"])
            self.assertFalse(receipt["processTreeVerified"])
            self.assertFalse(receipt["compilerArtifactEvidence"])
            self.assertFalse(receipt["formalAcceptance"])
            self.assertEqual(hashlib.sha256(b"actual root output\n").hexdigest(), receipt["stdout"]["sha256"])
            self.assertEqual(7, json.loads(out.getvalue().splitlines()[-1])["exitCode"])
            with self.assertRaises(FileExistsError):
                local_cargo._ordinary_receipt(command, repo, target, {}, output, selected)

    @staticmethod
    def linked_sources(repo):
        """Only .git/commondir fixtures; never run Git or touch actual Git data."""
        peer = repo.parent / "linked-peer"
        peer.mkdir()
        (peer / "Cargo.toml").write_text('[workspace]\nmembers = ["selected"]\n', encoding="utf-8")
        (peer / "selected").mkdir()
        (peer / "selected/Cargo.toml").write_text('[package]\nname = "selected_package"\nversion = "0.1.0"\n', encoding="utf-8")
        common = repo.parent / "fixture-git-common"
        for source, name in ((repo, "one"), (peer, "two")):
            gitdir = common / "worktrees" / name
            gitdir.mkdir(parents=True)
            relative = os.path.relpath(gitdir, source).replace(os.sep, "/")
            (source / ".git").write_text("gitdir: " + relative + "\n", encoding="utf-8")
            (gitdir / "commondir").write_text("../..\n", encoding="utf-8")
        return peer, common

    def test_two_linked_worktrees_share_default_target_and_cargo_home_in_readonly_previews(self):
        with self.source() as (repo, root):
            peer, common = self.linked_sources(repo)
            first_code, first = self.preview(repo, root, default_target=True)
            second_code, second = self.preview(peer, root, mode="--query-only", default_target=True)
            self.assertEqual(0, first_code, first)
            self.assertEqual(0, second_code, second)
            self.assertEqual(first["targetDirectory"], second["targetDirectory"])
            self.assertEqual(first["outputEnvironment"]["CARGO_HOME"], second["outputEnvironment"]["CARGO_HOME"])
            self.assertNotEqual(first["cwd"], second["cwd"])
            self.assertFalse(root.exists())
            identity = local_cargo.repository_cache_identity(repo)
            self.assertEqual(identity, local_cargo.repository_cache_identity(peer))
            self.assertEqual(os.path.normcase(str(common)), identity["path"])
            self.assertEqual("git-common-directory", identity["kind"])

    def test_distinct_git_common_directories_do_not_share_a_pool(self):
        with self.source() as (repo, root), self.mock_storage(root):
            peer, _common = self.linked_sources(repo)
            original = local_cargo._default_target(repo, self.argv(), dry_run=True)
            other_common = repo.parent / "another-git-common"
            other_gitdir = other_common / "worktrees/peer"
            other_gitdir.mkdir(parents=True)
            (other_gitdir / "commondir").write_text("../..\n", encoding="utf-8")
            (peer / ".git").write_text("gitdir: " + str(other_gitdir) + "\n", encoding="utf-8")
            self.assertNotEqual(original, local_cargo._default_target(peer, self.argv(), dry_run=True))

    def test_same_git_pool_is_separated_for_incompatible_toolchain_input(self):
        with self.source() as (repo, root), self.mock_storage(root):
            peer, _common = self.linked_sources(repo)
            for source in (repo, peer):
                (source / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "stable"\n', encoding="utf-8")
            original = local_cargo._default_target(repo, self.argv(), dry_run=True)
            self.assertEqual(original, local_cargo._default_target(peer, self.argv(), dry_run=False))
            (peer / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "1.94.1"\n', encoding="utf-8")
            self.assertNotEqual(original, local_cargo._default_target(peer, self.argv(), dry_run=True))

    def test_git_cache_identity_rejects_alias_and_broken_metadata_without_fallback(self):
        with self.source() as (repo, _root):
            _peer, common = self.linked_sources(repo)
            with patch.object(Path, "is_symlink", autospec=True, side_effect=lambda path: path == common):
                with self.assertRaisesRegex(ValueError, "alias"):
                    local_cargo.repository_cache_identity(repo)
            (repo / ".git").write_text("invalid control pointer\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "pointer"):
                local_cargo.repository_cache_identity(repo)
            (repo / ".git").write_text("gitdir: missing-directory\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "missing"):
                local_cargo.repository_cache_identity(repo)

    def test_default_target_reuses_package_feature_cache_but_separates_non_git_sources(self):
        with self.source() as (repo, root), self.mock_storage(root):
            first = local_cargo._default_target(repo, self.argv(), dry_run=True)
            second = local_cargo._default_target(repo, self.argv(package="selected_plugin") + ["--features", "one"], dry_run=False)
            self.assertEqual(first, second)
            self.assertEqual("source-root-without-git", local_cargo.repository_cache_identity(repo)["kind"])
            self.assertNotEqual(first, local_cargo._default_target(repo.parent / "other", self.argv(), dry_run=True))


if __name__ == "__main__":
    unittest.main()
