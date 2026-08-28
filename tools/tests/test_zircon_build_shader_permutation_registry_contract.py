import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools import zircon_build
from tools.zircon_build_shader_prewarm import (
    validate_shader_permutation_registry_export_contract,
)


class ZirconBuildShaderPermutationRegistryContractTests(unittest.TestCase):
    def test_validate_generated_registry_requires_selected_plugin_ids(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_permutation_registry.json"
            registry_path.write_text(
                json.dumps(
                    {
                        "geometry_source_ids": [
                            {"token": "custom:virtual_geometry", "id": 4}
                        ],
                        "shading_model_ids": [],
                    }
                ),
                encoding="utf-8",
            )
            config = _FakePrewarmConfig()
            config.plugins = (
                _FakePluginPackage(
                    shader_geometry_source_ids=("custom:virtual_geometry=4",),
                    shader_shading_model_ids=("custom:toon=16",),
                ),
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "missing selected shader shading model ids: custom:toon=16",
            ):
                validate_shader_permutation_registry_export_contract(
                    registry_path,
                    config=config,
                )

    def test_validate_generated_registry_requires_selected_plugin_shader_modules(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            registry_path = Path(temp_dir) / "shader_permutation_registry.json"
            registry_path.write_text(
                json.dumps(
                    {
                        "geometry_source_ids": [],
                        "shading_model_ids": [],
                        "shader_modules": [],
                    }
                ),
                encoding="utf-8",
            )
            config = _FakePrewarmConfig()
            config.plugins = (
                _FakePluginPackage(
                    shader_modules=(
                        {
                            "import_path": "custom::toon::noise",
                            "content_hash": "b" * 64,
                        },
                    ),
                ),
            )

            with self.assertRaisesRegex(
                RuntimeError,
                "missing selected shader modules: custom::toon::noise",
            ):
                validate_shader_permutation_registry_export_contract(
                    registry_path,
                    config=config,
                )

    @patch.object(zircon_build, "managed_cargo_environment")
    def test_prewarm_shaders_validates_generated_registry_before_run(
        self, managed_cargo_environment
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            config = _FakePrewarmConfig()
            config.engine_root = Path(temp_dir) / "ZirconEngine"
            config.shader_geometry_source_ids = ("custom:gpu-driven=4",)
            events: list[str] = []
            managed_environment = {"ZR_TEST_MANAGED_ENV": "1"}
            managed_cargo_environment.return_value = managed_environment

            def fake_run(command, cwd, check, env):
                self.assertFalse(check)
                self.assertEqual(config.repo_root, cwd)
                self.assertIs(managed_environment, env)
                events.append("run")
                return subprocess.CompletedProcess(command, 0)

            def fake_validate_permutation_registry(registry_path, *, config):
                events.append(f"permutation:{registry_path}:{config is config}")

            with patch.object(zircon_build.subprocess, "run", side_effect=fake_run):
                with patch.object(
                    zircon_build,
                    "print_shader_prewarm_report_dimensions",
                    side_effect=lambda report_path: events.append(f"summary:{report_path}"),
                ):
                    with patch.object(
                        zircon_build,
                        "validate_shader_permutation_registry_export_contract",
                        side_effect=fake_validate_permutation_registry,
                    ):
                        with patch.object(
                            zircon_build,
                            "validate_staged_shader_prewarm_acceptance_contract",
                            side_effect=lambda actual_config: events.append(
                                f"acceptance:{actual_config is config}"
                            ),
                        ):
                            zircon_build.prewarm_shaders(config)

            managed_cargo_environment.assert_called_once_with(
                config.targets_root / "shader_prewarm", config.targets_root
            )
            self.assertEqual(
                [
                    f"permutation:{config.shader_prewarm_permutation_registry_path}:True",
                    "run",
                    f"summary:{config.shader_prewarm_report_path}",
                    "acceptance:True",
                ],
                events,
            )

    @patch.object(zircon_build, "managed_cargo_environment")
    def test_prewarm_shaders_passes_selected_custom_ids_to_acceptance_contract(
        self, managed_cargo_environment
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            config = _FakePrewarmConfig()
            config.engine_root = Path(temp_dir) / "ZirconEngine"
            config.shader_geometry_source_ids = ("custom:gpu-driven=4",)
            config.plugins = (
                _FakePluginPackage(
                    shader_geometry_source_ids=("custom:virtual_geometry=5",),
                    shader_shading_model_ids=("toon=16",),
                ),
            )
            captured_acceptance_config = []
            managed_environment = {"ZR_TEST_MANAGED_ENV": "1"}
            managed_cargo_environment.return_value = managed_environment

            def fake_run(command, cwd, check, env):
                self.assertFalse(check)
                self.assertEqual(config.repo_root, cwd)
                self.assertIs(managed_environment, env)
                return subprocess.CompletedProcess(command, 0)

            def fake_acceptance_contract(actual_config):
                captured_acceptance_config.append(actual_config)

            with patch.object(zircon_build.subprocess, "run", side_effect=fake_run):
                with patch.object(
                    zircon_build,
                    "print_shader_prewarm_report_dimensions",
                    side_effect=lambda report_path: None,
                ):
                    with patch.object(
                        zircon_build,
                        "validate_shader_permutation_registry_export_contract",
                        side_effect=lambda *args, **kwargs: None,
                    ):
                        with patch.object(
                            zircon_build,
                            "validate_staged_shader_prewarm_acceptance_contract",
                            side_effect=fake_acceptance_contract,
                        ):
                            zircon_build.prewarm_shaders(config)

            managed_cargo_environment.assert_called_once_with(
                config.targets_root / "shader_prewarm", config.targets_root
            )
            self.assertEqual([config], captured_acceptance_config)
            self.assertEqual(("custom:gpu-driven=4",), config.shader_geometry_source_ids)
            self.assertEqual(
                ("custom:virtual_geometry=5",),
                config.plugins[0].shader_geometry_source_ids,
            )
            self.assertEqual(("toon=16",), config.plugins[0].shader_shading_model_ids)


class _FakePrewarmConfig:
    cargo = "cargo"
    dry_run = False
    engine_root = Path("stage") / "ZirconEngine"
    jobs = "1"
    locked = True
    mode = "debug"
    plugins: tuple[object, ...] = ()
    repo_root = Path(".")
    shader_geometry_source_ids: tuple[str, ...] = ()
    shader_geometry_sources = ("static",)
    shader_permutation_registries: tuple[Path, ...] = ()
    shader_quality_tiers = ("medium",)
    shader_resource_registry = None
    shader_shading_model_ids: tuple[str, ...] = ()
    targets_root = Path("target") / "prewarm-registry-contract-test"
    validate_wgpu_shaders = False

    @property
    def shader_prewarm_cache_root(self) -> Path:
        return self.engine_root / "cache" / "shader_variants"

    @property
    def shader_prewarm_report_path(self) -> Path:
        return self.engine_root / "cache" / "shader_variants_report.json"

    @property
    def shader_prewarm_resource_registry_path(self) -> Path:
        return self.engine_root / "cache" / "shader_resource_records.json"

    @property
    def shader_prewarm_permutation_registry_path(self) -> Path:
        return self.engine_root / "cache" / "shader_permutation_registry.json"

    def feature_arg_for_target(self, target: str) -> str:
        self._target = target
        return "target-server"


class _FakePluginPackage:
    def __init__(
        self,
        shader_geometry_source_ids: tuple[str, ...] = (),
        shader_shading_model_ids: tuple[str, ...] = (),
        shader_modules: tuple[dict[str, object], ...] = (),
    ):
        self.shader_geometry_source_ids = shader_geometry_source_ids
        self.shader_shading_model_ids = shader_shading_model_ids
        self.shader_modules = shader_modules


if __name__ == "__main__":
    unittest.main()
