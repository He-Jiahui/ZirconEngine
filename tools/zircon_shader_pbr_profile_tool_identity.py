#!/usr/bin/env python3
"""Validate the exact tool implementation used by a shader PBR profile."""

from __future__ import annotations

import hashlib
import re
from collections.abc import Mapping
from pathlib import Path, PurePosixPath


PROFILE_TOOL_PATHS = (
    "tools/performance-machine-manifest.ps1",
    "tools/profile-capture-manifest.ps1",
    "tools/shader-pbr-profile-contract.ps1",
    "tools/shader-pbr-profile-evidence-identity.ps1",
    "tools/shader-pbr-profile-publication.ps1",
    "tools/shader-pbr-profile-runtime-evidence.ps1",
    "tools/shader-pbr-profile-toolchain.ps1",
    "tools/write_zircon_shader_pbr_build_provenance.ps1",
    "tools/zircon_pbr_visual_oracle.py",
    "tools/zircon_profile_shader_pbr_viewer.ps1",
    "tools/zircon_shader_pbr_evidence_identity.py",
    "tools/zircon_shader_pbr_profile_tool_identity.py",
    "tools/zircon_summarize_shader_pbr_profile.py",
    "tools/zircon_validate_shader_pbr_gpu_timing_evidence.py",
    "tools/zircon_validate_shader_pbr_renderdoc_replay.py",
    "tools/zircon_validate_shader_pbr_viewer_evidence.py",
)

_FINGERPRINT_FIELDS = frozenset({"relative_path", "sha256", "byte_length"})
_SAFE_RELATIVE_PATH_PATTERN = re.compile(r"[A-Za-z0-9._/-]+\Z")
_SHA256_PATTERN = re.compile(r"[0-9a-f]{64}\Z")


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _is_path_within(root: Path, candidate: Path) -> bool:
    try:
        candidate.relative_to(root)
    except ValueError:
        return False
    return True


def _require_safe_relative_path(value: object, manifest_path: Path) -> str:
    if not isinstance(value, str) or not value:
        raise RuntimeError(
            f"Shader PBR profile manifest has an invalid profile tool path: path={manifest_path}"
        )
    if _SAFE_RELATIVE_PATH_PATTERN.fullmatch(value) is None or "\\" in value:
        raise RuntimeError(
            f"Shader PBR profile manifest has an unsafe profile tool path: "
            f"relative_path={value!r} path={manifest_path}"
        )
    segments = value.split("/")
    relative_path = PurePosixPath(value)
    if relative_path.is_absolute() or any(segment in {"", ".", ".."} for segment in segments):
        raise RuntimeError(
            f"Shader PBR profile manifest has an unsafe profile tool path: "
            f"relative_path={value!r} path={manifest_path}"
        )
    return value


def validate_profile_tool_files(
    repository: Mapping[str, object],
    repository_root: Path,
    manifest_path: Path,
) -> None:
    records = repository.get("profile_tool_files")
    if not isinstance(records, list) or not records:
        raise RuntimeError(
            f"Shader PBR profile manifest has no profile tool files: path={manifest_path}"
        )

    expected_paths = frozenset(PROFILE_TOOL_PATHS)
    fingerprints: dict[str, tuple[Path, str, int]] = {}
    for value in records:
        if not isinstance(value, Mapping) or frozenset(value) != _FINGERPRINT_FIELDS:
            raise RuntimeError(
                f"Shader PBR profile manifest has an invalid profile tool fingerprint: "
                f"path={manifest_path}"
            )
        relative_path = _require_safe_relative_path(value.get("relative_path"), manifest_path)
        if relative_path in fingerprints:
            raise RuntimeError(
                f"Shader PBR profile manifest has a duplicate profile tool path: "
                f"relative_path={relative_path!r} path={manifest_path}"
            )
        sha256 = value.get("sha256")
        byte_length = value.get("byte_length")
        if not isinstance(sha256, str) or _SHA256_PATTERN.fullmatch(sha256) is None:
            raise RuntimeError(
                f"Shader PBR profile manifest has an invalid profile tool SHA-256: "
                f"relative_path={relative_path!r} path={manifest_path}"
            )
        if isinstance(byte_length, bool) or not isinstance(byte_length, int) or byte_length < 0:
            raise RuntimeError(
                f"Shader PBR profile manifest has an invalid profile tool byte length: "
                f"relative_path={relative_path!r} path={manifest_path}"
            )
        tool_path = (repository_root / Path(relative_path)).resolve()
        if not _is_path_within(repository_root, tool_path):
            raise RuntimeError(
                f"Shader PBR profile tool escapes the repository: "
                f"relative_path={relative_path!r} path={manifest_path}"
            )
        fingerprints[relative_path] = (tool_path, sha256, byte_length)

    actual_paths = frozenset(fingerprints)
    if actual_paths != expected_paths:
        missing = sorted(expected_paths - actual_paths)
        extra = sorted(actual_paths - expected_paths)
        raise RuntimeError(
            "Shader PBR profile manifest does not bind the exact profile tool closure: "
            f"missing={missing} extra={extra} path={manifest_path}"
        )

    for relative_path in PROFILE_TOOL_PATHS:
        tool_path, expected_sha256, expected_byte_length = fingerprints[relative_path]
        if not tool_path.is_file():
            raise RuntimeError(
                f"Shader PBR profile tool is missing: relative_path={relative_path!r} "
                f"path={manifest_path}"
            )
        actual_byte_length = tool_path.stat().st_size
        if actual_byte_length != expected_byte_length:
            raise RuntimeError(
                f"Shader PBR profile tool byte length changed: relative_path={relative_path!r} "
                f"expected={expected_byte_length} actual={actual_byte_length} path={manifest_path}"
            )
        actual_sha256 = _sha256_file(tool_path)
        if actual_sha256 != expected_sha256:
            raise RuntimeError(
                f"Shader PBR profile tool SHA-256 changed: relative_path={relative_path!r} "
                f"expected={expected_sha256} actual={actual_sha256} path={manifest_path}"
            )
