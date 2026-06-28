from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import re
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11 fallback.
    import tomli as tomllib  # type: ignore[no-redef]


FIRST_PARTY_RUNTIME_CAPABILITY_ROOTS = [
    "ai",
    "animation",
    "hybrid_gi",
    "navigation",
    "net",
    "particles",
    "physics",
    "prefab_tools",
    "rendering",
    "solari",
    "terrain",
    "texture",
    "tilemap_2d",
    "virtual_geometry",
    "zr_vm_language",
]

FIRST_PARTY_EDITOR_RUNTIME_MIRROR_ROOTS = [
    "animation",
    "physics",
    "net",
]


@dataclass(frozen=True)
class PluginCapabilityAudit:
    audited_runtime_roots: list[str]
    editor_runtime_mirror_roots: list[str]
    missing_capability_owner_files: list[str]
    missing_runtime_capability_exports: list[str]
    root_capability_mismatches: list[str]
    module_capability_mismatches: list[str]
    lib_capability_literal_sites: list[str]
    sdk_builder_mirror_violations: list[str]
    editor_runtime_mirror_violations: list[str]

    def to_json(self) -> dict[str, Any]:
        mismatch_details = [
            *self.missing_capability_owner_files,
            *self.missing_runtime_capability_exports,
            *self.root_capability_mismatches,
            *self.module_capability_mismatches,
            *self.lib_capability_literal_sites,
            *self.sdk_builder_mirror_violations,
        ]
        return {
            "audited_runtime_roots": self.audited_runtime_roots,
            "audited_runtime_root_count": len(self.audited_runtime_roots),
            "editor_runtime_mirror_roots": self.editor_runtime_mirror_roots,
            "editor_runtime_mirror_root_count": len(
                self.editor_runtime_mirror_roots
            ),
            "missing_capability_owner_files": len(
                self.missing_capability_owner_files
            ),
            "missing_capability_owner_file_details": (
                self.missing_capability_owner_files
            ),
            "missing_runtime_capability_exports": len(
                self.missing_runtime_capability_exports
            ),
            "missing_runtime_capability_export_details": (
                self.missing_runtime_capability_exports
            ),
            "root_capability_mismatches": len(self.root_capability_mismatches),
            "root_capability_mismatch_details": self.root_capability_mismatches,
            "module_capability_mismatches": len(self.module_capability_mismatches),
            "module_capability_mismatch_details": self.module_capability_mismatches,
            "lib_capability_literal_sites": len(self.lib_capability_literal_sites),
            "lib_capability_literal_site_details": self.lib_capability_literal_sites,
            "sdk_builder_mirror_violations": len(self.sdk_builder_mirror_violations),
            "sdk_builder_mirror_violation_details": self.sdk_builder_mirror_violations,
            "editor_runtime_mirror_violations": len(
                self.editor_runtime_mirror_violations
            ),
            "editor_runtime_mirror_violation_details": (
                self.editor_runtime_mirror_violations
            ),
            "capability_source_mismatches": len(mismatch_details),
            "capability_source_mismatch_details": mismatch_details,
            "m4_runtime_capability_gate_status": (
                "runtime-capability-single-source-clean"
                if not self.runtime_capability_mismatch_details()
                else "runtime-capability-source-debt-present"
            ),
            "m4_t2_builder_mirror_gate_status": (
                "sdk-builder-mirror-clean"
                if not self.sdk_builder_mirror_violations
                else "sdk-builder-mirror-debt-present"
            ),
            "d9_editor_runtime_mirror_gate_status": (
                "editor-runtime-mirror-clean"
                if not self.editor_runtime_mirror_violations
                else "editor-runtime-mirror-debt-present"
            ),
        }

    def runtime_capability_mismatch_details(self) -> list[str]:
        return [
            *self.missing_capability_owner_files,
            *self.missing_runtime_capability_exports,
            *self.root_capability_mismatches,
            *self.module_capability_mismatches,
            *self.lib_capability_literal_sites,
        ]


def audit_plugin_capability_conformance(repo_root: Path) -> PluginCapabilityAudit:
    plugin_workspace = repo_root / "zircon_plugins"
    missing_capability_owner_files: list[str] = []
    missing_runtime_capability_exports: list[str] = []
    root_capability_mismatches: list[str] = []
    module_capability_mismatches: list[str] = []
    lib_capability_literal_sites: list[str] = []

    for root in FIRST_PARTY_RUNTIME_CAPABILITY_ROOTS:
        runtime_src = plugin_workspace / root / "runtime" / "src"
        capability_path = runtime_src / "capability.rs"
        lib_path = runtime_src / "lib.rs"
        manifest_path = plugin_workspace / root / "plugin.toml"

        if not capability_path.exists():
            missing_capability_owner_files.append(
                capability_path.relative_to(repo_root).as_posix()
            )
            continue

        constants = parse_capability_string_constants(capability_path)
        runtime_capabilities = parse_runtime_capability_values(capability_path, constants)
        display_root = root

        if not runtime_capabilities:
            missing_runtime_capability_exports.append(
                f"{capability_path.relative_to(repo_root).as_posix()}: missing non-empty RUNTIME_CAPABILITIES"
            )

        manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
        root_capabilities = as_string_list(manifest.get("capabilities"))
        if root_capabilities != runtime_capabilities:
            root_capability_mismatches.append(
                f"{manifest_path.relative_to(repo_root).as_posix()}: root capabilities {root_capabilities!r} != capability.rs RUNTIME_CAPABILITIES {runtime_capabilities!r}"
            )

        runtime_module = runtime_module_for_root(manifest, root)
        module_capabilities = as_string_list(runtime_module.get("capabilities"))
        if module_capabilities != runtime_capabilities:
            module_capability_mismatches.append(
                f"{manifest_path.relative_to(repo_root).as_posix()}: runtime module capabilities {module_capabilities!r} != capability.rs RUNTIME_CAPABILITIES {runtime_capabilities!r}"
            )

        collect_lib_capability_literal_sites(
            repo_root,
            lib_path,
            display_root,
            lib_capability_literal_sites,
        )

    return PluginCapabilityAudit(
        audited_runtime_roots=FIRST_PARTY_RUNTIME_CAPABILITY_ROOTS,
        editor_runtime_mirror_roots=FIRST_PARTY_EDITOR_RUNTIME_MIRROR_ROOTS,
        missing_capability_owner_files=missing_capability_owner_files,
        missing_runtime_capability_exports=missing_runtime_capability_exports,
        root_capability_mismatches=root_capability_mismatches,
        module_capability_mismatches=module_capability_mismatches,
        lib_capability_literal_sites=lib_capability_literal_sites,
        sdk_builder_mirror_violations=collect_sdk_builder_mirror_violations(
            repo_root
        ),
        editor_runtime_mirror_violations=collect_editor_runtime_mirror_violations(
            repo_root
        ),
    )


def parse_capability_string_constants(capability_path: Path) -> dict[str, str]:
    constants: dict[str, str] = {}
    text = capability_path.read_text(encoding="utf-8")
    for match in re.finditer(
        r"pub\s+const\s+([A-Z0-9_]*CAPABILITY)\s*:\s*&str\s*=\s*\"([^\"]+)\"\s*;",
        text,
        re.MULTILINE,
    ):
        constants[match.group(1)] = match.group(2)
    return constants


def parse_runtime_capability_values(
    capability_path: Path,
    constants: dict[str, str],
) -> list[str]:
    text = capability_path.read_text(encoding="utf-8")
    match = re.search(
        r"pub\s+const\s+RUNTIME_CAPABILITIES\s*:\s*&\[\s*&str\s*\]\s*=\s*&\[(.*?)\]\s*;",
        text,
        re.DOTALL,
    )
    if match is None:
        return []
    body = match.group(1)
    values: list[str] = []
    for token in body.split(","):
        name = token.strip()
        if not name:
            continue
        value = constants.get(name)
        if value is not None:
            values.append(value)
    return values


def as_string_list(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [entry for entry in value if isinstance(entry, str)]


def runtime_module_for_root(manifest: dict[str, Any], root: str) -> dict[str, Any]:
    expected_crate = f"zircon_plugin_{root}_runtime"
    for module in manifest.get("modules", []):
        if (
            isinstance(module, dict)
            and module.get("kind") == "runtime"
            and module.get("crate_name") == expected_crate
        ):
            return module
    return {}


def collect_lib_capability_literal_sites(
    repo_root: Path,
    lib_path: Path,
    root: str,
    literal_sites: list[str],
) -> None:
    if not lib_path.exists():
        return
    lib_text = lib_path.read_text(encoding="utf-8")
    lines = lib_text.splitlines()
    has_capability_mod = any(line.strip() == "mod capability;" for line in lines)
    has_runtime_export = exposes_runtime_capabilities_from_single_source(
        lib_path,
        lib_text,
    )
    if not has_capability_mod:
        literal_sites.append(
            f"{lib_path.relative_to(repo_root).as_posix()}: missing `mod capability;`"
        )
    if not has_runtime_export:
        literal_sites.append(
            f"{lib_path.relative_to(repo_root).as_posix()}: runtime_capabilities() is not projected from RUNTIME_CAPABILITIES"
        )

    for line_number, line in enumerate(lines, start=1):
        stripped = line.strip()
        if stripped.startswith("pub const ") and "CAPABILITY" in stripped and ": &str" in stripped:
            literal_sites.append(
                f"{lib_path.relative_to(repo_root).as_posix()}:{line_number}: capability string constant belongs in capability.rs"
            )
        if (
            root in {"prefab_tools", "terrain", "tilemap_2d"}
            and 'with_required_capabilities(["runtime.' in stripped
        ):
            literal_sites.append(
                f"{lib_path.relative_to(repo_root).as_posix()}:{line_number}: importer capability literal belongs in capability.rs"
            )


def exposes_runtime_capabilities_from_single_source(
    lib_path: Path,
    lib_text: str,
) -> bool:
    if "pub fn runtime_capabilities()" in lib_text and "RUNTIME_CAPABILITIES" in lib_text:
        return True

    plugin_path = lib_path.parent / "plugin.rs"
    if not plugin_path.exists():
        return False
    if not re.search(
        r"pub\s+use\s+plugin::\s*(?:\{[^}]*runtime_capabilities[^}]*\}|runtime_capabilities)",
        lib_text,
        re.DOTALL,
    ):
        return False

    plugin_text = plugin_path.read_text(encoding="utf-8")
    return (
        "pub fn runtime_capabilities()" in plugin_text
        and "RUNTIME_CAPABILITIES" in plugin_text
    )


def collect_sdk_builder_mirror_violations(repo_root: Path) -> list[str]:
    violations: list[str] = []
    sdk_src = repo_root / "zircon_plugins" / "plugin_sdk" / "src"
    required_file_patterns = {
        sdk_src / "manifest" / "feature_bundle_builder.rs": [
            "pub struct PluginFeatureBundleBuilder",
            "pub fn with_runtime_capability_module",
            "pub fn with_editor_capability_module",
        ],
        sdk_src / "manifest" / "mod.rs": [
            "pub use feature_bundle_builder::PluginFeatureBundleBuilder",
        ],
        sdk_src / "lib.rs": [
            "PluginFeatureBundleBuilder",
        ],
        sdk_src / "prelude.rs": [
            "PluginFeatureBundleBuilder",
        ],
        sdk_src / "editor.rs": [
            "pub fn mirrors_runtime(",
            "pub fn mirrors_runtime_manifest(",
            "pub fn mirrored_runtime_package_id(",
            "mirrors_runtime: $runtime_declaration:expr",
            "editor_declaration_mirrors_runtime_manifest_and_keeps_editor_capabilities",
        ],
        sdk_src / "manifest" / "tests.rs": [
            "feature_bundle_builder_projects_capability_to_feature_and_modules",
        ],
    }
    for path, patterns in required_file_patterns.items():
        if not path.exists():
            violations.append(
                f"{path.relative_to(repo_root).as_posix()}: missing SDK builder/mirror owner"
            )
            continue
        text = path.read_text(encoding="utf-8")
        for pattern in patterns:
            if pattern not in text:
                violations.append(
                    f"{path.relative_to(repo_root).as_posix()}: missing `{pattern}`"
                )
    return violations


def collect_editor_runtime_mirror_violations(repo_root: Path) -> list[str]:
    violations: list[str] = []
    plugin_workspace = repo_root / "zircon_plugins"
    for root in FIRST_PARTY_EDITOR_RUNTIME_MIRROR_ROOTS:
        editor_root = plugin_workspace / root / "editor"
        plugin_path = editor_root / "src" / "plugin.rs"
        cargo_path = editor_root / "Cargo.toml"
        runtime_crate = f"zircon_plugin_{root}_runtime"

        if not plugin_path.exists():
            violations.append(
                f"{plugin_path.relative_to(repo_root).as_posix()}: missing editor plugin owner"
            )
            continue

        plugin_text = plugin_path.read_text(encoding="utf-8")
        if "EditorPluginDeclaration" not in plugin_text:
            violations.append(
                f"{plugin_path.relative_to(repo_root).as_posix()}: editor plugin must be declared through EditorPluginDeclaration"
            )
        expected_manifest_mirror = (
            f".mirrors_runtime_manifest({runtime_crate}::package_manifest())"
        )
        expected_manifest_macro = (
            f"mirrors_runtime_manifest: {runtime_crate}::package_manifest()"
        )
        expected_declaration_mirror = f".mirrors_runtime(&{runtime_crate}::"
        if (
            expected_manifest_mirror not in plugin_text
            and expected_manifest_macro not in plugin_text
            and expected_declaration_mirror not in plugin_text
        ):
            violations.append(
                f"{plugin_path.relative_to(repo_root).as_posix()}: missing explicit SDK mirror to `{runtime_crate}`"
            )
        if "EditorPluginRegistrationReport::from_plugin(" in plugin_text:
            violations.append(
                f"{plugin_path.relative_to(repo_root).as_posix()}: registration must flow through EditorPluginDeclaration::registration_report"
            )

        sdk_dependency = load_dependency(cargo_path, "zircon_plugin_sdk")
        if not (
            isinstance(sdk_dependency, dict)
            and sdk_dependency.get("workspace") is True
            and "editor" in as_string_list(sdk_dependency.get("features"))
        ):
            violations.append(
                f"{cargo_path.relative_to(repo_root).as_posix()}: missing workspace zircon_plugin_sdk dependency with editor feature"
            )

        tests_text = collect_editor_test_text(editor_root / "src")
        if "mirrored_runtime_package_id()" not in tests_text:
            violations.append(
                f"{(editor_root / 'src').relative_to(repo_root).as_posix()}: tests must assert mirrored_runtime_package_id()"
            )
        if runtime_crate not in tests_text or ".package_manifest" not in tests_text:
            violations.append(
                f"{(editor_root / 'src').relative_to(repo_root).as_posix()}: tests must assert mirrored runtime package capabilities"
            )
    return violations


def load_dependency(cargo_path: Path, name: str) -> Any:
    if not cargo_path.exists():
        return None
    manifest = tomllib.loads(cargo_path.read_text(encoding="utf-8"))
    dependencies = manifest.get("dependencies")
    if not isinstance(dependencies, dict):
        return None
    return dependencies.get(name)


def collect_editor_test_text(src_root: Path) -> str:
    test_files: list[Path] = []
    tests_rs = src_root / "tests.rs"
    if tests_rs.exists():
        test_files.append(tests_rs)
    tests_dir = src_root / "tests"
    if tests_dir.exists():
        test_files.extend(sorted(tests_dir.rglob("*.rs")))
    return "\n".join(path.read_text(encoding="utf-8") for path in test_files)
