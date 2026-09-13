from __future__ import annotations

import hashlib
import io
import subprocess
import tarfile
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.pinned_cargo_planner import (
    PinnedCargoInputClosurePlanner,
    PinnedCargoPlannerView,
    _cargo_manifest_topology_paths,
    _run_cargo_metadata,
)
from tools.session_coordinator.pinned_metadata_cache import PinnedMetadataCache
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.validation_copy_external import ExternalGitSource


class PinnedCargoPlannerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.root = root
        self.repo = init_repo(root / "repo")
        self.planner_parent = root / "targets"
        self.planner_parent.mkdir()
        self._write_workspace_dependency("dep_a")
        self._commit("test: add cargo workspace")
        self.baseline_commit = self._git_output("rev-parse", "HEAD")
        self.cache_patch = mock.patch(
            "tools.session_coordinator.pinned_cargo_planner._PINNED_METADATA_CACHE",
            new=PinnedMetadataCache(),
        )
        self.cache_patch.start()
        self.tool_identity_patch = mock.patch(
            "tools.session_coordinator.pinned_cargo_planner."
            "_trusted_metadata_tool_identity",
            return_value="test-toolchain",
        )
        self.tool_identity_patch.start()

    def tearDown(self) -> None:
        self.tool_identity_patch.stop()
        self.cache_patch.stop()
        self.temporary.cleanup()

    def test_planner_uses_pinned_manifests_after_live_topology_changes(self) -> None:
        self._write_workspace_dependency("dep_b")

        observed_dependencies: list[str] = []

        def metadata_executor(
            source_root: Path, _command: tuple[str, ...]
        ) -> dict[str, object]:
            dependency = self._manifest_dependency(source_root / "app/Cargo.toml")
            observed_dependencies.append(dependency)
            return self._metadata(source_root, dependency)

        with PinnedCargoPlannerView(
            self.repo,
            self.planner_parent,
            baseline_commit=self.baseline_commit,
        ) as view:
            closure = PinnedCargoInputClosurePlanner(
                view,
                metadata_executor=metadata_executor,
            ).plan(
                ("cargo", "test", "-p", "app"),
                baseline_commit=self.baseline_commit,
            )

        self.assertEqual(["dep_a"], observed_dependencies)
        self.assertIn("dep_a/Cargo.toml", closure.repository_paths)
        self.assertIn("dep_a/src/lib.rs", closure.repository_paths)
        self.assertNotIn("dep_b/Cargo.toml", closure.repository_paths)
        self.assertEqual([], list(self.planner_parent.iterdir()))

    def test_check_lib_does_not_expand_cfg_test_path_module_as_resource(self) -> None:
        product = self.repo / "app/src/product.rs"
        product.write_text(
            '#[cfg(test)]\n#[path = "product/tests.rs"]\nmod tests;\n'
            "pub fn ready() {}\n",
            encoding="utf-8",
        )
        tests = self.repo / "app/src/product/tests.rs"
        tests.parent.mkdir(parents=True, exist_ok=True)
        tests.write_text(
            'const SOURCE: &str = include_str!("product.rs");\n',
            encoding="utf-8",
        )
        (self.repo / "app/src/lib.rs").write_text(
            "mod product;\n",
            encoding="utf-8",
        )
        self._commit("test: add cfg test path module")
        baseline = self._git_output("rev-parse", "HEAD")

        def metadata_executor(
            source_root: Path, _command: tuple[str, ...]
        ) -> dict[str, object]:
            metadata = self._metadata(source_root, "dep_a")
            app = next(
                package
                for package in metadata["packages"]
                if package["name"] == "app"
            )
            app["targets"] = [
                {
                    "name": "app",
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "src_path": str(source_root / "app/src/lib.rs"),
                }
            ]
            return metadata

        with PinnedCargoPlannerView(
            self.repo,
            self.planner_parent,
            baseline_commit=baseline,
        ) as view:
            closure = PinnedCargoInputClosurePlanner(
                view,
                metadata_executor=metadata_executor,
            ).plan_pinned(
                ("cargo", "check", "-p", "app", "--lib"),
                baseline_commit=baseline,
            )

        self.assertIn("app/src/product.rs", closure.repository_paths)
        self.assertIn("app/src/product/tests.rs", closure.repository_paths)

    def test_absolute_manifest_path_is_rewritten_into_the_pinned_view(self) -> None:
        cargo_config = self.repo / ".cargo/config.toml"
        cargo_config.parent.mkdir()
        cargo_config.write_text("[net]\noffline = true\n", encoding="utf-8")
        self._commit("test: add pinned cargo config")
        baseline = self._git_output("rev-parse", "HEAD")
        observed_commands: list[tuple[str, ...]] = []

        def metadata_executor(
            source_root: Path, command: tuple[str, ...]
        ) -> dict[str, object]:
            observed_commands.append(command)
            return self._metadata(source_root, "dep_a")

        with PinnedCargoPlannerView(
            self.repo,
            self.planner_parent,
            baseline_commit=baseline,
        ) as view:
            PinnedCargoInputClosurePlanner(
                view,
                metadata_executor=metadata_executor,
            ).plan(
                (
                    "cargo",
                    "check",
                    "--manifest-path",
                    str(self.repo / "app/Cargo.toml"),
                    "--config",
                    str(self.repo / ".cargo/config.toml"),
                    "-p",
                    "app",
                ),
                baseline_commit=baseline,
            )
            expected_manifest = view.repo_root / "app/Cargo.toml"
            expected_config = view.repo_root / ".cargo/config.toml"

        manifest_index = observed_commands[0].index("--manifest-path") + 1
        self.assertEqual(str(expected_manifest), observed_commands[0][manifest_index])
        self.assertNotEqual(
            str(self.repo / "app/Cargo.toml"), observed_commands[0][manifest_index]
        )
        config_index = observed_commands[0].index("--config") + 1
        self.assertEqual(str(expected_config), observed_commands[0][config_index])

    def test_metadata_places_resolution_flags_after_subcommand(self) -> None:
        completed = subprocess.CompletedProcess(
            ["cargo"], 0, stdout="{}", stderr=""
        )
        with (
            mock.patch(
                "tools.session_coordinator.pinned_cargo_planner.bind_trusted_cargo",
                side_effect=lambda command, *_args, **_kwargs: command,
            ),
            mock.patch(
                "tools.session_coordinator.pinned_cargo_planner.bind_trusted_rust_environment",
                side_effect=lambda environment, *_args, **_kwargs: environment,
            ),
            mock.patch(
                "tools.session_coordinator.pinned_cargo_planner.prepare_shared_metadata_cargo_home",
                return_value=self.root / "cargo-home",
            ),
            mock.patch(
                "tools.session_coordinator.pinned_cargo_planner.subprocess.run",
                return_value=completed,
            ) as run,
        ):
            _run_cargo_metadata(
                self.repo,
                ("cargo", "+1.94.1", "test", "-p", "app", "--locked"),
            )

        metadata_command = run.call_args.args[0]
        self.assertEqual(["cargo", "+1.94.1"], metadata_command[:2])
        self.assertLess(metadata_command.index("metadata"), metadata_command.index("--locked"))

    def test_metadata_uses_stable_shared_cargo_home(self) -> None:
        completed = subprocess.CompletedProcess(
            ["cargo"], 0, stdout="{}", stderr=""
        )
        shared_home = self.root / "shared-metadata-cargo-home"
        with (
            mock.patch(
                "tools.session_coordinator.pinned_cargo_planner.bind_trusted_cargo",
                side_effect=lambda command, *_args, **_kwargs: command,
            ),
            mock.patch(
                "tools.session_coordinator.pinned_cargo_planner.bind_trusted_rust_environment",
                side_effect=lambda environment, *_args, **_kwargs: environment,
            ),
            mock.patch(
                "tools.session_coordinator.pinned_cargo_planner.prepare_shared_metadata_cargo_home",
                return_value=shared_home,
            ) as prepare_shared,
            mock.patch(
                "tools.session_coordinator.pinned_cargo_planner.subprocess.run",
                return_value=completed,
            ) as run,
        ):
            _run_cargo_metadata(
                self.repo,
                ("cargo", "metadata", "--locked"),
            )

        prepare_shared.assert_called_once_with(self.repo)
        self.assertEqual(str(shared_home), run.call_args.kwargs["env"]["CARGO_HOME"])

    def test_metadata_preserves_global_config_before_subcommand_and_stops_at_delimiter(
        self,
    ) -> None:
        completed = subprocess.CompletedProcess(
            ["cargo"], 0, stdout="{}", stderr=""
        )
        with (
            mock.patch(
                "tools.session_coordinator.pinned_cargo_planner.bind_trusted_cargo",
                side_effect=lambda command, *_args, **_kwargs: command,
            ),
            mock.patch(
                "tools.session_coordinator.pinned_cargo_planner.bind_trusted_rust_environment",
                side_effect=lambda environment, *_args, **_kwargs: environment,
            ),
            mock.patch(
                "tools.session_coordinator.pinned_cargo_planner.prepare_shared_metadata_cargo_home",
                return_value=self.root / "cargo-home",
            ),
            mock.patch(
                "tools.session_coordinator.pinned_cargo_planner.subprocess.run",
                return_value=completed,
            ) as run,
        ):
            _run_cargo_metadata(
                self.repo,
                (
                    "cargo",
                    "+1.94.1",
                    "--config",
                    "ci/cargo.toml",
                    "--offline",
                    "--locked",
                    "test",
                    "-p",
                    "app",
                    "--",
                    "--config",
                    "test-binary.toml",
                ),
            )

        metadata_command = run.call_args.args[0]
        self.assertIn("--offline", metadata_command)
        config_index = metadata_command.index("--config")
        self.assertEqual("ci/cargo.toml", metadata_command[config_index + 1])
        metadata_index = metadata_command.index("metadata")
        self.assertLess(config_index, metadata_index)
        self.assertLess(metadata_index, metadata_command.index("--offline"))
        self.assertLess(metadata_index, metadata_command.index("--locked"))
        self.assertNotIn("test-binary.toml", metadata_command)

    def test_metadata_qualifies_features_for_selected_package(self) -> None:
        completed = subprocess.CompletedProcess(
            ["cargo"], 0, stdout="{}", stderr=""
        )
        with (
            mock.patch(
                "tools.session_coordinator.pinned_cargo_planner.bind_trusted_cargo",
                side_effect=lambda command, *_args, **_kwargs: command,
            ),
            mock.patch(
                "tools.session_coordinator.pinned_cargo_planner.bind_trusted_rust_environment",
                side_effect=lambda environment, *_args, **_kwargs: environment,
            ),
            mock.patch(
                "tools.session_coordinator.pinned_cargo_planner.prepare_shared_metadata_cargo_home",
                return_value=self.root / "cargo-home",
            ),
            mock.patch(
                "tools.session_coordinator.pinned_cargo_planner.subprocess.run",
                return_value=completed,
            ) as run,
        ):
            _run_cargo_metadata(
                self.repo,
                (
                    "cargo",
                    "+1.94.1",
                    "test",
                    "-p",
                    "app",
                    "--features",
                    "optional_ext",
                    "--locked",
                ),
            )

        metadata_command = run.call_args.args[0]
        feature_index = metadata_command.index("--features")
        self.assertEqual("app/optional_ext", metadata_command[feature_index + 1])

    def test_manifest_patch_path_is_discovered(self) -> None:
        self._create_external_binding("zr_vm")
        (self.repo / "app/Cargo.toml").write_text(
            "[package]\nname='app'\nversion='0.1.0'\nedition='2021'\n"
            "[patch.crates-io]\nbinding={path='../../zr_vm/binding'}\n",
            encoding="utf-8",
        )
        self._commit("test: use manifest patch path")
        baseline = self._git_output("rev-parse", "HEAD")

        with PinnedCargoPlannerView(
            self.repo,
            self.planner_parent,
            baseline_commit=baseline,
            discover_external_sources=True,
        ) as view:
            self.assertTrue((view.root / "zr_vm").exists())

    def test_manifest_target_may_leave_package_but_not_repository(self) -> None:
        view = self.root / "topology-view"
        manifest = view / "app/Cargo.toml"
        manifest.parent.mkdir(parents=True)
        manifest.write_text(
            "[package]\nname='app'\nversion='0.1.0'\n"
            "[lib]\npath='../shared/lib.rs'\n",
            encoding="utf-8",
        )
        shared = view / "shared/lib.rs"
        shared.parent.mkdir(parents=True)
        shared.write_text("mod child;\n", encoding="utf-8")
        available = {"app/Cargo.toml", "shared/lib.rs"}

        targets, _references = _cargo_manifest_topology_paths(view, available)

        self.assertEqual({"shared/lib.rs"}, targets)

    def test_relative_manifest_escape_is_rejected(self) -> None:
        with PinnedCargoPlannerView(
            self.repo,
            self.planner_parent,
            baseline_commit=self.baseline_commit,
        ) as view:
            planner = PinnedCargoInputClosurePlanner(
                view,
                metadata_executor=lambda _root, _command: {},
            )
            with self.assertRaises(Exception) as raised:
                planner._command_for_view(
                    ("cargo", "check", "--manifest-path=../live/Cargo.toml")
                )

        self.assertEqual(
            "cargo_source_path_argument_invalid", raised.exception.code
        )

    def test_sealed_overlay_manifest_controls_metadata_and_new_package_topology(
        self,
    ) -> None:
        overlay = {
            "Cargo.toml": self._root_manifest("dep_b").encode("utf-8"),
            "app/Cargo.toml": self._app_manifest("dep_b").encode("utf-8"),
            "dep_b/Cargo.toml": self._dependency_manifest("dep_b").encode("utf-8"),
            "dep_b/src/lib.rs": b"pub fn dep_b() {}\n",
        }

        def metadata_executor(
            source_root: Path, _command: tuple[str, ...], **_kwargs
        ) -> dict[str, object]:
            dependency = self._manifest_dependency(source_root / "app/Cargo.toml")
            return self._metadata(source_root, dependency)

        with PinnedCargoPlannerView(
            self.repo,
            self.planner_parent,
            baseline_commit=self.baseline_commit,
            overlay_files=overlay,
        ) as view:
            self.assertEqual(
                "dep_b", self._manifest_dependency(view.repo_root / "app/Cargo.toml")
            )
            closure = PinnedCargoInputClosurePlanner(
                view,
                metadata_executor=metadata_executor,
            ).plan(
                ("cargo", "check", "-p", "app"),
                overlay_paths=tuple(overlay),
                baseline_commit=self.baseline_commit,
            )

        self.assertIn("dep_b/Cargo.toml", closure.repository_paths)
        self.assertIn("dep_b/src/lib.rs", closure.repository_paths)
        self.assertNotIn("dep_a/src/lib.rs", closure.repository_paths)

    def test_overlay_tombstone_removes_pinned_file(self) -> None:
        with PinnedCargoPlannerView(
            self.repo,
            self.planner_parent,
            baseline_commit=self.baseline_commit,
            overlay_files={"dep_a/src/lib.rs": None},
        ) as view:
            self.assertFalse((view.repo_root / "dep_a/src/lib.rs").exists())

    def test_planner_view_is_removed_when_consumer_raises(self) -> None:
        with self.assertRaisesRegex(RuntimeError, "stop"):
            with PinnedCargoPlannerView(
                self.repo,
                self.planner_parent,
                baseline_commit=self.baseline_commit,
            ):
                raise RuntimeError("stop")

        self.assertEqual([], list(self.planner_parent.iterdir()))

    def test_external_metadata_paths_map_to_the_pinned_sibling_descriptor(self) -> None:
        external, external_commit = self._create_external_binding("external")
        self._write_workspace_dependency("dep_a")
        (self.repo / "Cargo.toml").write_text(
            "[workspace]\nmembers=['app']\nresolver='2'\n",
            encoding="utf-8",
        )
        (self.repo / "app/Cargo.toml").write_text(
            "[package]\nname='app'\nversion='0.1.0'\nedition='2021'\n"
            "[dependencies]\nbinding={path='../../external/binding'}\n",
            encoding="utf-8",
        )
        self._commit("test: use external binding")
        baseline = self._git_output("rev-parse", "HEAD")
        descriptor = ExternalGitSource.from_payload(
            {
                "repoRoot": str(external),
                "commit": external_commit,
                "mountPath": "external",
                "includeRoots": ["binding"],
            }
        )

        def metadata_executor(
            source_root: Path, _command: tuple[str, ...]
        ) -> dict[str, object]:
            external_view = source_root.parent / "external"
            app_id = "app-id"
            binding_id = "binding-id"
            return {
                "packages": [
                    {
                        "id": app_id,
                        "name": "app",
                        "source": None,
                        "manifest_path": str(source_root / "app/Cargo.toml"),
                        "targets": [
                            {"src_path": str(source_root / "app/src/lib.rs")}
                        ],
                    },
                    {
                        "id": binding_id,
                        "name": "binding",
                        "source": None,
                        "manifest_path": str(
                            external_view / "binding/Cargo.toml"
                        ),
                        "targets": [
                            {
                                "src_path": str(
                                    external_view / "binding/src/lib.rs"
                                )
                            }
                        ],
                    },
                ],
                "resolve": {
                    "nodes": [
                        {"id": app_id, "deps": [{"pkg": binding_id}]},
                        {"id": binding_id, "deps": []},
                    ]
                },
                "workspace_members": [app_id],
            }

        with PinnedCargoPlannerView(
            self.repo,
            self.planner_parent,
            baseline_commit=baseline,
            external_sources=(descriptor,),
        ) as view:
            closure = PinnedCargoInputClosurePlanner(
                view,
                metadata_executor=metadata_executor,
            ).plan_pinned(
                ("cargo", "check", "-p", "app"),
                external_sources=(descriptor,),
                baseline_commit=baseline,
            )

        self.assertEqual(1, len(closure.external_sources))
        self.assertEqual(external_commit, closure.external_sources[0].commit)
        self.assertEqual("external", closure.external_sources[0].mount_path)
        self.assertIn("binding", closure.external_sources[0].include_roots)

    def test_external_discovery_pins_sibling_before_metadata(self) -> None:
        external, external_commit = self._create_external_binding("discovered")
        (self.repo / "Cargo.toml").write_text(
            "[workspace]\nmembers=['app']\nresolver='2'\n",
            encoding="utf-8",
        )
        (self.repo / "app/Cargo.toml").write_text(
            "[package]\nname='app'\nversion='0.1.0'\nedition='2021'\n"
            "[dependencies]\nbinding={path='../../discovered/binding'}\n",
            encoding="utf-8",
        )
        self._commit("test: discover external binding")
        baseline = self._git_output("rev-parse", "HEAD")

        def metadata_executor(
            source_root: Path, _command: tuple[str, ...]
        ) -> dict[str, object]:
            external_view = source_root.parent / "discovered"
            self.assertTrue((external_view / "binding/Cargo.toml").is_file())
            return self._external_metadata(source_root, external_view)

        with PinnedCargoPlannerView(
            self.repo,
            self.planner_parent,
            baseline_commit=baseline,
            discover_external_sources=True,
        ) as view:
            closure = PinnedCargoInputClosurePlanner(
                view,
                metadata_executor=metadata_executor,
            ).plan_pinned(
                ("cargo", "check", "-p", "app"),
                external_sources=view.external_sources,
                discover_external_sources=True,
                baseline_commit=baseline,
            )

        self.assertEqual(1, len(closure.external_sources))
        self.assertEqual(external.resolve(), closure.external_sources[0].repo_root)
        self.assertEqual(external_commit, closure.external_sources[0].commit)
        self.assertEqual("discovered", closure.external_sources[0].mount_path)

    def test_real_metadata_subprocess_resolves_pinned_sibling_workspace(self) -> None:
        """The production metadata argv must work against an immutable sibling view."""
        root = self.root / "real-metadata"
        main = init_repo(root / "main")
        sibling = init_repo(root / "sibling")

        (sibling / "dep/src").mkdir(parents=True)
        (sibling / "dep/Cargo.toml").write_text(
            "[package]\nname='sibling_dep'\nversion='0.1.0'\nedition='2021'\n",
            encoding="utf-8",
        )
        (sibling / "dep/src/lib.rs").write_text(
            "pub fn dependency_ready() {}\n", encoding="utf-8"
        )
        (sibling / "Cargo.lock").write_text("version = 4\n", encoding="utf-8")
        subprocess.run(["git", "add", "."], cwd=sibling, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add sibling dependency"],
            cwd=sibling,
            check=True,
        )

        (main / "app/src").mkdir(parents=True)
        (main / "Cargo.toml").write_text(
            "[workspace]\nmembers=['app']\nresolver='2'\n", encoding="utf-8"
        )
        (main / "app/Cargo.toml").write_text(
            "[package]\nname='app'\nversion='0.1.0'\nedition='2021'\n"
            "[dependencies]\nsibling_dep={path='../../sibling/dep'}\n",
            encoding="utf-8",
        )
        (main / "app/src/lib.rs").write_text(
            "pub fn app_ready() { sibling_dep::dependency_ready(); }\n",
            encoding="utf-8",
        )
        (main / "Cargo.lock").write_text(
            "version = 4\n\n[[package]]\nname = 'app'\nversion = '0.1.0'\n"
            "dependencies = ['sibling_dep']\n\n[[package]]\nname = 'sibling_dep'\n"
            "version = '0.1.0'\n",
            encoding="utf-8",
        )
        subprocess.run(["git", "add", "."], cwd=main, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add pinned workspace"],
            cwd=main,
            check=True,
        )
        baseline = subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=main, text=True
        ).strip()
        planner_parent = root / "targets"
        planner_parent.mkdir()

        with PinnedCargoPlannerView(
            main,
            planner_parent,
            baseline_commit=baseline,
            discover_external_sources=True,
        ) as view:
            metadata = _run_cargo_metadata(
                view.repo_root,
                ("cargo", "check", "-p", "app", "--locked"),
                trust_root=main,
            )

        self.assertEqual(
            {"app", "sibling_dep"},
            {str(package["name"]) for package in metadata["packages"]},
        )
        self.assertEqual(1, len(view.external_sources))
        self.assertEqual("sibling", view.external_sources[0].mount_path)

    def test_registered_job_root_contains_planner_during_metadata(self) -> None:
        from tools.session_coordinator.artifact_governance import (
            ArtifactGovernanceService,
        )
        from tools.session_coordinator.database import Database
        from tools.session_coordinator.migrations import migrate
        from tools.session_coordinator.sessions import SessionService
        from tools.session_coordinator.workspace_copy import WorkspaceCopyService

        target_root = self.root / "registered-cargo-targets"
        target_root.mkdir()
        database = Database(self.root / "registered.sqlite3")
        migrate(database)
        SessionService(database, self.repo).register(
            session_id="session-a",
            plan_path="docs/plans/test.md",
        )
        with mock.patch(
            "tools.session_coordinator.workspace_copy._is_managed_validation_root",
            return_value=True,
        ):
            service = WorkspaceCopyService(database, self.repo, (target_root,))
        job_root = (target_root / "verify" / "registered-job").resolve()
        with database.transaction() as connection:
            connection.execute(
                """INSERT INTO validation_copies(
                       job_id, session_id, job_root, source_root, target_root,
                       head_commit, manifest_json, status, created_at,
                       external_sources_json
                   ) VALUES ('registered-job', 'session-a', ?, ?, ?, ?, '[]',
                             'planned', 'now', '[]')""",
                (
                    str(job_root),
                    str(job_root / "source"),
                    str(job_root / "target"),
                    self.baseline_commit,
                ),
            )

        def metadata_executor(
            source_root: Path, _command: tuple[str, ...], **_kwargs
        ) -> dict[str, object]:
            self.assertTrue(source_root.is_relative_to(job_root))
            unmanaged = ArtifactGovernanceService(
                database, roots=(target_root,)
            ).scan()
            self.assertEqual((), unmanaged)
            return self._metadata(source_root, "dep_a")

        with mock.patch(
            "tools.session_coordinator.pinned_cargo_planner._run_cargo_metadata",
            side_effect=metadata_executor,
        ):
            service._plan_cargo_closure_pinned(
                command=("cargo", "check", "-p", "app", "--locked"),
                descriptors=(),
                discover_external_sources=False,
                overlays=(),
                baseline_commit=self.baseline_commit,
                planner_parent=job_root,
            )

        self.assertTrue(job_root.is_dir())
        self.assertEqual([], list(job_root.iterdir()))

    def test_workspace_copy_service_production_path_uses_the_pinned_view(self) -> None:
        from tools.session_coordinator.database import Database
        from tools.session_coordinator.workspace_copy import WorkspaceCopyService

        self._write_workspace_dependency("dep_b")
        target_root = self.root / "cargo-targets"
        target_root.mkdir()
        with mock.patch(
            "tools.session_coordinator.workspace_copy._is_managed_validation_root",
            return_value=True,
        ):
            service = WorkspaceCopyService(
                Database(self.root / "coordinator.sqlite3"),
                self.repo,
                (target_root,),
            )

        observed_dependencies: list[str] = []

        def metadata_executor(
            source_root: Path, _command: tuple[str, ...], **_kwargs
        ) -> dict[str, object]:
            dependency = self._manifest_dependency(source_root / "app/Cargo.toml")
            observed_dependencies.append(dependency)
            return self._metadata(source_root, dependency)

        with mock.patch(
            "tools.session_coordinator.pinned_cargo_planner._run_cargo_metadata",
            side_effect=metadata_executor,
        ):
            closure = service._plan_cargo_closure_pinned(
                command=("cargo", "check", "-p", "app", "--locked"),
                descriptors=(),
                discover_external_sources=False,
                overlays=(),
                baseline_commit=self.baseline_commit,
            )

        self.assertEqual(["dep_a"], observed_dependencies)
        self.assertIn("dep_a/src/lib.rs", closure.repository_paths)
        self.assertNotIn("dep_b/src/lib.rs", closure.repository_paths)
        self.assertEqual([], list((target_root / "verify").iterdir()))

    def test_workspace_copy_pinned_view_loads_sealed_external_archive(self) -> None:
        from tools.session_coordinator.database import Database
        from tools.session_coordinator.workspace_copy import WorkspaceCopyService

        buffer = io.BytesIO()
        with tarfile.open(fileobj=buffer, mode="w:") as archive:
            for relative, payload in {
                "Cargo.toml": b"[workspace]\nmembers=['binding']\n",
                "binding/Cargo.toml": (
                    b"[package]\nname='binding'\nversion='0.1.0'\n"
                ),
                "binding/src/lib.rs": b"pub fn binding() {}\n",
            }.items():
                member = tarfile.TarInfo(relative)
                member.size = len(payload)
                archive.addfile(member, io.BytesIO(payload))
        archive_bytes = buffer.getvalue()
        archive_hash = hashlib.sha256(archive_bytes).hexdigest()
        descriptor = ExternalGitSource.from_payload(
            {
                "repoRoot": str(self.root / "sealed"),
                "commit": "a" * 40,
                "mountPath": "sealed",
                "includeRoots": ["@repo-root"],
                "archiveHash": archive_hash,
                "archiveByteCount": len(archive_bytes),
            }
        )
        object_store = mock.Mock()
        object_store.get.return_value = archive_bytes
        target_root = self.root / "sealed-cargo-targets"
        target_root.mkdir()
        with mock.patch(
            "tools.session_coordinator.workspace_copy._is_managed_validation_root",
            return_value=True,
        ):
            service = WorkspaceCopyService(
                Database(self.root / "sealed-coordinator.sqlite3"),
                self.repo,
                (target_root,),
                object_store=object_store,
            )

        def metadata_executor(
            source_root: Path, _command: tuple[str, ...], **_kwargs
        ) -> dict[str, object]:
            self.assertEqual(
                "pub fn binding() {}\n",
                (source_root.parent / "sealed/binding/src/lib.rs").read_text(
                    encoding="utf-8"
                ),
            )
            return self._metadata(source_root, "dep_a")

        with mock.patch(
            "tools.session_coordinator.pinned_cargo_planner._run_cargo_metadata",
            side_effect=metadata_executor,
        ):
            service._plan_cargo_closure_pinned(
                command=("cargo", "check", "-p", "app", "--locked"),
                descriptors=(descriptor,),
                discover_external_sources=False,
                overlays=(),
                baseline_commit=self.baseline_commit,
            )

        object_store.get.assert_called_once_with(archive_hash)

    def _write_workspace_dependency(self, dependency: str) -> None:
        files = {
            "Cargo.toml": self._root_manifest(dependency),
            "Cargo.lock": "# pinned lock\n",
            "app/Cargo.toml": self._app_manifest(dependency),
            "app/src/lib.rs": "pub fn app() {}\n",
            f"{dependency}/Cargo.toml": self._dependency_manifest(dependency),
            f"{dependency}/src/lib.rs": f"pub fn {dependency}() {{}}\n",
        }
        for relative, content in files.items():
            path = self.repo / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")

    @staticmethod
    def _root_manifest(dependency: str) -> str:
        return f"[workspace]\nmembers=['app','{dependency}']\nresolver='2'\n"

    @staticmethod
    def _app_manifest(dependency: str) -> str:
        return (
            "[package]\nname='app'\nversion='0.1.0'\nedition='2021'\n"
            f"[dependencies]\n{dependency}={{path='../{dependency}'}}\n"
        )

    @staticmethod
    def _dependency_manifest(dependency: str) -> str:
        return (
            f"[package]\nname='{dependency}'\nversion='0.1.0'\nedition='2021'\n"
        )

    @staticmethod
    def _manifest_dependency(manifest: Path) -> str:
        payload = tomllib.loads(manifest.read_text(encoding="utf-8"))
        dependencies = payload["dependencies"]
        return next(iter(dependencies))

    @staticmethod
    def _metadata(source_root: Path, dependency: str) -> dict[str, object]:
        app_id = "app 0.1.0 (path+file:///app)"
        dependency_id = f"{dependency} 0.1.0 (path+file:///{dependency})"
        return {
            "packages": [
                {
                    "id": app_id,
                    "name": "app",
                    "source": None,
                    "manifest_path": str(source_root / "app/Cargo.toml"),
                    "targets": [{"src_path": str(source_root / "app/src/lib.rs")}],
                },
                {
                    "id": dependency_id,
                    "name": dependency,
                    "source": None,
                    "manifest_path": str(source_root / f"{dependency}/Cargo.toml"),
                    "targets": [
                        {"src_path": str(source_root / f"{dependency}/src/lib.rs")}
                    ],
                },
            ],
            "resolve": {
                "nodes": [
                    {"id": app_id, "deps": [{"pkg": dependency_id}]},
                    {"id": dependency_id, "deps": []},
                ]
            },
            "workspace_members": [app_id, dependency_id],
        }

    @staticmethod
    def _external_metadata(
        source_root: Path, external_view: Path
    ) -> dict[str, object]:
        app_id = "app-id"
        binding_id = "binding-id"
        return {
            "packages": [
                {
                    "id": app_id,
                    "name": "app",
                    "source": None,
                    "manifest_path": str(source_root / "app/Cargo.toml"),
                    "targets": [{"src_path": str(source_root / "app/src/lib.rs")}],
                },
                {
                    "id": binding_id,
                    "name": "binding",
                    "source": None,
                    "manifest_path": str(external_view / "binding/Cargo.toml"),
                    "targets": [
                        {"src_path": str(external_view / "binding/src/lib.rs")}
                    ],
                },
            ],
            "resolve": {
                "nodes": [
                    {"id": app_id, "deps": [{"pkg": binding_id}]},
                    {"id": binding_id, "deps": []},
                ]
            },
            "workspace_members": [app_id],
        }

    def _create_external_binding(self, name: str) -> tuple[Path, str]:
        external = init_repo(self.repo.parent / name)
        files = {
            "Cargo.toml": "[workspace]\nmembers=['binding']\nresolver='2'\n",
            "binding/Cargo.toml": (
                "[package]\nname='binding'\nversion='0.1.0'\nedition='2021'\n"
            ),
            "binding/src/lib.rs": "pub fn binding() {}\n",
        }
        for relative, content in files.items():
            path = external / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        subprocess.run(["git", "add", "--all"], cwd=external, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add external binding"],
            cwd=external,
            check=True,
        )
        commit = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=external,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        return external, commit

    def _commit(self, message: str) -> None:
        self._git_output("add", "--all")
        self._git_output("commit", "-q", "-m", message)

    def _git_output(self, *arguments: str) -> str:
        return subprocess.run(
            ["git", *arguments],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()


if __name__ == "__main__":
    unittest.main()
