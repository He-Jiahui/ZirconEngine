"""Validate one Zircon PBR viewer ready-frame PNG and its provenance sidecar."""

from __future__ import annotations

import argparse
import json
import math
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Mapping
from tools.zircon_shader_pbr_evidence_identity import validate_ready_frame_identity

from tools.zircon_pbr_visual_oracle import (
    DisplayVisualOracleResult,
    decode_rgba_png,
    rgba_statistics,
    validate_display_visual_oracle,
)


_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v14"
_V15_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v15"
_V16_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v16"
_CURRENT_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v17"
_V3_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v3"
_V4_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v4"
_V5_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v5"
_V6_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v6"
_V7_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v7"
_V8_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v8"
_V9_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v9"
_V10_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v10"
_V11_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v11"
_V12_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v12"
_V13_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v13"
# Decimal serialization of the canonical Rust IBL_BAKE_ALGORITHM_VERSION.
_CURRENT_IBL_BAKE_ALGORITHM_VERSION = "202608260008"
_SUPPORTED_READY_FRAME_EVIDENCE_SCHEMAS = frozenset(
    {
        "zircon_shader_pbr_viewer_ready_frame_evidence_v2",
        _V3_READY_FRAME_EVIDENCE_SCHEMA,
        _V4_READY_FRAME_EVIDENCE_SCHEMA,
        _V5_READY_FRAME_EVIDENCE_SCHEMA,
        _V6_READY_FRAME_EVIDENCE_SCHEMA,
        _V7_READY_FRAME_EVIDENCE_SCHEMA,
        _V8_READY_FRAME_EVIDENCE_SCHEMA,
        _V9_READY_FRAME_EVIDENCE_SCHEMA,
        _V10_READY_FRAME_EVIDENCE_SCHEMA,
        _V11_READY_FRAME_EVIDENCE_SCHEMA,
        _V12_READY_FRAME_EVIDENCE_SCHEMA,
        _V13_READY_FRAME_EVIDENCE_SCHEMA,
        _READY_FRAME_EVIDENCE_SCHEMA,
        _V15_READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
        _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
    }
)
_PROCESS_LOCAL_MESH_PIPELINE_CACHE = "process_local_mesh_pipeline_cache"
_PREWARM_NOT_REQUESTED_CACHE_SCOPE = "not_requested"
_MAX_REPORTED_DISTINCT_COLORS = 4_096
_VIEWPORT_PATTERN = re.compile(r"([1-9][0-9]*)x([1-9][0-9]*)\Z")
_POSITIVE_INTEGER_PATTERN = re.compile(r"[1-9][0-9]*\Z")
_NON_NEGATIVE_INTEGER_PATTERN = re.compile(r"[0-9]+\Z")
_MAX_U64 = (1 << 64) - 1
_REQUIRED_METADATA_FIELDS = (
    "schema",
    "screenshot",
    "screenshot_presentation",
    "interactive_direct_present_enabled",
    "backend",
    "hdri_path",
    "requested_source_face_size",
    "requested_pmrem_face_size",
    "active_source_cubemap_face_size",
    "active_source_cubemap_mip_count",
    "active_pmrem_face_size",
    "active_pmrem_mip_count",
    "render_profile",
    "environment_only_base_prewarm_cache_hit",
    "environment_only_base_prewarm_cache_scope",
    "environment_only_base_prewarm_shader_source_resolution_ns",
    "environment_only_base_prewarm_pipeline_creation_ns",
    "environment_only_base_prewarm_elapsed_ns",
    "viewport",
    "camera_yaw_degrees",
    "camera_pitch_degrees",
    "ibl_bake_algorithm_version",
    "ibl_staging_status",
    "ibl_staging_elapsed_ns",
    "ibl_total_elapsed_ns",
    "ready_frame_render_elapsed_ns",
    "ready_frame_extract_ns",
    "ready_frame_renderer_call_ns",
    "ready_frame_readback_and_completion_ns",
)
_V3_REQUIRED_METADATA_FIELDS = ("environment_only_base_prewarm_pipeline_ready",)
_V4_REQUIRED_METADATA_FIELDS = (
    "environment_only_base_prewarm_pipeline_ready",
    "environment_only_base_pipeline_ready_at_capture",
)
_V5_REQUIRED_METADATA_FIELDS = (
    *_V4_REQUIRED_METADATA_FIELDS,
    "scene_startup_hdri_decode_ns",
    "scene_startup_project_assets_ns",
    "scene_startup_runtime_bootstrap_ns",
    "scene_startup_project_open_ns",
    "scene_startup_world_load_ns",
    "scene_startup_renderer_initialization_ns",
    "scene_startup_renderer_backend_initialization_ns",
    "scene_startup_renderer_deferred_initialization_ns",
    "scene_startup_renderer_deferred_standard_pipeline_ns",
    "scene_startup_resource_streamer_initialization_ns",
    "scene_startup_ibl_restore_ns",
    "scene_startup_total_ns",
)
_V6_REQUIRED_METADATA_FIELDS = (
    *_V5_REQUIRED_METADATA_FIELDS,
    "one_shot_base_pipeline_wait_elapsed_ns",
)
_V7_REQUIRED_METADATA_FIELDS = (
    *_V6_REQUIRED_METADATA_FIELDS,
    "viewer_scene_load_elapsed_ns",
)
_V8_REQUIRED_METADATA_FIELDS = (
    *_V7_REQUIRED_METADATA_FIELDS,
    "viewer_ready_elapsed_ns",
)
_V9_REQUIRED_METADATA_FIELDS = (
    *_V8_REQUIRED_METADATA_FIELDS,
    "ibl_staging_source_decode_ns",
    "ibl_staging_cubemap_build_ns",
    "ibl_staging_equirect_projection_ns",
    "ibl_staging_source_mip_build_ns",
    "ibl_staging_pmrem_build_ns",
    "ibl_staging_sh9_build_ns",
    "ibl_staging_irradiance_cube_build_ns",
    "ibl_staging_bundle_write_ns",
)
_V10_REQUIRED_METADATA_FIELDS = (
    *_V9_REQUIRED_METADATA_FIELDS,
    "ibl_staging_source_zcube_bytes",
    "ibl_staging_asset_derived_bytes",
    "ibl_staging_parallel_executor_work_items",
)
_V11_REQUIRED_METADATA_FIELDS = (
    *_V10_REQUIRED_METADATA_FIELDS,
    "ibl_staging_irradiance_cube_source_sample_visits",
)
_V12_REQUIRED_METADATA_FIELDS = (
    *_V11_REQUIRED_METADATA_FIELDS,
    "registered_pipeline_variant_count",
    "registered_shader_variant_count",
    "texture_presence_normalized_pipeline_variant_count",
    "texture_presence_equivalent_pipeline_variant_count",
    "cached_render_pipeline_count",
    "cached_shader_module_count",
    "render_pipeline_creation_count",
    "shader_module_creation_count",
    "render_pipeline_creation_cpu_microseconds",
    "shader_module_creation_cpu_microseconds",
    "async_base_pipeline_queue_wait_count",
    "async_base_pipeline_queue_wait_microseconds",
)
_V13_REQUIRED_METADATA_FIELDS = (
    *_V12_REQUIRED_METADATA_FIELDS,
    "scene_startup_renderer_environment_brdf_lut_payload_cache_built",
    "scene_startup_renderer_environment_brdf_lut_payload_cache_wait_ns",
    "scene_startup_renderer_environment_brdf_lut_payload_build_ns",
    "scene_startup_renderer_environment_brdf_lut_texture_upload_submission_ns",
)
_V14_REQUIRED_METADATA_FIELDS = (
    *_V13_REQUIRED_METADATA_FIELDS,
    "host_mode",
    "host_composition_id",
    "scene_id",
    "capture_target",
    "gpu_scene_surface_present_count",
)
_V15_REQUIRED_METADATA_FIELDS = (
    *_V14_REQUIRED_METADATA_FIELDS,
    "screenshot_sha256",
    "screenshot_byte_length",
    "evidence_identity_schema",
    "evidence_run_id",
    "evidence_validation_policy",
    "evidence_identity_path",
    "evidence_identity_sha256",
    "evidence_identity_byte_length",
    "viewer_binary_path",
    "viewer_binary_sha256",
    "viewer_binary_byte_length",
    "hdri_sha256",
    "hdri_byte_length",
    "build_provenance_path",
    "build_provenance_sha256",
    "build_provenance_byte_length",
    "source_manifest_sha256",
)
_V16_REQUIRED_METADATA_FIELDS = (
    *_V15_REQUIRED_METADATA_FIELDS,
    "material_fixture",
    "required_material_base_pipeline_kind",
    "required_material_base_pipeline_ready_at_capture",
    "environment_only_base_prewarm_requested",
)
_V17_RETIRED_BRDF_LUT_STARTUP_FIELDS = frozenset(
    {
        "scene_startup_renderer_environment_brdf_lut_payload_cache_built",
        "scene_startup_renderer_environment_brdf_lut_payload_cache_wait_ns",
        "scene_startup_renderer_environment_brdf_lut_payload_build_ns",
    }
)
_V17_REQUIRED_METADATA_FIELDS = tuple(
    field
    for field in _V16_REQUIRED_METADATA_FIELDS
    if field not in _V17_RETIRED_BRDF_LUT_STARTUP_FIELDS
) + (
    "scene_startup_renderer_environment_brdf_lut_builtin_payload_materialized",
    "scene_startup_renderer_environment_brdf_lut_builtin_payload_cache_wait_ns",
    "scene_startup_renderer_environment_brdf_lut_builtin_payload_materialization_ns",
)


@dataclass(frozen=True)
class ReadyFrameEvidence:
    screenshot_path: Path
    sidecar_path: Path
    viewport: tuple[int, int]
    backend: str
    render_profile: str
    distinct_rgba_colors: int
    non_black_pixel_count: int
    metadata: Mapping[str, str]
    display_visual_oracle: DisplayVisualOracleResult | None


def validate_ready_frame_evidence(
    png_path: str | Path,
    *,
    expected_backend: str | None = None,
    require_direct_present: bool = False,
    expected_host_mode: str | None = None,
    visual_oracle_path: str | Path | None = None,
    required_schema: str | None = None,
    expected_ibl_bake_algorithm_version: str | None = None,
    min_distinct_rgba_colors: int = 2,
    min_non_black_pixels: int = 1,
) -> ReadyFrameEvidence:
    """Validate a ready-frame PNG/sidecar pair without starting the engine."""

    if min_distinct_rgba_colors < 2 or min_non_black_pixels < 1:
        raise ValueError("visual evidence thresholds must be positive and distinguish colors")
    screenshot_path = Path(png_path)
    screenshot_image = decode_rgba_png(screenshot_path)
    width = screenshot_image.width
    height = screenshot_image.height
    distinct_colors, non_black_pixels = rgba_statistics(
        screenshot_image,
        max_reported_distinct_colors=_MAX_REPORTED_DISTINCT_COLORS,
    )
    sidecar_path = screenshot_path.with_name(f"{screenshot_path.name}.txt")
    metadata = _read_metadata(sidecar_path)
    _validate_metadata(
        metadata,
        screenshot_path=screenshot_path,
        width=width,
        height=height,
        expected_backend=expected_backend,
        require_direct_present=require_direct_present,
        expected_host_mode=expected_host_mode,
        required_schema=required_schema,
        expected_ibl_bake_algorithm_version=expected_ibl_bake_algorithm_version,
    )
    if (
        distinct_colors < min_distinct_rgba_colors
        or non_black_pixels < min_non_black_pixels
    ):
        raise RuntimeError(
            "ready-frame PNG is visually insufficient: "
            f"distinct_rgba_colors={distinct_colors} "
            f"non_black_pixels={non_black_pixels} path={screenshot_path}"
        )
    display_visual_oracle = (
        validate_display_visual_oracle(
            screenshot_path,
            metadata=metadata,
            oracle_path=visual_oracle_path,
            _candidate_image=screenshot_image,
        )
        if visual_oracle_path is not None
        else None
    )
    return ReadyFrameEvidence(
        screenshot_path=screenshot_path,
        sidecar_path=sidecar_path,
        viewport=(width, height),
        backend=metadata["backend"],
        render_profile=metadata["render_profile"],
        distinct_rgba_colors=distinct_colors,
        non_black_pixel_count=non_black_pixels,
        metadata=metadata,
        display_visual_oracle=display_visual_oracle,
    )


def validate_current_ready_frame_evidence(
    png_path: str | Path,
    *,
    expected_backend: str | None = None,
    require_direct_present: bool = False,
    expected_host_mode: str | None = None,
    visual_oracle_path: str | Path | None = None,
    min_distinct_rgba_colors: int = 2,
    min_non_black_pixels: int = 1,
) -> ReadyFrameEvidence:
    """Validate the current ready-frame schema and canonical IBL recipe."""

    return validate_ready_frame_evidence(
        png_path,
        expected_backend=expected_backend,
        require_direct_present=require_direct_present,
        expected_host_mode=expected_host_mode,
        visual_oracle_path=visual_oracle_path,
        required_schema=_CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
        expected_ibl_bake_algorithm_version=_CURRENT_IBL_BAKE_ALGORITHM_VERSION,
        min_distinct_rgba_colors=min_distinct_rgba_colors,
        min_non_black_pixels=min_non_black_pixels,
    )


def ready_frame_evidence_summary(evidence: ReadyFrameEvidence) -> dict[str, object]:
    """Return the stable JSON payload emitted by the evidence CLI."""

    summary: dict[str, object] = {
        "schema": evidence.metadata["schema"],
        "png": str(evidence.screenshot_path),
        "sidecar": str(evidence.sidecar_path),
        "viewport": list(evidence.viewport),
        "backend": evidence.backend,
        "render_profile": evidence.render_profile,
        "distinct_rgba_colors": evidence.distinct_rgba_colors,
        "non_black_pixel_count": evidence.non_black_pixel_count,
    }
    if evidence.metadata["schema"] in (
        _V13_READY_FRAME_EVIDENCE_SCHEMA,
        _READY_FRAME_EVIDENCE_SCHEMA,
        _V15_READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
        _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
    ):
        summary["shader_pipeline_metrics"] = {
            field: int(evidence.metadata[field])
            for field in _V12_REQUIRED_METADATA_FIELDS
            if field not in _V11_REQUIRED_METADATA_FIELDS
        }
        brdf_lut_timing_fields = (
            (
                "scene_startup_renderer_environment_brdf_lut_builtin_payload_cache_wait_ns",
                "scene_startup_renderer_environment_brdf_lut_builtin_payload_materialization_ns",
            )
            if evidence.metadata["schema"] == _CURRENT_READY_FRAME_EVIDENCE_SCHEMA
            else (
                "scene_startup_renderer_environment_brdf_lut_payload_cache_wait_ns",
                "scene_startup_renderer_environment_brdf_lut_payload_build_ns",
            )
        )
        summary["startup_timing_ns"] = {
            field: int(evidence.metadata[field])
            for field in (
                "scene_startup_renderer_initialization_ns",
                *brdf_lut_timing_fields,
                "scene_startup_renderer_environment_brdf_lut_texture_upload_submission_ns",
                "scene_startup_renderer_deferred_standard_pipeline_ns",
                "scene_startup_ibl_restore_ns",
                "scene_startup_total_ns",
                "one_shot_base_pipeline_wait_elapsed_ns",
                "viewer_ready_elapsed_ns",
                "ready_frame_render_elapsed_ns",
            )
        }
    if evidence.metadata["schema"] == _CURRENT_READY_FRAME_EVIDENCE_SCHEMA:
        summary["material_pipeline"] = {
            field: evidence.metadata[field]
            for field in (
                "material_fixture",
                "required_material_base_pipeline_kind",
                "required_material_base_pipeline_ready_at_capture",
                "environment_only_base_prewarm_requested",
            )
        }
        summary["evidence_identity"] = {
            field: evidence.metadata[field]
            for field in (
                "evidence_run_id",
                "evidence_validation_policy",
                "evidence_identity_path",
                "evidence_identity_sha256",
                "evidence_identity_byte_length",
                "viewer_binary_path",
                "viewer_binary_sha256",
                "viewer_binary_byte_length",
                "source_manifest_sha256",
            )
        }
    if evidence.display_visual_oracle is not None:
        oracle = evidence.display_visual_oracle
        summary["display_visual_oracle"] = {
            "oracle": str(oracle.oracle_path),
            "oracle_sha256": oracle.oracle_sha256,
            "reference_png": str(oracle.reference_png_path),
            "reference_png_sha256": oracle.reference_png_sha256,
            "compared_pixel_count": oracle.compared_pixel_count,
            "mean_abs_error": oracle.mean_abs_error,
            "p99_abs_error": oracle.p99_abs_error,
            "exceeding_pixel_fraction": oracle.exceeding_pixel_fraction,
            "semantic_region_mean_abs_errors": dict(
                oracle.semantic_region_mean_abs_errors
            ),
        }
    return summary


def _read_metadata(sidecar_path: Path) -> dict[str, str]:
    try:
        text = sidecar_path.read_text(encoding="utf-8")
    except OSError as error:
        raise RuntimeError(f"ready-frame provenance sidecar is unavailable: {sidecar_path}") from error
    metadata: dict[str, str] = {}
    for line_number, line in enumerate(text.splitlines(), start=1):
        if not line:
            continue
        key, separator, value = line.partition("=")
        if not separator or not key or key != key.strip():
            raise RuntimeError(
                "ready-frame provenance sidecar contains an invalid field: "
                f"line={line_number} path={sidecar_path}"
            )
        if key in metadata:
            raise RuntimeError(
                "ready-frame provenance sidecar repeats a field: "
                f"field={key} path={sidecar_path}"
            )
        metadata[key] = value
    return metadata


def _validate_metadata(
    metadata: Mapping[str, str],
    *,
    screenshot_path: Path,
    width: int,
    height: int,
    expected_backend: str | None,
    require_direct_present: bool,
    expected_host_mode: str | None,
    required_schema: str | None,
    expected_ibl_bake_algorithm_version: str | None,
) -> None:
    missing = [field for field in _REQUIRED_METADATA_FIELDS if field not in metadata]
    if missing:
        raise RuntimeError(
            "ready-frame provenance sidecar is missing required fields: "
            f"{', '.join(missing)} path={screenshot_path}"
        )
    if metadata["schema"] not in _SUPPORTED_READY_FRAME_EVIDENCE_SCHEMAS:
        raise RuntimeError(
            "ready-frame provenance schema is unsupported: "
            f"schema={metadata['schema']} path={screenshot_path}"
        )
    if required_schema is not None and metadata["schema"] != required_schema:
        raise RuntimeError(
            "ready-frame provenance requires schema="
            f"{required_schema}: actual={metadata['schema']} path={screenshot_path}"
        )
    if metadata["schema"] in (
        _V15_READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
        _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
    ):
        required_identity_bound_fields = (
            _V17_REQUIRED_METADATA_FIELDS
            if metadata["schema"] == _CURRENT_READY_FRAME_EVIDENCE_SCHEMA
            else _V15_REQUIRED_METADATA_FIELDS
        )
        missing_identity_bound = [
            field for field in required_identity_bound_fields if field not in metadata
        ]
        if missing_identity_bound:
            raise RuntimeError(
                "ready-frame identity-bound provenance sidecar is missing required fields: "
                f"{', '.join(missing_identity_bound)} path={screenshot_path}"
            )
        validation_policy = {
            _V15_READY_FRAME_EVIDENCE_SCHEMA: "zircon_shader_pbr_viewer_ready_frame_v15",
            _V16_READY_FRAME_EVIDENCE_SCHEMA: "zircon_shader_pbr_viewer_ready_frame_v16",
            _CURRENT_READY_FRAME_EVIDENCE_SCHEMA: "zircon_shader_pbr_viewer_ready_frame_v17",
        }[metadata["schema"]]
        validate_ready_frame_identity(
            metadata,
            screenshot_path=screenshot_path,
            validation_policy=validation_policy,
        )
    if metadata["schema"] == _V15_READY_FRAME_EVIDENCE_SCHEMA:
        metadata = {**metadata, "schema": _READY_FRAME_EVIDENCE_SCHEMA}

    if metadata["schema"] == _V16_READY_FRAME_EVIDENCE_SCHEMA:
        missing_v16 = [
            field for field in _V16_REQUIRED_METADATA_FIELDS if field not in metadata
        ]
        if missing_v16:
            raise RuntimeError(
                "ready-frame v16 provenance sidecar is missing required fields: "
                f"{', '.join(missing_v16)} path={screenshot_path}"
            )
        _validate_v16_material_pipeline(metadata, screenshot_path=screenshot_path)

    if metadata["schema"] == _CURRENT_READY_FRAME_EVIDENCE_SCHEMA:
        missing_v17 = [
            field for field in _V17_REQUIRED_METADATA_FIELDS if field not in metadata
        ]
        if missing_v17:
            raise RuntimeError(
                "ready-frame v17 provenance sidecar is missing required fields: "
                f"{', '.join(missing_v17)} path={screenshot_path}"
            )
        _validate_v16_material_pipeline(metadata, screenshot_path=screenshot_path)

    if metadata["schema"] == _V3_READY_FRAME_EVIDENCE_SCHEMA:
        missing_v3 = [
            field for field in _V3_REQUIRED_METADATA_FIELDS if field not in metadata
        ]
        if missing_v3:
            raise RuntimeError(
                "ready-frame v3 provenance sidecar is missing required fields: "
                f"{', '.join(missing_v3)} path={screenshot_path}"
            )
    if metadata["schema"] == _V4_READY_FRAME_EVIDENCE_SCHEMA:
        missing_v4 = [
            field for field in _V4_REQUIRED_METADATA_FIELDS if field not in metadata
        ]
        if missing_v4:
            raise RuntimeError(
                "ready-frame v4 provenance sidecar is missing required fields: "
                f"{', '.join(missing_v4)} path={screenshot_path}"
            )
    if metadata["schema"] in (
        _V5_READY_FRAME_EVIDENCE_SCHEMA,
        _V6_READY_FRAME_EVIDENCE_SCHEMA,
        _V7_READY_FRAME_EVIDENCE_SCHEMA,
        _V8_READY_FRAME_EVIDENCE_SCHEMA,
        _V9_READY_FRAME_EVIDENCE_SCHEMA,
        _V10_READY_FRAME_EVIDENCE_SCHEMA,
        _V11_READY_FRAME_EVIDENCE_SCHEMA,
        _V13_READY_FRAME_EVIDENCE_SCHEMA,
        _READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
        _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
    ):
        missing_v5 = [
            field for field in _V5_REQUIRED_METADATA_FIELDS if field not in metadata
        ]
        if missing_v5:
            raise RuntimeError(
                "ready-frame v5 provenance sidecar is missing required fields: "
                f"{', '.join(missing_v5)} path={screenshot_path}"
            )
    if metadata["schema"] in (
        _V6_READY_FRAME_EVIDENCE_SCHEMA,
        _V7_READY_FRAME_EVIDENCE_SCHEMA,
        _V8_READY_FRAME_EVIDENCE_SCHEMA,
        _V9_READY_FRAME_EVIDENCE_SCHEMA,
        _V10_READY_FRAME_EVIDENCE_SCHEMA,
        _V11_READY_FRAME_EVIDENCE_SCHEMA,
        _V13_READY_FRAME_EVIDENCE_SCHEMA,
        _READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
        _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
    ):
        missing_v6 = [
            field for field in _V6_REQUIRED_METADATA_FIELDS if field not in metadata
        ]
        if missing_v6:
            raise RuntimeError(
                "ready-frame v6 provenance sidecar is missing required fields: "
                f"{', '.join(missing_v6)} path={screenshot_path}"
            )
    if metadata["schema"] == _V7_READY_FRAME_EVIDENCE_SCHEMA:
        missing_v7 = [
            field for field in _V7_REQUIRED_METADATA_FIELDS if field not in metadata
        ]
        if missing_v7:
            raise RuntimeError(
                "ready-frame v7 provenance sidecar is missing required fields: "
                f"{', '.join(missing_v7)} path={screenshot_path}"
            )
    if metadata["schema"] == _V8_READY_FRAME_EVIDENCE_SCHEMA:
        missing_v8 = [
            field for field in _V8_REQUIRED_METADATA_FIELDS if field not in metadata
        ]
        if missing_v8:
            raise RuntimeError(
                "ready-frame v8 provenance sidecar is missing required fields: "
                f"{', '.join(missing_v8)} path={screenshot_path}"
            )
    if metadata["schema"] == _V9_READY_FRAME_EVIDENCE_SCHEMA:
        missing_v9 = [
            field for field in _V9_REQUIRED_METADATA_FIELDS if field not in metadata
        ]
        if missing_v9:
            raise RuntimeError(
                "ready-frame v9 provenance sidecar is missing required fields: "
                f"{', '.join(missing_v9)} path={screenshot_path}"
            )
    if metadata["schema"] == _V10_READY_FRAME_EVIDENCE_SCHEMA:
        missing_v10 = [
            field for field in _V10_REQUIRED_METADATA_FIELDS if field not in metadata
        ]
        if missing_v10:
            raise RuntimeError(
                "ready-frame v10 provenance sidecar is missing required fields: "
                f"{', '.join(missing_v10)} path={screenshot_path}"
            )
    if metadata["schema"] == _V11_READY_FRAME_EVIDENCE_SCHEMA:
        missing_v11 = [
            field for field in _V11_REQUIRED_METADATA_FIELDS if field not in metadata
        ]
        if missing_v11:
            raise RuntimeError(
                "ready-frame v11 provenance sidecar is missing required fields: "
                f"{', '.join(missing_v11)} path={screenshot_path}"
            )
    if metadata["schema"] == _V12_READY_FRAME_EVIDENCE_SCHEMA:
        missing_v12 = [
            field for field in _V12_REQUIRED_METADATA_FIELDS if field not in metadata
        ]
        if missing_v12:
            raise RuntimeError(
                "ready-frame v12 provenance sidecar is missing required fields: "
                f"{', '.join(missing_v12)} path={screenshot_path}"
            )
    if metadata["schema"] in (
        _V13_READY_FRAME_EVIDENCE_SCHEMA,
        _READY_FRAME_EVIDENCE_SCHEMA,
        _V15_READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
    ):
        missing_v13 = [
            field for field in _V13_REQUIRED_METADATA_FIELDS if field not in metadata
        ]
        if missing_v13:
            raise RuntimeError(
                "ready-frame v13 provenance sidecar is missing required fields: "
                f"{', '.join(missing_v13)} path={screenshot_path}"
            )
    if metadata["schema"] == _READY_FRAME_EVIDENCE_SCHEMA:
        missing_v14 = [
            field for field in _V14_REQUIRED_METADATA_FIELDS if field not in metadata
        ]
        if missing_v14:
            raise RuntimeError(
                "ready-frame v14 provenance sidecar is missing required fields: "
                f"{', '.join(missing_v14)} path={screenshot_path}"
            )
    if metadata["screenshot"] != screenshot_path.name:
        raise RuntimeError(
            "ready-frame provenance screenshot name does not match PNG: "
            f"sidecar={metadata['screenshot']} png={screenshot_path.name}"
        )
    if metadata["screenshot_presentation"] != "cpu_readback":
        raise RuntimeError(
            "ready-frame provenance must identify the CPU readback capture path: "
            f"path={screenshot_path}"
        )
    _require_boolean(metadata, "interactive_direct_present_enabled", screenshot_path)
    _require_boolean(metadata, "environment_only_base_prewarm_cache_hit", screenshot_path)
    if metadata["schema"] in (
        _V3_READY_FRAME_EVIDENCE_SCHEMA,
        _V4_READY_FRAME_EVIDENCE_SCHEMA,
        _V5_READY_FRAME_EVIDENCE_SCHEMA,
        _V6_READY_FRAME_EVIDENCE_SCHEMA,
        _V7_READY_FRAME_EVIDENCE_SCHEMA,
        _V8_READY_FRAME_EVIDENCE_SCHEMA,
        _V9_READY_FRAME_EVIDENCE_SCHEMA,
        _V10_READY_FRAME_EVIDENCE_SCHEMA,
        _V11_READY_FRAME_EVIDENCE_SCHEMA,
        _V13_READY_FRAME_EVIDENCE_SCHEMA,
        _READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
        _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
    ):
        _require_boolean(
            metadata,
            "environment_only_base_prewarm_pipeline_ready",
            screenshot_path,
        )
    if metadata["schema"] in (
        _V4_READY_FRAME_EVIDENCE_SCHEMA,
        _V5_READY_FRAME_EVIDENCE_SCHEMA,
        _V6_READY_FRAME_EVIDENCE_SCHEMA,
        _V7_READY_FRAME_EVIDENCE_SCHEMA,
        _V8_READY_FRAME_EVIDENCE_SCHEMA,
        _V9_READY_FRAME_EVIDENCE_SCHEMA,
        _V10_READY_FRAME_EVIDENCE_SCHEMA,
        _V11_READY_FRAME_EVIDENCE_SCHEMA,
        _V13_READY_FRAME_EVIDENCE_SCHEMA,
        _READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
        _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
    ):
        _require_boolean(
            metadata,
            "environment_only_base_pipeline_ready_at_capture",
            screenshot_path,
        )
        if (
            metadata["schema"]
            not in (
                _V16_READY_FRAME_EVIDENCE_SCHEMA,
                _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
            )
            and metadata["environment_only_base_pipeline_ready_at_capture"] != "true"
        ):
            raise RuntimeError(
                "ready-frame provenance requires capture-time Base pipeline readiness: "
                f"path={screenshot_path}"
            )
    if require_direct_present and metadata["interactive_direct_present_enabled"] != "true":
        raise RuntimeError(
            "ready-frame provenance requires the interactive direct-present path: "
            f"path={screenshot_path}"
        )
    if metadata["schema"] == _READY_FRAME_EVIDENCE_SCHEMA:
        _validate_v14_host_capability(
            metadata,
            screenshot_path=screenshot_path,
            expected_host_mode=expected_host_mode,
        )
    _require_nonempty(metadata, "backend", screenshot_path)
    _require_nonempty(metadata, "hdri_path", screenshot_path)
    _require_nonempty(metadata, "render_profile", screenshot_path)
    _require_nonempty(metadata, "ibl_staging_status", screenshot_path)
    if expected_backend is not None and metadata["backend"] != expected_backend:
        raise RuntimeError(
            "ready-frame provenance backend does not match expectation: "
            f"expected={expected_backend} actual={metadata['backend']} path={screenshot_path}"
        )
    if metadata["render_profile"] != "environment_only_pbr_preview":
        raise RuntimeError(
            "ready-frame provenance must use the environment-only PBR profile: "
            f"profile={metadata['render_profile']} path={screenshot_path}"
        )
    if (
        metadata["schema"]
        not in (
            _V16_READY_FRAME_EVIDENCE_SCHEMA,
            _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
        )
        and metadata["environment_only_base_prewarm_cache_scope"]
        != _PROCESS_LOCAL_MESH_PIPELINE_CACHE
    ):
        raise RuntimeError(
            "ready-frame provenance cache scope is not the process-local MeshPipelineCache: "
            f"path={screenshot_path}"
        )
    _require_face_size(metadata, "requested_source_face_size", screenshot_path)
    _require_face_size(metadata, "requested_pmrem_face_size", screenshot_path)
    for field in (
        "active_source_cubemap_face_size",
        "active_source_cubemap_mip_count",
        "active_pmrem_face_size",
        "active_pmrem_mip_count",
        "ibl_bake_algorithm_version",
    ):
        _require_positive_integer(metadata, field, screenshot_path)
    if (
        expected_ibl_bake_algorithm_version is not None
        and metadata["ibl_bake_algorithm_version"]
        != expected_ibl_bake_algorithm_version
    ):
        raise RuntimeError(
            "ready-frame provenance requires IBL bake algorithm version="
            f"{expected_ibl_bake_algorithm_version}: "
            f"actual={metadata['ibl_bake_algorithm_version']} path={screenshot_path}"
        )
    _require_complete_cubemap_mip_chain(
        metadata,
        face_size_field="active_source_cubemap_face_size",
        mip_count_field="active_source_cubemap_mip_count",
        screenshot_path=screenshot_path,
    )
    _require_complete_cubemap_mip_chain(
        metadata,
        face_size_field="active_pmrem_face_size",
        mip_count_field="active_pmrem_mip_count",
        screenshot_path=screenshot_path,
    )
    for field in (
        "environment_only_base_prewarm_shader_source_resolution_ns",
        "environment_only_base_prewarm_pipeline_creation_ns",
        "environment_only_base_prewarm_elapsed_ns",
        "ibl_staging_elapsed_ns",
        "ibl_total_elapsed_ns",
        "ready_frame_render_elapsed_ns",
        "ready_frame_extract_ns",
        "ready_frame_renderer_call_ns",
        "ready_frame_readback_and_completion_ns",
    ):
        _require_non_negative_integer(metadata, field, screenshot_path)
    _require_duration_hierarchy(
        metadata,
        total_field="environment_only_base_prewarm_elapsed_ns",
        component_fields=(
            "environment_only_base_prewarm_shader_source_resolution_ns",
            "environment_only_base_prewarm_pipeline_creation_ns",
        ),
        screenshot_path=screenshot_path,
    )
    if metadata["schema"] in (
        _V5_READY_FRAME_EVIDENCE_SCHEMA,
        _V6_READY_FRAME_EVIDENCE_SCHEMA,
        _V7_READY_FRAME_EVIDENCE_SCHEMA,
        _V8_READY_FRAME_EVIDENCE_SCHEMA,
        _V9_READY_FRAME_EVIDENCE_SCHEMA,
        _V10_READY_FRAME_EVIDENCE_SCHEMA,
        _V11_READY_FRAME_EVIDENCE_SCHEMA,
        _V13_READY_FRAME_EVIDENCE_SCHEMA,
        _READY_FRAME_EVIDENCE_SCHEMA,
        _V15_READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
        _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
    ):
        startup_phase_fields = (
            "scene_startup_hdri_decode_ns",
            "scene_startup_project_assets_ns",
            "scene_startup_runtime_bootstrap_ns",
            "scene_startup_project_open_ns",
            "scene_startup_world_load_ns",
            "scene_startup_renderer_initialization_ns",
            "scene_startup_renderer_backend_initialization_ns",
            "scene_startup_renderer_deferred_initialization_ns",
            "scene_startup_renderer_deferred_standard_pipeline_ns",
            "scene_startup_resource_streamer_initialization_ns",
            "scene_startup_ibl_restore_ns",
            "scene_startup_total_ns",
        )
        for field in startup_phase_fields:
            _require_non_negative_integer(metadata, field, screenshot_path)
        _require_duration_hierarchy(
            metadata,
            total_field="scene_startup_renderer_initialization_ns",
            component_fields=(
                "scene_startup_renderer_backend_initialization_ns",
                "scene_startup_renderer_deferred_initialization_ns",
                "scene_startup_resource_streamer_initialization_ns",
            ),
            screenshot_path=screenshot_path,
        )
    if metadata["schema"] == _CURRENT_READY_FRAME_EVIDENCE_SCHEMA:
        _require_boolean(
            metadata,
            "scene_startup_renderer_environment_brdf_lut_builtin_payload_materialized",
            screenshot_path,
        )
        for field in (
            "scene_startup_renderer_environment_brdf_lut_builtin_payload_cache_wait_ns",
            "scene_startup_renderer_environment_brdf_lut_builtin_payload_materialization_ns",
            "scene_startup_renderer_environment_brdf_lut_texture_upload_submission_ns",
        ):
            _require_non_negative_integer(metadata, field, screenshot_path)
        _require_duration_hierarchy(
            metadata,
            total_field="scene_startup_renderer_environment_brdf_lut_builtin_payload_cache_wait_ns",
            component_fields=(
                "scene_startup_renderer_environment_brdf_lut_builtin_payload_materialization_ns",
            ),
            screenshot_path=screenshot_path,
        )
        _require_duration_hierarchy(
            metadata,
            total_field="scene_startup_renderer_deferred_initialization_ns",
            component_fields=("scene_startup_renderer_deferred_standard_pipeline_ns",),
            screenshot_path=screenshot_path,
        )
        _require_duration_hierarchy(
            metadata,
            total_field="scene_startup_total_ns",
            component_fields=(
                "scene_startup_hdri_decode_ns",
                "scene_startup_project_assets_ns",
                "scene_startup_runtime_bootstrap_ns",
                "scene_startup_project_open_ns",
                "scene_startup_world_load_ns",
                "scene_startup_renderer_initialization_ns",
                "scene_startup_ibl_restore_ns",
            ),
            screenshot_path=screenshot_path,
        )
    if metadata["schema"] in (
        _V5_READY_FRAME_EVIDENCE_SCHEMA,
        _V6_READY_FRAME_EVIDENCE_SCHEMA,
        _V7_READY_FRAME_EVIDENCE_SCHEMA,
        _V8_READY_FRAME_EVIDENCE_SCHEMA,
        _V9_READY_FRAME_EVIDENCE_SCHEMA,
        _V10_READY_FRAME_EVIDENCE_SCHEMA,
        _V11_READY_FRAME_EVIDENCE_SCHEMA,
        _V13_READY_FRAME_EVIDENCE_SCHEMA,
        _READY_FRAME_EVIDENCE_SCHEMA,
        _V15_READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
    ):
        _require_duration_hierarchy(
            metadata,
            total_field="scene_startup_renderer_deferred_initialization_ns",
            component_fields=("scene_startup_renderer_deferred_standard_pipeline_ns",),
            screenshot_path=screenshot_path,
        )
        _require_duration_hierarchy(
            metadata,
            total_field="scene_startup_total_ns",
            component_fields=(
                "scene_startup_hdri_decode_ns",
                "scene_startup_project_assets_ns",
                "scene_startup_runtime_bootstrap_ns",
                "scene_startup_project_open_ns",
                "scene_startup_world_load_ns",
                "scene_startup_renderer_initialization_ns",
                "scene_startup_ibl_restore_ns",
            ),
            screenshot_path=screenshot_path,
        )
    if metadata["schema"] in (
        _V13_READY_FRAME_EVIDENCE_SCHEMA,
        _READY_FRAME_EVIDENCE_SCHEMA,
        _V15_READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
    ):
        _require_boolean(
            metadata,
            "scene_startup_renderer_environment_brdf_lut_payload_cache_built",
            screenshot_path,
        )
        for field in (
            "scene_startup_renderer_environment_brdf_lut_payload_cache_wait_ns",
            "scene_startup_renderer_environment_brdf_lut_payload_build_ns",
            "scene_startup_renderer_environment_brdf_lut_texture_upload_submission_ns",
        ):
            _require_non_negative_integer(metadata, field, screenshot_path)
        _require_duration_hierarchy(
            metadata,
            total_field="scene_startup_renderer_environment_brdf_lut_payload_cache_wait_ns",
            component_fields=(
                "scene_startup_renderer_environment_brdf_lut_payload_build_ns",
            ),
            screenshot_path=screenshot_path,
        )
    if metadata["schema"] in (
        _V6_READY_FRAME_EVIDENCE_SCHEMA,
        _V7_READY_FRAME_EVIDENCE_SCHEMA,
        _V8_READY_FRAME_EVIDENCE_SCHEMA,
        _V9_READY_FRAME_EVIDENCE_SCHEMA,
        _V10_READY_FRAME_EVIDENCE_SCHEMA,
        _V11_READY_FRAME_EVIDENCE_SCHEMA,
        _V13_READY_FRAME_EVIDENCE_SCHEMA,
        _READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
        _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
    ):
        _require_non_negative_integer(
            metadata,
            "one_shot_base_pipeline_wait_elapsed_ns",
            screenshot_path,
        )
    if metadata["schema"] in (
        _V7_READY_FRAME_EVIDENCE_SCHEMA,
        _V8_READY_FRAME_EVIDENCE_SCHEMA,
        _V9_READY_FRAME_EVIDENCE_SCHEMA,
        _V10_READY_FRAME_EVIDENCE_SCHEMA,
        _V11_READY_FRAME_EVIDENCE_SCHEMA,
        _V13_READY_FRAME_EVIDENCE_SCHEMA,
        _READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
        _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
    ):
        _require_non_negative_integer(
            metadata,
            "viewer_scene_load_elapsed_ns",
            screenshot_path,
        )
        _require_duration_hierarchy(
            metadata,
            total_field="viewer_scene_load_elapsed_ns",
            component_fields=("scene_startup_total_ns",),
            screenshot_path=screenshot_path,
        )
    if metadata["schema"] in (
        _V8_READY_FRAME_EVIDENCE_SCHEMA,
        _V9_READY_FRAME_EVIDENCE_SCHEMA,
        _V10_READY_FRAME_EVIDENCE_SCHEMA,
        _V11_READY_FRAME_EVIDENCE_SCHEMA,
        _V13_READY_FRAME_EVIDENCE_SCHEMA,
        _READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
        _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
    ):
        _require_non_negative_integer(
            metadata,
            "viewer_ready_elapsed_ns",
            screenshot_path,
        )
        _require_duration_hierarchy(
            metadata,
            total_field="viewer_ready_elapsed_ns",
            component_fields=(
                "viewer_scene_load_elapsed_ns",
                "one_shot_base_pipeline_wait_elapsed_ns",
                "ready_frame_render_elapsed_ns",
            ),
            screenshot_path=screenshot_path,
        )
    if metadata["schema"] in (
        _V9_READY_FRAME_EVIDENCE_SCHEMA,
        _V10_READY_FRAME_EVIDENCE_SCHEMA,
        _V11_READY_FRAME_EVIDENCE_SCHEMA,
        _V13_READY_FRAME_EVIDENCE_SCHEMA,
        _READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
        _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
    ):
        for field in (
            "ibl_staging_source_decode_ns",
            "ibl_staging_cubemap_build_ns",
            "ibl_staging_equirect_projection_ns",
            "ibl_staging_source_mip_build_ns",
            "ibl_staging_pmrem_build_ns",
            "ibl_staging_sh9_build_ns",
            "ibl_staging_irradiance_cube_build_ns",
            "ibl_staging_bundle_write_ns",
        ):
            _require_non_negative_integer(metadata, field, screenshot_path)
        _require_duration_hierarchy(
            metadata,
            total_field="ibl_staging_cubemap_build_ns",
            component_fields=(
                "ibl_staging_equirect_projection_ns",
                "ibl_staging_source_mip_build_ns",
                "ibl_staging_pmrem_build_ns",
                "ibl_staging_sh9_build_ns",
            ),
            screenshot_path=screenshot_path,
        )
        _require_duration_hierarchy(
            metadata,
            total_field="ibl_staging_elapsed_ns",
            component_fields=(
                "ibl_staging_source_decode_ns",
                "ibl_staging_cubemap_build_ns",
                "ibl_staging_irradiance_cube_build_ns",
                "ibl_staging_bundle_write_ns",
            ),
            screenshot_path=screenshot_path,
        )
    if metadata["schema"] in (
        _V10_READY_FRAME_EVIDENCE_SCHEMA,
        _V11_READY_FRAME_EVIDENCE_SCHEMA,
        _V13_READY_FRAME_EVIDENCE_SCHEMA,
        _READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
        _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
    ):
        _require_staging_output_metrics(
            metadata,
            screenshot_path,
            schema_label={
                _V10_READY_FRAME_EVIDENCE_SCHEMA: "v10",
                _V11_READY_FRAME_EVIDENCE_SCHEMA: "v11",
                _V13_READY_FRAME_EVIDENCE_SCHEMA: "v13",
                _READY_FRAME_EVIDENCE_SCHEMA: "v14",
                _V16_READY_FRAME_EVIDENCE_SCHEMA: "v16",
                _CURRENT_READY_FRAME_EVIDENCE_SCHEMA: "v17",
            }[metadata["schema"]],
            require_irradiance_cube_source_sample_visits=(
                metadata["schema"]
                in (
                _V11_READY_FRAME_EVIDENCE_SCHEMA,
                _V13_READY_FRAME_EVIDENCE_SCHEMA,
                _READY_FRAME_EVIDENCE_SCHEMA,
                _V16_READY_FRAME_EVIDENCE_SCHEMA,
                _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
            )
            ),
        )
    if metadata["schema"] in (
        _V13_READY_FRAME_EVIDENCE_SCHEMA,
        _READY_FRAME_EVIDENCE_SCHEMA,
        _V16_READY_FRAME_EVIDENCE_SCHEMA,
        _CURRENT_READY_FRAME_EVIDENCE_SCHEMA,
    ):
        _require_shader_pipeline_metrics(metadata, screenshot_path)
    _require_duration_hierarchy(
        metadata,
        total_field="ibl_total_elapsed_ns",
        component_fields=("ibl_staging_elapsed_ns",),
        screenshot_path=screenshot_path,
    )
    _require_duration_hierarchy(
        metadata,
        total_field="ready_frame_render_elapsed_ns",
        component_fields=(
            "ready_frame_extract_ns",
            "ready_frame_renderer_call_ns",
            "ready_frame_readback_and_completion_ns",
        ),
        screenshot_path=screenshot_path,
    )
    for field in ("camera_yaw_degrees", "camera_pitch_degrees"):
        _require_finite_float(metadata, field, screenshot_path)
    viewport_match = _VIEWPORT_PATTERN.fullmatch(metadata["viewport"])
    if viewport_match is None:
        raise RuntimeError(
            "ready-frame provenance viewport is malformed: "
            f"viewport={metadata['viewport']} path={screenshot_path}"
        )
    viewport = (int(viewport_match.group(1)), int(viewport_match.group(2)))
    if viewport != (width, height):
        raise RuntimeError(
            "ready-frame provenance viewport does not match PNG dimensions: "
            f"sidecar={viewport[0]}x{viewport[1]} png={width}x{height} path={screenshot_path}"
        )


def _validate_v16_material_pipeline(
    metadata: Mapping[str, str], *, screenshot_path: Path
) -> None:
    for field in (
        "material_fixture",
        "required_material_base_pipeline_kind",
    ):
        _require_nonempty(metadata, field, screenshot_path)
    for field in (
        "required_material_base_pipeline_ready_at_capture",
        "environment_only_base_prewarm_requested",
        "environment_only_base_prewarm_pipeline_ready",
        "environment_only_base_pipeline_ready_at_capture",
        "environment_only_base_prewarm_cache_hit",
    ):
        _require_boolean(metadata, field, screenshot_path)

    if metadata["required_material_base_pipeline_ready_at_capture"] != "true":
        raise RuntimeError(
            "ready-frame v16 provenance requires the submitted material Base pipeline to be ready at capture: "
            f"path={screenshot_path}"
        )

    fixture = metadata["material_fixture"]
    required_pipeline = metadata["required_material_base_pipeline_kind"]
    expected_pipeline_by_fixture = {
        "metal-mirror": "environment-only-pbr-base",
        "dielectric-ior": "generic-forward-pbr-ior",
    }
    if expected_pipeline_by_fixture.get(fixture) != required_pipeline:
        raise RuntimeError(
            "ready-frame v16 provenance fixture does not match its required Base pipeline: "
            f"fixture={fixture} pipeline={required_pipeline} path={screenshot_path}"
        )

    prewarm_requested = metadata["environment_only_base_prewarm_requested"] == "true"
    cache_scope = metadata["environment_only_base_prewarm_cache_scope"]
    if prewarm_requested:
        if fixture != "metal-mirror":
            raise RuntimeError(
                "ready-frame v16 provenance only permits the specialized environment-only prewarm for the mirror fixture: "
                f"path={screenshot_path}"
            )
        if metadata["environment_only_base_pipeline_ready_at_capture"] != "true":
            raise RuntimeError(
                "ready-frame v16 provenance requires the specialized mirror pipeline to be ready at capture: "
                f"path={screenshot_path}"
            )
        if cache_scope != _PROCESS_LOCAL_MESH_PIPELINE_CACHE:
            raise RuntimeError(
                "ready-frame v16 provenance has an invalid specialized prewarm cache scope: "
                f"path={screenshot_path}"
            )
        return

    if fixture != "dielectric-ior":
        raise RuntimeError(
            "ready-frame v16 provenance may omit the specialized prewarm only for the explicit IOR fixture: "
            f"path={screenshot_path}"
        )
    if cache_scope != _PREWARM_NOT_REQUESTED_CACHE_SCOPE:
        raise RuntimeError(
            "ready-frame v16 provenance must mark the unused specialized prewarm as not requested: "
            f"path={screenshot_path}"
        )
    if any(
        metadata[field] != "false"
        for field in (
            "environment_only_base_prewarm_pipeline_ready",
            "environment_only_base_pipeline_ready_at_capture",
            "environment_only_base_prewarm_cache_hit",
        )
    ):
        raise RuntimeError(
            "ready-frame v16 provenance must not report specialized prewarm readiness for the IOR fixture: "
            f"path={screenshot_path}"
        )
    if any(
        metadata[field] != "0"
        for field in (
            "environment_only_base_prewarm_shader_source_resolution_ns",
            "environment_only_base_prewarm_pipeline_creation_ns",
            "environment_only_base_prewarm_elapsed_ns",
        )
    ):
        raise RuntimeError(
            "ready-frame v16 provenance must report zero specialized-prewarm timing when it was not requested: "
            f"path={screenshot_path}"
        )


def _validate_v14_host_capability(
    metadata: Mapping[str, str],
    *,
    screenshot_path: Path,
    expected_host_mode: str | None,
) -> None:
    for field in (
        "host_mode",
        "host_composition_id",
        "scene_id",
        "capture_target",
    ):
        _require_nonempty(metadata, field, screenshot_path)
    if expected_host_mode is not None and metadata["host_mode"] != expected_host_mode:
        raise RuntimeError(
            "ready-frame provenance host mode does not match expectation: "
            f"expected={expected_host_mode} actual={metadata['host_mode']} path={screenshot_path}"
        )
    if metadata["host_mode"] != "offscreen-diagnostic":
        raise RuntimeError(
            "ready-frame CPU readback evidence must declare host_mode=offscreen-diagnostic: "
            f"actual={metadata['host_mode']} path={screenshot_path}"
        )
    expected_fields = {
        "host_composition_id": "zircon_shader_pbr_viewer_standalone_diagnostic_v1",
        "scene_id": "single_pbr_mirror_sphere",
        "capture_target": "offscreen-scene-renderer-cpu-readback",
        "gpu_scene_surface_present_count": "0",
        "interactive_direct_present_enabled": "false",
    }
    for field, expected in expected_fields.items():
        if metadata[field] != expected:
            raise RuntimeError(
                "ready-frame offscreen-diagnostic capability is inconsistent: "
                f"field={field} expected={expected} actual={metadata[field]} path={screenshot_path}"
            )


def _require_boolean(metadata: Mapping[str, str], field: str, screenshot_path: Path) -> None:
    if metadata[field] not in ("true", "false"):
        raise RuntimeError(
            "ready-frame provenance boolean is malformed: "
            f"field={field} path={screenshot_path}"
        )


def _require_nonempty(metadata: Mapping[str, str], field: str, screenshot_path: Path) -> None:
    value = metadata[field]
    if not value or value != value.strip():
        raise RuntimeError(
            "ready-frame provenance value is blank or padded: "
            f"field={field} path={screenshot_path}"
        )


def _require_face_size(metadata: Mapping[str, str], field: str, screenshot_path: Path) -> None:
    if metadata[field] != "automatic":
        _require_positive_integer(metadata, field, screenshot_path)


def _require_complete_cubemap_mip_chain(
    metadata: Mapping[str, str],
    *,
    face_size_field: str,
    mip_count_field: str,
    screenshot_path: Path,
) -> None:
    face_size = int(metadata[face_size_field])
    mip_count = int(metadata[mip_count_field])
    expected_mip_count = face_size.bit_length()
    if mip_count != expected_mip_count:
        raise RuntimeError(
            "ready-frame provenance cubemap mip layout is inconsistent: "
            f"face_size={face_size} mip_count={mip_count} expected={expected_mip_count} "
            f"source={face_size_field} path={screenshot_path}"
        )


def _require_positive_integer(
    metadata: Mapping[str, str], field: str, screenshot_path: Path
) -> None:
    if _POSITIVE_INTEGER_PATTERN.fullmatch(metadata[field]) is None:
        raise RuntimeError(
            "ready-frame provenance positive integer is malformed: "
            f"field={field} path={screenshot_path}"
        )


def _require_staging_output_metrics(
    metadata: Mapping[str, str],
    screenshot_path: Path,
    *,
    schema_label: str,
    require_irradiance_cube_source_sample_visits: bool,
) -> None:
    source_bytes = _require_non_negative_integer(
        metadata, "ibl_staging_source_zcube_bytes", screenshot_path
    )
    artifact_bytes = _require_non_negative_integer(
        metadata, "ibl_staging_asset_derived_bytes", screenshot_path
    )
    work_items = _require_non_negative_integer(
        metadata, "ibl_staging_parallel_executor_work_items", screenshot_path
    )
    irradiance_cube_source_sample_visits = None
    if require_irradiance_cube_source_sample_visits:
        irradiance_cube_source_sample_visits = _require_non_negative_u64_count(
            metadata,
            "ibl_staging_irradiance_cube_source_sample_visits",
            screenshot_path,
        )
    status = metadata["ibl_staging_status"]
    if status not in ("Written", "Reused"):
        raise RuntimeError(
            f"ready-frame {schema_label} provenance has an unsupported IBL staging status: "
            f"status={status} path={screenshot_path}"
        )
    if source_bytes == 0 or artifact_bytes == 0:
        raise RuntimeError(
            f"ready-frame {schema_label} provenance requires non-empty staged IBL outputs: "
            f"path={screenshot_path}"
        )
    if status == "Reused" and work_items != 0:
        raise RuntimeError(
            f"ready-frame {schema_label} provenance cache reuse must not submit IBL executor work: "
            f"work_items={work_items} path={screenshot_path}"
        )
    if status == "Reused" and irradiance_cube_source_sample_visits not in (None, 0):
        raise RuntimeError(
            f"ready-frame {schema_label} provenance cache reuse must not report IEM candidate iterations: "
            f"sample_visits={irradiance_cube_source_sample_visits} path={screenshot_path}"
        )
    if status == "Written" and work_items == 0:
        raise RuntimeError(
            f"ready-frame {schema_label} provenance written HDRI must report IBL executor work: "
            f"path={screenshot_path}"
        )


def _require_shader_pipeline_metrics(
    metadata: Mapping[str, str], screenshot_path: Path
) -> None:
    fields = (
        "registered_pipeline_variant_count",
        "registered_shader_variant_count",
        "texture_presence_normalized_pipeline_variant_count",
        "texture_presence_equivalent_pipeline_variant_count",
        "cached_render_pipeline_count",
        "cached_shader_module_count",
        "render_pipeline_creation_count",
        "shader_module_creation_count",
        "render_pipeline_creation_cpu_microseconds",
        "shader_module_creation_cpu_microseconds",
        "async_base_pipeline_queue_wait_count",
        "async_base_pipeline_queue_wait_microseconds",
    )
    values = {
        field: _require_non_negative_u64_count(metadata, field, screenshot_path)
        for field in fields
    }
    registered_pipeline_count = values["registered_pipeline_variant_count"]
    registered_shader_count = values["registered_shader_variant_count"]
    normalized_pipeline_count = values[
        "texture_presence_normalized_pipeline_variant_count"
    ]
    equivalent_pipeline_count = values[
        "texture_presence_equivalent_pipeline_variant_count"
    ]
    if registered_pipeline_count == 0 or registered_shader_count == 0:
        raise RuntimeError(
            "ready-frame v12 provenance requires registered shader and pipeline variants: "
            f"path={screenshot_path}"
        )
    if registered_shader_count > registered_pipeline_count:
        raise RuntimeError(
            "ready-frame v12 provenance shader variant count exceeds pipeline variants: "
            f"path={screenshot_path}"
        )
    if (
        values["cached_render_pipeline_count"] == 0
        or values["cached_shader_module_count"] == 0
    ):
        raise RuntimeError(
            "ready-frame v12 provenance requires resident Base pipeline GPU objects: "
            f"path={screenshot_path}"
        )
    if (
        normalized_pipeline_count > registered_pipeline_count
        or equivalent_pipeline_count
        != registered_pipeline_count - normalized_pipeline_count
    ):
        raise RuntimeError(
            "ready-frame v12 provenance texture-presence normalization is inconsistent: "
            f"path={screenshot_path}"
        )
    if values["cached_render_pipeline_count"] > values["render_pipeline_creation_count"]:
        raise RuntimeError(
            "ready-frame v12 provenance cached render pipelines exceed creation events: "
            f"path={screenshot_path}"
        )
    if values["cached_shader_module_count"] > values["shader_module_creation_count"]:
        raise RuntimeError(
            "ready-frame v12 provenance cached shader modules exceed creation events: "
            f"path={screenshot_path}"
        )
    if (
        values["async_base_pipeline_queue_wait_count"] == 0
        and values["async_base_pipeline_queue_wait_microseconds"] != 0
    ):
        raise RuntimeError(
            "ready-frame v12 provenance queue wait time has no admitted async job: "
            f"path={screenshot_path}"
        )


def _require_non_negative_integer(
    metadata: Mapping[str, str], field: str, screenshot_path: Path
) -> int:
    if _NON_NEGATIVE_INTEGER_PATTERN.fullmatch(metadata[field]) is None:
        raise RuntimeError(
            "ready-frame provenance duration is malformed: "
            f"field={field} path={screenshot_path}"
        )
    return int(metadata[field])


def _require_non_negative_u64_count(
    metadata: Mapping[str, str], field: str, screenshot_path: Path
) -> int:
    if _NON_NEGATIVE_INTEGER_PATTERN.fullmatch(metadata[field]) is None:
        raise RuntimeError(
            "ready-frame provenance non-negative count is malformed: "
            f"field={field} path={screenshot_path}"
        )
    value = int(metadata[field])
    if value > _MAX_U64:
        raise RuntimeError(
            "ready-frame provenance count exceeds u64: "
            f"field={field} value={value} path={screenshot_path}"
        )
    return value


def _require_duration_hierarchy(
    metadata: Mapping[str, str],
    *,
    total_field: str,
    component_fields: tuple[str, ...],
    screenshot_path: Path,
) -> None:
    total = int(metadata[total_field])
    component_total = sum(int(metadata[field]) for field in component_fields)
    if total < component_total:
        raise RuntimeError(
            "ready-frame provenance duration hierarchy is inconsistent: "
            f"total_field={total_field} total_ns={total} "
            f"component_fields={','.join(component_fields)} component_total_ns={component_total} "
            f"path={screenshot_path}"
        )


def _require_finite_float(metadata: Mapping[str, str], field: str, screenshot_path: Path) -> None:
    try:
        value = float(metadata[field])
    except ValueError as error:
        raise RuntimeError(
            "ready-frame provenance camera angle is malformed: "
            f"field={field} path={screenshot_path}"
        ) from error
    if not math.isfinite(value):
        raise RuntimeError(
            "ready-frame provenance camera angle must be finite: "
            f"field={field} path={screenshot_path}"
        )


def _parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate a Zircon PBR viewer ready-frame PNG and v15 provenance sidecar."
    )
    parser.add_argument("png", type=Path, help="Ready-frame PNG written by zircon_shader_pbr_viewer")
    parser.add_argument("--expected-backend", help="Require the recorded backend, for example Dx12")
    parser.add_argument(
        "--expected-host-mode",
        help="Require the current-schema host mode, for example offscreen-diagnostic",
    )
    parser.add_argument(
        "--display-visual-oracle",
        type=Path,
        help=(
            "Require a versioned display-output visual oracle bound to the ready-frame "
            "provenance and reference PNG"
        ),
    )
    parser.add_argument(
        "--require-direct-present",
        action="store_true",
        help="Require the normal interactive path to have used direct presentation",
    )
    parser.add_argument(
        "--allow-legacy-schema",
        action="store_true",
        help=(
            "Allow v2-v14 or stale IBL bake provenance only when inspecting "
            "historical baseline evidence"
        ),
    )
    return parser.parse_args()


def main() -> int:
    arguments = _parse_arguments()
    try:
        if arguments.allow_legacy_schema:
            evidence = validate_ready_frame_evidence(
                arguments.png,
                expected_backend=arguments.expected_backend,
                require_direct_present=arguments.require_direct_present,
                expected_host_mode=arguments.expected_host_mode,
                visual_oracle_path=arguments.display_visual_oracle,
            )
        else:
            evidence = validate_current_ready_frame_evidence(
                arguments.png,
                expected_backend=arguments.expected_backend,
                require_direct_present=arguments.require_direct_present,
                expected_host_mode=arguments.expected_host_mode,
                visual_oracle_path=arguments.display_visual_oracle,
            )
    except (OSError, RuntimeError, ValueError) as error:
        print(f"PBR viewer evidence validation failed: {error}", file=sys.stderr)
        return 1
    print(json.dumps(ready_frame_evidence_summary(evidence), sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
