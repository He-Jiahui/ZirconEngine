from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class PluginRegistrationAudit:
    asset_importer_family_roots: list[str]
    split_importer_roots: list[str]
    runtime_registration_builder_roots: list[str]
    runtime_registration_builder_violations: list[str]
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
            "asset_importer_family_roots": self.asset_importer_family_roots,
            "split_importer_roots": self.split_importer_roots,
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
    r"\.module\s*\(\s*PLUGIN_RUNTIME_MODULE_NAME\s*,\s*"
    r"(?:module_descriptor\s*\(\s*\)|module_descriptor_with_manager\s*\()",
    re.DOTALL,
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
        runtime_registration_builder_roots=runtime_registration_roots,
        runtime_registration_builder_violations=runtime_registration_violations,
        free_function_registration_sites=free_function_sites,
        registration_owner_files=registration_owner_files,
        trait_entry_files=trait_entry_files,
        split_importer_free_function_registration_sites=split_free_function_sites,
        split_importer_registration_owner_files=split_registration_owner_files,
        split_importer_trait_entry_files=split_trait_entry_files,
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
            f"{plugin_path}:missing:.module(PLUGIN_RUNTIME_MODULE_NAME, module_descriptor())"
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
