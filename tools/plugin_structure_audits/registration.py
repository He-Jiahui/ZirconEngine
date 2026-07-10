from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path
from pathlib import PurePosixPath
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # Python 3.10 used by the WSL/CI validation lane.
    import tomli as tomllib


@dataclass(frozen=True)
class PluginRegistrationAudit:
    asset_importer_family_roots: list[str]
    split_importer_roots: list[str]
    runtime_plugin_descriptor_roots: list[str]
    runtime_plugin_descriptor_single_source_violations: list[str]
    runtime_registration_builder_roots: list[str]
    runtime_registration_builder_violations: list[str]
    global_free_function_registration_sites: list[str]
    registration_compatibility_shim_sites: list[str]
    free_function_registration_sites: list[str]
    registration_owner_files: list[str]
    trait_entry_files: list[str]
    split_importer_free_function_registration_sites: list[str]
    split_importer_registration_owner_files: list[str]
    split_importer_trait_entry_files: list[str]

    def to_json(self) -> dict[str, Any]:
        all_free_function_sites = [
            *self.free_function_registration_sites,
            *self.split_importer_free_function_registration_sites,
        ]
        all_registration_owner_files = [
            *self.registration_owner_files,
            *self.split_importer_registration_owner_files,
        ]
        return {
            "free_function_registration_sites": len(
                self.global_free_function_registration_sites
            ),
            "free_function_registration_site_details": (
                self.global_free_function_registration_sites
            ),
            "registration_compatibility_shim_sites": len(
                self.registration_compatibility_shim_sites
            ),
            "registration_compatibility_shim_site_details": (
                self.registration_compatibility_shim_sites
            ),
            "m3_hard_cut_gate_status": (
                "registration-hard-cut-clean"
                if not self.global_free_function_registration_sites
                and not self.registration_compatibility_shim_sites
                else "registration-compatibility-debt-present"
            ),
            "asset_importer_family_roots": self.asset_importer_family_roots,
            "split_importer_roots": self.split_importer_roots,
            "runtime_plugin_descriptor_roots": self.runtime_plugin_descriptor_roots,
            "runtime_plugin_descriptor_root_count": len(
                self.runtime_plugin_descriptor_roots
            ),
            "runtime_plugin_descriptor_single_source_violation_count": len(
                self.runtime_plugin_descriptor_single_source_violations
            ),
            "runtime_plugin_descriptor_single_source_violations": (
                self.runtime_plugin_descriptor_single_source_violations
            ),
            "frameworks_02_runtime_plugin_descriptor_status": (
                "runtime-plugin-descriptor-single-source-clean"
                if not self.runtime_plugin_descriptor_single_source_violations
                else "runtime-plugin-descriptor-single-source-debt-present"
            ),
            "runtime_registration_builder_roots": (
                self.runtime_registration_builder_roots
            ),
            "runtime_registration_builder_violation_count": len(
                self.runtime_registration_builder_violations
            ),
            "runtime_registration_builder_violations": (
                self.runtime_registration_builder_violations
            ),
            "m3_t2_runtime_registration_builder_status": (
                "runtime-registration-builder-clean"
                if not self.runtime_registration_builder_violations
                else "runtime-registration-builder-debt-present"
            ),
            "asset_importer_family_free_function_registration_sites": len(
                self.free_function_registration_sites
            ),
            "asset_importer_family_free_function_registration_site_details": (
                self.free_function_registration_sites
            ),
            "asset_importer_family_registration_owner_files": len(
                self.registration_owner_files
            ),
            "asset_importer_family_registration_owner_file_details": (
                self.registration_owner_files
            ),
            "asset_importer_family_trait_entry_files": self.trait_entry_files,
            "m3_t1_gate_status": (
                "family-single-entry-clean"
                if not self.free_function_registration_sites
                and not self.registration_owner_files
                else "family-registration-debt-present"
            ),
            "split_importer_free_function_registration_sites": len(
                self.split_importer_free_function_registration_sites
            ),
            "split_importer_free_function_registration_site_details": (
                self.split_importer_free_function_registration_sites
            ),
            "split_importer_registration_owner_files": len(
                self.split_importer_registration_owner_files
            ),
            "split_importer_registration_owner_file_details": (
                self.split_importer_registration_owner_files
            ),
            "split_importer_trait_entry_files": self.split_importer_trait_entry_files,
            "m3_split_importer_gate_status": (
                "split-importer-single-entry-clean"
                if not self.split_importer_free_function_registration_sites
                and not self.split_importer_registration_owner_files
                else "split-importer-registration-debt-present"
            ),
            "importer_free_function_registration_sites": len(all_free_function_sites),
            "importer_free_function_registration_site_details": all_free_function_sites,
            "importer_registration_owner_files": len(all_registration_owner_files),
            "importer_registration_owner_file_details": all_registration_owner_files,
            "m3_importer_gate_status": (
                "importer-single-entry-clean"
                if not all_free_function_sites and not all_registration_owner_files
                else "importer-registration-debt-present"
            ),
        }


SPLIT_IMPORTER_ROOTS = [
    "audio_importer",
    "gltf_importer",
    "obj_importer",
    "opus_importer",
    "shader_wgsl_importer",
    "texture_importer",
    "ui_document_importer",
]

D8_RUNTIME_REGISTRATION_ROOTS = ["animation", "physics", "net"]

RUNTIME_MODULE_REGISTRATION_CALL_PATTERN = re.compile(
    r"\.module\s*\(\s*PLUGIN_RUNTIME_MODULE_NAME\s*\)",
    re.DOTALL,
)
RUNTIME_PLUGIN_IMPL_PATTERN = re.compile(
    r"impl\s+(?:zircon_runtime::plugin::)?RuntimePlugin\s+for\s+"
)
EMBEDDED_MODULE_DESCRIPTOR_PATTERN = re.compile(r"\.with_module_descriptor\s*\(")
DIRECT_MODULE_REGISTRATION_PATTERN = re.compile(r"register_module\s*\(")
ROOT_REGISTRATION_MOD_PATTERN = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+registration\s*;"
)
ROOT_REGISTRATION_USE_PATTERN = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?use\s+"
    r"(?:(?:crate|self)::)?registration(?:::|\s*;)"
)


def audit_plugin_registration_conformance(repo_root: Path) -> PluginRegistrationAudit:
    plugin_workspace = repo_root / "zircon_plugins"
    family_root = plugin_workspace / "asset_importers"
    roots: list[str] = []
    free_function_sites: list[str] = []
    registration_owner_files: list[str] = []
    trait_entry_files: list[str] = []
    split_roots: list[str] = []
    split_free_function_sites: list[str] = []
    split_registration_owner_files: list[str] = []
    split_trait_entry_files: list[str] = []
    runtime_registration_roots: list[str] = []
    runtime_registration_violations: list[str] = []
    runtime_plugin_descriptor_roots: list[str] = []
    runtime_plugin_descriptor_violations: list[str] = []
    global_free_function_sites: list[str] = []
    registration_compatibility_sites: list[str] = []

    for runtime_src in plugin_runtime_workspace_source_roots(plugin_workspace):
        collect_public_register_function_sites(
            repo_root,
            runtime_src,
            global_free_function_sites,
        )
        audit_runtime_plugin_descriptor_single_source(
            repo_root,
            runtime_src,
            runtime_plugin_descriptor_roots,
            runtime_plugin_descriptor_violations,
        )
    for source_root in plugin_workspace_source_roots(
        plugin_workspace,
        {"runtime", "editor"},
    ):
        collect_root_registration_compatibility_sites(
            repo_root,
            source_root,
            registration_compatibility_sites,
        )
    registration_compatibility_sites.sort()

    if family_root.exists():
        for child in sorted(path for path in family_root.iterdir() if path.is_dir()):
            root_name = f"asset_importers/{child.name}"
            roots.append(root_name)
            audit_runtime_src(
                repo_root,
                child / "runtime" / "src",
                free_function_sites,
                registration_owner_files,
                trait_entry_files,
            )

    for root_name in SPLIT_IMPORTER_ROOTS:
        child = plugin_workspace / root_name
        if not child.exists():
            continue
        split_roots.append(child.name)
        audit_runtime_src(
            repo_root,
            child / "runtime" / "src",
            split_free_function_sites,
            split_registration_owner_files,
            split_trait_entry_files,
        )

    for root_name in D8_RUNTIME_REGISTRATION_ROOTS:
        child = plugin_workspace / root_name
        if not child.exists():
            continue
        runtime_registration_roots.append(root_name)
        audit_runtime_registration_builder(
            repo_root,
            child / "runtime" / "src",
            runtime_registration_violations,
        )

    return PluginRegistrationAudit(
        asset_importer_family_roots=roots,
        split_importer_roots=split_roots,
        runtime_plugin_descriptor_roots=runtime_plugin_descriptor_roots,
        runtime_plugin_descriptor_single_source_violations=(
            runtime_plugin_descriptor_violations
        ),
        runtime_registration_builder_roots=runtime_registration_roots,
        runtime_registration_builder_violations=runtime_registration_violations,
        global_free_function_registration_sites=global_free_function_sites,
        registration_compatibility_shim_sites=registration_compatibility_sites,
        free_function_registration_sites=free_function_sites,
        registration_owner_files=registration_owner_files,
        trait_entry_files=trait_entry_files,
        split_importer_free_function_registration_sites=split_free_function_sites,
        split_importer_registration_owner_files=split_registration_owner_files,
        split_importer_trait_entry_files=split_trait_entry_files,
    )


def audit_runtime_plugin_descriptor_single_source(
    repo_root: Path,
    runtime_src: Path,
    audited_roots: list[str],
    violations: list[str],
) -> None:
    if not runtime_src.exists():
        return

    plugin_rs = runtime_src / "plugin.rs"
    if not plugin_rs.exists():
        return
    plugin_source = plugin_rs.read_text(encoding="utf-8")
    if not RUNTIME_PLUGIN_IMPL_PATTERN.search(plugin_source):
        return

    production_files = [plugin_rs]
    runtime_plugin_rs = runtime_src / "runtime_plugin.rs"
    if runtime_plugin_rs.exists():
        production_files.append(runtime_plugin_rs)
    runtime_plugin_root = runtime_src / "runtime_plugin"
    if runtime_plugin_root.exists():
        production_files.extend(sorted(runtime_plugin_root.rglob("*.rs")))
    sources = [
        (rust_file, rust_file.read_text(encoding="utf-8"))
        for rust_file in production_files
    ]
    combined_source = "\n".join(source for _, source in sources)

    root_path = runtime_src.relative_to(repo_root).as_posix()
    audited_roots.append(root_path)
    descriptor_count = len(
        EMBEDDED_MODULE_DESCRIPTOR_PATTERN.findall(combined_source)
    )
    if descriptor_count == 0:
        violations.append(f"{root_path}:missing:.with_module_descriptor(...)")
    elif descriptor_count != 1:
        violations.append(
            f"{root_path}:multiple:.with_module_descriptor(...):{descriptor_count}"
        )

    for rust_file, source in sources:
        for line_number, line in enumerate(source.splitlines(), start=1):
            if DIRECT_MODULE_REGISTRATION_PATTERN.search(line):
                path = rust_file.relative_to(repo_root).as_posix()
                violations.append(
                    f"{path}:{line_number}:stale:register_module(...)"
                )


def audit_runtime_src(
    repo_root: Path,
    runtime_src: Path,
    free_function_sites: list[str],
    registration_owner_files: list[str],
    trait_entry_files: list[str],
) -> None:
    if not runtime_src.exists():
        return
    registration_rs = runtime_src / "registration.rs"
    if registration_rs.exists():
        registration_owner_files.append(registration_rs.relative_to(repo_root).as_posix())
    plugin_rs = runtime_src / "plugin.rs"
    if plugin_rs.exists():
        trait_entry_files.append(plugin_rs.relative_to(repo_root).as_posix())
    collect_public_register_function_sites(
        repo_root,
        runtime_src,
        free_function_sites,
    )


def collect_public_register_function_sites(
    repo_root: Path,
    runtime_src: Path,
    free_function_sites: list[str],
) -> None:
    if not runtime_src.exists():
        return
    for rust_file in sorted(runtime_src.rglob("*.rs")):
        if "tests" in rust_file.relative_to(runtime_src).parts:
            continue
        for line_number, line in enumerate(
            rust_file.read_text(encoding="utf-8").splitlines(),
            start=1,
        ):
            if "pub fn register(" in line:
                free_function_sites.append(
                    f"{rust_file.relative_to(repo_root).as_posix()}:{line_number}"
                )


def plugin_runtime_workspace_source_roots(plugin_workspace: Path) -> list[Path]:
    return plugin_workspace_source_roots(plugin_workspace, {"runtime"})


def plugin_workspace_source_roots(
    plugin_workspace: Path,
    package_kinds: set[str],
) -> list[Path]:
    cargo_manifest = tomllib.loads(
        (plugin_workspace / "Cargo.toml").read_text(encoding="utf-8")
    )
    members = cargo_manifest.get("workspace", {}).get("members", [])
    roots: list[Path] = []
    for member in members:
        member_path = PurePosixPath(member)
        if not member_path.parts or member_path.parts[-1] not in package_kinds:
            continue
        roots.append(plugin_workspace / Path(*member_path.parts) / "src")
    return sorted(roots)


def collect_root_registration_compatibility_sites(
    repo_root: Path,
    source_root: Path,
    sites: list[str],
) -> None:
    if not source_root.exists():
        return
    for owner_file in (
        source_root / "registration.rs",
        source_root / "registration" / "mod.rs",
    ):
        if owner_file.exists():
            sites.append(
                f"{owner_file.relative_to(repo_root).as_posix()}:root-owner"
            )

    lib_rs = source_root / "lib.rs"
    if not lib_rs.exists():
        return
    for line_number, line in enumerate(
        lib_rs.read_text(encoding="utf-8").splitlines(),
        start=1,
    ):
        if ROOT_REGISTRATION_MOD_PATTERN.search(
            line
        ) or ROOT_REGISTRATION_USE_PATTERN.search(line):
            sites.append(f"{lib_rs.relative_to(repo_root).as_posix()}:{line_number}")


def audit_runtime_registration_builder(
    repo_root: Path,
    runtime_src: Path,
    violations: list[str],
) -> None:
    plugin_rs = runtime_src / "plugin.rs"
    runtime_system_rs = runtime_src / "runtime_system.rs"
    if not plugin_rs.exists():
        violations.append(f"{plugin_rs.relative_to(repo_root).as_posix()}:missing")
        return
    if not runtime_system_rs.exists():
        violations.append(
            f"{runtime_system_rs.relative_to(repo_root).as_posix()}:missing"
        )
        return

    plugin_source = plugin_rs.read_text(encoding="utf-8")
    runtime_system_source = runtime_system_rs.read_text(encoding="utf-8")
    plugin_path = plugin_rs.relative_to(repo_root).as_posix()
    runtime_system_path = runtime_system_rs.relative_to(repo_root).as_posix()

    builder_fragment = "RuntimePluginRegistrationBuilder::new(registry)"
    if builder_fragment not in plugin_source:
        violations.append(f"{plugin_path}:missing:{builder_fragment}")
    if not runtime_plugin_uses_registration_builder_module(plugin_source):
        violations.append(
            f"{plugin_path}:missing:.module(PLUGIN_RUNTIME_MODULE_NAME)"
        )

    if ".module(PLUGIN_RUNTIME_MODULE_NAME," in plugin_source:
        violations.append(
            f"{plugin_path}:stale:module builder descriptor argument"
        )

    for stale in ["intern_plugin_module(", "register_module("]:
        if stale in plugin_source:
            violations.append(f"{plugin_path}:stale:{stale}")

    required_runtime_fragments = [
        "RuntimePluginModuleRegistration",
        ".runtime_scene_system(",
    ]
    for fragment in required_runtime_fragments:
        if fragment not in runtime_system_source:
            violations.append(f"{runtime_system_path}:missing:{fragment}")

    for stale in [
        "PluginModuleId",
        "RuntimeExtensionRegistry,",
        "register_runtime_scene_system(",
        "intern_system_set(",
        "register_event::<",
    ]:
        if stale in runtime_system_source:
            violations.append(f"{runtime_system_path}:stale:{stale}")


def runtime_plugin_uses_registration_builder_module(plugin_source: str) -> bool:
    return bool(RUNTIME_MODULE_REGISTRATION_CALL_PATTERN.search(plugin_source))
