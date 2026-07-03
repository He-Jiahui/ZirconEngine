"""Export-template root resolution helpers."""

from __future__ import annotations

import tomllib
from pathlib import Path
from typing import Any

from .export_template import (
    EXPORT_TEMPLATE_FORMAT_VERSION,
    EXPORT_TEMPLATE_MANIFEST_NAME,
    resolve_export_template_path,
    validate_export_template,
)
from .export_template_manifest import normalize_target_platform


def resolve_export_template_from_root(
    *,
    template_root: Path,
    profile: str,
    expected_engine_version: str | None,
    expected_target_platform: str | None,
) -> dict[str, Any]:
    diagnostics: list[str] = []
    root = resolve_export_template_path(
        label="export template root",
        path=template_root,
        diagnostics=diagnostics,
    )
    report: dict[str, Any] = {
        "template_root": str(root or template_root),
        "profile": profile,
        "expected_engine_version": expected_engine_version,
        "expected_target_platform": expected_target_platform,
        "fatal": False,
        "diagnostics": diagnostics,
        "candidates": [],
        "skipped_candidates": [],
        "template_dir": None,
    }
    if root is None:
        report["fatal"] = True
        return report

    if not root.exists():
        diagnostics.append(f"export template root {root} does not exist")
        report["fatal"] = True
        return report
    if not root.is_dir():
        diagnostics.append(f"export template root {root} is not a directory")
        report["fatal"] = True
        return report

    for manifest_path in sorted(root.glob(f"*/{EXPORT_TEMPLATE_MANIFEST_NAME}")):
        candidate_diagnostics: list[str] = []
        manifest = read_template_manifest_for_resolution(
            manifest_path,
            candidate_diagnostics,
        )
        if manifest is None:
            if candidate_diagnostics:
                report["skipped_candidates"].append(
                    {
                        "template_dir": str(
                            resolve_export_template_path(
                                label="export template directory",
                                path=manifest_path.parent,
                                diagnostics=candidate_diagnostics,
                            )
                            or manifest_path.parent
                        ),
                        "diagnostics": candidate_diagnostics,
                    }
                )
            continue
        if not template_manifest_matches_resolution(
            manifest,
            profile=profile,
            expected_engine_version=expected_engine_version,
            expected_target_platform=expected_target_platform,
        ):
            continue
        candidate_validation = validate_export_template(
            template_dir=manifest_path.parent,
            expected_engine_version=expected_engine_version,
            profile=profile,
            expected_target_platform=expected_target_platform,
        )
        if candidate_validation["fatal"]:
            report["skipped_candidates"].append(
                {
                    "template_dir": str(candidate_validation["template_dir"]),
                    "diagnostics": candidate_validation["diagnostics"],
                }
            )
            continue
        candidate = template_resolution_candidate(
            Path(candidate_validation["template_dir"]),
            manifest,
        )
        report["candidates"].append(candidate)

    candidates = report["candidates"]
    if not candidates:
        target_note = expected_target_platform or "<any>"
        engine_note = expected_engine_version or "<unresolved>"
        diagnostics.append(
            "no export template under "
            f"{root} matched profile={profile} target_platform={target_note} "
            f"engine_version={engine_note}"
        )
    elif len(candidates) > 1:
        diagnostics.append(
            "multiple export templates matched profile="
            f"{profile}: "
            + ", ".join(str(candidate["template_dir"]) for candidate in candidates)
        )
    else:
        report["template_dir"] = candidates[0]["template_dir"]

    report["fatal"] = bool(diagnostics) and report["template_dir"] is None
    return report


def read_template_manifest_for_resolution(
    manifest_path: Path,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    if not manifest_path.is_file():
        diagnostics.append(f"export template manifest {manifest_path} is not a file")
        return None
    try:
        with manifest_path.open("rb") as manifest_file:
            manifest = tomllib.load(manifest_file)
    except OSError as error:
        diagnostics.append(
            f"export template manifest {manifest_path} could not be read: {error}"
        )
        return None
    except tomllib.TOMLDecodeError as error:
        diagnostics.append(
            f"export template manifest {manifest_path} is not valid TOML: {error}"
        )
        return None
    if not isinstance(manifest, dict):
        diagnostics.append(
            f"export template manifest {manifest_path} must be a TOML table"
        )
        return None
    return manifest


def template_manifest_matches_resolution(
    manifest: dict[str, Any],
    *,
    profile: str,
    expected_engine_version: str | None,
    expected_target_platform: str | None,
) -> bool:
    if manifest.get("format_version") != EXPORT_TEMPLATE_FORMAT_VERSION:
        return False
    engine_version = manifest.get("engine_version")
    if expected_engine_version and engine_version != expected_engine_version:
        return False
    target_platform = manifest.get("target_platform")
    if expected_target_platform:
        if not isinstance(target_platform, str):
            return False
        if normalize_target_platform(target_platform) != normalize_target_platform(
            expected_target_platform
        ):
            return False
    compatible_profiles = manifest.get("compatible_profiles", [])
    if not compatible_profiles:
        return True
    if not isinstance(compatible_profiles, list):
        return False
    return profile in compatible_profiles


def template_resolution_candidate(
    template_dir: Path,
    manifest: dict[str, Any],
) -> dict[str, Any]:
    return {
        "template_dir": str(template_dir),
        "template_id": manifest.get("template_id"),
        "engine_version": manifest.get("engine_version"),
        "target_platform": manifest.get("target_platform"),
        "host_artifact": manifest.get("host_artifact"),
        "compatible_profiles": manifest.get("compatible_profiles", []),
        "bundle_format": manifest.get("bundle_format"),
    }
