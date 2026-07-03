"""PlatformBundle final report cross-stage handoff diagnostics."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .pipeline_report_platform_bundle_file_evidence import (
    resolve_user_path_or_diagnostic,
)


def delta_verification_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "pack":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        delta_pack = report.get("delta_pack")
        if delta_pack is None:
            continue
        if not isinstance(delta_pack, str) or not delta_pack.strip():
            diagnostics.append("pack report delta_pack must be a non-empty string")
            continue
        if delta_pack.strip() != delta_pack:
            continue
        if report.get("delta_apply_verified") is not True:
            diagnostics.append(
                "pack report delta_pack is present but delta_apply_verified is not true"
            )
    return diagnostics


def platform_bundle_host_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    host_path = None
    compile_host_failed = compile_host_stage_report_failed(stage_reports)
    for stage_report in stage_reports:
        host_path = compile_host_report_host_path(stage_report, diagnostics)
        if host_path is not None:
            break
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "platform_bundle":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        bundled_host = report.get("host_executable")
        if bundled_host is None:
            continue
        if not isinstance(bundled_host, str) or not bundled_host:
            diagnostics.append(
                "PlatformBundle report host_executable must be a non-empty string"
            )
            continue
        host_source = report.get("host_source")
        if host_source is None:
            diagnostics.append(
                "PlatformBundle report host_source is required when host_executable is present"
            )
            continue
        if not isinstance(host_source, str) or not host_source:
            diagnostics.append(
                "PlatformBundle report host_source must be a non-empty string"
            )
            continue
        host_origin = report.get("host_source_origin")
        if host_origin is None:
            diagnostics.append(
                "PlatformBundle report host_source_origin is required when host_executable is present"
            )
            continue
        if not isinstance(host_origin, str) or not host_origin:
            diagnostics.append(
                "PlatformBundle report host_source_origin must be a non-empty string"
            )
            continue
        if host_origin not in {"compile_host_report", "argument", "template"}:
            diagnostics.append(
                "PlatformBundle report host_source_origin must be compile_host_report, argument, or template"
            )
            continue
        if host_origin == "compile_host_report" and host_path is None:
            if compile_host_failed:
                continue
            diagnostics.append(
                "PlatformBundle report host_executable is present but CompileHost report does not contain host_executable evidence"
            )
            continue
        if host_origin != "compile_host_report":
            continue
        resolved_host_source = resolve_user_path_or_diagnostic(
            host_source,
            diagnostics,
            "PlatformBundle report host_source",
        )
        if resolved_host_source is None:
            continue
        if resolved_host_source != host_path:
            diagnostics.append(
                "PlatformBundle report host_source does not match CompileHost report host_executable"
            )
    return diagnostics


def compile_host_stage_report_failed(
    stage_reports: list[dict[str, Any]],
) -> bool:
    return any(
        stage_report.get("stage_key") == "compile_host"
        and stage_report.get("fatal") is True
        for stage_report in stage_reports
    )


def compile_host_report_host_path(
    stage_report: dict[str, Any],
    diagnostics: list[str],
) -> Path | None:
    if stage_report.get("stage_key") != "compile_host":
        return None
    if stage_report.get("fatal") is True:
        return None
    report = stage_report.get("report")
    if not isinstance(report, dict):
        return None
    host = report.get("host_executable")
    if not isinstance(host, str) or not host:
        return None
    return resolve_user_path_or_diagnostic(
        host,
        diagnostics,
        "CompileHost report host_executable",
    )


def platform_bundle_pack_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    pack_path = None
    pack_failed = pack_stage_report_failed(stage_reports)
    for stage_report in stage_reports:
        pack_path = pack_report_pack_path(stage_report, diagnostics)
        if pack_path is not None:
            break
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "platform_bundle":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        bundled_pack = report.get("pack")
        if bundled_pack is None:
            continue
        if not isinstance(bundled_pack, str) or not bundled_pack:
            diagnostics.append("PlatformBundle report pack must be a non-empty string")
            continue
        pack_source = report.get("pack_source")
        if pack_source is None:
            diagnostics.append(
                "PlatformBundle report pack_source is required when pack is present"
            )
            continue
        if not isinstance(pack_source, str) or not pack_source:
            diagnostics.append(
                "PlatformBundle report pack_source must be a non-empty string"
            )
            continue
        pack_origin = report.get("pack_source_origin")
        if pack_origin is None:
            diagnostics.append(
                "PlatformBundle report pack_source_origin is required when pack is present"
            )
            continue
        if not isinstance(pack_origin, str) or not pack_origin:
            diagnostics.append(
                "PlatformBundle report pack_source_origin must be a non-empty string"
            )
            continue
        if pack_origin not in {"pack_report", "argument"}:
            diagnostics.append(
                "PlatformBundle report pack_source_origin must be pack_report or argument"
            )
            continue
        if pack_origin == "pack_report" and pack_path is None:
            if pack_failed:
                continue
            diagnostics.append(
                "PlatformBundle report pack is present but Pack report does not contain pack evidence"
            )
            continue
        if pack_origin != "pack_report":
            continue
        resolved_pack_source = resolve_user_path_or_diagnostic(
            pack_source,
            diagnostics,
            "PlatformBundle report pack_source",
        )
        if resolved_pack_source is None:
            continue
        if resolved_pack_source != pack_path:
            diagnostics.append(
                "PlatformBundle report pack_source does not match Pack report pack"
            )
    return diagnostics


def pack_stage_report_failed(
    stage_reports: list[dict[str, Any]],
) -> bool:
    return any(
        stage_report.get("stage_key") == "pack"
        and stage_report.get("fatal") is True
        for stage_report in stage_reports
    )


def pack_report_pack_path(
    stage_report: dict[str, Any],
    diagnostics: list[str],
) -> Path | None:
    if stage_report.get("stage_key") != "pack":
        return None
    if stage_report.get("fatal") is True:
        return None
    report = stage_report.get("report")
    if not isinstance(report, dict):
        return None
    pack = report.get("pack")
    if not isinstance(pack, str) or not pack:
        return None
    return resolve_user_path_or_diagnostic(
        pack,
        diagnostics,
        "Pack report pack",
    )


def platform_bundle_delta_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    verified_pack_delta = None
    pack_failed = pack_stage_report_failed(stage_reports)
    for stage_report in stage_reports:
        if not pack_report_has_verified_delta(stage_report):
            continue
        verified_pack_delta = pack_report_delta_path(stage_report, diagnostics)
        if verified_pack_delta is not None:
            break
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "platform_bundle":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        delta_pack = report.get("delta_pack")
        if delta_pack is None:
            continue
        if not isinstance(delta_pack, str) or not delta_pack.strip():
            diagnostics.append(
                "PlatformBundle report delta_pack must be a non-empty string"
            )
            continue
        if verified_pack_delta is None:
            if pack_failed:
                continue
            diagnostics.append(
                "PlatformBundle report delta_pack is present but Pack report does not contain verified delta_pack evidence"
            )
            continue
        delta_pack_source = report.get("delta_pack_source")
        if delta_pack_source is None:
            diagnostics.append(
                "PlatformBundle report delta_pack_source is required when delta_pack is present"
            )
            continue
        if not isinstance(delta_pack_source, str) or not delta_pack_source:
            diagnostics.append(
                "PlatformBundle report delta_pack_source must be a non-empty string"
            )
            continue
        delta_origin = report.get("delta_pack_source_origin")
        if delta_origin is None:
            diagnostics.append(
                "PlatformBundle report delta_pack_source_origin is required when delta_pack is present"
            )
            continue
        if not isinstance(delta_origin, str) or not delta_origin:
            diagnostics.append(
                "PlatformBundle report delta_pack_source_origin must be a non-empty string"
            )
            continue
        if delta_origin not in {"pack_report", "argument"}:
            diagnostics.append(
                "PlatformBundle report delta_pack_source_origin must be pack_report or argument"
            )
            continue
        resolved_delta_pack_source = resolve_user_path_or_diagnostic(
            delta_pack_source,
            diagnostics,
            "PlatformBundle report delta_pack_source",
        )
        if resolved_delta_pack_source is None:
            continue
        if resolved_delta_pack_source != verified_pack_delta:
            diagnostics.append(
                "PlatformBundle report delta_pack_source does not match Pack report delta_pack"
            )
    return diagnostics


def native_dynamic_stage_report_failed(
    stage_reports: list[dict[str, Any]],
) -> bool:
    return any(
        stage_report.get("stage_key") == "native_dynamic"
        and stage_report.get("fatal") is True
        for stage_report in stage_reports
    )


def pack_report_has_verified_delta(stage_report: dict[str, Any]) -> bool:
    if stage_report.get("stage_key") != "pack":
        return False
    if stage_report.get("fatal") is True:
        return False
    report = stage_report.get("report")
    if not isinstance(report, dict):
        return False
    delta_pack = report.get("delta_pack")
    return (
        isinstance(delta_pack, str)
        and bool(delta_pack.strip())
        and report.get("delta_apply_verified") is True
    )


def pack_report_delta_path(
    stage_report: dict[str, Any],
    diagnostics: list[str],
) -> Path | None:
    report = stage_report.get("report")
    if not isinstance(report, dict):
        return None
    delta_pack = report.get("delta_pack")
    if not isinstance(delta_pack, str) or not delta_pack.strip():
        return None
    return resolve_user_path_or_diagnostic(
        delta_pack,
        diagnostics,
        "Pack report delta_pack",
    )
