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
    from tools.zircon_validate_shader_pbr_viewer_evidence import (
        ready_frame_evidence_summary,
        validate_current_ready_frame_evidence,
    )
except ModuleNotFoundError:
    from zircon_validate_shader_pbr_viewer_evidence import (
        ready_frame_evidence_summary,
        validate_current_ready_frame_evidence,
    )


_PROFILE_KIND = "zircon_shader_pbr_viewer_startup_matrix"
_RUN_PROFILE_KIND = "zircon_shader_pbr_viewer_startup_run"
_PROFILE_MANIFEST_KIND = "zircon_shader_pbr_viewer_startup"
_READY_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v12"
_GPU_TIMING_SCHEMA = "zircon_shader_pbr_viewer_gpu_timing_evidence_v1"
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
_MAX_U64 = (1 << 64) - 1
_STARTUP_DURATION_FIELDS = {
    "renderer_initialization": "scene_startup_renderer_initialization_ns",
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


def summarize_profile(summary_path: str | Path) -> dict[str, Any]:
    """Validate a matrix summary and return comparable cold/warm aggregates."""

    path = Path(summary_path)
    summary = _read_json_mapping(path, "profile summary")
    if summary.get("schema_version") != 1:
        raise RuntimeError(f"Shader PBR profile summary has unsupported schema: {path}")
    if summary.get("profile_kind") != _PROFILE_KIND:
        raise RuntimeError(f"Shader PBR profile summary has an unexpected profile kind: {path}")
    repetitions = _read_positive_int(summary, "repetitions_per_mode", path)
    requested_layout = _validate_profile_identity(summary, path)
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
            mode, reports, repetitions, requested_layout, path
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
        "modes": mode_summaries,
        "driver_cache_note": summary.get("driver_cache_note"),
    }


def _summarize_mode(
    mode: str,
    reports: list[object],
    repetitions: int,
    requested_layout: Mapping[str, int | None],
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
    energy_samples: list[float] = []
    energy_statuses: set[str] = set()
    cpu_sampling_statuses: set[str] = set()

    for report_object in reports:
        report = _require_mapping_value(report_object, "run report", summary_path)
        _validate_run_report(report, mode, expected_status, requested_layout, summary_path)
        ordinal = _read_positive_int(report, "ordinal", summary_path)
        if ordinal in observed_ordinals:
            raise RuntimeError(
                f"Shader PBR profile has a duplicate measured ordinal: mode={mode} ordinal={ordinal} path={summary_path}"
            )
        observed_ordinals.add(ordinal)
        _validate_artifact_fingerprints(report, summary_path)
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

        for pass_name, gpu_time_us in _read_gpu_passes(report, summary_path).items():
            gpu_samples.setdefault(pass_name, []).append(gpu_time_us)

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
    bottleneck_candidates = {
        "renderer_initialization": startup_median_ns["renderer_initialization"],
        "ibl_restore": startup_median_ns["ibl_restore"],
        "render_pipeline_creation_cpu": pso_median_us["render_pipeline_creation_cpu"] * 1_000,
        "shader_module_creation_cpu": pso_median_us["shader_module_creation_cpu"] * 1_000,
        "async_base_pipeline_queue_wait": pso_median_us["async_base_pipeline_queue_wait"] * 1_000,
    }
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
) -> dict[str, int | None]:
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
    _validate_build_provenance(
        _require_mapping(manifest, "build_provenance", manifest_path),
        manifest,
        manifest_path,
    )
    return {
        layout_name: _read_optional_positive_int(manifest_input, source_field, manifest_path)
        for layout_name, source_field in _REQUESTED_LAYOUT_FIELDS.items()
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
        provenance.get("schema_version") != 1
        or provenance.get("provenance_kind") != "zircon_local_viewer_capture_provenance"
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
    profile_git = _require_mapping(profile_repository, "git", profile_manifest_path)
    provenance_repository = _require_mapping(provenance, "repository", provenance_path)
    if _require_string(provenance_repository, "git_revision", provenance_path) != _require_string(
        profile_git, "revision", profile_manifest_path
    ):
        raise RuntimeError(
            f"Shader PBR profile build provenance Git revision does not match profile manifest: path={provenance_path}"
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
        normalized_receipt_sources,
        sort_keys=True,
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


def _validate_artifact_fingerprints(report: Mapping[str, object], summary_path: Path) -> None:
    artifacts = _require_mapping(report, "artifacts", summary_path)
    for field in ("ready_png", "ready_sidecar", "ready_validation", "gpu_timing"):
        _validate_file_fingerprint(_require_mapping(artifacts, field, summary_path), field, summary_path)
    _validate_ready_evidence(report, summary_path)
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
        _validate_file_fingerprint(
            _require_mapping_value(renderdoc_capture, "renderdoc_capture", summary_path),
            "renderdoc_capture",
            summary_path,
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


def _validate_ready_evidence(report: Mapping[str, object], summary_path: Path) -> None:
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
    try:
        replay = ready_frame_evidence_summary(
            validate_current_ready_frame_evidence(screenshot_path, expected_backend="Dx12")
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
        "backend": "Dx12",
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


def _read_gpu_passes(report: Mapping[str, object], summary_path: Path) -> dict[str, int]:
    artifacts = _require_mapping(report, "artifacts", summary_path)
    timing = _require_mapping(artifacts, "gpu_timing", summary_path)
    timing_path = Path(_require_string(timing, "path", summary_path))
    fields = _read_key_value_file(timing_path, "GPU timing", summary_path)
    if fields.get("schema") != _GPU_TIMING_SCHEMA or fields.get("status") != "measured":
        raise RuntimeError(f"Shader PBR profile has an invalid GPU timing report: path={timing_path}")
    standard_fields = {"schema", "status", "screenshot", "screenshot_sha256", "frame_generation"}
    unexpected_fields = sorted(
        field for field in fields if field not in standard_fields and not field.startswith("pass.")
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
    _read_positive_int(fields, "frame_generation", timing_path)
    passes = {
        field.removeprefix("pass."): _read_u64(fields, field, timing_path)
        for field in fields
        if field.startswith("pass.")
    }
    missing_passes = sorted(_REQUIRED_GPU_PASSES.difference(passes))
    if missing_passes:
        raise RuntimeError(
            f"Shader PBR profile GPU timing report is missing required passes: passes={', '.join(missing_passes)} path={timing_path}"
        )
    unexpected_passes = sorted(set(passes).difference(_REQUIRED_GPU_PASSES | {"direct_realtime_ibl", "direct_ui"}))
    if unexpected_passes:
        raise RuntimeError(
            f"Shader PBR profile GPU timing report has unexpected passes: passes={', '.join(unexpected_passes)} path={timing_path}"
        )
    return passes


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


def _mean(values: Iterable[float]) -> int | float:
    result = statistics.fmean(values)
    return int(result) if result.is_integer() else result


def _parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate and summarize Zircon PBR viewer cold/warm startup profiling evidence."
    )
    parser.add_argument("summary", type=Path, help="profile_summary.json written by the capture script")
    parser.add_argument("--output", type=Path, help="write the validated summary JSON to this path")
    return parser.parse_args()


def main() -> int:
    arguments = _parse_arguments()
    try:
        summary = summarize_profile(arguments.summary)
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
