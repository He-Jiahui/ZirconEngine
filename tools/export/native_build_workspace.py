"""NativeBuild TOML and workspace cdylib crate metadata helpers."""

from __future__ import annotations

from pathlib import Path
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python <3.11 fallback.
    import tomli as tomllib  # type: ignore[no-redef]


NATIVE_BUILD_CDYLIB_CRATE_TYPE = "cdylib"


def resolve_native_build_path(
    label: str,
    path: Path,
    diagnostics: list[str],
) -> Path | None:
    try:
        return path.resolve()
    except OSError as error:
        diagnostics.append(f"{label} {path} could not be resolved: {error}")
        return None


def native_dynamic_workspace_crate_index(
    workspace_manifest: Path,
    diagnostics: list[str],
) -> dict[str, dict[str, object]]:
    if not workspace_manifest.exists():
        diagnostics.append(
            f"native dynamic plugin workspace manifest {workspace_manifest} does not exist"
        )
        return {}
    workspace = read_toml(workspace_manifest, diagnostics)
    if workspace is None:
        return {}
    members = workspace.get("workspace", {}).get("members", [])
    if not isinstance(members, list):
        diagnostics.append("native dynamic plugin workspace members must be an array")
        return {}

    crates: dict[str, dict[str, object]] = {}
    plugins_root = workspace_manifest.parent
    for index, member in enumerate(members):
        member_label = f"native dynamic plugin workspace members[{index}]"
        if not isinstance(member, str):
            diagnostics.append(f"{member_label} must be a string")
            continue
        if not member.strip():
            diagnostics.append(f"{member_label} must be a non-empty string")
            continue
        if member.strip() != member:
            diagnostics.append(f"{member_label} must be a non-empty trimmed string")
            continue
        member_path = Path(member)
        if member_path.is_absolute() or ".." in member_path.parts:
            diagnostics.append(f"{member_label} must be a safe relative path")
            continue
        member_manifest = resolve_native_build_path(
            f"native dynamic workspace member {member} manifest",
            plugins_root / member_path / "Cargo.toml",
            diagnostics,
        )
        if member_manifest is None:
            continue
        crate_manifest = read_toml(member_manifest, diagnostics)
        if crate_manifest is None:
            continue
        package = crate_manifest.get("package", {})
        crate_name = package.get("name") if isinstance(package, dict) else None
        if not isinstance(crate_name, str):
            diagnostics.append(
                f"native dynamic crate manifest {member_manifest} package.name must be a string"
            )
            continue
        if not crate_name.strip():
            diagnostics.append(
                f"native dynamic crate manifest {member_manifest} package.name must be a non-empty string"
            )
            continue
        if crate_name.strip() != crate_name:
            diagnostics.append(
                f"native dynamic crate manifest {member_manifest} package.name must be a non-empty trimmed string"
            )
            continue
        lib = crate_manifest.get("lib", {})
        if not isinstance(lib, dict):
            diagnostics.append(
                f"native dynamic crate manifest {member_manifest} lib must be an object"
            )
            continue
        crate_types = lib.get("crate-type", [])
        if not isinstance(crate_types, list):
            diagnostics.append(
                f"native dynamic crate manifest {member_manifest} lib.crate-type must be an array"
            )
            continue
        if native_dynamic_crate_type_schema_invalid(
            member_manifest,
            crate_types,
            diagnostics,
        ):
            continue
        crates[crate_name] = {
            "member": member,
            "manifest_path": member_manifest,
            "crate_types": crate_types,
        }
    return crates


def native_dynamic_cdylib_crate_index(
    workspace_manifest: Path,
    diagnostics: list[str],
) -> dict[str, dict[str, object]]:
    return native_dynamic_cdylib_crate_index_from_workspace(
        native_dynamic_workspace_crate_index(workspace_manifest, diagnostics)
    )


def native_dynamic_cdylib_crate_index_from_workspace(
    crate_index: dict[str, dict[str, object]],
) -> dict[str, dict[str, object]]:
    return {
        crate_name: crate
        for crate_name, crate in crate_index.items()
        if NATIVE_BUILD_CDYLIB_CRATE_TYPE in crate.get("crate_types", [])
    }


def native_dynamic_crate_type_schema_invalid(
    member_manifest: Path,
    crate_types: list[object],
    diagnostics: list[str],
) -> bool:
    has_invalid_entry = False
    for index, crate_type in enumerate(crate_types):
        label = f"native dynamic crate manifest {member_manifest} lib.crate-type[{index}]"
        if not isinstance(crate_type, str):
            diagnostics.append(f"{label} must be a string")
            has_invalid_entry = True
            continue
        if not crate_type.strip():
            diagnostics.append(f"{label} must be a non-empty string")
            has_invalid_entry = True
            continue
        if crate_type.strip() != crate_type:
            diagnostics.append(f"{label} must be a non-empty trimmed string")
            has_invalid_entry = True
    return has_invalid_entry


def native_dynamic_source_cdylib_crate_name(
    plugin_manifest_path: Path,
    crate_index: dict[str, dict[str, object]],
    package_id: str,
    diagnostics: list[str],
) -> str | None:
    manifest = read_toml(plugin_manifest_path, diagnostics)
    if manifest is None:
        return None
    modules = manifest.get("modules", [])
    if modules is None:
        modules = []
    if not isinstance(modules, list):
        diagnostics.append(
            f"native dynamic package {package_id} plugin.toml modules must be an array"
        )
        return None
    crate_names: list[str] = []
    module_schema_invalid = False
    for index, module in enumerate(modules):
        if not isinstance(module, dict):
            diagnostics.append(
                f"native dynamic package {package_id} plugin.toml "
                f"modules[{index}] must be an object"
            )
            module_schema_invalid = True
            continue
        crate_name = module.get("crate_name")
        if crate_name is None:
            continue
        if not isinstance(crate_name, str):
            diagnostics.append(
                f"native dynamic package {package_id} plugin.toml "
                f"modules[{index}].crate_name must be a string"
            )
            module_schema_invalid = True
            continue
        if not crate_name.strip():
            diagnostics.append(
                f"native dynamic package {package_id} plugin.toml "
                f"modules[{index}].crate_name must be a non-empty string"
            )
            module_schema_invalid = True
            continue
        if crate_name.strip() != crate_name:
            diagnostics.append(
                f"native dynamic package {package_id} plugin.toml "
                f"modules[{index}].crate_name must be a non-empty trimmed string"
            )
            module_schema_invalid = True
            continue
        if crate_name in crate_index:
            crate_names.append(crate_name)
    if module_schema_invalid:
        return None
    crate_names = dedupe(crate_names)
    if len(crate_names) == 1:
        return crate_names[0]
    if len(crate_names) > 1:
        diagnostics.append(
            f"native dynamic package {package_id} declares multiple cdylib crates: "
            + ", ".join(crate_names)
        )
        return None
    diagnostics.append(
        f"native dynamic package {package_id} declares no cdylib crate in plugin.toml modules"
    )
    return None


def read_toml(path: Path, diagnostics: list[str]) -> dict[str, Any] | None:
    if not path.exists():
        diagnostics.append(f"TOML file {path} does not exist")
        return None
    if not path.is_file():
        diagnostics.append(f"TOML file {path} is not a file")
        return None
    try:
        with path.open("rb") as toml_file:
            payload = tomllib.load(toml_file)
    except tomllib.TOMLDecodeError as error:
        diagnostics.append(f"TOML file {path} could not be parsed: {error}")
        return None
    except OSError as error:
        diagnostics.append(f"TOML file {path} could not be read: {error}")
        return None
    if not isinstance(payload, dict):
        diagnostics.append(f"TOML file {path} must contain a table")
        return None
    return payload


def dedupe(values: list[str]) -> list[str]:
    result: list[str] = []
    seen: set[str] = set()
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        result.append(value)
    return result
