"""NativeDynamic build-plan schema diagnostics."""

from __future__ import annotations

from typing import Any, Callable

from .pipeline_report_native_dynamic_build_audit_common import (
    native_dynamic_build_audit_package_count_diagnostics,
    native_dynamic_build_audit_package_id_uniqueness_diagnostics,
    native_dynamic_fatal_report_diagnostics,
    native_dynamic_non_fatal_report_diagnostics,
    string_array_unique_entries_schema_diagnostics,
    table_non_negative_integer_schema_diagnostics,
)
from .pipeline_report_native_dynamic_build_plan_package_details import (
    NATIVE_DYNAMIC_BUILD_PLAN_CARGO_PROFILES,
    native_dynamic_build_plan_package_expected_artifact_diagnostics,
    native_dynamic_build_plan_package_header_diagnostics,
)
from .pipeline_report_native_dynamic_build_plan_schema_helpers import (
    native_dynamic_build_plan_diagnostics_array_schema_diagnostics,
    native_dynamic_build_plan_feature_array_schema_diagnostics,
    native_dynamic_build_plan_packages_schema_diagnostics,
    native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean,
    native_dynamic_build_plan_trimmed_non_empty_string_schema_diagnostics,
)
from .pipeline_report_schema_string_array import (
    string_array_no_blank_entries_schema_diagnostics,
    string_array_trimmed_non_empty_entries_schema_diagnostics,
)

NATIVE_DYNAMIC_BUILD_PLAN_FIELDS = (
    "build_features",
    "cargo_profile",
    "diagnostics",
    "fatal",
    "package_count",
    "packages",
    "release",
    "target_dir",
    "workspace_manifest",
)
NATIVE_DYNAMIC_BUILD_PLAN_STRING_FIELDS = (
    "cargo_profile",
    "target_dir",
    "workspace_manifest",
)
NATIVE_DYNAMIC_BUILD_PLAN_BOOL_FIELDS = ("fatal", "release")
NATIVE_DYNAMIC_BUILD_PLAN_INTEGER_FIELDS = ("package_count",)
NATIVE_DYNAMIC_BUILD_PLAN_NON_NEGATIVE_INTEGER_FIELDS = ("package_count",)
NATIVE_DYNAMIC_BUILD_PLAN_STRING_ARRAY_FIELDS = (
    "build_features",
    "diagnostics",
)
NATIVE_DYNAMIC_BUILD_PLAN_OBJECT_ARRAY_FIELDS = ("packages",)
NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_STRING_FIELDS = (
    "cargo_profile",
    "target_dir",
    "workspace_manifest",
)
NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_BOOL_FIELDS = (
    "fatal",
    "release",
)
NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_INTEGER_FIELDS = (
    "package_count",
)
NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS = (
    "build_features",
    "diagnostics",
)
NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS = (
    "packages",
)

SchemaDiagnostic = Callable[[str, Any], list[str]]

def native_dynamic_build_plan_execution_state_diagnostics(
    build_plan: Any,
    build_execution: Any,
    report_fatal: Any,
) -> list[str]:
    if not (
        report_fatal is False
        and isinstance(build_plan, dict)
        and isinstance(build_execution, dict)
        and build_execution.get("enabled") is True
        and build_plan.get("fatal") is True
    ):
        return []
    return [
        "native_dynamic report native_build_plan.fatal must be False "
        "when native_build_execution.enabled is True and "
        "NativeDynamic report fatal is False"
    ]


def native_dynamic_build_plan_schema_diagnostics(
    label: str,
    build_plan: dict[str, Any],
    *,
    validate_bool_schema_diagnostics: SchemaDiagnostic,
    validate_integer_schema_diagnostics: SchemaDiagnostic,
    validate_string_schema_diagnostics: SchemaDiagnostic,
    validate_string_array_schema_diagnostics: SchemaDiagnostic,
    validate_object_array_schema_diagnostics: SchemaDiagnostic,
    require_release_evidence: bool = False,
    target_platform: object = None,
) -> list[str]:
    diagnostics: list[str] = []
    diagnostics.extend(
        f"{label} unknown field {field}"
        for field in sorted(build_plan)
        if field not in NATIVE_DYNAMIC_BUILD_PLAN_FIELDS
    )
    if require_release_evidence:
        for field in NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_STRING_FIELDS:
            if field not in build_plan or build_plan.get(field) is None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"{label}.{field}",
                        build_plan.get(field),
                    )
                )
            elif (
                isinstance(build_plan.get(field), str)
                and not build_plan.get(field).strip()
            ):
                diagnostics.append(f"{label}.{field} must be a non-empty string")
        for field in NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_BOOL_FIELDS:
            if field not in build_plan or build_plan.get(field) is None:
                diagnostics.extend(
                    validate_bool_schema_diagnostics(
                        f"{label}.{field}",
                        build_plan.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_INTEGER_FIELDS:
            if field not in build_plan or build_plan.get(field) is None:
                diagnostics.extend(
                    validate_integer_schema_diagnostics(
                        f"{label}.{field}",
                        build_plan.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS:
            if field not in build_plan or build_plan.get(field) is None:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"{label}.{field}",
                        build_plan.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS:
            if field not in build_plan or build_plan.get(field) is None:
                diagnostics.extend(
                    validate_object_array_schema_diagnostics(
                        f"{label}.{field}",
                        build_plan.get(field),
                    )
                )
    for field in NATIVE_DYNAMIC_BUILD_PLAN_STRING_FIELDS:
        if field in build_plan and build_plan.get(field) is not None:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"{label}.{field}",
                    build_plan.get(field),
                )
            )
            diagnostics.extend(
                native_dynamic_build_plan_trimmed_non_empty_string_schema_diagnostics(
                    f"{label}.{field}",
                    build_plan.get(field),
                )
            )
    for field in NATIVE_DYNAMIC_BUILD_PLAN_BOOL_FIELDS:
        if field in build_plan and build_plan.get(field) is not None:
            diagnostics.extend(
                validate_bool_schema_diagnostics(
                    f"{label}.{field}",
                    build_plan.get(field),
                )
            )
    for field in NATIVE_DYNAMIC_BUILD_PLAN_INTEGER_FIELDS:
        if field in build_plan and build_plan.get(field) is not None:
            diagnostics.extend(
                validate_integer_schema_diagnostics(
                    f"{label}.{field}",
                    build_plan.get(field),
                )
            )
    diagnostics.extend(
        table_non_negative_integer_schema_diagnostics(
            label,
            build_plan,
            NATIVE_DYNAMIC_BUILD_PLAN_NON_NEGATIVE_INTEGER_FIELDS,
        )
    )
    for field in NATIVE_DYNAMIC_BUILD_PLAN_STRING_ARRAY_FIELDS:
        if field in build_plan and build_plan.get(field) is not None:
            field_label = f"{label}.{field}"
            if field == "build_features":
                diagnostics.extend(
                    native_dynamic_build_plan_feature_array_schema_diagnostics(
                        field_label,
                        build_plan.get(field),
                    )
                )
            elif field == "diagnostics":
                diagnostics.extend(
                    native_dynamic_build_plan_diagnostics_array_schema_diagnostics(
                        field_label,
                        build_plan.get(field),
                    )
                )
            else:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        field_label,
                        build_plan.get(field),
                    )
                )
            if field == "build_features":
                diagnostics.extend(
                    string_array_no_blank_entries_schema_diagnostics(
                        field_label,
                        build_plan.get(field),
                    )
                )
                diagnostics.extend(
                    string_array_trimmed_non_empty_entries_schema_diagnostics(
                        field_label,
                        build_plan.get(field),
                    )
                )
                diagnostics.extend(
                    string_array_unique_entries_schema_diagnostics(
                        field_label,
                        build_plan.get(field),
                    )
                )
            if field == "diagnostics":
                diagnostics.extend(
                    string_array_no_blank_entries_schema_diagnostics(
                        field_label,
                        build_plan.get(field),
                    )
                )
                diagnostics.extend(
                    string_array_trimmed_non_empty_entries_schema_diagnostics(
                        field_label,
                        build_plan.get(field),
                    )
                )
    diagnostics.extend(
        native_dynamic_build_plan_profile_release_diagnostics(
            label,
            build_plan,
        )
    )
    diagnostics.extend(native_dynamic_non_fatal_report_diagnostics(label, build_plan))
    diagnostics.extend(native_dynamic_fatal_report_diagnostics(label, build_plan))
    packages = build_plan.get("packages")
    if "packages" in build_plan and packages is not None:
        diagnostics.extend(
            validate_object_array_schema_diagnostics(
                f"{label}.packages",
                packages,
            )
        )
    diagnostics.extend(
        native_dynamic_build_audit_package_count_diagnostics(label, build_plan)
    )
    if isinstance(packages, list):
        diagnostics.extend(
            native_dynamic_build_audit_package_id_uniqueness_diagnostics(
                f"{label}.packages",
                packages,
            )
        )
        diagnostics.extend(
            native_dynamic_build_plan_packages_schema_diagnostics(
                f"{label}.packages",
                packages,
                validate_bool_schema_diagnostics=validate_bool_schema_diagnostics,
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
                validate_string_array_schema_diagnostics=(
                    validate_string_array_schema_diagnostics
                ),
            )
        )
        diagnostics.extend(
            native_dynamic_build_plan_package_header_diagnostics(
                label,
                build_plan,
                packages,
            )
        )
        diagnostics.extend(
            native_dynamic_build_plan_package_expected_artifact_diagnostics(
                f"{label}.packages",
                packages,
                target_platform,
            )
        )
    return diagnostics


def native_dynamic_build_plan_profile_release_diagnostics(
    label: str,
    build_plan: dict[str, Any],
) -> list[str]:
    cargo_profile = build_plan.get("cargo_profile")
    release = build_plan.get("release")
    if not native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
        cargo_profile
    ):
        return []
    if cargo_profile not in NATIVE_DYNAMIC_BUILD_PLAN_CARGO_PROFILES:
        return [f"{label}.cargo_profile must be debug or release"]
    if type(release) is bool and release != (cargo_profile == "release"):
        return [f"{label}.release must match cargo_profile"]
    return []

