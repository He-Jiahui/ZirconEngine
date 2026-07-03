"""PlatformBundle NativeDynamic stage-report handoff diagnostics."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass
class PlatformBundleNativePluginsStageReportHandoff:
    diagnostics: list[str]
    payload_stage_report_matches: bool
    suppress_unbacked_stage_audits: bool
    effective_native_dynamic_report_path: Path | None


def _resolve_user_path(path: str | Path) -> Path:
    return Path(path).expanduser().resolve()


def _resolve_user_path_or_diagnostic(
    path: str | Path,
    diagnostics: list[str],
    label: str,
) -> Path | None:
    try:
        return _resolve_user_path(path)
    except OSError as error:
        diagnostics.append(f"{label} {path} could not be resolved: {error}")
        return None


def stage_payload_source_diagnostics(
    payload: dict[str, Any],
    payload_stage_report_path: Path,
) -> list[str]:
    diagnostics: list[str] = []
    payload_source = payload.get("source")
    expected_source = payload_stage_report_path.parent / "plugins"
    if not isinstance(payload_source, str) or not payload_source:
        return [
            "PlatformBundle report native_plugins_payload source must be a non-empty string for stage-backed payloads"
        ]
    payload_source_path = _resolve_user_path_or_diagnostic(
        payload_source,
        diagnostics,
        "PlatformBundle report native_plugins_payload source",
    )
    expected_source_path = _resolve_user_path_or_diagnostic(
        expected_source,
        diagnostics,
        "PlatformBundle expected NativeDynamic plugins source",
    )
    if (
        payload_source_path is not None
        and expected_source_path is not None
        and payload_source_path != expected_source_path
    ):
        diagnostics.append(
            "PlatformBundle report native_plugins_payload source "
            f"{payload_source_path} does not match NativeDynamic plugins "
            f"{expected_source_path}"
        )
    return diagnostics


def platform_bundle_native_plugins_stage_report_handoff(
    payload: dict[str, Any],
    plugins_dir: Path,
    native_dynamic_report_path: Path | None,
    *,
    native_dynamic_stage_report_failed: bool = False,
) -> PlatformBundleNativePluginsStageReportHandoff:
    diagnostics: list[str] = []
    payload_stage_report_matches = False
    suppress_unbacked_stage_audits = False
    effective_native_dynamic_report_path = native_dynamic_report_path
    payload_stage_report = payload.get("stage_report")
    if payload_stage_report is None:
        if native_dynamic_report_path is not None:
            diagnostics.append(
                "PlatformBundle report native_plugins_payload stage_report is required "
                "when a non-fatal NativeDynamic stage report is present"
            )
    elif not isinstance(payload_stage_report, str) or not payload_stage_report:
        diagnostics.append(
            "PlatformBundle report native_plugins_payload stage_report must be null or a non-empty string"
        )
    elif native_dynamic_report_path is None:
        payload_stage_report_path = _resolve_user_path_or_diagnostic(
            payload_stage_report,
            diagnostics,
            "PlatformBundle report native_plugins_payload stage_report",
        )
        expected_stage_report_path = current_output_native_dynamic_report_path(
            plugins_dir,
            diagnostics,
        )
        if (
            payload_stage_report_path is not None
            and expected_stage_report_path is not None
            and payload_stage_report_path != expected_stage_report_path
        ):
            diagnostics.append(
                "PlatformBundle report native_plugins_payload stage_report "
                f"{payload_stage_report_path} does not match NativeDynamic report "
                f"{expected_stage_report_path}"
            )
        elif native_dynamic_stage_report_failed:
            suppress_unbacked_stage_audits = True
        elif payload_stage_report_path is None:
            pass
        elif not payload_stage_report_path.exists():
            diagnostics.append(
                "PlatformBundle report native_plugins_payload stage_report is present "
                f"but NativeDynamic report {payload_stage_report_path} is missing"
            )
        else:
            payload_stage_report_matches = True
            effective_native_dynamic_report_path = payload_stage_report_path
            diagnostics.extend(
                stage_payload_source_diagnostics(
                    payload,
                    payload_stage_report_path,
                )
            )
    else:
        payload_stage_report_path = _resolve_user_path_or_diagnostic(
            payload_stage_report,
            diagnostics,
            "PlatformBundle report native_plugins_payload stage_report",
        )
        expected_report_path = _resolve_user_path_or_diagnostic(
            native_dynamic_report_path,
            diagnostics,
            "NativeDynamic report path",
        )
        if payload_stage_report_path is None or expected_report_path is None:
            pass
        elif payload_stage_report_path != expected_report_path:
            diagnostics.append(
                "PlatformBundle report native_plugins_payload stage_report "
                f"{payload_stage_report_path} does not match NativeDynamic report "
                f"{expected_report_path}"
            )
        else:
            payload_stage_report_matches = True
            diagnostics.extend(
                stage_payload_source_diagnostics(
                    payload,
                    payload_stage_report_path,
                )
            )
    return PlatformBundleNativePluginsStageReportHandoff(
        diagnostics=diagnostics,
        payload_stage_report_matches=payload_stage_report_matches,
        suppress_unbacked_stage_audits=suppress_unbacked_stage_audits,
        effective_native_dynamic_report_path=effective_native_dynamic_report_path,
    )


def current_output_native_dynamic_report_path(
    plugins_dir: Path,
    diagnostics: list[str] | None = None,
) -> Path | None:
    try:
        return plugins_dir.resolve().parents[2] / "stages" / "native_dynamic" / "report.json"
    except IndexError:
        return None
    except OSError as error:
        if diagnostics is not None:
            diagnostics.append(
                f"PlatformBundle report native_plugins {plugins_dir} "
                f"could not be resolved: {error}"
            )
        return None


def native_dynamic_stage_report_path(
    stage_reports: list[dict[str, Any]],
    diagnostics: list[str] | None = None,
) -> Path | None:
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "native_dynamic":
            continue
        if stage_report.get("fatal") is True:
            return None
        report_path = stage_report.get("path")
        if not isinstance(report_path, str) or not report_path:
            return None
        if diagnostics is None:
            return _resolve_user_path(report_path)
        return _resolve_user_path_or_diagnostic(
            report_path,
            diagnostics,
            "NativeDynamic stage report path",
        )
    return None
