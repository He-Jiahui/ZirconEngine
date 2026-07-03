"""Cargo command semantics for standalone plugin builds."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path
from typing import Sequence

from .native_dynamic_contract import native_dynamic_package_directory


PLUGIN_BUILD_DEFAULT_OUT = "zircon-plugin-build"
PLUGIN_BUILD_DIST_FEATURE = "dist"


def default_target_dir(out_root: Path | None, plugin_id: str) -> Path:
    base = out_root if out_root is not None else Path(PLUGIN_BUILD_DEFAULT_OUT)
    return base / ".target" / native_dynamic_package_directory(plugin_id)


def plugin_build_features(
    extra_features: list[str],
    diagnostics: list[str],
) -> list[str]:
    features = [PLUGIN_BUILD_DIST_FEATURE]
    for index, feature in enumerate(extra_features):
        label = f"plugin build features[{index}]"
        if not isinstance(feature, str) or not feature.strip():
            diagnostics.append(f"{label} must be a non-empty string")
            continue
        if feature.strip() != feature:
            diagnostics.append(f"{label} must be trimmed")
            continue
        if feature not in features:
            features.append(feature)
    return features


def plugin_build_cargo_command(
    *,
    cargo: str,
    workspace_manifest: Path,
    dist_crate: str,
    target_dir: Path,
    mode: str,
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
        dist_crate,
        "--target-dir",
        str(target_dir),
        "--no-default-features",
        "--features",
        ",".join(features),
    ]
    if locked:
        command.append("--locked")
    if mode == "release":
        command.append("--release")
    if offline:
        command.append("--offline")
    return command


def run_plugin_build_command(
    command: list[str],
    repo_root: Path,
    diagnostics: list[str],
) -> subprocess.CompletedProcess[str] | None:
    try:
        completed = subprocess.run(
            command,
            cwd=repo_root,
            text=True,
            capture_output=True,
            check=False,
        )
    except OSError as error:
        diagnostics.append(f"plugin build cargo command could not start: {error}")
        return None
    if completed.returncode != 0:
        diagnostics.append(
            f"plugin build cargo command exited with code {completed.returncode}"
        )
        if completed.stderr:
            diagnostics.append(completed.stderr.strip())
    return completed


def shell_join(command: Sequence[str]) -> str:
    return " ".join(shell_quote(part) for part in command)


def shell_quote(value: str) -> str:
    if sys.platform == "win32":
        return subprocess.list2cmdline([value])
    import shlex

    return shlex.quote(value)
