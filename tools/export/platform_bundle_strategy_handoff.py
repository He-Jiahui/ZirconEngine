"""PlatformBundle Validate strategy handoff diagnostics."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .stage_handoff import (
    load_stage_report_object,
    stage_report_diagnostics_diagnostic,
    stage_report_fatal_diagnostic,
    stage_report_identity_diagnostic,
    stage_report_metadata_handoff_diagnostic,
)
from .stage_handoff_strategy import (
    export_strategy_diagnostics,
    export_strategies_from_validate_report,
    native_dynamic_payload_allowed,
    validate_report_requires_bundle_strategy_diagnostics,
)


REPORT_FILE_NAME = "report.json"


def platform_bundle_strategy_handoff_diagnostics(
    out_root: Path,
    profile: str,
    native_plugins_payload: dict[str, object] | None,
) -> list[str]:
    diagnostics: list[str] = []
    validate_strategy_diagnostic = validate_report_strategy_handoff_diagnostic(
        out_root,
        profile,
    )
    if validate_strategy_diagnostic:
        diagnostics.append(validate_strategy_diagnostic)
        return diagnostics

    bundle_strategy_diagnostics = validate_report_requires_bundle_strategy_diagnostics(
        out_root,
        profile,
        "PlatformBundle",
    )
    if bundle_strategy_diagnostics:
        diagnostics.extend(bundle_strategy_diagnostics)
        return diagnostics

    diagnostics.extend(validate_report_strategy_diagnostics(out_root, profile))
    if native_plugins_payload is not None and not validate_report_allows_native_plugins(
        out_root,
        profile,
    ):
        diagnostics.append(
            "PlatformBundle report native_plugins requires the native_dynamic strategy"
        )
    elif native_plugins_payload is None:
        if validate_report_uses_strategy(out_root, profile, "native_dynamic"):
            diagnostics.append(
                "NativeDynamic profile requires native plugins from a matching non-fatal "
                "NativeDynamic stage report or --native-plugins-dir"
            )
    return diagnostics


def validate_report_uses_strategy(out_root: Path, profile: str, strategy: str) -> bool:
    report = load_trusted_validate_strategy_report(out_root, profile)
    if report is None:
        return False
    return strategy in export_strategies_from_validate_report(report)


def validate_report_allows_native_plugins(out_root: Path, profile: str) -> bool:
    report, diagnostic = load_trusted_validate_strategy_report_with_diagnostic(
        out_root,
        profile,
    )
    if diagnostic:
        return False
    return native_dynamic_payload_allowed(report)


def validate_report_strategy_diagnostics(
    out_root: Path,
    profile: str,
) -> list[str]:
    report, diagnostic = load_trusted_validate_strategy_report_with_diagnostic(
        out_root,
        profile,
    )
    if diagnostic:
        return [diagnostic]
    return export_strategy_diagnostics(report)


def load_trusted_validate_strategy_report(
    out_root: Path,
    profile: str,
) -> dict[str, Any] | None:
    report, _diagnostic = load_trusted_validate_strategy_report_with_diagnostic(
        out_root,
        profile,
    )
    return report


def load_trusted_validate_strategy_report_with_diagnostic(
    out_root: Path,
    profile: str,
) -> tuple[dict[str, Any] | None, str | None]:
    report_path = out_root / "stages" / "validate" / REPORT_FILE_NAME
    if not report_path.exists():
        return None, None
    report, diagnostic = load_stage_report_object(report_path, "Validate")
    if diagnostic:
        return None, diagnostic
    if report is None:
        return None, None
    metadata_diagnostic = (
        stage_report_identity_diagnostic(report, "validate")
        or stage_report_fatal_diagnostic(report, "validate")
        or stage_report_diagnostics_diagnostic(report, "validate")
    )
    if metadata_diagnostic:
        return None, metadata_diagnostic
    if report.get("fatal") or report.get("profile") != profile:
        return None, None
    return report, None


def validate_report_strategy_handoff_diagnostic(
    out_root: Path,
    profile: str,
) -> str | None:
    return stage_report_metadata_handoff_diagnostic(out_root, "validate", profile)
