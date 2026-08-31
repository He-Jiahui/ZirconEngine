"""Private validation helpers for plugin shader contribution descriptors."""

from __future__ import annotations

from pathlib import Path, PurePosixPath
from typing import Sequence

try:
    from .zircon_export.file_digest import file_sha256
except ImportError:  # pragma: no cover - exercised when zircon_build.py is run directly.
    from zircon_export.file_digest import file_sha256


def collect_descriptor_rows(
    manifest_path: Path, data: dict[str, object], field: str
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
        token = normalize_optional_string(entry.get("token"))
        id_value = entry.get("id")
        if token is None:
            raise SystemExit(f"{manifest_path}: {field}[{index}].token is required.")
        if isinstance(id_value, bool) or not isinstance(id_value, int):
            raise SystemExit(
                f"{manifest_path}: {field}[{index}].id must be an integer."
            )
        descriptors.append(dict(entry))
    return tuple(descriptors)


def descriptor_id_specs(
    descriptors: Sequence[dict[str, object]],
) -> tuple[str, ...]:
    return tuple(
        unique_in_order(
            [f"{descriptor['token']}={descriptor['id']}" for descriptor in descriptors]
        )
    )


def normalize_optional_string(value: object) -> str | None:
    if value is None:
        return None
    text = str(value).strip()
    return text or None


def shader_module_source_path(manifest_path: Path, source: str, index: int) -> Path:
    if Path(source).is_absolute():
        raise SystemExit(
            f"{manifest_path}: shader_permutation.shader_modules[{index}].source must be package-relative."
        )
    if "\\" in source:
        raise SystemExit(
            f"{manifest_path}: shader_permutation.shader_modules[{index}].source must use forward slashes."
        )
    posix_path = PurePosixPath(source)
    if posix_path.is_absolute() or any(
        part in {"", ".", ".."} for part in posix_path.parts
    ):
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


def shader_module_content_hash(source_path: Path) -> str:
    return file_sha256(source_path)


def unique_in_order(values: Sequence[str]) -> list[str]:
    return list(dict.fromkeys(values))
