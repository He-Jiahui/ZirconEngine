"""NativeDynamic operation-audit stage package/artifact schema diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_native_dynamic_operation_audit_schema_helpers import (
    artifact_exit_code_success_schema_diagnostics,
    artifact_safe_relative_path_schema_diagnostics,
    object_array_non_negative_integer_schema_diagnostics,
    object_array_required_non_empty_string_schema_diagnostics,
    object_array_required_trimmed_non_empty_string_schema_diagnostics,
    object_array_sha256_hex_string_schema_diagnostics,
    object_array_unique_string_field_schema_diagnostics,
    operation_audit_artifact_command_schema_diagnostics,
)
from .pipeline_report_schema_table import object_array_schema_diagnostics
from .pipeline_report_schema_string_array import non_empty_string_array_schema_diagnostics

NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_FIELDS = (
    "artifact_count",
    "artifacts",
    "package_id",
)

NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_STRING_FIELDS = ("package_id",)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_STRING_FIELDS = ("package_id",)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_NON_EMPTY_STRING_FIELDS = (
    "package_id",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_INTEGER_FIELDS = ("artifact_count",)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_INTEGER_FIELDS = (
    "artifact_count",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_OBJECT_ARRAY_FIELDS = (
    "artifacts",
)

NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_FIELDS = (
    "after_sha256",
    "artifact",
    "before_sha256",
    "command",
    "exit_code",
    "package_relative_artifact",
    "stderr",
    "stdout",
)

NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_STRING_FIELDS = (
    "after_sha256",
    "artifact",
    "before_sha256",
    "package_relative_artifact",
    "stderr",
    "stdout",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_STRING_FIELDS = (
    "after_sha256",
    "artifact",
    "before_sha256",
    "package_relative_artifact",
    "stderr",
    "stdout",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_NON_EMPTY_STRING_FIELDS = (
    "after_sha256",
    "artifact",
    "before_sha256",
    "package_relative_artifact",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_SHA256_FIELDS = (
    "after_sha256",
    "before_sha256",
)

NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_INTEGER_FIELDS = ("exit_code",)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_INTEGER_FIELDS = (
    "exit_code",
)


def native_dynamic_operation_audit_stage_packages_schema_diagnostics(
    label: str,
    audit: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    diagnostics.extend(
        object_array_schema_diagnostics(
            label,
            audit,
            "packages",
            NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_FIELDS,
            string_fields=NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_STRING_FIELDS,
            integer_fields=NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_INTEGER_FIELDS,
            required_string_fields=(
                NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_STRING_FIELDS
            ),
            required_integer_fields=(
                NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_INTEGER_FIELDS
            ),
            required_object_array_fields=(
                NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_OBJECT_ARRAY_FIELDS
            ),
            require_present=True,
        )
    )
    diagnostics.extend(
        object_array_non_negative_integer_schema_diagnostics(
            label,
            audit,
            "packages",
            NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_INTEGER_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_required_non_empty_string_schema_diagnostics(
            label,
            audit,
            "packages",
            NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_NON_EMPTY_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_required_trimmed_non_empty_string_schema_diagnostics(
            label,
            audit,
            "packages",
            NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_NON_EMPTY_STRING_FIELDS,
        )
    )
    packages = audit.get("packages")
    if isinstance(packages, list):
        for index, package in enumerate(packages):
            if not isinstance(package, dict):
                continue
            diagnostics.extend(
                object_array_schema_diagnostics(
                    f"{label} packages[{index}]",
                    package,
                    "artifacts",
                    NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_FIELDS,
                    string_fields=(
                        NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_STRING_FIELDS
                    ),
                    integer_fields=(
                        NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_INTEGER_FIELDS
                    ),
                    required_string_fields=(
                        NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_STRING_FIELDS
                    ),
                    required_integer_fields=(
                        NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_INTEGER_FIELDS
                    ),
                )
            )
            diagnostics.extend(
                object_array_required_non_empty_string_schema_diagnostics(
                    f"{label} packages[{index}]",
                    package,
                    "artifacts",
                    NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_NON_EMPTY_STRING_FIELDS,
                )
            )
            diagnostics.extend(
                object_array_required_trimmed_non_empty_string_schema_diagnostics(
                    f"{label} packages[{index}]",
                    package,
                    "artifacts",
                    NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_NON_EMPTY_STRING_FIELDS,
                )
            )
            diagnostics.extend(
                object_array_sha256_hex_string_schema_diagnostics(
                    f"{label} packages[{index}]",
                    package,
                    "artifacts",
                    NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_SHA256_FIELDS,
                )
            )
            diagnostics.extend(
                object_array_unique_string_field_schema_diagnostics(
                    f"{label} packages[{index}]",
                    package,
                    "artifacts",
                    "package_relative_artifact",
                )
            )
            artifacts = package.get("artifacts")
            if not isinstance(artifacts, list):
                continue
            for artifact_index, artifact in enumerate(artifacts):
                if not isinstance(artifact, dict):
                    continue
                artifact_label = (
                    f"{label} packages[{index}] artifacts[{artifact_index}]"
                )
                diagnostics.extend(
                    operation_audit_artifact_command_schema_diagnostics(
                        artifact_label,
                        artifact,
                    )
                )
                diagnostics.extend(
                    non_empty_string_array_schema_diagnostics(
                        f"{artifact_label}.command",
                        artifact.get("command"),
                    )
                )
                diagnostics.extend(
                    artifact_safe_relative_path_schema_diagnostics(
                        artifact_label,
                        artifact,
                    )
                )
                if audit.get("fatal") is False:
                    diagnostics.extend(
                        artifact_exit_code_success_schema_diagnostics(
                            artifact_label,
                            artifact,
                        )
                    )
    return diagnostics
