"""Validate hash-bound input identity for a PBR viewer Ready-frame sidecar."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path
from typing import Mapping


EVIDENCE_IDENTITY_SCHEMA = "zircon_shader_pbr_viewer_evidence_identity_v1"
EVIDENCE_VALIDATION_POLICY = "zircon_shader_pbr_viewer_ready_frame_v17"

_SHA256_PATTERN = re.compile(r"[0-9a-f]{64}\Z")
_RUN_ID_PATTERN = re.compile(r"[a-z][a-z0-9-]{2,159}\Z")
_SIDE_CAR_IDENTITY_FIELDS = (
    "screenshot_sha256",
    "screenshot_byte_length",
    "evidence_identity_schema",
    "evidence_run_id",
    "evidence_validation_policy",
    "evidence_identity_path",
    "evidence_identity_sha256",
    "evidence_identity_byte_length",
    "viewer_binary_path",
    "viewer_binary_sha256",
    "viewer_binary_byte_length",
    "hdri_sha256",
    "hdri_byte_length",
    "build_provenance_path",
    "build_provenance_sha256",
    "build_provenance_byte_length",
    "source_manifest_sha256",
)
_IDENTITY_MANIFEST_FIELDS = frozenset(
    {
        "schema",
        "run_id",
        "validation_policy",
        "source_manifest_sha256",
        "viewer_binary",
        "hdri",
        "build_provenance",
    }
)
_FILE_FINGERPRINT_FIELDS = frozenset({"path", "sha256", "byte_length"})


def validate_ready_frame_identity(
    metadata: Mapping[str, str],
    *,
    screenshot_path: Path,
    validation_policy: str = EVIDENCE_VALIDATION_POLICY,
) -> None:
    """Fail closed when an identity-bound sidecar cannot bind all producer inputs to bytes."""

    missing = [field for field in _SIDE_CAR_IDENTITY_FIELDS if field not in metadata]
    if missing:
        raise RuntimeError(
            "ready-frame v15 provenance sidecar is missing identity fields: "
            f"{', '.join(missing)} path={screenshot_path}"
        )
    _require_exact(metadata, "evidence_identity_schema", EVIDENCE_IDENTITY_SCHEMA, screenshot_path)
    _require_exact(
        metadata,
        "evidence_validation_policy",
        validation_policy,
        screenshot_path,
    )
    _require_run_id(metadata["evidence_run_id"], screenshot_path)
    _require_sha256(metadata["source_manifest_sha256"], "source_manifest_sha256", screenshot_path)
    _require_file_fingerprint(
        screenshot_path,
        sha256=metadata["screenshot_sha256"],
        byte_length=metadata["screenshot_byte_length"],
        label="Ready-frame PNG",
    )

    identity_path = normalize_evidence_path(metadata["evidence_identity_path"])
    _require_file_fingerprint(
        identity_path,
        sha256=metadata["evidence_identity_sha256"],
        byte_length=metadata["evidence_identity_byte_length"],
        label="evidence identity manifest",
    )
    try:
        identity = json.loads(identity_path.read_text(encoding="utf-8-sig"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RuntimeError(
            f"ready-frame evidence identity manifest is malformed: path={identity_path}"
        ) from error
    if not isinstance(identity, dict) or set(identity) != _IDENTITY_MANIFEST_FIELDS:
        raise RuntimeError(
            f"ready-frame evidence identity manifest has an unexpected schema: path={identity_path}"
        )
    if identity["schema"] != EVIDENCE_IDENTITY_SCHEMA:
        raise RuntimeError(
            f"ready-frame evidence identity manifest schema does not match the sidecar policy: path={identity_path}"
        )
    if identity["validation_policy"] != validation_policy:
        raise RuntimeError(
            f"ready-frame evidence identity validation policy does not match the sidecar: path={identity_path}"
        )
    _require_run_id_value(identity["run_id"], identity_path)
    _require_sha256_value(
        identity["source_manifest_sha256"], "source_manifest_sha256", identity_path
    )
    if identity["run_id"] != metadata["evidence_run_id"]:
        raise RuntimeError(
            f"ready-frame evidence run id does not match identity manifest: path={screenshot_path}"
        )
    if identity["source_manifest_sha256"] != metadata["source_manifest_sha256"]:
        raise RuntimeError(
            f"ready-frame source manifest hash does not match identity manifest: path={screenshot_path}"
        )

    _validate_identity_file(
        identity,
        "viewer_binary",
        metadata,
        screenshot_path,
        require_sidecar_path=True,
    )
    _validate_identity_file(
        identity,
        "hdri",
        metadata,
        screenshot_path,
        require_sidecar_path=False,
    )
    _validate_identity_file(
        identity,
        "build_provenance",
        metadata,
        screenshot_path,
        require_sidecar_path=True,
    )


def _validate_identity_file(
    identity: Mapping[str, object],
    name: str,
    metadata: Mapping[str, str],
    screenshot_path: Path,
    *,
    require_sidecar_path: bool,
) -> None:
    record = identity[name]
    if not isinstance(record, dict) or set(record) != _FILE_FINGERPRINT_FIELDS:
        raise RuntimeError(
            f"ready-frame evidence identity {name} fingerprint has an unexpected schema: "
            f"path={screenshot_path}"
        )
    path_value = record["path"]
    sha256 = record["sha256"]
    byte_length = record["byte_length"]
    if not isinstance(path_value, str) or not path_value.strip() or path_value != path_value.strip():
        raise RuntimeError(
            f"ready-frame evidence identity {name} path is invalid: path={screenshot_path}"
        )
    _require_sha256_value(sha256, f"identity.{name}.sha256", screenshot_path)
    if not isinstance(byte_length, int) or isinstance(byte_length, bool) or byte_length < 0:
        raise RuntimeError(
            f"ready-frame evidence identity {name} byte length is invalid: path={screenshot_path}"
        )
    if (
        require_sidecar_path
        and normalize_evidence_path(metadata[f"{name}_path"])
        != normalize_evidence_path(path_value)
    ):
        raise RuntimeError(
            f"ready-frame {name} path does not match identity manifest: path={screenshot_path}"
        )
    if metadata[f"{name}_sha256"] != sha256 or metadata[f"{name}_byte_length"] != str(byte_length):
        raise RuntimeError(
            f"ready-frame {name} fingerprint does not match identity manifest: path={screenshot_path}"
        )
    _require_file_fingerprint(
        normalize_evidence_path(path_value),
        sha256=sha256,
        byte_length=str(byte_length),
        label=name,
    )


def normalize_evidence_path(value: str) -> Path:
    """Return the cross-process path representation used by the v15 protocol."""

    if value.startswith("\\\\?\\UNC\\"):
        value = "\\\\" + value[len("\\\\?\\UNC\\") :]
    elif value.startswith("\\\\?\\"):
        value = value[len("\\\\?\\") :]
    return Path(value)


def _require_exact(
    metadata: Mapping[str, str], field: str, expected: str, screenshot_path: Path
) -> None:
    if metadata[field] != expected:
        raise RuntimeError(
            f"ready-frame provenance {field} is invalid: expected={expected} path={screenshot_path}"
        )


def _require_run_id(value: str, screenshot_path: Path) -> None:
    _require_run_id_value(value, screenshot_path)


def _require_run_id_value(value: object, path: Path) -> None:
    if not isinstance(value, str) or _RUN_ID_PATTERN.fullmatch(value) is None:
        raise RuntimeError(f"ready-frame evidence run id is invalid: path={path}")


def _require_sha256(value: str, field: str, screenshot_path: Path) -> None:
    _require_sha256_value(value, field, screenshot_path)


def _require_sha256_value(value: object, field: str, path: Path) -> None:
    if not isinstance(value, str) or _SHA256_PATTERN.fullmatch(value) is None:
        raise RuntimeError(f"ready-frame {field} SHA-256 is invalid: path={path}")


def _require_file_fingerprint(
    path: Path, *, sha256: str, byte_length: str, label: str
) -> None:
    _require_sha256(sha256, f"{label}", path)
    if not byte_length.isdecimal():
        raise RuntimeError(f"ready-frame {label} byte length is invalid: path={path}")
    try:
        actual_length = path.stat().st_size
        actual_sha256 = _sha256_file(path)
    except OSError as error:
        raise RuntimeError(f"ready-frame {label} is unavailable: path={path}") from error
    if actual_length != int(byte_length) or actual_sha256 != sha256:
        raise RuntimeError(f"ready-frame {label} fingerprint does not match bytes: path={path}")


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(64 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()
