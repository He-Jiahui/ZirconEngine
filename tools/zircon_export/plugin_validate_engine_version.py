"""Engine version discovery for plugin validation."""

from __future__ import annotations

from pathlib import Path

from .native_build import read_toml
from .plugin_validate_distribution_contract import plugin_validate_parse_engine_version


PLUGIN_VALIDATE_ENGINE_VERSION_FIELD = "workspace.package.version"


def plugin_validate_engine_version(
    repo_root: Path | None,
    diagnostics: list[str],
) -> str | None:
    if repo_root is None:
        return None
    manifest_path = repo_root / "Cargo.toml"
    manifest = read_toml(manifest_path, diagnostics)
    if manifest is None:
        return None
    workspace = manifest.get("workspace")
    if not isinstance(workspace, dict):
        diagnostics.append(f"{manifest_path} workspace must be a table")
        return None
    package = workspace.get("package")
    if not isinstance(package, dict):
        diagnostics.append(f"{manifest_path} workspace.package must be a table")
        return None
    label = f"{manifest_path} {PLUGIN_VALIDATE_ENGINE_VERSION_FIELD}"
    version = package.get("version")
    if not isinstance(version, str) or not version.strip() or version.strip() != version:
        diagnostics.append(f"{label} must be a non-empty trimmed string")
        return None
    try:
        plugin_validate_parse_engine_version(version)
    except ValueError as error:
        diagnostics.append(f"{label} is invalid: {error}")
        return None
    return version
