"""Validate report SourceTemplate plan schema diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_schema_primitives import (
    validate_bool_schema_diagnostics,
    validate_object_schema_diagnostics,
    validate_string_schema_diagnostics,
)

SOURCE_TEMPLATE_VALIDATE_BUILD_PLAN_FIELDS = (
    "cargo_profile",
    "command",
    "manifest_path",
    "release",
    "target_dir",
)
SOURCE_TEMPLATE_VALIDATE_BUILD_PLAN_STRING_FIELDS = (
    "cargo_profile",
    "manifest_path",
    "target_dir",
)
SOURCE_TEMPLATE_VALIDATE_BUILD_PLAN_BOOL_FIELDS = ("release",)
SOURCE_TEMPLATE_VALIDATE_BUILD_PLAN_STRING_ARRAY_FIELDS = ("command",)

SOURCE_TEMPLATE_VALIDATE_GENERATED_FILE_FIELDS = (
    "contents",
    "path",
    "purpose",
)
SOURCE_TEMPLATE_VALIDATE_GENERATED_FILE_STRING_FIELDS = (
    "contents",
    "path",
    "purpose",
)


def source_template_validate_build_plan_schema_diagnostics(
    source_template_build: Any,
) -> list[str]:
    object_diagnostics = validate_object_schema_diagnostics(
        "SourceTemplate Validate plan_summary.source_template_build", source_template_build
    )
    if object_diagnostics:
        return object_diagnostics

    known_fields = set(SOURCE_TEMPLATE_VALIDATE_BUILD_PLAN_FIELDS)
    diagnostics = [
        f"SourceTemplate Validate source_template_build unknown field {field}"
        for field in sorted(source_template_build)
        if field not in known_fields
    ]
    for field in SOURCE_TEMPLATE_VALIDATE_BUILD_PLAN_STRING_FIELDS:
        value = source_template_build.get(field)
        if not isinstance(value, str) or not value.strip():
            diagnostics.append(
                f"SourceTemplate Validate source_template_build {field} "
                "must be a non-empty string"
            )
    for field in SOURCE_TEMPLATE_VALIDATE_BUILD_PLAN_BOOL_FIELDS:
        diagnostics.extend(
            validate_bool_schema_diagnostics(
                f"SourceTemplate Validate source_template_build {field}",
                source_template_build.get(field),
            )
        )
    for field in SOURCE_TEMPLATE_VALIDATE_BUILD_PLAN_STRING_ARRAY_FIELDS:
        if not source_template_is_non_empty_string_array(
            source_template_build.get(field)
        ):
            diagnostics.append(
                f"SourceTemplate Validate source_template_build {field} "
                "must be a non-empty string array"
            )
    return diagnostics


def source_template_validate_generated_file_schema_diagnostics(
    file: dict[str, Any],
    index: int,
) -> list[str]:
    known_fields = set(SOURCE_TEMPLATE_VALIDATE_GENERATED_FILE_FIELDS)
    diagnostics = [
        f"SourceTemplate Validate generated_files[{index}] unknown field {field}"
        for field in sorted(file)
        if field not in known_fields
    ]
    for field in SOURCE_TEMPLATE_VALIDATE_GENERATED_FILE_STRING_FIELDS:
        value = file.get(field)
        if field == "path" and (
            not isinstance(value, str) or not value.strip()
        ):
            diagnostics.append(
                "SourceTemplate Validate generated file path "
                "must be a non-empty string"
            )
        elif not isinstance(value, str):
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"SourceTemplate Validate generated_files[{index}].{field}",
                    value,
                )
            )
        elif field == "purpose" and not value.strip():
            diagnostics.append(
                f"SourceTemplate Validate generated_files[{index}].{field} "
                "must be a non-empty string"
            )
    return diagnostics


def source_template_validate_generated_files_schema_diagnostics(
    generated_files: Any,
) -> list[str]:
    if not isinstance(generated_files, list):
        return ["SourceTemplate Validate plan_summary.generated_files must be a list"]

    diagnostics: list[str] = []
    for index, file in enumerate(generated_files):
        if not isinstance(file, dict):
            diagnostics.append(
                "SourceTemplate Validate generated file entry must be an object"
            )
            continue
        diagnostics.extend(
            source_template_validate_generated_file_schema_diagnostics(file, index)
        )
    return diagnostics


def source_template_is_non_empty_string_array(value: Any) -> bool:
    return (
        isinstance(value, list)
        and bool(value)
        and all(isinstance(item, str) and item.strip() for item in value)
    )
