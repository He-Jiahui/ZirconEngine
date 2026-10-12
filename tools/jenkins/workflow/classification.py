"""Conservative, read-only change classification for Jenkins admission.

Rust source is scanned with the small lexer in this module before a change is
classified. The lexer intentionally knows only the lexical layer: it keeps
documentation comments as tokens, removes ordinary comments and Rust's
``Pattern_White_Space``, and fails closed for malformed or ambiguous input.
"""
from __future__ import annotations

import json
import re
import tomllib
from pathlib import Path
from typing import Iterable, Mapping


# Rust's lexer uses the Unicode Pattern_White_Space property. Python's
# ``str.isspace`` is deliberately not used because its set differs.
_RUST_PATTERN_WHITESPACE = frozenset("\t\n\v\f\r \u0085\u200e\u200f\u2028\u2029")
_PUNCTUATION = tuple(sorted({
    "...", "..=", "..", "::", "=>", "->", "<<=", ">>=", "<<", ">>",
    "<=", ">=", "==", "!=", "&&", "||", "+=", "-=", "*=", "/=", "%=",
    "^=", "&=", "|=", "[", "]", "(", ")", "{", "}", ",", ";", ":",
    ".", "?", "+", "-", "*", "/", "%", "^", "!", "&", "|", "=", ">",
    "<", "@", "#", "$", "~",
}, key=len, reverse=True))
_OPEN_DELIMITERS = frozenset(("(", "[", "{"))
_LINE_SENSITIVE_NAMES = frozenset(("line", "column", "file"))
_SOURCE_EMBEDDING_NAMES = frozenset(("include", "include_str", "include_bytes"))
_CONDITIONAL_BUILD_NAMES = frozenset((
    "cfg", "cfg_attr", "env", "option_env", "compile_error",
    "include", "include_str", "include_bytes",
))
_CONDITIONAL_BUILD_ATTRIBUTES = frozenset((
    "cfg", "cfg_attr", "path", "link", "link_name", "link_section",
    "no_std", "no_main", "feature", "target_feature",
))
_RUSTC_DIRECTIVE_RE = re.compile(
    r"\b(?:rustc-[A-Za-z0-9_.-]+|cargo:(?:rustc|rerun-if|warning)-[A-Za-z0-9_.-]+)\b",
    re.IGNORECASE,
)


def _is_rust_whitespace(char: str) -> bool:
    return char in _RUST_PATTERN_WHITESPACE


def _is_ident_start(char: str) -> bool:
    return char == "_" or (bool(char) and (char.isalpha() or char.isidentifier()))


def _is_ident_continue(char: str) -> bool:
    return _is_ident_start(char) or (bool(char) and char.isdigit())


def _consume_identifier(text: str, start: int) -> int:
    if start >= len(text) or not _is_ident_start(text[start]):
        raise ValueError(f"identifier expected at offset {start}")
    end = start + 1
    while end < len(text) and _is_ident_continue(text[end]):
        end += 1
    return end


def _consume_suffix(text: str, end: int) -> int:
    while end < len(text) and _is_ident_continue(text[end]):
        end += 1
    return end


def _scan_quoted(text: str, quote_start: int, quote: str) -> int:
    """Return the exclusive end of a non-raw quoted literal."""
    end = quote_start + 1
    while end < len(text):
        char = text[end]
        if char == "\\":
            if end + 1 >= len(text):
                raise ValueError("unterminated escaped literal")
            end += 2
            continue
        if char == quote:
            return end + 1
        if char == "\r":
            # CRLF is normalized to LF by the Rust input phase. An isolated
            # CR remains an invalid literal character.
            if end + 1 < len(text) and text[end + 1] == "\n":
                end += 1
                continue
            raise ValueError("carriage return in literal")
        end += 1
    raise ValueError("unterminated literal")


def _scan_raw_literal(text: str, start: int, prefix_length: int) -> int | None:
    """Scan r/br/cr raw strings, returning None when no raw opener exists."""
    hash_start = start + prefix_length
    hash_end = hash_start
    while hash_end < len(text) and text[hash_end] == "#":
        hash_end += 1
    hash_count = hash_end - hash_start
    if hash_end >= len(text) or text[hash_end] != '"':
        return None
    if hash_count > 255:
        raise ValueError("raw literal has more than 255 hash delimiters")
    body_start = hash_end + 1
    cursor = body_start
    while True:
        quote = text.find('"', cursor)
        if quote < 0:
            raise ValueError("unterminated raw literal")
        if text.startswith("#" * hash_count, quote + 1):
            body = text[body_start:quote]
            if any(char == "\r" and (index + 1 >= len(body) or body[index + 1] != "\n")
                   for index, char in enumerate(body)):
                raise ValueError("carriage return in raw literal")
            return _consume_suffix(text, quote + 1 + hash_count)
        cursor = quote + 1


def _scan_number(text: str, start: int) -> int:
    """Consume a numeric literal sufficiently precisely for token equality."""
    end = start
    if text.startswith(("0x", "0X", "0b", "0B", "0o", "0O"), start):
        prefix = text[start:start + 2]
        end = start + 2
        digit_start = end
        valid = "0123456789abcdefABCDEF_" if prefix[1] in "xX" else (
            "01_" if prefix[1] in "bB" else "01234567_")
        while end < len(text) and text[end] in valid:
            end += 1
        if end == digit_start:
            raise ValueError("numeric literal has no digits")
        return _consume_suffix(text, end)
    while end < len(text) and (text[end].isdigit() or text[end] == "_"):
        end += 1
    if end < len(text) and text[end] == "." and not text.startswith("..", end):
        end += 1
        while end < len(text) and (text[end].isdigit() or text[end] == "_"):
            end += 1
    if end < len(text) and text[end] in "eE":
        exponent = end + 1
        if exponent < len(text) and text[exponent] in "+-":
            exponent += 1
        digits = exponent
        while exponent < len(text) and (text[exponent].isdigit() or text[exponent] == "_"):
            exponent += 1
        if exponent == digits:
            raise ValueError("numeric literal has an incomplete exponent")
        end = exponent
    return _consume_suffix(text, end)


def _scan_apostrophe(text: str, start: int) -> int:
    """Scan a character literal or a lifetime/label beginning at ``'``."""
    n = len(text)
    if start + 2 < n and text[start + 1] == "\\":
        escape = start + 2
        if text[escape] == "x":
            escape += 3
        elif text[escape] == "u" and escape + 1 < n and text[escape + 1] == "{":
            close = text.find("}", escape + 2)
            if close < 0:
                raise ValueError("unterminated Unicode character escape")
            escape = close + 1
        elif text[escape] in "nrt\\'\"0":
            escape += 1
        else:
            raise ValueError("invalid character escape")
        if escape < n and text[escape] == "'":
            return _consume_suffix(text, escape + 1)
    elif start + 2 < n and text[start + 1] not in "\n\r\t" and text[start + 2] == "'":
        return _consume_suffix(text, start + 3)

    end = start + 1
    if end < n and text.startswith("r#", end):
        end += 2
        if end >= n or not _is_ident_start(text[end]):
            raise ValueError("invalid raw lifetime")
        return _consume_identifier(text, end)
    if end < n and _is_ident_start(text[end]):
        return _consume_identifier(text, end)
    raise ValueError("ambiguous character or lifetime token")


def _comment_is_doc(text: str, start: int, *, block: bool) -> bool:
    if block:
        if text.startswith("/*!", start):
            return True
        # `/**/` is empty ordinary block comment; `/***` begins ordinary
        # comment text. A doc block needs a character after the `/**` opener.
        return text.startswith("/**", start) and (
            start + 3 >= len(text) or text[start + 3] not in "*/"
        )
    if text.startswith("//!", start):
        return True
    return text.startswith("///", start) and not text.startswith("////", start)


def _ordered_unique(values: Iterable[str]) -> tuple[str, ...]:
    seen: set[str] = set()
    result: list[str] = []
    for value in values:
        if value not in seen:
            seen.add(value)
            result.append(value)
    return tuple(result)


def _macro_calls(tokens: list[str], names: frozenset[str]) -> tuple[str, ...]:
    calls: list[str] = []
    for index in range(len(tokens) - 2):
        if tokens[index] in names and tokens[index + 1] == "!" and tokens[index + 2] in _OPEN_DELIMITERS:
            calls.append(tokens[index] + "!")
    return _ordered_unique(calls)


def rust_lexical_evidence(text: str) -> dict:
    """Return conservative lexical evidence for a Rust source string."""
    tokens: list[str] = []
    token_spans: list[tuple[int, int]] = []
    ordinary_comment_spans: list[tuple[int, int]] = []
    doc_comment_spans: list[tuple[int, int]] = []
    i = 0
    n = len(text)

    def add_token(start: int, end: int) -> None:
        tokens.append(text[start:end])
        token_spans.append((start, end))

    while i < n:
        char = text[i]
        if i == 0 and char == "\ufeff":
            # Rust removes a leading UTF-8 byte order mark before lexing.
            i += 1
            continue
        if _is_rust_whitespace(char):
            i += 1
            continue
        # Rust removes a shebang before tokenization.
        if i == 0 and text.startswith("#!", i) and (i + 2 >= n or text[i + 2] != "["):
            end = text.find("\n", i + 2)
            i = n if end < 0 else end
            continue
        if text.startswith("//", i):
            end = text.find("\n", i + 2)
            end = n if end < 0 else end
            if _comment_is_doc(text, i, block=False):
                add_token(i, end)
                doc_comment_spans.append((i, end))
            else:
                ordinary_comment_spans.append((i, end))
            i = end
            continue
        if text.startswith("/*", i):
            doc = _comment_is_doc(text, i, block=True)
            depth = 1
            end = i + 2
            while end < n and depth:
                if text.startswith("/*", end):
                    depth += 1
                    end += 2
                elif text.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            if depth:
                raise ValueError("unterminated block comment")
            if doc:
                add_token(i, end)
                doc_comment_spans.append((i, end))
            else:
                ordinary_comment_spans.append((i, end))
            i = end
            continue

        raw_end = None
        for prefix in ("br", "cr", "r"):
            if text.startswith(prefix, i):
                raw_end = _scan_raw_literal(text, i, len(prefix))
                if raw_end is not None:
                    break
        if raw_end is not None:
            add_token(i, raw_end)
            i = raw_end
            continue
        if char == '"':
            end = _consume_suffix(text, _scan_quoted(text, i, '"'))
            add_token(i, end)
            i = end
            continue
        if text.startswith('b"', i) or text.startswith('c"', i):
            end = _consume_suffix(text, _scan_quoted(text, i + 1, '"'))
            add_token(i, end)
            i = end
            continue
        if text.startswith("b'", i):
            end = _scan_apostrophe(text, i + 1)
            add_token(i, end)
            i = end
            continue
        if char == "'":
            end = _scan_apostrophe(text, i)
            add_token(i, end)
            i = end
            continue
        if char.isdigit():
            end = _scan_number(text, i)
            add_token(i, end)
            i = end
            continue
        if text.startswith("r#", i) and i + 2 < n and _is_ident_start(text[i + 2]):
            end = _consume_identifier(text, i + 2)
            add_token(i, end)
            i = end
            continue
        if _is_ident_start(char):
            end = _consume_identifier(text, i)
            add_token(i, end)
            i = end
            continue
        punctuation = next((candidate for candidate in _PUNCTUATION if text.startswith(candidate, i)), None)
        if punctuation is not None:
            add_token(i, i + len(punctuation))
            i += len(punctuation)
            continue
        raise ValueError(f"unrecognized Rust lexical input at offset {i}")

    line_sensitive_macros = _macro_calls(tokens, _LINE_SENSITIVE_NAMES)
    source_embedding_macros = _macro_calls(tokens, _SOURCE_EMBEDDING_NAMES)
    conditional_build_sensitive_macros = _macro_calls(tokens, _CONDITIONAL_BUILD_NAMES)
    conditional_build_sensitive_attributes: list[str] = []
    for index, token in enumerate(tokens):
        if token != "#":
            continue
        bracket = index + 1
        if bracket < len(tokens) and tokens[bracket] == "!":
            bracket += 1
        if bracket + 1 < len(tokens) and tokens[bracket] == "[":
            name = tokens[bracket + 1]
            if name in _CONDITIONAL_BUILD_ATTRIBUTES:
                conditional_build_sensitive_attributes.append(name)
    attrs = _ordered_unique(conditional_build_sensitive_attributes)

    rustc_directive_comment_spans: list[tuple[int, int]] = []
    rustc_directive_comment_markers: list[str] = []
    for start, end in tuple(ordinary_comment_spans) + tuple(doc_comment_spans):
        matches = _RUSTC_DIRECTIVE_RE.findall(text[start:end])
        if matches:
            rustc_directive_comment_spans.append((start, end))
            rustc_directive_comment_markers.extend(matches)

    directive_markers = tuple(rustc_directive_comment_markers)
    return {
        "tokens": tuple(tokens),
        "token_spans": tuple(token_spans),
        "ordinary_comment_spans": tuple(ordinary_comment_spans),
        "doc_comment_spans": tuple(doc_comment_spans),
        "line_sensitive_macros": line_sensitive_macros,
        "source_embedding_macros": source_embedding_macros,
        "conditional_build_sensitive_macros": conditional_build_sensitive_macros,
        "conditional_build_sensitive_attributes": attrs,
        "rustc_directive_comment_spans": tuple(rustc_directive_comment_spans),
        "rustc_directive_comment_markers": directive_markers,
        # Friendly aliases used by callers that describe these as flags.
        "rustc_directive_comments": bool(directive_markers),
        "conditional_build_sensitive": bool(conditional_build_sensitive_macros or attrs),
    }


def rust_non_doc_tokens(text: str) -> tuple[str, ...]:
    """Return compiler-facing Rust tokens with ordinary comments removed."""
    # Documentation comments intentionally remain in this stream. Despite the
    # historical name, callers use the stream to prove that a doc-comment edit
    # is observable and therefore semantic.
    return rust_lexical_evidence(text)["tokens"]


def _span_overlaps_lines(text: str, span: tuple[int, int], selected: set[int]) -> bool:
    if not selected or span[0] >= span[1]:
        return False
    line = text.count("\n", 0, span[0]) + 1
    last_line = text.count("\n", 0, span[1] - 1) + 1
    return any(line <= selected_line <= last_line for selected_line in selected)


def classify_rust_source(text: str, *, changed_lines: Iterable[int] | None = None) -> dict:
    """Return conservative evidence for a Rust source fragment.

    ``changed_lines=None`` selects the whole file. Selected code lines are
    semantic even when an ordinary inline comment is also present.
    """
    lines = text.splitlines()
    selected = set(range(1, len(lines) + 1)) if changed_lines is None else set(changed_lines)
    if any(line < 1 or line > len(lines) for line in selected):
        return {"kind": "blocked", "reason": "changed_line_out_of_range"}
    try:
        evidence = rust_lexical_evidence(text)
    except ValueError as error:
        reason = "unterminated_block_comment" if "block comment" in str(error) else "rust_lexical_error"
        return {"kind": "blocked", "reason": reason}
    if evidence["rustc_directive_comment_markers"]:
        return {"kind": "semantic", "reason": "build_or_conditional_directive"}
    if evidence["conditional_build_sensitive_macros"] or evidence["conditional_build_sensitive_attributes"]:
        return {"kind": "semantic", "reason": "build_or_conditional_directive"}
    if evidence["source_embedding_macros"]:
        return {"kind": "semantic", "reason": "build_or_conditional_directive"}
    if evidence["line_sensitive_macros"]:
        return {"kind": "semantic", "reason": "line_sensitive_macro"}
    for span in evidence["doc_comment_spans"]:
        if _span_overlaps_lines(text, span, selected):
            return {"kind": "semantic", "reason": "documentation_comment"}
    for span in evidence["token_spans"]:
        if _span_overlaps_lines(text, span, selected):
            return {"kind": "semantic", "reason": "non_comment_token"}
    return {"kind": "comments_only", "reason": "all_changed_tokens_are_non_doc_comments"}


def classify_path(path: str | Path, content: str | None = None, *, changed_lines: Iterable[int] | None = None) -> dict:
    p = Path(path)
    suffix = p.suffix.lower()
    if content is None:
        content = p.read_text(encoding="utf-8")
    if suffix == ".rs":
        return classify_rust_source(content, changed_lines=changed_lines)
    if suffix == ".py":
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
    results = {
        path: classify_path(path, content, changed_lines=(changed_lines or {}).get(path))
        for path, content in files.items()
    }
    kinds = {result["kind"] for result in results.values()}
    if not results:
        return {"kind": "blocked", "reason": "empty_change", "files": results}
    if kinds == {"comments_only"}:
        return {"kind": "comments_only", "files": results}
    if "blocked" in kinds:
        return {"kind": "blocked", "reason": "unproven_file_scope", "files": results}
    return {"kind": "semantic", "files": results}


__all__ = [
    "classify_change", "classify_rust_source", "classify_path",
    "rust_lexical_evidence", "rust_non_doc_tokens",
]
