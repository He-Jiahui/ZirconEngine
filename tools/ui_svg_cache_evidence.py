"""Fail-closed evidence gate for retained SVG CPU and GPU products."""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
import re
from typing import Any


SCHEMA_VERSION = 2
DEVICE_IMAGE_BUDGET_BYTES = 64 * 1024 * 1024
REQUIRED_SOURCE_PATHS = (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/svg/cache.rs",
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/svg/pixels.rs",
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/loading/cache.rs",
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/loading/async_loader.rs",
    "zircon_editor/src/ui/retained_host/app/assets/refresh.rs",
    "zircon_editor/src/ui/retained_host/ui_perf.rs",
    "zircon_editor/src/ui/retained_host/ui_perf/counter_catalog.rs",
    "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/stats.rs",
    "zircon_runtime/crates/zr_rhi/src/ui_surface.rs",
    "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs",
    "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/presentation.rs",
    "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shared_image_registry.rs",
    "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shared_image_registry/allocation_ledger.rs",
    "tools/ui-profile-counter-evidence.ps1",
)
_WORK_COUNTER_SUFFIXES = (
    "visual_asset_cache_miss_count",
    "visual_asset_cache_candidate_build_count",
    "svg_tree_cache_miss_count",
    "svg_parse_count",
    "svg_parse_bytes",
    "svg_raster_count",
    "svg_raster_pixels",
    "svg_raster_product_miss_count",
    "svg_raster_unique_bucket_count",
    "visual_asset_async_stale_discard_count",
    "gpu_image_prepare_command_visits",
    "gpu_image_upload_writes",
    "gpu_image_shared_upload_writes",
    "gpu_image_cache_key_allocations",
)
_HIT_COUNTER_SUFFIXES = {
    "visual_asset": "visual_asset_cache_hit_count",
    "svg_tree": "svg_tree_cache_memory_hit_count",
    "raster_product": "svg_raster_product_hit_count",
    "gpu_prepare": "gpu_image_prepare_cache_hits",
}
_DEVICE_GAUGE_SUFFIXES = (
    "gpu_image_device_allocation_count",
    "gpu_image_device_allocation_bytes",
    "gpu_image_registry_evicted_pinned_bytes",
    "gpu_image_surface_pin_count",
    "gpu_image_in_flight_present_pin_count",
)
_DEVICE_CUMULATIVE_SUFFIXES = ("gpu_image_eviction_completion_count",)
_SPAN_NAMES = {
    "svg_parse": "visual_assets_render_svg_parse",
    "svg_raster": "visual_assets_render_svg_raster",
}
_HEX_40 = re.compile(r"[0-9a-fA-F]{40}\Z")
_HEX_64 = re.compile(r"[0-9a-fA-F]{64}\Z")


def evaluate_stable_svg_cache_evidence(
    timeline: dict[str, object],
    scenario: str,
    *,
    require_quiescent: bool = False,
) -> dict[str, object]:
    """Evaluate one stable UI phase without treating absent counters as zero."""

    prefix = f"ui.{scenario}."
    required_counters = tuple(
        f"{prefix}{suffix}"
        for suffix in (*_WORK_COUNTER_SUFFIXES, *_HIT_COUNTER_SUFFIXES.values())
    )
    values, blockers = _collect_counters(timeline, required_counters)

    for suffix in _WORK_COUNTER_SUFFIXES:
        name = f"{prefix}{suffix}"
        value = values.get(name)
        if value is not None and value != 0:
            blockers.append(
                {
                    "code": "stable_svg_work_detected",
                    "counter": name,
                    "value": value,
                }
            )

    retained_hits: dict[str, int | None] = {}
    for label, suffix in _HIT_COUNTER_SUFFIXES.items():
        name = f"{prefix}{suffix}"
        value = values.get(name)
        retained_hits[label] = value
        if value is not None and value == 0:
            blockers.append(
                {
                    "code": "missing_retained_hit",
                    "counter": name,
                    "value": value,
                }
            )

    device_residency, device_blockers = _evaluate_device_residency(
        timeline,
        prefix,
        require_quiescent=require_quiescent,
        require_retained_allocation=(retained_hits.get("gpu_prepare") or 0) > 0,
    )
    blockers.extend(device_blockers)

    return {
        "schema_version": SCHEMA_VERSION,
        "scenario": scenario,
        "ready": not blockers,
        "blockers": blockers,
        "retained_hits": retained_hits,
        "device_residency": device_residency,
        "capture_wide_spans": _summarize_legacy_spans(timeline),
    }


def _collect_counters(
    timeline: dict[str, object], required_names: tuple[str, ...]
) -> tuple[dict[str, int], list[dict[str, object]]]:
    series, blockers = _collect_counter_series(timeline, required_names)
    return {name: sum(raw_values) for name, raw_values in series.items()}, blockers


def _collect_counter_series(
    timeline: dict[str, object], required_names: tuple[str, ...]
) -> tuple[dict[str, list[int]], list[dict[str, object]]]:
    samples: dict[str, list[object]] = {name: [] for name in required_names}
    raw_counters = timeline.get("counters", [])
    if isinstance(raw_counters, list):
        for counter in raw_counters:
            if not isinstance(counter, dict):
                continue
            name = counter.get("name")
            if isinstance(name, str) and name in samples:
                samples[name].append(counter.get("value"))

    values: dict[str, list[int]] = {}
    blockers: list[dict[str, object]] = []
    for name in required_names:
        raw_values = samples[name]
        if not raw_values:
            blockers.append({"code": "missing_counter", "counter": name})
            continue
        if any(not _is_non_negative_integer(value) for value in raw_values):
            blockers.append(
                {
                    "code": "invalid_counter_value",
                    "counter": name,
                    "values": raw_values,
                }
            )
            continue
        values[name] = [int(value) for value in raw_values]
    return values, blockers


def _evaluate_device_residency(
    timeline: dict[str, object],
    prefix: str,
    *,
    require_quiescent: bool,
    require_retained_allocation: bool,
) -> tuple[dict[str, object], list[dict[str, object]]]:
    suffixes = (*_DEVICE_GAUGE_SUFFIXES, *_DEVICE_CUMULATIVE_SUFFIXES)
    required_names = tuple(f"{prefix}{suffix}" for suffix in suffixes)
    series, blockers = _collect_counter_series(timeline, required_names)
    summary: dict[str, object] = {
        "device_budget_bytes": DEVICE_IMAGE_BUDGET_BYTES,
        "require_quiescent": require_quiescent,
        "gauges": {},
        "cumulative": {},
    }
    if len(series) != len(required_names):
        return summary, blockers

    sample_counts = {len(values) for values in series.values()}
    if len(sample_counts) != 1:
        blockers.append(
            {
                "code": "misaligned_device_counter_samples",
                "sample_counts": {
                    name: len(values) for name, values in series.items()
                },
            }
        )
        return summary, blockers

    gauges = {
        suffix: series[f"{prefix}{suffix}"] for suffix in _DEVICE_GAUGE_SUFFIXES
    }
    cumulative = {
        suffix: series[f"{prefix}{suffix}"]
        for suffix in _DEVICE_CUMULATIVE_SUFFIXES
    }
    summary["gauges"] = {
        suffix: {"peak": max(values), "final": values[-1]}
        for suffix, values in gauges.items()
    }
    summary["cumulative"] = {
        suffix: {
            "initial": values[0],
            "final": values[-1],
            "delta": values[-1] - values[0],
        }
        for suffix, values in cumulative.items()
    }

    allocation_counts = gauges["gpu_image_device_allocation_count"]
    allocation_bytes = gauges["gpu_image_device_allocation_bytes"]
    evicted_pinned_bytes = gauges["gpu_image_registry_evicted_pinned_bytes"]
    surface_pin_counts = gauges["gpu_image_surface_pin_count"]
    in_flight_pin_counts = gauges["gpu_image_in_flight_present_pin_count"]
    for sample_index, (count, byte_count, evicted, surface_pins, in_flight) in enumerate(
        zip(
            allocation_counts,
            allocation_bytes,
            evicted_pinned_bytes,
            surface_pin_counts,
            in_flight_pin_counts,
            strict=True,
        )
    ):
        if byte_count > DEVICE_IMAGE_BUDGET_BYTES:
            blockers.append(
                {
                    "code": "device_allocation_budget_exceeded",
                    "counter": f"{prefix}gpu_image_device_allocation_bytes",
                    "sample_index": sample_index,
                    "value": byte_count,
                    "budget_bytes": DEVICE_IMAGE_BUDGET_BYTES,
                }
            )
        if evicted > byte_count or ((count == 0) != (byte_count == 0)):
            blockers.append(
                {
                    "code": "invalid_device_allocation_relationship",
                    "sample_index": sample_index,
                    "allocation_count": count,
                    "allocation_bytes": byte_count,
                    "registry_evicted_pinned_bytes": evicted,
                }
            )
        if count == 0 and (surface_pins != 0 or in_flight != 0):
            blockers.append(
                {
                    "code": "invalid_device_pin_relationship",
                    "sample_index": sample_index,
                    "allocation_count": count,
                    "surface_pin_count": surface_pins,
                    "in_flight_present_pin_count": in_flight,
                }
            )

    if require_retained_allocation and (
        max(allocation_counts) == 0
        or max(allocation_bytes) == 0
        or max(surface_pin_counts) == 0
    ):
        blockers.append(
            {
                "code": "missing_retained_device_allocation",
                "detail": "GPU prepare hits require a retained ledger allocation and surface pin",
            }
        )

    for suffix, values in cumulative.items():
        if any(current < previous for previous, current in zip(values, values[1:])):
            blockers.append(
                {
                    "code": "non_monotonic_device_counter",
                    "counter": f"{prefix}{suffix}",
                    "values": values,
                }
            )

    if require_quiescent:
        for suffix in (
            "gpu_image_registry_evicted_pinned_bytes",
            "gpu_image_in_flight_present_pin_count",
        ):
            values = gauges[suffix]
            if values[-1] != 0:
                blockers.append(
                    {
                        "code": "device_residency_not_quiescent",
                        "counter": f"{prefix}{suffix}",
                        "value": values[-1],
                    }
                )

    return summary, blockers


def _is_non_negative_integer(value: object) -> bool:
    return (
        not isinstance(value, bool)
        and isinstance(value, (int, float))
        and math.isfinite(value)
        and value >= 0
        and value == math.floor(value)
    )


def _summarize_legacy_spans(timeline: dict[str, object]) -> dict[str, object]:
    summary: dict[str, object] = {
        "authority": "diagnostic_only_not_scenario_attribution"
    }
    raw_spans = timeline.get("spans", [])
    spans = raw_spans if isinstance(raw_spans, list) else []
    for label, expected_name in _SPAN_NAMES.items():
        count = 0
        duration_us = 0
        for span in spans:
            if not isinstance(span, dict) or span.get("name") != expected_name:
                continue
            count += 1
            duration = span.get("duration_us")
            if _is_non_negative_integer(duration):
                duration_us += int(duration)
        summary[label] = {"count": count, "duration_us": duration_us}
    return summary


def validate_source_manifest(
    manifest: dict[str, object], scenario: str
) -> list[dict[str, object]]:
    """Validate source and measured-run binding for an SVG cache result."""

    blockers: list[dict[str, object]] = []
    if manifest.get("scenario") != scenario:
        blockers.append(
            {
                "code": "invalid_capture_contract",
                "detail": "source manifest scenario does not match the evidence scenario",
            }
        )

    repository = manifest.get("repository")
    repository = repository if isinstance(repository, dict) else {}
    git_binding = repository.get("git")
    git_binding = git_binding if isinstance(git_binding, dict) else {}
    if _HEX_40.fullmatch(str(git_binding.get("revision", ""))) is None:
        blockers.append(
            {"code": "invalid_source_binding", "detail": "missing git revision"}
        )
    if _HEX_64.fullmatch(str(git_binding.get("dirty_tree_sha256", ""))) is None:
        blockers.append(
            {
                "code": "invalid_source_binding",
                "detail": "missing dirty-tree fingerprint",
            }
        )

    raw_sources = repository.get("critical_source_files")
    sources = raw_sources if isinstance(raw_sources, list) else []
    sources_by_path = {
        str(source.get("relative_path", "")).replace("\\", "/"): source
        for source in sources
        if isinstance(source, dict)
    }
    for path in REQUIRED_SOURCE_PATHS:
        source = sources_by_path.get(path)
        if source is None:
            blockers.append(
                {"code": "missing_critical_source", "relative_path": path}
            )
            continue
        if (
            _HEX_64.fullmatch(str(source.get("sha256", ""))) is None
            or not _is_positive_integer(source.get("byte_length"))
        ):
            blockers.append(
                {"code": "invalid_source_fingerprint", "relative_path": path}
            )

    capture = manifest.get("capture")
    capture = capture if isinstance(capture, dict) else {}
    options = capture.get("options")
    options = options if isinstance(options, dict) else {}
    ordinal = options.get("run_ordinal")
    measured_count = options.get("measured_run_count")
    if (
        options.get("run_phase") != "measured"
        or not _is_positive_integer(ordinal)
        or not _is_positive_integer(measured_count)
        or int(ordinal) > int(measured_count)
    ):
        blockers.append(
            {
                "code": "invalid_capture_contract",
                "detail": "evidence must bind a valid measured run ordinal",
            }
        )
    return blockers


def build_svg_cache_report(
    timeline: dict[str, object],
    scenario: str,
    source_manifest: dict[str, object] | None,
    *,
    require_quiescent: bool = False,
) -> dict[str, object]:
    result = evaluate_stable_svg_cache_evidence(
        timeline, scenario, require_quiescent=require_quiescent
    )
    if source_manifest is None:
        result["blockers"].append(
            {
                "code": "missing_source_manifest",
                "detail": "diagnostic output is not source-bound acceptance evidence",
            }
        )
    else:
        result["blockers"].extend(
            validate_source_manifest(source_manifest, scenario)
        )
    result["ready"] = not result["blockers"]
    return result


def _is_positive_integer(value: object) -> bool:
    return _is_non_negative_integer(value) and int(value) > 0


def validate_output_path(path: Path) -> Path:
    resolved = path.resolve()
    if resolved.drive.upper() not in {"D:", "E:", "F:"}:
        raise ValueError("SVG cache evidence must be written below D:, E:, or F:.")
    return resolved


def _read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8-sig"))
    if not isinstance(value, dict):
        raise ValueError(f"expected a JSON object: {path}")
    return value


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--timeline", type=Path, required=True)
    parser.add_argument("--source-manifest", type=Path)
    parser.add_argument("--scenario", required=True)
    parser.add_argument("--require-quiescent", action="store_true")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    output_path = validate_output_path(args.output)
    source_manifest = (
        _read_json(args.source_manifest) if args.source_manifest is not None else None
    )
    result = build_svg_cache_report(
        _read_json(args.timeline),
        args.scenario,
        source_manifest,
        require_quiescent=args.require_quiescent,
    )
    result["source_manifest"] = (
        str(args.source_manifest.resolve())
        if args.source_manifest is not None
        else None
    )
    result["timeline"] = str(args.timeline.resolve())
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(json.dumps({"ready": result["ready"], "output": str(output_path)}))
    return 0 if result["ready"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
