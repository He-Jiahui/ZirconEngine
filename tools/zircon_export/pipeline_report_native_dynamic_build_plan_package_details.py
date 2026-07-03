"""NativeDynamic build-plan package detail diagnostics."""

from __future__ import annotations

from typing import Any

from .native_build_command import platform_dynamic_library_name
from .pipeline_report_native_dynamic_build_plan_schema_helpers import (
    native_dynamic_build_plan_string_array_is_schema_clean,
    native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean,
)

NATIVE_DYNAMIC_BUILD_PLAN_CARGO_PROFILES = ("debug", "release")


def native_dynamic_build_plan_package_header_diagnostics(
    label: str,
    build_plan: dict[str, Any],
    packages: list[Any],
) -> list[str]:
    diagnostics: list[str] = []
    field_pairs = (
        ("workspace_manifest", "workspace_manifest"),
        ("target_dir", "target_dir"),
        ("cargo_profile", "cargo_profile"),
        ("release", "release"),
        ("build_features", "features"),
    )
    for index, package in enumerate(packages):
        if not isinstance(package, dict):
            continue
        package_label = f"{label}.packages[{index}]"
        for plan_field, package_field in field_pairs:
            diagnostics.extend(
                native_dynamic_build_plan_package_header_field_diagnostics(
                    label,
                    package_label,
                    plan_field,
                    package_field,
                    build_plan,
                    package,
                )
            )
    return diagnostics


def native_dynamic_build_plan_package_expected_artifact_diagnostics(
    label: str,
    packages: list[Any],
    target_platform: object,
) -> list[str]:
    if not isinstance(target_platform, str) or not target_platform.strip():
        return []
    diagnostics: list[str] = []
    for index, package in enumerate(packages):
        if not isinstance(package, dict):
            continue
        package_label = f"{label}[{index}]"
        target_dir = package.get("target_dir")
        cargo_profile = package.get("cargo_profile")
        crate_name = package.get("crate_name")
        expected_loadable_artifact = package.get("expected_loadable_artifact")
        if not (
            native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
                target_dir
            )
            and isinstance(cargo_profile, str)
            and cargo_profile in NATIVE_DYNAMIC_BUILD_PLAN_CARGO_PROFILES
            and native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
                crate_name
            )
            and native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
                expected_loadable_artifact
            )
        ):
            continue
        target_dir_normalized = native_dynamic_normalized_path(target_dir).rstrip("/")
        derived_artifact = (
            f"{target_dir_normalized}/{cargo_profile}/"
            f"{platform_dynamic_library_name(crate_name, target_platform)}"
        )
        actual_artifact = native_dynamic_normalized_path(
            expected_loadable_artifact
        )
        if actual_artifact != derived_artifact:
            diagnostics.append(
                f"{package_label}.expected_loadable_artifact "
                f"{actual_artifact} does not match derived artifact "
                f"{derived_artifact}"
            )
    return diagnostics


def native_dynamic_normalized_path(value: str) -> str:
    return value.replace("\\", "/")


def native_dynamic_build_plan_package_header_field_diagnostics(
    plan_label: str,
    package_label: str,
    plan_field: str,
    package_field: str,
    build_plan: dict[str, Any],
    package: dict[str, Any],
) -> list[str]:
    plan_value = build_plan.get(plan_field)
    package_value = package.get(package_field)
    if not (
        native_dynamic_build_plan_header_value_is_comparable(plan_value)
        and native_dynamic_build_plan_header_value_is_comparable(package_value)
    ):
        return []
    if package_value == plan_value:
        return []
    return [
        f"{package_label}.{package_field} {package_value} does not match "
        f"{plan_label}.{plan_field} {plan_value}"
    ]


def native_dynamic_build_plan_header_value_is_comparable(value: Any) -> bool:
    if isinstance(value, str):
        return native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
            value
        )
    if type(value) is bool:
        return True
    if isinstance(value, list):
        return native_dynamic_build_plan_string_array_is_schema_clean(
            value,
            allow_empty=True,
            require_unique=True,
        )
    return False
