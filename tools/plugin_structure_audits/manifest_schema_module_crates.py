from __future__ import annotations

from pathlib import Path, PurePosixPath
from typing import Any, Iterable

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11 fallback.
    import tomli as tomllib  # type: ignore[no-redef]


ManifestEntry = tuple[str, dict[str, Any]]
WorkspaceCrateIndex = dict[str, dict[str, object]]


def collect_module_workspace_crate_violations(
    plugin_workspace: Path,
    manifests: Iterable[ManifestEntry],
    violations: list[str],
) -> None:
    collect_module_workspace_crate_violations_from_index(
        plugin_workspace,
        manifests,
        workspace_crate_index(plugin_workspace),
        violations,
    )


def collect_module_workspace_crate_violations_from_index(
    plugin_workspace: Path,
    manifests: Iterable[ManifestEntry],
    crate_index: WorkspaceCrateIndex,
    violations: list[str],
) -> None:
    for display_path, manifest in manifests:
        package_root = manifest_package_root(display_path, plugin_workspace.name)
        package_id = manifest.get("id")
        if not is_non_empty_trimmed_string(package_id):
            package_id = None
        collect_module_rows_workspace_crate_violations(
            display_path,
            "modules",
            manifest.get("modules"),
            package_root,
            crate_index,
            violations,
        )
        collect_optional_feature_module_workspace_crate_violations(
            display_path,
            package_root,
            package_id,
            manifest.get("optional_features"),
            crate_index,
            violations,
        )
        collect_module_rows_in_table_array_workspace_crate_violations(
            display_path,
            "feature_extensions",
            manifest.get("feature_extensions"),
            package_root,
            crate_index,
            violations,
        )


def collect_optional_feature_module_workspace_crate_violations(
    display_path: str,
    package_root: PurePosixPath,
    package_id: str | None,
    optional_features: object,
    crate_index: WorkspaceCrateIndex,
    violations: list[str],
) -> None:
    if not isinstance(optional_features, list):
        return
    for feature_index, feature in enumerate(optional_features):
        if not isinstance(feature, dict):
            continue
        feature_id = feature.get("id")
        if not is_non_empty_trimmed_string(feature_id):
            continue
        feature_root = optional_feature_root(package_root, package_id, feature_id)
        collect_module_rows_workspace_crate_violations(
            display_path,
            f"optional_features[{feature_index}].modules",
            feature.get("modules"),
            feature_root,
            crate_index,
            violations,
        )


def collect_module_rows_in_table_array_workspace_crate_violations(
    display_path: str,
    field_label: str,
    tables: object,
    expected_root: PurePosixPath,
    crate_index: WorkspaceCrateIndex,
    violations: list[str],
) -> None:
    if not isinstance(tables, list):
        return
    for table_index, table in enumerate(tables):
        if not isinstance(table, dict):
            continue
        collect_module_rows_workspace_crate_violations(
            display_path,
            f"{field_label}[{table_index}].modules",
            table.get("modules"),
            expected_root,
            crate_index,
            violations,
        )


def collect_module_rows_workspace_crate_violations(
    display_path: str,
    field_label: str,
    modules: object,
    expected_root: PurePosixPath,
    crate_index: WorkspaceCrateIndex,
    violations: list[str],
) -> None:
    if not isinstance(modules, list):
        return
    for module_index, module in enumerate(modules):
        if not isinstance(module, dict):
            continue
        crate_name = module.get("crate_name")
        if not is_non_empty_trimmed_string(crate_name):
            continue
        label = f"{field_label}[{module_index}].crate_name"
        collect_single_module_workspace_crate_violation(
            display_path,
            label,
            crate_name,
            expected_root,
            crate_index,
            violations,
        )


def collect_single_module_workspace_crate_violation(
    display_path: str,
    label: str,
    crate_name: str,
    expected_root: PurePosixPath,
    crate_index: WorkspaceCrateIndex,
    violations: list[str],
) -> None:
    workspace_crate = crate_index.get(crate_name)
    if workspace_crate is None:
        violations.append(
            f"{display_path}: {label} {crate_name} "
            "must be a zircon_plugins workspace member"
        )
        return
    member_text = workspace_crate_member_text(workspace_crate)
    if member_text is None:
        return
    if workspace_member_is_relative_to(member_text, expected_root):
        return
    violations.append(
        f"{display_path}: {label} {crate_name} workspace member {member_text} "
        f"must stay under {expected_root.as_posix()}"
    )


def workspace_crate_index(plugin_workspace: Path) -> WorkspaceCrateIndex:
    workspace_manifest = plugin_workspace / "Cargo.toml"
    if not workspace_manifest.exists():
        return {}
    try:
        workspace = tomllib.loads(workspace_manifest.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError:
        return {}
    members = workspace.get("workspace", {}).get("members", [])
    if not isinstance(members, list):
        return {}
    crates: WorkspaceCrateIndex = {}
    for member in members:
        if not is_safe_workspace_member(member):
            continue
        member_manifest = plugin_workspace / PurePosixPath(member) / "Cargo.toml"
        if not member_manifest.exists():
            continue
        try:
            crate_manifest = tomllib.loads(member_manifest.read_text(encoding="utf-8"))
        except tomllib.TOMLDecodeError:
            continue
        package = crate_manifest.get("package", {})
        crate_name = package.get("name") if isinstance(package, dict) else None
        if not is_non_empty_trimmed_string(crate_name):
            continue
        crates[crate_name] = {
            "member": member,
            "manifest_path": member_manifest,
        }
    return crates


def workspace_crate_member_text(workspace_crate: dict[str, object]) -> str | None:
    member = workspace_crate.get("member")
    if is_non_empty_trimmed_string(member):
        return member
    manifest_path = workspace_crate.get("manifest_path")
    if isinstance(manifest_path, Path):
        return manifest_path.parent.as_posix()
    return None


def manifest_package_root(display_path: str, workspace_name: str) -> PurePosixPath:
    manifest_path = PurePosixPath(display_path)
    package_root = manifest_path.parent
    if package_root.parts and package_root.parts[0] == workspace_name:
        return PurePosixPath(*package_root.parts[1:])
    return package_root


def optional_feature_root(
    package_root: PurePosixPath,
    package_id: str | None,
    feature_id: str,
) -> PurePosixPath:
    feature_suffix = (
        feature_id.removeprefix(f"{package_id}.")
        if package_id is not None and feature_id.startswith(f"{package_id}.")
        else feature_id
    )
    return package_root / "features" / PurePosixPath(feature_suffix.replace(".", "/"))


def workspace_member_is_relative_to(member_text: str, root: PurePosixPath) -> bool:
    member = PurePosixPath(member_text)
    if member == root:
        return True
    return len(member.parts) > len(root.parts) and member.parts[: len(root.parts)] == root.parts


def is_safe_workspace_member(value: object) -> bool:
    if not is_non_empty_trimmed_string(value):
        return False
    member = PurePosixPath(value)
    return not member.is_absolute() and ".." not in member.parts


def is_non_empty_trimmed_string(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value
