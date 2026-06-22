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
STRING_FIELDS = {
    "id",
    "version",
    "sdk_api_version",
    "display_name",
    "category",
    "description",
    "maturity",
    "name",
    "kind",
    "crate_name",
}
STRING_ARRAY_FIELDS = {
    "supported_targets",
    "supported_platforms",
    "capabilities",
    "target_modes",
}


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

    modules = manifest.get("modules")
    if not isinstance(modules, list) or not modules:
        violations.append(f"{display_path}: missing non-empty [[modules]]")
        return

    for index, module in enumerate(modules):
        if not isinstance(module, dict):
            violations.append(f"{display_path}: [[modules]][{index}] must be a table")
            continue
        for field in REQUIRED_MODULE_FIELDS:
            collect_required_field_violation(
                display_path,
                f"modules[{index}].{field}",
                module,
                violations,
                field_name=field,
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
