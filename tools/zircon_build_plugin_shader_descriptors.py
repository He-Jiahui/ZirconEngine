"""Plugin shader contribution descriptor discovery for zircon_build."""

from __future__ import annotations

import hashlib
from pathlib import Path, PurePosixPath
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
        modules.append(
            {
                "import_path": import_path,
                "source": source,
                "content_hash": _shader_module_content_hash(source_path),
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


def _shader_module_source_path(manifest_path: Path, source: str, index: int) -> Path:
    if Path(source).is_absolute():
        raise SystemExit(
            f"{manifest_path}: shader_permutation.shader_modules[{index}].source must be package-relative."
        )
    if "\\" in source:
        raise SystemExit(
            f"{manifest_path}: shader_permutation.shader_modules[{index}].source must use forward slashes."
        )
    posix_path = PurePosixPath(source)
    if posix_path.is_absolute() or any(part in {"", ".", ".."} for part in posix_path.parts):
        raise SystemExit(
            f"{manifest_path}: shader_permutation.shader_modules[{index}].source must be package-relative."
        )
    if posix_path.suffix not in {".zshader", ".wgsl"}:
        raise SystemExit(
            f"{manifest_path}: shader_permutation.shader_modules[{index}].source must end with .zshader or .wgsl."
        )
    source_path = manifest_path.parent / source
    if not source_path.is_file():
        raise SystemExit(
            f"{manifest_path}: shader_permutation.shader_modules[{index}].source does not exist: {source}"
        )
    return source_path


def _shader_module_content_hash(source_path: Path) -> str:
    return hashlib.sha256(source_path.read_bytes()).hexdigest()


def _unique_in_order(values: Sequence[str]) -> list[str]:
    seen: set[str] = set()
    result: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        result.append(value)
    return result
