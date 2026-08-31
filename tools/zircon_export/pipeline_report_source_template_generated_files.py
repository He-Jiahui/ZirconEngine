"""SourceTemplate generated file diagnostics for final report validation."""

from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Any

from .pipeline_report_source_template_path_semantics import (
    source_template_generated_file_path,
    source_template_is_non_empty_trimmed_string,
)
from .pipeline_report_source_template_stage_schema import (
    source_template_generated_file_schema_diagnostics,
    source_template_sha256_is_valid,
)
from .pipeline_report_source_template_validate_schema import (
    source_template_validate_generated_file_schema_diagnostics,
)


def source_template_generated_file_diagnostics(
    report: dict[str, Any],
    project_dir: Path,
    validate_report: dict[str, Any] | None,
) -> list[str]:
    generated_files = report.get("generated_files")
    if not isinstance(generated_files, list):
        return ["SourceTemplate report generated_files must be a list"]
    diagnostics: list[str] = []
    report_paths: list[str] = []
    report_path_set: set[str] = set()
    for index, file in enumerate(generated_files):
        if not isinstance(file, dict):
            diagnostics.append("SourceTemplate generated file entry must be an object")
            continue
        diagnostics.extend(source_template_generated_file_schema_diagnostics(file, index))
        path = file.get("path")
        if not isinstance(path, str) or not path.strip():
            diagnostics.append("SourceTemplate generated file path must be a non-empty string")
            continue
        if not source_template_is_non_empty_trimmed_string(path):
            continue
        if path in report_path_set:
            diagnostics.append(
                f"SourceTemplate report generated file path {path} is duplicated"
            )
        report_paths.append(path)
        report_path_set.add(path)
        output_path = source_template_generated_file_path(project_dir, path, diagnostics)
        if output_path is None:
            continue
        if not output_path.exists():
            diagnostics.append(f"SourceTemplate generated file {path} does not exist")
            continue
        if not output_path.is_file():
            diagnostics.append(f"SourceTemplate generated file {path} is not a file")
            continue
        try:
            contents = output_path.read_bytes()
        except OSError as error:
            diagnostics.append(
                f"SourceTemplate generated file {path} could not be read: {error}"
            )
            continue
        expected_size = file.get("size")
        if not isinstance(expected_size, int):
            diagnostics.append(
                f"SourceTemplate generated file {path} size must be an integer"
            )
        elif expected_size != len(contents):
            diagnostics.append(
                f"SourceTemplate generated file {path} size {len(contents)} "
                f"does not match report size {expected_size}"
            )
        expected_sha256 = file.get("sha256")
        if not isinstance(expected_sha256, str) or not source_template_sha256_is_valid(
            expected_sha256
        ):
            diagnostics.append(
                f"SourceTemplate generated file {path} sha256 must be a 64-character hex string"
            )
        else:
            actual_sha256 = hashlib.sha256(contents).hexdigest()
            if actual_sha256 != expected_sha256:
                diagnostics.append(
                    f"SourceTemplate generated file {path} sha256 {actual_sha256} "
                    f"does not match report sha256 {expected_sha256}"
                )
    diagnostics.extend(
        source_template_generated_file_plan_diagnostics(
            report_paths,
            validate_report,
            project_dir,
        )
    )
    return diagnostics


def source_template_generated_file_plan_diagnostics(
    report_paths: list[str],
    validate_report: dict[str, Any] | None,
    project_dir: Path,
) -> list[str]:
    plan_paths, diagnostics = source_template_validate_generated_file_paths(
        validate_report,
        project_dir,
    )
    if plan_paths is None:
        return ["SourceTemplate Validate plan_summary.generated_files must be a list"]
    report_path_set = set(report_paths)
    plan_path_set = set(plan_paths)
    for path in plan_paths:
        if path not in report_path_set:
            diagnostics.append(
                f"SourceTemplate report missing generated file from Validate plan: {path}"
            )
    for path in report_paths:
        if path not in plan_path_set:
            diagnostics.append(
                f"SourceTemplate report generated file {path} is not declared by Validate plan"
            )
    return diagnostics


def source_template_validate_generated_file_paths(
    validate_report: dict[str, Any] | None,
    project_dir: Path,
) -> tuple[list[str] | None, list[str]]:
    diagnostics: list[str] = []
    if not isinstance(validate_report, dict):
        return None, diagnostics
    plan_summary = validate_report.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return None, diagnostics
    generated_files = plan_summary.get("generated_files")
    if not isinstance(generated_files, list):
        return None, diagnostics
    paths: list[str] = []
    path_set: set[str] = set()
    for index, file in enumerate(generated_files):
        if not isinstance(file, dict):
            diagnostics.append("SourceTemplate Validate generated file entry must be an object")
            continue
        diagnostics.extend(
            source_template_validate_generated_file_schema_diagnostics(file, index)
        )
        path = file.get("path")
        if not isinstance(path, str) or not path.strip():
            diagnostics.append(
                "SourceTemplate Validate generated file path must be a non-empty string"
            )
            continue
        if not source_template_is_non_empty_trimmed_string(path):
            continue
        if source_template_generated_file_path(
            project_dir,
            path,
            diagnostics,
            kind="SourceTemplate Validate generated file path",
        ) is None:
            continue
        if path in path_set:
            diagnostics.append(
                f"SourceTemplate Validate generated file path {path} is duplicated"
            )
            continue
        paths.append(path)
        path_set.add(path)
    return paths, diagnostics
