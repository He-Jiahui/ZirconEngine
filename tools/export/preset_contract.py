"""Versioned .zpreset projection used by the staged export CLI."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any
from uuid import UUID


PRESET_SCHEMA_ID = "zircon.export-preset"
PRESET_SCHEMA_VERSION = 0
PAYLOAD_FIELDS = {
    "profile_ref",
    "target_mode",
    "debug",
    "include_filter",
    "exclude_filter",
    "entry_scenes",
    "keep_list",
    "plugin_subset",
    "cook",
    "customized_files",
}


def load_export_preset(path: str) -> dict[str, Any]:
    preset_path = Path(path).expanduser().resolve()
    try:
        document = json.loads(preset_path.read_text(encoding="utf-8"))
    except OSError as error:
        raise ValueError(f"failed to read export preset {preset_path}: {error}") from error
    except json.JSONDecodeError as error:
        raise ValueError(f"failed to decode export preset {preset_path}: {error}") from error

    if not isinstance(document, dict) or set(document) != {"$zircon"}:
        raise ValueError(f"export preset {preset_path} must contain only the $zircon envelope")
    envelope = document["$zircon"]
    if not isinstance(envelope, dict) or set(envelope) != {"header", "payload"}:
        raise ValueError(f"export preset {preset_path} has an invalid $zircon envelope")
    header = envelope["header"]
    if not isinstance(header, dict) or set(header) != {"schema_id", "schema_version"}:
        raise ValueError(f"export preset {preset_path} has an invalid header")
    if header.get("schema_id") != PRESET_SCHEMA_ID:
        raise ValueError(
            f"export preset {preset_path} schema_id must be {PRESET_SCHEMA_ID}"
        )
    if (
        type(header.get("schema_version")) is not int
        or header.get("schema_version") != PRESET_SCHEMA_VERSION
    ):
        raise ValueError(
            f"export preset {preset_path} schema_version must be {PRESET_SCHEMA_VERSION}"
        )
    payload = envelope["payload"]
    if not isinstance(payload, dict):
        raise ValueError(f"export preset {preset_path} payload must be an object")
    unknown = set(payload) - PAYLOAD_FIELDS
    if unknown:
        raise ValueError(
            f"export preset {preset_path} payload has unknown fields: {', '.join(sorted(unknown))}"
        )
    profile_ref = payload.get("profile_ref")
    if not isinstance(profile_ref, str) or not profile_ref.strip():
        raise ValueError(f"export preset {preset_path} profile_ref must be non-empty")
    target_mode = payload.get("target_mode")
    if target_mode not in {"client_runtime", "server_runtime"}:
        raise ValueError(
            f"export preset {preset_path} target_mode must be client_runtime or server_runtime"
        )

    normalized = {
        "profile_ref": profile_ref,
        "target_mode": target_mode,
        "debug": _boolean(payload, "debug", False, preset_path),
        "include_filter": _string(payload, "include_filter", "", preset_path),
        "exclude_filter": _string(payload, "exclude_filter", "", preset_path),
        "entry_scenes": _asset_refs(payload, "entry_scenes", preset_path),
        "keep_list": _asset_refs(payload, "keep_list", preset_path),
        "plugin_subset": _plugin_subset(payload.get("plugin_subset"), preset_path),
        "cook": _cook(payload.get("cook"), preset_path),
        "customized_files": _customized_files(
            payload.get("customized_files"), preset_path
        ),
    }
    return normalized


def _boolean(payload: dict[str, Any], key: str, default: bool, path: Path) -> bool:
    value = payload.get(key, default)
    if not isinstance(value, bool):
        raise ValueError(f"export preset {path} {key} must be a boolean")
    return value


def _string(payload: dict[str, Any], key: str, default: str, path: Path) -> str:
    value = payload.get(key, default)
    if not isinstance(value, str):
        raise ValueError(f"export preset {path} {key} must be a string")
    return value


def _asset_refs(payload: dict[str, Any], key: str, path: Path) -> list[dict[str, Any]]:
    values = payload.get(key, [])
    if not isinstance(values, list):
        raise ValueError(f"export preset {path} {key} must be an array")
    for index, value in enumerate(values):
        if not isinstance(value, dict) or set(value) != {"guid", "path_hint", "sub"}:
            raise ValueError(
                f"export preset {path} {key}[{index}] must contain guid, path_hint, and sub"
            )
        try:
            UUID(value["guid"])
        except (TypeError, ValueError, AttributeError) as error:
            raise ValueError(
                f"export preset {path} {key}[{index}].guid must be a UUID"
            ) from error
        value["path_hint"] = _rel_path(
            value["path_hint"], f"{key}[{index}].path_hint", path
        )
        if value["sub"] is not None and not isinstance(value["sub"], str):
            raise ValueError(
                f"export preset {path} {key}[{index}].sub must be a string or null"
            )
        if isinstance(value["sub"], str) and (
            not value["sub"]
            or "#" in value["sub"]
            or any(ord(character) < 32 or 127 <= ord(character) <= 159 for character in value["sub"])
        ):
            raise ValueError(
                f"export preset {path} {key}[{index}].sub is not a valid subasset path"
            )
    return values


def _plugin_subset(value: Any, path: Path) -> dict[str, Any] | None:
    if value is None:
        return None
    if not isinstance(value, dict) or set(value) - {"package_ids", "features"}:
        raise ValueError(f"export preset {path} plugin_subset has invalid fields")
    package_ids = value.get("package_ids", [])
    features = value.get("features", {})
    if (
        not isinstance(package_ids, list)
        or any(not isinstance(item, str) or not item.strip() for item in package_ids)
        or len(set(package_ids)) != len(package_ids)
    ):
        raise ValueError(
            f"export preset {path} plugin_subset.package_ids must contain unique non-empty strings"
        )
    if not isinstance(features, dict) or any(
        not isinstance(package_id, str)
        or not isinstance(items, list)
        or any(not isinstance(item, str) for item in items)
        for package_id, items in features.items()
    ):
        raise ValueError(
            f"export preset {path} plugin_subset.features must map strings to string arrays"
        )
    return {"package_ids": package_ids, "features": features}


def _cook(value: Any, path: Path) -> dict[str, Any]:
    if value is None:
        value = {}
    if not isinstance(value, dict) or set(value) - {
        "deterministic",
        "binary_assets",
        "compression",
    }:
        raise ValueError(f"export preset {path} cook has invalid fields")
    deterministic = value.get("deterministic", True)
    binary_assets = value.get("binary_assets", True)
    compression = value.get("compression", "zstd")
    if not isinstance(deterministic, bool) or not isinstance(binary_assets, bool):
        raise ValueError(f"export preset {path} cook flags must be booleans")
    if compression not in {"none", "zstd", "lz4"}:
        raise ValueError(f"export preset {path} cook.compression is invalid")
    return {
        "deterministic": deterministic,
        "binary_assets": binary_assets,
        "compression": compression,
    }


def _customized_files(value: Any, path: Path) -> dict[str, str]:
    if value is None:
        return {}
    if not isinstance(value, dict) or any(
        mode not in {"default", "include", "exclude"} for mode in value.values()
    ):
        raise ValueError(
            f"export preset {path} customized_files must map paths to default/include/exclude"
        )
    normalized: dict[str, str] = {}
    for key, mode in value.items():
        normalized_key = _rel_path(key, "customized_files key", path)
        if normalized_key in normalized:
            raise ValueError(
                f"export preset {path} customized_files has duplicate normalized path {normalized_key}"
            )
        normalized[normalized_key] = mode
    return normalized


def _rel_path(value: Any, label: str, path: Path) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError(f"export preset {path} {label} must be a non-empty string")
    portable = value.replace("\\", "/")
    first = portable.split("/", 1)[0]
    if portable.startswith("/") or (
        len(first) >= 2 and first[0].isalpha() and first[1] == ":"
    ):
        raise ValueError(f"export preset {path} {label} must be relative")
    components = [component for component in portable.split("/") if component]
    if not components or any(component in {".", ".."} for component in components):
        raise ValueError(f"export preset {path} {label} has an invalid dot component")
    return "/".join(components)
