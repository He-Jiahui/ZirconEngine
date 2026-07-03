from __future__ import annotations

from typing import Any, Iterable

try:
    from tools.zircon_export.plugin_package_identity import feature_provider_package_id
except ModuleNotFoundError:  # pragma: no cover - script entrypoint import mode.
    from zircon_export.plugin_package_identity import feature_provider_package_id


ManifestEntry = tuple[str, dict[str, Any]]


def collect_feature_provider_target_identity_violations(
    manifests: Iterable[ManifestEntry],
    violations: list[str],
) -> None:
    seen_targets: dict[str, str] = {}
    for display_path, manifest in manifests:
        collect_root_distribution_target_identity(
            display_path,
            manifest,
            seen_targets,
            violations,
        )
        collect_optional_feature_provider_target_identities(
            display_path,
            manifest,
            seen_targets,
            violations,
        )


def collect_root_distribution_target_identity(
    display_path: str,
    manifest: dict[str, Any],
    seen_targets: dict[str, str],
    violations: list[str],
) -> None:
    package_id = manifest.get("id")
    distribution = manifest.get("distribution")
    if not isinstance(distribution, dict):
        return
    if not is_non_empty_trimmed_string(package_id):
        return
    append_target_identity(
        package_id,
        display_path,
        seen_targets,
        violations,
    )


def collect_optional_feature_provider_target_identities(
    display_path: str,
    manifest: dict[str, Any],
    seen_targets: dict[str, str],
    violations: list[str],
) -> None:
    optional_features = manifest.get("optional_features")
    if not isinstance(optional_features, list):
        return
    for feature_index, feature in enumerate(optional_features):
        if not isinstance(feature, dict):
            continue
        distribution = feature.get("distribution")
        if not isinstance(distribution, dict):
            continue
        feature_id = feature.get("id")
        if not is_non_empty_trimmed_string(feature_id):
            continue
        provider_id = feature_provider_package_id(feature, feature_id)
        if provider_id is None:
            continue
        append_target_identity(
            provider_id,
            f"{display_path} optional_features[{feature_index}]",
            seen_targets,
            violations,
        )


def append_target_identity(
    target_id: str,
    source: str,
    seen_targets: dict[str, str],
    violations: list[str],
) -> None:
    previous = seen_targets.get(target_id)
    if previous is not None:
        violations.append(
            f"plugin validate target {target_id} is duplicated by "
            f"{previous} and {source}"
        )
        return
    seen_targets[target_id] = source


def is_non_empty_trimmed_string(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value
