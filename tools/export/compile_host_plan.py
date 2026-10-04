"""CompileHost plan loading and evidence diagnostics."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .compile_host_plan_command_semantics import (
    compile_host_plan_command_semantic_diagnostics,
)
from .export_template_manifest import is_safe_relative_path, normalize_relative_path
from .command_plan import command_option_value_diagnostic
from .pipeline_report_schema_primitives import (
    validate_object_array_schema_diagnostics,
)
from .pipeline_report_schema_string_array import (
    string_array_duplicate_entry_index_schema_diagnostics,
    string_array_no_blank_entries_schema_diagnostics,
    string_array_trimmed_non_empty_entries_schema_diagnostics,
)
from .pipeline_report_validate_compile_host_linkage_schema import (
    linked_runtime_crates_cover_expected_plugins_diagnostics,
    linked_runtime_crates_only_expected_plugins_diagnostics,
    validate_linked_runtime_crate_schema_diagnostics,
)
from .pipeline_report_validate_compile_host_semantics import (
    compile_host_target_selector_schema_diagnostics,
)
from .pipeline_report_validate_identifier_schema import (
    validate_project_plugin_package_id_array_schema_diagnostics,
    validate_unique_project_plugin_package_id_array_schema_diagnostics,
)
from .pipeline_report_validate_string_array_schema import (
    validate_string_array_schema_diagnostics,
)
from .stage_handoff import (
    load_stage_report_with_diagnostics,
    stage_report_metadata_diagnostic,
)
from .stage_handoff_strategy import (
    export_strategies_from_validate_report,
    export_strategy_diagnostics,
)


COMPILE_HOST_REQUIRED_EVIDENCE_FIELDS = (
    "app_features",
    "expected_runtime_plugins",
    "linked_runtime_crates",
    "manifest_path",
    "package",
    "runtime_features",
    "target_dir",
)
COMPILE_HOST_STRING_EVIDENCE_FIELDS = (
    "binary",
    "cargo_profile",
    "manifest_path",
    "package",
    "target_dir",
)
COMPILE_HOST_PATH_EVIDENCE_FIELDS = (
    "manifest_path",
    "target_dir",
)
COMPILE_HOST_STRING_ARRAY_EVIDENCE_FIELDS = (
    "app_features",
    "runtime_features",
)


def load_compile_host_plan(
    validate_report: Path,
    profile: str,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    if not validate_report.exists():
        diagnostics.append(f"validate report {validate_report} does not exist")
        return None
    report = load_stage_report_with_diagnostics(
        validate_report,
        "validate",
        diagnostics,
    )
    if report is None:
        return None

    metadata_diagnostic = stage_report_metadata_diagnostic(report, "validate", profile)
    if metadata_diagnostic:
        diagnostics.append(metadata_diagnostic)
        return None
    if report.get("fatal"):
        diagnostics.append("validate report is fatal; CompileHost will not run")
        return None
    strategy_diagnostics = export_strategy_diagnostics(report)
    if strategy_diagnostics:
        diagnostics.extend(strategy_diagnostics)
        return None
    if validate_report_requires_compile_host_strategy(report):
        diagnostics.append(
            "CompileHost stage requires library_embed or native_dynamic strategy"
        )
        return None

    plan_summary = report.get("plan_summary")
    if not isinstance(plan_summary, dict):
        diagnostics.append("validate report does not contain plan_summary")
        return None
    compile_plan = plan_summary.get("library_embed_compile_host")
    if not isinstance(compile_plan, dict):
        diagnostics.append("validate report does not contain a LibraryEmbed CompileHost plan")
        return None
    binary = compile_plan.get("binary")
    if not isinstance(binary, str) or not binary.strip():
        diagnostics.append("CompileHost plan binary must be a non-empty string")
        return None
    cargo_profile = compile_plan.get("cargo_profile")
    if not isinstance(cargo_profile, str) or not cargo_profile.strip():
        diagnostics.append("CompileHost plan cargo_profile must be a non-empty string")
        return None
    string_evidence_diagnostics = compile_host_plan_string_evidence_diagnostics(
        compile_plan
    )
    if string_evidence_diagnostics:
        diagnostics.extend(string_evidence_diagnostics)
        return None
    target_selector_diagnostics = compile_host_target_selector_schema_diagnostics(
        compile_plan,
        package_label="CompileHost plan package",
        binary_label="CompileHost plan binary",
    )
    if target_selector_diagnostics:
        diagnostics.extend(target_selector_diagnostics)
        return None
    if cargo_profile not in {"debug", "release"}:
        diagnostics.append("CompileHost plan cargo_profile must be debug or release")
        return None
    release = compile_plan.get("release")
    if not isinstance(release, bool):
        diagnostics.append("CompileHost plan release must be a boolean")
        return None
    if release != (cargo_profile == "release"):
        diagnostics.append("CompileHost plan release must match cargo_profile")
        return None
    missing_fields = [
        field for field in COMPILE_HOST_REQUIRED_EVIDENCE_FIELDS if field not in compile_plan
    ]
    if missing_fields:
        diagnostics.extend(
            f"CompileHost plan {field} is required" for field in missing_fields
        )
        return None
    array_evidence_diagnostics = compile_host_plan_array_evidence_diagnostics(
        compile_plan
    )
    if array_evidence_diagnostics:
        diagnostics.extend(array_evidence_diagnostics)
        return None
    command = compile_plan.get("command")
    if not isinstance(command, list) or not command:
        diagnostics.append("CompileHost plan command must be a non-empty string array")
        return None
    command_string_array_diagnostics = validate_string_array_schema_diagnostics(
        "CompileHost plan command",
        command,
    )
    if command_string_array_diagnostics:
        diagnostics.extend(command_string_array_diagnostics)
        return None
    if any(not value.strip() for value in command):
        diagnostics.append("CompileHost plan command must be a non-empty string array")
        return None
    command_trimmed_diagnostics = string_array_trimmed_non_empty_entries_schema_diagnostics(
        "CompileHost plan command",
        command,
    )
    if command_trimmed_diagnostics:
        diagnostics.extend(command_trimmed_diagnostics)
        return None
    if len(command) < 2 or command[0] != "cargo" or command[1] != "build":
        diagnostics.append("CompileHost plan command must run cargo build")
        return None
    target_dir_diagnostic = command_option_value_diagnostic(
        command,
        "--target-dir",
        "CompileHost plan command",
    )
    if target_dir_diagnostic:
        diagnostics.append(target_dir_diagnostic)
        return None
    command_semantic_diagnostics = compile_host_plan_command_semantic_diagnostics(
        compile_plan
    )
    if command_semantic_diagnostics:
        diagnostics.extend(command_semantic_diagnostics)
        return None
    return compile_plan


def compile_host_plan_string_evidence_diagnostics(
    compile_plan: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    for field in COMPILE_HOST_STRING_EVIDENCE_FIELDS:
        if field not in compile_plan:
            continue
        value = compile_plan.get(field)
        if not isinstance(value, str):
            diagnostics.append(f"CompileHost plan {field} must be a string")
            continue
        if not value.strip() or value != value.strip():
            diagnostics.append(
                f"CompileHost plan {field} must be a non-empty trimmed string"
            )
            continue
        if (
            field in COMPILE_HOST_PATH_EVIDENCE_FIELDS
            and value.strip()
            and value == value.strip()
            and not is_safe_relative_path(normalize_relative_path(value))
        ):
            diagnostics.append(
                f"CompileHost plan {field} must be a safe relative path"
            )
    return diagnostics


def compile_host_plan_array_evidence_diagnostics(
    compile_plan: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    for field in COMPILE_HOST_STRING_ARRAY_EVIDENCE_FIELDS:
        value = compile_plan.get(field)
        label = f"CompileHost plan {field}"
        string_array_diagnostics = validate_string_array_schema_diagnostics(
            label,
            value,
        )
        if string_array_diagnostics:
            diagnostics.extend(string_array_diagnostics)
            continue
        diagnostics.extend(
            string_array_no_blank_entries_schema_diagnostics(
                label,
                value,
            )
        )
        diagnostics.extend(
            string_array_trimmed_non_empty_entries_schema_diagnostics(
                label,
                value,
            )
        )
        diagnostics.extend(
            string_array_duplicate_entry_index_schema_diagnostics(
                label,
                value,
            )
        )

    diagnostics.extend(
        validate_project_plugin_package_id_array_schema_diagnostics(
            "CompileHost plan expected_runtime_plugins",
            compile_plan.get("expected_runtime_plugins"),
        )
    )
    diagnostics.extend(
        validate_unique_project_plugin_package_id_array_schema_diagnostics(
            "CompileHost plan expected_runtime_plugins",
            compile_plan.get("expected_runtime_plugins"),
        )
    )

    linked_runtime_crates = compile_plan.get("linked_runtime_crates")
    linked_crate_label = "CompileHost plan linked_runtime_crates"
    linked_crate_shape_diagnostics = validate_object_array_schema_diagnostics(
        linked_crate_label,
        linked_runtime_crates,
    )
    diagnostics.extend(linked_crate_shape_diagnostics)
    if not linked_crate_shape_diagnostics and isinstance(linked_runtime_crates, list):
        diagnostics.extend(
            validate_linked_runtime_crate_schema_diagnostics(
                linked_runtime_crates,
                label=linked_crate_label,
            )
        )
    diagnostics.extend(
        linked_runtime_crates_cover_expected_plugins_diagnostics(
            compile_plan.get("expected_runtime_plugins"),
            linked_runtime_crates,
            label="CompileHost plan",
            field_separator=" ",
        )
    )
    diagnostics.extend(
        linked_runtime_crates_only_expected_plugins_diagnostics(
            compile_plan.get("expected_runtime_plugins"),
            linked_runtime_crates,
            label="CompileHost plan",
            field_separator=" ",
        )
    )
    return diagnostics


def validate_report_requires_compile_host_strategy(report: dict[str, Any]) -> bool:
    profile_summary = report.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return False
    if "strategies" not in profile_summary:
        return False
    strategies = profile_summary.get("strategies")
    if not isinstance(strategies, list):
        return False
    normalized_strategies = export_strategies_from_validate_report(report)
    return not ({"library_embed", "native_dynamic"} & normalized_strategies)
