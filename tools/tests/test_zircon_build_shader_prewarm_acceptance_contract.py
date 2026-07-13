import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.zircon_build_shader_prewarm_acceptance import (
    validate_staged_shader_prewarm_acceptance_contract,
)


class ZirconBuildShaderPrewarmAcceptanceContractTests(unittest.TestCase):
    def test_acceptance_contract_validates_report_cache_and_exported_registry(self):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(
                config,
                requested_count=2,
                written_count=2,
                written_variants=(
                    _written_variant(cache_hash=_CACHE_HASH),
                    _written_variant(
                        cache_hash=_SECOND_CACHE_HASH,
                        canonical_string="pass=gbuffer|geometry=0|shading=0",
                    ),
                ),
            ).cleanup
        )
        config.validate_wgpu_shaders = True
        config.shader_quality_tiers = ("medium", "high")
        config.shader_geometry_sources = ("static", "skinned")
        config.shader_geometry_source_ids = ("custom:gpu-driven=4",)
        config.shader_shading_model_ids = ("custom:toon=16",)
        config.plugins = (
            _FakePluginPackage(
                shader_geometry_source_ids=("custom:virtual-geometry=5",),
                shader_shading_model_ids=("custom:subsurface=17",),
            ),
        )
        events: list[str] = []

        def fake_validate_report(
            report_path,
            *,
            require_wgpu_module_validation,
            require_wgpu_pipeline_validation,
            require_source_provenance,
            expected_pass_types,
            expected_quality_tiers,
            expected_geometry_sources,
            expected_geometry_source_ids,
            expected_shading_model_ids,
        ):
            events.append(
                f"report:{report_path}:"
                f"{require_wgpu_module_validation}:"
                f"{require_wgpu_pipeline_validation}:{require_source_provenance}:"
                f"{expected_pass_types}:{expected_quality_tiers}:"
                f"{expected_geometry_sources}:"
                f"{expected_geometry_source_ids}:{expected_shading_model_ids}"
            )

        def fake_validate_cache(
            cache_root,
            *,
            report_path,
            expected_pass_types,
            expected_quality_tiers,
            expected_geometry_sources,
            expected_geometry_source_ids,
            expected_shading_model_ids,
        ):
            events.append(
                f"cache:{cache_root}:{report_path}:"
                f"{expected_pass_types}:{expected_quality_tiers}:"
                f"{expected_geometry_sources}:"
                f"{expected_geometry_source_ids}:{expected_shading_model_ids}"
            )

        def fake_validate_registry(
            registry_path,
            *,
            report_path,
            require_usable_shader_records=False,
            require_report_registry_backed_sources=False,
        ):
            events.append(
                f"registry:{registry_path}:{report_path}:"
                f"{require_usable_shader_records}:"
                f"{require_report_registry_backed_sources}"
            )

        with patch(
            "tools.zircon_build_shader_prewarm_acceptance."
            "validate_shader_prewarm_report_contract",
            side_effect=fake_validate_report,
        ):
            with patch(
                "tools.zircon_build_shader_prewarm_acceptance."
                "validate_shader_prewarm_cache_artifact_contract",
                side_effect=fake_validate_cache,
            ):
                with patch(
                    "tools.zircon_build_shader_prewarm_acceptance."
                    "validate_shader_resource_registry_export_contract",
                    side_effect=fake_validate_registry,
                ):
                    validate_staged_shader_prewarm_acceptance_contract(config)

        expected_geometry_ids = (
            "custom:gpu-driven=4",
            "custom:virtual-geometry=5",
        )
        expected_shading_ids = ("custom:toon=16", "custom:subsurface=17")
        expected_pass_types = (
            "forward",
            "gbuffer",
            "depth_prepass",
            "shadow",
            "velocity",
            "taa_reactive_mask",
        )
        self.assertEqual(
            [
                f"report:{config.shader_prewarm_report_path}:True:False:True:"
                f"{expected_pass_types}:{config.shader_quality_tiers}:"
                f"{config.shader_geometry_sources}:"
                f"{expected_geometry_ids}:{expected_shading_ids}",
                f"cache:{config.shader_prewarm_cache_root}:"
                f"{config.shader_prewarm_report_path}:"
                f"{expected_pass_types}:{config.shader_quality_tiers}:"
                f"{config.shader_geometry_sources}:"
                f"{expected_geometry_ids}:{expected_shading_ids}",
                f"registry:{config.shader_prewarm_resource_registry_path}:"
                f"{config.shader_prewarm_report_path}:False:False",
            ],
            events,
        )

    def test_acceptance_contract_requires_pipeline_validation_when_enabled(self):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(
                config,
                requested_count=1,
                written_count=1,
                written_variants=(_written_variant(),),
            ).cleanup
        )
        config.validate_wgpu_pipelines = True
        events: list[str] = []

        def fake_validate_report(
            report_path,
            *,
            require_wgpu_module_validation,
            require_wgpu_pipeline_validation,
            require_source_provenance,
            expected_pass_types,
            expected_quality_tiers,
            expected_geometry_sources,
            expected_geometry_source_ids,
            expected_shading_model_ids,
        ):
            events.append(
                f"report:{require_wgpu_module_validation}:"
                f"{require_wgpu_pipeline_validation}:{require_source_provenance}"
            )

        with patch(
            "tools.zircon_build_shader_prewarm_acceptance."
            "validate_shader_prewarm_report_contract",
            side_effect=fake_validate_report,
        ):
            with patch(
                "tools.zircon_build_shader_prewarm_acceptance."
                "validate_shader_prewarm_cache_artifact_contract",
            ):
                with patch(
                    "tools.zircon_build_shader_prewarm_acceptance."
                    "validate_shader_resource_registry_export_contract",
                ):
                    validate_staged_shader_prewarm_acceptance_contract(config)

        self.assertIn("report:False:True:True", events)

    def test_acceptance_contract_validates_explicit_registry_against_report(self):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(
                config,
                requested_count=1,
                written_count=1,
                written_variants=(_written_variant(),),
            ).cleanup
        )
        config.shader_resource_registry = Path("Project") / "shader_resource_records.json"
        events: list[str] = []

        def fake_validate_registry(
            registry_path,
            *,
            report_path,
            require_usable_shader_records=False,
            require_report_registry_backed_sources=False,
        ):
            events.append(
                f"registry:{registry_path}:{report_path}:"
                f"{require_usable_shader_records}:"
                f"{require_report_registry_backed_sources}"
            )

        with patch(
            "tools.zircon_build_shader_prewarm_acceptance."
            "validate_shader_prewarm_report_contract",
            side_effect=lambda *args, **kwargs: events.append("report"),
        ):
            with patch(
                "tools.zircon_build_shader_prewarm_acceptance."
                "validate_shader_prewarm_cache_artifact_contract",
                side_effect=lambda *args, **kwargs: events.append("cache"),
            ):
                with patch(
                    "tools.zircon_build_shader_prewarm_acceptance."
                    "validate_shader_resource_registry_export_contract",
                    side_effect=fake_validate_registry,
                ):
                    validate_staged_shader_prewarm_acceptance_contract(config)

        self.assertEqual(
            [
                "report",
                "cache",
                f"registry:{config.shader_resource_registry}:"
                f"{config.shader_prewarm_report_path}:False:False",
            ],
            events,
        )

    def test_acceptance_contract_requires_usable_records_for_project_plugin_auto_export(
        self,
    ):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(
                config,
                requested_count=1,
                written_count=1,
                written_variants=(
                    _written_variant(source_label="builtin://shader/pbr.wgsl"),
                ),
                source_provenance=_source_provenance("builtin://shader/pbr.wgsl"),
            ).cleanup
        )
        config.shader_asset_roots = (Path("Project") / "assets",)
        config.plugins = (
            _FakePluginPackage(
                asset_roots=(Path("plugins") / "toon" / "assets",),
            ),
        )
        config.shader_prewarm_resource_registry_path.write_text(
            json.dumps({"resources": []}),
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
                    "usable Shader ResourceRecord",
                ):
                    validate_staged_shader_prewarm_acceptance_contract(config)

    def test_acceptance_contract_rejects_explicit_registry_without_ready_revision(
        self,
    ):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(
                config,
                requested_count=1,
                written_count=1,
                written_variants=(_written_variant(),),
                source_provenance=_source_provenance("res://materials/prewarm-test.wgsl"),
            ).cleanup
        )
        config.shader_resource_registry = config.engine_root / "live_resource_records.json"
        config.shader_resource_registry.write_text(
            json.dumps(
                {
                    "resources": [
                        _shader_resource_record(
                            "res://materials/prewarm-test.wgsl",
                            revision=0,
                        )
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
                    "usable shader ResourceRecord revisions",
                ):
                    validate_staged_shader_prewarm_acceptance_contract(config)

    def test_acceptance_contract_rejects_runtime_fallback_layout_drift(self):
        config = _FakePrewarmConfig()
        config.shader_prewarm_cache_root_override = (
            Path("stage") / "ZirconEngine" / ".zircon/cache" / "shader_variants"
        )

        with self.assertRaisesRegex(
            RuntimeError,
            "staged shader prewarm cache root must match runtime fallback root",
        ):
            validate_staged_shader_prewarm_acceptance_contract(config)

    def test_acceptance_contract_accepts_runtime_fallback_layout(self):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(
                config,
                requested_count=1,
                written_count=1,
                written_variants=(_written_variant(),),
            ).cleanup
        )
        events: list[str] = []

        with patch(
            "tools.zircon_build_shader_prewarm_acceptance."
            "validate_shader_prewarm_report_contract",
            side_effect=lambda *args, **kwargs: events.append("report"),
        ):
            with patch(
                "tools.zircon_build_shader_prewarm_acceptance."
                "validate_shader_prewarm_cache_artifact_contract",
                side_effect=lambda *args, **kwargs: events.append("cache"),
            ):
                with patch(
                    "tools.zircon_build_shader_prewarm_acceptance."
                    "validate_shader_resource_registry_export_contract",
                    side_effect=lambda *args, **kwargs: events.append("registry"),
                ):
                    validate_staged_shader_prewarm_acceptance_contract(config)

        self.assertEqual(["report", "cache", "registry"], events)

    def test_acceptance_contract_rejects_empty_success_report(self):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(config, requested_count=0, written_count=0).cleanup
        )

        with self.assertRaisesRegex(
            RuntimeError,
            "staged shader prewarm acceptance requires written variants",
        ):
            validate_staged_shader_prewarm_acceptance_contract(config)

    def test_acceptance_contract_rejects_failed_success_report(self):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(
                config,
                requested_count=2,
                written_count=1,
                failed_count=1,
            ).cleanup
        )

        with self.assertRaisesRegex(
            RuntimeError,
            "staged shader prewarm acceptance requires zero failed variants",
        ):
            validate_staged_shader_prewarm_acceptance_contract(config)

    def test_acceptance_contract_rejects_partial_written_success_report(self):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(
                config,
                requested_count=2,
                written_count=1,
                written_variants=(_written_variant(),),
            ).cleanup
        )

        with self.assertRaisesRegex(
            RuntimeError,
            "staged shader prewarm acceptance requires all requested variants written",
        ):
            validate_staged_shader_prewarm_acceptance_contract(config)

    def test_acceptance_contract_requires_written_variant_identity(self):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(config, requested_count=1, written_count=1).cleanup
        )

        with self.assertRaisesRegex(
            RuntimeError,
            "staged shader prewarm acceptance requires written cache variants",
        ):
            validate_staged_shader_prewarm_acceptance_contract(config)

    def test_acceptance_contract_rejects_incomplete_written_variant_identity(self):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(
                config,
                requested_count=1,
                written_count=1,
                written_variants=(
                    {
                        "cache_hash": _CACHE_HASH,
                        "canonical_string": "pass=forward|geometry=0|shading=0",
                    },
                ),
            ).cleanup
        )

        with self.assertRaisesRegex(
            RuntimeError,
            "staged shader prewarm acceptance requires written cache variant identity",
        ):
            validate_staged_shader_prewarm_acceptance_contract(config)

    def test_acceptance_contract_requires_written_variant_source_label_identity(self):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(
                config,
                requested_count=1,
                written_count=1,
                written_variants=(
                    _written_variant(include_source_label=False),
                ),
            ).cleanup
        )

        with self.assertRaisesRegex(
            RuntimeError,
            "staged shader prewarm acceptance requires written cache variant identity",
        ):
            validate_staged_shader_prewarm_acceptance_contract(config)

    def test_acceptance_contract_rejects_blank_written_variant_source_label(self):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(
                config,
                requested_count=1,
                written_count=1,
                written_variants=(
                    _written_variant(source_label="   "),
                ),
            ).cleanup
        )

        with patch(
            "tools.zircon_build_shader_prewarm_acceptance."
            "validate_shader_prewarm_report_contract",
        ):
            with patch(
                "tools.zircon_build_shader_prewarm_acceptance."
                "validate_shader_prewarm_cache_artifact_contract",
            ):
                with patch(
                    "tools.zircon_build_shader_prewarm_acceptance."
                    "validate_shader_resource_registry_export_contract",
                ):
                    with self.assertRaisesRegex(
                        RuntimeError,
                        "staged shader prewarm acceptance requires written cache variant identity",
                    ):
                        validate_staged_shader_prewarm_acceptance_contract(config)

    def test_acceptance_contract_rejects_untrimmed_written_variant_source_label(
        self,
    ):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(
                config,
                requested_count=1,
                written_count=1,
                written_variants=(
                    _written_variant(source_label=" res://materials/prewarm-test.wgsl "),
                ),
            ).cleanup
        )

        with patch(
            "tools.zircon_build_shader_prewarm_acceptance."
            "validate_shader_prewarm_report_contract",
        ):
            with patch(
                "tools.zircon_build_shader_prewarm_acceptance."
                "validate_shader_prewarm_cache_artifact_contract",
            ):
                with patch(
                    "tools.zircon_build_shader_prewarm_acceptance."
                    "validate_shader_resource_registry_export_contract",
                ):
                    with self.assertRaisesRegex(
                        RuntimeError,
                        "staged shader prewarm acceptance requires written cache variant identity",
                    ):
                        validate_staged_shader_prewarm_acceptance_contract(config)

    def test_acceptance_contract_rejects_duplicate_written_variant_identity(self):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(
                config,
                requested_count=2,
                written_count=2,
                written_variants=(
                    _written_variant(),
                    _written_variant(),
                ),
            ).cleanup
        )

        with patch(
            "tools.zircon_build_shader_prewarm_acceptance."
            "validate_shader_prewarm_report_contract",
        ):
            with patch(
                "tools.zircon_build_shader_prewarm_acceptance."
                "validate_shader_prewarm_cache_artifact_contract",
            ):
                with patch(
                    "tools.zircon_build_shader_prewarm_acceptance."
                    "validate_shader_resource_registry_export_contract",
                ):
                    with self.assertRaisesRegex(
                        RuntimeError,
                        "duplicate written cache variant identity",
                    ):
                        validate_staged_shader_prewarm_acceptance_contract(config)

    def test_acceptance_contract_rejects_invalid_written_variant_cache_hash_shape(
        self,
    ):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(
                config,
                requested_count=1,
                written_count=1,
                written_variants=(
                    _written_variant(cache_hash="not-a-cache-key"),
                ),
            ).cleanup
        )

        with patch(
            "tools.zircon_build_shader_prewarm_acceptance."
            "validate_shader_prewarm_report_contract",
        ):
            with patch(
                "tools.zircon_build_shader_prewarm_acceptance."
                "validate_shader_prewarm_cache_artifact_contract",
            ):
                with patch(
                    "tools.zircon_build_shader_prewarm_acceptance."
                    "validate_shader_resource_registry_export_contract",
                ):
                    with self.assertRaisesRegex(
                        RuntimeError,
                        "cache hash shape mismatch",
                    ):
                        validate_staged_shader_prewarm_acceptance_contract(config)

    def test_acceptance_contract_rejects_forward_only_staged_pass_report(self):
        config = _FakePrewarmConfig()
        self.addCleanup(
            _write_acceptance_report(
                config,
                requested_count=1,
                written_count=1,
                written_variants=(_written_variant(),),
                dimension_summary={
                    "pass_types": {
                        "forward": {
                            "requested_count": 1,
                            "written_count": 1,
                            "failed_count": 0,
                        },
                    },
                },
                source_provenance={
                    "source_count": 1,
                    "variant_count": 1,
                    "sources": {
                        "res://materials/prewarm-test.wgsl": {
                            "source_label": "res://materials/prewarm-test.wgsl",
                            "source_hash": "source-hash",
                            "template_revision": "zr-material-template-v1",
                            "requested_count": 1,
                            "written_count": 1,
                            "failed_count": 0,
                        },
                    },
                },
            ).cleanup
        )

        with self.assertRaisesRegex(
            RuntimeError,
            "shader prewarm report is missing requested pass types: "
            "gbuffer, depth_prepass, shadow, velocity, taa_reactive_mask",
        ):
            validate_staged_shader_prewarm_acceptance_contract(config)

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
    shader_asset_roots: tuple[Path, ...] = ()
    shader_permutation_registries: tuple[Path, ...] = ()
    shader_quality_tiers = ("medium",)
    shader_resource_registry = None
    shader_shading_model_ids: tuple[str, ...] = ()
    shader_prewarm_cache_root_override: Path | None = None
    targets_root = Path("target") / "prewarm-acceptance-contract-test"
    validate_wgpu_shaders = False

    @property
    def shader_prewarm_cache_root(self) -> Path:
        if self.shader_prewarm_cache_root_override is not None:
            return self.shader_prewarm_cache_root_override
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
        asset_roots: tuple[Path, ...] = (),
    ):
        self.shader_geometry_source_ids = shader_geometry_source_ids
        self.shader_shading_model_ids = shader_shading_model_ids
        self.asset_roots = asset_roots


def _write_acceptance_report(
    config: _FakePrewarmConfig,
    *,
    requested_count: int,
    written_count: int,
    failed_count: int = 0,
    written_variants: tuple[dict[str, object], ...] | None = None,
    dimension_summary: dict[str, object] | None = None,
    source_provenance: dict[str, object] | None = None,
) -> tempfile.TemporaryDirectory:
    temp_dir = tempfile.TemporaryDirectory()
    config._acceptance_report_temp_dir = temp_dir
    config.engine_root = Path(temp_dir.name) / "ZirconEngine"
    config.shader_prewarm_report_path.parent.mkdir(parents=True)
    report = {
        "requested_count": requested_count,
        "written_count": written_count,
        "failed_count": failed_count,
    }
    if written_variants is not None:
        report["written_variants"] = list(written_variants)
    if dimension_summary is not None:
        report["dimension_summary"] = dimension_summary
    if source_provenance is not None:
        report["source_provenance"] = source_provenance
    config.shader_prewarm_report_path.write_text(
        json.dumps(report),
        encoding="utf-8",
    )
    return temp_dir


_CACHE_HASH = "a" * 64
_SECOND_CACHE_HASH = "b" * 64


def _written_variant(
    *,
    cache_hash: str = _CACHE_HASH,
    canonical_string: str = "pass=forward|geometry=0|shading=0",
    source_label: str = "res://materials/prewarm-test.wgsl",
    include_source_label: bool = True,
) -> dict[str, object]:
    variant = {
        "cache_hash": cache_hash,
        "canonical_string": canonical_string,
        "source_label": source_label,
        "template_revision": "zr-material-template-v1",
        "naga_version": "test-naga",
        "wgpu_version": "test-wgpu",
    }
    if not include_source_label:
        del variant["source_label"]
    return variant


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


def _shader_resource_record(locator: str, *, revision: int = 1) -> dict[str, object]:
    return {
        "id": "00000000-0000-0000-0000-000000000001",
        "kind": "Shader",
        "primary_locator": locator,
        "artifact_locator": None,
        "revision": revision,
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
