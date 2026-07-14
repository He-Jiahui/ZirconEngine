from __future__ import annotations

import argparse
import json
import re
from collections import Counter
from dataclasses import asdict, dataclass
from pathlib import Path


RUNTIME_DOMAINS = frozenset(
    {
        "animation",
        "asset",
        "builtin",
        "core",
        "diagnostic_log",
        "dynamic_api",
        "engine_module",
        "foundation",
        "graphics",
        "input",
        "navigation",
        "platform",
        "plugin",
        "render_graph",
        "rhi",
        "rhi_wgpu",
        "scene",
        "script",
        "ui",
    }
)

CRATE_DOMAIN_REFERENCE = re.compile(
    r"\bcrate::([A-Za-z_][A-Za-z0-9_]*)(?=::|\s*(?:as\b|[,;}]))"
)
GROUPED_CRATE_USE_START = re.compile(r"\buse\s+crate::\{")
CFG_TEST_ATTRIBUTE = re.compile(
    r"#\s*\[\s*cfg\s*\(\s*"
    r"(?:test|all\s*\(\s*test(?:\s*,[^)\r\n]*)?\))"
    r"\s*\)\s*\]"
)
EXTERNAL_MODULE_DECLARATION = re.compile(
    r"(?P<attributes>(?:#\s*\[[^\]]*\]\s*)*)"
    r"(?:(?:pub(?:\s*\([^)]*\))?|unsafe)\s+)*"
    r"mod\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*;"
)
PATH_ATTRIBUTE = re.compile(r'#\s*\[\s*path\s*=\s*"([^"]+)"\s*\]')
RAW_STRING_LITERAL_START = re.compile(r'(?:b|c)?r(?P<hashes>#{0,255})"')
WHITESPACE = re.compile(r"\s*")
SIMPLE_CHAR_ESCAPES = frozenset({"'", '"', "n", "r", "t", "\\", "0"})
HEX_DIGITS = frozenset("0123456789abcdefABCDEF")


@dataclass(frozen=True, order=True)
class DomainReference:
    source_domain: str
    target_domain: str
    path: str
    line: int
    source: str


def _rust_char_literal_end(source: str, start: int) -> int | None:
    """Return the end of a valid Rust character literal, preserving lifetimes."""
    cursor = start + 1
    if cursor >= len(source) or source[cursor] in {"'", "\r", "\n"}:
        return None

    if source[cursor] != "\\":
        cursor += 1
    else:
        cursor += 1
        if cursor >= len(source):
            return None
        escape = source[cursor]
        if escape in SIMPLE_CHAR_ESCAPES:
            cursor += 1
        elif escape == "x":
            digits = source[cursor + 1 : cursor + 3]
            if len(digits) != 2 or any(digit not in HEX_DIGITS for digit in digits):
                return None
            cursor += 3
        elif escape == "u" and source[cursor + 1 : cursor + 2] == "{":
            closing = source.find("}", cursor + 2)
            if closing == -1:
                return None
            digits = source[cursor + 2 : closing].replace("_", "")
            if not 1 <= len(digits) <= 6 or any(
                digit not in HEX_DIGITS for digit in digits
            ):
                return None
            cursor = closing + 1
        else:
            return None

    return cursor + 1 if source[cursor : cursor + 1] == "'" else None


def is_production_rust_source(relative_path: Path) -> bool:
    if relative_path.suffix != ".rs":
        return False
    parts = relative_path.parts
    file_name = relative_path.name
    if (
        "tests" in parts
        or file_name in {"tests.rs", "test.rs"}
        or file_name.endswith("_tests.rs")
        or file_name.startswith("test_")
    ):
        return False
    return parts and parts[0] in RUNTIME_DOMAINS


def _rust_code_view(source: str) -> str:
    """Blank Rust comments and literals while preserving offsets and newlines."""
    rendered = list(source)
    index = 0
    block_comment_depth = 0
    source_length = len(source)

    def blank(start: int, end: int) -> None:
        for offset in range(start, end):
            if rendered[offset] not in {"\r", "\n"}:
                rendered[offset] = " "

    while index < source_length:
        character = source[index]
        if block_comment_depth:
            if character == "/" and source.startswith("/*", index):
                blank(index, index + 2)
                block_comment_depth += 1
                index += 2
            elif character == "*" and source.startswith("*/", index):
                blank(index, index + 2)
                block_comment_depth -= 1
                index += 2
            else:
                blank(index, index + 1)
                index += 1
            continue

        if character == "/" and source.startswith("//", index):
            line_end = source.find("\n", index)
            if line_end == -1:
                line_end = source_length
            blank(index, line_end)
            index = line_end
            continue
        if character == "/" and source.startswith("/*", index):
            blank(index, index + 2)
            block_comment_depth = 1
            index += 2
            continue

        raw_string_match = (
            RAW_STRING_LITERAL_START.match(source, index)
            if character == "r"
            or (
                character in {"b", "c"}
                and source[index + 1 : index + 2] == "r"
            )
            else None
        )
        if raw_string_match is not None:
            delimiter = '"' + raw_string_match.group("hashes")
            literal_end = source.find(
                delimiter, raw_string_match.end()
            )
            if literal_end == -1:
                literal_end = source_length
            else:
                literal_end += len(delimiter)
            blank(index, literal_end)
            index = literal_end
            continue

        quote_index = index
        if character in {"b", "c"} and source[index + 1 : index + 2] == '"':
            quote_index += 1
        if source[quote_index : quote_index + 1] == '"':
            literal_end = quote_index + 1
            escaped = False
            while literal_end < source_length:
                character = source[literal_end]
                literal_end += 1
                if escaped:
                    escaped = False
                elif character == "\\":
                    escaped = True
                elif character == '"':
                    break
            blank(index, literal_end)
            index = literal_end
            continue

        if character == "'":
            literal_end = _rust_char_literal_end(source, index)
            if literal_end is not None:
                blank(index, literal_end)
                index = literal_end
            else:
                index += 1
            continue

        index += 1

    return "".join(rendered)


def _matching_delimiter_end(source: str, start: int, opening: str, closing: str) -> int:
    depth = 0
    for index in range(start, len(source)):
        character = source[index]
        if character == opening:
            depth += 1
        elif character == closing:
            depth -= 1
            if depth == 0:
                return index + 1
    return len(source)


def _grouped_crate_use_references(code_view: str) -> list[tuple[str, int]]:
    """Return top-level domains from `use crate::{...}` trees and their lines."""
    references: list[tuple[str, int]] = []

    def append_entry(entry_start: int, entry_end: int) -> None:
        entry = code_view[entry_start:entry_end]
        identifier = re.search(r"\b([A-Za-z_][A-Za-z0-9_]*)\b", entry)
        if identifier is None or identifier.group(1) not in RUNTIME_DOMAINS:
            return
        absolute_offset = entry_start + identifier.start(1)
        references.append(
            (identifier.group(1), code_view.count("\n", 0, absolute_offset) + 1)
        )

    for grouped_use in GROUPED_CRATE_USE_START.finditer(code_view):
        opening = grouped_use.end() - 1
        closing = _matching_delimiter_end(code_view, opening, "{", "}")
        if closing <= opening + 1:
            continue
        entry_start = opening + 1
        nested_depth = 0
        for cursor in range(entry_start, closing - 1):
            character = code_view[cursor]
            if character == "{":
                nested_depth += 1
            elif character == "}":
                nested_depth = max(0, nested_depth - 1)
            elif character == "," and nested_depth == 0:
                append_entry(entry_start, cursor)
                entry_start = cursor + 1
        append_entry(entry_start, closing - 1)

    return references


def _skip_whitespace_and_attributes(code_view: str, start: int) -> int:
    cursor = start
    while cursor < len(code_view):
        whitespace = WHITESPACE.match(code_view, cursor)
        cursor = whitespace.end() if whitespace is not None else cursor
        if not code_view.startswith("#[", cursor):
            return cursor
        cursor = _matching_delimiter_end(code_view, cursor + 1, "[", "]")
    return cursor


def _cfg_test_item_spans(code_view: str) -> list[tuple[int, int]]:
    spans: list[tuple[int, int]] = []
    for cfg_match in CFG_TEST_ATTRIBUTE.finditer(code_view):
        item_start = _skip_whitespace_and_attributes(code_view, cfg_match.end())
        parenthesis_depth = 0
        bracket_depth = 0
        cursor = item_start
        while cursor < len(code_view):
            character = code_view[cursor]
            if character == "(":
                parenthesis_depth += 1
            elif character == ")":
                parenthesis_depth = max(0, parenthesis_depth - 1)
            elif character == "[":
                bracket_depth += 1
            elif character == "]":
                bracket_depth = max(0, bracket_depth - 1)
            elif character == ";" and parenthesis_depth == 0 and bracket_depth == 0:
                spans.append((cfg_match.start(), cursor + 1))
                break
            elif character == "{" and parenthesis_depth == 0 and bracket_depth == 0:
                spans.append(
                    (
                        cfg_match.start(),
                        _matching_delimiter_end(code_view, cursor, "{", "}"),
                    )
                )
                break
            cursor += 1
    return spans


def _brace_depths(code_view: str) -> list[int]:
    depths = [0] * (len(code_view) + 1)
    depth = 0
    for index, character in enumerate(code_view):
        depths[index] = depth
        if character == "{":
            depth += 1
        elif character == "}":
            depth = max(0, depth - 1)
    depths[len(code_view)] = depth
    return depths


def _resolve_module_path(
    parent_path: Path, module_name: str, explicit_path: str | None
) -> Path | None:
    if explicit_path is not None:
        candidate_paths = [parent_path.parent / explicit_path]
    else:
        module_root = (
            parent_path.parent
            if parent_path.name in {"lib.rs", "main.rs", "mod.rs"}
            else parent_path.parent / parent_path.stem
        )
        candidate_paths = [
            module_root / f"{module_name}.rs",
            module_root / module_name / "mod.rs",
        ]
    return next((path for path in candidate_paths if path.is_file()), None)


def _module_edges(
    source_path: Path, source: str, code_view: str | None = None
) -> list[tuple[Path, bool]]:
    if code_view is None:
        code_view = _rust_code_view(source)
    brace_depths = _brace_depths(code_view)
    edges: list[tuple[Path, bool]] = []
    for declaration in EXTERNAL_MODULE_DECLARATION.finditer(code_view):
        if brace_depths[declaration.start()] != 0:
            continue
        original_attributes = source[
            declaration.start("attributes") : declaration.end("attributes")
        ]
        path_match = PATH_ATTRIBUTE.search(original_attributes)
        target_path = _resolve_module_path(
            source_path,
            declaration.group("name"),
            path_match.group(1) if path_match is not None else None,
        )
        if target_path is not None:
            edges.append(
                (target_path, CFG_TEST_ATTRIBUTE.search(original_attributes) is not None)
            )
    return edges


def _test_only_source_paths(
    source_root: Path,
) -> tuple[set[Path], dict[Path, str], dict[Path, str]]:
    source_paths = {
        path
        for domain in RUNTIME_DOMAINS
        if (domain_root := source_root / domain).is_dir()
        for path in domain_root.rglob("*.rs")
    }
    sources = {path: path.read_text(encoding="utf-8") for path in source_paths}
    edges: dict[Path, list[tuple[Path, bool]]] = {}
    code_views: dict[Path, str] = {}
    for path, source in sources.items():
        if re.search(r"\bmod\s", source) is None:
            continue
        code_view = _rust_code_view(source)
        code_views[path] = code_view
        edges[path] = _module_edges(path, source, code_view)
    resolved_source_root = source_root
    direct_test_paths = {
        path
        for path in source_paths
        if not is_production_rust_source(path.relative_to(resolved_source_root))
    }
    direct_test_paths.update(
        target
        for declarations in edges.values()
        for target, cfg_test in declarations
        if cfg_test
    )

    test_reachable = set(direct_test_paths)
    pending = list(direct_test_paths)
    while pending:
        parent = pending.pop()
        for target, _ in edges.get(parent, []):
            if target not in test_reachable:
                test_reachable.add(target)
                pending.append(target)

    production_reachable = source_paths - test_reachable
    changed = True
    while changed:
        changed = False
        for parent in tuple(production_reachable):
            for target, cfg_test in edges.get(parent, []):
                if not cfg_test and target not in production_reachable:
                    production_reachable.add(target)
                    changed = True

    return test_reachable - production_reachable, sources, code_views


def audit_runtime_domain_dependencies(repo_root: Path) -> dict[str, object]:
    source_root = repo_root / "zircon_runtime" / "src"
    references: set[DomainReference] = set()
    test_only_source_paths, sources, code_views = _test_only_source_paths(source_root)

    for source_path in sorted(sources):
        relative_path = source_path.relative_to(source_root)
        if (
            not is_production_rust_source(relative_path)
            or source_path in test_only_source_paths
        ):
            continue

        source_domain = relative_path.parts[0]
        source = sources[source_path]
        code_view = code_views.get(source_path)
        if code_view is None:
            code_view = _rust_code_view(source)
        audit_source = code_view
        if CFG_TEST_ATTRIBUTE.search(source) is not None:
            masked_source = list(code_view)
            for span_start, span_end in _cfg_test_item_spans(code_view):
                for offset in range(span_start, span_end):
                    if masked_source[offset] not in {"\r", "\n"}:
                        masked_source[offset] = " "
            audit_source = "".join(masked_source)
        grouped_targets_by_line: dict[int, set[str]] = {}
        for target_domain, line_number in _grouped_crate_use_references(audit_source):
            grouped_targets_by_line.setdefault(line_number, set()).add(target_domain)
        for line_number, (line, audit_line) in enumerate(
            zip(source.splitlines(), audit_source.splitlines(), strict=True),
            start=1,
        ):
            targets = {
                match.group(1)
                for match in CRATE_DOMAIN_REFERENCE.finditer(audit_line)
                if match.group(1) in RUNTIME_DOMAINS
                and match.group(1) != source_domain
            }
            targets.update(
                target_domain
                for target_domain in grouped_targets_by_line.get(line_number, ())
                if target_domain != source_domain
            )
            for target_domain in targets:
                references.add(
                    DomainReference(
                        source_domain=source_domain,
                        target_domain=target_domain,
                        path=(Path("zircon_runtime/src") / relative_path).as_posix(),
                        line=line_number,
                        source=line.strip(),
                    )
                )

    ordered_references = sorted(references)
    matrix = Counter(
        (reference.source_domain, reference.target_domain)
        for reference in ordered_references
    )
    matrix_rows = [
        {
            "source_domain": source_domain,
            "target_domain": target_domain,
            "reference_count": count,
        }
        for (source_domain, target_domain), count in sorted(matrix.items())
    ]

    return {
        "schema_version": 1,
        "runtime_source_root": "zircon_runtime/src",
        "production_reference_count": len(ordered_references),
        "domain_edge_count": len(matrix_rows),
        "matrix": matrix_rows,
        "references": [asdict(reference) for reference in ordered_references],
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Audit direct cross-domain references in zircon_runtime production Rust."
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="Repository root override.",
    )
    parser.add_argument(
        "--pretty",
        action="store_true",
        help="Pretty-print JSON for review or baseline capture.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Write the JSON report to this path instead of stdout.",
    )
    args = parser.parse_args()

    report = audit_runtime_domain_dependencies(args.repo_root.resolve())
    rendered = json.dumps(
        report,
        ensure_ascii=False,
        indent=2 if args.pretty else None,
        sort_keys=True,
    )
    if args.output is None:
        print(rendered)
    else:
        output_path = args.output.resolve()
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(f"{rendered}\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
