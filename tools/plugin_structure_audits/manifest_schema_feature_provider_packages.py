from __future__ import annotations

from pathlib import Path, PurePosixPath
from typing import Any

try:
    from tools.zircon_export.plugin_package_identity import (
        feature_provider_package_id as resolve_feature_provider_package_id,
    )
    from tools.zircon_export.plugin_package_template import (
        feature_provider_package_manifest_template,
    )
    from tools.zircon_export.plugin_validate_feature_provider import (
        validate_plugin_feature_provider_package_projection,
    )
except ModuleNotFoundError:  # pragma: no cover - direct script execution fallback.
    from zircon_export.plugin_package_identity import (
        feature_provider_package_id as resolve_feature_provider_package_id,
    )
    from zircon_export.plugin_package_template import (
        feature_provider_package_manifest_template,
    )
    from zircon_export.plugin_validate_feature_provider import (
        validate_plugin_feature_provider_package_projection,
    )


LoadedManifest = tuple[str, dict[str, Any]]


def collect_feature_provider_package_projection_violations(
    repo_root: Path,
    loaded_manifests: list[LoadedManifest],
    violations: list[str],
) -> int:
    projection_count = 0
    for display_path, manifest in loaded_manifests:
        optional_features = manifest.get("optional_features")
        if not isinstance(optional_features, list):
            continue
        for feature in optional_features:
            if not isinstance(feature, dict):
                continue
            distribution = feature.get("distribution")
            if not isinstance(distribution, dict):
                continue
            provider_package_id = feature_provider_package_identity(feature)
            if provider_package_id is None:
                continue
            projection_count += 1
            collect_single_feature_provider_package_projection_violations(
                repo_root,
                display_path,
                manifest,
                feature,
                distribution,
                provider_package_id,
                violations,
            )
    return projection_count


def collect_single_feature_provider_package_projection_violations(
    repo_root: Path,
    display_path: str,
    manifest: dict[str, Any],
    feature: dict[str, Any],
    distribution: dict[str, Any],
    provider_package_id: str,
    violations: list[str],
) -> None:
    package_manifest_text = feature_provider_package_manifest_template(
        owner_manifest=manifest,
        feature=feature,
        provider_package_id=provider_package_id,
        distribution=distribution,
    )
    diagnostics: list[str] = []
    validate_plugin_feature_provider_package_projection(
        plugin_manifest_path=repo_root / path_from_display_path(display_path),
        package_manifest_text=package_manifest_text,
        requested_plugin_id=provider_package_id,
        package_id=provider_package_id,
        diagnostics=diagnostics,
    )
    for diagnostic in diagnostics:
        violations.append(f"{display_path}: {diagnostic}")


def feature_provider_package_identity(feature: dict[str, Any]) -> str | None:
    feature_id = feature.get("id")
    if not is_non_empty_trimmed_string(feature_id):
        return None
    provider_package_id = resolve_feature_provider_package_id(feature, feature_id)
    if not is_non_empty_trimmed_string(provider_package_id):
        return None
    return provider_package_id


def path_from_display_path(display_path: str) -> Path:
    return Path(*PurePosixPath(display_path).parts)


def is_non_empty_trimmed_string(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value
