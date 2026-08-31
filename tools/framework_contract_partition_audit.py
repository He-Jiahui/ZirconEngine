from __future__ import annotations

import argparse
import json
import re
from collections import Counter
from dataclasses import asdict, dataclass
from pathlib import Path

if __package__:
    from tools.runtime_domain_dependency_audit import (
        _brace_depths,
        _cfg_test_item_spans,
        _matching_delimiter_end,
        _module_edges,
        _rust_code_view,
        is_production_rust_source,
    )
else:
    from runtime_domain_dependency_audit import (
        _brace_depths,
        _cfg_test_item_spans,
        _matching_delimiter_end,
        _module_edges,
        _rust_code_view,
        is_production_rust_source,
    )


RUST_IDENTIFIER_PATTERN = r"(?:r#)?[A-Za-z_][A-Za-z0-9_]*"
FUNCTION_DECLARATION = re.compile(
    rf"(?:(?P<visibility>\bpub(?:\s*\([^)]*\))?)\s+)?"
    rf"(?:(?:const|async|unsafe|extern|default)\s+)*"
    rf"fn\s+{RUST_IDENTIFIER_PATTERN}\b"
)
STRUCTURAL_DECLARATION = re.compile(
    rf"(?:(?P<visibility>\bpub(?:\s*\([^)]*\))?)\s+)?"
    rf"\b(?P<kind>struct|enum|trait|union|type)\s+{RUST_IDENTIFIER_PATTERN}\b"
)
TRAIT_DECLARATION = re.compile(
    rf"(?m)^[ \t]*(?:pub(?:\s*\([^)]*\))?\s+)?"
    rf"(?:unsafe\s+)?trait\s+{RUST_IDENTIFIER_PATTERN}\b"
)
IMPL_DECLARATION = re.compile(
    r"(?m)^[ \t]*(?:(?:unsafe|default)\s+)?impl\b"
)
STATIC_ITEM = re.compile(
    rf"(?m)^[ \t]*(?:pub(?:\s*\([^)]*\))?\s+)?"
    rf"(?:unsafe\s+)?static(?:\s+mut)?\s+{RUST_IDENTIFIER_PATTERN}\b"
)
MACRO_DEFINITION = re.compile(
    rf"\bmacro_rules\s*!\s*{RUST_IDENTIFIER_PATTERN}\b"
)
MACRO_INVOCATION = re.compile(
    rf"(?P<path>{RUST_IDENTIFIER_PATTERN}"
    rf"(?:\s*::\s*{RUST_IDENTIFIER_PATTERN})*)"
    r"\s*!\s*(?P<delimiter>[{([])"
)


@dataclass(frozen=True)
class RustSourceClassification:
    classification: str
    manual_review_required: bool
    function_body_count: int
    public_function_body_count: int
    restricted_function_body_count: int
    free_function_body_count: int
    impl_function_body_count: int
    trait_default_function_body_count: int
    nested_function_body_count: int
    impl_block_count: int
    static_item_count: int
    macro_definition_count: int
    macro_invocation_review_count: int
    trait_count: int
    public_trait_count: int
    struct_count: int
    public_struct_count: int
    enum_count: int
    public_enum_count: int
    union_count: int
    public_union_count: int
    type_alias_count: int
    public_type_alias_count: int
    nonempty_code_lines: int


def _mask_spans(source: str, spans: list[tuple[int, int]]) -> str:
    if not spans:
        return source
    rendered = list(source)
    for start, end in spans:
        for offset in range(start, end):
            if rendered[offset] not in {"\r", "\n"}:
                rendered[offset] = " "
    return "".join(rendered)


def _production_code_view(source: str, code_view: str | None = None) -> str:
    if code_view is None:
        code_view = _rust_code_view(source)
    return _mask_spans(code_view, _cfg_test_item_spans(code_view, source))


def _item_body_span(
    code_view: str, start: int, depths: list[int]
) -> tuple[int, int] | None:
    base_depth = depths[start]
    parenthesis_depth = 0
    bracket_depth = 0
    angle_bracket_depth = 0
    cursor = start

    while cursor < len(code_view):
        character = code_view[cursor]
        current_depth = depths[cursor]
        if character == "(":
            parenthesis_depth += 1
        elif character == ")":
            parenthesis_depth = max(0, parenthesis_depth - 1)
        elif character == "[":
            bracket_depth += 1
        elif character == "]":
            bracket_depth = max(0, bracket_depth - 1)
        elif (
            character == "<"
            and current_depth == base_depth
            and parenthesis_depth == 0
            and bracket_depth == 0
        ):
            angle_bracket_depth += 1
        elif (
            character == ">"
            and current_depth == base_depth
            and parenthesis_depth == 0
            and bracket_depth == 0
        ):
            angle_bracket_depth = max(0, angle_bracket_depth - 1)
        elif (
            character == ";"
            and current_depth == base_depth
            and parenthesis_depth == 0
            and bracket_depth == 0
            and angle_bracket_depth == 0
        ):
            return None
        elif (
            character == "{"
            and current_depth == base_depth
            and parenthesis_depth == 0
            and bracket_depth == 0
            and angle_bracket_depth == 0
        ):
            return (
                cursor,
                _matching_delimiter_end(code_view, cursor, "{", "}"),
            )
        elif current_depth < base_depth:
            return None
        cursor += 1

    return None


def _visibility(visibility: str | None) -> str:
    if visibility is not None and "(" in visibility:
        return "restricted"
    if visibility == "pub":
        return "public"
    return "private"


def _span_contains(spans: list[tuple[int, int]], offset: int) -> bool:
    return any(start < offset < end for start, end in spans)


def _macro_definition_spans(code_view: str) -> list[tuple[int, int]]:
    if "macro_rules" not in code_view:
        return []
    spans: list[tuple[int, int]] = []
    delimiters = {"{": "}", "(": ")", "[": "]"}
    for declaration in MACRO_DEFINITION.finditer(code_view):
        cursor = declaration.end()
        while cursor < len(code_view) and code_view[cursor].isspace():
            cursor += 1
        if cursor >= len(code_view) or code_view[cursor] not in delimiters:
            continue
        spans.append(
            (
                declaration.start(),
                _matching_delimiter_end(
                    code_view,
                    cursor,
                    code_view[cursor],
                    delimiters[code_view[cursor]],
                ),
            )
        )
    return spans


def _macro_invocation_spans(
    code_view: str, ignored_spans: list[tuple[int, int]]
) -> list[tuple[int, int]]:
    bang_cursor = code_view.find("!")
    has_delimited_invocation = False
    while bang_cursor >= 0:
        delimiter_cursor = bang_cursor + 1
        while (
            delimiter_cursor < len(code_view)
            and code_view[delimiter_cursor].isspace()
        ):
            delimiter_cursor += 1
        if (
            delimiter_cursor < len(code_view)
            and code_view[delimiter_cursor] in "{(["
        ):
            has_delimited_invocation = True
            break
        bang_cursor = code_view.find("!", bang_cursor + 1)
    if not has_delimited_invocation:
        return []
    spans: list[tuple[int, int]] = []
    delimiters = {"{": "}", "(": ")", "[": "]"}
    for invocation in MACRO_INVOCATION.finditer(code_view):
        if _span_contains(ignored_spans + spans, invocation.start()):
            continue
        delimiter_start = invocation.start("delimiter")
        opening = invocation.group("delimiter")
        spans.append(
            (
                invocation.start(),
                _matching_delimiter_end(
                    code_view,
                    delimiter_start,
                    opening,
                    delimiters[opening],
                ),
            )
        )
    return spans


def _block_spans(
    code_view: str, pattern: re.Pattern[str], depths: list[int]
) -> list[tuple[int, int]]:
    spans: list[tuple[int, int]] = []
    for declaration in pattern.finditer(code_view):
        body_span = _item_body_span(code_view, declaration.end(), depths)
        if body_span is not None:
            spans.append(body_span)
    return spans


def classify_rust_source(
    source: str, code_view: str | None = None
) -> RustSourceClassification:
    production_code_view = _production_code_view(source, code_view)
    macro_spans = _macro_definition_spans(production_code_view)
    macro_invocation_spans = _macro_invocation_spans(
        production_code_view, macro_spans
    )
    code_view = _mask_spans(
        production_code_view, macro_spans + macro_invocation_spans
    )
    depths = _brace_depths(code_view)
    trait_spans = _block_spans(code_view, TRAIT_DECLARATION, depths)

    function_rows: list[tuple[re.Match[str], tuple[int, int]]] = []
    for function in FUNCTION_DECLARATION.finditer(code_view):
        body_span = _item_body_span(code_view, function.end(), depths)
        if body_span is not None:
            function_rows.append((function, body_span))
    function_spans = [body_span for _function, body_span in function_rows]

    impl_spans = [
        body_span
        for declaration in IMPL_DECLARATION.finditer(code_view)
        if not _span_contains(function_spans, declaration.start())
        if (
            body_span := _item_body_span(code_view, declaration.end(), depths)
        )
        is not None
    ]

    function_body_count = 0
    public_function_body_count = 0
    restricted_function_body_count = 0
    free_function_body_count = 0
    impl_function_body_count = 0
    trait_default_function_body_count = 0
    nested_function_body_count = 0

    for function, _body_span in function_rows:
        function_body_count += 1
        visibility = _visibility(function.group("visibility"))
        if visibility == "public":
            public_function_body_count += 1
        elif visibility == "restricted":
            restricted_function_body_count += 1

        if _span_contains(function_spans, function.start()):
            nested_function_body_count += 1
        elif _span_contains(impl_spans, function.start()):
            impl_function_body_count += 1
        elif _span_contains(trait_spans, function.start()):
            trait_default_function_body_count += 1
        else:
            free_function_body_count += 1

    declaration_exclusion_spans = function_spans + impl_spans + trait_spans
    declaration_matches = [
        declaration
        for declaration in STRUCTURAL_DECLARATION.finditer(code_view)
        if not _span_contains(declaration_exclusion_spans, declaration.start())
    ]
    declarations = Counter(match.group("kind") for match in declaration_matches)
    public_declarations = Counter(
        match.group("kind")
        for match in declaration_matches
        if match.group("visibility") == "pub"
    )
    static_item_count = sum(
        1
        for item in STATIC_ITEM.finditer(code_view)
        if not _span_contains(declaration_exclusion_spans, item.start())
    )
    macro_definition_count = sum(
        1
        for start, _end in macro_spans
        if not _span_contains(function_spans, start)
    )
    macro_invocation_review_count = sum(
        1
        for start, _end in macro_invocation_spans
        if not _span_contains(function_spans, start)
    )
    structural_declaration_count = sum(declarations.values())
    has_behavior = bool(function_body_count or static_item_count)
    if not has_behavior:
        if structural_declaration_count:
            classification = "declaration_only"
        elif impl_spans:
            classification = "contract_binding_only"
        elif macro_definition_count or macro_invocation_review_count:
            classification = "macro_generated_review"
        else:
            classification = "declaration_only"
    elif structural_declaration_count:
        classification = "mixed"
    else:
        classification = "behavior_only"

    return RustSourceClassification(
        classification=classification,
        manual_review_required=bool(
            macro_definition_count or macro_invocation_review_count
        ),
        function_body_count=function_body_count,
        public_function_body_count=public_function_body_count,
        restricted_function_body_count=restricted_function_body_count,
        free_function_body_count=free_function_body_count,
        impl_function_body_count=impl_function_body_count,
        trait_default_function_body_count=trait_default_function_body_count,
        nested_function_body_count=nested_function_body_count,
        impl_block_count=len(impl_spans),
        static_item_count=static_item_count,
        macro_definition_count=macro_definition_count,
        macro_invocation_review_count=macro_invocation_review_count,
        trait_count=declarations["trait"],
        public_trait_count=public_declarations["trait"],
        struct_count=declarations["struct"],
        public_struct_count=public_declarations["struct"],
        enum_count=declarations["enum"],
        public_enum_count=public_declarations["enum"],
        union_count=declarations["union"],
        public_union_count=public_declarations["union"],
        type_alias_count=declarations["type"],
        public_type_alias_count=public_declarations["type"],
        nonempty_code_lines=sum(
            1 for line in production_code_view.splitlines() if line.strip()
        ),
    )


def _framework_source_inventory(
    runtime_source_root: Path, framework_root: Path
) -> tuple[set[Path], dict[Path, str], dict[Path, str]]:
    source_paths = set(framework_root.rglob("*.rs"))
    sources = {path: path.read_text(encoding="utf-8") for path in source_paths}
    edges: dict[Path, list[tuple[Path, bool]]] = {}
    code_views: dict[Path, str] = {}
    for path, source in sources.items():
        if re.search(r"\bmod\s", source) is not None:
            code_view = _rust_code_view(source)
            code_views[path] = code_view
            edges[path] = _module_edges(path, source, code_view)

    direct_test_paths = {
        path
        for path in source_paths
        if not is_production_rust_source(path.relative_to(runtime_source_root))
    }
    direct_test_paths.update(
        target
        for declarations in edges.values()
        for target, cfg_test in declarations
        if cfg_test and target in source_paths
    )

    test_reachable = set(direct_test_paths)
    pending = list(direct_test_paths)
    while pending:
        parent = pending.pop()
        for target, _cfg_test in edges.get(parent, []):
            if target in source_paths and target not in test_reachable:
                test_reachable.add(target)
                pending.append(target)

    production_reachable = source_paths - test_reachable
    changed = True
    while changed:
        changed = False
        for parent in tuple(production_reachable):
            for target, cfg_test in edges.get(parent, []):
                if (
                    target in source_paths
                    and not cfg_test
                    and target not in production_reachable
                ):
                    production_reachable.add(target)
                    changed = True

    return test_reachable - production_reachable, sources, code_views


def audit_framework_partition(repo_root: Path) -> dict[str, object]:
    resolved_repo_root = repo_root.resolve()
    runtime_source_root = resolved_repo_root / "zircon_runtime" / "src"
    framework_root = runtime_source_root / "core" / "framework"
    test_only_paths, sources, code_views = _framework_source_inventory(
        runtime_source_root, framework_root
    )
    file_rows: list[dict[str, object]] = []

    for source_path in sorted(framework_root.rglob("*.rs")):
        runtime_relative_path = source_path.relative_to(runtime_source_root)
        if (
            not is_production_rust_source(runtime_relative_path)
            or source_path in test_only_paths
        ):
            continue
        result = classify_rust_source(
            sources[source_path], code_views.get(source_path)
        )
        framework_relative_path = source_path.relative_to(framework_root)
        file_rows.append(
            {
                "path": source_path.relative_to(resolved_repo_root).as_posix(),
                "domain": (
                    framework_relative_path.parts[0]
                    if len(framework_relative_path.parts) > 1
                    else "_root"
                ),
                **asdict(result),
            }
        )

    classifications = Counter(row["classification"] for row in file_rows)
    metric_names = (
        "function_body_count",
        "public_function_body_count",
        "restricted_function_body_count",
        "free_function_body_count",
        "impl_function_body_count",
        "trait_default_function_body_count",
        "nested_function_body_count",
        "impl_block_count",
        "static_item_count",
        "macro_definition_count",
        "macro_invocation_review_count",
        "trait_count",
        "public_trait_count",
        "struct_count",
        "public_struct_count",
        "enum_count",
        "public_enum_count",
        "union_count",
        "public_union_count",
        "type_alias_count",
        "public_type_alias_count",
        "nonempty_code_lines",
    )
    totals = {
        "file_count": len(file_rows),
        **{
            name: sum(int(row[name]) for row in file_rows)
            for name in metric_names
        },
        "classification_counts": {
            name: classifications[name]
            for name in (
                "declaration_only",
                "contract_binding_only",
                "macro_generated_review",
                "mixed",
                "behavior_only",
            )
        },
        "manual_review_file_count": sum(
            bool(row["manual_review_required"]) for row in file_rows
        ),
    }

    domains: list[dict[str, object]] = []
    for domain in sorted({str(row["domain"]) for row in file_rows}):
        domain_rows = [row for row in file_rows if row["domain"] == domain]
        domain_classifications = Counter(
            row["classification"] for row in domain_rows
        )
        domains.append(
            {
                "domain": domain,
                "file_count": len(domain_rows),
                "function_body_count": sum(
                    int(row["function_body_count"]) for row in domain_rows
                ),
                "public_function_body_count": sum(
                    int(row["public_function_body_count"])
                    for row in domain_rows
                ),
                "restricted_function_body_count": sum(
                    int(row["restricted_function_body_count"])
                    for row in domain_rows
                ),
                "free_function_body_count": sum(
                    int(row["free_function_body_count"]) for row in domain_rows
                ),
                "impl_function_body_count": sum(
                    int(row["impl_function_body_count"]) for row in domain_rows
                ),
                "trait_default_function_body_count": sum(
                    int(row["trait_default_function_body_count"])
                    for row in domain_rows
                ),
                "impl_block_count": sum(
                    int(row["impl_block_count"]) for row in domain_rows
                ),
                "static_item_count": sum(
                    int(row["static_item_count"]) for row in domain_rows
                ),
                "macro_definition_count": sum(
                    int(row["macro_definition_count"]) for row in domain_rows
                ),
                "macro_invocation_review_count": sum(
                    int(row["macro_invocation_review_count"])
                    for row in domain_rows
                ),
                "nonempty_code_lines": sum(
                    int(row["nonempty_code_lines"]) for row in domain_rows
                ),
                "classification_counts": {
                    name: domain_classifications[name]
                    for name in (
                        "declaration_only",
                        "contract_binding_only",
                        "macro_generated_review",
                        "mixed",
                        "behavior_only",
                    )
                },
                "manual_review_file_count": sum(
                    bool(row["manual_review_required"]) for row in domain_rows
                ),
            }
        )

    return {
        "schema_version": 2,
        "source_root": "zircon_runtime/src/core/framework",
        "totals": totals,
        "domains": domains,
        "files": file_rows,
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Classify zircon_runtime core/framework production Rust files for the "
            "zr_contracts declaration/behavior hard partition."
        )
    )
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--output", type=Path)
    arguments = parser.parse_args()

    report = audit_framework_partition(arguments.repo_root)
    rendered = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if arguments.output is None:
        print(rendered, end="")
    else:
        arguments.output.parent.mkdir(parents=True, exist_ok=True)
        arguments.output.write_text(rendered, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
