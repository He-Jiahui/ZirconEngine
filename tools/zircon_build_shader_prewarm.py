"""Shader prewarm command helpers for tools/zircon_build.py."""

from __future__ import annotations

import json
from collections.abc import Mapping, Sequence
from pathlib import Path

try:
    from .zircon_build_shader_resource_registry import (
        validate_shader_resource_registry_export_contract,
    )
    from .zircon_build_shader_prewarm_report_contract import (
        parse_shader_id_record,
        shader_prewarm_dimension_summary_lines,
        shader_prewarm_report_dimension_summary_lines,
        validate_shader_prewarm_report_contract,
    )
except ImportError:  # pragma: no cover - exercised when run as a script.
    from zircon_build_shader_resource_registry import (
        validate_shader_resource_registry_export_contract,
    )
    from zircon_build_shader_prewarm_report_contract import (
        parse_shader_id_record,
        shader_prewarm_dimension_summary_lines,
        shader_prewarm_report_dimension_summary_lines,
        validate_shader_prewarm_report_contract,
    )

__all__ = (
    "shader_prewarm_dimension_summary_lines",
    "shader_prewarm_report_dimension_summary_lines",
    "validate_shader_resource_registry_export_contract",
    "validate_shader_prewarm_report_contract",
)


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


def parse_shader_geometry_source_ids(raw: Sequence[str]) -> tuple[str, ...]:
    return _unique_in_order(tuple(raw))


def parse_shader_shading_model_ids(raw: Sequence[str]) -> tuple[str, ...]:
    return _unique_in_order(tuple(raw))


def print_shader_prewarm_plan(config) -> None:
    print("  shader prewarm: enabled")
    if getattr(config, "validate_wgpu_shaders", False):
        print("  shader WGPU module validation: enabled")
    if getattr(config, "validate_wgpu_pipelines", False):
        print("  shader WGPU render pipeline validation: enabled")
    print(f"  shader quality tiers: {','.join(config.shader_quality_tiers)}")
    print(f"  shader geometry sources: {','.join(config.shader_geometry_sources)}")
    print(
        "  shader asset roots: "
        f"{','.join(str(path) for path in shader_asset_root_paths_for_prewarm(config))}"
    )
    print("  shader prewarm cache root: " f"{config.shader_prewarm_cache_root}")
    print("  shader prewarm report: " f"{config.shader_prewarm_report_path}")
    print(
        "  shader runtime fallback root: "
        f"{config.engine_root / 'cache' / 'shader_variants'}"
    )
    geometry_source_ids = shader_geometry_source_id_specs(config)
    shading_model_ids = shader_shading_model_id_specs(config)
    if geometry_source_ids:
        print(
            "  shader geometry source ids: "
            f"{','.join(geometry_source_ids)}"
        )
    if shading_model_ids:
        print(
            "  shader shading model ids: "
            f"{','.join(shading_model_ids)}"
        )
    if config.shader_permutation_registries:
        print(
            "  shader permutation registries: "
            f"{','.join(str(path) for path in config.shader_permutation_registries)}"
        )
    else:
        registry_path = generated_shader_permutation_registry_path(config)
        if registry_path:
            print(f"  shader permutation registry export: {registry_path}")
    if config.shader_resource_registry:
        print(f"  shader resource registry: {config.shader_resource_registry}")
    else:
        print(
            "  shader resource registry export: "
            f"{config.shader_prewarm_resource_registry_path}"
        )


def print_shader_prewarm_report_dimensions(report_path: Path) -> None:
    for line in shader_prewarm_report_dimension_summary_lines(report_path):
        print(line)


def validate_shader_permutation_registry_export_contract(
    registry_path: Path,
    *,
    config,
) -> None:
    registry_path = Path(registry_path)
    try:
        registry = json.loads(registry_path.read_text(encoding="utf-8"))
    except OSError as error:
        raise RuntimeError(
            "shader prewarm permutation registry export unavailable "
            f"({registry_path}: {error})"
        ) from error
    except json.JSONDecodeError as error:
        raise RuntimeError(
            "shader prewarm permutation registry export is not valid JSON "
            f"({registry_path}: {error})"
        ) from error

    if not isinstance(registry, Mapping):
        raise RuntimeError(
            "shader prewarm permutation registry export did not produce "
            "a registry object"
        )
    _validate_expected_shader_id_specs(
        registry,
        "geometry_source_ids",
        "selected shader geometry source ids",
        shader_geometry_source_id_specs(config),
    )
    _validate_expected_shader_id_specs(
        registry,
        "shading_model_ids",
        "selected shader shading model ids",
        shader_shading_model_id_specs(config),
    )
    _validate_expected_shader_modules(registry, shader_modules(config))


def _validate_expected_shader_id_specs(
    registry: Mapping[str, object],
    field: str,
    label: str,
    raw_specs: Sequence[str],
) -> None:
    if not raw_specs:
        return
    try:
        expected = {
            parse_shader_id_record(raw_spec, label) for raw_spec in tuple(raw_specs)
        }
    except ValueError as error:
        raise RuntimeError(str(error)) from error
    records = registry.get(field)
    if not isinstance(records, list):
        found: set[tuple[str, int]] = set()
    else:
        found = _shader_id_record_set(records)
    missing = [
        f"{token}={id_value}"
        for token, id_value in sorted(expected)
        if (token, id_value) not in found
    ]
    if missing:
        raise RuntimeError(
            "shader prewarm permutation registry export is missing "
            f"{label}: {', '.join(missing)}"
        )


def _shader_id_record_set(records: list[object]) -> set[tuple[str, int]]:
    ids: set[tuple[str, int]] = set()
    for record in records:
        if not isinstance(record, Mapping):
            continue
        token = record.get("token")
        id_value = record.get("id")
        if (
            isinstance(token, str)
            and isinstance(id_value, int)
            and not isinstance(id_value, bool)
        ):
            ids.add((token, id_value))
    return ids


def _validate_expected_shader_modules(
    registry: Mapping[str, object],
    expected_modules: Sequence[Mapping[str, object]],
) -> None:
    if not expected_modules:
        return
    records = registry.get("shader_modules")
    found: set[tuple[str, str]] = set()
    if isinstance(records, list):
        for record in records:
            if not isinstance(record, Mapping):
                continue
            import_path = record.get("import_path")
            content_hash = record.get("content_hash")
            if isinstance(import_path, str) and isinstance(content_hash, str):
                found.add((import_path, content_hash))
    expected: set[tuple[str, str]] = set()
    for module in expected_modules:
        import_path = module.get("import_path")
        content_hash = module.get("content_hash")
        if isinstance(import_path, str) and isinstance(content_hash, str):
            expected.add((import_path, content_hash))
    missing = [
        import_path
        for import_path, content_hash in sorted(expected)
        if (import_path, content_hash) not in found
    ]
    if missing:
        raise RuntimeError(
            "shader prewarm permutation registry export is missing "
            f"selected shader modules: {', '.join(missing)}"
        )


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
            "--builtin-fallback",
            "--pretty",
        ]
    )
    if getattr(config, "validate_wgpu_shaders", False):
        command.append("--validate-wgpu-modules")
    if getattr(config, "validate_wgpu_pipelines", False):
        command.append("--validate-wgpu-pipelines")
    for asset_root in shader_asset_root_paths_for_prewarm(config):
        command.extend(["--asset-root", str(asset_root)])
    for quality_tier in config.shader_quality_tiers:
        command.extend(["--quality-tier", quality_tier])
    for geometry_source in config.shader_geometry_sources:
        command.extend(["--geometry-source", geometry_source])
    for geometry_source_id in config.shader_geometry_source_ids:
        command.extend(["--geometry-source-id", geometry_source_id])
    for shading_model_id in config.shader_shading_model_ids:
        command.extend(["--shading-model-id", shading_model_id])
    for registry in shader_permutation_registry_paths_for_prewarm(config):
        command.extend(["--shader-permutation-registry", str(registry)])
    if config.shader_resource_registry:
        command.extend(["--resource-registry", str(config.shader_resource_registry)])
    else:
        command.extend(
            [
                "--export-resource-registry",
                str(config.shader_prewarm_resource_registry_path),
            ]
        )
    validate_shader_prewarm_command_contract(config, command)
    return command


def validate_shader_prewarm_command_contract(config, command: Sequence[str]) -> None:
    command = tuple(str(value) for value in command)
    command_flags = _command_flag_index(command)
    _require_command_flag(command_flags, "--builtin-fallback", "builtin fallback")
    _require_command_flag(command_flags, "--pretty", "pretty report output")
    _require_flag_values(
        command_flags,
        "--project-root",
        (config.engine_root,),
        "project root",
    )
    _require_flag_values(
        command_flags,
        "--cache-dir",
        (config.shader_prewarm_cache_root,),
        "staged cache root",
    )
    _require_flag_values(
        command_flags,
        "--report",
        (config.shader_prewarm_report_path,),
        "prewarm report path",
    )
    if getattr(config, "validate_wgpu_shaders", False):
        _require_command_flag(
            command_flags,
            "--validate-wgpu-modules",
            "WGPU module validation",
        )
    else:
        _forbid_command_flag(
            command_flags,
            "--validate-wgpu-modules",
            "WGPU module validation",
        )
    if getattr(config, "validate_wgpu_pipelines", False):
        _require_command_flag(
            command_flags,
            "--validate-wgpu-pipelines",
            "WGPU render pipeline validation",
        )
    else:
        _forbid_command_flag(
            command_flags,
            "--validate-wgpu-pipelines",
            "WGPU render pipeline validation",
        )
    _require_flag_values(
        command_flags,
        "--asset-root",
        shader_asset_root_paths_for_prewarm(config),
        "shader asset roots",
    )
    _require_flag_values(
        command_flags,
        "--quality-tier",
        config.shader_quality_tiers,
        "shader quality tiers",
    )
    _require_flag_values(
        command_flags,
        "--geometry-source",
        config.shader_geometry_sources,
        "shader geometry sources",
    )
    _require_flag_values(
        command_flags,
        "--geometry-source-id",
        config.shader_geometry_source_ids,
        "explicit shader geometry source ids",
    )
    _require_flag_values(
        command_flags,
        "--shading-model-id",
        config.shader_shading_model_ids,
        "explicit shader shading model ids",
    )
    _require_flag_values(
        command_flags,
        "--shader-permutation-registry",
        shader_permutation_registry_paths_for_prewarm(config),
        "shader permutation registries",
    )
    if config.shader_resource_registry:
        _require_flag_values(
            command_flags,
            "--resource-registry",
            (config.shader_resource_registry,),
            "shader resource registry input",
        )
        _require_flag_values(
            command_flags,
            "--export-resource-registry",
            (),
            "shader resource registry export",
        )
    else:
        _require_flag_values(
            command_flags,
            "--resource-registry",
            (),
            "shader resource registry input",
        )
        _require_flag_values(
            command_flags,
            "--export-resource-registry",
            (config.shader_prewarm_resource_registry_path,),
            "shader resource registry export",
        )


def shader_asset_root_paths_for_prewarm(config) -> tuple[Path, ...]:
    roots = [Path(config.engine_root) / "assets"]
    roots.extend(Path(root) for root in getattr(config, "shader_asset_roots", ()))
    for plugin in getattr(config, "plugins", ()):
        roots.extend(Path(root) for root in getattr(plugin, "asset_roots", ()))
    return _unique_path_values(roots)


def shader_permutation_registry_paths_for_prewarm(config) -> tuple[Path, ...]:
    if config.shader_permutation_registries:
        return tuple(config.shader_permutation_registries)
    registry_path = generated_shader_permutation_registry_path(config)
    if registry_path:
        return (registry_path,)
    return ()


def generated_shader_permutation_registry_path(config) -> Path | None:
    if config.shader_permutation_registries:
        return None
    if not (
        shader_geometry_source_id_specs(config)
        or shader_shading_model_id_specs(config)
        or shader_modules(config)
    ):
        return None
    return Path(config.shader_prewarm_permutation_registry_path)


def write_generated_shader_permutation_registry(config) -> Path | None:
    registry_path = generated_shader_permutation_registry_path(config)
    if registry_path is None:
        return None
    registry_path.parent.mkdir(parents=True, exist_ok=True)
    try:
        document = generated_shader_permutation_registry_document(config)
    except ValueError as error:
        raise SystemExit(str(error)) from error
    registry_path.write_text(
        json.dumps(document, indent=2) + "\n",
        encoding="utf-8",
    )
    return registry_path


def generated_shader_permutation_registry_document(config) -> dict[str, list[dict[str, object]]]:
    return {
        "geometry_source_ids": _shader_id_records(
            shader_geometry_source_id_specs(config),
            "shader geometry source id",
        ),
        "geometry_source_descriptors": shader_geometry_source_descriptors(config),
        "shading_model_ids": _shader_id_records(
            shader_shading_model_id_specs(config),
            "shader shading model id",
        ),
        "shading_model_descriptors": shader_shading_model_descriptors(config),
        "shader_modules": shader_modules(config),
    }


def shader_geometry_source_id_specs(config) -> tuple[str, ...]:
    return _combined_plugin_id_specs(config, "shader_geometry_source_ids")


def shader_shading_model_id_specs(config) -> tuple[str, ...]:
    return _combined_plugin_id_specs(config, "shader_shading_model_ids")


def shader_geometry_source_descriptors(config) -> list[dict[str, object]]:
    descriptors: list[dict[str, object]] = []
    seen: set[tuple[str, int]] = set()
    for plugin in getattr(config, "plugins", ()):
        for descriptor in getattr(plugin, "shader_geometry_source_descriptors", ()):
            if not isinstance(descriptor, Mapping):
                continue
            token = descriptor.get("token")
            id_value = descriptor.get("id")
            if (
                not isinstance(token, str)
                or isinstance(id_value, bool)
                or not isinstance(id_value, int)
            ):
                continue
            key = (token, id_value)
            if key in seen:
                continue
            seen.add(key)
            descriptors.append(dict(descriptor))
    return descriptors


def shader_shading_model_descriptors(config) -> list[dict[str, object]]:
    descriptors: list[dict[str, object]] = []
    seen: set[tuple[str, int]] = set()
    for plugin in getattr(config, "plugins", ()):
        for descriptor in getattr(plugin, "shader_shading_model_descriptors", ()):
            if not isinstance(descriptor, Mapping):
                continue
            token = descriptor.get("token")
            id_value = descriptor.get("id")
            if (
                not isinstance(token, str)
                or isinstance(id_value, bool)
                or not isinstance(id_value, int)
            ):
                continue
            key = (token, id_value)
            if key in seen:
                continue
            seen.add(key)
            descriptors.append(dict(descriptor))
    return descriptors


def shader_modules(config) -> list[dict[str, object]]:
    modules: list[dict[str, object]] = []
    seen_hashes: dict[str, str] = {}
    for plugin in getattr(config, "plugins", ()):
        for module in getattr(plugin, "shader_modules", ()):
            if not isinstance(module, Mapping):
                continue
            import_path = module.get("import_path")
            content_hash = module.get("content_hash")
            source = module.get("source")
            if not isinstance(import_path, str) or not isinstance(content_hash, str):
                continue
            existing_hash = seen_hashes.get(import_path)
            if existing_hash is not None:
                if existing_hash != content_hash:
                    raise ValueError(
                        "shader module import path "
                        f"{import_path} has multiple content hashes"
                    )
                continue
            seen_hashes[import_path] = content_hash
            record: dict[str, object] = {
                "import_path": import_path,
                "content_hash": content_hash,
            }
            if isinstance(source, str):
                record["source"] = source
            modules.append(record)
    return modules


def _shader_id_records(raw_values: Sequence[str], label: str) -> list[dict[str, object]]:
    records: list[dict[str, object]] = []
    seen: set[tuple[str, int]] = set()
    for raw_value in raw_values:
        token, id_value = parse_shader_id_record(raw_value, label)
        key = (token, id_value)
        if key in seen:
            continue
        seen.add(key)
        records.append({"token": token, "id": id_value})
    return records


def _combined_plugin_id_specs(config, field: str) -> tuple[str, ...]:
    values = list(getattr(config, field, ()))
    for plugin in getattr(config, "plugins", ()):
        values.extend(getattr(plugin, field, ()))
    return _unique_in_order(tuple(values))


def _require_command_flag(
    command_flags: Mapping[str, tuple[str, ...] | None],
    flag: str,
    label: str,
) -> None:
    if flag not in command_flags:
        raise RuntimeError(
            f"shader prewarm command missing {label} flag: {flag}"
        )


def _forbid_command_flag(
    command_flags: Mapping[str, tuple[str, ...] | None],
    flag: str,
    label: str,
) -> None:
    if flag in command_flags:
        raise RuntimeError(
            f"shader prewarm command unexpectedly enabled {label}: {flag}"
        )


def _require_flag_values(
    command_flags: Mapping[str, tuple[str, ...] | None],
    flag: str,
    expected: Sequence[object],
    label: str,
) -> None:
    actual_values = command_flags.get(flag, ())
    if actual_values is None:
        raise RuntimeError(
            f"shader prewarm command flag {flag} is missing a value"
        )
    expected_values = tuple(str(value) for value in expected)
    if actual_values != expected_values:
        raise RuntimeError(
            f"shader prewarm command {label} mismatch for {flag}: "
            f"expected {expected_values}, got {actual_values}"
        )


def _command_flag_values(command: Sequence[str], flag: str) -> tuple[str, ...]:
    values = _command_flag_index(command).get(flag, ())
    if values is None:
        raise RuntimeError(
            f"shader prewarm command flag {flag} is missing a value"
        )
    return values


def _command_flag_index(
    command: Sequence[str],
) -> dict[str, tuple[str, ...] | None]:
    values_by_flag: dict[str, list[str]] = {}
    missing_value_flags: set[str] = set()
    for index, value in enumerate(command):
        if not value.startswith("--"):
            continue
        if index + 1 >= len(command):
            missing_value_flags.add(value)
            continue
        values_by_flag.setdefault(value, []).append(str(command[index + 1]))
    command_flags: dict[str, tuple[str, ...] | None] = {
        flag: tuple(values) for flag, values in values_by_flag.items()
    }
    for flag in missing_value_flags:
        command_flags[flag] = None
    return command_flags


def _unique_in_order(values: Sequence[str]) -> tuple[str, ...]:
    seen: set[str] = set()
    ordered: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        ordered.append(value)
    return tuple(ordered)


def _unique_path_values(values: Sequence[Path]) -> tuple[Path, ...]:
    seen: set[str] = set()
    ordered: list[Path] = []
    for value in values:
        key = str(value)
        if key in seen:
            continue
        seen.add(key)
        ordered.append(value)
    return tuple(ordered)
