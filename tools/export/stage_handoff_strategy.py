"""Validate strategy handoff helpers for export pipeline stage reports."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .export_strategy_contract import normalize_export_strategy
from .stage_handoff import (
    matching_stage_report,
    stage_report_metadata_handoff_diagnostic,
    stage_report_path,
)


def validate_report_requires_bundle_strategy_diagnostic(
    out_root: Path,
    profile: str,
    stage_label: str,
) -> str | None:
    diagnostics = validate_report_requires_bundle_strategy_diagnostics(
        out_root,
        profile,
        stage_label,
    )
    return diagnostics[0] if diagnostics else None


def validate_report_requires_bundle_strategy_diagnostics(
    out_root: Path,
    profile: str,
    stage_label: str,
) -> list[str]:
    report_path = stage_report_path(out_root, "validate")
    if not report_path.exists():
        return []
    handoff_diagnostic = stage_report_metadata_handoff_diagnostic(
        out_root,
        "validate",
        profile,
    )
    if handoff_diagnostic:
        return [handoff_diagnostic]
    report = matching_stage_report(out_root, "validate", profile)
    if report is None:
        return []
    profile_summary = report.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return []
    if "strategies" not in profile_summary:
        return []
    strategies = profile_summary.get("strategies")
    if not isinstance(strategies, list):
        return ["profile_summary.strategies must be a list"]
    if not strategies:
        return [
            "profile_summary.strategies must include at least one "
            "supported export strategy"
        ]
    unsupported_strategies = unsupported_export_strategies_from_validate_report(report)
    if unsupported_strategies:
        return [
            f"unsupported export strategy {strategy}"
            for strategy in unsupported_strategies
        ]
    normalized_strategies = export_strategies_from_validate_report(report)
    if {"library_embed", "native_dynamic"} & normalized_strategies:
        return []
    return [f"{stage_label} stage requires library_embed or native_dynamic strategy"]


def native_dynamic_payload_allowed(validate_report: dict[str, Any] | None) -> bool:
    if not isinstance(validate_report, dict):
        return True
    profile_summary = validate_report.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return True
    if "strategies" not in profile_summary:
        return True
    strategies = profile_summary.get("strategies")
    if not isinstance(strategies, list):
        return False
    return "native_dynamic" in export_strategies_from_validate_report(validate_report)


def export_strategy_diagnostics(validate_report: dict[str, Any] | None) -> list[str]:
    diagnostics: list[str] = []
    if export_strategy_list_is_invalid(validate_report):
        diagnostics.append("profile_summary.strategies must be a list")
    if export_strategy_list_is_empty(validate_report):
        diagnostics.append(
            "profile_summary.strategies must include at least one supported export strategy"
        )
    diagnostics.extend(
        f"unsupported export strategy {strategy}"
        for strategy in unsupported_export_strategies_from_validate_report(validate_report)
    )
    return diagnostics


def export_strategy_list_is_invalid(validate_report: dict[str, Any] | None) -> bool:
    if not isinstance(validate_report, dict):
        return False
    profile_summary = validate_report.get("profile_summary")
    if not isinstance(profile_summary, dict) or "strategies" not in profile_summary:
        return False
    return not isinstance(profile_summary.get("strategies"), list)


def export_strategy_list_is_empty(validate_report: dict[str, Any] | None) -> bool:
    if not isinstance(validate_report, dict):
        return False
    profile_summary = validate_report.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return False
    strategies = profile_summary.get("strategies")
    return isinstance(strategies, list) and not strategies


def export_strategies_from_validate_report(
    report: dict[str, Any] | None,
) -> set[str]:
    if not isinstance(report, dict):
        return set()
    profile_summary = report.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return set()
    strategies = profile_summary.get("strategies")
    if not isinstance(strategies, list):
        return set()
    normalized: set[str] = set()
    for strategy in strategies:
        normalized_strategy = normalize_export_strategy(strategy)
        if normalized_strategy:
            normalized.add(normalized_strategy)
    return normalized


def unsupported_export_strategies_from_validate_report(
    report: dict[str, Any] | None,
) -> list[str]:
    if not isinstance(report, dict):
        return []
    profile_summary = report.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return []
    strategies = profile_summary.get("strategies")
    if not isinstance(strategies, list):
        return []
    unsupported: list[str] = []
    for strategy in strategies:
        if normalize_export_strategy(strategy):
            continue
        if isinstance(strategy, str):
            unsupported.append(strategy)
        else:
            unsupported.append(repr(strategy))
    return _dedupe(unsupported)


def _dedupe(values: list[str]) -> list[str]:
    deduped: list[str] = []
    seen: set[str] = set()
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        deduped.append(value)
    return deduped
