"""NativeDynamic payload projection for final report aggregation."""

from __future__ import annotations

from typing import Any


def pipeline_native_plugins_payload(
    stage_reports: list[dict[str, Any]],
) -> dict[str, Any] | None:
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "platform_bundle":
            continue
        if stage_report.get("fatal") is True:
            return None
        report = stage_report.get("report")
        if not isinstance(report, dict):
            return None
        native_plugins_payload = report.get("native_plugins_payload")
        if isinstance(native_plugins_payload, dict):
            return native_plugins_payload
        return None
    return None
