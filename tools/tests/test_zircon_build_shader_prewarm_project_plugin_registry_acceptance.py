import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.zircon_build_shader_prewarm_acceptance import (
    validate_staged_shader_prewarm_acceptance_contract,
)


class ZirconBuildShaderPrewarmProjectPluginRegistryAcceptanceTests(unittest.TestCase):
    def test_acceptance_contract_requires_registry_source_for_project_plugin_auto_export(
        self,
    ):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(
                config,
                source_label="builtin://shader/pbr.wgsl",
            ).cleanup
        )
        config.shader_asset_roots = (Path("Project") / "assets",)
        config.plugins = (
            _FakePluginPackage(
                asset_roots=(Path("plugins") / "toon" / "assets",),
            ),
        )
        config.shader_prewarm_resource_registry_path.write_text(
            json.dumps(
                {
                    "resources": [
                        _shader_resource_record("res://materials/prewarm-test.wgsl")
                    ]
                }
            ),
            encoding="utf-8",
        )

        with patch(
            "tools.zircon_build_shader_prewarm_acceptance."
            "validate_shader_prewarm_report_contract",
        ):
            with patch(
                "tools.zircon_build_shader_prewarm_acceptance."
                "validate_shader_prewarm_cache_artifact_contract",
            ):
                with self.assertRaisesRegex(
                    RuntimeError,
                    "registry-backed report source",
                ):
                    validate_staged_shader_prewarm_acceptance_contract(config)

    def test_acceptance_contract_accepts_registry_source_for_project_plugin_auto_export(
        self,
    ):
        source_label = "res://materials/prewarm-test.wgsl"
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(config, source_label=source_label).cleanup
        )
        config.shader_asset_roots = (Path("Project") / "assets",)
        config.plugins = (
            _FakePluginPackage(
                asset_roots=(Path("plugins") / "toon" / "assets",),
            ),
        )
        config.shader_prewarm_resource_registry_path.write_text(
            json.dumps({"resources": [_shader_resource_record(source_label)]}),
            encoding="utf-8",
        )

        with patch(
            "tools.zircon_build_shader_prewarm_acceptance."
            "validate_shader_prewarm_report_contract",
        ):
            with patch(
                "tools.zircon_build_shader_prewarm_acceptance."
                "validate_shader_prewarm_cache_artifact_contract",
            ):
                validate_staged_shader_prewarm_acceptance_contract(config)


class _FakePrewarmConfig:
    engine_root = Path("stage") / "ZirconEngine"
    plugins: tuple[object, ...] = ()
    shader_asset_roots: tuple[Path, ...] = ()
    shader_geometry_source_ids: tuple[str, ...] = ()
    shader_geometry_sources = ("static",)
    shader_quality_tiers = ("medium",)
    shader_resource_registry = None
    shader_shading_model_ids: tuple[str, ...] = ()
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


class _FakePluginPackage:
    def __init__(self, asset_roots: tuple[Path, ...] = ()):
        self.asset_roots = asset_roots


def _write_acceptance_report(
    config: _FakePrewarmConfig,
    *,
    source_label: str,
) -> tempfile.TemporaryDirectory:
    temp_dir = tempfile.TemporaryDirectory()
    config.engine_root = Path(temp_dir.name) / "ZirconEngine"
    config.shader_prewarm_report_path.parent.mkdir(parents=True)
    config.shader_prewarm_report_path.write_text(
        json.dumps(
            {
                "requested_count": 1,
                "written_count": 1,
                "failed_count": 0,
                "written_variants": [_written_variant(source_label)],
                "source_provenance": _source_provenance(source_label),
            }
        ),
        encoding="utf-8",
    )
    return temp_dir


def _written_variant(source_label: str) -> dict[str, object]:
    return {
        "cache_hash": "a" * 64,
        "canonical_string": "pass=forward|geometry=0|shading=0",
        "source_label": source_label,
        "template_revision": "zr-material-template-v1",
        "naga_version": "test-naga",
        "wgpu_version": "test-wgpu",
    }


def _source_provenance(source_label: str) -> dict[str, object]:
    return {
        "source_count": 1,
        "variant_count": 1,
        "sources": {
            f"{source_label}#source-a#template-a": {
                "source_label": source_label,
                "source_hash": "source-a",
                "template_revision": "template-a",
                "requested_count": 1,
                "written_count": 1,
                "failed_count": 0,
            }
        },
    }


def _shader_resource_record(locator: str) -> dict[str, object]:
    return {
        "id": "00000000-0000-0000-0000-000000000001",
        "kind": "Shader",
        "primary_locator": locator,
        "artifact_locator": None,
        "revision": 1,
        "state": "Ready",
        "dependency_ids": [],
        "diagnostics": [],
        "source_hash": "source-a",
        "importer_id": "zircon_shader_importer",
        "importer_version": 1,
        "config_hash": "shader-config-hash",
    }


if __name__ == "__main__":
    unittest.main()
