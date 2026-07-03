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
    "default_packaging",
)
MANIFEST_ROOT_FIELDS = frozenset(
    """
    asset_importers asset_roots capabilities capability_statuses category components
    content_roots default_packaging dependencies description display_name distribution
    event_catalogs feature_extensions geometry_sources id maturity modules optional_features
    options package_company package_kind package_name package_prefix provides_interfaces
    sdk_api_version shader_permutation shading_models supported_platforms supported_targets
    ui_components version
    """.split()
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
INIT_LEVEL_VALUES = ("kernel", "servers", "scene", "editor", "post")
SUPPORTED_PLATFORM_VALUES = (
    "windows",
    "linux",
    "macos",
    "android",
    "ios",
    "web_gpu",
    "wasm",
    "headless",
    "windows-x86_64",
    "linux-x86_64",
    "macos-aarch64",
)
SUPPORTED_PLATFORM_ALIASES = {"windows-x86_64": "windows", "linux-x86_64": "linux", "macos-aarch64": "macos"}
MATURITY_VALUES = ("stable", "beta", "experimental")
MODULE_KIND_VALUES = ("runtime", "editor", "native", "vm")
PACKAGING_VALUES = ("source_template", "library_embed", "native_dynamic")
@dataclass(frozen=True)
class PluginManifestSchemaAudit:
    expected_manifest_roots: list[str]
    missing_plugin_toml_paths: list[str]
    manifest_schema_violation_details: list[str]
    generated_manifest_header_violation_paths: list[str]
    feature_provider_package_projection_count: int

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
            "feature_provider_package_projection_count": (
                self.feature_provider_package_projection_count
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
    loaded_manifests: list[tuple[str, dict[str, Any]]] = []

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
        loaded_manifests.append((display_path, manifest))
        collect_manifest_schema_violations(display_path, manifest, violations)

    from .manifest_schema_asset_importer_capability_gates import collect_asset_importer_required_capability_gate_violations
    from .manifest_schema_dependency_capability_targets import collect_dependency_capability_target_violations
    from .manifest_schema_event_catalog_namespaces import collect_global_event_catalog_namespace_violations
    from .manifest_schema_feature_provider_packages import collect_feature_provider_package_projection_violations
    from .manifest_schema_feature_provider_targets import collect_feature_provider_target_identity_violations
    from .manifest_schema_global_identities import collect_global_manifest_identity_violations
    from .manifest_schema_module_crates import collect_module_workspace_crate_violations
    from .manifest_schema_option_capability_gates import collect_option_required_capability_gate_violations

    collect_global_manifest_identity_violations(loaded_manifests, violations)
    collect_global_event_catalog_namespace_violations(loaded_manifests, violations)
    collect_feature_provider_target_identity_violations(loaded_manifests, violations)
    feature_provider_package_projection_count = (
        collect_feature_provider_package_projection_violations(
            repo_root,
            loaded_manifests,
            violations,
        )
    )
    collect_dependency_capability_target_violations(loaded_manifests, violations)
    collect_asset_importer_required_capability_gate_violations(
        loaded_manifests,
        violations,
    )
    collect_option_required_capability_gate_violations(loaded_manifests, violations)
    collect_module_workspace_crate_violations(
        plugin_workspace,
        loaded_manifests,
        violations,
    )

    return PluginManifestSchemaAudit(
        expected_manifest_roots=expected_roots,
        missing_plugin_toml_paths=missing_paths,
        manifest_schema_violation_details=violations,
        generated_manifest_header_violation_paths=generated_header_violations,
        feature_provider_package_projection_count=(
            feature_provider_package_projection_count
        ),
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
    from .manifest_schema_root_metadata import collect_root_metadata_schema_violations
    from .manifest_schema_layout_roots import collect_layout_root_schema_violations

    collect_manifest_root_known_field_violations(display_path, manifest, violations)
    for field in REQUIRED_ROOT_FIELDS:
        collect_required_field_violation(display_path, field, manifest, violations)
    collect_root_metadata_schema_violations(display_path, manifest, violations)
    collect_layout_root_schema_violations(display_path, manifest, violations)

    asset_importers = manifest.get("asset_importers")
    if asset_importers is not None:
        from .manifest_schema_asset_importers import (
            collect_asset_importers_schema_violations,
        )

        collect_asset_importers_schema_violations(
            display_path,
            manifest,
            asset_importers,
            violations,
        )

    capability_statuses = manifest.get("capability_statuses")
    if capability_statuses is not None:
        from .manifest_schema_capability_statuses import (
            collect_capability_statuses_schema_violations,
        )

        collect_capability_statuses_schema_violations(
            display_path,
            manifest,
            capability_statuses,
            violations,
        )

    from .manifest_schema_components import collect_components_schema_violations

    collect_components_schema_violations(display_path, manifest, violations)

    from .manifest_schema_event_catalogs import collect_event_catalogs_schema_violations

    collect_event_catalogs_schema_violations(display_path, manifest, violations)

    from .manifest_schema_dependencies import collect_dependencies_schema_violations

    collect_dependencies_schema_violations(display_path, manifest, violations)

    from .manifest_schema_interfaces import (
        collect_provided_interfaces_schema_violations,
    )

    collect_provided_interfaces_schema_violations(display_path, manifest, violations)

    options = manifest.get("options")
    if options is not None:
        from .manifest_schema_options import collect_options_schema_violations

        collect_options_schema_violations(display_path, manifest, violations)

    from .manifest_schema_geometry_sources import (
        collect_geometry_source_schema_violations,
    )

    collect_geometry_source_schema_violations(display_path, manifest, violations)

    from .manifest_schema_shading_models import (
        collect_shading_model_schema_violations,
    )

    collect_shading_model_schema_violations(display_path, manifest, violations)

    from .manifest_schema_modules import module_supported_targets

    module_namespace = manifest.get("id")
    if not is_non_empty_trimmed_string(module_namespace):
        module_namespace = None
    supported_targets = module_supported_targets(manifest)
    module_seen_names: dict[str, str] = {}
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
            namespace_id=module_namespace,
            supported_targets=supported_targets,
            seen_names=module_seen_names,
            row_identity=f"modules[{index}]",
        )

    optional_features = manifest.get("optional_features")
    if optional_features is not None:
        from .manifest_schema_optional_features import (
            collect_optional_features_schema_violations,
        )

        collect_optional_features_schema_violations(
            display_path,
            manifest,
            optional_features,
            violations,
            module_seen_names=module_seen_names,
            module_supported_targets=supported_targets,
        )

    feature_extensions = manifest.get("feature_extensions")
    if feature_extensions is not None:
        from .manifest_schema_feature_extensions import (
            collect_feature_extensions_schema_violations,
        )

        collect_feature_extensions_schema_violations(
            display_path,
            manifest,
            feature_extensions,
            violations,
            module_seen_names=module_seen_names,
            module_supported_targets=supported_targets,
        )


def collect_manifest_root_known_field_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    for field in sorted(manifest):
        if field not in MANIFEST_ROOT_FIELDS:
            violations.append(
                f"{display_path}: {field} is not a known manifest root field"
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
    namespace_id: str | None = None,
    supported_targets: set[str] | None = None,
    seen_names: dict[str, str] | None = None,
    row_identity: str | None = None,
) -> None:
    from .manifest_schema_modules import (
        collect_module_schema_violations as collect_module_row_schema_violations,
    )

    collect_module_row_schema_violations(
        display_path,
        field_label,
        module,
        violations,
        table_label=table_label,
        namespace_id=namespace_id,
        supported_targets=supported_targets,
        seen_names=seen_names,
        row_identity=row_identity,
    )


def collect_feature_dependency_schema_violations(
    display_path: str,
    field_label: str,
    dependency: object,
    violations: list[str],
) -> None:
    if not isinstance(dependency, dict):
        violations.append(f"{display_path}: {field_label} must be a table")
        return
    violations.extend(
        f"{display_path}: {field_label}.{field} is not a known optional feature dependency field"
        for field in sorted(set(dependency) - set(REQUIRED_FEATURE_DEPENDENCY_FIELDS))
    )
    for field in REQUIRED_FEATURE_DEPENDENCY_FIELDS:
        collect_required_field_violation(
            display_path,
            f"{field_label}.{field}",
            dependency,
            violations,
            field_name=field,
        )

def collect_feature_dependency_primary_count_violation(
    display_path: str,
    dependency_label: str,
    dependencies: list[object],
    violations: list[str],
) -> None:
    if not all(
        optional_feature_dependency_row_supports_primary_count(dependency)
        for dependency in dependencies
    ):
        return
    primary_count = sum(
        1
        for dependency in dependencies
        if isinstance(dependency, dict) and dependency["primary"] is True
    )
    if primary_count != 1:
        violations.append(
            f"{display_path}: {dependency_label} "
            "should declare exactly one primary dependency"
        )

def collect_feature_dependency_duplicate_identity_violations(
    display_path: str,
    dependency_label: str,
    dependencies: list[object],
    violations: list[str],
) -> None:
    seen: dict[tuple[str, str], int] = {}
    for dependency_index, dependency in enumerate(dependencies):
        if not optional_feature_dependency_row_supports_primary_count(dependency):
            continue
        if not isinstance(dependency, dict):
            continue
        identity = (dependency["plugin_id"], dependency["capability"])
        previous_index = seen.get(identity)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {dependency_label}[{dependency_index}] "
                f"duplicates dependency row {previous_index}"
            )
            continue
        seen[identity] = dependency_index

def collect_feature_dependency_primary_target_violations(
    display_path: str,
    dependency_label: str,
    dependencies: list[object],
    target_plugin_id: str,
    target_capabilities: set[str] | None,
    plugin_id_message: str,
    capability_message: str,
    violations: list[str],
) -> None:
    if not all(
        optional_feature_dependency_row_supports_primary_count(dependency)
        for dependency in dependencies
    ):
        return
    primary_dependencies = [
        (dependency_index, dependency)
        for dependency_index, dependency in enumerate(dependencies)
        if isinstance(dependency, dict) and dependency["primary"] is True
    ]
    if len(primary_dependencies) != 1:
        return
    dependency_index, dependency = primary_dependencies[0]
    if not isinstance(dependency, dict):
        return
    diagnostic_label = f"{display_path}: {dependency_label}[{dependency_index}]"
    if dependency["plugin_id"] != target_plugin_id:
        violations.append(
            f"{diagnostic_label} {plugin_id_message} {target_plugin_id}"
        )
    if target_capabilities is not None and dependency["capability"] not in target_capabilities:
        violations.append(
            f"{diagnostic_label} {capability_message}"
        )

def optional_feature_dependency_primary_target(
    manifest: dict[str, Any],
) -> tuple[str, set[str]] | None:
    package_id = manifest.get("id")
    capabilities = manifest.get("capabilities")
    if not is_non_empty_trimmed_string(package_id):
        return None
    if not isinstance(capabilities, list) or not capabilities:
        return None
    if not all(is_non_empty_trimmed_string(capability) for capability in capabilities):
        return None
    return (package_id, set(capabilities))


def optional_feature_dependency_row_supports_primary_count(
    dependency: object,
) -> bool:
    if not isinstance(dependency, dict):
        return False
    return (
        is_non_empty_trimmed_string(dependency.get("plugin_id"))
        and is_non_empty_trimmed_string(dependency.get("capability"))
        and type(dependency.get("primary")) is bool
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
        if not isinstance(value, str) or not value.strip() or value.strip() != value:
            violations.append(
                f"{display_path}: {field_label} must be a non-empty trimmed string"
            )
        return
    if field in STRING_ARRAY_FIELDS:
        if not isinstance(value, list) or not value:
            violations.append(
                f"{display_path}: {field_label} must be a non-empty string array"
            )
            return
        for index, entry in enumerate(value):
            if (
                not isinstance(entry, str)
                or not entry.strip()
                or entry.strip() != entry
            ):
                violations.append(
                    f"{display_path}: {field_label}[{index}] "
                    "must be a non-empty trimmed string"
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

def is_non_empty_trimmed_string(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value


def collect_allowed_string_value(
    display_path: str,
    field_label: str,
    table: dict[str, Any],
    field_name: str,
    allowed_values: tuple[str, ...],
    violations: list[str],
) -> None:
    value = table.get(field_name)
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
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
        if not isinstance(entry, str) or not entry.strip() or entry.strip() != entry:
            continue
        if entry not in allowed:
            violations.append(
                f'{display_path}: {field_label}[{index}] "{entry}" is unsupported; '
                f"expected one of {expected}"
            )
