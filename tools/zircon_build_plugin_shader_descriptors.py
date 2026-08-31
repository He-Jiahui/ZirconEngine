"""Plugin shader contribution descriptor discovery for zircon_build."""

from __future__ import annotations

from pathlib import Path
from typing import Any, Sequence

try:
    from .zircon_build_plugin_shader_descriptor_support import (
        collect_descriptor_rows as _collect_descriptor_rows,
        descriptor_id_specs as _descriptor_id_specs,
        normalize_optional_string as _normalize_optional_string,
        shader_module_content_hash as _shader_module_content_hash,
        shader_module_source_path as _shader_module_source_path,
        unique_in_order as _unique_in_order,
    )
except ImportError:
    from zircon_build_plugin_shader_descriptor_support import (
        collect_descriptor_rows as _collect_descriptor_rows,
        descriptor_id_specs as _descriptor_id_specs,
        normalize_optional_string as _normalize_optional_string,
        shader_module_content_hash as _shader_module_content_hash,
        shader_module_source_path as _shader_module_source_path,
        unique_in_order as _unique_in_order,
    )


def collect_shader_permutation_id_specs(
    manifest_path: Path, data: dict[str, Any], field: str
) -> tuple[str, ...]:
    permutation = data.get("shader_permutation", {})
    if not permutation:
        return ()
    if not isinstance(permutation, dict):
        raise SystemExit(
            f"{manifest_path}: shader_permutation must be a TOML table when present."
        )
    entries = permutation.get(field, [])
    if not entries:
        return ()
    if not isinstance(entries, list):
        raise SystemExit(f"{manifest_path}: shader_permutation.{field} must be a list.")
    specs: list[str] = []
    for index, entry in enumerate(entries, start=1):
        if not isinstance(entry, dict):
            raise SystemExit(
                f"{manifest_path}: shader_permutation.{field}[{index}] must be a table."
            )
        token = _normalize_optional_string(entry.get("token"))
        id_value = entry.get("id")
        if token is None:
            raise SystemExit(
                f"{manifest_path}: shader_permutation.{field}[{index}].token is required."
            )
        if isinstance(id_value, bool) or not isinstance(id_value, int):
            raise SystemExit(
                f"{manifest_path}: shader_permutation.{field}[{index}].id must be an integer."
            )
        specs.append(f"{token}={id_value}")
    return tuple(_unique_in_order(specs))


def collect_shader_module_specs(
    manifest_path: Path, data: dict[str, Any]
) -> tuple[dict[str, object], ...]:
    permutation = data.get("shader_permutation", {})
    if not permutation:
        return ()
    if not isinstance(permutation, dict):
        raise SystemExit(
            f"{manifest_path}: shader_permutation must be a TOML table when present."
        )
    entries = permutation.get("shader_modules", [])
    if not entries:
        return ()
    if not isinstance(entries, list):
        raise SystemExit(
            f"{manifest_path}: shader_permutation.shader_modules must be a list."
        )
    modules: list[dict[str, object]] = []
    seen: set[str] = set()
    content_hash_by_source: dict[Path, str] = {}
    for index, entry in enumerate(entries, start=1):
        if not isinstance(entry, dict):
            raise SystemExit(
                f"{manifest_path}: shader_permutation.shader_modules[{index}] must be a table."
            )
        import_path = _normalize_optional_string(entry.get("import_path"))
        source = _normalize_optional_string(entry.get("source"))
        if import_path is None:
            raise SystemExit(
                f"{manifest_path}: shader_permutation.shader_modules[{index}].import_path is required."
            )
        if source is None:
            raise SystemExit(
                f"{manifest_path}: shader_permutation.shader_modules[{index}].source is required."
            )
        if import_path in seen:
            continue
        seen.add(import_path)
        source_path = _shader_module_source_path(manifest_path, source, index)
        if source_path not in content_hash_by_source:
            content_hash_by_source[source_path] = _shader_module_content_hash(source_path)
        modules.append(
            {
                "import_path": import_path,
                "source": source,
                "content_hash": content_hash_by_source[source_path],
            }
        )
    return tuple(modules)


def collect_geometry_source_descriptors(
    manifest_path: Path, data: dict[str, Any]
) -> tuple[dict[str, object], ...]:
    return _collect_descriptor_rows(manifest_path, data, "geometry_sources")


def collect_geometry_source_descriptor_id_specs(
    descriptors: Sequence[dict[str, object]],
) -> tuple[str, ...]:
    return _descriptor_id_specs(descriptors)


def geometry_source_descriptor_id_specs(
    descriptors: Sequence[dict[str, object]],
) -> tuple[str, ...]:
    return collect_geometry_source_descriptor_id_specs(descriptors)


def collect_shading_model_descriptors(
    manifest_path: Path, data: dict[str, Any]
) -> tuple[dict[str, object], ...]:
    return _collect_descriptor_rows(manifest_path, data, "shading_models")


def shading_model_descriptor_id_specs(
    descriptors: Sequence[dict[str, object]],
) -> tuple[str, ...]:
    return _descriptor_id_specs(descriptors)


def collect_shading_model_descriptor_id_specs(
    manifest_path: Path, data: dict[str, Any]
) -> tuple[str, ...]:
    return shading_model_descriptor_id_specs(
        collect_shading_model_descriptors(manifest_path, data)
    )
