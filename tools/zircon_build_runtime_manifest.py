"""Generate the lockstep identity sidecar for a staged internal Runtime DLL."""

from __future__ import annotations

import hashlib
import json
import os
import platform
import struct
import sys
from pathlib import Path


RUNTIME_ARTIFACT_MANIFEST_SCHEMA_VERSION = 1
RUNTIME_INTERFACE_SPEC_RELATIVE_PATH = (
    Path("zircon_runtime_interface")
    / "src"
    / "runtime_build_set"
    / "interface_spec_v1.json"
)
RUNTIME_PAYLOAD_SCHEMA_SET_RELATIVE_PATH = (
    Path("zircon_runtime_interface")
    / "src"
    / "runtime_build_set"
    / "payload_schema_set_v1.json"
)
INTERFACE_SPEC_KEY_ORDER = (
    "family",
    "spec_version",
    "runtime_api_version",
    "entry_symbol",
    "runtime_api_required_slots",
    "runtime_api_optional_slots",
    "host_api_optional_slots",
)


def write_runtime_artifact_manifest(config: object) -> Path:
    """Bind the staged DLL and every staged host executable to one BuildSet."""

    library_path = Path(config.engine_root) / runtime_library_file_name()
    manifest_path = runtime_artifact_manifest_path(library_path)
    if getattr(config, "dry_run", False):
        print(f"DRY-RUN write {manifest_path}")
        return manifest_path
    if not library_path.is_file():
        raise SystemExit(
            f"Cannot write runtime artifact manifest: staged runtime library is missing: {library_path}"
        )
    host_artifacts = _host_artifacts(Path(config.engine_root))
    if not host_artifacts:
        raise SystemExit(
            "Cannot write runtime artifact manifest: staged runtime library requires at least one host executable."
        )

    interface_spec = _load_interface_spec(Path(config.repo_root))
    interface_spec_digest = _sha256_json(interface_spec)
    target = _target_model()
    artifact = _artifact_identity(library_path)
    payload_schema_digest = _load_payload_schema_set_digest(Path(config.repo_root))
    runtime_features = sorted({str(feature) for feature in config.runtime_features})
    build_set_id = _sha256_json(
        {
            "artifact": artifact,
            "build_mode": str(config.mode),
            "capabilities": [],
            "host_artifacts": host_artifacts,
            "interface_spec_digest": interface_spec_digest,
            "payload_schema_digest": payload_schema_digest,
            "runtime_features": runtime_features,
            "target": target,
        }
    )
    payload = {
        "schema_version": RUNTIME_ARTIFACT_MANIFEST_SCHEMA_VERSION,
        "build_set_id": build_set_id,
        "build_mode": str(config.mode),
        "runtime_features": runtime_features,
        "interface_spec_digest": interface_spec_digest,
        "interface_spec": interface_spec,
        "payload_schema_digest": payload_schema_digest,
        "target": target,
        "artifact": artifact,
        "host_artifacts": host_artifacts,
        "capabilities": [],
    }
    _write_json_atomically(manifest_path, payload)
    print(f"Wrote {manifest_path}")
    return manifest_path


def runtime_artifact_manifest_path(library_path: Path) -> Path:
    return library_path.with_name(f"{library_path.name}.manifest.json")


def runtime_library_file_name() -> str:
    if os.name == "nt":
        return "zircon_runtime.dll"
    if platform.system().lower() == "darwin":
        return "libzircon_runtime.dylib"
    return "libzircon_runtime.so"


def runtime_host_file_names() -> tuple[str, ...]:
    suffix = ".exe" if os.name == "nt" else ""
    return (f"zircon_editor{suffix}", f"zircon_runtime{suffix}")


def _host_artifacts(engine_root: Path) -> list[dict[str, str]]:
    return [
        _artifact_identity(engine_root / name)
        for name in runtime_host_file_names()
        if (engine_root / name).is_file()
    ]


def _artifact_identity(path: Path) -> dict[str, str]:
    return {"file_name": path.name, "sha256": _file_sha256(path)}


def _load_interface_spec(repo_root: Path) -> dict[str, object]:
    path = repo_root / RUNTIME_INTERFACE_SPEC_RELATIVE_PATH
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise SystemExit(f"Cannot read Runtime InterfaceSpec {path}: {error}") from error
    except json.JSONDecodeError as error:
        raise SystemExit(f"Runtime InterfaceSpec {path} is invalid JSON: {error}") from error
    if not isinstance(value, dict):
        raise SystemExit(f"Runtime InterfaceSpec {path} must be a JSON object.")
    expected_keys = set(INTERFACE_SPEC_KEY_ORDER)
    if set(value) != expected_keys:
        raise SystemExit(
            f"Runtime InterfaceSpec {path} must contain exactly {sorted(expected_keys)}."
        )
    # Match serde's declaration order, regardless of source-file formatting.
    return {key: value[key] for key in INTERFACE_SPEC_KEY_ORDER}


def _load_payload_schema_set_digest(repo_root: Path) -> str:
    path = repo_root / RUNTIME_PAYLOAD_SCHEMA_SET_RELATIVE_PATH
    try:
        source = path.read_bytes()
    except OSError as error:
        raise SystemExit(f"Cannot read Runtime payload schema set {path}: {error}") from error
    try:
        value = json.loads(source)
    except json.JSONDecodeError as error:
        raise SystemExit(
            f"Runtime payload schema set {path} is invalid JSON: {error}"
        ) from error
    expected_keys = {
        "family",
        "spec_version",
        "encoding",
        "serialization",
        "schema_status",
    }
    if not isinstance(value, dict) or set(value) != expected_keys:
        raise SystemExit(
            f"Runtime payload schema set {path} must contain exactly {sorted(expected_keys)}."
        )
    return hashlib.sha256(source).hexdigest()


def _target_model() -> dict[str, object]:
    architecture = platform.machine().lower()
    architecture = {
        "amd64": "x86_64",
        "x64": "x86_64",
        "arm64": "aarch64",
    }.get(architecture, architecture)
    operating_system = {
        "darwin": "macos",
        "win32": "windows",
    }.get(sys.platform, sys.platform)
    return {
        "architecture": architecture,
        "operating_system": operating_system,
        "pointer_width": struct.calcsize("P") * 8,
        "endian": sys.byteorder,
    }


def _sha256_json(value: object) -> str:
    return hashlib.sha256(_canonical_json(value)).hexdigest()


def _canonical_json(value: object) -> bytes:
    return json.dumps(value, separators=(",", ":"), ensure_ascii=True).encode("utf-8")


def _file_sha256(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def _write_json_atomically(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary_path = path.with_suffix(path.suffix + ".tmp")
    temporary_path.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    os.replace(temporary_path, path)
