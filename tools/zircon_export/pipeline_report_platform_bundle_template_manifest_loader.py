"""Input admission for PlatformBundle template manifests."""

from __future__ import annotations

from pathlib import Path
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python <3.11 fallback.
    import tomli as tomllib  # type: ignore[no-redef]

from .export_template import EXPORT_TEMPLATE_FORMAT_VERSION


def template_report_manifest_load(
    label: str,
    template: dict[str, Any],
) -> tuple[dict[str, Any] | None, Path | None, str | None]:
    template_dir = template.get("template_dir")
    manifest_path = template.get("manifest")
    if (
        not isinstance(template_dir, str)
        or not template_dir.strip()
        or not isinstance(manifest_path, str)
        or not manifest_path.strip()
    ):
        return None, None, None
    try:
        expected_manifest = (Path(template_dir).expanduser() / "template.toml").resolve()
        actual_manifest = Path(manifest_path).expanduser().resolve()
    except OSError as error:
        return None, None, f"{label}.manifest could not be resolved: {error}"
    if actual_manifest != expected_manifest:
        return None, None, f"{label}.manifest does not match template_dir/template.toml"
    if not actual_manifest.exists():
        return None, None, f"{label}.manifest {actual_manifest} does not exist"
    if not actual_manifest.is_file():
        return None, None, f"{label}.manifest {actual_manifest} is not a file"
    try:
        with actual_manifest.open("rb") as manifest_file:
            manifest = tomllib.load(manifest_file)
    except OSError as error:
        return None, None, f"{label}.manifest {actual_manifest} could not be read: {error}"
    except tomllib.TOMLDecodeError as error:
        return None, None, f"{label}.manifest {actual_manifest} is not valid TOML: {error}"
    if not isinstance(manifest, dict):
        return None, None, f"{label}.manifest {actual_manifest} must be a TOML table"
    format_version = manifest.get("format_version")
    if type(format_version) is not int:
        return None, None, f"{label}.manifest format_version must be an integer"
    if format_version != EXPORT_TEMPLATE_FORMAT_VERSION:
        return (
            None,
            None,
            f"{label}.manifest format_version {format_version} is not supported; "
            f"expected {EXPORT_TEMPLATE_FORMAT_VERSION}",
        )
    return manifest, expected_manifest.parent, None
