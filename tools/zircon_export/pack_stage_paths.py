"""Pack stage path and argument preflight helpers."""

from __future__ import annotations

import argparse
import os
from pathlib import Path

from .cook_assets import default_cooked_asset_manifest
from .path_resolve import resolve_stage_optional_path


def pack_asset_manifest_argument_diagnostic(args: argparse.Namespace) -> str | None:
    return pack_optional_path_argument_diagnostic(
        getattr(args, "asset_manifest", None),
        "asset_manifest",
    )


def pack_file_argument_diagnostic(args: argparse.Namespace) -> str | None:
    return pack_optional_path_argument_diagnostic(
        getattr(args, "pack_file", None),
        "pack_file",
    )


def pack_optional_path_argument_diagnostic(
    value: object,
    label: str,
) -> str | None:
    if value is None:
        return None
    if not isinstance(value, str) or not value.strip():
        return f"{label} argument must be a non-empty string"
    if value.strip() != value:
        return f"{label} argument must be a non-empty trimmed string"
    return None


def pack_asset_manifest_path(
    args: argparse.Namespace,
    out_root: Path,
    asset_manifest_argument_diagnostic: str | None,
    cook_assets_handoff_diagnostic: str | None,
    diagnostics: list[str],
) -> Path | None:
    if asset_manifest_argument_diagnostic or cook_assets_handoff_diagnostic:
        return None
    if args.asset_manifest is not None:
        return resolve_pack_optional_path(args.asset_manifest, "asset_manifest", diagnostics)
    return default_cooked_asset_manifest(out_root)


def pack_output_path(
    args: argparse.Namespace,
    stage_dir: Path,
    pack_file_argument_diagnostic: str | None,
    diagnostics: list[str],
) -> Path | None:
    if pack_file_argument_diagnostic:
        return None
    if args.pack_file is not None:
        return resolve_pack_optional_path(args.pack_file, "pack_file", diagnostics)
    return stage_dir / "assets.zrpack"


def resolve_pack_optional_path(
    value: object,
    label: str,
    diagnostics: list[str],
) -> Path | None:
    if value is None:
        return None
    if not isinstance(value, str) or not value.strip():
        return None
    try:
        return resolve_user_path(value)
    except OSError as error:
        diagnostics.append(f"{label} {value} could not be resolved: {error}")
        return None


def resolve_pack_stage_path(
    value: object,
    label: str,
    diagnostics: list[str],
) -> Path | None:
    return resolve_stage_optional_path(value, label, diagnostics, prefix="Pack")


def pack_delta_argument_diagnostics(args: argparse.Namespace) -> list[str]:
    previous_pack = getattr(args, "previous_pack", None)
    delta_pack = getattr(args, "delta_pack", None)
    diagnostics: list[str] = []
    previous_pack_diagnostic = pack_optional_path_argument_diagnostic(
        previous_pack,
        "previous_pack",
    )
    if previous_pack_diagnostic:
        diagnostics.append(previous_pack_diagnostic)
    delta_pack_diagnostic = pack_optional_path_argument_diagnostic(
        delta_pack,
        "delta_pack",
    )
    if delta_pack_diagnostic:
        diagnostics.append(delta_pack_diagnostic)
    if not diagnostics and ((previous_pack is None) != (delta_pack is None)):
        diagnostics.append("previous_pack and delta_pack must be supplied together")
    return diagnostics


def pack_asset_manifest_diagnostic(asset_manifest: Path) -> str | None:
    if not asset_manifest.exists():
        return (
            f"asset manifest {asset_manifest} does not exist; "
            "run CookAssets first or pass --asset-manifest"
        )
    if not asset_manifest.is_file():
        return f"asset manifest {asset_manifest} is not a file"
    return None


def resolve_user_path(path: str | os.PathLike[str]) -> Path:
    return Path(path).expanduser().resolve()
