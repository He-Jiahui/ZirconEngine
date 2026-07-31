"""Explicit Validate contents-artifact loading for SourceTemplate."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path
from typing import Any


CONTENTS_ARTIFACT_SCHEMA_VERSION = 1
LOWERCASE_SHA256_RE = re.compile(r"[0-9a-f]{64}")


def load_source_template_contents_artifact(
    validate_report_path: Path,
    validate_payload: dict[str, Any],
    diagnostics: list[str],
) -> list[dict[str, str]] | None:
    artifact_path = resolve_contents_artifact_path(
        validate_report_path,
        validate_payload.get("generated_contents_artifact_path"),
        diagnostics,
    )
    expected_byte_length = validate_payload.get(
        "generated_contents_artifact_byte_length"
    )
    digest = validate_payload.get("generated_contents_artifact_digest")
    if type(expected_byte_length) is not int or expected_byte_length < 0:
        diagnostics.append(
            "validate report generated_contents_artifact_byte_length "
            "must be a non-negative integer"
        )
    if not isinstance(digest, str) or LOWERCASE_SHA256_RE.fullmatch(digest) is None:
        diagnostics.append(
            "validate report generated_contents_artifact_digest must be "
            "a 64-character lowercase SHA-256 digest"
        )
    if artifact_path is None or diagnostics:
        return None
    if not artifact_path.exists():
        diagnostics.append(
            f"generated contents artifact {artifact_path} does not exist"
        )
        return None
    if not artifact_path.is_file():
        diagnostics.append(
            f"generated contents artifact {artifact_path} is not a file"
        )
        return None
    try:
        encoded_artifact = artifact_path.read_bytes()
    except OSError as error:
        diagnostics.append(
            f"generated contents artifact {artifact_path} could not be read: {error}"
        )
        return None
    if len(encoded_artifact) != expected_byte_length:
        diagnostics.append(
            f"generated contents artifact {artifact_path} byte length "
            f"{len(encoded_artifact)} does not match Validate report "
            f"{expected_byte_length}"
        )
        return None
    actual_digest = hashlib.sha256(encoded_artifact).hexdigest()
    if actual_digest != digest:
        diagnostics.append(
            "generated contents artifact SHA-256 does not match Validate report"
        )
        return None
    try:
        artifact = json.loads(encoded_artifact.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        diagnostics.append(
            f"generated contents artifact {artifact_path} is not valid UTF-8 JSON: {error}"
        )
        return None
    artifact_files = contents_artifact_schema_files(artifact, diagnostics)
    if artifact_files is None:
        return None
    compact_files = compact_generated_files(validate_payload)
    if compact_files is None:
        diagnostics.append(
            "validate report does not contain compact generated file metadata"
        )
        return None
    diagnostics.extend(
        contents_artifact_handoff_diagnostics(compact_files, artifact_files)
    )
    return artifact_files if not diagnostics else None


def resolve_contents_artifact_path(
    validate_report_path: Path,
    value: object,
    diagnostics: list[str],
) -> Path | None:
    if not isinstance(value, str) or not value.strip():
        diagnostics.append(
            "validate report generated_contents_artifact_path "
            "must be a non-empty string"
        )
        return None
    if value != value.strip():
        diagnostics.append(
            "validate report generated_contents_artifact_path "
            "must be a non-empty trimmed string"
        )
        return None
    candidate = Path(value)
    if not candidate.is_absolute():
        candidate = validate_report_path.parent / candidate
    try:
        return candidate.resolve()
    except OSError as error:
        diagnostics.append(
            f"generated contents artifact {candidate} could not be resolved: {error}"
        )
        return None


def contents_artifact_schema_files(
    artifact: object,
    diagnostics: list[str],
) -> list[dict[str, str]] | None:
    if not isinstance(artifact, dict):
        diagnostics.append("generated contents artifact must be a JSON object")
        return None
    unknown_fields = sorted(set(artifact) - {"schema_version", "generated_files"})
    diagnostics.extend(
        f"generated contents artifact unknown field {field}"
        for field in unknown_fields
    )
    schema_version = artifact.get("schema_version")
    if (
        type(schema_version) is not int
        or schema_version != CONTENTS_ARTIFACT_SCHEMA_VERSION
    ):
        diagnostics.append("generated contents artifact schema_version must be 1")
    files = artifact.get("generated_files")
    if not isinstance(files, list):
        diagnostics.append("generated contents artifact generated_files must be a list")
        return None
    normalized_files: list[dict[str, str]] = []
    seen_paths: set[str] = set()
    for index, file in enumerate(files):
        if not isinstance(file, dict):
            diagnostics.append(
                f"generated contents artifact generated_files[{index}] must be an object"
            )
            continue
        diagnostics.extend(
            f"generated contents artifact generated_files[{index}] unknown field {field}"
            for field in sorted(set(file) - {"path", "purpose", "contents"})
        )
        path = file.get("path")
        purpose = file.get("purpose")
        contents = file.get("contents")
        if not isinstance(path, str) or not path.strip() or path != path.strip():
            diagnostics.append(
                f"generated contents artifact generated_files[{index}].path "
                "must be a non-empty trimmed string"
            )
        if (
            not isinstance(purpose, str)
            or not purpose.strip()
            or purpose != purpose.strip()
        ):
            diagnostics.append(
                f"generated contents artifact generated_files[{index}].purpose "
                "must be a non-empty trimmed string"
            )
        if not isinstance(contents, str):
            diagnostics.append(
                f"generated contents artifact generated_files[{index}].contents "
                "must be a string"
            )
        if not all(isinstance(value, str) for value in (path, purpose, contents)):
            continue
        if path in seen_paths:
            diagnostics.append(
                f"generated contents artifact generated file path {path} is duplicated"
            )
        seen_paths.add(path)
        normalized_files.append(
            {"path": path, "purpose": purpose, "contents": contents}
        )
    return normalized_files if not diagnostics else None


def compact_generated_files(
    validate_payload: dict[str, Any],
) -> list[dict[str, Any]] | None:
    plan_summary = validate_payload.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return None
    files = plan_summary.get("generated_files")
    if not isinstance(files, list) or any(not isinstance(file, dict) for file in files):
        return None
    return files


def contents_artifact_handoff_diagnostics(
    compact_files: list[dict[str, Any]],
    artifact_files: list[dict[str, str]],
) -> list[str]:
    if len(compact_files) != len(artifact_files):
        return [
            "generated contents artifact file count does not match compact Validate metadata"
        ]
    diagnostics: list[str] = []
    for index, (compact, artifact) in enumerate(zip(compact_files, artifact_files)):
        path = artifact["path"]
        if compact.get("path") != path or compact.get("purpose") != artifact["purpose"]:
            diagnostics.append(
                f"generated contents artifact row {index} path/purpose does not match "
                "compact Validate metadata"
            )
            continue
        actual_byte_length = len(artifact["contents"].encode("utf-8"))
        if compact.get("byte_length") != actual_byte_length:
            diagnostics.append(
                f"generated contents artifact byte length for {path} does not match "
                "compact Validate metadata"
            )
        content_digest = compact.get("content_digest")
        if (
            not isinstance(content_digest, str)
            or LOWERCASE_SHA256_RE.fullmatch(content_digest) is None
        ):
            diagnostics.append(
                f"compact Validate content_digest for {path} must be a "
                "64-character lowercase SHA-256 digest"
            )
        elif (
            hashlib.sha256(artifact["contents"].encode("utf-8")).hexdigest()
            != content_digest
        ):
            diagnostics.append(
                f"generated contents artifact SHA-256 for {path} does not match "
                "compact Validate metadata"
            )
    return diagnostics
