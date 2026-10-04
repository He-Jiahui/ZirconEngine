"""PlatformBundle copied template file schema diagnostics."""

from __future__ import annotations

from .pipeline_report_platform_bundle_template_schema_helpers import (
    table_present_trimmed_non_empty_string_diagnostics,
    table_required_non_empty_string_diagnostics,
    table_string_schema_diagnostics,
    table_unknown_field_diagnostics,
)

PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_FIELDS = ("destination", "source")

PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_STRING_FIELDS = (
    PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_FIELDS
)


def platform_bundle_template_copied_files_schema_diagnostics(
    template_files: list[object],
    label: str = "PlatformBundle report template_files",
) -> list[str]:
    diagnostics: list[str] = []
    seen_entries: dict[tuple[str, str], int] = {}
    for index, entry in enumerate(template_files):
        if not isinstance(entry, dict):
            diagnostics.append(f"{label}[{index}] must be an object")
            continue
        diagnostics.extend(
            table_unknown_field_diagnostics(
                f"{label}[{index}]",
                entry,
                PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_FIELDS,
            )
        )
        diagnostics.extend(
            table_string_schema_diagnostics(
                f"{label}[{index}]",
                entry,
                PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_STRING_FIELDS,
            )
        )
        diagnostics.extend(
            table_required_non_empty_string_diagnostics(
                f"{label}[{index}]",
                entry,
                PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_STRING_FIELDS,
            )
        )
        diagnostics.extend(
            table_present_trimmed_non_empty_string_diagnostics(
                f"{label}[{index}]",
                entry,
                PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_STRING_FIELDS,
            )
        )
        source = entry.get("source")
        destination = entry.get("destination")
        if (
            isinstance(source, str)
            and source.strip()
            and isinstance(destination, str)
            and destination.strip()
        ):
            key = (source, destination)
            previous_index = seen_entries.get(key)
            if previous_index is not None:
                diagnostics.append(
                    f"{label}[{index}] duplicates {label}[{previous_index}]"
                )
            else:
                seen_entries[key] = index
    return diagnostics
