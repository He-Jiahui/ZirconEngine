"""Generate the lockstep identity sidecar for a staged internal Runtime DLL."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import stat
import struct
import sys
from pathlib import Path
from types import SimpleNamespace
from typing import Mapping


RUNTIME_ARTIFACT_MANIFEST_SCHEMA_VERSION = 1
TRUSTED_HOST_METADATA_SCHEMA_VERSION = 1
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


def _trusted_host_metadata(
    config: object,
    product_profile: object | None,
    product_build_set_id: str | None,
) -> dict[str, object] | None:
    source_build_set_id = getattr(config, "trusted_host_build_set_id", None)
    if source_build_set_id is None:
        return None
    if (
        not isinstance(source_build_set_id, str)
        or len(source_build_set_id) != 64
        or source_build_set_id.lower() != source_build_set_id
        or any(character not in "0123456789abcdef" for character in source_build_set_id)
    ):
        raise SystemExit(
            "trusted_host_build_set_id must be exactly 64 lowercase hexadecimal characters"
        )
    metadata: dict[str, object] = {
        "schema_version": TRUSTED_HOST_METADATA_SCHEMA_VERSION,
        "source_build_set_id": source_build_set_id,
    }
    if product_profile is not None:
        metadata["product_profile"] = product_profile.name
        metadata["product_build_set_id"] = product_build_set_id
    return metadata


def write_runtime_artifact_manifest(
    config: object,
    *,
    product_report_reference: Mapping[str, object] | None = None,
    create_once: bool = False,
) -> Path:
    """Bind the staged DLL and every staged host executable to one BuildSet."""

    library_path = Path(config.engine_root) / runtime_library_file_name()
    manifest_path = runtime_artifact_manifest_path(library_path)
    if getattr(config, "dry_run", False):
        print(f"DRY-RUN write {manifest_path}")
        return manifest_path
    if not library_path.is_file() or _is_reparse_point(library_path):
        raise SystemExit(
            "Cannot write runtime artifact manifest: staged runtime library is "
            f"missing or unsafe: {library_path}"
        )
    host_artifacts = _host_artifacts(Path(config.engine_root), config)
    if not host_artifacts:
        raise SystemExit(
            "Cannot write runtime artifact manifest: staged runtime library requires at least one host executable."
        )

    interface_spec = _load_interface_spec(Path(config.repo_root))
    interface_spec_digest = _sha256_json(interface_spec)
    target = _target_model(config)
    artifact = _artifact_identity(library_path)
    payload_schema_digest = _load_payload_schema_set_digest(Path(config.repo_root))
    runtime_features = sorted({str(feature) for feature in config.runtime_features})
    build_set_identity: dict[str, object] = {
        "artifact": artifact,
        "build_mode": str(config.mode),
        "capabilities": [],
        "host_artifacts": host_artifacts,
        "interface_spec_digest": interface_spec_digest,
        "payload_schema_digest": payload_schema_digest,
        "runtime_features": runtime_features,
        "target": target,
    }
    product_profile = getattr(config, "product_profile", None)
    if product_profile is not None:
        product_build_set_id = getattr(config, "build_set_id", None)
        if not isinstance(product_build_set_id, str) or not product_build_set_id:
            product_build_set_id = product_profile.build_set_id
        build_set_identity["product_profile"] = product_profile.name
        build_set_identity["product_build_set_id"] = product_build_set_id
    build_set_id = _sha256_json(build_set_identity)
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
    if product_profile is not None:
        product_build_set_id = getattr(config, "build_set_id", None)
        if not isinstance(product_build_set_id, str) or not product_build_set_id:
            product_build_set_id = product_profile.build_set_id
        cargo_profile = getattr(
            config, "cargo_profile_name", product_profile.cargo_profile
        )
        if callable(cargo_profile):
            cargo_profile = cargo_profile()
        product_payload: dict[str, object] = {
            "profile": product_profile.name,
            "build_set_id": product_build_set_id,
            "cargo_profile": cargo_profile,
            "target_triple": product_profile.target_triple,
            "app_binary": product_profile.app_binary,
            "cargo_app_binary": product_profile.cargo_app_binary,
            "runtime_library": product_profile.runtime_library,
            "features": {
                "runtime": list(product_profile.runtime_features),
                "app": list(product_profile.app_features),
            },
            "budget": {
                "warning_bytes": product_profile.warning_bytes,
                "hard_limit_bytes": product_profile.hard_limit_bytes,
            },
            "size_report": "../reports/"
            + product_profile.name
            + "/product_size_report.json",
            "symbols": "../symbols/" + product_profile.name,
            "required_exports": ["zircon_runtime_get_api_v8"],
            "ffi_no_unwind_guard": True,
        }
        if product_report_reference is not None:
            product_payload["size_report_reference"] = dict(product_report_reference)
        payload["product"] = product_payload
    trusted_host = _trusted_host_metadata(
        config, product_profile, product_build_set_id if product_profile is not None else None
    )
    if trusted_host is not None:
        payload["trusted_host"] = trusted_host
    if create_once:
        _write_json_new(manifest_path, payload)
    else:
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


def runtime_host_file_names(config: object | None = None) -> tuple[str, ...]:
    suffix = ".exe" if os.name == "nt" else ""
    profile = getattr(config, "product_profile", None) if config is not None else None
    if profile is not None:
        if profile.product_kind == "runtime":
            return (f"zircon_runtime{suffix}",)
        if profile.product_kind == "editor":
            return (f"zircon_editor{suffix}",)
    return (f"zircon_editor{suffix}", f"zircon_runtime{suffix}")


def _host_artifacts(engine_root: Path, config: object | None = None) -> list[dict[str, str]]:
    return [
        _artifact_identity(engine_root / name)
        for name in runtime_host_file_names(config)
        if (engine_root / name).is_file() and not _is_reparse_point(engine_root / name)
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


def _target_model(config: object | None = None) -> dict[str, object]:
    configured_target = getattr(config, "effective_target_triple", None) if config is not None else None
    if callable(configured_target):
        configured_target = configured_target()
    if configured_target:
        target_text = str(configured_target)
        if target_text.startswith("x86_64-pc-windows"):
            return {
                "architecture": "x86_64",
                "operating_system": "windows",
                "pointer_width": 64,
                "endian": "little",
                "target_triple": target_text,
            }
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
    _assert_no_reparse_ancestors(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary_path = path.with_suffix(path.suffix + ".tmp")
    if _is_reparse_point(temporary_path):
        raise SystemExit(
            f"Cannot write runtime artifact manifest through a reparse point: {temporary_path}"
        )
    temporary_path.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    try:
        os.replace(temporary_path, path)
    finally:
        if temporary_path.exists() or _is_reparse_point(temporary_path):
            if _is_reparse_point(temporary_path):
                raise SystemExit(
                    "Cannot remove unsafe runtime artifact manifest temporary path: "
                    f"{temporary_path}"
                )
            temporary_path.unlink()


def _write_json_new(path: Path, payload: dict[str, object]) -> None:
    """Create a private staging sidecar while a directory lease forbids renames."""
    _assert_no_reparse_ancestors(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("x", encoding="utf-8") as destination:
        json.dump(payload, destination, indent=2, sort_keys=True)
        destination.write("\n")
        destination.flush()
        os.fsync(destination.fileno())


def _assert_no_reparse_ancestors(path: Path) -> None:
    current = path if path.is_absolute() else Path.cwd() / path
    while True:
        if _is_reparse_point(current):
            raise SystemExit(
                f"Cannot write runtime artifact manifest through a reparse point: {current}"
            )
        if current.parent == current:
            return
        current = current.parent


def _is_reparse_point(path: Path) -> bool:
    if path.is_symlink():
        return True
    try:
        attributes = os.lstat(path).st_file_attributes
    except (OSError, AttributeError):
        return False
    return bool(attributes & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0x400))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--engine-root", type=Path, required=True)
    parser.add_argument("--mode", required=True)
    parser.add_argument("--runtime-features", required=True)
    parser.add_argument("--create-once", action="store_true")
    args = parser.parse_args()
    features = tuple(feature.strip() for feature in args.runtime_features.split(","))
    if any(not feature for feature in features):
        parser.error("--runtime-features requires non-empty feature names")
    write_runtime_artifact_manifest(
        SimpleNamespace(
            repo_root=args.repo_root,
            engine_root=args.engine_root,
            mode=args.mode,
            runtime_features=features,
            dry_run=False,
        ),
        create_once=args.create_once,
    )


if __name__ == "__main__":
    main()
