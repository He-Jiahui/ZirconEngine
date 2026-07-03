"""NativeDynamic cdylib build-plan discovery for export reports."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_command import (
    native_dynamic_cargo_build_command,
    native_dynamic_cargo_profile,
    native_dynamic_expected_loadable_artifact,
    normalized_native_dynamic_build_features,
)
from .native_build_workspace import (
    native_dynamic_cdylib_crate_index,
    native_dynamic_source_cdylib_crate_name,
    resolve_native_build_path,
)


def native_dynamic_build_plan(
    *,
    repo_root: Path,
    stage_dir: Path,
    target_dir: Path | None = None,
    package_exports: list[dict[str, Any]],
    source_packages: dict[str, Path],
    validate_payload: dict[str, Any] | None,
    target_platform: str | None,
    cargo: str,
    locked: bool,
    offline: bool,
    build_features: list[str],
    diagnostics: list[str],
) -> dict[str, object]:
    """Build a non-executing Cargo plan for selected NativeDynamic cdylibs.

    The NativeDynamic stage can still consume prebuilt package artifacts, but
    the report must also expose the exact cdylib Cargo commands needed for the
    same package selection so later execution/signing stages do not infer them.
    """

    plugins_workspace = repo_root / "zircon_plugins" / "Cargo.toml"
    crate_index_diagnostics: list[str] = []
    crate_index = native_dynamic_cdylib_crate_index(
        plugins_workspace, crate_index_diagnostics
    )
    diagnostics.extend(crate_index_diagnostics)
    cargo_profile_diagnostics: list[str] = []
    cargo_profile = native_dynamic_cargo_profile(
        validate_payload, cargo_profile_diagnostics
    )
    diagnostics.extend(cargo_profile_diagnostics)
    feature_diagnostics: list[str] = []
    features = normalized_native_dynamic_build_features(
        build_features,
        feature_diagnostics,
    )
    diagnostics.extend(feature_diagnostics)
    target_dir = resolve_native_build_path(
        "native dynamic build target directory",
        target_dir.expanduser() if target_dir else stage_dir / "target",
        diagnostics,
    )
    resolved_plugins_workspace = resolve_native_build_path(
        "native dynamic plugin workspace manifest",
        plugins_workspace,
        diagnostics,
    )
    packages: list[dict[str, object]] = []

    if (
        target_dir is not None
        and resolved_plugins_workspace is not None
        and not cargo_profile_diagnostics
        and not crate_index_diagnostics
        and not feature_diagnostics
    ):
        for package_export in package_exports:
            package_id = str(package_export["package_id"])
            source_package = source_packages.get(package_id)
            if source_package is None:
                continue
            crate_name = native_dynamic_source_cdylib_crate_name(
                source_package / "plugin.toml",
                crate_index,
                package_id,
                diagnostics,
            )
            if crate_name is None:
                continue
            crate = crate_index.get(crate_name)
            if crate is None:
                diagnostics.append(
                    f"native dynamic package {package_id} crate {crate_name} is not a cdylib workspace member"
                )
                continue
            command = native_dynamic_cargo_build_command(
                cargo=cargo,
                workspace_manifest=resolved_plugins_workspace,
                crate_name=crate_name,
                target_dir=target_dir,
                cargo_profile=cargo_profile,
                locked=locked,
                offline=offline,
                features=features,
            )
            packages.append(
                {
                    "package_id": package_id,
                    "crate_name": crate_name,
                    "manifest_path": str(crate["manifest_path"]),
                    "workspace_manifest": str(resolved_plugins_workspace),
                    "target_dir": str(target_dir),
                    "cargo_profile": cargo_profile,
                    "release": cargo_profile == "release",
                    "features": features,
                    "command": command,
                    "expected_loadable_artifact": str(
                        native_dynamic_expected_loadable_artifact(
                            target_dir,
                            cargo_profile,
                            crate_name,
                            target_platform,
                        )
                    ),
                }
            )

    return {
        "fatal": bool(diagnostics),
        "diagnostics": list(diagnostics),
        "workspace_manifest": str(resolved_plugins_workspace or plugins_workspace),
        "target_dir": str(target_dir or stage_dir / "target"),
        "cargo_profile": cargo_profile,
        "release": cargo_profile == "release",
        "build_features": features,
        "package_count": len(packages),
        "packages": packages,
    }
