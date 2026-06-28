"""Dist crate Cargo manifest preflight checks for plugin validation."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build import read_toml
from .plugin_build import PLUGIN_BUILD_DIST_FEATURE


PLUGIN_VALIDATE_SDK_DEPENDENCY = "zircon_plugin_sdk"
PLUGIN_VALIDATE_SDK_ABI_FEATURES = ("native", "dist")
PLUGIN_VALIDATE_FORBIDDEN_DIST_FEATURE_DEPENDENCIES = ("zircon_runtime",)


def plugin_validate_dist_crate_manifest(
    crate_index: dict[str, dict[str, Any]],
    package_id: str,
    dist_crate: str | None,
    diagnostics: list[str],
) -> Path | None:
    if not dist_crate:
        return None
    crate = crate_index.get(dist_crate)
    if crate is None:
        diagnostics.append(
            f"plugin {package_id} distribution dist_crate {dist_crate} "
            "is not a cdylib workspace member"
        )
        return None
    manifest_path = crate.get("manifest_path")
    if isinstance(manifest_path, Path):
        return manifest_path
    return None


def validate_plugin_dist_crate_workspace_member(
    crate_index: dict[str, dict[str, Any]],
    package_id: str,
    dist_crate: str | None,
    diagnostics: list[str],
) -> Path | None:
    dist_crate_manifest = plugin_validate_dist_crate_manifest(
        crate_index,
        package_id,
        dist_crate,
        diagnostics,
    )
    if dist_crate_manifest is None or dist_crate is None:
        return dist_crate_manifest
    validate_plugin_dist_crate_feature(
        dist_crate_manifest,
        package_id,
        dist_crate,
        diagnostics,
    )
    return dist_crate_manifest


def validate_plugin_dist_crate_feature(
    dist_crate_manifest: Path,
    package_id: str,
    dist_crate: str,
    diagnostics: list[str],
) -> None:
    manifest = read_toml(dist_crate_manifest, diagnostics)
    if manifest is None:
        return
    features = manifest.get("features")
    if not isinstance(features, dict):
        diagnostics.append(
            f"plugin {package_id} dist crate {dist_crate} Cargo.toml features "
            "must be a table"
        )
        return
    feature = features.get(PLUGIN_BUILD_DIST_FEATURE)
    if feature is None:
        diagnostics.append(
            f"plugin {package_id} dist crate {dist_crate} "
            f"must declare Cargo feature {PLUGIN_BUILD_DIST_FEATURE}"
        )
        return
    if not isinstance(feature, list):
        diagnostics.append(
            f"plugin {package_id} dist crate {dist_crate} "
            f"Cargo feature {PLUGIN_BUILD_DIST_FEATURE} must be an array"
        )
        return
    dist_feature_entries: list[str] = []
    for index, item in enumerate(feature):
        if not isinstance(item, str) or not item.strip():
            diagnostics.append(
                f"plugin {package_id} dist crate {dist_crate} "
                f"Cargo feature {PLUGIN_BUILD_DIST_FEATURE}[{index}] "
                "must be a non-empty trimmed string"
            )
            continue
        if item.strip() != item:
            diagnostics.append(
                f"plugin {package_id} dist crate {dist_crate} "
                f"Cargo feature {PLUGIN_BUILD_DIST_FEATURE}[{index}] "
                "must be a non-empty trimmed string"
            )
            continue
        dist_feature_entries.append(item)
    validate_plugin_dist_crate_sdk_dependency(
        manifest,
        package_id,
        dist_crate,
        dist_feature_entries,
        diagnostics,
    )


def validate_plugin_dist_crate_sdk_dependency(
    manifest: dict[str, Any],
    package_id: str,
    dist_crate: str,
    dist_feature_entries: list[str],
    diagnostics: list[str],
) -> None:
    sdk_dependency = plugin_validate_find_dependency(
        manifest,
        PLUGIN_VALIDATE_SDK_DEPENDENCY,
    )
    if sdk_dependency is None:
        diagnostics.append(
            f"plugin {package_id} dist crate {dist_crate} must depend on "
            f"{PLUGIN_VALIDATE_SDK_DEPENDENCY}"
        )
        return
    if (
        not isinstance(sdk_dependency, dict)
        or sdk_dependency.get("default-features") is not False
    ):
        diagnostics.append(
            f"plugin {package_id} dist crate {dist_crate} dependency "
            f"{PLUGIN_VALIDATE_SDK_DEPENDENCY} must set default-features = false"
        )
    if not plugin_validate_sdk_dependency_enables_abi_helpers(
        sdk_dependency,
        dist_feature_entries,
    ):
        diagnostics.append(
            f"plugin {package_id} dist crate {dist_crate} dependency "
            f"{PLUGIN_VALIDATE_SDK_DEPENDENCY} must enable native/dist ABI helpers "
            f"directly or through Cargo feature {PLUGIN_BUILD_DIST_FEATURE}"
        )
    for dependency_name in PLUGIN_VALIDATE_FORBIDDEN_DIST_FEATURE_DEPENDENCIES:
        if plugin_validate_feature_enables_dependency(
            dist_feature_entries,
            dependency_name,
        ):
            diagnostics.append(
                f"plugin {package_id} dist crate {dist_crate} "
                f"Cargo feature {PLUGIN_BUILD_DIST_FEATURE} must not enable "
                f"forbidden dependency {dependency_name}"
            )


def plugin_validate_find_dependency(
    manifest: dict[str, Any],
    dependency_name: str,
) -> object | None:
    for dependencies in plugin_validate_dependency_tables(manifest):
        if dependency_name in dependencies:
            return dependencies[dependency_name]
    return None


def plugin_validate_dependency_tables(
    manifest: dict[str, Any],
) -> list[dict[str, Any]]:
    tables: list[dict[str, Any]] = []
    for section in ("dependencies", "build-dependencies"):
        dependencies = manifest.get(section, {})
        if isinstance(dependencies, dict):
            tables.append(dependencies)
    target = manifest.get("target", {})
    if isinstance(target, dict):
        for target_table in target.values():
            if not isinstance(target_table, dict):
                continue
            for section in ("dependencies", "build-dependencies"):
                dependencies = target_table.get(section, {})
                if isinstance(dependencies, dict):
                    tables.append(dependencies)
    return tables


def plugin_validate_sdk_dependency_enables_abi_helpers(
    dependency_spec: object,
    dist_feature_entries: list[str],
) -> bool:
    if isinstance(dependency_spec, dict):
        features = dependency_spec.get("features", [])
        if isinstance(features, list) and any(
            isinstance(feature, str)
            and feature in PLUGIN_VALIDATE_SDK_ABI_FEATURES
            for feature in features
        ):
            return True
    return any(
        feature
        in {
            f"{PLUGIN_VALIDATE_SDK_DEPENDENCY}/native",
            f"{PLUGIN_VALIDATE_SDK_DEPENDENCY}/dist",
        }
        for feature in dist_feature_entries
    )


def plugin_validate_feature_enables_dependency(
    feature_entries: list[str],
    dependency_name: str,
) -> bool:
    return any(
        feature_entry == dependency_name
        or feature_entry == f"dep:{dependency_name}"
        or feature_entry.startswith(f"{dependency_name}/")
        for feature_entry in feature_entries
    )
