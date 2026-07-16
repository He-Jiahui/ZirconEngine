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
        "text",
        "ui",
    }
)

RUST_IDENTIFIER_PATTERN = r"(?:r#)?[A-Za-z_][A-Za-z0-9_]*"
CRATE_DOMAIN_REFERENCE = re.compile(
    rf"\bcrate\s*::\s*({RUST_IDENTIFIER_PATTERN})"
    r"(?=\s*::|\s*(?:as\b|[,;}]))"
)
GROUPED_CRATE_USE_START = re.compile(r"\buse\s+crate\s*::\s*\{")
CFG_ATTRIBUTE_START = re.compile(r"#\s*\[\s*cfg\s*\(")
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
RUST_USE_TOKEN = re.compile(rf"::|{RUST_IDENTIFIER_PATTERN}|[{{}},*]")


@dataclass(frozen=True, order=True)
class DomainReference:
    source_domain: str
    target_domain: str
    path: str
    line: int
    source: str


@dataclass(frozen=True)
class _CfgExpression:
    name: str
    arguments: tuple["_CfgExpression", ...] | None = None
    atom_key: str | None = None


def _canonical_rust_identifier(identifier: str) -> str:
    return identifier[2:] if identifier.startswith("r#") else identifier


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


def _rust_use_paths(
    code_view: str,
) -> list[tuple[tuple[str, ...], str | None, int]]:
    """Return canonical-looking leaf paths, aliases, and lines from Rust use trees."""
    paths: list[tuple[tuple[str, ...], str | None, int]] = []

    def parse_tree(
        tokens: list[tuple[str, int]], index: int, prefix: tuple[str, ...]
    ) -> tuple[list[tuple[tuple[str, ...], str | None, int]], int]:
        parsed: list[tuple[tuple[str, ...], str | None, int]] = []
        if index >= len(tokens):
            return parsed, index

        token, offset = tokens[index]
        if token == "{":
            index += 1
            while index < len(tokens) and tokens[index][0] != "}":
                if tokens[index][0] == ",":
                    index += 1
                    continue
                children, index = parse_tree(tokens, index, prefix)
                parsed.extend(children)
            return parsed, min(index + 1, len(tokens))

        if token == "*":
            parsed.append((prefix + (token,), None, offset))
            return parsed, index + 1

        if token in {",", "}"}:
            return parsed, index + 1

        path = prefix if token == "self" and prefix else prefix + (token,)
        index += 1
        if index < len(tokens) and tokens[index][0] == "::":
            return parse_tree(tokens, index + 1, path)

        alias = None
        if index < len(tokens) and tokens[index][0] == "as":
            if index + 1 < len(tokens):
                alias = tokens[index + 1][0]
                index += 2
            else:
                index += 1
        parsed.append((path, alias, offset))
        return parsed, index

    for use_statement in re.finditer(r"\buse\s+(?P<body>[^;]+);", code_view):
        body_start = use_statement.start("body")
        tokens = [
            (
                _canonical_rust_identifier(token.group(0)),
                body_start + token.start(),
            )
            for token in RUST_USE_TOKEN.finditer(use_statement.group("body"))
        ]
        index = 1 if tokens and tokens[0][0] == "::" else 0
        while index < len(tokens):
            parsed, next_index = parse_tree(tokens, index, ())
            paths.extend(
                (path, alias, code_view.count("\n", 0, offset) + 1)
                for path, alias, offset in parsed
            )
            index = max(next_index, index + 1)

    return paths


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


def _parse_cfg_expression(
    expression: str, start: int = 0
) -> tuple[_CfgExpression | None, int]:
    cursor = start
    while cursor < len(expression) and expression[cursor].isspace():
        cursor += 1
    atom_start = cursor
    identifier = re.match(r"[A-Za-z_][A-Za-z0-9_]*", expression[cursor:])
    if identifier is None:
        return None, cursor
    name = identifier.group(0)
    cursor += identifier.end()
    while cursor < len(expression) and expression[cursor].isspace():
        cursor += 1
    if cursor >= len(expression) or expression[cursor] != "(":
        in_string = False
        escaped = False
        while cursor < len(expression):
            character = expression[cursor]
            if in_string:
                if escaped:
                    escaped = False
                elif character == "\\":
                    escaped = True
                elif character == '"':
                    in_string = False
            elif character == '"':
                in_string = True
            elif character in {",", ")"}:
                break
            cursor += 1
        return _CfgExpression(
            name=name,
            atom_key=_normalize_cfg_atom(expression[atom_start:cursor]),
        ), cursor

    cursor += 1
    arguments: list[_CfgExpression] = []
    while cursor < len(expression):
        while cursor < len(expression) and expression[cursor].isspace():
            cursor += 1
        if cursor >= len(expression) or expression[cursor] == ")":
            end = cursor + 1
            return _CfgExpression(
                name=name,
                arguments=tuple(arguments),
                atom_key=(
                    None
                    if name in {"all", "any", "not"}
                    else _normalize_cfg_atom(expression[atom_start:end])
                ),
            ), end
        argument, next_cursor = _parse_cfg_expression(expression, cursor)
        if argument is None or next_cursor <= cursor:
            return None, cursor
        arguments.append(argument)
        cursor = next_cursor
        while cursor < len(expression) and expression[cursor].isspace():
            cursor += 1
        if cursor < len(expression) and expression[cursor] == ",":
            cursor += 1
            continue
        if cursor < len(expression) and expression[cursor] == ")":
            end = cursor + 1
            return _CfgExpression(
                name=name,
                arguments=tuple(arguments),
                atom_key=(
                    None
                    if name in {"all", "any", "not"}
                    else _normalize_cfg_atom(expression[atom_start:end])
                ),
            ), end
        return None, cursor
    return None, cursor


def _normalize_cfg_atom(atom: str) -> str:
    rendered: list[str] = []
    in_string = False
    escaped = False
    for character in atom:
        if in_string:
            rendered.append(character)
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == '"':
                in_string = False
        elif character == '"':
            in_string = True
            rendered.append(character)
        elif not character.isspace():
            rendered.append(character)
    return "".join(rendered)


def _cfg_unknown_atoms(expression: _CfgExpression) -> set[str]:
    if expression.arguments is None or expression.name not in {"all", "any", "not"}:
        if expression.name == "test" and expression.atom_key == "test":
            return set()
        return {expression.atom_key or expression.name}
    return {
        atom
        for argument in expression.arguments
        for atom in _cfg_unknown_atoms(argument)
    }


def _evaluate_cfg_with_test_false(
    expression: _CfgExpression, assignment: dict[str, bool]
) -> bool:
    if expression.arguments is None or expression.name not in {"all", "any", "not"}:
        if expression.name == "test" and expression.atom_key == "test":
            return False
        return assignment[expression.atom_key or expression.name]
    if expression.name == "all":
        return all(
            _evaluate_cfg_with_test_false(argument, assignment)
            for argument in expression.arguments
        )
    if expression.name == "any":
        return any(
            _evaluate_cfg_with_test_false(argument, assignment)
            for argument in expression.arguments
        )
    return len(expression.arguments) == 1 and not _evaluate_cfg_with_test_false(
        expression.arguments[0], assignment
    )


def _cfg_attributes(
    code_view: str, cfg_source: str | None = None
) -> list[tuple[int, int, _CfgExpression]]:
    attributes: list[tuple[int, int, _CfgExpression]] = []
    for attribute in CFG_ATTRIBUTE_START.finditer(code_view):
        opening = attribute.end() - 1
        closing = _matching_delimiter_end(code_view, opening, "(", ")")
        predicate_source = cfg_source if cfg_source is not None else code_view
        expression, _cursor = _parse_cfg_expression(
            predicate_source[opening + 1 : closing - 1]
        )
        if expression is None:
            continue
        attribute_end = closing
        while attribute_end < len(code_view) and code_view[attribute_end].isspace():
            attribute_end += 1
        if attribute_end < len(code_view) and code_view[attribute_end] == "]":
            attribute_end += 1
        attributes.append((attribute.start(), attribute_end, expression))
    return attributes


def _cfg_expressions_imply_test(expressions: list[_CfgExpression]) -> bool:
    if not expressions:
        return False
    combined = _CfgExpression(name="all", arguments=tuple(expressions))
    atoms = sorted(_cfg_unknown_atoms(combined))
    if len(atoms) > 16:
        return False
    for bit_mask in range(1 << len(atoms)):
        assignment = {
            atom: bool(bit_mask & (1 << index))
            for index, atom in enumerate(atoms)
        }
        if _evaluate_cfg_with_test_false(combined, assignment):
            return False
    return True


def _grouped_crate_use_references(code_view: str) -> list[tuple[str, int]]:
    """Return top-level domains from `use crate::{...}` trees and their lines."""
    references: list[tuple[str, int]] = []

    def append_entry(entry_start: int, entry_end: int) -> None:
        entry = code_view[entry_start:entry_end]
        identifier = re.search(rf"\b({RUST_IDENTIFIER_PATTERN})\b", entry)
        if identifier is None:
            return
        domain = _canonical_rust_identifier(identifier.group(1))
        if domain not in RUNTIME_DOMAINS:
            return
        absolute_offset = entry_start + identifier.start(1)
        references.append(
            (domain, code_view.count("\n", 0, absolute_offset) + 1)
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


def _cfg_test_item_spans(
    code_view: str, cfg_source: str | None = None
) -> list[tuple[int, int]]:
    spans: list[tuple[int, int]] = []
    attributes_by_item: dict[int, list[tuple[int, _CfgExpression]]] = {}
    for cfg_start, cfg_end, expression in _cfg_attributes(code_view, cfg_source):
        item_start = _skip_whitespace_and_attributes(code_view, cfg_end)
        attributes_by_item.setdefault(item_start, []).append((cfg_start, expression))
    for item_start, attributes in attributes_by_item.items():
        if not _cfg_expressions_imply_test(
            [expression for _start, expression in attributes]
        ):
            continue
        cfg_start = min(start for start, _expression in attributes)
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
                spans.append((cfg_start, cursor + 1))
                break
            elif character == "{" and parenthesis_depth == 0 and bracket_depth == 0:
                spans.append(
                    (
                        cfg_start,
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
                (
                    target_path,
                    _cfg_expressions_imply_test(
                        [
                            expression
                            for _start, _end, expression in _cfg_attributes(
                                _rust_code_view(original_attributes),
                                original_attributes,
                            )
                        ]
                    ),
                )
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
        test_item_spans = _cfg_test_item_spans(code_view, source)
        if test_item_spans:
            masked_source = list(code_view)
            for span_start, span_end in test_item_spans:
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
            targets = set()
            for match in CRATE_DOMAIN_REFERENCE.finditer(audit_line):
                target_domain = _canonical_rust_identifier(match.group(1))
                if target_domain in RUNTIME_DOMAINS and target_domain != source_domain:
                    targets.add(target_domain)
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
