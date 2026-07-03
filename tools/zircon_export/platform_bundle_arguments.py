"""PlatformBundle argument origin and path resolution helpers."""

from __future__ import annotations

import argparse
import os
from pathlib import Path

from .path_resolve import resolve_stage_optional_path


def host_source_origin_from_args(args: argparse.Namespace) -> str | None:
    origin = getattr(args, "host_executable_source_origin", None)
    if isinstance(origin, str) and origin:
        return origin
    if getattr(args, "host_executable_explicit", False):
        return "argument"
    if getattr(args, "host_executable", None) is not None:
        return "argument"
    return None


def pack_source_origin(args: argparse.Namespace) -> str:
    return "argument" if getattr(args, "pack_file_explicit", False) else "pack_report"


def delta_pack_source_origin(args: argparse.Namespace) -> str:
    return "argument" if getattr(args, "delta_pack_explicit", False) else "pack_report"


def platform_bundle_argument_diagnostics(args: argparse.Namespace) -> list[str]:
    diagnostics: list[str] = []
    for field in (
        "host_executable",
        "pack_file",
        "delta_pack",
        "native_plugins_dir",
    ):
        value = getattr(args, field, None)
        if value is not None and (not isinstance(value, str) or not value.strip()):
            diagnostics.append(f"{field} argument must be a non-empty string")
    return diagnostics


def resolve_optional_platform_bundle_path_argument(
    args: argparse.Namespace,
    field: str,
    diagnostics: list[str],
) -> Path | None:
    value = getattr(args, field, None)
    if value is None:
        return None
    if not isinstance(value, str) or not value.strip():
        return None
    try:
        return resolve_user_path(value)
    except OSError as error:
        diagnostics.append(f"{field} {value} could not be resolved: {error}")
        return None


def resolve_platform_bundle_path(
    value: object,
    label: str,
    diagnostics: list[str],
) -> Path | None:
    return resolve_stage_optional_path(value, label, diagnostics, prefix="PlatformBundle")


def resolve_repo_root(repo_root: str | None) -> Path:
    if repo_root:
        return resolve_user_path(repo_root)
    return default_repo_root()


def default_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_user_path(path: str | os.PathLike[str]) -> Path:
    return Path(path).expanduser().resolve()
