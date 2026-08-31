#!/usr/bin/env python3
"""Summarize source-bound cold/warm Zircon PBR viewer startup evidence."""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
import math
import re
import statistics
import sys
from pathlib import Path
from typing import Any, Iterable, Mapping

try:
    from tools.zircon_shader_pbr_evidence_identity import normalize_evidence_path
    from tools.zircon_shader_pbr_profile_tool_identity import validate_profile_tool_files
    from tools.zircon_validate_shader_pbr_viewer_evidence import (
        ready_frame_evidence_summary,
        validate_current_ready_frame_evidence,
    )
except ModuleNotFoundError:
    from zircon_shader_pbr_evidence_identity import normalize_evidence_path
    from zircon_shader_pbr_profile_tool_identity import validate_profile_tool_files
    from zircon_validate_shader_pbr_viewer_evidence import (
        ready_frame_evidence_summary,
        validate_current_ready_frame_evidence,
    )


_PROFILE_KIND = "zircon_shader_pbr_viewer_startup_matrix"
_RUN_PROFILE_KIND = "zircon_shader_pbr_viewer_startup_run"
_PROFILE_MANIFEST_KIND = "zircon_shader_pbr_viewer_startup"
_PROFILE_COMPLETION_RECEIPT_SCHEMA_VERSION = 1
_PROFILE_COMPLETION_RECEIPT_KIND = "zircon_shader_pbr_profile_completion"
_MANAGED_BUILD_PROVENANCE_SCHEMA_VERSION = 2
_MANAGED_BUILD_PROVENANCE_KIND = "zircon_managed_viewer_artifact_provenance"
_MANAGED_ARTIFACT_KIND = "shader-pbr-viewer"
_READY_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v17"
_GPU_TIMING_SCHEMA = "zircon_shader_pbr_viewer_gpu_timing_evidence_v3"
_GPU_TIMING_WARMUP_SAMPLE_COUNT = 5
_GPU_TIMING_MEASURED_SAMPLE_COUNT = 31
_REQUIRED_GPU_PASSES = frozenset(
    {
        "direct_gpu_scene_upload",
        "direct_scene_content",
        "direct_output_transfer",
        "direct_overlays",
    }
)
_U64_PATTERN = re.compile(r"[0-9]+\Z")
_SHA256_PATTERN = re.compile(r"[0-9a-f]{64}\Z")
_COORDINATOR_ID_PATTERN = re.compile(r"[0-9a-f]{32}\Z")
_MANAGED_TARGET_RELATIVE_PATH_PATTERN = re.compile(
    r"(?:debug|release|profiling)/zircon_shader_pbr_viewer(?:\.exe)?\Z"
)
_GPU_PASS_AGGREGATE_PATTERN = re.compile(
    r"pass\.([a-z][a-z0-9_]*)\.(min|median|p95|max)_us\Z"
)
_GPU_TOTAL_AGGREGATE_PATTERN = re.compile(r"total\.(min|median|p95|max)_us\Z")
_GPU_SAMPLE_STANDARD_PATTERN = re.compile(
    r"sample\.([0-9]{3})\.(frame_generation|total_us)\Z"
)
_GPU_SAMPLE_PASS_PATTERN = re.compile(
    r"sample\.([0-9]{3})\.pass\.([a-z][a-z0-9_]*)_us\Z"
)
_GPU_SAMPLE_MESH_SUBMISSION_PATTERN = re.compile(
    r"sample\.([0-9]{3})\.mesh\.([a-z][a-z0-9_]*)\Z"
)
_MAX_U64 = (1 << 64) - 1
_MAX_U32 = (1 << 32) - 1
_GPU_MESH_SUBMISSION_FIELDS = (
    "opaque_command_count",
    "advanced_pbr_opaque_command_count",
    "cached_command_hit_count",
    "command_rebuild_count",
    "dynamic_command_count",
)
_RUNTIME_PROFILE_SCHEMA = "zircon_shader_pbr_runtime_profile_v1"
_RUNTIME_PROFILE_ARTIFACTS = {
    "timeline": "timeline.zrtrace.json",
    "hotspots": "hotspots.json",
    "counter_hotspots": "counter_hotspots.json",
    "summary": "summary.md",
}
_SHADER_PIPELINE_STAGES = (
    "material_requirement_admission",
    "mesh_source_build",
    "module_include_resolution",
    "template_assembly",
    "source_hash",
    "naga_validation",
    "disk_cache_lookup",
    "disk_cache_write",
    "wgpu_pipeline_error_scope_pop",
)
_STARTUP_DURATION_FIELDS = {
    "renderer_initialization": "scene_startup_renderer_initialization_ns",
    "environment_brdf_lut_builtin_payload_cache_wait": "scene_startup_renderer_environment_brdf_lut_builtin_payload_cache_wait_ns",
    "environment_brdf_lut_builtin_payload_materialization": "scene_startup_renderer_environment_brdf_lut_builtin_payload_materialization_ns",
    "environment_brdf_lut_texture_upload_submission": "scene_startup_renderer_environment_brdf_lut_texture_upload_submission_ns",
    "renderer_deferred_standard_pipeline": "scene_startup_renderer_deferred_standard_pipeline_ns",
    "ibl_restore": "scene_startup_ibl_restore_ns",
    "startup_total": "scene_startup_total_ns",
    "viewer_ready": "viewer_ready_elapsed_ns",
}
_PSO_DURATION_FIELDS = {
    "render_pipeline_creation_cpu": "render_pipeline_creation_cpu_microseconds",
    "shader_module_creation_cpu": "shader_module_creation_cpu_microseconds",
    "async_base_pipeline_queue_wait": "async_base_pipeline_queue_wait_microseconds",
}
_IBL_STAGING_DURATION_FIELDS = {
    "source_decode": "ibl_staging_source_decode_ns",
    "cubemap_build": "ibl_staging_cubemap_build_ns",
    "equirect_projection": "ibl_staging_equirect_projection_ns",
    "source_mip_build": "ibl_staging_source_mip_build_ns",
    "pmrem_build": "ibl_staging_pmrem_build_ns",
    "sh9_build": "ibl_staging_sh9_build_ns",
    "irradiance_cube_build": "ibl_staging_irradiance_cube_build_ns",
    "bundle_write": "ibl_staging_bundle_write_ns",
}
_IBL_STAGING_PARALLEL_WORK_ITEM_FIELDS = {
    "equirect_projection": "ibl_staging_equirect_projection_parallel_work_items",
    "source_mip_build": "ibl_staging_source_mip_build_parallel_work_items",
    "pmrem_build": "ibl_staging_pmrem_build_parallel_work_items",
    "irradiance_cube_build": "ibl_staging_irradiance_cube_build_parallel_work_items",
}
_REQUESTED_LAYOUT_FIELDS = {
    "requested_source_face_size": "requested_source_face_size",
    "requested_pmrem_face_size": "requested_pmrem_face_size",
}
_ACTIVE_LAYOUT_FIELDS = {
    "source_cubemap_face_size": "active_source_cubemap_face_size",
    "source_cubemap_mip_count": "active_source_cubemap_mip_count",
    "pmrem_face_size": "active_pmrem_face_size",
    "pmrem_mip_count": "active_pmrem_mip_count",
}
_CACHE_LAYER_NAMES = (
    "engine_cache",
    "shader_cache",
    "os_file_cache",
    "driver_cache",
)
_CACHE_CONTROL_STATES = {"controlled", "uncontrolled"}
_CACHE_COMPARISON_SCOPE = "process_and_caller_owned_engine_cache"
_WGPU_BACKEND_SELECTORS = frozenset({"vulkan", "metal", "dx12", "gl", "webgpu"})
_MACHINE_MANIFEST_SCHEMA_VERSION = 1
_MACHINE_MANIFEST_KIND = "zircon_performance_machine_snapshot"
_MACHINE_MANIFEST_CATEGORIES = (
    "cpu",
    "gpu",
    "memory",
    "bios",
    "os",
    "display_modes",
    "power_policy",
    "thermal_frequency",
    "background_load",
    "virtualization",
)
_MACHINE_OBSERVATION_STATUSES = {"captured", "unavailable"}
_PROFILE_ID_PATTERN = re.compile(r"[a-z][a-z0-9-]{2,127}\Z")
_PROFILE_ARTIFACT_RELATIVE_PATH_PATTERN = re.compile(r"[A-Za-z0-9._/-]+\Z")


def validate_profile_completion_receipt(
    summary_path: str | Path,
    completion_receipt_path: str | Path,
) -> dict[str, object]:
    """Verify the immutable completion marker and its complete staging-root closure."""

    resolved_summary = Path(summary_path).resolve()
    profile_root = resolved_summary.parent
    receipt_path = Path(completion_receipt_path).resolve()
    receipt = _read_json_mapping(receipt_path, "profile completion receipt")
    expected_fields = {
        "schema_version",
        "receipt_kind",
        "status",
        "profile_id",
        "profile_root",
        "completed_utc",
        "artifacts",
    }
    if set(receipt) != expected_fields:
        raise RuntimeError(
            "Shader PBR profile completion receipt has an unexpected schema: "
            f"path={receipt_path}"
        )
    if receipt.get("schema_version") != _PROFILE_COMPLETION_RECEIPT_SCHEMA_VERSION:
        raise RuntimeError(
            "Shader PBR profile completion receipt has an unsupported schema: "
            f"path={receipt_path}"
        )
    if receipt.get("receipt_kind") != _PROFILE_COMPLETION_RECEIPT_KIND:
        raise RuntimeError(
            "Shader PBR profile completion receipt has an unexpected kind: "
            f"path={receipt_path}"
        )
    if receipt.get("status") != "completed":
        raise RuntimeError(
            "Shader PBR profile completion receipt is not committed: "
            f"path={receipt_path}"
        )
    profile_id = _require_string(receipt, "profile_id", receipt_path)
    if _PROFILE_ID_PATTERN.fullmatch(profile_id) is None:
        raise RuntimeError(
            "Shader PBR profile completion receipt has an invalid profile id: "
            f"path={receipt_path}"
        )
    recorded_root = Path(_require_string(receipt, "profile_root", receipt_path)).resolve()
    if recorded_root != profile_root:
        raise RuntimeError(
            "Shader PBR profile completion receipt root does not bind the summary: "
            f"receipt_root={recorded_root} summary_root={profile_root}"
        )
    if not _require_string(receipt, "completed_utc", receipt_path):
        raise RuntimeError(
            "Shader PBR profile completion receipt has no completion timestamp: "
            f"path={receipt_path}"
        )
    expected_artifacts = _completion_receipt_artifacts(
        _require_list(receipt, "artifacts", receipt_path),
        receipt_path,
    )
    actual_artifacts = _profile_root_artifacts(profile_root)
    if tuple(expected_artifacts) != tuple(actual_artifacts):
        raise RuntimeError(
            "Shader PBR profile completion receipt artifact closure changed: "
            f"path={receipt_path}"
        )
    for relative_path, expected in expected_artifacts.items():
        actual = actual_artifacts[relative_path]
        if expected["sha256"] != actual["sha256"]:
            raise RuntimeError(
                "Shader PBR profile completion receipt artifact SHA-256 changed: "
                f"path={relative_path}"
            )
        if expected["byte_length"] != actual["byte_length"]:
            raise RuntimeError(
                "Shader PBR profile completion receipt artifact byte length changed: "
                f"path={relative_path}"
            )
    return receipt


def _completion_receipt_artifacts(
    artifacts: list[object], receipt_path: Path
) -> dict[str, dict[str, int | str]]:
    if not artifacts:
        raise RuntimeError(
            "Shader PBR profile completion receipt has no artifacts: "
            f"path={receipt_path}"
        )
    expected: dict[str, dict[str, int | str]] = {}
    previous_path: str | None = None
    for artifact in artifacts:
        entry = _require_mapping_value(artifact, "completion receipt artifact", receipt_path)
        if set(entry) != {"relative_path", "sha256", "byte_length"}:
            raise RuntimeError(
                "Shader PBR profile completion receipt artifact has an unexpected schema: "
                f"path={receipt_path}"
            )
        relative_path = _require_string(entry, "relative_path", receipt_path)
        if (
            _PROFILE_ARTIFACT_RELATIVE_PATH_PATTERN.fullmatch(relative_path) is None
            or any(part in {"", ".", ".."} for part in relative_path.split("/"))
        ):
            raise RuntimeError(
                "Shader PBR profile completion receipt artifact path is unsafe: "
                f"path={relative_path}"
            )
        if relative_path in expected:
            raise RuntimeError(
                "Shader PBR profile completion receipt has a duplicate artifact: "
                f"path={relative_path}"
            )
        if previous_path is not None and relative_path <= previous_path:
            raise RuntimeError(
                "Shader PBR profile completion receipt artifacts are not in stable ordinal order: "
                f"path={receipt_path}"
            )
        expected[relative_path] = {
            "sha256": _require_sha256_identifier(entry, "sha256", receipt_path),
            "byte_length": _read_non_negative_int(entry, "byte_length", receipt_path),
        }
        previous_path = relative_path
    return expected


def _profile_root_artifacts(profile_root: Path) -> dict[str, dict[str, int | str]]:
    if not profile_root.is_dir():
        raise RuntimeError(f"Shader PBR profile root is unavailable: path={profile_root}")
    artifacts: dict[str, dict[str, int | str]] = {}
    for candidate in profile_root.rglob("*"):
        if candidate.is_symlink():
            raise RuntimeError(
                "Shader PBR profile completion receipt rejects symlinks in the profile root: "
                f"path={candidate}"
            )
        if candidate.is_dir():
            continue
        if not candidate.is_file():
            raise RuntimeError(
                "Shader PBR profile completion receipt has a non-regular artifact: "
                f"path={candidate}"
            )
        resolved_candidate = candidate.resolve()
        if not _is_path_within(profile_root, resolved_candidate):
            raise RuntimeError(
                "Shader PBR profile completion receipt artifact escapes its root: "
                f"path={candidate} root={profile_root}"
            )
        relative_path = resolved_candidate.relative_to(profile_root).as_posix()
        if _PROFILE_ARTIFACT_RELATIVE_PATH_PATTERN.fullmatch(relative_path) is None:
            raise RuntimeError(
                "Shader PBR profile completion receipt artifact path is unsafe: "
                f"path={relative_path}"
            )
        artifacts[relative_path] = {
            "sha256": _sha256_file(resolved_candidate),
            "byte_length": resolved_candidate.stat().st_size,
        }
    if not artifacts:
        raise RuntimeError(f"Shader PBR profile root has no artifacts: path={profile_root}")
    return dict(sorted(artifacts.items()))


def summarize_profile(
    summary_path: str | Path,
    *,
    completion_receipt_path: str | Path | None = None,
) -> dict[str, Any]:
    """Validate a matrix summary and return comparable cold/warm aggregates."""

    path = Path(summary_path)
    if completion_receipt_path is not None:
        validate_profile_completion_receipt(path, completion_receipt_path)
    summary = _read_json_mapping(path, "profile summary")
    if summary.get("schema_version") != 1:
        raise RuntimeError(f"Shader PBR profile summary has unsupported schema: {path}")
    if summary.get("profile_kind") != _PROFILE_KIND:
        raise RuntimeError(f"Shader PBR profile summary has an unexpected profile kind: {path}")
    repetitions = _read_positive_int(summary, "repetitions_per_mode", path)
    requested_layout, cache_contract, profile_identity = _validate_profile_identity(
        summary, path
    )
    modes = _require_mapping(summary, "modes", path)
    if set(modes) != {"cold", "warm"}:
        raise RuntimeError(f"Shader PBR profile summary must include exactly cold and warm modes: path={path}")

    mode_summaries: dict[str, dict[str, Any]] = {}
    for mode, reports in modes.items():
        if mode not in {"cold", "warm"}:
            raise RuntimeError(f"Shader PBR profile summary has an unknown mode: mode={mode!r} path={path}")
        if not isinstance(reports, list):
            raise RuntimeError(f"Shader PBR profile mode is not an array: mode={mode} path={path}")
        mode_summaries[mode] = _summarize_mode(
            mode,
            reports,
            repetitions,
            requested_layout,
            cache_contract,
            profile_identity,
            path,
        )
    active_layouts = {
        tuple(
            mode_summary["active_layout"][layout_name]
            for layout_name in _ACTIVE_LAYOUT_FIELDS
        )
        for mode_summary in mode_summaries.values()
    }
    if len(active_layouts) != 1:
        raise RuntimeError(
            f"Shader PBR profile has inconsistent active cubemap layouts across cold and warm: path={path}"
        )

    return {
        "schema_version": 1,
        "profile_kind": _PROFILE_KIND,
        "profile_summary": str(path.resolve()),
        "repetitions_per_mode": repetitions,
        "requested_layout": requested_layout,
        "material_fixture": profile_identity["material_fixture"],
        "display_visual_oracle": profile_identity["display_visual_oracle"],
        "cache_contract": cache_contract,
        "performance_qualification": _performance_qualification(cache_contract),
        "modes": mode_summaries,
        "driver_cache_note": summary.get("driver_cache_note"),
    }


def _summarize_mode(
    mode: str,
    reports: list[object],
    repetitions: int,
    requested_layout: Mapping[str, int | None],
    cache_contract: Mapping[str, object],
    profile_identity: Mapping[str, object],
    summary_path: Path,
) -> dict[str, Any]:
    expected_status = "Written" if mode == "cold" else "Reused"
    expected_ordinals = set(range(1, repetitions + 1))
    observed_ordinals: set[int] = set()
    startup_samples: dict[str, list[int]] = {name: [] for name in _STARTUP_DURATION_FIELDS}
    pso_samples: dict[str, list[int]] = {name: [] for name in _PSO_DURATION_FIELDS}
    ibl_staging_samples: dict[str, list[int]] = {
        name: [] for name in _IBL_STAGING_DURATION_FIELDS
    }
    ibl_post_stage_hydration_samples: list[int] = []
    ibl_staging_parallel_work_item_samples: dict[str, list[int]] = {
        name: [] for name in _IBL_STAGING_PARALLEL_WORK_ITEM_FIELDS
    }
    active_layouts: set[tuple[int, int, int, int]] = set()
    gpu_samples: dict[str, list[int]] = {}
    gpu_mesh_submission_samples: dict[str, list[int]] = {
        field_name: [] for field_name in _GPU_MESH_SUBMISSION_FIELDS
    }
    energy_samples: list[float] = []
    energy_statuses: set[str] = set()
    cpu_sampling_statuses: set[str] = set()
    shader_pipeline_duration_samples: dict[str, list[int]] = {
        stage: [] for stage in _SHADER_PIPELINE_STAGES
    }
    shader_pipeline_span_count_samples: dict[str, list[int]] = {
        stage: [] for stage in _SHADER_PIPELINE_STAGES
    }

    for report_object in reports:
        report = _require_mapping_value(report_object, "run report", summary_path)
        _validate_run_report(
            report,
            mode,
            expected_status,
            requested_layout,
            profile_identity,
            summary_path,
        )
        ordinal = _read_positive_int(report, "ordinal", summary_path)
        if ordinal in observed_ordinals:
            raise RuntimeError(
                f"Shader PBR profile has a duplicate measured ordinal: mode={mode} ordinal={ordinal} path={summary_path}"
            )
        observed_ordinals.add(ordinal)
        shader_pipeline_run = _validate_artifact_fingerprints(
            report,
            cache_contract,
            profile_identity,
            summary_path,
        )
        for stage in _SHADER_PIPELINE_STAGES:
            shader_pipeline_duration_samples[stage].append(
                shader_pipeline_run[stage]["duration_us"]
            )
            shader_pipeline_span_count_samples[stage].append(
                shader_pipeline_run[stage]["span_count"]
            )
        cpu_sampling = _require_mapping(_require_mapping(report, "artifacts", summary_path), "cpu_sampling", summary_path)
        cpu_sampling_statuses.add(_require_string(cpu_sampling, "status", summary_path))

        ready_sidecar = _require_mapping(report, "ready_sidecar", summary_path)
        for output_name, source_field in _STARTUP_DURATION_FIELDS.items():
            startup_samples[output_name].append(_read_u64(ready_sidecar, source_field, summary_path))
        for output_name, source_field in _PSO_DURATION_FIELDS.items():
            pso_samples[output_name].append(_read_u64(ready_sidecar, source_field, summary_path))
        for output_name, source_field in _IBL_STAGING_DURATION_FIELDS.items():
            ibl_staging_samples[output_name].append(
                _read_u64(ready_sidecar, source_field, summary_path)
            )
        ibl_post_stage_hydration_samples.append(
            _post_stage_hydration_elapsed_ns(ready_sidecar, mode, summary_path)
        )
        parallel_work_items = {
            output_name: _read_u64(ready_sidecar, source_field, summary_path)
            for output_name, source_field in _IBL_STAGING_PARALLEL_WORK_ITEM_FIELDS.items()
        }
        if _read_u64(
            ready_sidecar,
            "ibl_staging_parallel_executor_work_items",
            summary_path,
        ) != sum(parallel_work_items.values()):
            raise RuntimeError(
                "Shader PBR profile IBL parallel-work total does not match its phase attribution: "
                f"mode={mode} path={summary_path}"
            )
        for output_name, work_items in parallel_work_items.items():
            ibl_staging_parallel_work_item_samples[output_name].append(work_items)
        active_layouts.add(
            tuple(
                _read_positive_int(ready_sidecar, source_field, summary_path)
                for source_field in _ACTIVE_LAYOUT_FIELDS.values()
            )
        )

        gpu_passes, gpu_mesh_submission = _read_gpu_timing(report, summary_path)
        for pass_name, gpu_time_us in gpu_passes.items():
            gpu_samples.setdefault(pass_name, []).append(gpu_time_us)
        for field_name, value in gpu_mesh_submission.items():
            gpu_mesh_submission_samples[field_name].append(value)

        energy = _require_mapping(_require_mapping(report, "artifacts", summary_path), "energy_meter", summary_path)
        energy_status = _require_string(energy, "status", summary_path)
        energy_statuses.add(energy_status)
        if energy_status == "captured":
            energy_samples.extend(_read_energy_meter_power_samples(energy, summary_path))
        elif energy_status != "unavailable":
            raise RuntimeError(
                f"Shader PBR profile has unusable Energy Meter evidence: mode={mode} status={energy_status} path={summary_path}"
            )

    if observed_ordinals != expected_ordinals:
        raise RuntimeError(
            "Shader PBR profile measured ordinals do not match the requested matrix: "
            f"mode={mode} expected={sorted(expected_ordinals)} actual={sorted(observed_ordinals)} path={summary_path}"
        )
    if len(energy_statuses) != 1:
        raise RuntimeError(
            f"Shader PBR profile has inconsistent Energy Meter availability: mode={mode} statuses={sorted(energy_statuses)} path={summary_path}"
        )
    if cpu_sampling_statuses != {"captured"}:
        raise RuntimeError(
            f"Shader PBR profile cannot attribute startup without WPR CPU sampling: mode={mode} statuses={sorted(cpu_sampling_statuses)} path={summary_path}"
        )
    if len(active_layouts) != 1:
        raise RuntimeError(
            f"Shader PBR profile has inconsistent active cubemap layouts: mode={mode} path={summary_path}"
        )

    startup_median_ns = {name: _median(samples) for name, samples in startup_samples.items()}
    pso_median_us = {name: _median(samples) for name, samples in pso_samples.items()}
    ibl_staging_median_ns = {
        name: _median(samples) for name, samples in ibl_staging_samples.items()
    }
    ibl_staging_parallel_work_item_median = {
        name: _median(samples)
        for name, samples in ibl_staging_parallel_work_item_samples.items()
    }
    shader_pipeline_cpu = {
        "unit": "microseconds",
        "aggregation": "per_run_stage_sum_then_upper_nearest_percentile",
        "duration_semantics": "inclusive_per_span; different stages may overlap",
        "stages": {
            stage: _summarize_shader_pipeline_stage(
                shader_pipeline_duration_samples[stage],
                shader_pipeline_span_count_samples[stage],
            )
            for stage in _SHADER_PIPELINE_STAGES
        },
    }
    bottleneck_candidates = {
        "renderer_initialization": startup_median_ns["renderer_initialization"],
        "environment_brdf_lut_builtin_payload_cache_wait": startup_median_ns[
            "environment_brdf_lut_builtin_payload_cache_wait"
        ],
        "environment_brdf_lut_builtin_payload_materialization": startup_median_ns[
            "environment_brdf_lut_builtin_payload_materialization"
        ],
        "environment_brdf_lut_texture_upload_submission": startup_median_ns[
            "environment_brdf_lut_texture_upload_submission"
        ],
        "ibl_restore": startup_median_ns["ibl_restore"],
        "render_pipeline_creation_cpu": pso_median_us["render_pipeline_creation_cpu"] * 1_000,
        "shader_module_creation_cpu": pso_median_us["shader_module_creation_cpu"] * 1_000,
        "async_base_pipeline_queue_wait": pso_median_us["async_base_pipeline_queue_wait"] * 1_000,
    }
    bottleneck_candidates.update(
        {
            f"shader_pipeline.{stage}": stage_summary["per_run_duration_us"]["p50"]
            * 1_000
            for stage, stage_summary in shader_pipeline_cpu["stages"].items()
        }
    )
    bottleneck = max(bottleneck_candidates, key=bottleneck_candidates.__getitem__)
    energy_summary: dict[str, Any] = {
        "status": next(iter(energy_statuses)),
    }
    if energy_samples:
        energy_summary.update(
            {
                "scope": "meter_instance_sum",
                "unit": "watts",
                "sample_count": len(energy_samples),
                "mean_power_watts": _mean(energy_samples),
                "median_power_watts": _median(energy_samples),
            }
        )
    source_face_size, source_mip_count, pmrem_face_size, pmrem_mip_count = next(
        iter(active_layouts)
    )

    return {
        "sample_count": len(reports),
        "expected_ibl_staging_status": expected_status,
        "startup_median_ns": startup_median_ns,
        "pso_median_us": pso_median_us,
        "ibl_staging_median_ns": ibl_staging_median_ns,
        "ibl_post_stage_hydration_median_ns": _median(
            ibl_post_stage_hydration_samples
        ),
        "ibl_staging_parallel_work_item_median": ibl_staging_parallel_work_item_median,
        "active_layout": {
            "source_cubemap_face_size": source_face_size,
            "source_cubemap_mip_count": source_mip_count,
            "pmrem_face_size": pmrem_face_size,
            "pmrem_mip_count": pmrem_mip_count,
        },
        "gpu_pass_median_us": {
            pass_name: _median(samples) for pass_name, samples in sorted(gpu_samples.items())
        },
        "gpu_mesh_submission": {
            field_name: _median(samples)
            for field_name, samples in sorted(gpu_mesh_submission_samples.items())
        },
        "shader_pipeline_cpu": shader_pipeline_cpu,
        "bottleneck": bottleneck,
        "bottleneck_median_ns": bottleneck_candidates[bottleneck],
        "energy_meter": energy_summary,
        "cpu_sampling": {
            "status": next(iter(cpu_sampling_statuses)),
            "attribution_ready": next(iter(cpu_sampling_statuses)) == "captured",
        },
    }


def _validate_run_report(
    report: Mapping[str, object],
    mode: str,
    expected_status: str,
    requested_layout: Mapping[str, int | None],
    profile_identity: Mapping[str, object],
    summary_path: Path,
) -> None:
    if report.get("schema_version") != 1 or report.get("profile_kind") != _RUN_PROFILE_KIND:
        raise RuntimeError(f"Shader PBR profile has an invalid run report: mode={mode} path={summary_path}")
    if report.get("mode") != mode or report.get("role") != "measured":
        raise RuntimeError(f"Shader PBR profile has a non-measured run in its matrix: mode={mode} path={summary_path}")
    if report.get("expected_ibl_staging_status") != expected_status:
        raise RuntimeError(
            f"Shader PBR profile run expected {expected_status}: mode={mode} path={summary_path}"
        )
    ready_sidecar = _require_mapping(report, "ready_sidecar", summary_path)
    if ready_sidecar.get("schema") != _READY_SCHEMA:
        raise RuntimeError(f"Shader PBR profile has an unexpected Ready sidecar schema: mode={mode} path={summary_path}")
    if ready_sidecar.get("ibl_staging_status") != expected_status:
        raise RuntimeError(
            f"Shader PBR profile Ready sidecar expected {expected_status}: mode={mode} path={summary_path}"
        )
    expected_material_fixture = _require_string(
        profile_identity, "material_fixture", summary_path
    )
    actual_material_fixture = _require_string(
        ready_sidecar, "material_fixture", summary_path
    )
    if actual_material_fixture != expected_material_fixture:
        raise RuntimeError(
            "Shader PBR profile Ready sidecar material fixture does not match its profile manifest: "
            f"expected={expected_material_fixture} actual={actual_material_fixture} path={summary_path}"
        )
    expected_visual_oracle = profile_identity.get("display_visual_oracle")
    actual_visual_oracle = report.get("display_visual_oracle")
    if expected_visual_oracle is None:
        if actual_visual_oracle is not None:
            raise RuntimeError(
                "Shader PBR profile run declares a display visual oracle absent from its profile manifest: "
                f"path={summary_path}"
            )
    else:
        _require_matching_fingerprint(
            _require_mapping_value(
                actual_visual_oracle,
                "run display visual oracle",
                summary_path,
            ),
            _require_mapping_value(
                expected_visual_oracle,
                "profile manifest display visual oracle",
                summary_path,
            ),
            "display visual oracle",
            summary_path,
        )
    ordinal = _read_positive_int(report, "ordinal", summary_path)
    expected_run_id = (
        f"{_require_string(profile_identity, 'profile_id', summary_path)}-"
        f"{mode}-measured-{ordinal:02d}"
    )
    if _require_string(ready_sidecar, "evidence_run_id", summary_path) != expected_run_id:
        raise RuntimeError(
            "Shader PBR profile Ready sidecar evidence run id does not match its profile run: "
            f"expected={expected_run_id} path={summary_path}"
        )
    for layout_name, source_field in _REQUESTED_LAYOUT_FIELDS.items():
        expected = requested_layout[layout_name]
        if expected is None:
            actual_label = _require_string(ready_sidecar, source_field, summary_path)
            if actual_label == "automatic":
                continue
            raise RuntimeError(
                "Shader PBR profile Ready sidecar requested layout does not match its profile manifest: "
                f"field={source_field} expected=automatic actual={actual_label} path={summary_path}"
            )
        actual = _read_positive_int(ready_sidecar, source_field, summary_path)
        if actual != expected:
            raise RuntimeError(
                "Shader PBR profile Ready sidecar requested layout does not match its profile manifest: "
                f"field={source_field} expected={expected} actual={actual} path={summary_path}"
            )
    for layout_name, source_field in (
        ("requested_source_face_size", "active_source_cubemap_face_size"),
        ("requested_pmrem_face_size", "active_pmrem_face_size"),
    ):
        expected = requested_layout[layout_name]
        if expected is None:
            continue
        actual = _read_positive_int(ready_sidecar, source_field, summary_path)
        if actual != expected:
            raise RuntimeError(
                "Shader PBR profile Ready sidecar active layout does not match its requested layout: "
                f"field={source_field} expected={expected} actual={actual} path={summary_path}"
            )


def _validate_profile_identity(
    summary: Mapping[str, object], summary_path: Path
) -> tuple[dict[str, int | None], dict[str, object], dict[str, object]]:
    profile_manifest = _require_mapping(summary, "profile_manifest", summary_path)
    _validate_file_fingerprint(
        profile_manifest,
        "profile_manifest",
        summary_path,
    )
    source_binary = _require_mapping(summary, "source_binary", summary_path)
    _validate_file_fingerprint(
        source_binary,
        "source_binary",
        summary_path,
    )
    source_hdri = _require_mapping(summary, "source_hdri", summary_path)
    _validate_file_fingerprint(
        source_hdri,
        "source_hdri",
        summary_path,
    )
    manifest_path = Path(_require_string(profile_manifest, "path", summary_path))
    manifest = _read_json_mapping(manifest_path, "profile manifest")
    if manifest.get("schema_version") != 1 or manifest.get("profile_kind") != _PROFILE_MANIFEST_KIND:
        raise RuntimeError(f"Shader PBR profile manifest has an unexpected schema: path={manifest_path}")
    _require_matching_fingerprint(
        source_binary,
        _require_mapping(manifest, "binary", manifest_path),
        "binary",
        manifest_path,
    )
    manifest_input = _require_mapping(manifest, "input", manifest_path)
    _require_matching_fingerprint(
        source_hdri,
        _require_mapping(manifest_input, "hdri", manifest_path),
        "HDR input",
        manifest_path,
    )
    repository = _require_mapping(manifest, "repository", manifest_path)
    repository_root = Path(_require_string(repository, "root", manifest_path)).resolve()
    critical_sources = _require_list(repository, "critical_source_files", manifest_path)
    if not critical_sources:
        raise RuntimeError(f"Shader PBR profile manifest has no critical source files: path={manifest_path}")
    for source_object in critical_sources:
        source = _require_mapping_value(source_object, "critical source file", manifest_path)
        relative_path = Path(_require_string(source, "relative_path", manifest_path))
        if relative_path.is_absolute():
            raise RuntimeError(f"Shader PBR profile manifest critical source must be relative: path={manifest_path}")
        source_path = (repository_root / relative_path).resolve()
        if not _is_path_within(repository_root, source_path):
            raise RuntimeError(f"Shader PBR profile manifest critical source escapes repository: path={manifest_path}")
        _validate_file_fingerprint(
            {
                "path": str(source_path),
                "sha256": source.get("sha256"),
                "byte_length": source.get("byte_length"),
            },
            "critical source",
            manifest_path,
        )
    validate_profile_tool_files(repository, repository_root, manifest_path)
    build_provenance = _require_mapping(manifest, "build_provenance", manifest_path)
    _validate_build_provenance(
        build_provenance,
        manifest,
        manifest_path,
    )
    provenance_path = Path(_require_string(build_provenance, "path", manifest_path))
    provenance = _read_json_mapping(provenance_path, "viewer capture provenance")
    source_validation_ticket = _require_mapping(
        provenance, "source_validation_ticket", provenance_path
    )
    profile_id = _require_string(summary, "profile_id", summary_path)
    if _PROFILE_ID_PATTERN.fullmatch(profile_id) is None:
        raise RuntimeError(
            f"Shader PBR profile summary has an invalid profile id: path={summary_path}"
        )
    material_fixture = _require_string(manifest_input, "material_fixture", manifest_path)
    if material_fixture not in {"metal-mirror", "dielectric-ior"}:
        raise RuntimeError(
            "Shader PBR profile manifest has an unsupported material fixture: "
            f"fixture={material_fixture!r} path={manifest_path}"
        )
    display_visual_oracle = None
    capture = manifest.get("capture")
    if capture is not None:
        capture = _require_mapping(manifest, "capture", manifest_path)
        if capture.get("display_visual_oracle") is not None:
            display_visual_oracle = _require_mapping(
                capture,
                "display_visual_oracle",
                manifest_path,
            )
            _validate_file_fingerprint(
                display_visual_oracle,
                "profile_manifest_display_visual_oracle",
                manifest_path,
            )
    return (
        {
            layout_name: _read_optional_positive_int(manifest_input, source_field, manifest_path)
            for layout_name, source_field in _REQUESTED_LAYOUT_FIELDS.items()
        },
        _validate_cache_contract(manifest, manifest_path),
        {
            "profile_id": profile_id,
            "binary": _require_mapping(manifest, "binary", manifest_path),
            "hdri": _require_mapping(manifest_input, "hdri", manifest_path),
            "build_provenance": build_provenance,
            "material_fixture": material_fixture,
            "display_visual_oracle": display_visual_oracle,
            "source_manifest_sha256": _require_sha256_identifier(
                source_validation_ticket,
                "source_manifest_hash",
                provenance_path,
            ),
        },
    )


def _validate_cache_contract(
    manifest: Mapping[str, object], manifest_path: Path
) -> dict[str, object]:
    capture = manifest.get("capture")
    if not isinstance(capture, Mapping) or "cache_layers" not in capture:
        return {
            "status": "legacy_unqualified",
            "strict_cold_eligible": False,
            "comparison_scope": "unknown",
        }

    cache_layers = _require_mapping(capture, "cache_layers", manifest_path)
    if set(cache_layers) != set(_CACHE_LAYER_NAMES):
        raise RuntimeError(
            f"Shader PBR profile cache contract must define exactly the supported cache layers: path={manifest_path}"
        )
    normalized_layers: dict[str, str] = {}
    for layer_name in _CACHE_LAYER_NAMES:
        layer = _require_mapping_value(cache_layers[layer_name], "cache layer", manifest_path)
        control_state = _require_string(layer, "control_state", manifest_path)
        if control_state not in _CACHE_CONTROL_STATES:
            raise RuntimeError(
                f"Shader PBR profile cache layer has an invalid control state: layer={layer_name} path={manifest_path}"
            )
        normalized_layers[layer_name] = control_state

    strict_cold_eligible = capture.get("strict_cold_eligible")
    if not isinstance(strict_cold_eligible, bool):
        raise RuntimeError(
            f"Shader PBR profile cache contract must declare strict cold eligibility: path={manifest_path}"
        )
    comparison_scope = _require_string(capture, "comparison_scope", manifest_path)
    if comparison_scope != _CACHE_COMPARISON_SCOPE:
        raise RuntimeError(
            f"Shader PBR profile cache contract has an unexpected comparison scope: path={manifest_path}"
        )
    if strict_cold_eligible and any(
        state != "controlled" for state in normalized_layers.values()
    ):
        raise RuntimeError(
            f"Shader PBR profile cannot claim strict cold while a cache layer is uncontrolled: path={manifest_path}"
        )
    toolchain = _require_mapping(capture, "toolchain", manifest_path)
    toolchain_manifest = _require_mapping(toolchain, "manifest", manifest_path)
    _validate_file_fingerprint(
        toolchain_manifest,
        "capture toolchain manifest",
        manifest_path,
    )
    toolchain_graphics = _require_mapping(toolchain, "graphics", manifest_path)
    wgpu_backend = _require_string(toolchain_graphics, "wgpu_backend", manifest_path)
    evidence_backend = _require_string(
        toolchain_graphics, "evidence_backend", manifest_path
    )
    if wgpu_backend not in _WGPU_BACKEND_SELECTORS:
        raise RuntimeError(
            f"Shader PBR profile capture toolchain has an unsupported WGPU backend selector: path={manifest_path}"
        )
    expected_evidence_backend = f"wgpu({wgpu_backend})"
    if evidence_backend != expected_evidence_backend:
        raise RuntimeError(
            "Shader PBR profile capture toolchain evidence backend does not match "
            f"its WGPU backend: expected={expected_evidence_backend} actual={evidence_backend} path={manifest_path}"
        )
    renderdoc = toolchain.get("renderdoc")
    if renderdoc is not None:
        renderdoc = _require_mapping_value(renderdoc, "capture RenderDoc toolchain", manifest_path)
        _validate_file_fingerprint(
            _require_mapping(renderdoc, "command", manifest_path),
            "capture RenderDoc command",
            manifest_path,
        )
    machine_manifest = _validate_machine_manifest(
        _require_mapping(capture, "machine_manifest", manifest_path),
        manifest_path,
    )
    return {
        "status": "scoped",
        "strict_cold_eligible": strict_cold_eligible,
        "comparison_scope": comparison_scope,
        "layers": normalized_layers,
        "toolchain": {
            "manifest": {
                "path": _require_string(toolchain_manifest, "path", manifest_path),
                "sha256": _require_string(toolchain_manifest, "sha256", manifest_path),
                "byte_length": _read_positive_int(
                    toolchain_manifest, "byte_length", manifest_path
                ),
            },
            "wgpu_backend": wgpu_backend,
            "evidence_backend": evidence_backend,
            "renderdoc": renderdoc,
        },
        "machine_manifest": machine_manifest,
    }


def _expected_evidence_backend(
    report: Mapping[str, object],
    cache_contract: Mapping[str, object],
    summary_path: Path,
) -> str:
    reported_backend = _require_string(report, "backend", summary_path)
    if cache_contract.get("status") != "scoped":
        return reported_backend
    toolchain = _require_mapping(cache_contract, "toolchain", summary_path)
    expected_backend = _require_string(toolchain, "evidence_backend", summary_path)
    if reported_backend != expected_backend:
        raise RuntimeError(
            "Shader PBR profile run backend does not match its capture toolchain: "
            f"expected={expected_backend} actual={reported_backend} path={summary_path}"
        )
    return expected_backend


def _performance_qualification(cache_contract: Mapping[str, object]) -> dict[str, object]:
    blocking_reasons: list[str] = []
    if cache_contract.get("status") != "scoped":
        blocking_reasons.extend(
            ["cache_contract_legacy_unqualified", "machine_manifest_unavailable"]
        )
    else:
        if cache_contract.get("strict_cold_eligible") is not True:
            blocking_reasons.append("strict_cold_cache_scope")
        machine_manifest = cache_contract.get("machine_manifest")
        if (
            not isinstance(machine_manifest, Mapping)
            or machine_manifest.get("all_required_observed") is not True
        ):
            blocking_reasons.append("machine_manifest_incomplete")

    blocking_reasons.append("coordinator_comparison_receipt_missing")
    return {
        "cross_machine_baseline_eligible": False,
        "blocking_reasons": blocking_reasons,
    }


def _validate_machine_manifest(
    machine_manifest: Mapping[str, object], manifest_path: Path
) -> dict[str, object]:
    if machine_manifest.get("schema_version") != _MACHINE_MANIFEST_SCHEMA_VERSION:
        raise RuntimeError(
            f"Shader PBR profile machine manifest has an unexpected schema: path={manifest_path}"
        )
    if machine_manifest.get("manifest_kind") != _MACHINE_MANIFEST_KIND:
        raise RuntimeError(
            f"Shader PBR profile machine manifest has an unexpected kind: path={manifest_path}"
        )
    _require_string(machine_manifest, "captured_utc", manifest_path)
    categories = _require_list(machine_manifest, "required_categories", manifest_path)
    if categories != list(_MACHINE_MANIFEST_CATEGORIES):
        raise RuntimeError(
            f"Shader PBR profile machine manifest required categories do not match the contract: path={manifest_path}"
        )
    all_required_observed = machine_manifest.get("all_required_observed")
    if not isinstance(all_required_observed, bool):
        raise RuntimeError(
            f"Shader PBR profile machine manifest all_required_observed must be a boolean: path={manifest_path}"
        )
    observation_statuses: dict[str, str] = {}
    for category in _MACHINE_MANIFEST_CATEGORIES:
        observation = _require_mapping(machine_manifest, category, manifest_path)
        status = _require_string(observation, "status", manifest_path)
        if status not in _MACHINE_OBSERVATION_STATUSES:
            raise RuntimeError(
                f"Shader PBR profile machine manifest category has an invalid status: category={category} path={manifest_path}"
            )
        if status == "captured":
            data = observation.get("data")
            if not isinstance(data, list) or not data:
                raise RuntimeError(
                    f"Shader PBR profile machine manifest captured category has no data: category={category} path={manifest_path}"
                )
        else:
            _require_string(observation, "reason", manifest_path)
        observation_statuses[category] = status
    if all_required_observed != all(
        status == "captured" for status in observation_statuses.values()
    ):
        raise RuntimeError(
            f"Shader PBR profile machine manifest all_required_observed does not match category states: path={manifest_path}"
        )
    return {
        "all_required_observed": all_required_observed,
        "categories": observation_statuses,
    }


def _validate_build_provenance(
    provenance_fingerprint: Mapping[str, object],
    profile_manifest: Mapping[str, object],
    profile_manifest_path: Path,
) -> None:
    _validate_file_fingerprint(
        provenance_fingerprint,
        "build_provenance",
        profile_manifest_path,
    )
    provenance_path = Path(_require_string(provenance_fingerprint, "path", profile_manifest_path))
    provenance = _read_json_mapping(provenance_path, "viewer capture provenance")
    if (
        provenance.get("schema_version") != _MANAGED_BUILD_PROVENANCE_SCHEMA_VERSION
        or provenance.get("provenance_kind") != _MANAGED_BUILD_PROVENANCE_KIND
    ):
        raise RuntimeError(
            f"Shader PBR profile build provenance has an unexpected schema: path={provenance_path}"
        )
    _require_matching_fingerprint(
        _require_mapping(profile_manifest, "binary", profile_manifest_path),
        _require_mapping(provenance, "binary", provenance_path),
        "build provenance binary",
        provenance_path,
    )
    profile_repository = _require_mapping(profile_manifest, "repository", profile_manifest_path)
    provenance_repository = _require_mapping(provenance, "repository", provenance_path)
    profile_repository_root = Path(
        _require_string(profile_repository, "root", profile_manifest_path)
    ).resolve()
    provenance_repository_root = Path(
        _require_string(provenance_repository, "root", provenance_path)
    ).resolve()
    if provenance_repository_root != profile_repository_root:
        raise RuntimeError(
            f"Shader PBR profile build provenance repository root does not match profile manifest: path={provenance_path}"
        )
    expected_sources = {
        _require_string(source, "relative_path", profile_manifest_path): _require_string(
            source, "sha256", profile_manifest_path
        )
        for source in (
            _require_mapping_value(source_object, "critical source file", profile_manifest_path)
            for source_object in _require_list(profile_repository, "critical_source_files", profile_manifest_path)
        )
    }
    recorded_sources = _require_mapping(provenance_repository, "source_manifest", provenance_path)
    if set(recorded_sources) != set(expected_sources):
        raise RuntimeError(
            f"Shader PBR profile build provenance source manifest does not match the critical source set: path={provenance_path}"
        )
    for relative_path, expected_sha256 in expected_sources.items():
        if recorded_sources.get(relative_path) != expected_sha256:
            raise RuntimeError(
                "Shader PBR profile build provenance source manifest does not match "
                f"{relative_path}: path={provenance_path}"
            )
    ticket = _require_mapping(provenance, "source_validation_ticket", provenance_path)
    ticket_id = _require_string(ticket, "validation_ticket_id", provenance_path)
    if not re.fullmatch(r"[0-9a-f]{32}", ticket_id):
        raise RuntimeError(
            f"Shader PBR profile build provenance has an invalid coordinator validation ticket: path={provenance_path}"
        )
    if _require_string(ticket, "status", provenance_path) != "passed":
        raise RuntimeError(
            f"Shader PBR profile build provenance requires a passed coordinator validation ticket: path={provenance_path}"
        )
    receipt_sources = _require_mapping(ticket, "source_manifest", provenance_path)
    normalized_receipt_sources: dict[str, str | None] = {}
    for relative_path, recorded_sha256 in receipt_sources.items():
        if not isinstance(relative_path, str) or not re.fullmatch(r"[A-Za-z0-9._/-]+", relative_path):
            raise RuntimeError(
                f"Shader PBR profile coordinator validation ticket has an unsafe source path: path={provenance_path}"
            )
        if any(segment in {"", ".", ".."} for segment in relative_path.split("/")):
            raise RuntimeError(
                f"Shader PBR profile coordinator validation ticket has an unsafe source path: path={provenance_path}"
            )
        if recorded_sha256 is not None and (
            not isinstance(recorded_sha256, str) or not _SHA256_PATTERN.fullmatch(recorded_sha256)
        ):
            raise RuntimeError(
                f"Shader PBR profile coordinator validation ticket has an invalid source hash: path={provenance_path}"
            )
        normalized_receipt_sources[relative_path] = recorded_sha256
    canonical_receipt_manifest = json.dumps(
        dict(
            sorted(
                normalized_receipt_sources.items(),
                key=lambda item: item[0].casefold(),
            )
        ),
        sort_keys=False,
        separators=(",", ":"),
        ensure_ascii=True,
        allow_nan=False,
    ).encode("utf-8")
    receipt_manifest_hash = hashlib.sha256(canonical_receipt_manifest).hexdigest()
    if _require_sha256_identifier(ticket, "source_manifest_hash", provenance_path) != receipt_manifest_hash:
        raise RuntimeError(
            "Shader PBR profile build provenance coordinator validation ticket source manifest hash does not match "
            f"its manifest: path={provenance_path}"
        )
    for relative_path, expected_sha256 in expected_sources.items():
        if normalized_receipt_sources.get(relative_path) != expected_sha256:
            raise RuntimeError(
                "Shader PBR profile coordinator validation ticket source manifest does not bind "
                f"{relative_path}: path={provenance_path}"
            )
    _validate_managed_artifact_receipt(
        _require_mapping(provenance, "artifact_receipt", provenance_path),
        _require_mapping(profile_manifest, "binary", profile_manifest_path),
        ticket,
        provenance_path,
    )


def _validate_managed_artifact_receipt(
    receipt: Mapping[str, object],
    binary: Mapping[str, object],
    source_validation_ticket: Mapping[str, object],
    provenance_path: Path,
) -> None:
    receipt_id = _require_string(receipt, "artifact_receipt_id", provenance_path)
    if _COORDINATOR_ID_PATTERN.fullmatch(receipt_id) is None:
        raise RuntimeError(
            f"Shader PBR profile managed artifact receipt has an invalid receipt id: path={provenance_path}"
        )
    if _require_string(receipt, "status", provenance_path) != "passed":
        raise RuntimeError(
            f"Shader PBR profile managed artifact receipt requires terminal passed status: path={provenance_path}"
        )
    if _require_string(receipt, "artifact_kind", provenance_path) != _MANAGED_ARTIFACT_KIND:
        raise RuntimeError(
            f"Shader PBR profile managed artifact receipt has an unexpected artifact kind: path={provenance_path}"
        )
    for field in ("job_id", "run_id", "validation_ticket_id"):
        if _COORDINATOR_ID_PATTERN.fullmatch(
            _require_string(receipt, field, provenance_path)
        ) is None:
            raise RuntimeError(
                f"Shader PBR profile managed artifact receipt has an invalid {field}: path={provenance_path}"
            )
    ticket_id = _require_string(source_validation_ticket, "validation_ticket_id", provenance_path)
    if _require_string(receipt, "validation_ticket_id", provenance_path) != ticket_id:
        raise RuntimeError(
            "Shader PBR profile managed artifact receipt does not match its validation ticket: "
            f"path={provenance_path}"
        )
    ticket_source_manifest_hash = _require_sha256_identifier(
        source_validation_ticket, "source_manifest_hash", provenance_path
    )
    if (
        _require_sha256_identifier(receipt, "source_manifest_hash", provenance_path)
        != ticket_source_manifest_hash
    ):
        raise RuntimeError(
            "Shader PBR profile managed artifact receipt source manifest does not match its validation ticket: "
            f"path={provenance_path}"
        )
    _require_sha256_identifier(receipt, "input_manifest_hash", provenance_path)
    _require_sha256_identifier(receipt, "command_sha256", provenance_path)
    target_relative_path = _require_string(receipt, "target_relative_path", provenance_path).replace(
        "\\", "/"
    )
    if _MANAGED_TARGET_RELATIVE_PATH_PATTERN.fullmatch(target_relative_path) is None:
        raise RuntimeError(
            f"Shader PBR profile managed artifact receipt has an invalid target-relative path: path={provenance_path}"
        )
    _require_matching_fingerprint(
        binary,
        {
            "path": _require_string(receipt, "artifact_path", provenance_path),
            "sha256": _require_sha256_identifier(receipt, "sha256", provenance_path),
            "byte_length": _read_non_negative_int(receipt, "byte_length", provenance_path),
        },
        "managed artifact receipt binary",
        provenance_path,
    )
    command = _require_list(receipt, "command", provenance_path)
    if any(not isinstance(value, str) or not value for value in command):
        raise RuntimeError(
            f"Shader PBR profile managed artifact receipt command is malformed: path={provenance_path}"
        )
    command_values = [str(value) for value in command]
    if (
        len(command_values) < 7
        or re.fullmatch(r"cargo(?:\.exe)?", Path(command_values[0]).name, re.IGNORECASE) is None
        or "build" not in command_values
        or "zircon_app" not in command_values
        or "zircon_shader_pbr_viewer" not in command_values
        or "--locked" not in command_values
    ):
        raise RuntimeError(
            "Shader PBR profile managed artifact receipt does not identify the allow-listed Cargo viewer build: "
            f"path={provenance_path}"
        )


def _require_matching_fingerprint(
    summary_fingerprint: Mapping[str, object],
    manifest_fingerprint: Mapping[str, object],
    description: str,
    manifest_path: Path,
) -> None:
    summary_path = Path(_require_string(summary_fingerprint, "path", manifest_path)).resolve()
    manifest_file_path = Path(_require_string(manifest_fingerprint, "path", manifest_path)).resolve()
    if (
        summary_path != manifest_file_path
        or summary_fingerprint.get("sha256") != manifest_fingerprint.get("sha256")
        or summary_fingerprint.get("byte_length") != manifest_fingerprint.get("byte_length")
    ):
        raise RuntimeError(
            f"Shader PBR profile manifest {description} does not match profile summary: path={manifest_path}"
        )


def _is_path_within(root: Path, candidate: Path) -> bool:
    try:
        candidate.relative_to(root)
    except ValueError:
        return False
    return True


def _validate_artifact_fingerprints(
    report: Mapping[str, object],
    cache_contract: Mapping[str, object],
    profile_identity: Mapping[str, object],
    summary_path: Path,
) -> dict[str, dict[str, int]]:
    artifacts = _require_mapping(report, "artifacts", summary_path)
    for field in (
        "ready_png",
        "ready_sidecar",
        "evidence_identity",
        "ready_validation",
        "gpu_timing",
    ):
        _validate_file_fingerprint(_require_mapping(artifacts, field, summary_path), field, summary_path)
    _validate_ready_evidence(
        report,
        _expected_evidence_backend(report, cache_contract, summary_path),
        profile_identity,
        summary_path,
    )
    cpu_sampling = _require_mapping(artifacts, "cpu_sampling", summary_path)
    cpu_sampling_status = _require_string(cpu_sampling, "status", summary_path)
    if cpu_sampling_status == "captured":
        _validate_file_fingerprint(
            _require_mapping(cpu_sampling, "etl", summary_path),
            "cpu_sampling_etl",
            summary_path,
        )
    elif cpu_sampling_status != "not_requested":
        raise RuntimeError(
            f"Shader PBR profile has unusable CPU sampling evidence: status={cpu_sampling_status} path={summary_path}"
        )
    renderdoc_capture = artifacts.get("renderdoc_capture")
    if renderdoc_capture is not None:
        if cache_contract.get("status") != "scoped":
            raise RuntimeError(
                f"Shader PBR profile RenderDoc capture requires a scoped capture toolchain: path={summary_path}"
            )
        _validate_file_fingerprint(
            _require_mapping_value(renderdoc_capture, "renderdoc_capture", summary_path),
            "renderdoc_capture",
            summary_path,
        )
        renderdoc_replay = _require_mapping(artifacts, "renderdoc_replay", summary_path)
        _validate_file_fingerprint(
            renderdoc_replay,
            "renderdoc_replay",
            summary_path,
        )
        replay_path = Path(_require_string(renderdoc_replay, "path", summary_path))
        replay = _read_json_mapping(replay_path, "RenderDoc replay", summary_path)
        capture_path = Path(
            _require_string(
                _require_mapping_value(renderdoc_capture, "renderdoc_capture", summary_path),
                "path",
                summary_path,
            )
        ).resolve()
        if Path(_require_string(replay, "capture_path", replay_path)).resolve() != capture_path:
            raise RuntimeError(
                f"Shader PBR profile RenderDoc replay does not bind its capture: path={replay_path}"
            )
        toolchain = _require_mapping(cache_contract, "toolchain", summary_path)
        expected_renderdoc = _require_mapping_value(
            toolchain.get("renderdoc"),
            "capture RenderDoc toolchain",
            summary_path,
        )
        expected_command = _require_mapping(expected_renderdoc, "command", summary_path)
        report_toolchain = _require_mapping(report, "capture_toolchain", summary_path)
        report_renderdoc = _require_mapping(report_toolchain, "renderdoc", summary_path)
        _require_matching_fingerprint(
            expected_command,
            _require_mapping(report_renderdoc, "command", summary_path),
            "RenderDoc replay command",
            summary_path,
        )
        if Path(_require_string(replay, "renderdoccmd", replay_path)).resolve() != Path(
            _require_string(expected_command, "path", summary_path)
        ).resolve():
            raise RuntimeError(
                f"Shader PBR profile RenderDoc replay did not use the pinned command: path={replay_path}"
            )
        if replay.get("replay_uses_verified_snapshot") is not True or replay.get("replay_returncode") != 0:
            raise RuntimeError(
                f"Shader PBR profile RenderDoc replay did not complete its immutable validation: path={replay_path}"
            )
    energy = _require_mapping(artifacts, "energy_meter", summary_path)
    if energy.get("status") == "captured":
        energy_output_path = Path(_require_string(energy, "output_path", summary_path)).resolve()
        energy_fingerprint = _require_mapping(energy, "csv_fingerprint", summary_path)
        energy_fingerprint_path = Path(_require_string(energy_fingerprint, "path", summary_path)).resolve()
        if energy_output_path != energy_fingerprint_path:
            raise RuntimeError(
                "Shader PBR profile Energy Meter output path does not match its fingerprint: "
                f"output={energy_output_path} fingerprint={energy_fingerprint_path}"
            )
        _validate_file_fingerprint(
            energy_fingerprint,
            "energy_meter_csv",
            summary_path,
        )
    return _read_shader_pipeline_runtime_profile(report, summary_path)


def _read_shader_pipeline_runtime_profile(
    report: Mapping[str, object], summary_path: Path
) -> dict[str, dict[str, int]]:
    artifacts = _require_mapping(report, "artifacts", summary_path)
    runtime_profile = _require_mapping(artifacts, "runtime_profile", summary_path)
    if runtime_profile.get("schema") != _RUNTIME_PROFILE_SCHEMA:
        raise RuntimeError(
            f"Shader PBR runtime profile has an unexpected schema: path={summary_path}"
        )
    expected_session = _require_string(
        _require_mapping(report, "ready_sidecar", summary_path),
        "evidence_run_id",
        summary_path,
    )
    session_id = _require_string(runtime_profile, "session_id", summary_path)
    if session_id != expected_session:
        raise RuntimeError(
            "Shader PBR runtime profile session does not match its measured run: "
            f"expected={expected_session} actual={session_id} path={summary_path}"
        )
    output_root = Path(
        _require_string(runtime_profile, "output_root", summary_path)
    ).resolve()
    profile_root = summary_path.resolve().parent
    if not _is_path_within(profile_root, output_root):
        raise RuntimeError(
            "Shader PBR runtime profile output root escapes its profile root: "
            f"output_root={output_root} profile_root={profile_root}"
        )

    runtime_artifacts = _require_mapping(runtime_profile, "artifacts", summary_path)
    if set(runtime_artifacts) != set(_RUNTIME_PROFILE_ARTIFACTS):
        raise RuntimeError(
            f"Shader PBR runtime profile has an unexpected artifact closure: path={summary_path}"
        )
    resolved_artifacts: dict[str, Path] = {}
    for field, expected_name in _RUNTIME_PROFILE_ARTIFACTS.items():
        fingerprint = _require_mapping(runtime_artifacts, field, summary_path)
        _validate_file_fingerprint(
            fingerprint,
            f"runtime_profile_{field}",
            summary_path,
        )
        artifact_path = Path(_require_string(fingerprint, "path", summary_path)).resolve()
        if artifact_path.name != expected_name or not _is_path_within(
            output_root, artifact_path
        ):
            raise RuntimeError(
                "Shader PBR runtime profile artifact is outside its bound output root: "
                f"field={field} path={artifact_path}"
            )
        resolved_artifacts[field] = artifact_path

    timeline_path = resolved_artifacts["timeline"]
    timeline = _read_json_mapping(timeline_path, "runtime profile timeline")
    if _require_string(timeline, "session_id", timeline_path) != session_id:
        raise RuntimeError(
            f"Shader PBR runtime profile timeline session mismatch: path={timeline_path}"
        )
    timeline_output_root = Path(
        _require_string(timeline, "output_root", timeline_path)
    ).resolve()
    if timeline_output_root != output_root:
        raise RuntimeError(
            "Shader PBR runtime profile timeline output root does not match its run report: "
            f"path={timeline_path}"
        )
    if timeline.get("active") is not False or timeline.get("feature_enabled") is not True:
        raise RuntimeError(
            "Shader PBR runtime profile timeline is not a completed enabled capture: "
            f"path={timeline_path}"
        )

    frames = _require_list(timeline, "frames", timeline_path)
    spans = _require_list(timeline, "spans", timeline_path)
    counters = _require_list(timeline, "counters", timeline_path)
    if _read_non_negative_int(runtime_profile, "span_count", summary_path) != len(spans):
        raise RuntimeError(
            f"Shader PBR runtime profile span count does not match its timeline: path={timeline_path}"
        )
    if _read_non_negative_int(runtime_profile, "counter_count", summary_path) != len(
        counters
    ):
        raise RuntimeError(
            f"Shader PBR runtime profile counter count does not match its timeline: path={timeline_path}"
        )

    timeline_retention = _require_list(
        timeline, "recorder_retention", timeline_path
    )
    reported_retention = _require_list(
        runtime_profile, "recorder_retention", summary_path
    )
    if reported_retention != timeline_retention:
        raise RuntimeError(
            f"Shader PBR runtime profile retention does not match its timeline: path={timeline_path}"
        )
    _validate_runtime_profile_retention(
        timeline_retention,
        {"frames": len(frames), "spans": len(spans), "counters": len(counters)},
        timeline_path,
    )

    stage_counts = _require_mapping(
        runtime_profile, "shader_pipeline_stage_counts", summary_path
    )
    if set(stage_counts) != set(_SHADER_PIPELINE_STAGES):
        raise RuntimeError(
            f"Shader PBR runtime profile has an unexpected shader stage closure: path={summary_path}"
        )
    result = {
        stage: {"duration_us": 0, "span_count": 0}
        for stage in _SHADER_PIPELINE_STAGES
    }
    for span_object in spans:
        span = _require_mapping_value(
            span_object, "runtime profile span", timeline_path
        )
        category = _require_string(span, "category", timeline_path)
        name = _require_string(span, "name", timeline_path)
        if category != "shader_pipeline" or name not in result:
            continue
        result[name]["duration_us"] += _read_non_negative_int(
            span, "duration_us", timeline_path
        )
        result[name]["span_count"] += 1
    for stage in _SHADER_PIPELINE_STAGES:
        if _read_non_negative_int(stage_counts, stage, summary_path) != result[stage][
            "span_count"
        ]:
            raise RuntimeError(
                "Shader PBR runtime profile stage count does not match its timeline: "
                f"stage={stage} path={timeline_path}"
            )
    return result


def _validate_runtime_profile_retention(
    retention_records: list[object],
    retained_counts: Mapping[str, int],
    timeline_path: Path,
) -> None:
    if not retention_records:
        raise RuntimeError(
            f"Shader PBR runtime profile is missing recorder retention evidence: path={timeline_path}"
        )
    observed_retained = {stream: 0 for stream in retained_counts}
    required_fields = {
        "capacity",
        "written",
        "overwritten",
        "retained",
        "oldest_sequence",
        "newest_sequence",
    }
    for record_object in retention_records:
        record = _require_mapping_value(
            record_object, "runtime profile recorder retention", timeline_path
        )
        for stream in retained_counts:
            retention = _require_mapping(record, stream, timeline_path)
            if set(retention) != required_fields:
                raise RuntimeError(
                    "Shader PBR runtime profile retention has an unexpected schema: "
                    f"stream={stream} path={timeline_path}"
                )
            capacity = _read_positive_int(retention, "capacity", timeline_path)
            written = _read_non_negative_int(retention, "written", timeline_path)
            overwritten = _read_non_negative_int(
                retention, "overwritten", timeline_path
            )
            retained = _read_non_negative_int(retention, "retained", timeline_path)
            if overwritten != 0:
                raise RuntimeError(
                    "Shader PBR runtime profile lost "
                    f"{stream} samples: overwritten={overwritten} path={timeline_path}"
                )
            if retained > capacity or written != retained + overwritten:
                raise RuntimeError(
                    "Shader PBR runtime profile retention counters are inconsistent: "
                    f"stream={stream} path={timeline_path}"
                )
            oldest = retention.get("oldest_sequence")
            newest = retention.get("newest_sequence")
            if retained == 0:
                if oldest is not None or newest is not None:
                    raise RuntimeError(
                        "Shader PBR runtime profile empty retention has sequence bounds: "
                        f"stream={stream} path={timeline_path}"
                    )
            else:
                oldest_value = _read_non_negative_int(
                    retention, "oldest_sequence", timeline_path
                )
                newest_value = _read_non_negative_int(
                    retention, "newest_sequence", timeline_path
                )
                if newest_value < oldest_value or newest_value - oldest_value + 1 != retained:
                    raise RuntimeError(
                        "Shader PBR runtime profile retention sequence bounds are inconsistent: "
                        f"stream={stream} path={timeline_path}"
                    )
            observed_retained[stream] += retained
    if observed_retained != dict(retained_counts):
        raise RuntimeError(
            "Shader PBR runtime profile retained sample counts do not match its timeline: "
            f"path={timeline_path}"
        )


def _validate_ready_evidence(
    report: Mapping[str, object],
    expected_backend: str,
    profile_identity: Mapping[str, object],
    summary_path: Path,
) -> None:
    artifacts = _require_mapping(report, "artifacts", summary_path)
    screenshot_path = Path(_require_string(_require_mapping(artifacts, "ready_png", summary_path), "path", summary_path))
    sidecar_path = Path(_require_string(_require_mapping(artifacts, "ready_sidecar", summary_path), "path", summary_path))
    actual = _read_key_value_file(sidecar_path, "Ready sidecar", summary_path)
    expected = _require_mapping(report, "ready_sidecar", summary_path)
    for field, expected_value in expected.items():
        if actual.get(field) != str(expected_value):
            raise RuntimeError(
                "Shader PBR profile Ready sidecar does not match run report: "
                f"field={field} expected={expected_value!r} actual={actual.get(field)!r} path={sidecar_path}"
            )
    if _require_string(report, "backend", summary_path) != expected_backend:
        raise RuntimeError(
            "Shader PBR profile run backend does not match Ready validation expectation: "
            f"expected={expected_backend} path={summary_path}"
        )
    _validate_ready_identity_binding(
        expected,
        _require_mapping(artifacts, "evidence_identity", summary_path),
        profile_identity,
        summary_path,
    )
    visual_oracle_path = None
    display_visual_oracle = profile_identity.get("display_visual_oracle")
    if display_visual_oracle is not None:
        display_visual_oracle = _require_mapping_value(
            display_visual_oracle,
            "profile manifest display visual oracle",
            summary_path,
        )
        _validate_file_fingerprint(
            display_visual_oracle,
            "profile manifest display visual oracle",
            summary_path,
        )
        visual_oracle_path = Path(
            _require_string(display_visual_oracle, "path", summary_path)
        )
    try:
        replay = ready_frame_evidence_summary(
            validate_current_ready_frame_evidence(
                screenshot_path,
                expected_backend=expected_backend,
                visual_oracle_path=visual_oracle_path,
            )
        )
    except (OSError, RuntimeError, ValueError) as error:
        raise RuntimeError(
            "Shader PBR profile Ready PNG/sidecar replay validation failed: "
            f"path={screenshot_path} output={error}"
        ) from error
    saved_validation = _read_json_mapping(
        Path(
            _require_string(
                _require_mapping(artifacts, "ready_validation", summary_path),
                "path",
                summary_path,
            )
        ),
        "saved Ready validation",
    )
    for field, expected_value in {
        "schema": _READY_SCHEMA,
        "backend": expected_backend,
        "render_profile": "environment_only_pbr_preview",
    }.items():
        if replay.get(field) != expected_value or saved_validation.get(field) != expected_value:
            raise RuntimeError(
                f"Shader PBR profile Ready validation has unexpected {field}: path={screenshot_path}"
            )
    for field, expected_path in {"png": screenshot_path, "sidecar": sidecar_path}.items():
        if Path(_require_string(replay, field, screenshot_path)).resolve() != expected_path.resolve():
            raise RuntimeError(f"Shader PBR profile Ready replay {field} does not match bound evidence: path={screenshot_path}")
        if Path(_require_string(saved_validation, field, screenshot_path)).resolve() != expected_path.resolve():
            raise RuntimeError(f"Shader PBR profile saved Ready validation {field} does not match bound evidence: path={screenshot_path}")
    if saved_validation != replay:
        raise RuntimeError(
            "Shader PBR profile saved Ready validation does not match replayed evidence: "
            f"path={screenshot_path}"
        )


def _validate_ready_identity_binding(
    ready_sidecar: Mapping[str, object],
    evidence_identity: Mapping[str, object],
    profile_identity: Mapping[str, object],
    summary_path: Path,
) -> None:
    identity_path = normalize_evidence_path(
        _require_string(evidence_identity, "path", summary_path)
    ).resolve()
    if normalize_evidence_path(
        _require_string(ready_sidecar, "evidence_identity_path", summary_path)
    ).resolve() != identity_path:
        raise RuntimeError(
            "Shader PBR profile Ready sidecar identity path does not match its artifact: "
            f"path={summary_path}"
        )
    for field in ("sha256", "byte_length"):
        sidecar_field = f"evidence_identity_{field}"
        if ready_sidecar.get(sidecar_field) != str(evidence_identity.get(field)):
            raise RuntimeError(
                "Shader PBR profile Ready sidecar identity fingerprint does not match its artifact: "
                f"field={sidecar_field} path={summary_path}"
            )

    for sidecar_prefix, identity_key, include_path in (
        ("viewer_binary", "binary", True),
        ("build_provenance", "build_provenance", True),
        ("hdri", "hdri", False),
    ):
        expected = _require_mapping(profile_identity, identity_key, summary_path)
        if include_path:
            actual_path = normalize_evidence_path(
                _require_string(ready_sidecar, f"{sidecar_prefix}_path", summary_path)
            ).resolve()
            expected_path = normalize_evidence_path(
                _require_string(expected, "path", summary_path)
            ).resolve()
            if actual_path != expected_path:
                raise RuntimeError(
                    "Shader PBR profile Ready sidecar identity path does not match the profile source: "
                    f"field={sidecar_prefix}_path path={summary_path}"
                )
        for field in ("sha256", "byte_length"):
            if ready_sidecar.get(f"{sidecar_prefix}_{field}") != str(expected.get(field)):
                raise RuntimeError(
                    "Shader PBR profile Ready sidecar identity fingerprint does not match the profile source: "
                    f"field={sidecar_prefix}_{field} path={summary_path}"
                )

    if _require_string(
        ready_sidecar, "source_manifest_sha256", summary_path
    ) != _require_string(profile_identity, "source_manifest_sha256", summary_path):
        raise RuntimeError(
            "Shader PBR profile Ready sidecar source manifest does not match its profile source: "
            f"path={summary_path}"
        )


def _validate_file_fingerprint(
    fingerprint: Mapping[str, object], description: str, summary_path: Path
) -> None:
    path = Path(_require_string(fingerprint, "path", summary_path))
    if not path.is_file():
        raise RuntimeError(f"Shader PBR profile {description} is unavailable: path={path}")
    expected_sha256 = _require_string(fingerprint, "sha256", summary_path)
    if _SHA256_PATTERN.fullmatch(expected_sha256) is None:
        raise RuntimeError(f"Shader PBR profile {description} SHA-256 is malformed: path={path}")
    actual_sha256 = _sha256_file(path)
    if actual_sha256 != expected_sha256:
        raise RuntimeError(
            f"Shader PBR profile {description} SHA-256 does not match: path={path} expected={expected_sha256} actual={actual_sha256}"
        )
    expected_size = _read_non_negative_int(fingerprint, "byte_length", summary_path)
    if path.stat().st_size != expected_size:
        raise RuntimeError(
            f"Shader PBR profile {description} byte length does not match: path={path} expected={expected_size} actual={path.stat().st_size}"
        )


def _read_gpu_timing(
    report: Mapping[str, object], summary_path: Path
) -> tuple[dict[str, int], dict[str, int]]:
    artifacts = _require_mapping(report, "artifacts", summary_path)
    timing = _require_mapping(artifacts, "gpu_timing", summary_path)
    timing_path = Path(_require_string(timing, "path", summary_path))
    fields = _read_key_value_file(timing_path, "GPU timing", summary_path)
    if fields.get("schema") != _GPU_TIMING_SCHEMA or fields.get("status") != "measured":
        raise RuntimeError(f"Shader PBR profile has an invalid GPU timing report: path={timing_path}")
    standard_fields = {
        "schema",
        "status",
        "screenshot",
        "screenshot_sha256",
        "screenshot_frame_generation",
        "warmup_sample_count",
        "warmup_first_frame_generation",
        "warmup_last_frame_generation",
        "measured_sample_count",
        "first_measured_frame_generation",
        "last_measured_frame_generation",
        "timestamp_period_ns_bits",
        "timestamp_period_ns",
        "timestamp_frequency_hz",
        "percentile_policy",
        "outlier_policy",
        "pass_coverage",
    }
    unexpected_fields = sorted(
        field
        for field in fields
        if field not in standard_fields
        and _GPU_PASS_AGGREGATE_PATTERN.fullmatch(field) is None
        and _GPU_TOTAL_AGGREGATE_PATTERN.fullmatch(field) is None
        and _GPU_SAMPLE_STANDARD_PATTERN.fullmatch(field) is None
        and _GPU_SAMPLE_PASS_PATTERN.fullmatch(field) is None
        and _GPU_SAMPLE_MESH_SUBMISSION_PATTERN.fullmatch(field) is None
    )
    if unexpected_fields:
        raise RuntimeError(
            f"Shader PBR profile GPU timing report has unexpected fields: fields={', '.join(unexpected_fields)} path={timing_path}"
        )
    screenshot_path = Path(_require_string(_require_mapping(artifacts, "ready_png", summary_path), "path", summary_path))
    if fields.get("screenshot") != screenshot_path.name:
        raise RuntimeError(f"Shader PBR profile GPU timing screenshot does not match Ready PNG: path={timing_path}")
    reported_screenshot_sha256 = fields.get("screenshot_sha256")
    if reported_screenshot_sha256 is None or _SHA256_PATTERN.fullmatch(reported_screenshot_sha256) is None:
        raise RuntimeError(f"Shader PBR profile GPU timing screenshot SHA-256 is malformed: path={timing_path}")
    actual_screenshot_sha256 = _sha256_file(screenshot_path)
    if reported_screenshot_sha256 != actual_screenshot_sha256:
        raise RuntimeError(f"Shader PBR profile GPU timing screenshot SHA-256 does not match Ready PNG: path={timing_path}")
    screenshot_generation = _read_positive_int(
        fields, "screenshot_frame_generation", timing_path
    )
    warmup_count = _read_positive_int(fields, "warmup_sample_count", timing_path)
    measured_count = _read_positive_int(fields, "measured_sample_count", timing_path)
    if warmup_count != _GPU_TIMING_WARMUP_SAMPLE_COUNT or measured_count != _GPU_TIMING_MEASURED_SAMPLE_COUNT:
        raise RuntimeError(
            f"Shader PBR profile GPU timing sampling policy is invalid: path={timing_path}"
        )
    warmup_first = _read_positive_int(fields, "warmup_first_frame_generation", timing_path)
    warmup_last = _read_positive_int(fields, "warmup_last_frame_generation", timing_path)
    first_measured = _read_positive_int(fields, "first_measured_frame_generation", timing_path)
    last_measured = _read_positive_int(fields, "last_measured_frame_generation", timing_path)
    if (
        warmup_first,
        warmup_last,
        first_measured,
        last_measured,
    ) != (
        screenshot_generation + 1,
        screenshot_generation + warmup_count,
        screenshot_generation + warmup_count + 1,
        screenshot_generation + warmup_count + measured_count,
    ):
        raise RuntimeError(
            f"Shader PBR profile GPU timing generations are not consecutive: path={timing_path}"
        )
    if fields.get("percentile_policy") != "nearest_rank" or fields.get("outlier_policy") != "none_all_samples_retained":
        raise RuntimeError(
            f"Shader PBR profile GPU timing distribution policy is invalid: path={timing_path}"
        )
    pass_coverage = fields.get("pass_coverage", "").split(",")
    if not pass_coverage or pass_coverage != sorted(set(pass_coverage)):
        raise RuntimeError(
            f"Shader PBR profile GPU timing pass coverage is malformed: path={timing_path}"
        )
    missing_passes = sorted(_REQUIRED_GPU_PASSES.difference(pass_coverage))
    if missing_passes:
        raise RuntimeError(
            f"Shader PBR profile GPU timing report is missing required passes: passes={', '.join(missing_passes)} path={timing_path}"
        )
    unexpected_passes = sorted(set(pass_coverage).difference(_REQUIRED_GPU_PASSES | {"direct_realtime_ibl", "direct_ui"}))
    if unexpected_passes:
        raise RuntimeError(
            f"Shader PBR profile GPU timing report has unexpected passes: passes={', '.join(unexpected_passes)} path={timing_path}"
        )
    expected_fields = set(standard_fields)
    expected_fields.update(
        f"total.{statistic}_us" for statistic in ("min", "median", "p95", "max")
    )
    for pass_name in pass_coverage:
        expected_fields.update(
            f"pass.{pass_name}.{statistic}_us"
            for statistic in ("min", "median", "p95", "max")
        )
    for index in range(_GPU_TIMING_MEASURED_SAMPLE_COUNT):
        prefix = f"sample.{index:03}"
        expected_fields.update((f"{prefix}.frame_generation", f"{prefix}.total_us"))
        expected_fields.update(
            f"{prefix}.pass.{pass_name}_us" for pass_name in pass_coverage
        )
        expected_fields.update(
            f"{prefix}.mesh.{field_name}"
            for field_name in _GPU_MESH_SUBMISSION_FIELDS
        )
    if set(fields) != expected_fields:
        missing = sorted(expected_fields.difference(fields))
        extra = sorted(set(fields).difference(expected_fields))
        raise RuntimeError(
            "Shader PBR profile GPU timing report does not contain the exact field set: "
            f"missing={missing} unexpected={extra} path={timing_path}"
        )
    passes = {}
    for pass_name in pass_coverage:
        statistics_us = {
            statistic: _read_u64(
                fields, f"pass.{pass_name}.{statistic}_us", timing_path
            )
            for statistic in ("min", "median", "p95", "max")
        }
        if not (
            statistics_us["min"]
            <= statistics_us["median"]
            <= statistics_us["p95"]
            <= statistics_us["max"]
        ):
            raise RuntimeError(
                f"Shader PBR profile GPU timing pass distribution is unordered: pass={pass_name} path={timing_path}"
            )
        passes[pass_name] = statistics_us["median"]
    expected_generations = range(first_measured, last_measured + 1)
    mesh_submission: dict[str, int] | None = None
    for index, expected_generation in enumerate(expected_generations):
        prefix = f"sample.{index:03}"
        if _read_positive_int(fields, f"{prefix}.frame_generation", timing_path) != expected_generation:
            raise RuntimeError(
                f"Shader PBR profile GPU timing samples are not consecutive: path={timing_path}"
            )
        pass_total = sum(
            _read_u64(fields, f"{prefix}.pass.{pass_name}_us", timing_path)
            for pass_name in pass_coverage
        )
        if pass_total > _MAX_U64 or _read_u64(fields, f"{prefix}.total_us", timing_path) != pass_total:
            raise RuntimeError(
                f"Shader PBR profile GPU timing sample total is invalid: path={timing_path}"
            )
        sample_mesh_submission = {
            field_name: _read_u64(
                fields, f"{prefix}.mesh.{field_name}", timing_path
            )
            for field_name in _GPU_MESH_SUBMISSION_FIELDS
        }
        if any(value > _MAX_U32 for value in sample_mesh_submission.values()):
            raise RuntimeError(
                f"Shader PBR profile GPU timing mesh submission is out of u32 range: path={timing_path}"
            )
        if mesh_submission is None:
            mesh_submission = sample_mesh_submission
        elif mesh_submission != sample_mesh_submission:
            raise RuntimeError(
                f"Shader PBR profile GPU timing mesh submission changed during measured distribution: path={timing_path}"
            )
    if mesh_submission is None:
        raise RuntimeError(
            f"Shader PBR profile GPU timing report contains no measured mesh submission: path={timing_path}"
        )
    return passes, mesh_submission


def _read_energy_meter_power_samples(
    energy: Mapping[str, object], summary_path: Path
) -> list[float]:
    energy_path = Path(_require_string(energy, "output_path", summary_path))
    if not energy_path.is_file():
        raise RuntimeError(f"Shader PBR profile Energy Meter CSV is unavailable: path={energy_path}")
    _read_positive_int(energy, "sample_interval_seconds", summary_path)
    units = _require_list(energy, "counter_units", summary_path)
    if not any(
        isinstance(unit, Mapping)
        and unit.get("counter_suffix") == "Power"
        and unit.get("unit") == "watts"
        for unit in units
    ):
        raise RuntimeError(f"Shader PBR profile Energy Meter Power does not declare watts: path={energy_path}")
    rows = _read_energy_meter_csv_rows(energy_path, summary_path)
    if len(rows) < 3:
        raise RuntimeError(f"Shader PBR profile Energy Meter has insufficient samples: path={energy_path}")
    header = rows[0]
    power_indices = [
        index
        for index, label in enumerate(header)
        if label.strip().lower().endswith("\\power")
    ]
    if not power_indices:
        raise RuntimeError(f"Shader PBR profile Energy Meter CSV contains no Power columns: path={energy_path}")
    samples: list[float] = []
    for row in rows[1:]:
        if len(row) <= max(power_indices):
            raise RuntimeError(f"Shader PBR profile Energy Meter row is truncated: path={energy_path}")
        try:
            sample = sum(float(row[index]) for index in power_indices)
        except ValueError as error:
            raise RuntimeError(f"Shader PBR profile Energy Meter value is malformed: path={energy_path}") from error
        if not math.isfinite(sample):
            raise RuntimeError(f"Shader PBR profile Energy Meter Power value is non-finite: path={energy_path}")
        samples.append(sample)
    if len(samples) < 2:
        raise RuntimeError(f"Shader PBR profile Energy Meter has insufficient Power samples: path={energy_path}")
    return samples


def _read_energy_meter_csv_rows(energy_path: Path, summary_path: Path) -> list[list[str]]:
    try:
        contents = energy_path.read_bytes()
    except OSError as error:
        raise RuntimeError(f"Shader PBR profile Energy Meter CSV is unavailable: path={energy_path}") from error
    encoding = "utf-16" if contents.startswith((b"\xff\xfe", b"\xfe\xff")) else "utf-8-sig"
    try:
        return list(csv.reader(io.StringIO(contents.decode(encoding))))
    except UnicodeDecodeError as error:
        raise RuntimeError(
            f"Shader PBR profile Energy Meter CSV has unsupported encoding: path={energy_path} summary={summary_path}"
        ) from error


def _read_key_value_file(path: Path, description: str, summary_path: Path) -> dict[str, str]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as error:
        raise RuntimeError(f"Shader PBR profile {description} is unavailable: path={path}") from error
    fields: dict[str, str] = {}
    for line in lines:
        if not line:
            continue
        key, separator, value = line.partition("=")
        if not separator or not key or not value or key in fields:
            raise RuntimeError(f"Shader PBR profile {description} has an invalid field: path={path} summary={summary_path}")
        fields[key] = value
    return fields


def _read_json_mapping(path: Path, description: str) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise RuntimeError(f"Shader PBR {description} is unavailable or malformed: path={path}") from error
    return _require_mapping_value(value, description, path)


def _require_mapping(mapping: Mapping[str, object], field: str, path: Path) -> dict[str, object]:
    return _require_mapping_value(mapping.get(field), field, path)


def _require_mapping_value(value: object, description: str, path: Path) -> dict[str, object]:
    if not isinstance(value, dict):
        raise RuntimeError(f"Shader PBR profile requires a mapping for {description}: path={path}")
    return value


def _require_list(mapping: Mapping[str, object], field: str, path: Path) -> list[object]:
    value = mapping.get(field)
    if not isinstance(value, list):
        raise RuntimeError(f"Shader PBR profile requires an array for {field}: path={path}")
    return value


def _require_string(mapping: Mapping[str, object], field: str, path: Path) -> str:
    value = mapping.get(field)
    if not isinstance(value, str) or not value:
        raise RuntimeError(f"Shader PBR profile requires a string for {field}: path={path}")
    return value


def _require_non_empty_identifier(mapping: Mapping[str, object], field: str, path: Path) -> str:
    value = _require_string(mapping, field, path)
    if not all(character.isascii() and (character.isalnum() or character in "_-") for character in value):
        raise RuntimeError(f"Shader PBR profile identifier is malformed: field={field} path={path}")
    return value


def _require_sha256_identifier(mapping: Mapping[str, object], field: str, path: Path) -> str:
    value = _require_string(mapping, field, path)
    if _SHA256_PATTERN.fullmatch(value) is None:
        raise RuntimeError(f"Shader PBR profile SHA-256 identifier is malformed: field={field} path={path}")
    return value


def _read_positive_int(mapping: Mapping[str, object], field: str, path: Path) -> int:
    value = _read_non_negative_int(mapping, field, path)
    if value == 0:
        raise RuntimeError(f"Shader PBR profile requires a positive integer for {field}: path={path}")
    return value


def _read_optional_positive_int(
    mapping: Mapping[str, object], field: str, path: Path
) -> int | None:
    if field not in mapping:
        raise RuntimeError(f"Shader PBR profile mapping is missing field={field}: path={path}")
    if mapping[field] is None:
        return None
    return _read_positive_int(mapping, field, path)


def _read_non_negative_int(mapping: Mapping[str, object], field: str, path: Path) -> int:
    raw_value = mapping.get(field)
    value = str(raw_value) if isinstance(raw_value, int) else raw_value
    if not isinstance(value, str) or _U64_PATTERN.fullmatch(value) is None:
        raise RuntimeError(f"Shader PBR profile integer is malformed: field={field} path={path}")
    parsed = int(value)
    if parsed > _MAX_U64:
        raise RuntimeError(f"Shader PBR profile integer exceeds u64: field={field} value={parsed} path={path}")
    return parsed


def _read_u64(mapping: Mapping[str, object], field: str, path: Path) -> int:
    return _read_non_negative_int(mapping, field, path)


def _post_stage_hydration_elapsed_ns(
    ready_sidecar: Mapping[str, object], mode: str, summary_path: Path
) -> int:
    ibl_staging_elapsed_ns = _read_u64(
        ready_sidecar, "ibl_staging_elapsed_ns", summary_path
    )
    ibl_total_elapsed_ns = _read_u64(
        ready_sidecar, "ibl_total_elapsed_ns", summary_path
    )
    if ibl_total_elapsed_ns < ibl_staging_elapsed_ns:
        raise RuntimeError(
            "Shader PBR profile IBL total elapsed is shorter than staging: "
            f"mode={mode} total_ns={ibl_total_elapsed_ns} "
            f"staging_ns={ibl_staging_elapsed_ns} path={summary_path}"
        )
    return ibl_total_elapsed_ns - ibl_staging_elapsed_ns


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _median(values: Iterable[int | float]) -> int | float:
    result = statistics.median(values)
    return int(result) if isinstance(result, float) and result.is_integer() else result


def _upper_nearest_percentile(values: list[int], percentile: int) -> int:
    if not values:
        return 0
    ordered = sorted(values)
    index = math.ceil((len(ordered) - 1) * percentile / 100)
    return ordered[min(index, len(ordered) - 1)]


def _summarize_shader_pipeline_stage(
    duration_samples: list[int], span_count_samples: list[int]
) -> dict[str, object]:
    if not duration_samples or len(duration_samples) != len(span_count_samples):
        raise RuntimeError(
            "Shader PBR runtime profile stage samples are incomplete or misaligned"
        )

    def distribution(samples: list[int]) -> dict[str, int]:
        return {
            "p50": _upper_nearest_percentile(samples, 50),
            "p95": _upper_nearest_percentile(samples, 95),
            "p99": _upper_nearest_percentile(samples, 99),
            "max": max(samples),
        }

    return {
        "run_sample_count": len(duration_samples),
        "run_presence_count": sum(count > 0 for count in span_count_samples),
        "span_count": sum(span_count_samples),
        "total_duration_us": sum(duration_samples),
        "per_run_duration_us": distribution(duration_samples),
        "per_run_span_count": distribution(span_count_samples),
    }


def _mean(values: Iterable[float]) -> int | float:
    result = statistics.fmean(values)
    return int(result) if result.is_integer() else result


def _parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate and summarize Zircon PBR viewer cold/warm startup profiling evidence."
    )
    parser.add_argument("summary", type=Path, help="profile_summary.json written by the capture script")
    parser.add_argument("--output", type=Path, help="write the validated summary JSON to this path")
    parser.add_argument(
        "--completion-receipt",
        type=Path,
        help="require and replay the immutable profile completion receipt before summarizing",
    )
    return parser.parse_args()


def main() -> int:
    arguments = _parse_arguments()
    try:
        summary = summarize_profile(arguments.summary, completion_receipt_path=arguments.completion_receipt)
        serialized = json.dumps(summary, sort_keys=True, indent=2)
        if arguments.output is None:
            print(serialized)
        else:
            _write_analysis_output(arguments.summary, arguments.output, serialized)
    except (OSError, RuntimeError, ValueError) as error:
        print(f"Shader PBR profile summary failed: {error}", file=sys.stderr)
        return 1
    return 0


def _write_analysis_output(summary_path: Path, output_path: Path, serialized: str) -> None:
    resolved_summary = summary_path.resolve()
    resolved_profile_root = resolved_summary.parent
    resolved_output = output_path.resolve()
    if resolved_output == resolved_summary:
        raise RuntimeError("Shader PBR profile analysis must not overwrite profile_summary.json")
    if not _is_path_within(resolved_profile_root, resolved_output):
        raise RuntimeError(
            f"Shader PBR profile analysis output must remain under its profile root: output={resolved_output} root={resolved_profile_root}"
        )
    if resolved_output.drive.upper() == "C:":
        raise RuntimeError(f"Shader PBR profile analysis output must not be written beneath C:: output={resolved_output}")
    resolved_output.write_text(serialized + "\n", encoding="utf-8")


if __name__ == "__main__":
    raise SystemExit(main())
