from __future__ import annotations

import tempfile
import tomllib
import unittest
import subprocess
from pathlib import Path
from unittest import mock

from tools.session_coordinator.artifact_governance import ArtifactGovernanceService
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.pinned_cargo_planner import (
    PinnedCargoInputClosurePlanner,
    PinnedCargoPlannerView,
    _run_cargo_metadata,
)
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.validation_copy_external import ExternalGitSource
from tools.session_coordinator.workspace_copy import WorkspaceCopyService


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

    def tearDown(self) -> None:
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

    def test_absolute_manifest_path_is_rewritten_into_the_pinned_view(self) -> None:
        observed_commands: list[tuple[str, ...]] = []

        def metadata_executor(
            source_root: Path, command: tuple[str, ...]
        ) -> dict[str, object]:
            observed_commands.append(command)
            return self._metadata(source_root, "dep_a")

        with PinnedCargoPlannerView(
            self.repo,
            self.planner_parent,
            baseline_commit=self.baseline_commit,
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
                baseline_commit=self.baseline_commit,
            )
            expected_manifest = view.repo_root / "app/Cargo.toml"

        manifest_index = observed_commands[0].index("--manifest-path") + 1
        self.assertEqual(str(expected_manifest), observed_commands[0][manifest_index])
        self.assertNotEqual(
            str(self.repo / "app/Cargo.toml"), observed_commands[0][manifest_index]
        )
        config_index = observed_commands[0].index("--config") + 1
        self.assertEqual(str(view.repo_root / ".cargo/config.toml"), observed_commands[0][config_index])

    def test_metadata_preserves_cargo_toolchain_selector(self) -> None:
        completed = subprocess.CompletedProcess(
            ["cargo"], 0, stdout="{}", stderr=""
        )
        with mock.patch(
            "tools.session_coordinator.pinned_cargo_planner.subprocess.run",
            return_value=completed,
        ) as run:
            _run_cargo_metadata(
                self.repo,
                ("cargo", "+1.94.1", "test", "-p", "app"),
            )

        metadata_command = run.call_args.args[0]
        self.assertEqual(
            ["cargo", "+1.94.1", "metadata"], metadata_command[:3]
        )

    def test_metadata_preserves_global_config_before_subcommand_and_stops_at_delimiter(
        self,
    ) -> None:
        completed = subprocess.CompletedProcess(
            ["cargo"], 0, stdout="{}", stderr=""
        )
        with mock.patch(
            "tools.session_coordinator.pinned_cargo_planner.subprocess.run",
            return_value=completed,
        ) as run:
            _run_cargo_metadata(
                self.repo,
                (
                    "cargo",
                    "+1.94.1",
                    "--config",
                    "ci/cargo.toml",
                    "--offline",
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
        self.assertNotIn("test-binary.toml", metadata_command)

    def test_metadata_qualifies_features_for_selected_package(self) -> None:
        completed = subprocess.CompletedProcess(
            ["cargo"], 0, stdout="{}", stderr=""
        )
        with mock.patch(
            "tools.session_coordinator.pinned_cargo_planner.subprocess.run",
            return_value=completed,
        ) as run:
            _run_cargo_metadata(
                self.repo,
                ("cargo", "+1.94.1", "test", "-p", "app", "--features", "optional_ext"),
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
            source_root: Path, _command: tuple[str, ...]
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
                "mountPath": "vendor-binding",
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
        self.assertEqual("vendor-binding", closure.external_sources[0].mount_path)
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

    def test_registered_job_root_contains_planner_during_metadata(self) -> None:
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
            source_root: Path, _command: tuple[str, ...]
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
                command=("cargo", "check", "-p", "app"),
                descriptors=(),
                discover_external_sources=False,
                overlays=(),
                baseline_commit=self.baseline_commit,
                planner_parent=job_root,
            )

        self.assertTrue(job_root.is_dir())
        self.assertEqual([], list(job_root.iterdir()))

    def test_workspace_copy_service_production_path_uses_the_pinned_view(self) -> None:
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
            source_root: Path, _command: tuple[str, ...]
        ) -> dict[str, object]:
            dependency = self._manifest_dependency(source_root / "app/Cargo.toml")
            observed_dependencies.append(dependency)
            return self._metadata(source_root, dependency)

        with mock.patch(
            "tools.session_coordinator.pinned_cargo_planner._run_cargo_metadata",
            side_effect=metadata_executor,
        ):
            closure = service._plan_cargo_closure_pinned(
                command=("cargo", "check", "-p", "app"),
                descriptors=(),
                discover_external_sources=False,
                overlays=(),
                baseline_commit=self.baseline_commit,
            )

        self.assertEqual(["dep_a"], observed_dependencies)
        self.assertIn("dep_a/src/lib.rs", closure.repository_paths)
        self.assertNotIn("dep_b/src/lib.rs", closure.repository_paths)
        self.assertEqual([], list((target_root / "verify").iterdir()))

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
