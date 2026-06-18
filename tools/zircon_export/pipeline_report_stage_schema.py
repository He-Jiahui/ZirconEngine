"""Stage report schema diagnostics for Zircon export final reports."""

from __future__ import annotations

from typing import Any

from .pipeline_report_compile_host_stage_schema import (
    COMPILE_HOST_REPORT_FIELDS,
    compile_host_report_schema_diagnostics,
)
from .pipeline_report_cook_assets_stage_schema import (
    COOK_ASSETS_REPORT_FIELDS,
    cook_assets_report_schema_diagnostics,
)
from .pipeline_report_native_dynamic_package_export_schema import (
    native_dynamic_package_export_schema_diagnostics as package_export_schema_diagnostics,
)
from .pipeline_report_native_dynamic_stage_schema import (
    NATIVE_DYNAMIC_REPORT_FIELDS,
    native_dynamic_report_schema_diagnostics,
)
from .pipeline_report_pack_stage_schema import (
    PACK_REPORT_FIELDS,
    pack_report_schema_diagnostics,
)
from .pipeline_report_schema_primitives import (
    validate_bool_schema_diagnostics,
    validate_integer_schema_diagnostics,
    validate_object_array_schema_diagnostics,
    validate_object_schema_diagnostics,
    validate_string_array_schema_diagnostics,
    validate_string_schema_diagnostics,
)
from .pipeline_report_platform_bundle_schema import (
    PLATFORM_BUNDLE_REPORT_FIELDS,
    platform_bundle_report_schema_diagnostics,
)
from .pipeline_report_source_template_stage_schema import (
    SOURCE_TEMPLATE_REPORT_FIELDS,
    source_template_generated_file_schema_diagnostics,
    source_template_report_schema_diagnostics,
)
from .pipeline_report_validate_stage_schema import (
    VALIDATE_REPORT_FIELDS,
    validate_report_schema_diagnostics,
)

STAGE_REPORT_FIELDS = {
    "validate": VALIDATE_REPORT_FIELDS,
    "compile_host": COMPILE_HOST_REPORT_FIELDS,
    "cook_assets": COOK_ASSETS_REPORT_FIELDS,
    "native_dynamic": NATIVE_DYNAMIC_REPORT_FIELDS,
    "pack": PACK_REPORT_FIELDS,
    "platform_bundle": PLATFORM_BUNDLE_REPORT_FIELDS,
    "source_template": SOURCE_TEMPLATE_REPORT_FIELDS,
}

def stage_report_schema_diagnostics(
    stage_key: str,
    report: dict[str, Any],
) -> list[str]:
    known_fields = STAGE_REPORT_FIELDS.get(stage_key)
    if known_fields is None:
        return []
    known_field_set = set(known_fields)
    diagnostics = []
    if stage_key != "platform_bundle":
        diagnostics.extend(
            f"{stage_key} report unknown field {field}"
            for field in sorted(report)
            if field not in known_field_set
        )
    if stage_key == "compile_host":
        diagnostics.extend(compile_host_report_schema_diagnostics(report))
    if stage_key == "cook_assets":
        diagnostics.extend(cook_assets_report_schema_diagnostics(report))
    if stage_key == "native_dynamic":
        diagnostics.extend(
            native_dynamic_report_schema_diagnostics(
                report,
                validate_bool_schema_diagnostics=validate_bool_schema_diagnostics,
                validate_integer_schema_diagnostics=(
                    validate_integer_schema_diagnostics
                ),
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
                validate_string_array_schema_diagnostics=(
                    validate_string_array_schema_diagnostics
                ),
                validate_object_schema_diagnostics=validate_object_schema_diagnostics,
                validate_object_array_schema_diagnostics=(
                    validate_object_array_schema_diagnostics
                ),
                native_dynamic_package_export_schema_diagnostics=(
                    package_export_schema_diagnostics
                ),
            )
        )
    if stage_key == "pack":
        diagnostics.extend(
            pack_report_schema_diagnostics(
                report,
                validate_bool_schema_diagnostics=validate_bool_schema_diagnostics,
                validate_integer_schema_diagnostics=(
                    validate_integer_schema_diagnostics
                ),
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
                validate_string_array_schema_diagnostics=(
                    validate_string_array_schema_diagnostics
                ),
                validate_object_schema_diagnostics=validate_object_schema_diagnostics,
                validate_object_array_schema_diagnostics=(
                    validate_object_array_schema_diagnostics
                ),
            )
        )
    if stage_key == "platform_bundle":
        diagnostics.extend(platform_bundle_report_schema_diagnostics(report))
    if stage_key == "source_template":
        diagnostics.extend(source_template_report_schema_diagnostics(report))
    if stage_key == "validate":
        diagnostics.extend(validate_report_schema_diagnostics(report))
    return diagnostics


