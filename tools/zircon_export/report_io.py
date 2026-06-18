"""Report file write helpers for Python-owned export stages."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Iterable

ReportTarget = tuple[str, Path]


def write_report_targets(
    targets: Iterable[ReportTarget],
    report: dict[str, Any],
) -> bool:
    """Write one report payload to all targets, keeping write failures diagnostic."""

    encoded_report = json.dumps(report, indent=2)
    written_targets: list[ReportTarget] = []
    failed = False
    for label, path in targets:
        try:
            path.write_text(encoded_report, encoding="utf-8")
        except OSError as error:
            add_report_write_diagnostic(report, label, path, error)
            failed = True
        else:
            written_targets.append((label, path))

    if not failed:
        return True

    for label, path in written_targets:
        try:
            rewritten_report = json.dumps(report, indent=2)
            path.write_text(rewritten_report, encoding="utf-8")
        except OSError as error:
            add_report_write_diagnostic(report, f"{label} update", path, error)
            remove_stale_report_target(label, path, report)
    return False


def add_report_write_diagnostic(
    report: dict[str, Any],
    label: str,
    path: Path,
    error: OSError,
) -> None:
    report["fatal"] = True
    diagnostics = report.get("diagnostics")
    if not isinstance(diagnostics, list):
        diagnostics = []
        report["diagnostics"] = diagnostics
    diagnostics.append(f"{label} {path} could not be written: {error}")


def remove_stale_report_target(
    label: str,
    path: Path,
    report: dict[str, Any],
) -> None:
    try:
        path.unlink()
    except FileNotFoundError:
        return
    except OSError as error:
        add_report_write_diagnostic(report, f"{label} stale report cleanup", path, error)
