"""Product-oriented build profile loading for the shipping build pipeline.

The runtime's serialized ``RuntimeProfileId`` describes project/runtime
semantics.  This module deliberately owns a separate, build-only identity so
that a shipping product can select a narrow Cargo feature closure without
changing project files or the runtime profile wire contract.
"""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python 3.10 fallback is unsupported.
    tomllib = None  # type: ignore[assignment]


PRODUCT_PROFILE_SCHEMA_VERSION = 1
DEFAULT_TARGET_TRIPLE = "x86_64-pc-windows-msvc"
DEFAULT_PROFILE_FILE_NAME = "product_build_profiles.toml"
RUNTIME_PRODUCT_CARGO_BINARY = "zircon_runtime_product"
SUPPORTED_PRODUCT_KINDS = frozenset({"runtime", "editor"})
SUPPORTED_CARGO_PROFILES = frozenset(
    {"debug", "release", "profiling", "shipping", "shipping-symbols"}
)
SUPPORTED_PLUGIN_CARRIERS = frozenset({"none", "all", "native_dynamic", "rlib_static"})
SUPPORTED_ASSET_SCOPES = frozenset({"none", "runtime", "editor", "all"})
SUPPORTED_SYMBOL_POLICIES = frozenset({"external", "inline"})
_PROFILE_NAME_RE = re.compile(r"^[a-z0-9][a-z0-9._-]*$")
_PACKAGE_NAME_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_-]*$")


class ProductBuildProfileError(ValueError):
    """Raised when a product profile is missing or violates its contract."""


@dataclass(frozen=True)
class ProductBuildProfile:
    """Validated immutable description of one build-only product composition."""

    name: str
    aliases: tuple[str, ...]
    product_kind: str
    target_triple: str
    cargo_profile: str
    app_package: str
    app_binary: str
    runtime_package: str
    runtime_library: str
    runtime_features: tuple[str, ...]
    app_features: tuple[str, ...]
    plugin_carrier: str
    asset_scope: str
    include_project_pack: bool
    symbols: str
    warning_bytes: int
    hard_limit_bytes: int
    hard_gate: bool
    archive: bool
    forbidden_path_tokens: tuple[str, ...]

    @property
    def is_runtime(self) -> bool:
        return self.product_kind == "runtime"

    @property
    def is_editor(self) -> bool:
        return self.product_kind == "editor"

    @property
    def cargo_app_binary(self) -> str:
        """Return the Cargo bin target used to produce the published host.

        The development ``zircon_runtime`` bin intentionally keeps its
        ``target-client`` required-feature contract.  Runtime products use a
        second target with the same source and the narrow ``runtime-product``
        feature gate, then publish it under ``app_binary``.
        """

        return RUNTIME_PRODUCT_CARGO_BINARY if self.is_runtime else self.app_binary

    @property
    def build_set_id(self) -> str:
        """Return a stable identity for the profile's build inputs.

        The digest intentionally excludes machine-local paths and timestamps.
        The selected plugin/feature overrides are supplied by
        :func:`build_set_id_for` when a concrete invocation is known.
        """

        return build_set_id_for(self)

    def normalized(self) -> dict[str, object]:
        """Return the canonical, JSON-safe representation used for hashing/reporting."""

        return {
            "schema_version": PRODUCT_PROFILE_SCHEMA_VERSION,
            "name": self.name,
            "aliases": sorted(self.aliases),
            "product_kind": self.product_kind,
            "target_triple": self.target_triple,
            "cargo_profile": self.cargo_profile,
            "app_package": self.app_package,
            "app_binary": self.app_binary,
            "cargo_app_binary": self.cargo_app_binary,
            "runtime_package": self.runtime_package,
            "runtime_library": self.runtime_library,
            "runtime_features": sorted(self.runtime_features),
            "app_features": sorted(self.app_features),
            "plugin_carrier": self.plugin_carrier,
            "asset_scope": self.asset_scope,
            "include_project_pack": self.include_project_pack,
            "symbols": self.symbols,
            "warning_bytes": self.warning_bytes,
            "hard_limit_bytes": self.hard_limit_bytes,
            "hard_gate": self.hard_gate,
            "archive": self.archive,
            "forbidden_path_tokens": sorted(self.forbidden_path_tokens),
        }


@dataclass(frozen=True)
class ProductBuildProfileSet:
    """The validated profile file plus its lookup aliases."""

    path: Path
    schema_version: int
    default_target_triple: str
    profiles: tuple[ProductBuildProfile, ...]

    def by_name(self, name: str) -> ProductBuildProfile:
        requested = _normalize_name(name, "product profile")
        for profile in self.profiles:
            if profile.name == requested or requested in profile.aliases:
                return profile
        known = ", ".join(profile.name for profile in self.profiles)
        raise ProductBuildProfileError(
            f"unknown product profile {name!r}; known profiles: {known}"
        )


def default_product_profile_path(repo_root: Path | None = None) -> Path:
    root = Path(repo_root) if repo_root is not None else Path(__file__).resolve().parents[2]
    return root / "tools" / "export" / DEFAULT_PROFILE_FILE_NAME


def load_product_build_profiles(path: Path | str | None = None) -> ProductBuildProfileSet:
    """Load and validate the build-only TOML profile set."""

    profile_path = Path(path) if path is not None else default_product_profile_path()
    profile_path = profile_path.expanduser().resolve()
    if tomllib is None:  # pragma: no cover
        raise ProductBuildProfileError("Python 3.11 or newer is required to parse TOML")
    try:
        with profile_path.open("rb") as handle:
            document = tomllib.load(handle)
    except OSError as error:
        raise ProductBuildProfileError(
            f"could not read product profile file {profile_path}: {error}"
        ) from error
    except tomllib.TOMLDecodeError as error:
        raise ProductBuildProfileError(
            f"product profile file {profile_path} is invalid TOML: {error}"
        ) from error
    return _parse_profile_set(profile_path, document)


def load_product_build_profile(
    name: str,
    *,
    path: Path | str | None = None,
) -> ProductBuildProfile:
    """Resolve a profile by canonical name or alias."""

    return load_product_build_profiles(path).by_name(name)


def build_set_id_for(
    profile: ProductBuildProfile,
    *,
    selected_plugins: tuple[str, ...] | list[str] = (),
    feature_overrides: tuple[str, ...] | list[str] = (),
) -> str:
    """Hash the profile and invocation-level selections into a Build Set id."""

    payload = {
        "profile": profile.normalized(),
        "selected_plugins": sorted({_normalize_token(value, "plugin id") for value in selected_plugins}),
        "feature_overrides": sorted(
            {_normalize_token(value, "feature override") for value in feature_overrides}
        ),
    }
    encoded = json.dumps(
        payload,
        ensure_ascii=True,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def profile_summary(profile: ProductBuildProfile) -> dict[str, object]:
    """Return a stable summary suitable for manifests and human diagnostics."""

    summary = profile.normalized()
    summary["build_set_id"] = profile.build_set_id
    return summary


def _parse_profile_set(path: Path, document: Mapping[str, Any]) -> ProductBuildProfileSet:
    if not isinstance(document, Mapping):
        raise ProductBuildProfileError(f"product profile file {path} must contain a table")
    _reject_unknown(document, {"schema_version", "default_target_triple", "profiles"}, "root")
    schema_version = _required_int(document, "schema_version", "root")
    if schema_version != PRODUCT_PROFILE_SCHEMA_VERSION:
        raise ProductBuildProfileError(
            f"product profile file {path} schema_version must be {PRODUCT_PROFILE_SCHEMA_VERSION}"
        )
    default_target = _required_string(document, "default_target_triple", "root")
    if default_target != DEFAULT_TARGET_TRIPLE:
        raise ProductBuildProfileError(
            f"product profiles currently require target {DEFAULT_TARGET_TRIPLE}"
        )
    profiles_table = document.get("profiles")
    if not isinstance(profiles_table, Mapping) or not profiles_table:
        raise ProductBuildProfileError("product profile file profiles must be a non-empty table")
    profiles: list[ProductBuildProfile] = []
    aliases: dict[str, str] = {}
    for raw_name, raw_profile in profiles_table.items():
        name = _normalize_name(raw_name, "profile name")
        profile = _parse_profile(name, raw_profile)
        if profile.target_triple != default_target:
            raise ProductBuildProfileError(
                f"profile {name} target_triple must match root default_target_triple {default_target}"
            )
        if name in aliases:
            raise ProductBuildProfileError(f"duplicate product profile name or alias {name}")
        aliases[name] = name
        for alias in profile.aliases:
            if alias in aliases:
                raise ProductBuildProfileError(
                    f"duplicate product profile name or alias {alias}"
                )
            aliases[alias] = name
        profiles.append(profile)
    return ProductBuildProfileSet(
        path=path,
        schema_version=schema_version,
        default_target_triple=default_target,
        profiles=tuple(sorted(profiles, key=lambda item: item.name)),
    )


def _parse_profile(name: str, raw_profile: object) -> ProductBuildProfile:
    if not isinstance(raw_profile, Mapping):
        raise ProductBuildProfileError(f"profile {name} must be a table")
    required = {
        "product_kind",
        "target_triple",
        "cargo_profile",
        "app_package",
        "app_binary",
        "runtime_package",
        "runtime_library",
        "runtime_features",
        "app_features",
        "plugin_carrier",
        "asset_scope",
        "include_project_pack",
        "symbols",
        "warning_bytes",
        "hard_limit_bytes",
        "hard_gate",
        "archive",
        "forbidden_path_tokens",
    }
    optional = {"aliases"}
    _reject_unknown(raw_profile, required | optional, f"profile {name}")
    product_kind = _required_string(raw_profile, "product_kind", name)
    if product_kind not in SUPPORTED_PRODUCT_KINDS:
        raise ProductBuildProfileError(
            f"profile {name} product_kind must be one of {sorted(SUPPORTED_PRODUCT_KINDS)}"
        )
    target_triple = _required_string(raw_profile, "target_triple", name)
    if target_triple != DEFAULT_TARGET_TRIPLE:
        raise ProductBuildProfileError(
            f"profile {name} only supports target {DEFAULT_TARGET_TRIPLE}"
        )
    cargo_profile = _required_string(raw_profile, "cargo_profile", name)
    if cargo_profile not in SUPPORTED_CARGO_PROFILES:
        raise ProductBuildProfileError(
            f"profile {name} cargo_profile must be one of {sorted(SUPPORTED_CARGO_PROFILES)}"
        )
    aliases = _string_array(raw_profile.get("aliases", []), f"profile {name} aliases")
    aliases = tuple(_normalize_name(value, f"profile {name} alias") for value in aliases)
    if name in aliases:
        raise ProductBuildProfileError(f"profile {name} cannot alias itself")
    app_package = _package_name(raw_profile, "app_package", name)
    app_binary = _relative_artifact_name(raw_profile, "app_binary", name)
    runtime_package = _package_name(raw_profile, "runtime_package", name)
    runtime_library = _relative_artifact_name(raw_profile, "runtime_library", name)
    runtime_features = tuple(
        _normalize_token(value, f"profile {name} runtime feature")
        for value in _string_array(
            raw_profile.get("runtime_features"), f"profile {name} runtime_features"
        )
    )
    app_features = tuple(
        _normalize_token(value, f"profile {name} app feature")
        for value in _string_array(
            raw_profile.get("app_features"), f"profile {name} app_features"
        )
    )
    if not runtime_features or not app_features:
        raise ProductBuildProfileError(
            f"profile {name} runtime_features and app_features must not be empty"
        )
    plugin_carrier = _required_string(raw_profile, "plugin_carrier", name)
    if plugin_carrier not in SUPPORTED_PLUGIN_CARRIERS:
        raise ProductBuildProfileError(
            f"profile {name} plugin_carrier must be one of {sorted(SUPPORTED_PLUGIN_CARRIERS)}"
        )
    asset_scope = _required_string(raw_profile, "asset_scope", name)
    if asset_scope not in SUPPORTED_ASSET_SCOPES:
        raise ProductBuildProfileError(
            f"profile {name} asset_scope must be one of {sorted(SUPPORTED_ASSET_SCOPES)}"
        )
    symbols = _required_string(raw_profile, "symbols", name)
    if symbols not in SUPPORTED_SYMBOL_POLICIES:
        raise ProductBuildProfileError(
            f"profile {name} symbols must be one of {sorted(SUPPORTED_SYMBOL_POLICIES)}"
        )
    warning_bytes = _required_positive_int(raw_profile, "warning_bytes", name)
    hard_limit_bytes = _required_positive_int(raw_profile, "hard_limit_bytes", name)
    if warning_bytes <= hard_limit_bytes:
        raise ProductBuildProfileError(
            f"profile {name} warning_bytes must be greater than hard_limit_bytes"
        )
    # The warning line is intentionally above the final hard gate (300/200 MB
    # for Runtime and 500/400 MB for Editor): it gives a stabilization window
    # before the strict limit is enabled.
    include_project_pack = _required_bool(raw_profile, "include_project_pack", name)
    hard_gate = _required_bool(raw_profile, "hard_gate", name)
    archive = _required_bool(raw_profile, "archive", name)
    forbidden = tuple(
        _path_token(value, f"profile {name} forbidden path token")
        for value in _string_array(
            raw_profile.get("forbidden_path_tokens"),
            f"profile {name} forbidden_path_tokens",
        )
    )
    if not forbidden:
        raise ProductBuildProfileError(
            f"profile {name} forbidden_path_tokens must not be empty"
        )
    # Shipping products must never silently carry development-only features.
    all_features = runtime_features + app_features
    if any("dev-dynamic" in feature or feature in {"debug", "profiling"} for feature in all_features):
        raise ProductBuildProfileError(
            f"profile {name} contains a development-only feature"
        )
    return ProductBuildProfile(
        name=name,
        aliases=aliases,
        product_kind=product_kind,
        target_triple=target_triple,
        cargo_profile=cargo_profile,
        app_package=app_package,
        app_binary=app_binary,
        runtime_package=runtime_package,
        runtime_library=runtime_library,
        runtime_features=tuple(dict.fromkeys(runtime_features)),
        app_features=tuple(dict.fromkeys(app_features)),
        plugin_carrier=plugin_carrier,
        asset_scope=asset_scope,
        include_project_pack=include_project_pack,
        symbols=symbols,
        warning_bytes=warning_bytes,
        hard_limit_bytes=hard_limit_bytes,
        hard_gate=hard_gate,
        archive=archive,
        forbidden_path_tokens=tuple(dict.fromkeys(forbidden)),
    )


def _reject_unknown(mapping: Mapping[str, Any], allowed: set[str], label: str) -> None:
    unknown = sorted(set(mapping) - allowed)
    if unknown:
        raise ProductBuildProfileError(
            f"{label} has unknown field(s): {', '.join(str(value) for value in unknown)}"
        )


def _required_string(mapping: Mapping[str, Any], key: str, label: str) -> str:
    value = mapping.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ProductBuildProfileError(f"{label} {key} must be a non-empty string")
    return value.strip()


def _normalize_name(value: object, label: str) -> str:
    if not isinstance(value, str):
        raise ProductBuildProfileError(f"{label} must be a string")
    normalized = value.strip().lower()
    if not normalized or not _PROFILE_NAME_RE.fullmatch(normalized):
        raise ProductBuildProfileError(
            f"{label} must match {_PROFILE_NAME_RE.pattern!r}"
        )
    return normalized


def _normalize_token(value: object, label: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise ProductBuildProfileError(f"{label} must be a non-empty string")
    return value.strip()


def _package_name(mapping: Mapping[str, Any], key: str, label: str) -> str:
    value = _required_string(mapping, key, label)
    if not _PACKAGE_NAME_RE.fullmatch(value):
        raise ProductBuildProfileError(
            f"{label} {key} must be a plain Cargo package name"
        )
    return value


def _relative_artifact_name(
    mapping: Mapping[str, Any], key: str, label: str
) -> str:
    value = _required_string(mapping, key, label).replace("\\", "/")
    if (
        value.startswith("/")
        or ":" in value
        or any(ord(character) < 0x20 for character in value)
    ):
        raise ProductBuildProfileError(
            f"{label} {key} must be a relative portable artifact path"
        )
    raw_components = value.split("/")
    components = [component for component in raw_components if component]
    if (
        not components
        or len(components) != len(raw_components)
        or any(component in {".", ".."} for component in components)
    ):
        raise ProductBuildProfileError(
            f"{label} {key} must not contain empty, current, or parent path segments"
        )
    return "/".join(components)


def _path_token(value: object, label: str) -> str:
    token = _normalize_token(value, label).replace("\\", "/")
    if (
        token.startswith("/")
        or ":" in token
        or any(ord(character) < 0x20 for character in token)
    ):
        raise ProductBuildProfileError(f"{label} must be a relative portable path token")
    raw_components = token.split("/")
    components = [component for component in raw_components if component]
    if (
        not components
        or len(components) != len(raw_components)
        or any(component in {".", ".."} for component in components)
    ):
        raise ProductBuildProfileError(
            f"{label} must not contain empty, current, or parent path segments"
        )
    return "/".join(components)


def _string_array(value: object, label: str) -> list[str]:
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        raise ProductBuildProfileError(f"{label} must be a string array")
    normalized = [item.strip() for item in value]
    if any(not item for item in normalized):
        raise ProductBuildProfileError(f"{label} must not contain blank entries")
    return normalized


def _required_int(mapping: Mapping[str, Any], key: str, label: str) -> int:
    value = mapping.get(key)
    if type(value) is not int:
        raise ProductBuildProfileError(f"{label} {key} must be an integer")
    return value


def _required_positive_int(mapping: Mapping[str, Any], key: str, label: str) -> int:
    value = _required_int(mapping, key, label)
    if value <= 0:
        raise ProductBuildProfileError(f"{label} {key} must be greater than zero")
    return value


def _required_bool(mapping: Mapping[str, Any], key: str, label: str) -> bool:
    value = mapping.get(key)
    if type(value) is not bool:
        raise ProductBuildProfileError(f"{label} {key} must be a boolean")
    return value
