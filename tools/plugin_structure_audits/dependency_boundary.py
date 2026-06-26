from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any

from .manifest_schema import expected_plugin_manifest_roots

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11 fallback.
    import tomli as tomllib  # type: ignore[no-redef]


FORBIDDEN_DIST_DEPENDENCIES = {
    "zircon_runtime",
    "zircon_editor",
    "zircon_app",
    "wgpu",
    "slint",
    "winit",
}
ALLOWED_DISTRIBUTION_FORMS = {"embed", "dist"}
DIST_PACKAGING = "native_dynamic"
EMBED_PACKAGING = "library_embed"
DIST_CRATE_TYPE = "cdylib"
EMBED_CRATE_TYPE = "rlib"
SDK_DEPENDENCY = "zircon_plugin_sdk"
SDK_DIST_FEATURES = {"native", "dist"}


@dataclass(frozen=True)
class PluginDependencyBoundaryAudit:
    dist_capable_plugins: list[str]
    dist_build_matrix_entries: list[dict[str, str]]
    distribution_section_violation_details: list[str]
    dist_dependency_boundary_violation_details: list[str]

    def to_json(self) -> dict[str, Any]:
        distribution_section_violations = len(
            self.distribution_section_violation_details
        )
        dist_dependency_boundary_violations = len(
            self.dist_dependency_boundary_violation_details
        )
        return {
            "dist_capable_plugin_count": len(self.dist_capable_plugins),
            "dist_capable_plugins": self.dist_capable_plugins,
            "dist_build_matrix_count": len(self.dist_build_matrix_entries),
            "dist_build_matrix_entries": self.dist_build_matrix_entries,
            "distribution_section_violations": distribution_section_violations,
            "distribution_section_violation_details": (
                self.distribution_section_violation_details
            ),
            "dist_dependency_boundary_violations": (
                dist_dependency_boundary_violations
            ),
            "dist_dependency_boundary_violation_details": (
                self.dist_dependency_boundary_violation_details
            ),
            "m1_dist_dependency_boundary_gate_status": (
                "dist-boundary-clean"
                if distribution_section_violations == 0
                and dist_dependency_boundary_violations == 0
                else "dist-boundary-debt-present"
            ),
        }


def audit_plugin_dependency_boundary(repo_root: Path) -> PluginDependencyBoundaryAudit:
    plugin_workspace = repo_root / "zircon_plugins"
    distribution_violations: list[str] = []
    boundary_violations: list[str] = []
    crate_index = workspace_crate_index(plugin_workspace, boundary_violations)
    dist_capable_plugins: list[str] = []
    dist_build_matrix_entries: list[dict[str, str]] = []

    for plugin_root in expected_plugin_manifest_roots(plugin_workspace):
        manifest_path = plugin_workspace / Path(plugin_root) / "plugin.toml"
        manifest = read_toml(manifest_path, distribution_violations)
        if manifest is None:
            continue
        distribution = manifest.get("distribution")
        if distribution is None:
            continue
        display_path = manifest_path.relative_to(repo_root).as_posix()
        if not isinstance(distribution, dict):
            distribution_violations.append(
                f"{display_path}: [distribution] must be a table"
            )
            continue

        forms = string_array_field(
            display_path,
            distribution,
            "distribution.forms",
            distribution_violations,
        )
        if forms is None:
            continue
        invalid_forms = sorted(set(forms) - ALLOWED_DISTRIBUTION_FORMS)
        if invalid_forms:
            distribution_violations.append(
                f"{display_path}: distribution.forms contains unsupported values: "
                + ", ".join(invalid_forms)
            )
        if "dist" not in forms:
            continue

        plugin_id = manifest.get("id")
        if not isinstance(plugin_id, str) or not plugin_id.strip():
            plugin_id = plugin_root
        dist_capable_plugins.append(plugin_id)
        collect_dist_distribution_violations(
            display_path,
            manifest,
            distribution,
            forms,
            distribution_violations,
        )
        dist_crate = distribution.get("dist_crate")
        if not isinstance(dist_crate, str) or not dist_crate.strip():
            continue
        crate = crate_index.get(dist_crate)
        if crate is None:
            boundary_violations.append(
                f"{display_path}: distribution.dist_crate {dist_crate} is not a "
                "zircon_plugins workspace package"
            )
            continue
        dist_build_matrix_entries.append(
            {
                "plugin_id": plugin_id,
                "package": dist_crate,
            }
        )
        collect_dist_crate_boundary_violations(
            display_path,
            dist_crate,
            forms,
            crate["manifest"],
            crate["manifest_path"],
            boundary_violations,
        )

    return PluginDependencyBoundaryAudit(
        dist_capable_plugins=sorted(dist_capable_plugins),
        dist_build_matrix_entries=sorted(
            dist_build_matrix_entries,
            key=lambda entry: (entry["plugin_id"], entry["package"]),
        ),
        distribution_section_violation_details=distribution_violations,
        dist_dependency_boundary_violation_details=boundary_violations,
    )


def workspace_crate_index(
    plugin_workspace: Path,
    violations: list[str],
) -> dict[str, dict[str, Any]]:
    workspace_manifest_path = plugin_workspace / "Cargo.toml"
    workspace_manifest = read_toml(workspace_manifest_path, violations)
    if workspace_manifest is None:
        return {}
    members = workspace_manifest.get("workspace", {}).get("members", [])
    if not isinstance(members, list):
        violations.append("zircon_plugins/Cargo.toml: workspace.members must be an array")
        return {}

    crates: dict[str, dict[str, Any]] = {}
    for index, member in enumerate(members):
        member_label = f"zircon_plugins/Cargo.toml: workspace.members[{index}]"
        if not isinstance(member, str) or not member.strip() or member.strip() != member:
            violations.append(f"{member_label} must be a non-empty trimmed string")
            continue
        member_path = Path(PurePosixPath(member))
        if member_path.is_absolute() or ".." in member_path.parts:
            violations.append(f"{member_label} must be a safe relative path")
            continue
        member_manifest_path = plugin_workspace / member_path / "Cargo.toml"
        member_manifest = read_toml(member_manifest_path, violations)
        if member_manifest is None:
            continue
        package = member_manifest.get("package", {})
        package_name = package.get("name") if isinstance(package, dict) else None
        if not isinstance(package_name, str) or not package_name.strip():
            violations.append(
                f"{member_manifest_path.as_posix()}: package.name must be a non-empty string"
            )
            continue
        crates[package_name] = {
            "member": member,
            "manifest_path": member_manifest_path,
            "manifest": member_manifest,
        }
    return crates


def collect_dist_distribution_violations(
    display_path: str,
    manifest: dict[str, Any],
    distribution: dict[str, Any],
    forms: list[str],
    violations: list[str],
) -> None:
    default_packaging = string_array_field(
        display_path,
        distribution,
        "distribution.default_packaging",
        violations,
    )
    for field in (
        "engine_compat",
        "dist_crate",
        "descriptor_symbol",
    ):
        string_field(display_path, distribution, f"distribution.{field}", violations)
    runtime_entry = optional_string_field(
        display_path,
        distribution,
        "distribution.runtime_entry",
        violations,
    )
    editor_entry = optional_string_field(
        display_path,
        distribution,
        "distribution.editor_entry",
        violations,
    )
    if runtime_entry is None and editor_entry is None:
        violations.append(
            f"{display_path}: distribution must declare runtime_entry or editor_entry"
        )
    abi_version = distribution.get("abi_version")
    if not isinstance(abi_version, int) or abi_version <= 0:
        violations.append(
            f"{display_path}: distribution.abi_version must be a positive integer"
        )
    if "assets" in distribution:
        string_array_field(display_path, distribution, "distribution.assets", violations)

    if default_packaging is not None:
        if "dist" in forms and DIST_PACKAGING not in default_packaging:
            violations.append(
                f"{display_path}: distribution.default_packaging must include "
                f"{DIST_PACKAGING} when distribution.forms includes dist"
            )
        if "embed" in forms and EMBED_PACKAGING not in default_packaging:
            violations.append(
                f"{display_path}: distribution.default_packaging must include "
                f"{EMBED_PACKAGING} when distribution.forms includes embed"
            )
    dist_crate = distribution.get("dist_crate")
    if isinstance(dist_crate, str) and dist_crate.strip():
        modules = manifest.get("modules")
        module_crates = (
            [
                module.get("crate_name")
                for module in modules
                if isinstance(module, dict) and isinstance(module.get("crate_name"), str)
            ]
            if isinstance(modules, list)
            else []
        )
        if dist_crate not in module_crates:
            violations.append(
                f"{display_path}: distribution.dist_crate {dist_crate} is not declared "
                "by any [[modules]].crate_name"
            )


def collect_dist_crate_boundary_violations(
    display_path: str,
    dist_crate: str,
    forms: list[str],
    crate_manifest: dict[str, Any],
    crate_manifest_path: Path,
    violations: list[str],
) -> None:
    crate_label = crate_manifest_path.as_posix()
    lib = crate_manifest.get("lib", {})
    crate_types = lib.get("crate-type", []) if isinstance(lib, dict) else []
    if not isinstance(crate_types, list) or not all(
        isinstance(crate_type, str) for crate_type in crate_types
    ):
        violations.append(f"{crate_label}: lib.crate-type must be a string array")
    else:
        if DIST_CRATE_TYPE not in crate_types:
            violations.append(
                f"{crate_label}: dist crate {dist_crate} must include "
                f"{DIST_CRATE_TYPE} in lib.crate-type"
            )
        if "embed" in forms and EMBED_CRATE_TYPE not in crate_types:
            violations.append(
                f"{crate_label}: embed-capable crate {dist_crate} must include "
                f"{EMBED_CRATE_TYPE} in lib.crate-type"
            )

    features = crate_manifest.get("features", {})
    if not isinstance(features, dict):
        violations.append(f"{crate_label}: features must be a table")
        features = {}
    dist_features = cargo_feature_value(
        crate_label,
        features,
        "dist",
        violations,
    )
    if dist_features is None:
        violations.append(
            f"{crate_label}: dist-capable crate {dist_crate} must define a dist feature"
        )
        dist_features = []

    for section, dependencies in dependency_tables(crate_manifest):
        for dependency_name, dependency_spec in dependencies.items():
            if dependency_name not in FORBIDDEN_DIST_DEPENDENCIES:
                continue
            if dependency_is_optional(dependency_spec):
                if feature_enables_dependency(dist_features, dependency_name):
                    violations.append(
                        f"{crate_label}: features.dist must not enable forbidden "
                        f"dependency {dependency_name}"
                    )
                continue
            violations.append(
                f"{crate_label}: {section}.{dependency_name} is forbidden for dist "
                "plugin crates unless it is optional and excluded from features.dist"
            )

    sdk_dependency = find_dependency(crate_manifest, SDK_DEPENDENCY)
    if sdk_dependency is None:
        violations.append(
            f"{crate_label}: dist-capable crate {dist_crate} must depend on "
            f"{SDK_DEPENDENCY}"
        )
        return
    if dependency_default_features_enabled(sdk_dependency):
        violations.append(
            f"{crate_label}: {SDK_DEPENDENCY} must set default-features = false for "
            "dist plugin crates"
        )
    if not dependency_or_dist_feature_enables_sdk_dist(sdk_dependency, dist_features):
        violations.append(
            f"{crate_label}: {SDK_DEPENDENCY} must enable native/dist ABI helpers "
            "directly or through features.dist"
        )
    if feature_enables_dependency(dist_features, "zircon_runtime"):
        violations.append(
            f"{display_path}: features.dist must not route through zircon_runtime"
        )


def dependency_tables(
    crate_manifest: dict[str, Any],
) -> list[tuple[str, dict[str, Any]]]:
    tables: list[tuple[str, dict[str, Any]]] = []
    for section in ("dependencies", "build-dependencies"):
        dependencies = crate_manifest.get(section, {})
        if isinstance(dependencies, dict):
            tables.append((section, dependencies))
    target = crate_manifest.get("target", {})
    if isinstance(target, dict):
        for target_name, target_table in target.items():
            if not isinstance(target_table, dict):
                continue
            for section in ("dependencies", "build-dependencies"):
                dependencies = target_table.get(section, {})
                if isinstance(dependencies, dict):
                    tables.append((f"target.{target_name}.{section}", dependencies))
    return tables


def find_dependency(crate_manifest: dict[str, Any], dependency_name: str) -> object | None:
    for _, dependencies in dependency_tables(crate_manifest):
        if dependency_name in dependencies:
            return dependencies[dependency_name]
    return None


def dependency_is_optional(dependency_spec: object) -> bool:
    return isinstance(dependency_spec, dict) and dependency_spec.get("optional") is True


def dependency_default_features_enabled(dependency_spec: object) -> bool:
    if not isinstance(dependency_spec, dict):
        return True
    return dependency_spec.get("default-features") is not False


def dependency_or_dist_feature_enables_sdk_dist(
    dependency_spec: object,
    dist_features: list[str],
) -> bool:
    if isinstance(dependency_spec, dict):
        features = dependency_spec.get("features", [])
        if isinstance(features, list) and any(
            isinstance(feature, str) and feature in SDK_DIST_FEATURES
            for feature in features
        ):
            return True
    return any(
        feature in {f"{SDK_DEPENDENCY}/native", f"{SDK_DEPENDENCY}/dist"}
        for feature in dist_features
    )


def feature_enables_dependency(feature_values: list[str], dependency_name: str) -> bool:
    return dependency_name in feature_values or f"dep:{dependency_name}" in feature_values


def string_array_field(
    display_path: str,
    table: dict[str, Any],
    field_label: str,
    violations: list[str],
) -> list[str] | None:
    field_name = field_label.split(".")[-1]
    return string_list_value(display_path, table, field_label, violations, field_name)


def string_list_value(
    display_path: str,
    table: dict[str, Any],
    field_label: str,
    violations: list[str],
    field_name: str | None = None,
) -> list[str] | None:
    field = field_name or field_label.split(".")[-1]
    if field not in table:
        violations.append(f"{display_path}: missing {field_label}")
        return None
    value = table[field]
    if not isinstance(value, list) or not value:
        violations.append(f"{display_path}: {field_label} must be a non-empty string array")
        return None
    values: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, str) or not entry.strip() or entry.strip() != entry:
            violations.append(
                f"{display_path}: {field_label}[{index}] must be a non-empty "
                "trimmed string"
            )
            continue
        values.append(entry)
    return values


def cargo_feature_value(
    display_path: str,
    features: dict[str, Any],
    feature_name: str,
    violations: list[str],
) -> list[str] | None:
    if feature_name not in features:
        return None
    value = features[feature_name]
    if not isinstance(value, list):
        violations.append(
            f"{display_path}: features.{feature_name} must be a string array"
        )
        return None
    values: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, str) or not entry.strip() or entry.strip() != entry:
            violations.append(
                f"{display_path}: features.{feature_name}[{index}] must be a "
                "non-empty trimmed string"
            )
            continue
        values.append(entry)
    return values


def string_field(
    display_path: str,
    table: dict[str, Any],
    field_label: str,
    violations: list[str],
) -> str | None:
    field = field_label.split(".")[-1]
    if field not in table:
        violations.append(f"{display_path}: missing {field_label}")
        return None
    value = table[field]
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        violations.append(
            f"{display_path}: {field_label} must be a non-empty trimmed string"
        )
        return None
    return value


def optional_string_field(
    display_path: str,
    table: dict[str, Any],
    field_label: str,
    violations: list[str],
) -> str | None:
    field = field_label.split(".")[-1]
    if field not in table:
        return None
    return string_field(display_path, table, field_label, violations)


def read_toml(path: Path, violations: list[str]) -> dict[str, Any] | None:
    try:
        with path.open("rb") as toml_file:
            manifest = tomllib.load(toml_file)
    except tomllib.TOMLDecodeError as error:
        violations.append(f"{path.as_posix()}: TOML parse error: {error}")
        return None
    except OSError as error:
        violations.append(f"{path.as_posix()}: could not be read: {error}")
        return None
    if not isinstance(manifest, dict):
        violations.append(f"{path.as_posix()}: TOML root must be a table")
        return None
    return manifest
