"""Staged shader prewarm acceptance contract helpers."""

from __future__ import annotations

import json
from collections.abc import Mapping
from pathlib import Path

try:
    from .zircon_build_shader_prewarm import (
        shader_geometry_source_id_specs,
        shader_shading_model_id_specs,
        validate_shader_resource_registry_export_contract,
    )
    from .zircon_build_shader_prewarm_cache_artifacts import (
        validate_shader_prewarm_cache_artifact_contract,
    )
    from .zircon_build_shader_prewarm_report_contract import (
        validate_shader_prewarm_report_contract,
    )
    from .zircon_build_shader_prewarm_written_variants import (
        ReportedWrittenVariant,
        validate_cache_hash_shape,
        validate_unique_written_variant_identity,
    )
except ImportError:  # pragma: no cover - exercised when run as a script.
    from zircon_build_shader_prewarm import (
        shader_geometry_source_id_specs,
        shader_shading_model_id_specs,
        validate_shader_resource_registry_export_contract,
    )
    from zircon_build_shader_prewarm_cache_artifacts import (
        validate_shader_prewarm_cache_artifact_contract,
    )
    from zircon_build_shader_prewarm_report_contract import (
        validate_shader_prewarm_report_contract,
    )
    from zircon_build_shader_prewarm_written_variants import (
        ReportedWrittenVariant,
        validate_cache_hash_shape,
        validate_unique_written_variant_identity,
    )


_PRODUCT_MATERIAL_MESH_PASS_TYPES = (
    "forward",
    "gbuffer",
    "depth_prepass",
    "shadow",
    "velocity",
    "taa_reactive_mask",
)


def validate_staged_shader_prewarm_acceptance_contract(config) -> None:
    validate_staged_shader_prewarm_runtime_fallback_layout(config)
    validate_staged_shader_prewarm_nonempty_success_report(config)
    expected_geometry_source_ids = shader_geometry_source_id_specs(config)
    expected_shading_model_ids = shader_shading_model_id_specs(config)

    validate_shader_prewarm_report_contract(
        config.shader_prewarm_report_path,
        require_wgpu_module_validation=getattr(
            config,
            "validate_wgpu_shaders",
            False,
        ),
        require_wgpu_pipeline_validation=getattr(
            config,
            "validate_wgpu_pipelines",
            False,
        ),
        require_source_provenance=True,
        expected_pass_types=_PRODUCT_MATERIAL_MESH_PASS_TYPES,
        expected_quality_tiers=config.shader_quality_tiers,
        expected_geometry_sources=config.shader_geometry_sources,
        expected_geometry_source_ids=expected_geometry_source_ids,
        expected_shading_model_ids=expected_shading_model_ids,
    )
    validate_shader_prewarm_cache_artifact_contract(
        config.shader_prewarm_cache_root,
        report_path=config.shader_prewarm_report_path,
        expected_pass_types=_PRODUCT_MATERIAL_MESH_PASS_TYPES,
        expected_quality_tiers=config.shader_quality_tiers,
        expected_geometry_sources=config.shader_geometry_sources,
        expected_geometry_source_ids=expected_geometry_source_ids,
        expected_shading_model_ids=expected_shading_model_ids,
    )
    registry_path = (
        getattr(config, "shader_resource_registry", None)
        or config.shader_prewarm_resource_registry_path
    )
    requires_project_plugin_auto_export = (
        _requires_project_plugin_registry_auto_export(config)
    )
    validate_shader_resource_registry_export_contract(
        registry_path,
        report_path=config.shader_prewarm_report_path,
        require_usable_shader_records=requires_project_plugin_auto_export,
        require_report_registry_backed_sources=requires_project_plugin_auto_export,
    )


def validate_staged_shader_prewarm_runtime_fallback_layout(config) -> None:
    engine_root = Path(config.engine_root)
    expected_cache_root = engine_root / "cache" / "shader_variants"
    expected_report_path = engine_root / "cache" / "shader_variants_report.json"
    expected_resource_registry_path = (
        engine_root / "cache" / "shader_resource_records.json"
    )
    _require_path_contract(
        Path(config.shader_prewarm_cache_root),
        expected_cache_root,
        "staged shader prewarm cache root must match runtime fallback root",
    )
    _require_path_contract(
        Path(config.shader_prewarm_report_path),
        expected_report_path,
        "staged shader prewarm report must live beside runtime fallback root",
    )
    if not getattr(config, "shader_resource_registry", None):
        _require_path_contract(
            Path(config.shader_prewarm_resource_registry_path),
            expected_resource_registry_path,
            "staged shader prewarm resource registry export must live beside runtime fallback root",
        )


def _requires_project_plugin_registry_auto_export(config) -> bool:
    if getattr(config, "shader_resource_registry", None):
        return False
    if tuple(getattr(config, "shader_asset_roots", ())):
        return True
    return any(
        tuple(getattr(plugin, "asset_roots", ()))
        for plugin in getattr(config, "plugins", ())
    )


def validate_staged_shader_prewarm_nonempty_success_report(config) -> None:
    report_path = Path(config.shader_prewarm_report_path)
    report = _read_staged_shader_prewarm_report(report_path)
    requested_count = _count_value(report, "requested")
    written_count = _count_value(report, "written")
    failed_count = _count_value(report, "failed")
    if requested_count <= 0 or written_count <= 0:
        raise RuntimeError(
            "staged shader prewarm acceptance requires written variants: "
            f"requested={requested_count} written={written_count} "
            f"report={report_path}"
        )
    if failed_count != 0:
        raise RuntimeError(
            "staged shader prewarm acceptance requires zero failed variants: "
            f"failed={failed_count} report={report_path}"
        )
    if written_count != requested_count:
        raise RuntimeError(
            "staged shader prewarm acceptance requires all requested variants written: "
            f"requested={requested_count} written={written_count} report={report_path}"
        )
    _validate_staged_shader_prewarm_written_variant_identity(
        report,
        written_count=written_count,
        report_path=report_path,
    )


def _require_path_contract(actual: Path, expected: Path, message: str) -> None:
    if actual != expected:
        raise RuntimeError(f"{message}: expected {expected}, got {actual}")


def _validate_staged_shader_prewarm_written_variant_identity(
    report: Mapping[str, object],
    *,
    written_count: int,
    report_path: Path,
) -> None:
    written_variants = report.get("written_variants")
    if not isinstance(written_variants, list):
        raise RuntimeError(
            "staged shader prewarm acceptance requires written cache variants: "
            f"written={written_count} report={report_path}"
        )
    if len(written_variants) != written_count:
        raise RuntimeError(
            "staged shader prewarm acceptance written cache variant count mismatch: "
            f"written={written_count} written_variants={len(written_variants)} "
            f"report={report_path}"
        )
    required_fields = (
        "cache_hash",
        "canonical_string",
        "source_label",
        "template_revision",
        "naga_version",
        "wgpu_version",
    )
    for index, variant in enumerate(written_variants):
        if not isinstance(variant, Mapping):
            raise RuntimeError(
                "staged shader prewarm acceptance requires written cache variant identity: "
                f"index={index} report={report_path}"
            )
        missing = []
        for field in required_fields:
            value = variant.get(field)
            if not isinstance(value, str) or not value.strip():
                missing.append(field)
            elif field == "source_label" and value != value.strip():
                missing.append(field)
        if missing:
            raise RuntimeError(
                "staged shader prewarm acceptance requires written cache variant identity: "
                f"index={index} missing={', '.join(missing)} "
                f"report={report_path}"
            )
        validate_cache_hash_shape(
            str(variant["cache_hash"]),
            source=f"written_variants[{index}].cache_hash",
        )
    validate_unique_written_variant_identity(
        _reported_written_variants_for_acceptance(written_variants),
        report_path=report_path,
        message_prefix="staged shader prewarm acceptance rejects",
    )


def _reported_written_variants_for_acceptance(
    written_variants: list[object],
) -> tuple[ReportedWrittenVariant, ...]:
    variants: list[ReportedWrittenVariant] = []
    for variant in written_variants:
        if not isinstance(variant, Mapping):
            continue
        variants.append(
            ReportedWrittenVariant(
                cache_hash=str(variant["cache_hash"]),
                canonical_string=str(variant["canonical_string"]),
                source_label=str(variant["source_label"]),
                template_revision=str(variant["template_revision"]),
                naga_version=str(variant["naga_version"]),
                wgpu_version=str(variant["wgpu_version"]),
            )
        )
    return tuple(variants)


def _read_staged_shader_prewarm_report(report_path: Path) -> Mapping[str, object]:
    try:
        raw_report = json.loads(report_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise RuntimeError(
            "staged shader prewarm acceptance could not read report: "
            f"{report_path}"
        ) from error
    if not isinstance(raw_report, Mapping):
        raise RuntimeError(
            "staged shader prewarm acceptance report must be an object: "
            f"{report_path}"
        )
    return raw_report


def _count_value(counts: Mapping[str, object], field: str) -> int:
    value = counts.get(field)
    if value is None:
        value = counts.get(f"{field}_count", 0)
    if isinstance(value, bool):
        return 0
    if isinstance(value, int) and value >= 0:
        return value
    return 0
