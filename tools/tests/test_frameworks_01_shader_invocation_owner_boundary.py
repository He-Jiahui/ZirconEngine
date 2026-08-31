import re
import subprocess
import tomllib
import unittest
import uuid
from pathlib import Path

from tools.runtime_domain_dependency_audit import _rust_code_view, _rust_use_paths


REPO_ROOT = Path(__file__).resolve().parents[2]
FRAMEWORK_SHADER_ROOT = REPO_ROOT / "zircon_runtime/src/core/framework/render/shader"
FRAMEWORK_RENDER_MOD = REPO_ROOT / "zircon_runtime/src/core/framework/render/mod.rs"
INVOCATION_ROOT = REPO_ROOT / "zircon_runtime/src/graphics/shader/invocation"
GRAPHICS_SHADER_MOD = REPO_ROOT / "zircon_runtime/src/graphics/shader/mod.rs"
INVOCATION_FIXTURE_ROOT = REPO_ROOT / "zircon_runtime/tests/fixtures/shader_invocation"
OLD_BUILDER_NAMES = {"ComputeDispatchBuilder", "FullscreenPassBuilder"}
REQUIRED_INVOCATION_FILES = {
    "compiler.rs",
    "diagnostic.rs",
    "mod.rs",
    "parameter_packer.rs",
    "resource_bindings.rs",
}
FORBIDDEN_FRAMEWORK_BEHAVIOR = (
    ("ComputeDispatchBuilder declaration", re.compile(r"\bstruct\s+ComputeDispatchBuilder\b")),
    ("FullscreenPassBuilder declaration", re.compile(r"\bstruct\s+FullscreenPassBuilder\b")),
    ("named resource validator", re.compile(r"\bfn\s+validate_named_resource_bindings\b")),
    ("entry point validator", re.compile(r"\bfn\s+validate_shader_entry_point\b")),
    ("fullscreen parameter encoder", re.compile(r"\bfn\s+fullscreen_parameter_words\b")),
    ("parameter byte writer", re.compile(r"\bfn\s+write_parameter_bytes\b")),
)
INVOCATION_FIXTURE_SPECS = {
    "compute_binding_probe": {
        "kind": "compute",
        "entry": ("cs_main", "compute"),
        "workgroup_size": 64,
        "properties": ("element_count", "scale"),
        "resources": (
            ("params", "uniform_buffer", "read", 0, 0, "uniform", None, "ComputeBindingProbeParams"),
            ("input_values", "storage_buffer", "read", 0, 1, "storage", "read", "array<f32>"),
            ("output_values", "storage_buffer", "read_write", 0, 2, "storage", "read_write", "array<f32>"),
        ),
        "uniform": (
            "ComputeBindingProbeParams",
            (
                ("element_count", "u32", 0, 4),
                ("scale", "f32", 4, 4),
                ("_padding", "vec2<u32>", 8, 8),
            ),
            16,
        ),
    },
    "fullscreen_binding_probe": {
        "kind": "fullscreen",
        "entry": ("fs_main", "fragment"),
        "workgroup_size": None,
        "properties": ("tint", "exposure"),
        "resources": (
            ("source_texture", "texture", "read", 0, 0, None, None, "texture_2d<f32>"),
            ("source_sampler", "sampler", "read", 0, 1, None, None, "sampler"),
            ("params", "uniform_buffer", "read", 0, 2, "uniform", None, "FullscreenBindingProbeParams"),
        ),
        "uniform": (
            "FullscreenBindingProbeParams",
            (
                ("tint", "vec4<f32>", 0, 16),
                ("exposure", "f32", 16, 4),
                ("_padding_0", "f32", 20, 4),
                ("_padding_1", "f32", 24, 4),
                ("_padding_2", "f32", 28, 4),
            ),
            32,
        ),
    },
}
WGSL_HOST_SHAREABLE_TYPE_LAYOUTS = {
    "f32": (4, 4),
    "u32": (4, 4),
    "vec2<u32>": (8, 8),
    "vec4<f32>": (16, 16),
}


def round_up(alignment: int, value: int) -> int:
    return (value + alignment - 1) // alignment * alignment


def wgsl_struct_layout(fields: list[tuple[str, str]]) -> tuple[list[tuple[str, str, int, int]], int]:
    offset = 0
    struct_alignment = 1
    layout = []
    for name, type_name in fields:
        alignment, size = WGSL_HOST_SHAREABLE_TYPE_LAYOUTS[type_name]
        offset = round_up(alignment, offset)
        layout.append((name, type_name, offset, size))
        offset += size
        struct_alignment = max(struct_alignment, alignment)
    return layout, round_up(struct_alignment, offset)


def wgsl_resource_declarations(source: str) -> dict[str, tuple[int, int, str | None, str | None, str]]:
    declarations = {}
    binding_slots = set()
    pattern = re.compile(
        r"@group\(\s*(?P<group>\d+)\s*\)\s*"
        r"@binding\(\s*(?P<binding>\d+)\s*\)\s*"
        r"var(?:<\s*(?P<qualifier>[^>]+?)\s*>)?\s+"
        r"(?P<name>[A-Za-z_]\w*)\s*:\s*(?P<type>[^;]+?)\s*;",
        re.MULTILINE,
    )
    for match in pattern.finditer(source):
        qualifier = match.group("qualifier")
        qualifier_parts = [] if qualifier is None else [part.strip() for part in qualifier.split(",")]
        if len(qualifier_parts) > 2:
            raise AssertionError(f"unsupported WGSL variable qualifier: {qualifier}")
        address_space = qualifier_parts[0] if qualifier_parts else None
        access = qualifier_parts[1] if len(qualifier_parts) == 2 else None
        group = int(match.group("group"))
        binding = int(match.group("binding"))
        name = match.group("name")
        if name in declarations:
            raise AssertionError(f"duplicate WGSL resource name: {name}")
        if (group, binding) in binding_slots:
            raise AssertionError(f"duplicate WGSL binding slot: group={group}, binding={binding}")
        binding_slots.add((group, binding))
        declarations[name] = (
            group,
            binding,
            address_space,
            access,
            re.sub(r"\s+", "", match.group("type")),
        )
    return declarations


def product_builder_candidate_paths() -> list[Path]:
    result = subprocess.run(
        [
            "git",
            "grep",
            "-l",
            "--untracked",
            "-e",
            "ComputeDispatchBuilder",
            "-e",
            "FullscreenPassBuilder",
            "-e",
            "pub use",
            "--",
            ":(glob)zircon_runtime/**/*.rs",
            ":(glob)zircon_app/**/*.rs",
            ":(glob)zircon_editor/**/*.rs",
            ":(glob)zircon_plugins/**/*.rs",
        ],
        cwd=REPO_ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode not in {0, 1}:
        raise RuntimeError(result.stderr.strip() or "git grep failed")
    return [REPO_ROOT / path for path in result.stdout.splitlines()]


def old_builder_import_lines(source: str) -> list[int]:
    code_view = _rust_code_view(source)
    use_paths = _rust_use_paths(code_view)
    public_use_line_spans = [
        (
            code_view.count("\n", 0, match.start()) + 1,
            code_view.count("\n", 0, match.end()) + 1,
        )
        for match in re.finditer(
            r"\bpub(?:\s*\([^)]*\))?\s+use\b[^;]*;",
            code_view,
            re.DOTALL,
        )
    ]
    lines = set()
    aliases: dict[str, tuple[str, ...]] = {}
    old_render_roots = {
        ("crate", "core", "framework", "render"),
        ("zircon_runtime", "core", "framework", "render"),
    }
    old_roots = old_render_roots | {root + ("shader",) for root in old_render_roots}

    def resolve_alias(path: tuple[str, ...]) -> tuple[str, ...]:
        resolved = tuple(segment for segment in path if segment not in {"self", "super"})
        seen = set()
        while resolved and resolved[0] in aliases and resolved[0] not in seen:
            seen.add(resolved[0])
            resolved = aliases[resolved[0]] + resolved[1:]
        return resolved

    for _ in range(len(use_paths) + 1):
        changed = False
        for path, alias, _line in use_paths:
            resolved = resolve_alias(path)
            if not any(root[: len(resolved)] == resolved for root in old_roots):
                continue
            effective_alias = alias or resolved[-1]
            if aliases.get(effective_alias) != resolved:
                aliases[effective_alias] = resolved
                changed = True
        if not changed:
            break

    imports_old_owner_glob = False
    for path, _alias, line in use_paths:
        normalized = resolve_alias(path)
        if normalized and normalized[-1] in OLD_BUILDER_NAMES and normalized[:4] in {
            ("crate", "core", "framework", "render"),
            ("zircon_runtime", "core", "framework", "render"),
        }:
            lines.add(line)
        if normalized and normalized[-1] == "*" and normalized[:-1] in old_roots:
            imports_old_owner_glob = True
            public_statement = next(
                (start for start, end in public_use_line_spans if start <= line <= end),
                None,
            )
            if public_statement is not None:
                lines.add(public_statement)

    qualified = re.compile(
        r"\b(?:crate|zircon_runtime)\s*::\s*core\s*::\s*framework\s*::\s*render\s*"
        r"::(?:\s*shader\s*::\s*)?(?:ComputeDispatchBuilder|FullscreenPassBuilder)\b"
    )
    for reference in qualified.finditer(code_view):
        lines.add(code_view.count("\n", 0, reference.start()) + 1)

    for alias, target in aliases.items():
        for root in old_roots:
            if root[: len(target)] != target:
                continue
            suffix = root[len(target) :]
            alias_builder = re.compile(
                rf"\b{re.escape(alias)}\s*"
                + "".join(rf"::\s*{re.escape(segment)}\s*" for segment in suffix)
                + r"::\s*(?:ComputeDispatchBuilder|FullscreenPassBuilder)\b"
            )
            for reference in alias_builder.finditer(code_view):
                lines.add(code_view.count("\n", 0, reference.start()) + 1)

    if imports_old_owner_glob:
        unqualified_builder = re.compile(
            r"(?<![:\w])(?:ComputeDispatchBuilder|FullscreenPassBuilder)\b"
        )
        for reference in unqualified_builder.finditer(code_view):
            lines.add(code_view.count("\n", 0, reference.start()) + 1)
    return sorted(lines)


class Frameworks01ShaderInvocationOwnerBoundaryTests(unittest.TestCase):
    def test_product_fixtures_cover_compute_and_fullscreen_invocation_bindings(self) -> None:
        expected_files = {
            relative_path
            for package_name in INVOCATION_FIXTURE_SPECS
            for relative_path in (
                f"{package_name}.zmeta",
                f"{package_name}/{package_name}.zshader",
                f"{package_name}/{package_name}.wgsl",
            )
        }
        self.assertEqual(
            expected_files,
            {
                path.relative_to(INVOCATION_FIXTURE_ROOT).as_posix()
                for path in INVOCATION_FIXTURE_ROOT.rglob("*")
                if path.is_file()
            },
        )

        entry_uuids = set()
        for package_name, spec in INVOCATION_FIXTURE_SPECS.items():
            meta_path = INVOCATION_FIXTURE_ROOT / f"{package_name}.zmeta"
            package_root = INVOCATION_FIXTURE_ROOT / package_name
            descriptor_path = package_root / f"{package_name}.zshader"
            wgsl_path = package_root / f"{package_name}.wgsl"
            meta = tomllib.loads(meta_path.read_text(encoding="utf-8"))
            descriptor = tomllib.loads(descriptor_path.read_text(encoding="utf-8"))
            wgsl = wgsl_path.read_text(encoding="utf-8")

            package_url = f"res://shader_invocation/{package_name}"
            self.assertEqual(7, meta["format_version"])
            self.assertEqual(package_url, meta["url"])
            self.assertEqual("Shader", meta["asset_kind"])
            self.assertEqual("compound", meta["unit"])
            self.assertEqual("zircon.builtin.shader.zshader_package", meta["importer_id"])
            self.assertEqual(1, meta["importer_version"])
            self.assertEqual("ready", meta["preview_state"])
            self.assertEqual(2, descriptor["version"])
            self.assertEqual(spec["kind"], descriptor["kind"])
            self.assertEqual([wgsl_path.name], descriptor["wgsl_files"])
            self.assertEqual(
                list(spec["properties"]),
                [property_["name"] for property_ in descriptor["properties"]],
            )
            self.assertEqual(
                [resource[:3] for resource in spec["resources"]],
                [
                    (resource["name"], resource["kind"], resource["access"])
                    for resource in descriptor["resources"]
                ],
            )

            entry_name, entry_stage = spec["entry"]
            self.assertEqual(
                [{"name": entry_name, "stage": entry_stage, "file": wgsl_path.name}],
                descriptor["entry_points"],
            )
            entry_match = re.search(
                rf"(?P<attributes>(?:@[A-Za-z_]\w*(?:\([^)]*\))?\s*)+)"
                rf"fn\s+{re.escape(entry_name)}\b",
                wgsl,
            )
            self.assertIsNotNone(entry_match)
            attributes = entry_match.group("attributes")
            self.assertEqual(
                [entry_stage],
                re.findall(r"@(vertex|fragment|compute)\b", attributes),
            )
            self.assertEqual(
                1,
                len(re.findall(r"@(vertex|fragment|compute)\b", wgsl)),
            )
            if spec["workgroup_size"] is None:
                self.assertNotRegex(attributes, r"@workgroup_size\b")
            else:
                self.assertRegex(
                    attributes,
                    rf"@workgroup_size\(\s*{spec['workgroup_size']}\s*\)",
                )

            declarations = wgsl_resource_declarations(wgsl)
            self.assertEqual(
                {resource[0] for resource in spec["resources"]},
                set(declarations),
            )
            for (
                resource_name,
                _kind,
                _descriptor_access,
                group,
                binding,
                address_space,
                wgsl_access,
                wgsl_type,
            ) in spec["resources"]:
                self.assertEqual(
                    (group, binding, address_space, wgsl_access, wgsl_type),
                    declarations[resource_name],
                )

            struct_name, expected_layout, expected_size = spec["uniform"]
            struct_match = re.search(
                rf"struct\s+{re.escape(struct_name)}\s*\{{(?P<body>.*?)\}}",
                wgsl,
                re.DOTALL,
            )
            self.assertIsNotNone(struct_match)
            fields = re.findall(
                r"(?m)^\s*([A-Za-z_]\w*)\s*:\s*([^,]+?)\s*,\s*$",
                struct_match.group("body"),
            )
            actual_layout, actual_size = wgsl_struct_layout(fields)
            self.assertEqual(list(expected_layout), actual_layout)
            self.assertEqual(expected_size, actual_size)

            included_paths = {
                INVOCATION_FIXTURE_ROOT
                / uri.removeprefix("res://shader_invocation/")
                for uri in meta["included_files"]
            }
            self.assertEqual({descriptor_path, wgsl_path}, included_paths)
            self.assertEqual(meta["uuid"], meta["entries"][0]["uuid"])
            self.assertEqual(
                [
                    (package_url, "Shader"),
                    (f"{package_url}#zshader:{package_name}.zshader", "Data"),
                    (f"{package_url}#wgsl:{package_name}.wgsl", "Data"),
                ],
                [(entry["url"], entry["asset_kind"]) for entry in meta["entries"]],
            )
            for entry in meta["entries"]:
                parsed = uuid.UUID(entry["uuid"])
                self.assertEqual(4, parsed.version)
                self.assertNotIn(entry["uuid"], entry_uuids)
                entry_uuids.add(entry["uuid"])

        self.assertEqual(6, len(entry_uuids))

    def test_framework_shader_owner_contains_no_invocation_behavior(self) -> None:
        violations = []
        for path in sorted(FRAMEWORK_SHADER_ROOT.rglob("*.rs")):
            code_view = _rust_code_view(path.read_text(encoding="utf-8"))
            for label, pattern in FORBIDDEN_FRAMEWORK_BEHAVIOR:
                if pattern.search(code_view):
                    violations.append(f"{path.relative_to(REPO_ROOT).as_posix()}: {label}")
        self.assertEqual([], violations)

    def test_framework_facades_do_not_export_invocation_builders(self) -> None:
        violations = []
        paths = [FRAMEWORK_RENDER_MOD, *sorted(FRAMEWORK_SHADER_ROOT.rglob("*.rs"))]
        for path in paths:
            code_view = _rust_code_view(path.read_text(encoding="utf-8"))
            for builder in sorted(OLD_BUILDER_NAMES):
                if re.search(rf"\b{builder}\b", code_view):
                    violations.append(f"{path.relative_to(REPO_ROOT).as_posix()}: {builder}")
        self.assertEqual([], violations)

    def test_product_consumers_do_not_import_builders_from_old_owner(self) -> None:
        violations = []
        for path in product_builder_candidate_paths():
            source = path.read_text(encoding="utf-8")
            for line in old_builder_import_lines(source):
                violations.append(f"{path.relative_to(REPO_ROOT).as_posix()}:{line}")
        self.assertEqual([], violations)

    def test_graphics_invocation_owner_is_folder_backed(self) -> None:
        self.assertTrue(INVOCATION_ROOT.is_dir())
        self.assertLessEqual(
            REQUIRED_INVOCATION_FILES,
            {path.name for path in INVOCATION_ROOT.glob("*.rs")},
        )
        graphics_shader_mod = _rust_code_view(
            GRAPHICS_SHADER_MOD.read_text(encoding="utf-8")
        )
        self.assertRegex(graphics_shader_mod, r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+invocation\s*;")

    def test_old_builder_import_scanner_handles_grouped_and_qualified_paths(self) -> None:
        grouped = """
use crate::core::framework::render::{ComputeDispatchBuilder, ShaderAssetKind};
"""
        shader_qualified = """
type Builder = zircon_runtime::core::framework::render::shader::FullscreenPassBuilder;
"""
        new_owner = """
use crate::graphics::shader::invocation::ComputeDispatchBuilder;
"""
        module_alias = """
use crate::core::framework::render as old_render;
type Builder = old_render::shader::FullscreenPassBuilder;
"""
        chained_alias = """
use crate::core as engine_core;
use engine_core::framework::render as old_render;
type Builder = old_render::ComputeDispatchBuilder;
"""
        old_glob = """
use crate::core::framework::render::*;
fn make() -> ComputeDispatchBuilder { todo!() }
"""
        core_alias_qualified = """
use crate::core as engine_core;
type Builder = engine_core::framework::render::ComputeDispatchBuilder;
"""
        framework_alias_qualified = """
use crate::core::framework as fw;
type Builder = fw::render::FullscreenPassBuilder;
"""
        public_old_glob = """
pub use crate::core::framework::render::*;
"""
        multiline_public_old_glob = """
pub(crate) use zircon_runtime::core::framework::render::{
    *,
};
"""

        self.assertEqual([2], old_builder_import_lines(grouped))
        self.assertEqual([2], old_builder_import_lines(shader_qualified))
        self.assertEqual([], old_builder_import_lines(new_owner))
        self.assertEqual([3], old_builder_import_lines(module_alias))
        self.assertEqual([4], old_builder_import_lines(chained_alias))
        self.assertEqual([3], old_builder_import_lines(old_glob))
        self.assertEqual([3], old_builder_import_lines(core_alias_qualified))
        self.assertEqual([3], old_builder_import_lines(framework_alias_qualified))
        self.assertEqual([2], old_builder_import_lines(public_old_glob))
        self.assertEqual([2], old_builder_import_lines(multiline_public_old_glob))


if __name__ == "__main__":
    unittest.main()
