from __future__ import annotations

import subprocess
import tempfile
import threading
import time
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.pinned_cargo_planner import (
    PinnedCargoInputClosurePlanner,
    PinnedCargoPlannerView,
    _trusted_metadata_tool_identity,
)
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.pinned_metadata_cache import (
    PinnedMetadataCache,
    build_pinned_metadata_cache_key,
)
from tools.session_coordinator.tests.helpers import init_repo


class PinnedMetadataCacheTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_cross_view_hit_rebases_paths_and_package_ids(self) -> None:
        cache = PinnedMetadataCache(max_entries=8, max_bytes=1024 * 1024)
        first = self._view("first")
        second = self._view("second")
        calls = 0

        def execute(root: Path) -> dict[str, object]:
            nonlocal calls
            calls += 1
            package_id = f"app 0.1.0 (path+{(root / 'app').as_uri()})"
            dependency_id = f"path+{(root.parent / 'sibling').as_uri()}#dep@0.1.0"
            return {
                "packages": [
                    {
                        "id": package_id,
                        "manifest_path": str(root / "app/Cargo.toml"),
                        "targets": [{"src_path": str(root / "app/src/lib.rs")}],
                    },
                    {
                        "id": dependency_id,
                        "manifest_path": str(root.parent / "sibling/Cargo.toml"),
                    },
                ],
                "resolve": {
                    "root": package_id,
                    "nodes": [
                        {
                            "id": package_id,
                            "dependencies": [dependency_id],
                            "deps": [{"pkg": dependency_id}],
                        }
                    ],
                },
                "workspace_members": [package_id],
                "workspace_default_members": [package_id],
                "workspace_root": str(root),
                "target_directory": str(root.parent / "metadata-target"),
            }

        first_result, first_observation = cache.get_or_execute_observed(
            "shared", first.parent, lambda: execute(first)
        )
        second_result, second_observation = cache.get_or_execute_observed(
            "shared", second.parent, lambda: execute(second)
        )

        self.assertEqual(1, calls)
        self.assertEqual(str(first), first_result["workspace_root"])
        self.assertEqual(str(second), second_result["workspace_root"])
        self.assertEqual(
            str(second / "app/Cargo.toml"),
            second_result["packages"][0]["manifest_path"],
        )
        self.assertIn(
            (second / "app").as_uri(), second_result["workspace_members"][0]
        )
        self.assertIn(
            (second / "app").as_uri(),
            second_result["workspace_default_members"][0],
        )
        self.assertIn(
            (second.parent / "sibling").as_uri(),
            second_result["resolve"]["nodes"][0]["deps"][0]["pkg"],
        )
        self.assertEqual(1, cache.stats().hits)
        self.assertEqual("miss", first_observation.outcome)
        self.assertGreaterEqual(first_observation.execution_seconds, 0.0)
        self.assertEqual("hit", second_observation.outcome)
        self.assertEqual(0.0, second_observation.execution_seconds)

    def test_planner_coalesces_same_executor_across_temporary_views(self) -> None:
        repo = init_repo(self.root / "repo")
        files = {
            "Cargo.toml": "[workspace]\nmembers=['app']\n",
            "Cargo.lock": "version = 3\n",
            "app/Cargo.toml": "[package]\nname='app'\nversion='0.1.0'\n",
            "app/src/lib.rs": "pub fn current() {}\n",
        }
        for relative, content in files.items():
            path = repo / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        subprocess.run(
            ["git", "add", "."], cwd=repo, check=True, capture_output=True
        )
        subprocess.run(
            ["git", "commit", "-m", "test: add workspace"],
            cwd=repo,
            check=True,
            capture_output=True,
        )
        baseline = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        planner_parent = self.root / "planner"
        planner_parent.mkdir()
        metadata_cache = PinnedMetadataCache()
        calls = 0
        first_root: Path | None = None

        def execute(source_root: Path, _command: tuple[str, ...]) -> dict[str, object]:
            nonlocal calls
            calls += 1
            package_id = f"path+{(source_root / 'app').as_uri()}#app@0.1.0"
            return {
                "packages": [
                    {
                        "id": package_id,
                        "manifest_path": str(source_root / "app/Cargo.toml"),
                        "targets": [{"src_path": str(source_root / "app/src/lib.rs")}],
                    }
                ],
                "resolve": {"nodes": [{"id": package_id, "deps": []}]},
                "workspace_members": [package_id],
            }

        with PinnedCargoPlannerView(
            repo, planner_parent, baseline_commit=baseline
        ) as view:
            first_root = view.require_active_repo_root()
            first_planner = PinnedCargoInputClosurePlanner(
                view,
                metadata_executor=execute,
                metadata_cache=metadata_cache,
                metadata_cache_namespace="planner-test",
            )
            first_planner._metadata(("cargo", "check", "--locked"))
        with PinnedCargoPlannerView(
            repo, planner_parent, baseline_commit=baseline
        ) as view:
            second_root = view.require_active_repo_root()
            second_planner = PinnedCargoInputClosurePlanner(
                view,
                metadata_executor=execute,
                metadata_cache=metadata_cache,
                metadata_cache_namespace="planner-test",
            )
            second = second_planner._metadata(("cargo", "check", "--locked"))

        self.assertEqual(1, calls)
        self.assertNotEqual(first_root, second_root)
        self.assertEqual(
            str(second_root / "app/Cargo.toml"),
            second["packages"][0]["manifest_path"],
        )
        self.assertIn(
            (second_root / "app").as_uri(), second["workspace_members"][0]
        )
        self.assertEqual("miss", first_planner.metadata_cache_result["outcome"])
        self.assertEqual("hit", second_planner.metadata_cache_result["outcome"])
        self.assertTrue(second_planner.metadata_cache_result["hit"])

    def test_topology_and_command_inputs_invalidate_cache_key(self) -> None:
        source = self._view("key")
        command = ("cargo", "check", "--locked", "-p", "app")

        def key(
            *,
            selected_command: tuple[str, ...] = command,
            external: tuple[str, ...] = ("external-rev-a",),
            tool: str = "cargo-a",
        ) -> str | None:
            return build_pinned_metadata_cache_key(
                source.parent,
                source,
                selected_command,
                external_identities=external,
                executor_namespace="test-executor",
                tool_identity=tool,
            )

        baseline = key()
        mutations = (
            ("feature", lambda: key(selected_command=command + ("--features", "fast"))),
            (
                "inline config",
                lambda: key(
                    selected_command=command
                    + ("--config", "net.offline=true")
                ),
            ),
            (
                "selected target",
                lambda: key(
                    selected_command=command
                    + ("--target", "x86_64-pc-windows-msvc")
                ),
            ),
            (
                "toolchain selector",
                lambda: key(selected_command=("cargo", "+nightly", *command[1:])),
            ),
            ("external pin", lambda: key(external=("external-rev-b",))),
            ("trusted tool", lambda: key(tool="cargo-b")),
        )
        for label, changed in mutations:
            with self.subTest(label=label):
                self.assertNotEqual(baseline, changed())

        files = (
            source / "Cargo.toml",
            source / "Cargo.lock",
            source / "rust-toolchain.toml",
            source / ".cargo/config.toml",
        )
        for path in files:
            original = path.read_bytes()
            with self.subTest(path=path.name):
                path.write_bytes(original + b"\n# changed")
                self.assertNotEqual(baseline, key())
                path.write_bytes(original)

        target = source / "app/src/bin/new_tool.rs"
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("fn main() {}\n", encoding="utf-8")
        self.assertNotEqual(baseline, key())

    def test_explicit_config_content_invalidates_cache_key(self) -> None:
        source = self._view("explicit")
        config = source / "ci/cargo.toml"
        config.parent.mkdir(parents=True)
        config.write_text("[net]\noffline = true\n", encoding="utf-8")
        command = ("cargo", "check", "--locked", "--config", str(config))
        first = build_pinned_metadata_cache_key(
            source.parent, source, command, executor_namespace="test"
        )
        config.write_text("[net]\noffline = false\n", encoding="utf-8")
        second = build_pinned_metadata_cache_key(
            source.parent, source, command, executor_namespace="test"
        )
        self.assertNotEqual(first, second)

    def test_config_filename_containing_equals_is_hashed_and_rebased(self) -> None:
        source = self._view("equals-config")
        config = source / "ci/foo=one.toml"
        config.parent.mkdir(parents=True)
        config.write_text("[net]\noffline = true\n", encoding="utf-8")
        command = ("cargo", "check", "--locked", "--config", str(config))
        first = build_pinned_metadata_cache_key(source.parent, source, command)
        config.write_text("[net]\noffline = false\n", encoding="utf-8")
        second = build_pinned_metadata_cache_key(source.parent, source, command)
        self.assertNotEqual(first, second)

        other_source = self._view("other-equals-config")
        other_config = other_source / "ci/foo=one.toml"
        other_config.parent.mkdir(parents=True)
        other_config.write_text("[net]\noffline = false\n", encoding="utf-8")
        other_command = (
            "cargo",
            "check",
            "--locked",
            "--config",
            str(other_config),
        )
        self.assertEqual(
            second,
            build_pinned_metadata_cache_key(
                other_source.parent, other_source, other_command
            ),
        )

        class View:
            logical_repo_root = source

            @staticmethod
            def ensure_main_view_file(relative: str) -> Path:
                return source / relative

            @staticmethod
            def view_path_for_logical(path: Path) -> Path:
                return path

        planner = object.__new__(PinnedCargoInputClosurePlanner)
        planner.view = View()
        planner.logical_repo_root = source
        self.assertEqual(
            str(config), planner._path_argument_for_view("--config", str(config))
        )

    def test_trusted_toolchain_identity_uses_pinned_view_and_failure_bypasses(self) -> None:
        source = self._view("tool-identity")
        command = ("cargo", "check", "--locked")
        with mock.patch(
            "tools.session_coordinator.pinned_cargo_planner."
            "trusted_rust_toolchain_identity",
            return_value='{"cargo":"pinned","rustc":"pinned"}',
        ) as identity:
            self.assertEqual(
                '{"cargo":"pinned","rustc":"pinned"}',
                _trusted_metadata_tool_identity(command, source, source),
            )
        identity.assert_called_once_with(
            command, source, working_directory=source
        )

        with mock.patch(
            "tools.session_coordinator.pinned_cargo_planner."
            "trusted_rust_toolchain_identity",
            side_effect=CoordinatorError("probe_failed", "probe failed"),
        ):
            self.assertIsNone(
                _trusted_metadata_tool_identity(command, source, source)
            )

    def test_source_content_change_with_same_shape_is_a_hit(self) -> None:
        source = self._view("source-content")
        production = source / "app/src/lib.rs"
        command = ("cargo", "check", "--locked")
        first = build_pinned_metadata_cache_key(source.parent, source, command)
        production.write_text("pub fn changed() -> bool { true }\n", encoding="utf-8")
        second = build_pinned_metadata_cache_key(source.parent, source, command)
        self.assertEqual(first, second)

    def test_concurrent_callers_share_one_execution(self) -> None:
        cache = PinnedMetadataCache(max_entries=8, max_bytes=1024 * 1024)
        source = self._view("concurrent")
        start = threading.Barrier(10)
        calls = 0
        call_lock = threading.Lock()

        def execute() -> dict[str, object]:
            nonlocal calls
            with call_lock:
                calls += 1
            time.sleep(0.05)
            return {"workspace_root": str(source)}

        outcomes: list[str] = []

        def worker() -> None:
            start.wait()
            _metadata, observation = cache.get_or_execute_observed(
                "same", source.parent, execute
            )
            outcomes.append(observation.outcome)

        threads = [threading.Thread(target=worker) for _ in range(10)]
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join(timeout=2)

        self.assertTrue(all(not thread.is_alive() for thread in threads))
        self.assertEqual(1, calls)
        self.assertEqual(10, len(outcomes))
        self.assertEqual(1, outcomes.count("miss"))
        self.assertEqual(9, outcomes.count("coalesced"))
        self.assertEqual(9, cache.stats().coalesced)
        self.assertEqual(1, cache.stats().executions)
        self.assertGreaterEqual(cache.stats().execution_seconds, 0.05)

    def test_eviction_and_failure_retry(self) -> None:
        cache = PinnedMetadataCache(max_entries=1, max_bytes=1024 * 1024)
        source = self._view("eviction")
        cache.get_or_execute("first", source.parent, lambda: {"value": 1})
        cache.get_or_execute("second", source.parent, lambda: {"value": 2})
        cache.get_or_execute("first", source.parent, lambda: {"value": 3})
        self.assertEqual(3, cache.stats().misses)
        self.assertEqual(1, cache.stats().entries)

        attempts = 0

        def flaky() -> dict[str, object]:
            nonlocal attempts
            attempts += 1
            if attempts == 1:
                raise RuntimeError("transient")
            return {"value": "ok"}

        with self.assertRaisesRegex(RuntimeError, "transient"):
            cache.get_or_execute("failure", source.parent, flaky)
        self.assertEqual(
            {"value": "ok"},
            cache.get_or_execute("failure", source.parent, flaky),
        )
        self.assertEqual(2, attempts)

        tiny = PinnedMetadataCache(max_entries=8, max_bytes=64)
        executions = 0

        def oversized() -> dict[str, object]:
            nonlocal executions
            executions += 1
            return {"payload": "x" * 1024}

        tiny.get_or_execute("large", source.parent, oversized)
        tiny.get_or_execute("large", source.parent, oversized)
        self.assertEqual(2, executions)
        self.assertEqual(0, tiny.stats().entries)

    def test_fresh_cache_service_starts_cold(self) -> None:
        source = self._view("cold")
        first = PinnedMetadataCache(max_entries=8, max_bytes=1024 * 1024)
        first.get_or_execute("key", source.parent, lambda: {"value": 1})
        second = PinnedMetadataCache(max_entries=8, max_bytes=1024 * 1024)
        second.get_or_execute("key", source.parent, lambda: {"value": 2})
        self.assertEqual(1, second.stats().misses)
        self.assertEqual(0, second.stats().hits)

    def _view(self, name: str) -> Path:
        view = self.root / name
        source = view / "source"
        files = {
            "Cargo.toml": "[workspace]\nmembers=['app']\n",
            "Cargo.lock": "version = 3\n",
            "rust-toolchain.toml": "[toolchain]\nchannel='stable'\n",
            ".cargo/config.toml": "[net]\noffline=true\n",
            "app/Cargo.toml": "[package]\nname='app'\nversion='0.1.0'\n",
            "app/src/lib.rs": "pub fn current() {}\n",
        }
        for relative, content in files.items():
            path = source / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        sibling = view / "sibling"
        sibling.mkdir()
        (sibling / "Cargo.toml").write_text(
            "[package]\nname='dep'\nversion='0.1.0'\n", encoding="utf-8"
        )
        return source


if __name__ == "__main__":
    unittest.main()
