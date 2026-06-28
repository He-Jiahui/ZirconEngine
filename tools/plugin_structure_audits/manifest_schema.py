from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11 fallback.
    import tomli as tomllib  # type: ignore[no-redef]


GENERATED_MANIFEST_HEADER = (
    "# @generated from Rust descriptor package_manifest(); do not edit by hand."
)
SKIPPED_WORKSPACE_ROOTS = {"editor_support", "first_party_runtime_catalog", "plugin_sdk"}
REQUIRED_ROOT_FIELDS = (
    "id",
    "version",
    "sdk_api_version",
    "display_name",
    "category",
    "description",
    "supported_targets",
    "supported_platforms",
    "capabilities",
    "maturity",
)
REQUIRED_MODULE_FIELDS = (
    "name",
    "kind",
    "crate_name",
    "target_modes",
    "capabilities",
)
REQUIRED_FEATURE_FIELDS = (
    "id",
    "display_name",
    "owner_plugin_id",
    "capabilities",
    "default_packaging",
    "enabled_by_default",
)
REQUIRED_FEATURE_DEPENDENCY_FIELDS = (
    "plugin_id",
    "capability",
    "primary",
)
REQUIRED_FEATURE_DISTRIBUTION_FIELDS = (
    "forms",
    "default_packaging",
    "abi_version",
    "engine_compat",
    "dist_crate",
    "descriptor_symbol",
)
OPTIONAL_FEATURE_DISTRIBUTION_STRING_FIELDS = (
    "runtime_entry",
    "editor_entry",
)
STRING_FIELDS = {
    "id",
    "version",
    "sdk_api_version",
    "display_name",
    "category",
    "description",
    "maturity",
    "owner_plugin_id",
    "name",
    "kind",
    "crate_name",
    "plugin_id",
    "capability",
    "engine_compat",
    "dist_crate",
    "descriptor_symbol",
    "runtime_entry",
    "editor_entry",
}
STRING_ARRAY_FIELDS = {
    "supported_targets",
    "supported_platforms",
    "capabilities",
    "target_modes",
    "default_packaging",
    "forms",
    "assets",
}
BOOL_FIELDS = {
    "enabled_by_default",
    "primary",
}
POSITIVE_INT_FIELDS = {
    "abi_version",
}
SUPPORTED_TARGET_VALUES = ("client_runtime", "server_runtime", "editor_host")
SUPPORTED_PLATFORM_VALUES = ("windows", "linux", "macos")
MATURITY_VALUES = ("stable", "beta", "experimental")
MODULE_KIND_VALUES = ("runtime", "editor", "native", "vm")
PACKAGING_VALUES = ("source_template", "library_embed", "native_dynamic")
DISTRIBUTION_FORM_VALUES = ("embed", "dist")


@dataclass(frozen=True)
class PluginManifestSchemaAudit:
    expected_manifest_roots: list[str]
    missing_plugin_toml_paths: list[str]
    manifest_schema_violation_details: list[str]
    generated_manifest_header_violation_paths: list[str]

    def to_json(self) -> dict[str, Any]:
        manifest_count = len(self.expected_manifest_roots) - len(
            self.missing_plugin_toml_paths
        )
        return {
            "expected_manifest_count": len(self.expected_manifest_roots),
            "manifest_count": manifest_count,
            "generated_manifest_count": manifest_count
            - self.hand_written_native_manifest_count,
            "hand_written_native_manifest_count": self.hand_written_native_manifest_count,
            "missing_plugin_toml": len(self.missing_plugin_toml_paths),
            "missing_plugin_toml_paths": self.missing_plugin_toml_paths,
            "manifest_schema_violations": len(self.manifest_schema_violation_details),
            "manifest_schema_violation_details": self.manifest_schema_violation_details,
            "generated_manifest_header_violations": len(
                self.generated_manifest_header_violation_paths
            ),
            "generated_manifest_header_violation_paths": (
                self.generated_manifest_header_violation_paths
            ),
        }

    @property
    def hand_written_native_manifest_count(self) -> int:
        return 1 if "native_dynamic_fixture" in self.expected_manifest_roots else 0


def audit_plugin_manifest_schema(repo_root: Path) -> PluginManifestSchemaAudit:
    plugin_workspace = repo_root / "zircon_plugins"
    expected_roots = expected_plugin_manifest_roots(plugin_workspace)
    missing_paths: list[str] = []
    violations: list[str] = []
    generated_header_violations: list[str] = []

    for plugin_root in expected_roots:
        manifest_path = plugin_workspace / Path(plugin_root) / "plugin.toml"
        display_path = manifest_path.relative_to(repo_root).as_posix()
        if not manifest_path.exists():
            missing_paths.append(display_path)
            continue
        manifest_text = manifest_path.read_text(encoding="utf-8")
        if (
            plugin_root != "native_dynamic_fixture"
            and not manifest_text.startswith(GENERATED_MANIFEST_HEADER)
        ):
            generated_header_violations.append(display_path)
            violations.append(f"{display_path}: missing generated manifest header")
        try:
            manifest = tomllib.loads(manifest_text)
        except tomllib.TOMLDecodeError as error:
            violations.append(f"{display_path}: TOML parse error: {error}")
            continue
        collect_manifest_schema_violations(display_path, manifest, violations)

    return PluginManifestSchemaAudit(
        expected_manifest_roots=expected_roots,
        missing_plugin_toml_paths=missing_paths,
        manifest_schema_violation_details=violations,
        generated_manifest_header_violation_paths=generated_header_violations,
    )


def expected_plugin_manifest_roots(plugin_workspace: Path) -> list[str]:
    cargo_toml = plugin_workspace / "Cargo.toml"
    cargo_manifest = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    members = cargo_manifest.get("workspace", {}).get("members", [])
    roots: set[str] = set()
    for member in members:
        parts = PurePosixPath(member).parts
        if not parts or parts[0] in SKIPPED_WORKSPACE_ROOTS:
            continue
        if parts[0] == "asset_importers":
            if len(parts) >= 2:
                roots.add(f"{parts[0]}/{parts[1]}")
            continue
        roots.add(parts[0])
    return sorted(roots)


def collect_manifest_schema_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    for field in REQUIRED_ROOT_FIELDS:
        collect_required_field_violation(display_path, field, manifest, violations)
    collect_allowed_string_array_values(
        display_path,
        "supported_targets",
        manifest,
        "supported_targets",
        SUPPORTED_TARGET_VALUES,
        violations,
    )
    collect_allowed_string_array_values(
        display_path,
        "supported_platforms",
        manifest,
        "supported_platforms",
        SUPPORTED_PLATFORM_VALUES,
        violations,
    )
    collect_allowed_string_value(
        display_path,
        "maturity",
        manifest,
        "maturity",
        MATURITY_VALUES,
        violations,
    )

    modules = manifest.get("modules")
    if not isinstance(modules, list) or not modules:
        violations.append(f"{display_path}: missing non-empty [[modules]]")
        return

    for index, module in enumerate(modules):
        collect_module_schema_violations(
            display_path,
            f"modules[{index}]",
            module,
            violations,
            table_label=f"[[modules]][{index}]",
        )

    optional_features = manifest.get("optional_features")
    if optional_features is None:
        return
    if not isinstance(optional_features, list):
        violations.append(f"{display_path}: optional_features must be an array of tables")
        return
    for feature_index, feature in enumerate(optional_features):
        if not isinstance(feature, dict):
            violations.append(
                f"{display_path}: optional_features[{feature_index}] must be a table"
            )
            continue
        collect_optional_feature_schema_violations(
            display_path,
            feature_index,
            feature,
            violations,
        )
        feature_dependencies = feature.get("dependencies")
        if feature_dependencies is not None:
            if not isinstance(feature_dependencies, list):
                violations.append(
                    f"{display_path}: optional_features[{feature_index}].dependencies must be an array of tables"
                )
            else:
                for dependency_index, dependency in enumerate(feature_dependencies):
                    collect_optional_feature_dependency_schema_violations(
                        display_path,
                        feature_index,
                        dependency_index,
                        dependency,
                        violations,
                    )
        feature_distribution = feature.get("distribution")
        if feature_distribution is not None:
            collect_optional_feature_distribution_schema_violations(
                display_path,
                feature_index,
                feature_distribution,
                violations,
            )
        feature_modules = feature.get("modules")
        if feature_modules is None:
            continue
        if not isinstance(feature_modules, list):
            violations.append(
                f"{display_path}: optional_features[{feature_index}].modules must be an array of tables"
            )
            continue
        for module_index, module in enumerate(feature_modules):
            collect_module_schema_violations(
                display_path,
                f"optional_features[{feature_index}].modules[{module_index}]",
                module,
                violations,
            )


def collect_optional_feature_schema_violations(
    display_path: str,
    feature_index: int,
    feature: dict[str, Any],
    violations: list[str],
) -> None:
    field_label = f"optional_features[{feature_index}]"
    for field in REQUIRED_FEATURE_FIELDS:
        collect_required_field_violation(
            display_path,
            f"{field_label}.{field}",
            feature,
            violations,
            field_name=field,
        )
    collect_optional_trimmed_string_field_violation(
        display_path,
        f"{field_label}.provider_package_id",
        feature,
        "provider_package_id",
        violations,
    )
    collect_allowed_string_array_values(
        display_path,
        f"{field_label}.default_packaging",
        feature,
        "default_packaging",
        PACKAGING_VALUES,
        violations,
        )


def collect_optional_trimmed_string_field_violation(
    display_path: str,
    field_label: str,
    table: dict[str, Any],
    field_name: str,
    violations: list[str],
) -> None:
    if field_name not in table:
        return
    value = table[field_name]
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        violations.append(
            f"{display_path}: {field_label} must be a non-empty trimmed string"
        )


def collect_module_schema_violations(
    display_path: str,
    field_label: str,
    module: object,
    violations: list[str],
    *,
    table_label: str | None = None,
) -> None:
    if not isinstance(module, dict):
        violations.append(
            f"{display_path}: {table_label or field_label} must be a table"
        )
        return
    for field in REQUIRED_MODULE_FIELDS:
        collect_required_field_violation(
            display_path,
            f"{field_label}.{field}",
            module,
            violations,
            field_name=field,
        )
    collect_allowed_string_value(
        display_path,
        f"{field_label}.kind",
        module,
        "kind",
        MODULE_KIND_VALUES,
        violations,
    )
    collect_allowed_string_array_values(
        display_path,
        f"{field_label}.target_modes",
        module,
        "target_modes",
        SUPPORTED_TARGET_VALUES,
        violations,
    )


def collect_optional_feature_dependency_schema_violations(
    display_path: str,
    feature_index: int,
    dependency_index: int,
    dependency: object,
    violations: list[str],
) -> None:
    field_label = f"optional_features[{feature_index}].dependencies[{dependency_index}]"
    if not isinstance(dependency, dict):
        violations.append(f"{display_path}: {field_label} must be a table")
        return
    for field in REQUIRED_FEATURE_DEPENDENCY_FIELDS:
        collect_required_field_violation(
            display_path,
            f"{field_label}.{field}",
            dependency,
            violations,
            field_name=field,
        )


def collect_optional_feature_distribution_schema_violations(
    display_path: str,
    feature_index: int,
    distribution: object,
    violations: list[str],
) -> None:
    field_label = f"optional_features[{feature_index}].distribution"
    if not isinstance(distribution, dict):
        violations.append(f"{display_path}: {field_label} must be a table")
        return
    for field in REQUIRED_FEATURE_DISTRIBUTION_FIELDS:
        collect_required_field_violation(
            display_path,
            f"{field_label}.{field}",
            distribution,
            violations,
            field_name=field,
        )
    for field in OPTIONAL_FEATURE_DISTRIBUTION_STRING_FIELDS:
        if field in distribution:
            collect_required_field_violation(
                display_path,
                f"{field_label}.{field}",
                distribution,
                violations,
                field_name=field,
            )
    if "assets" in distribution:
        collect_required_field_violation(
            display_path,
            f"{field_label}.assets",
            distribution,
            violations,
            field_name="assets",
        )
    if "runtime_entry" not in distribution and "editor_entry" not in distribution:
        violations.append(
            f"{display_path}: {field_label} must declare runtime_entry or editor_entry"
        )
    collect_allowed_string_array_values(
        display_path,
        f"{field_label}.forms",
        distribution,
        "forms",
        DISTRIBUTION_FORM_VALUES,
        violations,
    )
    collect_allowed_string_array_values(
        display_path,
        f"{field_label}.default_packaging",
        distribution,
        "default_packaging",
        PACKAGING_VALUES,
        violations,
    )


def collect_required_field_violation(
    display_path: str,
    field_label: str,
    table: dict[str, Any],
    violations: list[str],
    *,
    field_name: str | None = None,
) -> None:
    field = field_name or field_label
    if field not in table:
        violations.append(f"{display_path}: missing {field_label}")
        return
    value = table[field]
    if field in STRING_FIELDS:
        if not isinstance(value, str) or not value.strip():
            violations.append(f"{display_path}: {field_label} must be a non-empty string")
        return
    if field in STRING_ARRAY_FIELDS:
        if not isinstance(value, list) or not value:
            violations.append(
                f"{display_path}: {field_label} must be a non-empty string array"
            )
            return
        for index, entry in enumerate(value):
            if not isinstance(entry, str) or not entry.strip():
                violations.append(
                    f"{display_path}: {field_label}[{index}] must be a non-empty string"
                )
        return
    if field in BOOL_FIELDS:
        if not isinstance(value, bool):
            violations.append(f"{display_path}: {field_label} must be a bool")
        return
    if field in POSITIVE_INT_FIELDS:
        if type(value) is not int or value <= 0:
            violations.append(
                f"{display_path}: {field_label} must be a positive integer"
            )


def collect_allowed_string_value(
    display_path: str,
    field_label: str,
    table: dict[str, Any],
    field_name: str,
    allowed_values: tuple[str, ...],
    violations: list[str],
) -> None:
    value = table.get(field_name)
    if not isinstance(value, str) or not value.strip():
        return
    allowed = set(allowed_values)
    expected = ", ".join(allowed_values)
    if value not in allowed:
        violations.append(
            f'{display_path}: {field_label} "{value}" is unsupported; '
            f"expected one of {expected}"
        )


def collect_allowed_string_array_values(
    display_path: str,
    field_label: str,
    table: dict[str, Any],
    field_name: str,
    allowed_values: tuple[str, ...],
    violations: list[str],
) -> None:
    value = table.get(field_name)
    if not isinstance(value, list):
        return
    allowed = set(allowed_values)
    expected = ", ".join(allowed_values)
    for index, entry in enumerate(value):
        if not isinstance(entry, str) or not entry.strip():
            continue
        if entry not in allowed:
            violations.append(
                f'{display_path}: {field_label}[{index}] "{entry}" is unsupported; '
                f"expected one of {expected}"
            )
