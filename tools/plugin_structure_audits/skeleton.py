from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11 fallback.
    import tomli as tomllib  # type: ignore[no-redef]

from plugin_structure_audits.manifest_schema import (
    SKIPPED_WORKSPACE_ROOTS,
    expected_plugin_manifest_roots,
)


SKELETON_SAMPLE_ROOTS = ("plugin_sdk_examples",)
SKELETON_EXEMPT_ROOTS = {
    "native_dynamic_fixture": "native-only ABI fixture uses plugin_sdk::native and is exempt from runtime/editor skeleton rules",
}
OWNER_MODULE_KINDS = {"runtime", "editor"}
CORE_WORKSPACE_DEPENDENCIES = (
    "zircon_editor",
    "zircon_runtime",
    "zircon_runtime_interface",
)
SAMPLE_WORKSPACE_DEPENDENCIES = {
    "plugin_sdk_examples/editor": (
        "zircon_editor",
        "zircon_plugin_sdk",
        "zircon_runtime",
    ),
}
MAX_MIGRATION_DEBT_DETAILS = 64
MAX_CORE_WORKSPACE_DEPENDENCY_DETAILS = 64


@dataclass(frozen=True)
class PluginSkeletonAudit:
    sample_roots: list[str]
    sample_violation_details: list[str]
    core_workspace_dependency_count: int
    core_workspace_dependency_violation_details: list[str]
    migration_debt_roots: list[str]
    migration_debt_details: list[str]
    exempt_roots: dict[str, str]

    def to_json(self) -> dict[str, Any]:
        sample_violation_roots = {
            detail.split(":", maxsplit=1)[0]
            for detail in self.sample_violation_details
        }
        workspace_dependency_violations = [
            detail
            for detail in self.sample_violation_details
            if "workspace dependency" in detail
        ]
        sample_conforming_count = len(self.sample_roots) - len(sample_violation_roots)
        sample_status = (
            "sample-clean"
            if not self.sample_violation_details
            else "sample-violations-present"
        )
        core_workspace_dependency_violations = len(
            self.core_workspace_dependency_violation_details
        )
        return {
            "sample_conformance_status": sample_status,
            "sample_roots": self.sample_roots,
            "sample_expected_count": len(self.sample_roots),
            "sample_conforming_count": sample_conforming_count,
            "sample_violation_count": len(self.sample_violation_details),
            "sample_violations": self.sample_violation_details,
            "sample_workspace_dependency_status": (
                "sample-workspace-deps-clean"
                if not workspace_dependency_violations
                else "sample-workspace-deps-violations-present"
            ),
            "sample_workspace_dependency_violation_count": len(
                workspace_dependency_violations
            ),
            "core_workspace_dependency_status": (
                "core-workspace-deps-clean"
                if core_workspace_dependency_violations == 0
                else "core-workspace-deps-violations-present"
            ),
            "core_workspace_dependency_count": self.core_workspace_dependency_count,
            "core_workspace_dependency_violation_count": (
                core_workspace_dependency_violations
            ),
            "core_workspace_dependency_violations": (
                self.core_workspace_dependency_violation_details[
                    :MAX_CORE_WORKSPACE_DEPENDENCY_DETAILS
                ]
            ),
            "core_workspace_dependency_violations_truncated": (
                core_workspace_dependency_violations
                > MAX_CORE_WORKSPACE_DEPENDENCY_DETAILS
            ),
            "migration_debt_count": len(self.migration_debt_roots),
            "migration_debt_roots": self.migration_debt_roots,
            "migration_debt_detail_count": len(self.migration_debt_details),
            "migration_debt_details": self.migration_debt_details[
                :MAX_MIGRATION_DEBT_DETAILS
            ],
            "migration_debt_details_truncated": len(self.migration_debt_details)
            > MAX_MIGRATION_DEBT_DETAILS,
            "exempt": [
                {"root": root, "reason": reason}
                for root, reason in sorted(self.exempt_roots.items())
            ],
            "classification_counts": {
                "sample_conforming": sample_conforming_count,
                "sample_violating": len(sample_violation_roots),
                "migration_debt": len(self.migration_debt_roots),
                "exempt": len(self.exempt_roots),
            },
        }


def audit_plugin_skeleton_conformance(repo_root: Path) -> PluginSkeletonAudit:
    plugin_workspace = repo_root / "zircon_plugins"
    expected_roots = expected_plugin_manifest_roots(plugin_workspace)
    module_paths = workspace_module_paths_by_root(plugin_workspace)
    (
        core_workspace_dependency_count,
        core_workspace_dependency_violations,
    ) = collect_core_workspace_dependency_state(plugin_workspace)

    sample_roots = [root for root in SKELETON_SAMPLE_ROOTS if root in expected_roots]
    sample_violations: list[str] = []
    migration_debt_roots: list[str] = []
    migration_debt_details: list[str] = []

    for root in expected_roots:
        violations = collect_plugin_root_skeleton_violations(
            plugin_workspace,
            root,
            module_paths.get(root, []),
        )
        if root in sample_roots:
            sample_violations.extend(violations)
            continue
        if root in SKELETON_EXEMPT_ROOTS:
            continue
        if violations:
            migration_debt_roots.append(root)
            migration_debt_details.extend(violations)

    return PluginSkeletonAudit(
        sample_roots=sample_roots,
        sample_violation_details=sample_violations,
        core_workspace_dependency_count=core_workspace_dependency_count,
        core_workspace_dependency_violation_details=core_workspace_dependency_violations,
        migration_debt_roots=migration_debt_roots,
        migration_debt_details=migration_debt_details,
        exempt_roots={
            root: reason
            for root, reason in SKELETON_EXEMPT_ROOTS.items()
            if root in expected_roots
        },
    )


def workspace_module_paths_by_root(plugin_workspace: Path) -> dict[str, list[str]]:
    cargo_manifest = tomllib.loads(
        (plugin_workspace / "Cargo.toml").read_text(encoding="utf-8")
    )
    members = cargo_manifest.get("workspace", {}).get("members", [])
    module_paths: dict[str, list[str]] = {}
    for member in members:
        parts = PurePosixPath(member).parts
        if not parts or parts[0] in SKIPPED_WORKSPACE_ROOTS:
            continue
        if parts[0] == "asset_importers":
            if len(parts) >= 2:
                root = f"{parts[0]}/{parts[1]}"
            else:
                continue
        else:
            root = parts[0]
        module_paths.setdefault(root, []).append(member)
    return {
        root: sorted(paths)
        for root, paths in sorted(module_paths.items(), key=lambda item: item[0])
    }


def collect_core_workspace_dependency_state(
    plugin_workspace: Path,
) -> tuple[int, list[str]]:
    cargo_manifest = tomllib.loads(
        (plugin_workspace / "Cargo.toml").read_text(encoding="utf-8")
    )
    workspace = cargo_manifest.get("workspace", {})
    workspace_dependencies = (
        workspace.get("dependencies", {}) if isinstance(workspace, dict) else {}
    )
    violations: list[str] = []
    dependency_count = 0
    if not isinstance(workspace_dependencies, dict):
        violations.append(
            "zircon_plugins/Cargo.toml: [workspace.dependencies] must be a table"
        )
        workspace_dependencies = {}
    for dependency_name in CORE_WORKSPACE_DEPENDENCIES:
        spec = workspace_dependencies.get(dependency_name)
        if not isinstance(spec, dict) or "path" not in spec:
            violations.append(
                "zircon_plugins/Cargo.toml: "
                f"[workspace.dependencies].{dependency_name} must declare the root path"
            )

    members = workspace.get("members", []) if isinstance(workspace, dict) else []
    for member in members:
        if not isinstance(member, str) or not member.strip():
            continue
        cargo_toml = plugin_workspace / Path(PurePosixPath(member)) / "Cargo.toml"
        manifest = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
        for section, dependencies in dependency_tables(manifest):
            for dependency_name in CORE_WORKSPACE_DEPENDENCIES:
                spec = dependencies.get(dependency_name)
                if spec is None:
                    continue
                dependency_count += 1
                if not isinstance(spec, dict) or spec.get("workspace") is not True:
                    violations.append(
                        f"{member}: {section}.{dependency_name} must use "
                        "`workspace = true`"
                    )
                    continue
                if "path" in spec:
                    violations.append(
                        f"{member}: {section}.{dependency_name} must not repeat `path`"
                    )
    return dependency_count, violations


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


def collect_plugin_root_skeleton_violations(
    plugin_workspace: Path,
    root: str,
    module_paths: list[str],
) -> list[str]:
    violations: list[str] = []
    root_path = plugin_workspace / Path(root)
    if not (root_path / "plugin.toml").exists():
        violations.append(f"{root}: missing root plugin.toml")
    if not module_paths:
        violations.append(f"{root}: missing workspace module crate")
        return violations

    for module_path in module_paths:
        collect_module_skeleton_violations(plugin_workspace, root, module_path, violations)
    return violations


def collect_module_skeleton_violations(
    plugin_workspace: Path,
    root: str,
    module_path: str,
    violations: list[str],
) -> None:
    module_parts = PurePosixPath(module_path).parts
    module_kind = module_parts[-1] if module_parts else ""
    module_root = plugin_workspace / Path(module_path)
    cargo_toml = module_root / "Cargo.toml"
    lib_rs = module_root / "src" / "lib.rs"
    if not cargo_toml.exists():
        violations.append(f"{root}: {module_path} missing Cargo.toml")
    if not lib_rs.exists():
        violations.append(f"{root}: {module_path} missing src/lib.rs")
        return
    if root in SKELETON_SAMPLE_ROOTS:
        collect_sample_workspace_dependency_violations(
            cargo_toml,
            root,
            module_path,
            violations,
        )
    if module_kind not in OWNER_MODULE_KINDS:
        return

    plugin_rs = module_root / "src" / "plugin.rs"
    capability_rs = module_root / "src" / "capability.rs"
    if not plugin_rs.exists():
        violations.append(f"{root}: {module_path} missing src/plugin.rs")
    if not capability_rs.exists():
        violations.append(f"{root}: {module_path} missing src/capability.rs")

    lib_text = lib_rs.read_text(encoding="utf-8")
    for declaration in ("mod plugin;", "mod capability;"):
        if declaration not in lib_text:
            violations.append(f"{root}: {module_path}/src/lib.rs missing `{declaration}`")
    for forbidden in (
        "fn register_editor_extensions",
        "fn register(",
        "PluginPackageManifest::new",
    ):
        if forbidden in lib_text:
            violations.append(
                f"{root}: {module_path}/src/lib.rs contains behavior `{forbidden}`"
            )


def collect_sample_workspace_dependency_violations(
    cargo_toml: Path,
    root: str,
    module_path: str,
    violations: list[str],
) -> None:
    required_dependencies = SAMPLE_WORKSPACE_DEPENDENCIES.get(module_path)
    if not required_dependencies:
        return
    manifest = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    dependencies = manifest.get("dependencies", {})
    for dependency_name in required_dependencies:
        spec = dependencies.get(dependency_name)
        if not isinstance(spec, dict) or spec.get("workspace") is not True:
            violations.append(
                f"{root}: {module_path} workspace dependency `{dependency_name}` must use `workspace = true`"
            )
            continue
        if "path" in spec:
            violations.append(
                f"{root}: {module_path} workspace dependency `{dependency_name}` must not repeat `path`"
            )
