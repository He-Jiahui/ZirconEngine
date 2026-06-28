import unittest
from pathlib import Path

from tools.zircon_build_shader_prewarm import (
    build_shader_prewarm_command,
    validate_shader_prewarm_command_contract,
)


class ZirconBuildShaderPrewarmCommandContractTests(unittest.TestCase):
    def test_full_staged_wgpu_handoff_keeps_generated_registries_and_roots(self):
        config = _FakePrewarmConfig()
        config.validate_wgpu_shaders = True
        config.shader_quality_tiers = ("high", "ultra")
        config.shader_geometry_sources = ("static", "skinned")
        config.shader_geometry_source_ids = ("custom:gpu-driven=4",)
        config.shader_shading_model_ids = ("custom:toon=16",)
        config.plugins = (
            _FakePluginPackage(
                asset_roots=(Path("plugins") / "toon" / "assets",),
                shader_geometry_source_ids=("custom:virtual_geometry=5",),
                shader_shading_model_ids=("custom:subsurface=17",),
            ),
        )

        command = build_shader_prewarm_command(config)

        validate_shader_prewarm_command_contract(config, command)
        self.assertIn("--validate-wgpu-modules", command)
        self.assertEqual(
            (
                str(config.engine_root / "assets"),
                str(Path("plugins") / "toon" / "assets"),
            ),
            _flag_values(command, "--asset-root"),
        )
        self.assertEqual(
            (str(config.shader_prewarm_permutation_registry_path),),
            _flag_values(command, "--shader-permutation-registry"),
        )
        self.assertEqual(
            (str(config.shader_prewarm_resource_registry_path),),
            _flag_values(command, "--export-resource-registry"),
        )
        self.assertEqual(
            (str(config.shader_prewarm_cache_root),),
            _flag_values(command, "--cache-dir"),
        )
        self.assertEqual(
            (str(config.shader_prewarm_report_path),),
            _flag_values(command, "--report"),
        )
        self.assertEqual(
            ("custom:gpu-driven=4",),
            _flag_values(command, "--geometry-source-id"),
        )
        self.assertEqual(
            ("custom:toon=16",),
            _flag_values(command, "--shading-model-id"),
        )

    def test_command_contract_rejects_missing_wgpu_validation_flag(self):
        config = _FakePrewarmConfig()
        config.validate_wgpu_shaders = True
        command = build_shader_prewarm_command(config)
        command.remove("--validate-wgpu-modules")

        with self.assertRaisesRegex(RuntimeError, "WGPU module validation"):
            validate_shader_prewarm_command_contract(config, command)


def _flag_values(command: list[str], flag: str) -> tuple[str, ...]:
    return tuple(command[index + 1] for index, value in enumerate(command) if value == flag)


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
    targets_root = Path("target") / "prewarm-command-contract-test"
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
        asset_roots: tuple[Path, ...] = (),
        shader_geometry_source_ids: tuple[str, ...] = (),
        shader_shading_model_ids: tuple[str, ...] = (),
    ):
        self.asset_roots = asset_roots
        self.shader_geometry_source_ids = shader_geometry_source_ids
        self.shader_shading_model_ids = shader_shading_model_ids


if __name__ == "__main__":
    unittest.main()
