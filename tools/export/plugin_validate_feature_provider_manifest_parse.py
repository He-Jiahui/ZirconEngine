"""TOML parsing for generated feature-provider package manifests."""

from __future__ import annotations

from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python <3.11 fallback.
    import tomli as tomllib  # type: ignore[no-redef]


def plugin_validate_generated_package_manifest(
    package_manifest_text: str,
    package_id: str,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    try:
        manifest = tomllib.loads(package_manifest_text)
    except tomllib.TOMLDecodeError as error:
        diagnostics.append(
            f"plugin {package_id} generated package manifest is invalid TOML: {error}"
        )
        return None
    if not isinstance(manifest, dict):
        diagnostics.append(
            f"plugin {package_id} generated package manifest must be a table"
        )
        return None
    return manifest
