"""Standalone plugin build preflight helpers."""

from __future__ import annotations

import argparse
from typing import Any


def plugin_distribution_dist_crate(
    distribution: dict[str, Any] | None,
    package_id: str,
    diagnostics: list[str],
) -> str | None:
    if distribution is None:
        return None
    dist_crate = distribution.get("dist_crate")
    if not isinstance(dist_crate, str) or not dist_crate.strip():
        diagnostics.append(f"plugin {package_id} distribution.dist_crate must be a string")
        return None
    if dist_crate.strip() != dist_crate:
        diagnostics.append(f"plugin {package_id} distribution.dist_crate must be trimmed")
        return None
    return dist_crate


def plugin_distribution_abi_version(
    distribution: dict[str, Any] | None,
    package_id: str,
    diagnostics: list[str],
) -> int | None:
    if distribution is None:
        return None
    abi_version = distribution.get("abi_version")
    if not isinstance(abi_version, int):
        diagnostics.append(f"plugin {package_id} distribution.abi_version must be an integer")
        return None
    if abi_version != 3:
        diagnostics.append(f"plugin {package_id} distribution.abi_version must be 3")
        return None
    return abi_version


def plugin_build_optional_trimmed_string(
    value: object,
    field: str,
    diagnostics: list[str],
) -> str | None:
    if value is None:
        return None
    if not isinstance(value, str):
        diagnostics.append(f"{field} must be a string")
        return None
    if not value or value.strip() != value:
        diagnostics.append(f"{field} must be a non-empty trimmed string")
        return None
    return value


def plugin_build_string_array(
    value: object,
    field: str,
    diagnostics: list[str],
    *,
    lowercase: bool = False,
) -> list[str]:
    if value is None:
        return []
    if not isinstance(value, list):
        value = [value]
    values: list[str] = []
    seen: set[str] = set()
    for index, item in enumerate(value):
        if not isinstance(item, str):
            diagnostics.append(f"{field}[{index}] must be a string")
            continue
        if not item or item.strip() != item:
            diagnostics.append(f"{field}[{index}] must be a non-empty trimmed string")
            continue
        normalized = item.lower() if lowercase else item
        if normalized in seen:
            continue
        seen.add(normalized)
        values.append(normalized)
    return values


def plugin_build_failure_report(
    args: argparse.Namespace,
    diagnostics: list[str],
) -> dict[str, object]:
    return {
        "command": "plugin build",
        "plugin_id": args.plugin_id,
        "form": args.form,
        "target_platform": args.target_platform,
        "mode": args.mode,
        "fatal": True,
        "diagnostics": diagnostics,
    }
