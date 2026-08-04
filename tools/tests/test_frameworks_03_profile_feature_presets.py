from __future__ import annotations

import json
import subprocess
import sys
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PRESET_PATH = REPO_ROOT / "zircon_runtime" / "runtime-feature-presets.toml"
PRESET_TOOL = REPO_ROOT / "tools" / "runtime-profile-feature-presets.py"
RUNTIME_MANIFEST = REPO_ROOT / "zircon_runtime" / "Cargo.toml"
APP_MANIFEST = REPO_ROOT / "zircon_app" / "Cargo.toml"
DEV_FAST_BUILD = REPO_ROOT / "tools" / "dev-fast-build.ps1"
PROFILE_MATRIX_RUNNER = REPO_ROOT / "tools" / "check-runtime-profile-features.ps1"
CI_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "ci.yml"
RUNTIME_PROFILE_MODULE = (
    REPO_ROOT / "zircon_runtime" / "src" / "plugin" / "runtime_profile" / "feature_presets.rs"
)
RUNTIME_PROFILE_ROOT = REPO_ROOT / "zircon_runtime" / "src" / "plugin" / "runtime_profile.rs"
RUNTIME_PROFILE_DEFAULTS = (
    REPO_ROOT / "zircon_runtime" / "src" / "plugin" / "runtime_profile" / "defaults.rs"
)
RUNTIME_PROFILE_ASSEMBLY = (
    REPO_ROOT / "zircon_runtime" / "src" / "plugin" / "runtime_profile" / "assembly_presets.rs"
)
APP_ENTRY_RUNNER_MODULE = REPO_ROOT / "zircon_app" / "src" / "entry" / "entry_runner" / "mod.rs"
SCENE_PROJECT_IO_MODULE = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "scene"
    / "module"
    / "level_manager_project_io.rs"
)
EXPECTED_PROFILE_IDS = ("minimal", "client2d", "client3d", "editor", "dev", "server")
EXPECTED_BUILTIN_MODULES = (
    {"id": "foundation", "rust_variant": "Foundation"},
    {"id": "log", "rust_variant": "Log"},
    {"id": "tasks", "rust_variant": "Tasks"},
    {"id": "time", "rust_variant": "Time"},
    {"id": "frame_count", "rust_variant": "FrameCount"},
    {"id": "diagnostics_core", "rust_variant": "DiagnosticsCore"},
    {"id": "platform", "rust_variant": "Platform"},
    {"id": "input", "rust_variant": "Input"},
    {"id": "asset", "rust_variant": "Asset"},
    {"id": "scene", "rust_variant": "Scene"},
    {"id": "graphics", "rust_variant": "Graphics", "required_feature": "graphics"},
    {"id": "script", "rust_variant": "Script", "required_feature": "script"},
)


def profile_assembly(profile: dict[str, object]) -> dict[str, object]:
    feature_fields = {"id", "rust_variant", "cargo_feature", "runtime_features", "app_features"}
    return {key: value for key, value in profile.items() if key not in feature_fields}


def cargo_features(path: Path) -> dict[str, list[str]]:
    manifest = tomllib.loads(path.read_text(encoding="utf-8"))
    return manifest["features"]


class Frameworks03ProfileFeaturePresetTests(unittest.TestCase):
    def load_document(self) -> dict[str, object]:
        self.assertTrue(PRESET_PATH.is_file(), f"missing canonical preset source: {PRESET_PATH}")
        return tomllib.loads(PRESET_PATH.read_text(encoding="utf-8"))

    def test_canonical_source_declares_all_six_runtime_profiles(self) -> None:
        document = self.load_document()
        profiles = document["profiles"]
        self.assertEqual(tuple(profile["id"] for profile in profiles), EXPECTED_PROFILE_IDS)
        self.assertEqual(document["schema_version"], 2)

    def test_builtin_module_registry_declares_cfg_predicates(self) -> None:
        document = self.load_document()

        self.assertEqual(tuple(document["builtin_modules"]), EXPECTED_BUILTIN_MODULES)

    def test_runtime_profile_assembly_fields_match_canonical_values(self) -> None:
        profiles = {profile["id"]: profile for profile in self.load_document()["profiles"]}
        server_modules = [
            "foundation",
            "log",
            "tasks",
            "time",
            "frame_count",
            "diagnostics_core",
            "platform",
            "input",
            "asset",
            "scene",
        ]
        client_modules = [*server_modules, "graphics", "script"]

        self.assertEqual(
            profile_assembly(profiles["minimal"]),
            {
                "descriptor_name": "minimal",
                "target_mode": "client_runtime",
                "builtin_modules": [
                    "foundation",
                    "tasks",
                    "time",
                    "frame_count",
                    "diagnostics_core",
                ],
                "minimum_maturity": "core",
                "default_plugins": [],
                "optional_plugins": [],
                "required_capabilities": [
                    "runtime.core.lifecycle",
                    "runtime.core.tasks",
                    "runtime.core.time",
                    "runtime.core.frame_count",
                    "runtime.core.diagnostics",
                ],
                "allow_externalized_required_plugins": False,
            },
        )
        self.assertEqual(
            profile_assembly(profiles["client2d"]),
            {
                "descriptor_name": "client_2d",
                "target_mode": "client_runtime",
                "builtin_modules": client_modules,
                "minimum_maturity": "beta",
                "default_plugins": [
                    {"id": "ui", "required": True},
                    {"id": "sound", "required": True},
                    {"id": "rendering", "required": True},
                    {"id": "texture", "required": False},
                ],
                "optional_plugins": ["tilemap_2d", "particles", "animation"],
                "required_capabilities": [
                    "runtime.core.asset",
                    "runtime.core.scene",
                    "runtime.core.render.base",
                    "runtime.plugin.sound",
                    "runtime.plugin.rendering",
                ],
                "allow_externalized_required_plugins": False,
            },
        )
        self.assertEqual(
            profile_assembly(profiles["client3d"]),
            {
                "descriptor_name": "client_3d",
                "target_mode": "client_runtime",
                "builtin_modules": client_modules,
                "minimum_maturity": "beta",
                "default_plugins": [
                    {"id": "ui", "required": True},
                    {"id": "sound", "required": True},
                    {"id": "rendering", "required": True},
                    {"id": "texture", "required": False},
                ],
                "optional_plugins": [
                    "animation",
                    "ai",
                    "navigation",
                    "particles",
                    "virtual_geometry",
                    "hybrid_gi",
                    "solari",
                ],
                "required_capabilities": [
                    "runtime.core.asset",
                    "runtime.core.scene",
                    "runtime.core.render.base",
                    "runtime.plugin.sound",
                    "runtime.plugin.rendering",
                ],
                "allow_externalized_required_plugins": False,
            },
        )
        self.assertEqual(
            profile_assembly(profiles["editor"]),
            {
                "descriptor_name": "editor",
                "target_mode": "editor_host",
                "builtin_modules": client_modules,
                "minimum_maturity": "beta",
                "default_plugins": [
                    {"id": "ui", "required": True},
                    {"id": "sound", "required": True},
                    {"id": "rendering", "required": True},
                    {"id": "texture", "required": False},
                ],
                "optional_plugins": ["animation", "navigation", "particles", "net"],
                "required_capabilities": [
                    "editor.host.ui_shell",
                    "editor.host.plugin_management",
                ],
                "allow_externalized_required_plugins": False,
            },
        )
        self.assertEqual(
            profile_assembly(profiles["dev"]),
            {
                "descriptor_name": "dev",
                "target_mode": "editor_host",
                "builtin_modules": client_modules,
                "minimum_maturity": "experimental",
                "default_plugins": [
                    {"id": "ui", "required": True},
                    {"id": "sound", "required": True},
                    {"id": "rendering", "required": True},
                    {"id": "texture", "required": False},
                    {"id": "net", "required": False},
                ],
                "optional_plugins": [
                    "ai",
                    "animation",
                    "navigation",
                    "particles",
                    "virtual_geometry",
                    "hybrid_gi",
                    "solari",
                ],
                "required_capabilities": [
                    "runtime.core.diagnostics",
                    "editor.host.plugin_management",
                ],
                "allow_externalized_required_plugins": False,
            },
        )
        self.assertEqual(
            profile_assembly(profiles["server"]),
            {
                "descriptor_name": "server",
                "target_mode": "server_runtime",
                "builtin_modules": server_modules,
                "minimum_maturity": "beta",
                "default_plugins": [{"id": "net", "required": False}],
                "optional_plugins": ["ai", "physics", "animation", "navigation"],
                "required_capabilities": ["runtime.core.lifecycle", "runtime.core.scene"],
                "allow_externalized_required_plugins": False,
            },
        )

    def test_runtime_profile_wiring_hard_cuts_handwritten_defaults(self) -> None:
        runtime_profile_source = RUNTIME_PROFILE_ROOT.read_text(encoding="utf-8")

        self.assertFalse(RUNTIME_PROFILE_DEFAULTS.exists())
        self.assertNotIn("mod defaults;", runtime_profile_source)
        self.assertIn("mod assembly_presets;", runtime_profile_source)
        self.assertTrue(RUNTIME_PROFILE_ASSEMBLY.is_file())
        assembly_source = RUNTIME_PROFILE_ASSEMBLY.read_text(encoding="utf-8")
        self.assertIn(
            "runtime_profile_assembly_presets_generated.rs",
            assembly_source,
        )
        self.assertIn(
            "generated_runtime_profile_assembly_preset_for(id)",
            assembly_source,
        )
        self.assertNotIn(".find(|preset|", assembly_source)

    def test_profile_members_match_runtime_and_app_cargo_features(self) -> None:
        profiles = self.load_document()["profiles"]
        runtime_features = cargo_features(RUNTIME_MANIFEST)
        app_features = cargo_features(APP_MANIFEST)

        for profile in profiles:
            cargo_feature = profile["cargo_feature"]
            self.assertIn(cargo_feature, runtime_features, profile["id"])
            self.assertIn(cargo_feature, app_features, profile["id"])
            expected_runtime_members = (
                [cargo_feature]
                if cargo_feature == "core-min"
                else runtime_features[cargo_feature]
            )
            expected_app_members = app_features[cargo_feature]
            self.assertEqual(profile["runtime_features"], expected_runtime_members, profile["id"])
            self.assertEqual(profile["app_features"], expected_app_members, profile["id"])

    def test_duplicate_cargo_presets_cannot_drift_between_logical_profiles(self) -> None:
        profiles = self.load_document()["profiles"]
        by_cargo_feature: dict[str, tuple[tuple[str, ...], tuple[str, ...]]] = {}
        for profile in profiles:
            payload = (
                tuple(profile["runtime_features"]),
                tuple(profile["app_features"]),
            )
            previous = by_cargo_feature.setdefault(profile["cargo_feature"], payload)
            self.assertEqual(previous, payload, profile["id"])

    def test_runtime_tools_declare_their_real_feature_requirements(self) -> None:
        manifest = tomllib.loads(RUNTIME_MANIFEST.read_text(encoding="utf-8"))
        binaries = {binary["name"]: binary for binary in manifest["bin"]}

        self.assertEqual(
            binaries["zircon_host_reflection_docs"]["required-features"],
            ["script"],
        )
        self.assertEqual(
            binaries["zircon_shader_prewarm"]["required-features"],
            ["dynamic-api"],
        )
        self.assertEqual(
            binaries["zircon_shader_ide_env"]["required-features"],
            ["graphics"],
        )

    def test_minimal_app_does_not_mount_diagnostic_log_startup_parsing(self) -> None:
        source = APP_ENTRY_RUNNER_MODULE.read_text(encoding="utf-8")

        self.assertIn(
            '#[cfg(feature = "diagnostic-log")]\nmod diagnostic_log_args;',
            source,
        )

    def test_scene_artifact_failure_logging_respects_diagnostic_log_feature(self) -> None:
        source = SCENE_PROJECT_IO_MODULE.read_text(encoding="utf-8")

        self.assertIn(
            '#[cfg(feature = "diagnostic-log")]\n    crate::diagnostic_log::write_log(',
            source,
        )

    def test_tool_exports_profile_and_matrix_from_canonical_source(self) -> None:
        self.assertTrue(PRESET_TOOL.is_file(), f"missing preset tool: {PRESET_TOOL}")
        profile = subprocess.run(
            [sys.executable, str(PRESET_TOOL), "feature", "client3d"],
            cwd=REPO_ROOT,
            check=True,
            capture_output=True,
            text=True,
        )
        self.assertEqual(profile.stdout.strip(), "target-client")

        matrix = subprocess.run(
            [sys.executable, str(PRESET_TOOL), "matrix"],
            cwd=REPO_ROOT,
            check=True,
            capture_output=True,
            text=True,
        )
        payload = json.loads(matrix.stdout)
        self.assertEqual(
            tuple(entry["profile"] for entry in payload["include"]),
            EXPECTED_PROFILE_IDS,
        )

    def test_rust_tools_and_ci_consume_the_canonical_profile_source(self) -> None:
        self.assertTrue(RUNTIME_PROFILE_MODULE.is_file())
        runtime_source = RUNTIME_PROFILE_MODULE.read_text(encoding="utf-8")
        dev_script = DEV_FAST_BUILD.read_text(encoding="utf-8")
        profile_runner = PROFILE_MATRIX_RUNNER.read_text(encoding="utf-8")
        workflow = CI_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("runtime_profile_feature_presets_generated.rs", runtime_source)
        self.assertIn("runtime-profile-feature-presets.py", dev_script)
        self.assertIn("runtime-profile-feature-presets.py", profile_runner)
        self.assertIn(" matrix", profile_runner)
        self.assertNotIn('"client"', profile_runner)
        self.assertNotIn('"client" { return "target-client" }', dev_script)
        self.assertIn("runtime-profile-feature-matrix-plan", workflow)
        self.assertIn("runtime-profile-feature-matrix", workflow)
        self.assertIn("runtime-domain-feature-matrix", workflow)
        for feature in (
            "ai-contracts",
            "animation",
            "diagnostic-log",
            "dynamic-api",
            "graphics",
            "navigation",
            "net-contracts",
            "physics-contracts",
            "script",
            "sound-contracts",
            "text",
            "ui",
        ):
            self.assertIn(f"- {feature}", workflow)
        self.assertIn("runtime-profile-feature-presets.py matrix", workflow)
        self.assertIn("fromJSON(needs.runtime-profile-feature-matrix-plan.outputs.matrix)", workflow)


if __name__ == "__main__":
    unittest.main()
