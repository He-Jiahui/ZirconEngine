"""Validate report profile_summary schema diagnostics."""

from __future__ import annotations

from typing import Any

from .export_strategy_contract import normalize_export_strategy
from .pipeline_report_schema_primitives import validate_string_schema_diagnostics
from .pipeline_report_validate_identifier_schema import (
    validate_non_empty_trimmed_string_schema_diagnostics,
    validate_project_plugin_feature_id_schema_diagnostics,
    validate_project_plugin_package_id_array_schema_diagnostics,
    validate_project_plugin_package_id_schema_diagnostics,
    validate_unique_project_plugin_package_id_array_schema_diagnostics,
)
from .pipeline_report_validate_string_array_schema import (
    validate_string_array_schema_diagnostics,
)

VALIDATE_PROFILE_SUMMARY_FIELDS = (
    "asset_filter",
    "build_mode",
    "features",
    "name",
    "selected_plugins",
    "strategies",
    "target_mode",
    "target_platform",
)
VALIDATE_PROFILE_SUMMARY_STRING_FIELDS = (
    "asset_filter",
    "build_mode",
    "name",
    "target_mode",
    "target_platform",
)
VALIDATE_PROFILE_SUMMARY_BUILD_MODES = (
    "debug",
    "release",
)
VALIDATE_PROFILE_SUMMARY_TARGET_MODES = (
    "client_runtime",
    "server_runtime",
    "editor_host",
)
VALIDATE_PROFILE_SUMMARY_TARGET_PLATFORMS = (
    "android",
    "headless",
    "ios",
    "linux",
    "linux-x86_64",
    "macos",
    "macos-aarch64",
    "wasm",
    "web_gpu",
    "windows",
    "windows-x86_64",
)
VALIDATE_PROFILE_SUMMARY_STRING_ARRAY_FIELDS = ("strategies",)
VALIDATE_PROFILE_SUMMARY_PROJECT_PLUGIN_ID_ARRAY_FIELDS = ("selected_plugins",)
VALIDATE_PROFILE_SUMMARY_REQUIRED_STRING_FIELDS = (
    "build_mode",
    "name",
    "target_mode",
    "target_platform",
)
VALIDATE_PROFILE_SUMMARY_REQUIRED_PROJECT_PLUGIN_ID_ARRAY_FIELDS = (
    "selected_plugins",
)
VALIDATE_PROFILE_SUMMARY_REQUIRED_OBJECT_FIELDS = ("features",)


def validate_profile_summary_schema_diagnostics(
    profile_summary: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    known_profile_fields = set(VALIDATE_PROFILE_SUMMARY_FIELDS)
    diagnostics.extend(
        f"validate report profile_summary unknown field {field}"
        for field in sorted(profile_summary)
        if field not in known_profile_fields
    )
    for field in VALIDATE_PROFILE_SUMMARY_REQUIRED_STRING_FIELDS:
        if field not in profile_summary:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"validate report profile_summary.{field}",
                    profile_summary.get(field),
                )
            )
    for field in VALIDATE_PROFILE_SUMMARY_REQUIRED_PROJECT_PLUGIN_ID_ARRAY_FIELDS:
        if field not in profile_summary:
            diagnostics.extend(
                validate_project_plugin_package_id_array_schema_diagnostics(
                    f"validate report profile_summary.{field}",
                    profile_summary.get(field),
                )
            )
    for field in VALIDATE_PROFILE_SUMMARY_REQUIRED_OBJECT_FIELDS:
        if field not in profile_summary:
            diagnostics.extend(
                validate_profile_features_schema_diagnostics(
                    profile_summary.get(field)
                )
            )
    for field in VALIDATE_PROFILE_SUMMARY_STRING_FIELDS:
        if field in profile_summary:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"validate report profile_summary.{field}",
                    profile_summary.get(field),
                )
            )
    if "build_mode" in profile_summary:
        diagnostics.extend(
            validate_known_trimmed_string_schema_diagnostics(
                "validate report profile_summary.build_mode",
                profile_summary.get("build_mode"),
                VALIDATE_PROFILE_SUMMARY_BUILD_MODES,
                "export build mode",
            )
        )
    if "target_mode" in profile_summary:
        diagnostics.extend(
            validate_known_trimmed_string_schema_diagnostics(
                "validate report profile_summary.target_mode",
                profile_summary.get("target_mode"),
                VALIDATE_PROFILE_SUMMARY_TARGET_MODES,
                "runtime target mode",
            )
        )
    if "target_platform" in profile_summary:
        diagnostics.extend(
            validate_known_trimmed_string_schema_diagnostics(
                "validate report profile_summary.target_platform",
                profile_summary.get("target_platform"),
                VALIDATE_PROFILE_SUMMARY_TARGET_PLATFORMS,
                "export target platform",
            )
        )
    if "asset_filter" in profile_summary:
        diagnostics.extend(
            validate_known_trimmed_string_schema_diagnostics(
                "validate report profile_summary.asset_filter",
                profile_summary.get("asset_filter"),
                None,
                "asset filter",
            )
        )
    if "name" in profile_summary:
        diagnostics.extend(
            validate_known_trimmed_string_schema_diagnostics(
                "validate report profile_summary.name",
                profile_summary.get("name"),
                None,
                "profile name",
            )
        )
    for field in VALIDATE_PROFILE_SUMMARY_STRING_ARRAY_FIELDS:
        if field in profile_summary:
            diagnostics.extend(
                validate_string_array_schema_diagnostics(
                    f"validate report profile_summary.{field}",
                    profile_summary.get(field),
                )
            )
    strategies = profile_summary.get("strategies")
    if (
        "strategies" in profile_summary
        and isinstance(strategies, list)
        and not any(not isinstance(strategy, str) for strategy in strategies)
        and not strategies
    ):
        diagnostics.append(
            "validate report profile_summary.strategies must include "
            "at least one supported export strategy"
        )
    if (
        "strategies" in profile_summary
        and isinstance(strategies, list)
        and not any(not isinstance(strategy, str) for strategy in strategies)
    ):
        diagnostics.extend(validate_export_strategy_schema_diagnostics(strategies))
        diagnostics.extend(
            validate_unique_export_strategy_schema_diagnostics(strategies)
        )
    for field in VALIDATE_PROFILE_SUMMARY_PROJECT_PLUGIN_ID_ARRAY_FIELDS:
        if field in profile_summary:
            label = f"validate report profile_summary.{field}"
            value = profile_summary.get(field)
            diagnostics.extend(
                validate_project_plugin_package_id_array_schema_diagnostics(
                    label,
                    value,
                )
            )
            diagnostics.extend(
                validate_unique_project_plugin_package_id_array_schema_diagnostics(
                    label,
                    value,
                )
            )
    if "features" in profile_summary:
        diagnostics.extend(
            validate_profile_features_schema_diagnostics(
                profile_summary.get("features")
            )
        )
    return diagnostics


def validate_known_trimmed_string_schema_diagnostics(
    label: str,
    value: Any,
    allowed_values: tuple[str, ...] | None,
    contract: str,
) -> list[str]:
    if not isinstance(value, str):
        return []
    if not value or value.strip() != value:
        return [f"{label} must be a non-empty trimmed {contract}"]
    if allowed_values is None:
        return []
    if value not in allowed_values:
        return [f"{label} must be a known {contract}"]
    return []


def validate_export_strategy_schema_diagnostics(strategies: list[str]) -> list[str]:
    diagnostics: list[str] = []
    for index, strategy in enumerate(strategies):
        if not strategy or strategy.strip() != strategy:
            diagnostics.append(
                "validate report profile_summary.strategies"
                f"[{index}] must be a non-empty trimmed export strategy"
            )
            continue
        if normalize_export_strategy(strategy) is None:
                diagnostics.append(f"unsupported export strategy {strategy}")
    return diagnostics


def validate_unique_export_strategy_schema_diagnostics(
    strategies: list[str],
) -> list[str]:
    diagnostics: list[str] = []
    seen: dict[str, int] = {}
    for index, strategy in enumerate(strategies):
        if not strategy or strategy.strip() != strategy:
            continue
        normalized = normalize_export_strategy(strategy)
        if normalized is None:
            continue
        previous_index = seen.get(normalized)
        if previous_index is None:
            seen[normalized] = index
            continue
        diagnostics.append(
            "validate report profile_summary.strategies"
            f"[{index}] duplicates entry {previous_index}"
        )
    return diagnostics


def validate_profile_features_schema_diagnostics(features: Any) -> list[str]:
    if not isinstance(features, dict):
        return ["validate report profile_summary.features must be an object"]
    diagnostics: list[str] = []
    for plugin_id, feature_list in sorted(features.items()):
        plugin_id_diagnostics = (
            validate_non_empty_trimmed_string_schema_diagnostics(
                "validate report profile_summary.features plugin id",
                plugin_id,
            )
        )
        diagnostics.extend(plugin_id_diagnostics)
        plugin_id_schema_diagnostics: list[str] = []
        if not plugin_id_diagnostics:
            plugin_id_schema_diagnostics = (
                validate_project_plugin_package_id_schema_diagnostics(
                    "validate report profile_summary.features plugin id",
                    plugin_id,
                )
            )
            diagnostics.extend(plugin_id_schema_diagnostics)
        feature_list_diagnostics = validate_string_array_schema_diagnostics(
            "validate report profile_summary.features." f"{plugin_id}",
            feature_list,
        )
        if feature_list_diagnostics:
            diagnostics.extend(feature_list_diagnostics)
            continue
        seen_feature_ids: dict[str, int] = {}
        for index, feature_id in enumerate(feature_list):
            feature_id_diagnostics = (
                validate_non_empty_trimmed_string_schema_diagnostics(
                    "validate report profile_summary.features."
                    f"{plugin_id} feature id",
                    feature_id,
                )
            )
            diagnostics.extend(feature_id_diagnostics)
            if (
                not plugin_id_diagnostics
                and not plugin_id_schema_diagnostics
                and not feature_id_diagnostics
            ):
                feature_schema_diagnostics = (
                    validate_project_plugin_feature_id_schema_diagnostics(
                        "validate report profile_summary.features."
                        f"{plugin_id} feature id",
                        plugin_id,
                        feature_id,
                    )
                )
                diagnostics.extend(feature_schema_diagnostics)
                if feature_schema_diagnostics:
                    continue
                previous_index = seen_feature_ids.get(feature_id)
                if previous_index is None:
                    seen_feature_ids[feature_id] = index
                    continue
                diagnostics.append(
                    "validate report profile_summary.features."
                    f"{plugin_id}[{index}] duplicates entry {previous_index}"
                )
    return diagnostics
