"""NativeDynamic Cargo command and artifact naming helpers."""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

from .pipeline_report_validate_profile_summary_schema import (
    VALIDATE_PROFILE_SUMMARY_BUILD_MODES,
)


NATIVE_BUILD_DEFAULT_MODE = "debug"


def native_dynamic_cargo_profile(
    validate_payload: dict[str, Any] | None,
    diagnostics: list[str],
) -> str:
    if isinstance(validate_payload, dict):
        profile_summary = validate_payload.get("profile_summary")
        if isinstance(profile_summary, dict):
            build_mode = profile_summary.get("build_mode")
            if "build_mode" not in profile_summary:
                return NATIVE_BUILD_DEFAULT_MODE
            if not isinstance(build_mode, str):
                diagnostics.append(
                    "validate report profile_summary.build_mode must be a string"
                )
                return NATIVE_BUILD_DEFAULT_MODE
            if not build_mode.strip() or build_mode.strip() != build_mode:
                diagnostics.append(
                    "validate report profile_summary.build_mode "
                    "must be a non-empty trimmed export build mode"
                )
                return NATIVE_BUILD_DEFAULT_MODE
            normalized_mode = build_mode.lower()
            if normalized_mode not in VALIDATE_PROFILE_SUMMARY_BUILD_MODES:
                diagnostics.append(
                    "validate report profile_summary.build_mode "
                    "must be a known export build mode"
                )
                return NATIVE_BUILD_DEFAULT_MODE
            if normalized_mode == "release":
                return "release"
    return NATIVE_BUILD_DEFAULT_MODE


def native_dynamic_cargo_build_command(
    *,
    cargo: str,
    workspace_manifest: Path,
    crate_name: str,
    target_dir: Path,
    cargo_profile: str,
    locked: bool,
    offline: bool,
    features: list[str],
) -> list[str]:
    command = [
        cargo,
        "build",
        "--manifest-path",
        str(workspace_manifest),
        "-p",
        crate_name,
        "--target-dir",
        str(target_dir),
    ]
    if locked:
        command.append("--locked")
    if features:
        command.extend(["--features", ",".join(features)])
    if cargo_profile == "release":
        command.append("--release")
    if offline:
        command.append("--offline")
    return command


def native_dynamic_expected_loadable_artifact(
    target_dir: Path,
    cargo_profile: str,
    crate_name: str,
    target_platform: str | None,
) -> Path:
    return target_dir / cargo_profile / platform_dynamic_library_name(
        crate_name,
        target_platform,
    )


def platform_dynamic_library_name(crate_name: str, target_platform: str | None) -> str:
    if target_platform:
        platform = target_platform.split("-", maxsplit=1)[0].lower()
        if platform == "windows":
            return f"{crate_name}.dll"
        if platform == "macos":
            return f"lib{crate_name}.dylib"
        if platform == "linux":
            return f"lib{crate_name}.so"
    if os.name == "nt":
        return f"{crate_name}.dll"
    if hasattr(os, "uname") and os.uname().sysname.lower() == "darwin":
        return f"lib{crate_name}.dylib"
    return f"lib{crate_name}.so"


def normalized_native_dynamic_build_features(
    features: list[str],
    diagnostics: list[str],
) -> list[str]:
    result: list[str] = []
    seen: set[str] = set()
    for index, feature in enumerate(features):
        feature_label = f"NativeDynamic native build features[{index}]"
        if not isinstance(feature, str):
            diagnostics.append(f"{feature_label} must be a string")
            continue
        if not feature or feature.strip() != feature:
            diagnostics.append(f"{feature_label} must be a non-empty trimmed string")
            continue
        if feature in seen:
            continue
        seen.add(feature)
        result.append(feature)
    return result
