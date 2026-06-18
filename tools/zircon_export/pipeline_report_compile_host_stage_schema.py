"""CompileHost stage report schema diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_schema_primitives import (
    validate_integer_schema_diagnostics,
    validate_object_array_schema_diagnostics,
    validate_object_schema_diagnostics,
    validate_string_array_schema_diagnostics,
    validate_string_schema_diagnostics,
)
from .pipeline_report_validate_compile_host_linkage_schema import (
    validate_linked_runtime_crate_schema_diagnostics,
)
from .pipeline_report_validate_identifier_schema import (
    validate_project_plugin_package_id_array_schema_diagnostics,
)

COMPILE_HOST_REPORT_FIELDS = (
    "command",
    "diagnostics",
    "exit_code",
    "fatal",
    "host_executable",
    "link_plan",
    "profile",
    "stage",
    "stderr_lines",
    "stdout_lines",
)
COMPILE_HOST_REPORT_STRING_FIELDS = ("host_executable",)
COMPILE_HOST_REPORT_STRING_ARRAY_FIELDS = (
    "command",
    "stderr_lines",
    "stdout_lines",
)
COMPILE_HOST_REPORT_INTEGER_FIELDS = ("exit_code",)
COMPILE_HOST_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS = ("host_executable",)
COMPILE_HOST_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS = ("command",)
COMPILE_HOST_REPORT_REQUIRED_NON_FATAL_INTEGER_FIELDS = ("exit_code",)
COMPILE_HOST_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS = ("link_plan",)
COMPILE_HOST_REPORT_REQUIRED_STRING_ARRAY_FIELDS = (
    "stderr_lines",
    "stdout_lines",
)
COMPILE_HOST_LINK_PLAN_FIELDS = (
    "app_features",
    "expected_runtime_plugins",
    "linked_runtime_crates",
    "runtime_features",
)
COMPILE_HOST_LINK_PLAN_STRING_ARRAY_FIELDS = (
    "app_features",
    "runtime_features",
)
COMPILE_HOST_LINK_PLAN_PROJECT_PLUGIN_ID_ARRAY_FIELDS = (
    "expected_runtime_plugins",
)
COMPILE_HOST_LINK_PLAN_REQUIRED_STRING_ARRAY_FIELDS = (
    "app_features",
    "runtime_features",
    "expected_runtime_plugins",
)
COMPILE_HOST_LINK_PLAN_REQUIRED_OBJECT_ARRAY_FIELDS = ("linked_runtime_crates",)


def compile_host_report_schema_diagnostics(report: dict[str, Any]) -> list[str]:
    diagnostics: list[str] = []
    for field in COMPILE_HOST_REPORT_STRING_FIELDS:
        if field in report:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"compile_host report {field}",
                    report.get(field),
                )
            )
    for field in COMPILE_HOST_REPORT_STRING_ARRAY_FIELDS:
        if field in report:
            diagnostics.extend(
                validate_string_array_schema_diagnostics(
                    f"compile_host report {field}",
                    report.get(field),
                )
            )
    for field in COMPILE_HOST_REPORT_REQUIRED_STRING_ARRAY_FIELDS:
        if field not in report:
            diagnostics.extend(
                validate_string_array_schema_diagnostics(
                    f"compile_host report {field}",
                    report.get(field),
                )
            )
    if report.get("fatal") is False:
        for field in COMPILE_HOST_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"compile_host report {field}",
                        report.get(field),
                    )
                )
        for field in COMPILE_HOST_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"compile_host report {field}",
                        report.get(field),
                    )
                )
        for field in COMPILE_HOST_REPORT_REQUIRED_NON_FATAL_INTEGER_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_integer_schema_diagnostics(
                        f"compile_host report {field}",
                        report.get(field),
                    )
                )
        for field in COMPILE_HOST_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_object_schema_diagnostics(
                        f"compile_host report {field}",
                        report.get(field),
                    )
                )
    for field in COMPILE_HOST_REPORT_INTEGER_FIELDS:
        if field in report:
            diagnostics.extend(
                validate_integer_schema_diagnostics(
                    f"compile_host report {field}",
                    report.get(field),
                )
            )
    if "link_plan" in report:
        diagnostics.extend(
            compile_host_link_plan_schema_diagnostics(report.get("link_plan"))
        )
    return diagnostics


def compile_host_link_plan_schema_diagnostics(value: Any) -> list[str]:
    label = "compile_host report link_plan"
    diagnostics = validate_object_schema_diagnostics(label, value)
    if diagnostics:
        return diagnostics
    assert isinstance(value, dict)

    known_fields = set(COMPILE_HOST_LINK_PLAN_FIELDS)
    diagnostics.extend(
        f"{label} unknown field {field}"
        for field in sorted(value)
        if field not in known_fields
    )
    for field in COMPILE_HOST_LINK_PLAN_STRING_ARRAY_FIELDS:
        if field in value:
            diagnostics.extend(
                validate_string_array_schema_diagnostics(
                    f"{label}.{field}",
                    value.get(field),
                )
            )
    for field in COMPILE_HOST_LINK_PLAN_REQUIRED_STRING_ARRAY_FIELDS:
        if field not in value:
            diagnostics.extend(
                validate_string_array_schema_diagnostics(
                    f"{label}.{field}",
                    value.get(field),
                )
            )
    for field in COMPILE_HOST_LINK_PLAN_PROJECT_PLUGIN_ID_ARRAY_FIELDS:
        if field in value:
            diagnostics.extend(
                validate_project_plugin_package_id_array_schema_diagnostics(
                    f"{label}.{field}",
                    value.get(field),
                )
            )
    linked_runtime_crates = value.get("linked_runtime_crates")
    if "linked_runtime_crates" in value:
        diagnostics.extend(
            validate_object_array_schema_diagnostics(
                f"{label}.linked_runtime_crates",
                linked_runtime_crates,
            )
        )
    for field in COMPILE_HOST_LINK_PLAN_REQUIRED_OBJECT_ARRAY_FIELDS:
        if field not in value:
            diagnostics.extend(
                validate_object_array_schema_diagnostics(
                    f"{label}.{field}",
                    value.get(field),
                )
            )
    if isinstance(linked_runtime_crates, list):
        diagnostics.extend(
            validate_linked_runtime_crate_schema_diagnostics(
                linked_runtime_crates,
                label=f"{label}.linked_runtime_crates",
            )
        )
    return diagnostics
