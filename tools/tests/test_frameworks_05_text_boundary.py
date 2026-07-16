from __future__ import annotations

import re
import tempfile
import tomllib
import unittest
from functools import lru_cache
from pathlib import Path

from tools.runtime_domain_dependency_audit import (
    _canonical_rust_identifier,
    _cfg_test_item_spans,
    _rust_code_view,
    _rust_use_paths,
    audit_runtime_domain_dependencies,
)


REPO_ROOT = Path(__file__).resolve().parents[2]
RUNTIME_SRC = REPO_ROOT / "zircon_runtime" / "src"
TEXT_CONSUMER_ROOTS = (
    RUNTIME_SRC,
    REPO_ROOT / "zircon_runtime" / "tests",
    REPO_ROOT / "zircon_editor" / "src",
    REPO_ROOT / "zircon_app" / "src",
    REPO_ROOT / "zircon_plugins",
)
FORBIDDEN_TEXT_BACKEND_CRATES = ("wgpu", "fontdb", "glyphon", "cosmic_text", "swash")
CANONICAL_GRAPHICS_TEXT_REFERENCE = re.compile(
    r"\bcrate\s*::\s*(?:r#)?graphics\s*::\s*(?:r#)?text\b"
)
SCOPED_TEXT_REFERENCE = re.compile(
    r"\b((?:r#)?[A-Za-z_][A-Za-z0-9_]*)\s*::\s*(?:r#)?text\b"
)
CRATE_ALIAS_TEXT_REFERENCE = re.compile(
    r"\bcrate\s*::\s*((?:r#)?[A-Za-z_][A-Za-z0-9_]*)"
    r"\s*::\s*(?:r#)?text\b"
)


def production_rust_sources(root: Path) -> list[Path]:
    sources: list[Path] = []
    for path in root.rglob("*.rs"):
        relative = path.relative_to(root)
        if "tests" in relative.parts:
            continue
        if path.name in {"test.rs", "tests.rs"}:
            continue
        if path.name.startswith("test_") or path.name.endswith("_tests.rs"):
            continue
        sources.append(path)
    return sources


def production_code_view(path: Path) -> str:
    source = path.read_text(encoding="utf-8")
    code_view = _rust_code_view(source)
    masked = list(code_view)
    for start, end in _cfg_test_item_spans(code_view, source):
        for offset in range(start, end):
            if masked[offset] not in {"\r", "\n"}:
                masked[offset] = " "
    return "".join(masked)


def display_path(path: Path) -> str:
    try:
        return path.relative_to(REPO_ROOT).as_posix()
    except ValueError:
        return path.as_posix()


def crate_root_aliases(root: Path) -> dict[str, tuple[str, ...]]:
    crate_root = root / "lib.rs"
    if not crate_root.is_file():
        return {}
    code_view = production_code_view(crate_root)
    use_statements = list(re.finditer(r"\buse\s+[^;]+;", code_view))
    brace_depth = 0
    cursor = 0
    root_imports: list[tuple[tuple[str, ...], str | None]] = []
    for statement in use_statements:
        while cursor < statement.start():
            if code_view[cursor] == "{":
                brace_depth += 1
            elif code_view[cursor] == "}":
                brace_depth = max(0, brace_depth - 1)
            cursor += 1
        if brace_depth == 0:
            root_imports.extend(
                (raw_path, explicit_alias)
                for raw_path, explicit_alias, _line in _rust_use_paths(
                    statement.group(0)
                )
            )

    declared_names = {
        explicit_alias or raw_path[-1]
        for raw_path, explicit_alias in root_imports
        if raw_path and raw_path[-1] != "*"
    }
    aliases: dict[str, tuple[str, ...]] = {}
    if "graphics" not in declared_names:
        aliases["graphics"] = ("crate", "graphics")

    pending = set(range(len(root_imports)))
    while pending:
        progressed = False
        for import_index in tuple(pending):
            raw_path, explicit_alias = root_imports[import_index]
            if not raw_path:
                pending.remove(import_index)
                continue
            if raw_path[0] == "crate":
                if len(raw_path) >= 2 and raw_path[1] == "graphics":
                    resolved_path = raw_path
                elif len(raw_path) >= 2 and raw_path[1] in declared_names:
                    alias_path = aliases.get(raw_path[1])
                    if alias_path is None:
                        continue
                    resolved_path = alias_path + raw_path[2:]
                else:
                    resolved_path = raw_path
            elif raw_path[0] == "self" and len(raw_path) >= 2:
                alias_path = aliases.get(raw_path[1])
                if alias_path is None:
                    continue
                resolved_path = alias_path + raw_path[2:]
            else:
                alias_path = aliases.get(raw_path[0])
                if alias_path is None:
                    continue
                resolved_path = alias_path + raw_path[1:]
            pending.remove(import_index)
            progressed = True
            if raw_path[-1] != "*":
                aliases[explicit_alias or raw_path[-1]] = resolved_path
        if not progressed:
            break
    return aliases


def old_graphics_text_owner_offenders(root: Path) -> list[str]:
    offenders: list[str] = []
    crate_root = root / "lib.rs"
    root_aliases = crate_root_aliases(root)
    for path in production_rust_sources(root):
        source = path.read_text(encoding="utf-8")
        code_view = production_code_view(path)
        use_statements = list(re.finditer(r"\buse\s+[^;]+;", code_view))
        scoped_text_matches = list(SCOPED_TEXT_REFERENCE.finditer(code_view))
        relevant_offsets = {
            statement.start() for statement in use_statements
        } | {match.start(1) for match in scoped_text_matches}
        scopes_by_offset: dict[int, tuple[int, ...]] = {}
        scope_stack: list[int] = []
        cursor = 0
        for offset in sorted(relevant_offsets):
            while cursor < offset:
                if code_view[cursor] == "{":
                    scope_stack.append(cursor)
                elif code_view[cursor] == "}" and scope_stack:
                    scope_stack.pop()
                cursor += 1
            scopes_by_offset[offset] = tuple(scope_stack)

        imports: list[tuple[tuple[str, ...], str | None, int, tuple[int, ...]]] = []
        for statement in use_statements:
            preceding_lines = code_view.count("\n", 0, statement.start())
            scope = scopes_by_offset[statement.start()]
            imports.extend(
                (raw_path, explicit_alias, preceding_lines + local_line, scope)
                for raw_path, explicit_alias, local_line in _rust_use_paths(
                    statement.group(0)
                )
            )

        imports_by_scope: dict[tuple[int, ...], list[int]] = {}
        for import_index, (_raw_path, _explicit_alias, _line, scope) in enumerate(
            imports
        ):
            imports_by_scope.setdefault(scope, []).append(import_index)
        all_scopes = set(imports_by_scope)
        all_scopes.update(scopes_by_offset[offset] for offset in relevant_offsets)
        for scope in tuple(all_scopes):
            all_scopes.update(scope[:depth] for depth in range(len(scope)))
        all_scopes.add(())

        declared_names_by_scope: dict[tuple[int, ...], set[str]] = {}
        for scope, import_indices in imports_by_scope.items():
            declared_names_by_scope[scope] = {
                explicit_alias or raw_path[-1]
                for import_index in import_indices
                for raw_path, explicit_alias, _line, _scope in [imports[import_index]]
                if raw_path and raw_path[-1] != "*"
            }

        environments: dict[tuple[int, ...], dict[str, tuple[str, ...]]] = {}
        resolved_imports: dict[int, tuple[str, ...]] = {}
        for scope in sorted(all_scopes, key=lambda item: (len(item), item)):
            parent_scope = scope[:-1] if scope else None
            aliases = (
                environments[parent_scope].copy() if parent_scope is not None else {}
            )
            if not scope and path == crate_root:
                aliases.update(root_aliases)
            declared_local_names = declared_names_by_scope.get(scope, set())
            for local_name in declared_local_names:
                aliases.pop(local_name, None)

            def resolve_import_path(
                raw_path: tuple[str, ...],
            ) -> tuple[str, ...] | None:
                if not raw_path:
                    return None
                if raw_path[0] == "crate":
                    if len(raw_path) >= 2 and raw_path[1] == "graphics":
                        return raw_path
                    if len(raw_path) < 2 or raw_path[1] not in root_aliases:
                        return raw_path
                    return root_aliases[raw_path[1]] + raw_path[2:]
                if raw_path[0] == "self":
                    if len(raw_path) < 2:
                        return None
                    alias_path = aliases.get(raw_path[1])
                    return None if alias_path is None else alias_path + raw_path[2:]
                alias_path = aliases.get(raw_path[0])
                return None if alias_path is None else alias_path + raw_path[1:]

            pending = set(imports_by_scope.get(scope, ()))
            while pending:
                progressed = False
                for import_index in tuple(pending):
                    raw_path, explicit_alias, _line_number, _scope = imports[
                        import_index
                    ]
                    if not raw_path:
                        pending.remove(import_index)
                        continue
                    resolved_path = resolve_import_path(raw_path)
                    if resolved_path is None:
                        continue
                    resolved_imports[import_index] = resolved_path
                    pending.remove(import_index)
                    progressed = True
                    if raw_path[-1] != "*":
                        aliases[explicit_alias or raw_path[-1]] = resolved_path
                if not progressed:
                    break
            environments[scope] = aliases

        canonical_import_lines: set[int] = set()
        for import_index, (
            _raw_path,
            _explicit_alias,
            line_number,
            _scope,
        ) in enumerate(imports):
            resolved_path = resolved_imports.get(import_index)
            if resolved_path is None:
                continue
            if resolved_path[:3] == ("crate", "graphics", "text"):
                canonical_import_lines.add(line_number)

        match_offsets = {
            match.start()
            for match in CANONICAL_GRAPHICS_TEXT_REFERENCE.finditer(code_view)
        }
        for match in CRATE_ALIAS_TEXT_REFERENCE.finditer(code_view):
            alias = _canonical_rust_identifier(match.group(1))
            if root_aliases.get(alias) == ("crate", "graphics"):
                match_offsets.add(match.start())
        for match in scoped_text_matches:
            alias = _canonical_rust_identifier(match.group(1))
            scope = scopes_by_offset[match.start(1)]
            if environments[scope].get(alias) == ("crate", "graphics"):
                match_offsets.add(match.start(1))
        reported_lines: set[int] = set()
        for offset in sorted(match_offsets):
            line_number = code_view.count("\n", 0, offset) + 1
            if line_number in reported_lines:
                continue
            reported_lines.add(line_number)
            line = source.splitlines()[line_number - 1].strip()
            offenders.append(
                f"{display_path(path)}:{line_number}: {line}"
            )
        for line_number in sorted(canonical_import_lines - reported_lines):
            line = source.splitlines()[line_number - 1].strip()
            offenders.append(f"{display_path(path)}:{line_number}: {line}")
    return offenders


@lru_cache(maxsize=1)
def runtime_text_public_symbols() -> frozenset[str]:
    text_root = RUNTIME_SRC / "text" / "mod.rs"
    code_view = _rust_code_view(text_root.read_text(encoding="utf-8"))
    return frozenset(
        _canonical_rust_identifier(raw_path[-1])
        for raw_path, _alias, _line in _rust_use_paths(code_view)
        if raw_path and raw_path[0] in {"model", "rich"}
    )


def stale_render_text_facade_imports(root: Path) -> list[str]:
    offenders: list[str] = []
    public_symbols = runtime_text_public_symbols()
    for path in root.rglob("*.rs"):
        source = path.read_text(encoding="utf-8")
        code_view = _rust_code_view(source)
        reported_lines: set[int] = set()
        for raw_path, _alias, line_number in _rust_use_paths(code_view):
            canonical_path = tuple(
                _canonical_rust_identifier(component) for component in raw_path
            )
            if not canonical_path or canonical_path[-1] not in public_symbols:
                continue
            runtime_local = canonical_path[:4] == (
                "crate",
                "core",
                "framework",
                "render",
            )
            workspace_external = canonical_path[:4] == (
                "zircon_runtime",
                "core",
                "framework",
                "render",
            )
            if not runtime_local and not workspace_external:
                continue
            if line_number in reported_lines:
                continue
            reported_lines.add(line_number)
            line = source.splitlines()[line_number - 1].strip()
            offenders.append(f"{display_path(path)}:{line_number}: {line}")
    return offenders


@lru_cache(maxsize=1)
def runtime_dependency_report() -> dict[str, object]:
    return audit_runtime_domain_dependencies(REPO_ROOT)


def domain_edge_offenders(source_domain: str, target_domain: str) -> list[str]:
    report = runtime_dependency_report()
    return [
        f"{reference['path']}:{reference['line']}: {reference['source']}"
        for reference in report["references"]
        if reference["source_domain"] == source_domain
        and reference["target_domain"] == target_domain
    ]


def forbidden_backend_crates(code_view: str) -> list[str]:
    imported_roots = {
        path[0]
        for path, _alias, _line in _rust_use_paths(code_view)
        if path
    }
    return [
        crate_name
        for crate_name in FORBIDDEN_TEXT_BACKEND_CRATES
        if crate_name in imported_roots
        or re.search(
            rf"\bextern\s+crate\s+{crate_name}\b|\b{crate_name}\s*::", code_view
        )
        is not None
    ]


class Frameworks05TextBoundaryTests(unittest.TestCase):
    def test_shared_text_contract_has_neutral_framework_owner(self) -> None:
        contract_root = RUNTIME_SRC / "core" / "framework" / "text"
        self.assertTrue(
            contract_root.is_dir(),
            "shared text contracts must be owned by core/framework/text",
        )

        contract_source = "\n".join(
            production_code_view(path)
            for path in production_rust_sources(contract_root)
        )
        self.assertIn("pub struct TextShapeRequest", contract_source)
        self.assertIn("pub struct TextShapeResult", contract_source)
        self.assertIn("pub struct TextLayoutMetrics", contract_source)
        self.assertIn("pub trait TextLayoutService", contract_source)
        self.assertIn("pub enum TextRenderMode", contract_source)
        report = runtime_dependency_report()
        contract_domain_edges = [
            reference
            for reference in report["references"]
            if reference["path"].startswith(
                "zircon_runtime/src/core/framework/text/"
            )
            and reference["target_domain"] in {"graphics", "ui"}
        ]
        self.assertEqual([], contract_domain_edges)
        self.assertEqual(
            [],
            forbidden_backend_crates(contract_source),
            "framework text contracts must remain backend-neutral",
        )

    def test_render_framework_does_not_reexport_text_compatibility_surface(self) -> None:
        render_root = RUNTIME_SRC / "core" / "framework" / "render"
        render_mod = production_code_view(render_root / "mod.rs")

        self.assertFalse(
            (render_root / "text").exists(),
            "text contracts must be moved, not mirrored below framework::render",
        )
        self.assertNotIn("pub mod text;", render_mod)
        self.assertIsNone(
            re.search(r"\bpub\s+use\b[^;]*\btext\b", render_mod),
            "render framework must not alias the neutral text contract owner",
        )

    def test_shared_text_implementation_is_not_owned_by_graphics(self) -> None:
        text_root = RUNTIME_SRC / "text"
        self.assertTrue(
            (text_root / "mod.rs").is_file(),
            "shared text implementation must have a runtime text owner",
        )
        self.assertFalse(
            (RUNTIME_SRC / "graphics" / "text").exists(),
            "graphics/text must be hard-moved instead of retained as a bridge",
        )

        runtime_lib = (RUNTIME_SRC / "lib.rs").read_text(encoding="utf-8")
        graphics_mod = (RUNTIME_SRC / "graphics" / "mod.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("pub mod text;", runtime_lib)
        self.assertNotIn("pub mod text;", graphics_mod)
        self.assertNotIn("pub use text::", graphics_mod)

        text_source = "\n".join(
            path.read_text(encoding="utf-8")
            for path in production_rust_sources(text_root)
        )
        self.assertNotIn("crate::graphics::text", text_source)
        self.assertNotIn("crate::ui::text", text_source)
        self.assertEqual([], domain_edge_offenders("text", "graphics"))
        self.assertEqual([], domain_edge_offenders("text", "ui"))

    def test_text_production_has_no_interface_ui_dependency(self) -> None:
        text_root = RUNTIME_SRC / "text"
        offenders = []
        for path in production_rust_sources(text_root):
            source = path.read_text(encoding="utf-8")
            code_view = production_code_view(path)
            for match in re.finditer(
                r"\bzircon_runtime_interface\s*::\s*(?:r#)?ui\b", code_view
            ):
                line_number = code_view.count("\n", 0, match.start()) + 1
                offenders.append(
                    f"{display_path(path)}:{line_number}: "
                    f"{source.splitlines()[line_number - 1].strip()}"
                )
        self.assertEqual(
            [],
            offenders,
            "runtime text production must use neutral text-owned/framework DTOs; UI adapts at its boundary",
        )

    def test_graphics_mounts_the_shared_text_transport_boundary(self) -> None:
        graphics_mod = production_code_view(RUNTIME_SRC / "graphics" / "mod.rs")
        transport_path = RUNTIME_SRC / "graphics" / "text_transport" / "mod.rs"
        ui_adapter = production_code_view(RUNTIME_SRC / "ui" / "text" / "adapter.rs")

        self.assertIn(
            "mod text_transport;",
            graphics_mod,
            "graphics-only builds must mount the shared Text/UI transport conversions",
        )
        self.assertTrue(
            transport_path.is_file(),
            "Text/UI transport conversions must have a graphics-owned, folder-backed boundary",
        )
        transport_source = production_code_view(transport_path)
        required_conversions = (
            "impl From<UiRichTextFormat> for RichTextFormat",
            "impl From<RichTextFormat> for UiRichTextFormat",
            "impl From<UiTextDirection> for TextDirection",
            "impl From<TextDirection> for UiTextDirection",
            "impl From<UiFrame> for TextFrame",
            "impl From<TextFrame> for UiFrame",
            "impl From<UiTextWritingMode> for TextWritingMode",
            "impl From<UiTextAlign> for TextAlign",
            "impl From<UiTextWrap> for TextWrap",
        )
        for conversion in required_conversions:
            self.assertIn(conversion, transport_source)
        self.assertIsNone(
            re.search(r"\bimpl\s+From\s*<", ui_adapter),
            "UI may consume shared Text transport conversions but must not own duplicate impls",
        )

        cargo_manifest = tomllib.loads(
            (REPO_ROOT / "zircon_runtime" / "Cargo.toml").read_text(encoding="utf-8")
        )
        features = cargo_manifest["features"]
        self.assertIn("text", features["graphics"])
        self.assertIn("graphics", features["ui"])

    def test_text_owner_implements_only_the_canonical_layout_service(self) -> None:
        text_root = RUNTIME_SRC / "text"
        production_source = "\n".join(
            production_code_view(path) for path in production_rust_sources(text_root)
        )
        public_surface = production_code_view(text_root / "mod.rs")

        self.assertRegex(
            production_source,
            r"\bimpl\s+(?:crate\s*::\s*core\s*::\s*framework\s*::\s*text\s*::\s*)?TextLayoutService\s+for\b",
            "runtime text owner must provide the production canonical TextLayoutService",
        )
        self.assertNotRegex(
            production_source,
            r"\bpub\s+(?:struct|trait)\s+(?:TextShapeRequest|TextShapingService)\b",
            "runtime text must not publish a second shaping contract",
        )
        self.assertNotRegex(
            public_surface,
            r"\bpub\s+use\b[^;]*\b(?:TextShapeRequest|TextShapingService)\b",
            "runtime text root must expose only the canonical framework contract",
        )
        self.assertNotIn(
            "pub mod shaping_service;",
            production_source,
            "runtime text must delete the duplicate shaping-service module",
        )
        shape_pool = production_code_view(text_root / "parallel" / "shape_pool.rs")
        self.assertNotRegex(
            shape_pool,
            r"\bshape_text\s*\(",
            "parallel prewarm must enter the same canonical TextLayoutService adapter as layout",
        )
        self.assertIn(
            "shape_request_through_canonical_service",
            shape_pool,
            "parallel prewarm must preserve canonical validation and typed fallback reporting",
        )

    def test_ui_and_graphics_consume_the_canonical_text_layout_service(self) -> None:
        ui_source = "\n".join(
            production_code_view(path)
            for path in production_rust_sources(RUNTIME_SRC / "ui")
        )
        graphics_source = "\n".join(
            production_code_view(path)
            for path in production_rust_sources(RUNTIME_SRC / "graphics")
        )

        self.assertIn(
            "SharedTextLayoutSession",
            ui_source,
            "UI production must enter the Text-owned canonical layout session",
        )
        for private_backend in ("BackendShapeRequest", "TextShapeRunProvider"):
            self.assertNotRegex(
                ui_source,
                rf"\b{private_backend}\b",
                f"UI production must not bypass canonical service through {private_backend}",
            )
        self.assertIn(
            "TextLayoutService",
            graphics_source,
            "graphics production must consume canonical TextLayoutService",
        )
        self.assertIn(
            "TextShapeRequest",
            graphics_source,
            "graphics production must construct canonical TextShapeRequest",
        )
        render_stats = production_code_view(
            RUNTIME_SRC / "core" / "framework" / "render" / "backend_types.rs"
        )
        stats_projection = production_code_view(
            RUNTIME_SRC
            / "graphics"
            / "runtime"
            / "render_framework"
            / "submit_frame_extract"
            / "update_stats"
            / "base_stats.rs"
        )
        self.assertIn("last_ui_text_layout_fallback_count", render_stats)
        self.assertIn("last_ui_text_layout_fallback_count", stats_projection)

        consumer_source = ui_source + "\n" + graphics_source
        self.assertNotRegex(
            consumer_source,
            r"\b(?:TextFontFaceHandle::from_raw|FontFaceId\s*\([^)]*\.(?:raw\s*\(\)|0\b)|InstancedFaceId\s*\([^)]*\.(?:raw\s*\(\)|0\b))",
            "font handles must resolve through Text owner generation validation",
        )
        self.assertNotRegex(
            consumer_source,
            r"\bfont_(?:instance_)?id\s*\.\s*map\s*\(\s*\|[^|]+\|[^)]*\.0\b",
            "graphics must preserve versioned Text handles instead of extracting backend IDs",
        )
        text_sdf_source = "\n".join(
            production_code_view(path)
            for path in production_rust_sources(RUNTIME_SRC / "text" / "sdf")
        )
        self.assertNotRegex(
            text_sdf_source,
            r"\.map\s*\(\s*(?:FontFaceId|InstancedFaceId)\s*\)",
            "Text SDF must resolve versioned handles through the owner registry",
        )

    def test_graphics_does_not_own_cpu_text_service_state(self) -> None:
        graphics_root = RUNTIME_SRC / "graphics"
        graphics_source = "\n".join(
            production_code_view(path) for path in production_rust_sources(graphics_root)
        )
        forbidden_field_patterns = (
            r"\bfont_database\s*:\s*FontDatabase\b",
            r"\bbitmap_raster_worker_pool\s*:\s*Option\s*<\s*TextRasterWorkerPool\s*>",
            r"\bbitmap_atlas\s*:\s*GlyphAtlasSet\b",
            r"\bbitmap_source_cache\s*:\s*NativeBitmapAtlasSourceCache\b",
            r"\bbitmap_retry_state\s*:\s*GlyphAtlasBitmapRetryFrameState\b",
            r"\btext_state\s*\.\s*(?:font_system|font_database|swash_cache|bitmap_source_cache|bitmap_retry_state|bitmap_atlas|bitmap_raster_worker_pool|bitmap_atlas_frame_index)\b",
            r"\bwith_sdf_font_database\b",
            r"\bSdfFontBakeCache\b",
            r"\bfont_bake\s*:\s*",
        )
        offenders = []
        for path in production_rust_sources(graphics_root):
            source = path.read_text(encoding="utf-8")
            code_view = production_code_view(path)
            for pattern in forbidden_field_patterns:
                for match in re.finditer(pattern, code_view):
                    line_number = code_view.count("\n", 0, match.start()) + 1
                    offenders.append(
                        f"{display_path(path)}:{line_number}: "
                        f"{source.splitlines()[line_number - 1].strip()}"
                    )
        self.assertEqual(
            [],
            offenders,
            "graphics may own GPU upload/render state, not CPU font/raster/atlas service state",
        )
        all_runtime_source = "\n".join(
            production_code_view(path) for path in production_rust_sources(RUNTIME_SRC)
        )
        for migration_surface in (
            "SdfAtlasBakePlanView",
            "with_sdf_test_backends",
            "measure_sdf_glyph",
            "sdf_text_decoration_metrics",
        ):
            self.assertNotIn(
                migration_surface,
                all_runtime_source,
                f"{migration_surface} must not preserve a migration-only owner bypass",
            )
        self.assertIn(
            "prepare_sdf_runs_cpu",
            graphics_source,
            "graphics must consume Text-owned batch CPU preparation",
        )

    def test_font_face_invalidation_precedes_sdf_generation_and_upload(self) -> None:
        text_system = production_code_view(
            RUNTIME_SRC
            / "graphics"
            / "scene"
            / "scene_renderer"
            / "ui"
            / "text.rs"
        )
        prepare_start = text_system.index("    pub(super) fn prepare(")
        prepare_end = text_system.index("    pub(super) fn render", prepare_start)
        prepare = text_system[prepare_start:prepare_end]

        invalidate = prepare.index("self.invalidate_font_faces()")
        sdf_plan = prepare.index("self.sdf_atlas.prepare")
        sdf_generation = prepare.index("generation_failures_for_plan")
        self.assertLess(
            invalidate,
            sdf_plan,
            "known font-face changes must invalidate CPU/GPU text caches before SDF planning",
        )
        self.assertLess(
            invalidate,
            sdf_generation,
            "known font-face changes must invalidate caches before SDF generation lookup",
        )
        self.assertIn(
            "self.sdf_atlas.invalidate_font_faces()",
            text_system,
            "font-face invalidation must make stable SDF atlas slots dirty again",
        )

    def test_runtime_production_has_no_graphics_text_references(self) -> None:
        self.assertEqual([], old_graphics_text_owner_offenders(RUNTIME_SRC))
        self.assertEqual([], domain_edge_offenders("ui", "graphics"))

    def test_runtime_does_not_import_text_models_through_render_facade(self) -> None:
        offenders = [
            offender
            for root in TEXT_CONSUMER_ROOTS
            for offender in stale_render_text_facade_imports(root)
        ]
        self.assertEqual(
            [],
            offenders,
            "workspace text model consumers must import the canonical runtime text owner",
        )

    def test_old_owner_guard_handles_aliases_and_ignores_non_code(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            source = root / "alias.rs"
            source.write_text(
                "use crate::{graphics as gfx};\n"
                "fn shape() { gfx::text::shape(); }\n"
                "use crate::graphics::{text as gt};\n"
                "fn shape_alias() { gt::shape(); }\n"
                "use crate::graphics::{nested::{A}, text as nested_text};\n"
                "fn shape_nested() { nested_text::shape(); }\n"
                "use crate::graphics as graphics_root;\n"
                "use graphics_root::{text as two_stage_text};\n"
                "fn shape_two_stage() { two_stage_text::shape(); }\n"
                "use g2::{text as forward_text};\n"
                "use gfx_root as g2;\n"
                "use crate::graphics as gfx_root;\n"
                "fn shape_forward_alias() { forward_text::shape(); }\n"
                "use crate::other::graphics;\n"
                "fn shape_canonical() { crate::graphics::text::shape(); }\n"
                "use crate::r#graphics::{r#text as raw_text};\n"
                "fn shape_raw_alias() { raw_text::shape(); }\n"
                "use crate::qualified_graphics;\n"
                "use crate::qualified_graphics::{text as crate_qualified_text};\n"
                "use self::qualified_graphics::{text as self_qualified_text};\n"
                "fn shape_qualified_aliases() {\n"
                "    crate_qualified_text::shape();\n"
                "    self_qualified_text::shape();\n"
                "}\n"
                "fn shape_root_alias_expr() {\n"
                "    crate::qualified_graphics::text::shape();\n"
                "}\n"
                '// crate::graphics::text::comment_only();\n'
                'const NOTE: &str = "crate::graphics::text::string_only";\n',
                encoding="utf-8",
            )
            (root / "lib.rs").write_text(
                "pub(crate) use crate::graphics as qualified_graphics;\n"
                "pub mod test_gfx { pub mod text {} }\n"
                "pub mod nested_test_gfx { pub mod text {} }\n"
                "pub mod joint_test_gfx { pub mod text {} }\n"
                '#[cfg(all(feature = "x", test))]\n'
                "use crate::graphics as test_gfx;\n"
                '#[cfg(all(test, any(feature = "x", feature = "y")))]\n'
                "use crate::graphics as nested_test_gfx;\n"
                '#[cfg(any(test, feature = "x"))]\n'
                "use crate::graphics as maybe_graphics;\n"
                '#[cfg(any(test, feature = "x"))]\n'
                '#[cfg(not(feature = "x"))]\n'
                "use crate::graphics as joint_test_gfx;\n"
                "fn shape_root_relative() { graphics::text::shape(); }\n"
                "use self::graphics as self_graphics;\n"
                "use self_graphics::{text as self_text};\n"
                "fn shape_self_graphics() { self_text::shape(); }\n"
                "fn shadow() {\n"
                "    use crate::other::graphics;\n"
                "    let _shadowed = graphics::text::shape();\n"
                "}\n"
                "fn sibling_leak() { let _old_owner = graphics::text::shape(); }\n",
                encoding="utf-8",
            )
            (root / "cfg_reverse.rs").write_text(
                "use crate::test_gfx::{text as local_text};\n"
                "fn local_owner() { local_text::shape(); }\n"
                "use crate::nested_test_gfx::{text as nested_local_text};\n"
                "fn nested_local_owner() { nested_local_text::shape(); }\n"
                "use crate::maybe_graphics::{text as maybe_text};\n"
                "fn maybe_owner() { maybe_text::shape(); }\n"
                "use crate::joint_test_gfx::{text as joint_local_text};\n"
                "fn joint_local_owner() { joint_local_text::shape(); }\n",
                encoding="utf-8",
            )

            offenders = old_graphics_text_owner_offenders(root)

            self.assertEqual(14, len(offenders))
            self.assertIn("gfx::text::shape", offenders[0])
            self.assertTrue(any("g2::{text as forward_text}" in item for item in offenders))
            self.assertTrue(any("crate::graphics::text::shape" in item for item in offenders))
            self.assertTrue(any("r#graphics::{r#text" in item for item in offenders))
            self.assertTrue(any("shape_root_relative" in item for item in offenders))
            self.assertTrue(any("self_graphics::{text" in item for item in offenders))
            self.assertTrue(
                any("crate::qualified_graphics::{text" in item for item in offenders)
            )
            self.assertTrue(
                any("self::qualified_graphics::{text" in item for item in offenders)
            )
            self.assertTrue(any("_old_owner" in item for item in offenders))
            self.assertFalse(any("_shadowed" in item for item in offenders))
            self.assertTrue(
                any(
                    "crate::qualified_graphics::text::shape" in item
                    for item in offenders
                )
            )
            self.assertFalse(any("local_owner" in item for item in offenders))
            self.assertFalse(any("nested_local_owner" in item for item in offenders))
            self.assertTrue(any("maybe_graphics::{text" in item for item in offenders))
            self.assertFalse(any("joint_local_owner" in item for item in offenders))
            backend_code = _rust_code_view(
                "use {wgpu as gpu};\n"
                "struct Leaked(gpu::Texture);\n"
                '// use fontdb as fonts;\n'
                'const NOTE: &str = "glyphon::Cache";\n'
            )
            self.assertEqual(["wgpu"], forbidden_backend_crates(backend_code))

    def test_graphics_production_has_no_ui_references(self) -> None:
        self.assertEqual(
            [],
            domain_edge_offenders("graphics", "ui"),
            "graphics must not reach into the UI implementation domain",
        )


if __name__ == "__main__":
    unittest.main()
