"""CompileHost plan command semantic diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_validate_compile_host_command_value_semantics import (
    command_alias_value_match_diagnostics,
    command_features_match_diagnostics,
    command_option_path_value_match_diagnostics,
    command_option_value_match_diagnostics,
    compile_host_release_flag_schema_diagnostics,
)
from .pipeline_report_validate_compile_host_command_semantics import (
    command_forbidden_flag_diagnostics,
    command_flag_diagnostics,
    compile_host_command_forbidden_package_diagnostics,
    compile_host_command_forbidden_profile_diagnostics,
    compile_host_command_forbidden_target_diagnostics,
    compile_host_command_forbidden_target_triple_diagnostics,
    compile_host_command_forbidden_wrapper_policy_diagnostics,
)


def compile_host_plan_command_semantic_diagnostics(
    compile_plan: dict[str, Any],
) -> list[str]:
    command = compile_plan.get("command")
    if not isinstance(command, list) or any(
        not isinstance(entry, str) for entry in command
    ):
        return []

    label = "CompileHost plan"
    command_label = f"{label} command"
    diagnostics: list[str] = []
    diagnostics.extend(
        command_flag_diagnostics(
            command,
            "--no-default-features",
            label=command_label,
        )
    )
    diagnostics.extend(
        command_forbidden_flag_diagnostics(
            command,
            "--all-features",
            label=command_label,
            reason="because CompileHost plan app_features owns feature selection",
        )
    )
    diagnostics.extend(
        compile_host_command_forbidden_target_diagnostics(
            command,
            label=command_label,
        )
    )
    diagnostics.extend(
        compile_host_command_forbidden_target_triple_diagnostics(
            command,
            label=command_label,
        )
    )
    diagnostics.extend(
        compile_host_command_forbidden_package_diagnostics(
            command,
            label=command_label,
        )
    )
    diagnostics.extend(
        compile_host_command_forbidden_profile_diagnostics(
            command,
            label=command_label,
        )
    )
    diagnostics.extend(
        compile_host_command_forbidden_wrapper_policy_diagnostics(
            command,
            label=command_label,
        )
    )
    diagnostics.extend(
        command_alias_value_match_diagnostics(
            command,
            ("-p", "--package"),
            compile_plan.get("package"),
            f"{label} package",
            label=command_label,
            option_label="-p/--package",
        )
    )
    diagnostics.extend(
        command_option_value_match_diagnostics(
            command,
            "--bin",
            compile_plan.get("binary"),
            f"{label} binary",
            label=command_label,
        )
    )
    diagnostics.extend(
        command_features_match_diagnostics(
            command,
            compile_plan.get("app_features"),
            f"{label} app_features",
            label=command_label,
        )
    )
    diagnostics.extend(
        command_option_path_value_match_diagnostics(
            command,
            "--target-dir",
            compile_plan.get("target_dir"),
            f"{label} target_dir",
            label=command_label,
        )
    )
    diagnostics.extend(
        command_option_path_value_match_diagnostics(
            command,
            "--manifest-path",
            compile_plan.get("manifest_path"),
            f"{label} manifest_path",
            label=command_label,
        )
    )
    diagnostics.extend(
        compile_host_release_flag_schema_diagnostics(
            command,
            compile_plan,
            label=command_label,
        )
    )
    return diagnostics
