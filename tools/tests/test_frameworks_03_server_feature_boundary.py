from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

if sys.version_info >= (3, 11):
    import tomllib
else:
    import tomli as tomllib


REPO_ROOT = Path(__file__).resolve().parents[2]
RUNTIME_MANIFEST = REPO_ROOT / "zircon_runtime" / "Cargo.toml"
RUNTIME_ROOT = REPO_ROOT / "zircon_runtime" / "src" / "lib.rs"
RHI_BOUNDARY_TEST = (
    REPO_ROOT
    / "zircon_runtime"
    / "crates"
    / "zr_rhi"
    / "src"
    / "tests"
    / "boundary.rs"
)
APP_MANIFEST = REPO_ROOT / "zircon_app" / "Cargo.toml"
APP_PLUGIN_GROUPS = REPO_ROOT / "zircon_app" / "src" / "plugins" / "groups.rs"
APP_PLUGIN_GROUP_RESOLUTION = (
    REPO_ROOT / "zircon_app" / "src" / "plugins" / "groups" / "resolution.rs"
)
RUNTIME_CORE_MODULES = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "builtin"
    / "runtime_modules"
    / "core_modules.rs"
)


def reachable_feature_graph(
    root_manifest_path: Path,
    root_features: tuple[str, ...],
) -> tuple[set[tuple[str, str]], set[str]]:
    reachable_features: set[tuple[str, str]] = set()
    reachable_packages: set[str] = set()
    manifests: dict[Path, dict] = {}
    workspace_dependencies: dict[Path, dict] = {}
    requested_features_by_manifest: dict[Path, set[str]] = {}
    default_features_by_manifest: dict[Path, bool] = {}
    pending_manifests: list[Path] = []
    queued_manifests: set[Path] = set()

    def load_manifest(path: Path) -> dict:
        path = path.resolve()
        if path not in manifests:
            manifests[path] = tomllib.loads(path.read_text(encoding="utf-8"))
        return manifests[path]

    def workspace_dependencies_for(manifest_path: Path) -> tuple[Path, dict] | None:
        manifest_path = manifest_path.resolve()
        for parent in (manifest_path.parent, *manifest_path.parents):
            candidate = parent / "Cargo.toml"
            if not candidate.is_file():
                continue
            workspace_manifest = load_manifest(candidate)
            if "workspace" not in workspace_manifest:
                continue
            if candidate not in workspace_dependencies:
                workspace_dependencies[candidate] = workspace_manifest.get(
                    "workspace", {}
                ).get("dependencies", {})
            return candidate.parent, workspace_dependencies[candidate]
        return None

    def windows_target_matches(selector: str) -> bool | None:
        normalized = selector.replace(" ", "")
        if selector == "x86_64-pc-windows-msvc":
            return True
        if normalized in {
            "cfg(windows)",
            'cfg(target_os="windows")',
            'cfg(target_family="windows")',
        }:
            return True
        if normalized in {
            "cfg(unix)",
            'cfg(target_os="linux")',
            'cfg(target_os="macos")',
            'cfg(target_arch="wasm32")',
        }:
            return False
        return None

    def local_dependencies(manifest_path: Path, manifest: dict) -> dict[str, tuple[Path, dict]]:
        dependencies: dict[str, tuple[Path, dict]] = {}
        workspace = workspace_dependencies_for(manifest_path)
        dependency_tables = [manifest.get("dependencies", {})]
        for selector, target_table in manifest.get("target", {}).items():
            if not isinstance(selector, str) or not isinstance(target_table, dict):
                continue
            target_dependencies = target_table.get("dependencies", {})
            if not isinstance(target_dependencies, dict):
                continue
            target_matches = windows_target_matches(selector)
            if target_matches:
                dependency_tables.append(target_dependencies)
                continue
            if target_matches is None and any(
                isinstance(entry, dict)
                and ("path" in entry or entry.get("workspace"))
                for entry in target_dependencies.values()
            ):
                raise AssertionError(
                    f"cannot resolve local target dependencies for {selector!r} in "
                    f"{manifest_path}"
                )

        entries_by_name: dict[str, list[dict]] = {}
        for dependency_table in dependency_tables:
            for dependency_name, entry in dependency_table.items():
                if isinstance(entry, dict):
                    entries_by_name.setdefault(dependency_name, []).append(entry)

        for dependency_name, entries in entries_by_name.items():
            local_entries: list[tuple[Path, dict]] = []
            has_nonlocal_entry = False
            for entry in entries:
                resolved = dict(entry)
                dependency_root = manifest_path.parent
                if resolved.get("workspace"):
                    if workspace is None:
                        raise AssertionError(
                            f"cannot resolve workspace dependency {dependency_name!r} "
                            f"in {manifest_path}"
                        )
                    workspace_root, workspace_entries = workspace
                    inherited = workspace_entries.get(dependency_name)
                    if not isinstance(inherited, dict):
                        has_nonlocal_entry = True
                        continue
                    inherited_features = inherited.get("features", [])
                    member_features = resolved.get("features", [])
                    resolved = {**inherited, **resolved}
                    resolved["features"] = [
                        *(
                            feature
                            for feature in inherited_features
                            if isinstance(feature, str)
                        ),
                        *(
                            feature
                            for feature in member_features
                            if isinstance(feature, str)
                        ),
                    ]
                    dependency_root = workspace_root
                dependency_path = resolved.get("path")
                if not isinstance(dependency_path, str):
                    has_nonlocal_entry = True
                    continue
                local_entries.append(
                    ((dependency_root / dependency_path / "Cargo.toml").resolve(), resolved)
                )

            if not local_entries:
                continue
            if has_nonlocal_entry:
                raise AssertionError(
                    f"cannot merge local and external dependency {dependency_name!r} "
                    f"in {manifest_path}"
                )
            dependency_manifests = {entry_manifest for entry_manifest, _ in local_entries}
            if len(dependency_manifests) != 1:
                raise AssertionError(
                    f"cannot merge local dependency {dependency_name!r} with "
                    f"different manifests in {manifest_path}"
                )

            dependency_manifest = local_entries[0][0]
            merged = dict(local_entries[0][1])
            merged["features"] = sorted(
                {
                    feature
                    for _, entry in local_entries
                    for feature in entry.get("features", [])
                    if isinstance(feature, str)
                }
            )
            merged["optional"] = all(
                entry.get("optional", False) for _, entry in local_entries
            )
            merged["default-features"] = any(
                entry.get("default-features", True) for _, entry in local_entries
            )
            dependencies[dependency_name] = (dependency_manifest, merged)
        return dependencies

    def request_package(
        manifest_path: Path,
        requested_features: set[str],
        enable_default_features: bool,
    ) -> None:
        manifest_path = manifest_path.resolve()
        requested = requested_features_by_manifest.setdefault(manifest_path, set())
        requested_before = len(requested)
        requested.update(requested_features)
        default_before = default_features_by_manifest.get(manifest_path, False)
        default_features_by_manifest[manifest_path] = (
            default_before or enable_default_features
        )
        if (
            len(requested) == requested_before
            and default_features_by_manifest[manifest_path] == default_before
        ):
            return
        if manifest_path not in queued_manifests:
            pending_manifests.append(manifest_path)
            queued_manifests.add(manifest_path)

    def visit_package(manifest_path: Path) -> None:
        manifest_path = manifest_path.resolve()

        manifest = load_manifest(manifest_path)
        package_name = manifest["package"]["name"]
        reachable_packages.add(package_name)
        features = manifest.get("features", {})
        dependencies = local_dependencies(manifest_path, manifest)
        active_features = set(requested_features_by_manifest[manifest_path])
        if default_features_by_manifest[manifest_path] and "default" in features:
            active_features.add("default")
        active_dependencies: dict[str, set[str]] = {
            name: set()
            for name, (_, entry) in dependencies.items()
            if not entry.get("optional", False)
        }

        def activate_dependency(name: str, feature: str | None = None) -> bool:
            if name not in dependencies:
                return False
            was_active = name in active_dependencies
            requested = active_dependencies.setdefault(name, set())
            if feature is None:
                return not was_active
            if feature in requested:
                return False
            requested.add(feature)
            return True

        changed = True
        while changed:
            changed = False
            for feature in tuple(active_features):
                if feature not in features:
                    changed = activate_dependency(feature) or changed
                    continue
                for reference in features[feature]:
                    if reference.startswith("dep:"):
                        changed = activate_dependency(reference.removeprefix("dep:")) or changed
                        continue
                    if "/" not in reference:
                        if reference in features and reference not in active_features:
                            active_features.add(reference)
                            changed = True
                        elif reference in dependencies:
                            changed = activate_dependency(reference) or changed
                        continue
                    dependency, requested_feature = reference.split("/", maxsplit=1)
                    optional_dependency = dependency.endswith("?")
                    dependency = dependency.removesuffix("?")
                    if optional_dependency and dependency not in active_dependencies:
                        continue
                    changed = activate_dependency(dependency, requested_feature) or changed

        reachable_features.update(
            (package_name, feature) for feature in active_features if feature in features
        )

        for dependency, requested in active_dependencies.items():
            dependency_manifest, entry = dependencies[dependency]
            dependency_features = set(entry.get("features", [])) | requested
            request_package(
                dependency_manifest,
                dependency_features,
                entry.get("default-features", True),
            )

    request_package(root_manifest_path, set(root_features), False)
    while pending_manifests:
        manifest_path = pending_manifests.pop()
        queued_manifests.remove(manifest_path)
        visit_package(manifest_path)
    return reachable_features, reachable_packages


class Frameworks03ServerFeatureBoundaryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = tomllib.loads(RUNTIME_MANIFEST.read_text(encoding="utf-8"))
        cls.root_source = RUNTIME_ROOT.read_text(encoding="utf-8")
        cls.rhi_boundary_test = RHI_BOUNDARY_TEST.read_text(encoding="utf-8")
        cls.app_manifest = tomllib.loads(APP_MANIFEST.read_text(encoding="utf-8"))
        cls.app_plugin_groups = APP_PLUGIN_GROUPS.read_text(encoding="utf-8")
        cls.app_plugin_group_resolution = APP_PLUGIN_GROUP_RESOLUTION.read_text(
            encoding="utf-8"
        )
        cls.runtime_core_modules = RUNTIME_CORE_MODULES.read_text(encoding="utf-8")

    def test_domain_features_use_current_vocabulary(self) -> None:
        features = self.manifest["features"]
        for feature in (
            "graphics",
            "text",
            "ui",
            "animation",
            "navigation",
            "script",
            "diagnostic-log",
        ):
            self.assertIn(feature, features)

        server = set(features["target-server"])
        self.assertIn("diagnostic-log", server)
        self.assertTrue(
            server.isdisjoint(
                {
                    "graphics",
                    "text",
                    "ui",
                    "animation",
                    "navigation",
                    "script",
                }
            )
        )

    def test_graphics_backend_and_text_dependencies_are_optional(self) -> None:
        dependencies = self.manifest["dependencies"]
        for dependency in (
            "wgpu",
            "naga",
            "raw-window-handle",
            "pollster",
            "glyphon",
            "swash",
            "taffy",
            "ttf-parser",
            "unicode-linebreak",
            "unicode-normalization",
            "unicode-script",
            "unicode-segmentation",
            "zr_rhi_wgpu",
        ):
            entry = dependencies[dependency]
            self.assertIsInstance(entry, dict, dependency)
            self.assertTrue(entry.get("optional"), dependency)

        neutral_rhi = dependencies["zr_rhi"]
        self.assertIsInstance(neutral_rhi, dict)
        self.assertNotIn("optional", neutral_rhi)

    def test_neutral_rhi_wgpu_dependency_guard_covers_dotted_keys(self) -> None:
        required_contract = (
            "if declares_wgpu_dependency(line) {",
            "fn declares_wgpu_dependency(line: &str) -> bool {",
            '"wgpu.workspace = true"',
            'key.starts_with("wgpu.")',
            '"wgpu-types.workspace = true"',
            "!declares_wgpu_dependency(declaration)",
        )
        for marker in required_contract:
            self.assertIn(marker, self.rhi_boundary_test, marker)

    def test_root_domain_modules_are_cfg_gated(self) -> None:
        required = {
            "animation": "animation",
            "diagnostic_log": "diagnostic-log",
            "dynamic_api": "dynamic-api",
            "graphics": "graphics",
            "navigation": "navigation",
            "render_graph": "graphics",
            "rhi": "graphics",
            "script": "script",
            "ui": "ui",
        }
        for module, feature in required.items():
            self.assertIn(
                f'#[cfg(feature = "{feature}")]\npub mod {module};',
                self.root_source,
                module,
            )
        self.assertNotIn(
            "mod rhi_wgpu;",
            self.root_source,
            "the WGPU implementation must live in the physical zr_rhi_wgpu crate",
        )

    def test_app_target_server_does_not_reenable_client_domains(self) -> None:
        features = self.app_manifest["features"]
        server = set(features["target-server"])
        self.assertIn("diagnostic-log", server)
        self.assertTrue(
            server.isdisjoint(
                {
                    "graphics",
                    "text",
                    "ui",
                    "animation",
                    "navigation",
                    "script",
                    "dynamic-api",
                }
            )
        )
        self.assertIn(
            'resolve_builtin_plugin_group("HeadlessPlugins", RuntimeProfileId::Server, [])',
            self.app_plugin_groups,
        )
        self.assertIn(
            '#[cfg(feature = "ui")]\n    {\n        Some(Arc::new(zircon_runtime::ui::UiModule))',
            self.app_plugin_group_resolution,
        )
        self.assertIn(
            '#[cfg(not(feature = "ui"))]\n    {\n        None\n    }',
            self.app_plugin_group_resolution,
        )

    def test_desktop_profiles_keep_server_and_editor_owners_separate(self) -> None:
        app_features = self.app_manifest["features"]
        forbidden_desktop_features = {
            ("zircon_runtime", "target-server"),
            ("zircon_runtime", "platform-headless"),
            ("zircon_app", "target-server"),
            ("zircon_app", "platform-headless"),
        }

        for package in ("zircon_runtime", "zircon_app"):
            for profile in ("target-client", "target-editor-host"):
                manifest_path = (
                    RUNTIME_MANIFEST if package == "zircon_runtime" else APP_MANIFEST
                )
                reachable_features, _ = reachable_feature_graph(
                    manifest_path,
                    (profile,),
                )
                self.assertTrue(
                    reachable_features.isdisjoint(forbidden_desktop_features),
                    f"{package} {profile} must remain a desktop-only profile",
                )

        self.assertEqual(app_features["default"], ["target-client"])
        _, client_packages = reachable_feature_graph(
            APP_MANIFEST,
            ("target-client",),
        )
        self.assertNotIn("zircon_editor", client_packages)
        _, editor_packages = reachable_feature_graph(
            APP_MANIFEST,
            ("target-editor-host",),
        )
        self.assertIn("zircon_editor", editor_packages)

    def test_feature_closure_follows_activated_local_path_dependencies(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)

            def write_manifest(directory: str, content: str) -> Path:
                manifest_path = root / directory / "Cargo.toml"
                manifest_path.parent.mkdir(parents=True, exist_ok=True)
                manifest_path.write_text(content, encoding="utf-8")
                return manifest_path

            write_manifest(
                ".",
                """
[workspace]

[workspace.dependencies]
zircon_editor = { path = "editor" }
zircon_runtime = { path = "runtime" }
""",
            )
            app_manifest = write_manifest(
                "app",
                """
[package]
name = "fixture_app"
version = "0.1.0"

[features]
target-client = ["catalog/desktop-host"]

[dependencies]
catalog = { path = "../catalog", optional = true, default-features = false }
""",
            )
            write_manifest(
                "catalog",
                """
[package]
name = "fixture_catalog"
version = "0.1.0"

[features]
desktop-host = ["zircon_runtime/target-server"]

[dependencies]
zircon_editor = { workspace = true }
zircon_runtime = { workspace = true }
""",
            )
            write_manifest(
                "editor",
                """
[package]
name = "zircon_editor"
version = "0.1.0"
""",
            )
            write_manifest(
                "runtime",
                """
[package]
name = "zircon_runtime"
version = "0.1.0"

[features]
target-server = []
""",
            )

            features, packages = reachable_feature_graph(
                app_manifest,
                ("target-client",),
            )

        self.assertIn(("zircon_runtime", "target-server"), features)
        self.assertIn("zircon_editor", packages)

    def test_feature_closure_unifies_features_before_resolving_weak_edges(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)

            def write_manifest(directory: str, content: str) -> Path:
                manifest_path = root / directory / "Cargo.toml"
                manifest_path.parent.mkdir(parents=True, exist_ok=True)
                manifest_path.write_text(content, encoding="utf-8")
                return manifest_path

            app_manifest = write_manifest(
                "app",
                """
[package]
name = "fixture_app"
version = "0.1.0"

[features]
target-client = ["left/enable-server", "right/enable-runtime"]

[dependencies]
left = { path = "../left", optional = true, default-features = false }
right = { path = "../right", optional = true, default-features = false }
""",
            )
            write_manifest(
                "left",
                """
[package]
name = "fixture_left"
version = "0.1.0"

[features]
enable-server = ["catalog/enable-server"]

[dependencies]
catalog = { path = "../catalog", optional = true, default-features = false }
""",
            )
            write_manifest(
                "right",
                """
[package]
name = "fixture_right"
version = "0.1.0"

[features]
enable-runtime = ["catalog/enable-runtime"]

[dependencies]
catalog = { path = "../catalog", optional = true, default-features = false }
""",
            )
            write_manifest(
                "catalog",
                """
[package]
name = "fixture_catalog"
version = "0.1.0"

[features]
enable-server = ["zircon_runtime?/target-server"]
enable-runtime = ["zircon_runtime"]

[dependencies]
zircon_runtime = { path = "../runtime", optional = true, default-features = false }
""",
            )
            write_manifest(
                "runtime",
                """
[package]
name = "zircon_runtime"
version = "0.1.0"

[features]
target-server = []
""",
            )

            features, _ = reachable_feature_graph(
                app_manifest,
                ("target-client",),
            )

        self.assertIn(("zircon_runtime", "target-server"), features)

    def test_feature_closure_follows_windows_target_path_dependencies(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)

            def write_manifest(directory: str, content: str) -> Path:
                manifest_path = root / directory / "Cargo.toml"
                manifest_path.parent.mkdir(parents=True, exist_ok=True)
                manifest_path.write_text(content, encoding="utf-8")
                return manifest_path

            app_manifest = write_manifest(
                "app",
                """
[package]
name = "fixture_app"
version = "0.1.0"

[features]
target-client = ["catalog/desktop-host"]

[target.'cfg(windows)'.dependencies]
catalog = { path = "../catalog", optional = true, default-features = false }
""",
            )
            write_manifest(
                "catalog",
                """
[package]
name = "fixture_catalog"
version = "0.1.0"

[features]
desktop-host = ["zircon_runtime/target-server"]

[dependencies]
zircon_runtime = { path = "../runtime", optional = true, default-features = false }
""",
            )
            write_manifest(
                "runtime",
                """
[package]
name = "zircon_runtime"
version = "0.1.0"

[features]
target-server = []
""",
            )

            features, _ = reachable_feature_graph(
                app_manifest,
                ("target-client",),
            )

        self.assertIn(("zircon_runtime", "target-server"), features)

    def test_feature_closure_rejects_unmodeled_local_target_dependencies(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)

            def write_manifest(directory: str, content: str) -> Path:
                manifest_path = root / directory / "Cargo.toml"
                manifest_path.parent.mkdir(parents=True, exist_ok=True)
                manifest_path.write_text(content, encoding="utf-8")
                return manifest_path

            app_manifest = write_manifest(
                "app",
                """
[package]
name = "fixture_app"
version = "0.1.0"

[features]
target-client = ["catalog/desktop-host"]

[target.'cfg(target_pointer_width = "64")'.dependencies]
catalog = { path = "../catalog", optional = true, default-features = false }
""",
            )
            write_manifest(
                "catalog",
                """
[package]
name = "fixture_catalog"
version = "0.1.0"

[features]
desktop-host = []
""",
            )

            with self.assertRaisesRegex(
                AssertionError,
                "cannot resolve local target dependencies",
            ):
                reachable_feature_graph(app_manifest, ("target-client",))

    def test_feature_closure_merges_base_and_windows_local_dependencies(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)

            def write_manifest(directory: str, content: str) -> Path:
                manifest_path = root / directory / "Cargo.toml"
                manifest_path.parent.mkdir(parents=True, exist_ok=True)
                manifest_path.write_text(content, encoding="utf-8")
                return manifest_path

            app_manifest = write_manifest(
                "app",
                """
[package]
name = "fixture_app"
version = "0.1.0"

[features]
target-client = []

[dependencies]
catalog = { path = "../catalog", features = ["enable-server"], default-features = false }

[target.'cfg(windows)'.dependencies]
catalog = { path = "../catalog", optional = true, default-features = false }
""",
            )
            write_manifest(
                "catalog",
                """
[package]
name = "fixture_catalog"
version = "0.1.0"

[features]
enable-server = ["zircon_runtime/target-server"]

[dependencies]
zircon_runtime = { path = "../runtime", optional = true, default-features = false }
""",
            )
            write_manifest(
                "runtime",
                """
[package]
name = "zircon_runtime"
version = "0.1.0"

[features]
target-server = []
""",
            )

            features, _ = reachable_feature_graph(
                app_manifest,
                ("target-client",),
            )

        self.assertIn(("zircon_runtime", "target-server"), features)

    def test_feature_closure_merges_workspace_and_member_features(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)

            def write_manifest(directory: str, content: str) -> Path:
                manifest_path = root / directory / "Cargo.toml"
                manifest_path.parent.mkdir(parents=True, exist_ok=True)
                manifest_path.write_text(content, encoding="utf-8")
                return manifest_path

            write_manifest(
                ".",
                """
[workspace]

[workspace.dependencies]
catalog = { path = "catalog", features = ["enable-server"], default-features = false }
""",
            )
            app_manifest = write_manifest(
                "app",
                """
[package]
name = "fixture_app"
version = "0.1.0"

[features]
target-client = []

[dependencies]
catalog = { workspace = true, features = ["desktop-host"] }
""",
            )
            write_manifest(
                "catalog",
                """
[package]
name = "fixture_catalog"
version = "0.1.0"

[features]
desktop-host = []
enable-server = ["zircon_runtime/target-server"]

[dependencies]
zircon_runtime = { path = "../runtime", optional = true, default-features = false }
""",
            )
            write_manifest(
                "runtime",
                """
[package]
name = "zircon_runtime"
version = "0.1.0"

[features]
target-server = []
""",
            )

            features, _ = reachable_feature_graph(
                app_manifest,
                ("target-client",),
            )

        self.assertIn(("zircon_runtime", "target-server"), features)

    def test_server_runtime_selection_excludes_script_when_script_is_compiled(self) -> None:
        expected_gate = (
            '#[cfg(feature = "script")]\n'
            "    if target != RuntimeTargetMode::ServerRuntime {\n"
            "        modules.push(Arc::new(script::ScriptModule));\n"
            "    }"
        )
        self.assertEqual(self.runtime_core_modules.count(expected_gate), 2)


if __name__ == "__main__":
    unittest.main()
