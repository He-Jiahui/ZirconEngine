"""SourceTemplate stage report schema diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_schema_primitives import (
    validate_bool_schema_diagnostics,
    validate_integer_schema_diagnostics,
    validate_object_schema_diagnostics,
    validate_string_schema_diagnostics,
)
from .pipeline_report_schema_table import (
    string_array_trimmed_non_empty_entries_schema_diagnostics,
)
from .pipeline_report_source_template_string_array_schema import (
    source_template_non_empty_string_array_schema_diagnostics,
)


SOURCE_TEMPLATE_REPORT_FIELDS = (
    "build_executed",
    "build_validation",
    "cleanup_reason",
    "command",
    "diagnostics",
    "fatal",
    "generated_files",
    "profile",
    "project",
    "project_cleaned",
    "stage",
    "validate_report",
)
SOURCE_TEMPLATE_REPORT_STRING_FIELDS = (
    "project",
    "validate_report",
)
SOURCE_TEMPLATE_REPORT_NULLABLE_STRING_FIELDS = ("cleanup_reason",)
SOURCE_TEMPLATE_REPORT_STRING_ARRAY_FIELDS = ("command",)
SOURCE_TEMPLATE_REPORT_BOOL_FIELDS = (
    "build_executed",
    "project_cleaned",
)
SOURCE_TEMPLATE_REPORT_OBJECT_FIELDS = ("build_validation",)
SOURCE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS = (
    "project",
    "validate_report",
)
SOURCE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_NULLABLE_STRING_FIELDS = (
    "cleanup_reason",
)
SOURCE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS = ("command",)
SOURCE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_BOOL_FIELDS = (
    "build_executed",
    "project_cleaned",
)
SOURCE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS = ("build_validation",)
SOURCE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS = ("generated_files",)

SOURCE_TEMPLATE_GENERATED_FILE_FIELDS = (
    "path",
    "purpose",
    "sha256",
    "size",
)
SOURCE_TEMPLATE_GENERATED_FILE_STRING_FIELDS = (
    "path",
    "purpose",
    "sha256",
)
SOURCE_TEMPLATE_GENERATED_FILE_INTEGER_FIELDS = ("size",)

SOURCE_TEMPLATE_BUILD_VALIDATION_FIELDS = (
    "command",
    "executed",
    "exit_code",
    "requested",
    "stderr_lines",
    "status",
    "stdout_lines",
    "working_dir",
)
SOURCE_TEMPLATE_BUILD_VALIDATION_STRING_FIELDS = (
    "status",
    "working_dir",
)
SOURCE_TEMPLATE_BUILD_VALIDATION_BOOL_FIELDS = (
    "executed",
    "requested",
)
SOURCE_TEMPLATE_BUILD_VALIDATION_STRING_ARRAY_FIELDS = ("command",)
SOURCE_TEMPLATE_BUILD_VALIDATION_OUTPUT_LINE_FIELDS = (
    "stderr_lines",
    "stdout_lines",
)


def source_template_report_schema_diagnostics(report: dict[str, Any]) -> list[str]:
    known_fields = set(SOURCE_TEMPLATE_REPORT_FIELDS)
    diagnostics = [
        f"SourceTemplate report unknown field {field}"
        for field in sorted(report)
        if field not in known_fields
    ]
    for field in SOURCE_TEMPLATE_REPORT_STRING_FIELDS:
        if field in report and report.get(field) is not None:
            value = report.get(field)
            if not isinstance(value, str) or not value.strip():
                diagnostics.append(
                    f"SourceTemplate report {field} must be a non-empty string"
                )
            elif value != value.strip():
                diagnostics.append(
                    f"SourceTemplate report {field} "
                    "must be a non-empty trimmed string"
                )
    for field in SOURCE_TEMPLATE_REPORT_NULLABLE_STRING_FIELDS:
        if field in report:
            diagnostics.extend(
                source_template_nullable_string_schema_diagnostics(
                    f"SourceTemplate report {field}",
                    report.get(field),
                )
            )
    for field in SOURCE_TEMPLATE_REPORT_STRING_ARRAY_FIELDS:
        if field in report:
            value = report.get(field)
            string_array_diagnostics = (
                source_template_non_empty_string_array_schema_diagnostics(
                    f"SourceTemplate report {field}",
                    value,
                )
            )
            if string_array_diagnostics:
                diagnostics.extend(string_array_diagnostics)
            else:
                diagnostics.extend(
                    string_array_trimmed_non_empty_entries_schema_diagnostics(
                        f"SourceTemplate report {field}",
                        value,
                    )
                )
    for field in SOURCE_TEMPLATE_REPORT_BOOL_FIELDS:
        if field in report:
            diagnostics.extend(
                validate_bool_schema_diagnostics(f"SourceTemplate report {field}", report.get(field))
            )
    for field in SOURCE_TEMPLATE_REPORT_OBJECT_FIELDS:
        if field in report:
            diagnostics.extend(
                validate_object_schema_diagnostics(f"SourceTemplate report {field}", report.get(field))
            )
    if report.get("fatal") is False:
        for field in SOURCE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS:
            if field not in report or report.get(field) is None:
                diagnostics.append(
                    f"SourceTemplate report {field} must be a non-empty string"
                )
        for field in SOURCE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_NULLABLE_STRING_FIELDS:
            if field not in report:
                diagnostics.append(
                    f"SourceTemplate report {field} must be a non-empty string or null"
                )
        for field in SOURCE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS:
            if field not in report:
                diagnostics.append(
                    f"SourceTemplate report {field} must be a non-empty string array"
                )
        for field in SOURCE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_BOOL_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_bool_schema_diagnostics(
                        f"SourceTemplate report {field}",
                        report.get(field),
                    )
                )
        for field in SOURCE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_object_schema_diagnostics(
                        f"SourceTemplate report {field}",
                        report.get(field),
                    )
                )
        for field in SOURCE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS:
            if field not in report:
                diagnostics.append(
                    f"SourceTemplate report {field} must be an object array"
                )
    generated_files = report.get("generated_files")
    if "generated_files" in report:
        if not isinstance(generated_files, list):
            diagnostics.append("SourceTemplate report generated_files must be a list")
        else:
            for index, file in enumerate(generated_files):
                if not isinstance(file, dict):
                    diagnostics.append(
                        "SourceTemplate generated file entry must be an object"
                    )
                    continue
                diagnostics.extend(
                    source_template_generated_file_schema_diagnostics(file, index)
                )
    build_validation = report.get("build_validation")
    if isinstance(build_validation, dict):
        diagnostics.extend(
            source_template_build_validation_schema_diagnostics(build_validation)
        )
    return diagnostics


def source_template_generated_file_schema_diagnostics(
    file: dict[str, Any],
    index: int,
) -> list[str]:
    known_fields = set(SOURCE_TEMPLATE_GENERATED_FILE_FIELDS)
    diagnostics = [
        f"SourceTemplate report generated_files[{index}] unknown field {field}"
        for field in sorted(file)
        if field not in known_fields
    ]
    path = file.get("path")
    for field in SOURCE_TEMPLATE_GENERATED_FILE_STRING_FIELDS:
        value = file.get(field)
        if field == "path" and (
            not isinstance(value, str) or not value.strip()
        ):
            diagnostics.append(
                "SourceTemplate generated file path must be a non-empty string"
            )
        elif field == "path" and value != value.strip():
            diagnostics.append(
                "SourceTemplate generated file path "
                "must be a non-empty trimmed string"
            )
        elif not isinstance(value, str):
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    source_template_generated_file_field_label(path, field),
                    value,
                )
            )
        elif field == "purpose" and not value.strip():
            diagnostics.append(
                source_template_generated_file_field_label(path, field)
                + " must be a non-empty string"
            )
        elif field == "purpose" and value != value.strip():
            diagnostics.append(
                source_template_generated_file_field_label(path, field)
                + " must be a non-empty trimmed string"
            )
        elif field == "sha256" and not source_template_sha256_is_valid(value):
            diagnostics.append(
                source_template_generated_file_field_label(path, field)
                + " must be a 64-character hex string"
            )
    for field in SOURCE_TEMPLATE_GENERATED_FILE_INTEGER_FIELDS:
        diagnostics.extend(
            validate_integer_schema_diagnostics(
                source_template_generated_file_field_label(path, field),
                file.get(field),
            )
        )
    return diagnostics


def source_template_build_validation_schema_diagnostics(
    validation: dict[str, Any],
) -> list[str]:
    known_fields = set(SOURCE_TEMPLATE_BUILD_VALIDATION_FIELDS)
    diagnostics = [
        f"SourceTemplate build_validation unknown field {field}"
        for field in sorted(validation)
        if field not in known_fields
    ]
    for field in SOURCE_TEMPLATE_BUILD_VALIDATION_STRING_FIELDS:
        if field in validation:
            value = validation.get(field)
            if not isinstance(value, str) or not value.strip():
                diagnostics.append(
                    f"SourceTemplate build_validation {field} "
                    "must be a non-empty string"
                )
            elif value != value.strip():
                diagnostics.append(
                    f"SourceTemplate build_validation {field} "
                    "must be a non-empty trimmed string"
                )
    for field in SOURCE_TEMPLATE_BUILD_VALIDATION_BOOL_FIELDS:
        if field in validation:
            diagnostics.extend(
                validate_bool_schema_diagnostics(
                    f"SourceTemplate build_validation {field}", validation.get(field)
                )
            )
    for field in SOURCE_TEMPLATE_BUILD_VALIDATION_STRING_ARRAY_FIELDS:
        if field in validation:
            value = validation.get(field)
            string_array_diagnostics = (
                source_template_non_empty_string_array_schema_diagnostics(
                    f"SourceTemplate build_validation {field}",
                    value,
                )
            )
            if string_array_diagnostics:
                diagnostics.extend(string_array_diagnostics)
            else:
                diagnostics.extend(
                    string_array_trimmed_non_empty_entries_schema_diagnostics(
                        f"SourceTemplate build_validation {field}",
                        value,
                    )
                )
    for field in SOURCE_TEMPLATE_BUILD_VALIDATION_OUTPUT_LINE_FIELDS:
        if field in validation:
            diagnostics.extend(
                source_template_output_lines_array_schema_diagnostics(
                    f"SourceTemplate build_validation {field}", validation.get(field)
                )
            )
    if "exit_code" in validation:
        exit_code = validation.get("exit_code")
        if exit_code is not None and (
            not isinstance(exit_code, int) or isinstance(exit_code, bool)
        ):
            diagnostics.append(
                "SourceTemplate build_validation exit_code "
                "must be an integer or null"
            )
    return diagnostics


def source_template_sha256_is_valid(value: str) -> bool:
    if len(value) != 64:
        return False
    return all(character in "0123456789abcdef" for character in value)


def source_template_is_non_empty_string_array(value: Any) -> bool:
    return (
        isinstance(value, list)
        and bool(value)
        and all(isinstance(item, str) and item.strip() for item in value)
    )


def source_template_output_lines_array_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, list):
        return [f"{label} must be a string array"]
    return [
        f"{label}[{index}] must be a string"
        for index, item in enumerate(value)
        if not isinstance(item, str)
    ]


def source_template_nullable_string_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if value is None:
        return []
    if not isinstance(value, str) or not value.strip():
        return [f"{label} must be a non-empty string or null"]
    if value != value.strip():
        return [f"{label} must be a non-empty trimmed string or null"]
    return []


def source_template_generated_file_field_label(path: Any, field: str) -> str:
    if isinstance(path, str) and path:
        return f"SourceTemplate generated file {path} {field}"
    return f"SourceTemplate generated file {field}"
