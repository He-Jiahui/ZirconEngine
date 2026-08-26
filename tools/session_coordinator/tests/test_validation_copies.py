from __future__ import annotations

import subprocess
import tempfile
import threading
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.validation_copies import (
    CargoInputClosurePlanner,
    ExternalGitSource,
)
from tools.session_coordinator.workspace_copy import WorkspaceCopyService


class ValidationCopySourceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        self.external = init_repo(root / "zr_vm")
        external_workspace = self.external / "Cargo.toml"
        external_workspace.write_text(
            "[workspace]\nmembers=['binding']\n"
            "[workspace.package]\nversion='0.1.0'\nedition='2021'\n"
        )
        external_manifest = self.external / "binding/Cargo.toml"
        external_manifest.parent.mkdir(parents=True)
        external_manifest.write_text(
            "[package]\nname='binding'\nversion.workspace=true\nedition.workspace=true\n"
        )
        external_source = self.external / "binding/src/lib.rs"
        external_source.parent.mkdir(parents=True)
        external_source.write_text("pub fn binding() {}\n")
        subprocess.run(
            ["git", "add", "Cargo.toml", "binding/Cargo.toml", "binding/src/lib.rs"],
            cwd=self.external,
            check=True,
        )
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add external binding"],
            cwd=self.external,
            check=True,
        )
        self.external_commit = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.external,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        SessionService(self.database, self.repo).register(session_id="session-a")
        BaselineService(self.database, self.repo).initialize()
        self.target_root = root / "targets/zircon-engine"
        self.target_root.mkdir(parents=True)
        with mock.patch(
            "tools.session_coordinator.workspace_copy._is_managed_validation_root",
            return_value=True,
        ):
            self.service = WorkspaceCopyService(
                self.database, self.repo, (self.target_root,)
            )

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def _external_descriptor(self) -> dict[str, object]:
        return {
            "repoRoot": str(self.external),
            "commit": self.external_commit,
            "mountPath": "zr_vm",
            "includeRoots": ["binding"],
        }

    def test_external_git_source_uses_pinned_commit_and_survives_restart(self) -> None:
        (self.external / "binding/Cargo.toml").write_text("foreign dirty\n")

        record = self.service.materialize(
            "session-a",
            include_paths=("README.md",),
            external_sources=(self._external_descriptor(),),
        )

        mounted = record.job_root / "zr_vm/binding/Cargo.toml"
        self.assertIn("name='binding'", mounted.read_text(encoding="utf-8"))
        self.assertNotIn("foreign dirty", mounted.read_text(encoding="utf-8"))
        self.assertEqual(self.external_commit, record.external_sources[0]["commit"])
        with mock.patch(
            "tools.session_coordinator.workspace_copy._is_managed_validation_root",
            return_value=True,
        ):
            restarted = WorkspaceCopyService(
                self.database, self.repo, (self.target_root,)
            ).status("session-a", record.job_id)
        self.assertEqual(record.external_sources, restarted.external_sources)
        self.assertEqual(record.input_manifest_hash, restarted.input_manifest_hash)

    def test_external_mount_escape_and_missing_commit_fail_closed(self) -> None:
        escaped = self._external_descriptor()
        escaped["mountPath"] = "../foreign"
        with self.assertRaises(CoordinatorError) as escape:
            self.service.plan(
                "session-a",
                include_paths=("README.md",),
                external_sources=(escaped,),
            )
        self.assertEqual("validation_copy_external_mount_escape", escape.exception.code)

        missing = self._external_descriptor()
        missing["commit"] = "0" * 40
        with self.assertRaises(CoordinatorError) as absent:
            self.service.plan(
                "session-a",
                include_paths=("README.md",),
                external_sources=(missing,),
            )
        self.assertEqual("validation_copy_external_commit_missing", absent.exception.code)

    def test_async_materialization_persists_typed_failure_and_preflights_unowned_overlay(
        self,
    ) -> None:
        unowned = self.repo / "src/unowned.rs"
        unowned.parent.mkdir()
        unowned.write_text("unowned\n")
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.materialize_async(
                "session-a", include_paths=("src/unowned.rs",)
            )
        self.assertEqual("validation_copy_unowned_path", rejected.exception.code)

        failed = threading.Event()

        def archive_failure(*_args, **_kwargs):
            failed.set()
            raise CoordinatorError(
                "validation_copy_dependency_archive_failed",
                "simulated archive failure",
                details={"path": "README.md"},
            )

        with mock.patch.object(
            self.service, "_extract_baseline_manifest", side_effect=archive_failure
        ):
            record = self.service.materialize_async(
                "session-a", include_paths=("README.md",)
            )
            self.assertTrue(failed.wait(timeout=2))
        for _ in range(100):
            status = self.service.status("session-a", record.job_id)
            if status.status == "failed":
                break
            threading.Event().wait(0.01)
        self.assertEqual("validation_copy_dependency_archive_failed", status.error_code)
        self.assertEqual("baseline_archive", status.error_stage)
        self.assertEqual("README.md", status.error_path)

    def test_cargo_metadata_closure_includes_local_packages_and_requires_external_descriptor(
        self,
    ) -> None:
        files = {
            "Cargo.toml": (
                "[workspace]\n"
                "members=['app','local_dep','app/workspace_tool']\n"
                "exclude=['manifest_only']\n"
            ),
            "Cargo.lock": "# lock\n",
            "rust-toolchain.toml": "[toolchain]\nchannel='1.94.1'\n",
            "app/Cargo.toml": (
                "[package]\nname='app'\nversion='0.1.0'\n"
                "[dependencies]\n"
                "binding={path='../../zr_vm/binding', optional=true}\n"
                "manifest_only={path='../manifest_only', optional=true}\n"
            ),
            "app/src/lib.rs": "include_str!(\"schema.txt\");\n",
            "app/src/schema.txt": "schema-v1\n",
            "local_dep/Cargo.toml": "[package]\nname='local_dep'\nversion='0.1.0'\n",
            "local_dep/src/lib.rs": "pub fn local() {}\n",
            "app/workspace_tool/Cargo.toml": "[package]\nname='workspace_tool'\nversion='0.1.0'\n",
            "app/workspace_tool/src/lib.rs": "pub fn tool() {}\n",
            "manifest_only/Cargo.toml": "[package]\nname='manifest_only'\nversion='0.1.0'\n",
            "manifest_only/src/lib.rs": "pub fn manifest_only() {}\n",
            "manifest_only/src/unused.rs": "pub fn unused() {}\n",
        }
        for relative, content in files.items():
            target = self.repo / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content)
        subprocess.run(["git", "add", "--", *files], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add cargo closure"],
            cwd=self.repo,
            check=True,
        )
        metadata = {
            "packages": [
                {
                    "id": "app-id",
                    "name": "app",
                    "manifest_path": str(self.repo / "app/Cargo.toml"),
                    "targets": [{"src_path": str(self.repo / "app/src/lib.rs")}],
                },
                {
                    "id": "local-id",
                    "name": "local_dep",
                    "manifest_path": str(self.repo / "local_dep/Cargo.toml"),
                    "targets": [
                        {"src_path": str(self.repo / "local_dep/src/lib.rs")}
                    ],
                },
                {
                    "id": "external-id",
                    "name": "binding",
                    "manifest_path": str(self.external / "binding/Cargo.toml"),
                    "targets": [
                        {"src_path": str(self.external / "binding/src/lib.rs")}
                    ],
                },
                {
                    "id": "tool-id",
                    "name": "workspace_tool",
                    "manifest_path": str(self.repo / "app/workspace_tool/Cargo.toml"),
                    "targets": [
                        {
                            "src_path": str(
                                self.repo / "app/workspace_tool/src/lib.rs"
                            )
                        }
                    ],
                },
                {
                    "id": "manifest-only-id",
                    "name": "manifest_only",
                    "manifest_path": str(self.repo / "manifest_only/Cargo.toml"),
                    "targets": [
                        {"src_path": str(self.repo / "manifest_only/src/lib.rs")}
                    ],
                },
            ],
            "workspace_members": ["app-id", "local-id", "tool-id"],
            "resolve": {
                "nodes": [
                    {"id": "app-id", "deps": [{"pkg": "local-id"}]},
                    {"id": "local-id", "deps": []},
                    {"id": "external-id", "deps": []},
                    {"id": "tool-id", "deps": [{"pkg": "external-id"}]},
                    {"id": "manifest-only-id", "deps": []},
                ]
            },
        }
        planner = CargoInputClosurePlanner(
            self.repo, metadata_runner=lambda _command: metadata
        )
        descriptor = ExternalGitSource.from_payload(self._external_descriptor())

        closure = planner.plan(
            ("cargo", "test", "-p", "app", "--lib"),
            external_sources=(descriptor,),
        )

        self.assertTrue(
            {
                "Cargo.toml",
                "Cargo.lock",
                "rust-toolchain.toml",
                "app/src/schema.txt",
                "local_dep/src/lib.rs",
                "app/workspace_tool/Cargo.toml",
                "app/workspace_tool/src/lib.rs",
                "manifest_only/Cargo.toml",
                "manifest_only/src/lib.rs",
            }
            <= set(closure.repository_paths)
        )
        self.assertNotIn("manifest_only/src/unused.rs", closure.repository_paths)
        self.assertEqual(1, len(closure.external_sources))
        self.assertEqual(
            ("binding/Cargo.toml", "binding/src/lib.rs", "Cargo.toml"),
            closure.external_sources[0].include_roots,
        )
        with self.assertRaises(CoordinatorError) as missing:
            planner.plan(("cargo", "test", "-p", "app", "--lib"))
        self.assertEqual("validation_copy_external_source_missing", missing.exception.code)

        discovered = planner.plan(
            ("cargo", "test", "-p", "app", "--lib"),
            discover_external_sources=True,
        )
        self.assertEqual(1, len(discovered.external_sources))
        self.assertEqual(self.external.resolve(), discovered.external_sources[0].repo_root)
        self.assertEqual(self.external_commit, discovered.external_sources[0].commit)
        self.assertEqual("zr_vm", discovered.external_sources[0].mount_path)
        self.assertIn("binding/Cargo.toml", discovered.external_sources[0].include_roots)
        self.assertIn("binding/src/lib.rs", discovered.external_sources[0].include_roots)

        materialized = self.service.materialize_cargo(
            "session-a",
            command=("cargo", "test", "-p", "app", "--lib"),
            metadata_runner=lambda _command: metadata,
            discover_external_sources=True,
        )
        self.assertTrue((materialized.job_root / "zr_vm/Cargo.toml").is_file())
        self.assertTrue((materialized.job_root / "zr_vm/binding/Cargo.toml").is_file())
        self.assertTrue((materialized.job_root / "zr_vm/binding/src/lib.rs").is_file())
        self.assertTrue(
            (materialized.source_root / "app/workspace_tool/Cargo.toml").is_file()
        )
        self.assertTrue(
            (materialized.source_root / "app/workspace_tool/src/lib.rs").is_file()
        )
        self.assertTrue(
            (materialized.source_root / "manifest_only/src/lib.rs").is_file()
        )
        self.assertFalse(
            (materialized.source_root / "manifest_only/src/unused.rs").exists()
        )

        metadata["resolve"]["nodes"][0]["deps"].append({"pkg": "external-id"})
        selected_external = planner.plan(
            ("cargo", "test", "-p", "app", "--lib"),
            external_sources=(descriptor,),
        )
        self.assertEqual(
            ("binding", "binding/Cargo.toml", "Cargo.toml"),
            selected_external.external_sources[0].include_roots,
        )

        foreign = init_repo(self.external.parent / "foreign")
        foreign_manifest = foreign / "binding/Cargo.toml"
        foreign_manifest.parent.mkdir(parents=True)
        foreign_manifest.write_text("[package]\nname='foreign-binding'\nversion='0.1.0'\n")
        subprocess.run(
            ["git", "add", "binding/Cargo.toml"], cwd=foreign, check=True
        )
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add foreign binding"],
            cwd=foreign,
            check=True,
        )
        foreign_commit = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=foreign,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        foreign_descriptor = ExternalGitSource.from_payload(
            {
                "repoRoot": str(foreign),
                "commit": foreign_commit,
                "mountPath": "zr_vm",
                "includeRoots": ["binding"],
            }
        )
        metadata["packages"].append(
            {
                "id": "foreign-id",
                "name": "foreign-binding",
                "manifest_path": str(foreign_manifest),
            }
        )
        metadata["resolve"]["nodes"].append({"id": "foreign-id", "deps": []})
        metadata["resolve"]["nodes"][0]["deps"].append({"pkg": "foreign-id"})

        with self.assertRaises(CoordinatorError) as duplicate_mount:
            planner.plan(
                ("cargo", "test", "-p", "app", "--lib"),
                external_sources=(descriptor, foreign_descriptor),
            )
        self.assertEqual(
            "validation_copy_external_mount_conflict", duplicate_mount.exception.code
        )

        with self.assertRaises(CoordinatorError) as discovery_conflict:
            planner.plan(
                ("cargo", "test", "-p", "app", "--lib"),
                external_sources=(foreign_descriptor,),
                discover_external_sources=True,
            )
        self.assertEqual(
            "validation_copy_external_mount_conflict", discovery_conflict.exception.code
        )

    def test_package_scoped_compile_resources_ignore_unselected_workspace_members(
        self,
    ) -> None:
        files = {
            "Cargo.toml": "[workspace]\nmembers=['app','local_dep','workspace_tool']\n",
            "Cargo.lock": "# lock\n",
            "app/Cargo.toml": "[package]\nname='app'\nversion='0.1.0'\n",
            "app/src/lib.rs": "const _: &str = include_str!(\"schema.txt\");\n",
            "app/src/schema.txt": "selected schema\n",
            "local_dep/Cargo.toml": (
                "[package]\nname='local_dep'\nversion='0.1.0'\n"
            ),
            "local_dep/src/lib.rs": (
                "const _: &str = include_str!(\"schema.txt\");\n"
            ),
            "local_dep/src/schema.txt": "dependency schema\n",
            "app/workspace_tool/Cargo.toml": (
                "[package]\nname='workspace_tool'\nversion='0.1.0'\n"
            ),
            "app/workspace_tool/src/lib.rs": (
                "const _: &str = include_str!(\"missing.txt\");\n"
            ),
        }
        for relative, content in files.items():
            target = self.repo / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content, encoding="utf-8")
        subprocess.run(["git", "add", "--", *files], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add scoped Cargo closure"],
            cwd=self.repo,
            check=True,
        )
        metadata = {
            "packages": [
                {
                    "id": "app-id",
                    "name": "app",
                    "manifest_path": str(self.repo / "app/Cargo.toml"),
                    "targets": [{"src_path": str(self.repo / "app/src/lib.rs")}],
                },
                {
                    "id": "local-id",
                    "name": "local_dep",
                    "manifest_path": str(self.repo / "local_dep/Cargo.toml"),
                    "targets": [
                        {"src_path": str(self.repo / "local_dep/src/lib.rs")}
                    ],
                },
                {
                    "id": "tool-id",
                    "name": "workspace_tool",
                    "manifest_path": str(
                        self.repo / "app/workspace_tool/Cargo.toml"
                    ),
                    "targets": [
                        {
                            "src_path": str(
                                self.repo / "app/workspace_tool/src/lib.rs"
                            )
                        }
                    ],
                },
            ],
            "workspace_members": ["app-id", "local-id", "tool-id"],
            "resolve": {
                "nodes": [
                    {"id": "app-id", "deps": [{"pkg": "local-id"}]},
                    {"id": "local-id", "deps": []},
                    {"id": "tool-id", "deps": []},
                ]
            },
        }

        closure = CargoInputClosurePlanner(
            self.repo,
            metadata_runner=lambda _command: metadata,
        ).plan(("pwsh", "-Command", "cargo test", "--package", "app", "--lib"))

        self.assertIn("app/src/schema.txt", closure.repository_paths)
        self.assertIn("local_dep/src/lib.rs", closure.repository_paths)
        self.assertIn("local_dep/src/schema.txt", closure.repository_paths)
        self.assertIn("app/workspace_tool/Cargo.toml", closure.repository_paths)
        self.assertIn("app/workspace_tool/src/lib.rs", closure.repository_paths)
        self.assertNotIn("app/workspace_tool/src/missing.txt", closure.repository_paths)

        with self.assertRaises(CoordinatorError) as unscoped:
            CargoInputClosurePlanner(
                self.repo,
                metadata_runner=lambda _command: metadata,
            ).plan(("pwsh", "-Command", "cargo test --workspace"))
        self.assertEqual(
            "validation_copy_compile_time_resource_missing", unscoped.exception.code
        )
        self.assertEqual(
            (self.repo / "app/workspace_tool/src/lib.rs").resolve(),
            Path(str(unscoped.exception.details["sourcePath"])),
        )

    def test_cargo_closure_includes_compile_time_resources_outside_package_root(
        self,
    ) -> None:
        files = {
            "Cargo.toml": "[workspace]\nmembers=['interface']\n",
            "Cargo.lock": "# lock\n",
            "interface/Cargo.toml": "[package]\nname='interface'\nversion='0.1.0'\n",
            "interface/src/lib.rs": "mod embedded;\n",
            "interface/src/embedded.rs": (
                "const TEMPLATE_NAMESPACE: &'static str = \"renderable-empty\";\n"
                "macro_rules! template_bytes {\n"
                "    ($path:literal) => {\n"
                "        include_bytes!(concat!(\n"
                "            env!(\"CARGO_MANIFEST_DIR\"),\n"
                "            \"/../templates/projects/renderable-empty/\",\n"
                "            $path,\n"
                "        ))\n"
                "    };\n"
                "}\n"
                "const _: &[u8] = template_bytes!(\"scene.toml\");\n"
                "// include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../templates/ignored/\"))\n"
                "const EXAMPLE: &str = \"include_bytes!(concat!(env!(\\\"CARGO_MANIFEST_DIR\\\"), \\\"/../templates/ignored/\\\"))\";\n"
                "const RAW_EXAMPLE: &str = r##\"include_bytes!(concat!(env!(\\\"CARGO_MANIFEST_DIR\\\"), \\\"/../templates/ignored/\\\"))\"##;\n"
            ),
            "templates/projects/renderable-empty/scene.toml": "scene\n",
            "templates/projects/renderable-empty/assets/texture.bin": "texture\n",
            "templates/ignored/never-materialize.bin": "ignored\n",
        }
        for relative, content in files.items():
            target = self.repo / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content, encoding="utf-8")
        subprocess.run(["git", "add", "--", *files], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add compile-time resource fixture"],
            cwd=self.repo,
            check=True,
        )
        metadata = {
            "packages": [
                {
                    "id": "interface-id",
                    "name": "interface",
                    "manifest_path": str(self.repo / "interface/Cargo.toml"),
                }
            ],
            "workspace_members": ["interface-id"],
            "resolve": {"nodes": [{"id": "interface-id", "deps": []}]},
        }

        closure = CargoInputClosurePlanner(
            self.repo,
            metadata_runner=lambda _command: metadata,
        ).plan(("cargo", "test", "-p", "interface", "--lib"))

        self.assertTrue(
            {
                "templates/projects/renderable-empty/scene.toml",
                "templates/projects/renderable-empty/assets/texture.bin",
            }
            <= set(closure.repository_paths)
        )
        self.assertNotIn(
            "templates/ignored/never-materialize.bin", closure.repository_paths
        )

    def test_compile_time_resource_uses_tracked_baseline_when_worktree_file_is_deleted(
        self,
    ) -> None:
        files = {
            "Cargo.toml": "[workspace]\nmembers=['app']\n",
            "Cargo.lock": "# lock\n",
            "app/Cargo.toml": "[package]\nname='app'\nversion='0.1.0'\n",
            "app/src/lib.rs": "const _: &str = include_str!(\"schema.txt\");\n",
            "app/src/schema.txt": "committed schema\n",
        }
        for relative, content in files.items():
            target = self.repo / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content, encoding="utf-8")
        subprocess.run(["git", "add", "--", *files], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add baseline resource fixture"],
            cwd=self.repo,
            check=True,
        )
        (self.repo / "app/src/schema.txt").unlink()
        metadata = {
            "packages": [
                {
                    "id": "app-id",
                    "name": "app",
                    "manifest_path": str(self.repo / "app/Cargo.toml"),
                }
            ],
            "workspace_members": ["app-id"],
            "resolve": {"nodes": [{"id": "app-id", "deps": []}]},
        }

        closure = CargoInputClosurePlanner(
            self.repo,
            metadata_runner=lambda _command: metadata,
        ).plan(("cargo", "test", "-p", "app", "--lib"))

        self.assertIn("app/src/schema.txt", closure.repository_paths)

    def test_compile_time_resource_rejects_live_untracked_file(self) -> None:
        tracked = {
            "Cargo.toml": "[workspace]\nmembers=['app']\n",
            "Cargo.lock": "# lock\n",
            "app/Cargo.toml": "[package]\nname='app'\nversion='0.1.0'\n",
            "app/src/lib.rs": "const _: &str = include_str!(\"schema.txt\");\n",
        }
        for relative, content in tracked.items():
            target = self.repo / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content, encoding="utf-8")
        subprocess.run(["git", "add", "--", *tracked], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add untracked resource fixture"],
            cwd=self.repo,
            check=True,
        )
        (self.repo / "app/src/schema.txt").write_text(
            "untracked schema\n", encoding="utf-8"
        )
        metadata = {
            "packages": [
                {
                    "id": "app-id",
                    "name": "app",
                    "manifest_path": str(self.repo / "app/Cargo.toml"),
                }
            ],
            "workspace_members": ["app-id"],
            "resolve": {"nodes": [{"id": "app-id", "deps": []}]},
        }

        with self.assertRaises(CoordinatorError) as raised:
            CargoInputClosurePlanner(
                self.repo,
                metadata_runner=lambda _command: metadata,
            ).plan(("cargo", "test", "-p", "app", "--lib"))

        self.assertEqual(
            "validation_copy_compile_time_resource_missing", raised.exception.code
        )
        self.assertEqual(
            (self.repo / "app/src/schema.txt").resolve(),
            Path(str(raised.exception.details["resourcePath"])),
        )

    def test_compile_time_resource_discovery_uses_bounded_git_arguments(self) -> None:
        resource_count = 2_400
        resource_roots = {
            (
                "interface/resources/windows-command-length/"
                f"resource_{index:04d}_compile_time_fixture.txt"
            )
            for index in range(resource_count)
        }
        planner = CargoInputClosurePlanner(
            self.repo, metadata_runner=lambda _command: {}
        )

        def tracked_batch(command, *_args, **_kwargs):
            roots = tuple(str(part) for part in command[3:])
            return subprocess.CompletedProcess(
                args=command,
                returncode=0,
                stdout="\n".join(roots),
                stderr="",
            )

        with mock.patch(
            "tools.session_coordinator.validation_copies.subprocess.run",
            side_effect=tracked_batch,
        ) as run:
            resources = planner._tracked_compile_time_resources(resource_roots)

        self.assertEqual(resource_count, len(resources))
        commands = [
            tuple(str(part) for part in call.args[0]) for call in run.call_args_list
        ]
        self.assertGreater(len(commands), 1)
        self.assertEqual(
            tuple(sorted(resource_roots)),
            tuple(path for command in commands for path in command[3:]),
        )
        self.assertLess(
            max(len(subprocess.list2cmdline(command)) for command in commands),
            32_768,
        )

    def test_compile_time_resource_git_start_failure_is_actionable(self) -> None:
        files = {
            "Cargo.toml": "[workspace]\nmembers=['interface']\n",
            "interface/Cargo.toml": "[package]\nname='interface'\nversion='0.1.0'\n",
            "interface/src/lib.rs": "const _: &str = include_str!(\"schema.txt\");\n",
            "interface/src/schema.txt": "schema\n",
        }
        for relative, content in files.items():
            target = self.repo / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content, encoding="utf-8")
        subprocess.run(["git", "add", "--all"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add resource failure fixture"],
            cwd=self.repo,
            check=True,
        )
        metadata = {
            "packages": [
                {
                    "id": "interface-id",
                    "name": "interface",
                    "manifest_path": str(self.repo / "interface/Cargo.toml"),
                }
            ],
            "workspace_members": ["interface-id"],
            "resolve": {"nodes": [{"id": "interface-id", "deps": []}]},
        }
        real_run = subprocess.run
        failure = FileNotFoundError(2, "The filename or extension is too long")
        failure.winerror = 206

        def fail_resource_pathspec(command, *args, **kwargs):
            if any(str(part).endswith("schema.txt") for part in command):
                raise failure
            return real_run(command, *args, **kwargs)

        with mock.patch(
            "tools.session_coordinator.validation_copies.subprocess.run",
            side_effect=fail_resource_pathspec,
        ):
            with self.assertRaises(CoordinatorError) as raised:
                CargoInputClosurePlanner(
                    self.repo,
                    metadata_runner=lambda _command: metadata,
                ).plan(("cargo", "test", "-p", "interface", "--lib"))

        self.assertEqual(
            "validation_copy_compile_time_resource_git_failed",
            raised.exception.code,
        )
        self.assertEqual(
            {
                "operation": "git_ls_files_compile_time_resources",
                "errorType": "FileNotFoundError",
                "resourceRootCount": 1,
                "errno": 2,
                "winerror": 206,
            },
            raised.exception.details,
        )

    def test_cargo_metadata_is_decoded_as_utf8_independent_of_windows_locale(self) -> None:
        completed = subprocess.CompletedProcess(
            args=("cargo", "metadata"),
            returncode=0,
            stdout='{"packages": [], "resolve": {"nodes": []}}',
            stderr="",
        )
        planner = CargoInputClosurePlanner(self.repo)

        with mock.patch(
            "tools.session_coordinator.validation_copies.subprocess.run",
            return_value=completed,
        ) as run:
            metadata = planner._cargo_metadata(("cargo", "test"))

        self.assertEqual([], metadata["packages"])
        self.assertEqual("utf-8", run.call_args.kwargs.get("encoding"))

    def test_cargo_closure_ignores_registry_packages_outside_repository(self) -> None:
        app = self.repo / "app"
        app.mkdir()
        (app / "Cargo.toml").write_text(
            "[package]\nname='app'\nversion='0.1.0'\n",
            encoding="utf-8",
        )
        subprocess.run(["git", "add", "app/Cargo.toml"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add registry closure fixture"],
            cwd=self.repo,
            check=True,
        )
        metadata = {
            "packages": [
                {
                    "id": "app-id",
                    "name": "app",
                    "manifest_path": str(app / "Cargo.toml"),
                    "source": None,
                },
                {
                    "id": "registry-id",
                    "name": "registry_dep",
                    "manifest_path": str(
                        Path.home() / ".cargo/registry/src/example/registry_dep/Cargo.toml"
                    ),
                    "source": "registry+https://github.com/rust-lang/crates.io-index",
                },
            ],
            "workspace_members": ["app-id"],
            "resolve": {
                "nodes": [
                    {"id": "app-id", "deps": [{"pkg": "registry-id"}]},
                    {"id": "registry-id", "deps": []},
                ]
            },
        }

        closure = CargoInputClosurePlanner(
            self.repo,
            metadata_runner=lambda _command: metadata,
        ).plan(("cargo", "test", "-p", "app"), discover_external_sources=True)

        self.assertIn("app/Cargo.toml", closure.repository_paths)
        self.assertEqual((), closure.external_sources)


if __name__ == "__main__":
    unittest.main()
