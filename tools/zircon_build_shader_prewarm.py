"""Shader prewarm command helpers for tools/zircon_build.py."""

from __future__ import annotations

from typing import Sequence


def parse_shader_quality_tiers(raw: Sequence[str]) -> tuple[str, ...]:
    values = tuple(raw) or ("medium",)
    if "all" in values:
        return ("low", "medium", "high", "ultra")
    return _unique_in_order(values)


def parse_shader_geometry_sources(raw: Sequence[str]) -> tuple[str, ...]:
    values = tuple(raw) or ("static",)
    if "all" in values:
        return ("static", "skinned", "morphed", "skinned-morphed")
    return _unique_in_order(values)


def parse_shader_shading_model_ids(raw: Sequence[str]) -> tuple[str, ...]:
    return _unique_in_order(tuple(raw))


def print_shader_prewarm_plan(config) -> None:
    print("  shader prewarm: enabled")
    print(f"  shader quality tiers: {','.join(config.shader_quality_tiers)}")
    print(f"  shader geometry sources: {','.join(config.shader_geometry_sources)}")
    if config.shader_shading_model_ids:
        print(
            "  shader shading model ids: "
            f"{','.join(config.shader_shading_model_ids)}"
        )
    if config.shader_resource_registry:
        print(f"  shader resource registry: {config.shader_resource_registry}")


def build_shader_prewarm_command(config) -> list[str]:
    target_dir = config.targets_root / "shader_prewarm"
    command = [
        config.cargo,
        "run",
        "-p",
        "zircon_runtime",
        "--bin",
        "zircon_shader_prewarm",
        "--no-default-features",
        "--features",
        config.feature_arg_for_target("target-server"),
        "--target-dir",
        str(target_dir),
    ]
    if config.locked:
        command.append("--locked")
    if config.mode == "release":
        command.append("--release")
    elif config.mode == "profiling":
        command.extend(["--profile", "profiling"])
    if config.jobs:
        command.extend(["--jobs", config.jobs])
    command.extend(
        [
            "--",
            "--project-root",
            str(config.engine_root),
            "--cache-dir",
            str(config.shader_prewarm_cache_root),
            "--report",
            str(config.shader_prewarm_report_path),
            "--asset-root",
            str(config.engine_root / "assets"),
            "--builtin-fallback",
            "--pretty",
        ]
    )
    for quality_tier in config.shader_quality_tiers:
        command.extend(["--quality-tier", quality_tier])
    for geometry_source in config.shader_geometry_sources:
        command.extend(["--geometry-source", geometry_source])
    for shading_model_id in config.shader_shading_model_ids:
        command.extend(["--shading-model-id", shading_model_id])
    if config.shader_resource_registry:
        command.extend(["--resource-registry", str(config.shader_resource_registry)])
    return command


def _unique_in_order(values: Sequence[str]) -> tuple[str, ...]:
    seen: set[str] = set()
    ordered: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        ordered.append(value)
    return tuple(ordered)
