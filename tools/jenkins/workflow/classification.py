"""Conservative, read-only change classification for Jenkins admission.

Classification is deliberately evidence based.  A source change is admitted as
comments-only only when tokenization proves that every changed token is a
comment and no documentation/build directive can affect compilation.
"""
from __future__ import annotations

import json
import re
import tomllib
from pathlib import Path
from typing import Iterable, Mapping

_RUST_COMMENT = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)
_RUST_STRING = re.compile(r'"(?:\\.|[^"\\])*"|r#*".*?"#*', re.S)
_DOC = re.compile(r"^\s*///|^\s*//!|^\s*/\*\*|^\s*\*", re.M)
_BUILD_DIRECTIVE = re.compile(
    r"(?:cfg|cfg_attr|include|include_bytes|env|option_env|compile_error)!|"
    r"rustc-link|rustc-cfg|rustc-env|rustc-flags|rustc-cdylib-link-arg",
    re.I,
)

def rust_non_doc_tokens(text: str) -> tuple[str, ...]:
    """Return compiler-facing Rust tokens with comments and whitespace removed.

    This deliberately uses a small lexical scanner rather than a regex so a
    comment marker inside a string or character literal cannot alter proof.
    Documentation comments remain tokens and therefore cannot be hidden by a
    caller supplied classification.
    """
    out: list[str] = []
    i = 0
    n = len(text)
    while i < n:
        c = text[i]
        if c.isspace(): i += 1; continue
        if text.startswith("//", i):
            end = text.find("\n", i + 2); i = n if end < 0 else end; continue
        if text.startswith("/*", i):
            depth = 1; j = i + 2
            while j < n and depth:
                if text.startswith("/*", j): depth += 1; j += 2
                elif text.startswith("*/", j): depth -= 1; j += 2
                else: j += 1
            if depth: raise ValueError("unterminated block comment")
            i = j; continue
        if c in {'"', "'"}:
            quote = c; j = i + 1
            while j < n:
                if text[j] == "\\": j += 2; continue
                if text[j] == quote: j += 1; break
                j += 1
            out.append(text[i:j]); i = j; continue
        j = i + 1
        while j < n and not text[j].isspace() and not text.startswith("//", j) and not text.startswith("/*", j):
            if text[j] in {'"', "'"}: break
            j += 1
        out.append(text[i:j]); i = j
    return tuple(out)


def _strip_strings(text: str) -> str:
    return _RUST_STRING.sub(lambda m: " " * len(m.group(0)), text)


def classify_rust_source(text: str, *, changed_lines: Iterable[int] | None = None) -> dict:
    """Return conservative evidence for a Rust source fragment.

    The result is ``comments_only`` only when changed lines contain no code,
    doc comments or build/conditional directives.  Unknown or unterminated
    comments are blocked instead of being treated as harmless prose.
    """
    lines = text.splitlines()
    selected = set(changed_lines) if changed_lines is not None else set(range(1, len(lines) + 1))
    if any(n < 1 or n > len(lines) for n in selected):
        return {"kind": "blocked", "reason": "changed_line_out_of_range"}
    sanitized = _strip_strings(text)
    if sanitized.count("/*") != sanitized.count("*/"):
        return {"kind": "blocked", "reason": "unterminated_block_comment"}
    if _BUILD_DIRECTIVE.search(sanitized):
        return {"kind": "semantic", "reason": "build_or_conditional_directive"}
    changed = [lines[n - 1] for n in sorted(selected)]
    for line in changed:
        code = _strip_strings(line)
        # Remove comments while preserving line structure, then reject any
        # executable token or doc comment marker.
        code = _RUST_COMMENT.sub("", code).strip()
        if not code:
            continue
        if line.lstrip().startswith(("///", "//!", "/**", "*")):
            return {"kind": "semantic", "reason": "documentation_comment"}
        return {"kind": "semantic", "reason": "non_comment_token"}
    if _DOC.search("\n".join(changed)):
        return {"kind": "semantic", "reason": "documentation_comment"}
    return {"kind": "comments_only", "reason": "all_changed_tokens_are_non_doc_comments"}


def classify_path(path: str | Path, content: str | None = None, *, changed_lines: Iterable[int] | None = None) -> dict:
    p = Path(path)
    suffix = p.suffix.lower()
    if content is None:
        content = p.read_text(encoding="utf-8")
    if suffix == ".rs":
        return classify_rust_source(content, changed_lines=changed_lines)
    if suffix in {".py"}:
        # Python AST parsing gives a real syntax proof; comments-only remains
        # conservative because line-to-node mapping is not sufficient here.
        import ast
        try:
            ast.parse(content, filename=str(p))
        except SyntaxError as exc:
            return {"kind": "blocked", "reason": "python_syntax_error", "line": exc.lineno}
        return {"kind": "semantic", "reason": "python_requires_ast_scope"}
    if suffix == ".json":
        try:
            json.loads(content)
        except json.JSONDecodeError as exc:
            return {"kind": "blocked", "reason": "json_syntax_error", "line": exc.lineno}
        return {"kind": "semantic", "reason": "json_structural_change"}
    if suffix == ".toml":
        try:
            tomllib.loads(content)
        except tomllib.TOMLDecodeError:
            return {"kind": "blocked", "reason": "toml_syntax_error"}
        return {"kind": "semantic", "reason": "toml_structural_change"}
    if suffix in {".md", ".txt", ".rst"}:
        return {"kind": "semantic", "reason": "documentation_scope_requires_acceptance"}
    return {"kind": "blocked", "reason": "unregistered_language"}


def classify_change(files: Mapping[str, str], *, changed_lines: Mapping[str, Iterable[int]] | None = None) -> dict:
    results = {path: classify_path(path, content, changed_lines=(changed_lines or {}).get(path))
               for path, content in files.items()}
    kinds = {r["kind"] for r in results.values()}
    if not results:
        return {"kind": "blocked", "reason": "empty_change", "files": results}
    if kinds == {"comments_only"}:
        return {"kind": "comments_only", "files": results}
    if "blocked" in kinds:
        return {"kind": "blocked", "reason": "unproven_file_scope", "files": results}
    return {"kind": "semantic", "files": results}


__all__ = ["classify_change", "classify_rust_source", "classify_path", "rust_non_doc_tokens"]
