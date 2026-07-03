"""Validate report LibraryEmbed CompileHost identity semantic diagnostics."""

from __future__ import annotations

from typing import Any

VALIDATE_LIBRARY_EMBED_COMPILE_HOST_CARGO_PROFILES = {"debug", "release"}
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_PACKAGES = ("zircon_app",)
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_BINARIES = ("zircon_runtime", "zircon_editor")


def library_embed_compile_host_profile_release_diagnostics(
    value: dict[str, Any],
) -> list[str]:
    label = "validate report plan_summary.library_embed_compile_host"
    cargo_profile = value.get("cargo_profile")
    release = value.get("release")
    diagnostics: list[str] = []
    if not compile_host_cargo_profile_is_schema_clean(cargo_profile):
        return diagnostics
    if cargo_profile not in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_CARGO_PROFILES:
        diagnostics.append(f"{label}.cargo_profile must be debug or release")
        return diagnostics

    if isinstance(release, bool) and release != (cargo_profile == "release"):
        diagnostics.append(f"{label}.release must match cargo_profile")
    return diagnostics


def compile_host_cargo_profile_is_schema_clean(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value == value.strip()


def compile_host_target_selector_schema_diagnostics(
    value: dict[str, Any],
    *,
    package_label: str,
    binary_label: str,
) -> list[str]:
    diagnostics: list[str] = []
    package = value.get("package")
    if (
        isinstance(package, str)
        and package.strip()
        and package == package.strip()
        and package not in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_PACKAGES
    ):
        diagnostics.append(f"{package_label} must be zircon_app")
    binary = value.get("binary")
    if (
        isinstance(binary, str)
        and binary.strip()
        and binary == binary.strip()
        and binary not in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_BINARIES
    ):
        diagnostics.append(f"{binary_label} must be zircon_runtime or zircon_editor")
    return diagnostics
