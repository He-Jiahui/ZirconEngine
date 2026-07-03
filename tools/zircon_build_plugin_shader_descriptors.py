"""Plugin shader contribution descriptor discovery for zircon_build."""

from __future__ import annotations

from pathlib import Path
from typing import Any, Sequence


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


def _collect_descriptor_rows(
    manifest_path: Path, data: dict[str, Any], field: str
) -> tuple[dict[str, object], ...]:
    entries = data.get(field, [])
    if not entries:
        return ()
    if not isinstance(entries, list):
        raise SystemExit(f"{manifest_path}: {field} must be a list.")
    descriptors: list[dict[str, object]] = []
    for index, entry in enumerate(entries, start=1):
        if not isinstance(entry, dict):
            raise SystemExit(f"{manifest_path}: {field}[{index}] must be a table.")
        token = _normalize_optional_string(entry.get("token"))
        id_value = entry.get("id")
        if token is None:
            raise SystemExit(f"{manifest_path}: {field}[{index}].token is required.")
        if isinstance(id_value, bool) or not isinstance(id_value, int):
            raise SystemExit(
                f"{manifest_path}: {field}[{index}].id must be an integer."
            )
        descriptors.append(dict(entry))
    return tuple(descriptors)


def _descriptor_id_specs(
    descriptors: Sequence[dict[str, object]],
) -> tuple[str, ...]:
    specs: list[str] = []
    for descriptor in descriptors:
        specs.append(f"{descriptor['token']}={descriptor['id']}")
    return tuple(_unique_in_order(specs))


def _normalize_optional_string(value: object) -> str | None:
    if value is None:
        return None
    text = str(value).strip()
    return text or None


def _unique_in_order(values: Sequence[str]) -> list[str]:
    seen: set[str] = set()
    result: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        result.append(value)
    return result
