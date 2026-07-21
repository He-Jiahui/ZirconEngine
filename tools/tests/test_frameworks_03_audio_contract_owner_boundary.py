from __future__ import annotations

import re
import unittest
from functools import cache
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIO_ROOT = REPO_ROOT / "zircon_runtime" / "src" / "core" / "framework" / "audio"
SOUND_ROOT = REPO_ROOT / "zircon_runtime" / "src" / "core" / "framework" / "sound"
RUNTIME_LIB = REPO_ROOT / "zircon_runtime" / "src" / "lib.rs"
SOUND_LAYOUT_CONSUMERS = (
    "components.rs",
    "graph.rs",
    "mix.rs",
    "options.rs",
    "output.rs",
    "playback.rs",
    "status.rs",
)

RUST_TOKEN = re.compile(
    r"r#[A-Za-z_][A-Za-z0-9_]*|::|[A-Za-z_][A-Za-z0-9_]*|\*|[#\[\]{}(),;!=]"
)
RustToken = tuple[str, int, int]
UseLeaf = tuple[tuple[str, ...], str, str | None]
CfgCondition = tuple[str, ...]
UseDeclaration = tuple[bool, CfgCondition, tuple[UseLeaf, ...]]
AliasBinding = tuple[tuple[str, ...], CfgCondition]
SOUND_MODULE_PATH = ("crate", "core", "framework", "sound")
NEUTRAL_AUDIO_MODULE_PATH = ("crate", "core", "framework", "audio")
CFG_TOKEN = re.compile(r'[A-Za-z_][A-Za-z0-9_]*|"(?:\\.|[^"\\])*"|=|[(),]')
SINGLE_VALUE_CFG_KEYS = frozenset(
    {
        "panic",
        "target_arch",
        "target_endian",
        "target_env",
        "target_os",
        "target_pointer_width",
        "target_vendor",
    }
)


def rust_code_without_comments_and_literals(source: str) -> str:
    masked = list(source)

    def mask(start: int, end: int) -> None:
        for index in range(start, end):
            if masked[index] not in "\r\n":
                masked[index] = " "

    index = 0
    while index < len(source):
        if source.startswith("//", index):
            end = source.find("\n", index + 2)
            end = len(source) if end < 0 else end
            mask(index, end)
            index = end
            continue
        if source.startswith("/*", index):
            depth = 1
            end = index + 2
            while end < len(source) and depth:
                if source.startswith("/*", end):
                    depth += 1
                    end += 2
                elif source.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            mask(index, end)
            index = end
            continue

        raw_string = re.match(r'(?:br|rb|r)(?P<hashes>#{0,255})"', source[index:])
        if raw_string is not None:
            delimiter = '"' + raw_string.group("hashes")
            content_start = index + raw_string.end()
            close = source.find(delimiter, content_start)
            end = len(source) if close < 0 else close + len(delimiter)
            mask(index, end)
            index = end
            continue
        if source[index] == '"':
            end = index + 1
            while end < len(source):
                if source[end] == "\\":
                    end += 2
                elif source[end] == '"':
                    end += 1
                    break
                else:
                    end += 1
            mask(index, min(end, len(source)))
            index = end
            continue

        character = re.match(r"'(?:\\.|[^\\'\r\n])'", source[index:])
        if character is not None:
            end = index + character.end()
            mask(index, end)
            index = end
            continue
        index += 1

    return "".join(masked)


@cache
def crate_self_aliases() -> frozenset[str]:
    source = RUNTIME_LIB.read_text(encoding="utf-8")
    code = rust_code_without_comments_and_literals(source)
    return frozenset(
        rust_identifier_name(match.group("alias")) or match.group("alias")
        for match in re.finditer(
            r"\bextern\s+crate\s+self\s+as\s+"
            r"(?P<alias>(?:r#)?[A-Za-z_][A-Za-z0-9_]*)\s*;",
            code,
        )
    )


def normalized_use_path(
    prefix: tuple[str, ...], segments: list[str]
) -> tuple[str, ...]:
    if prefix and segments and segments[0] == "self":
        return prefix + tuple(segments[1:])
    return prefix + tuple(segments)


def rust_identifier_name(token: str) -> str | None:
    if re.fullmatch(r"r#[A-Za-z_][A-Za-z0-9_]*", token):
        return token[2:]
    if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", token):
        return token
    return None


def parse_use_tree(
    tokens: tuple[str, ...],
    index: int = 0,
    prefix: tuple[str, ...] = (),
) -> tuple[list[UseLeaf], int]:
    segments: list[str] = []
    while index < len(tokens):
        token = tokens[index]
        if token == "::":
            index += 1
            continue
        identifier = rust_identifier_name(token)
        if identifier is not None and token != "as":
            segments.append(identifier)
            index += 1
            continue
        break

    path = normalized_use_path(prefix, segments)
    if index < len(tokens) and tokens[index] == "{":
        leaves: list[UseLeaf] = []
        index += 1
        while index < len(tokens) and tokens[index] != "}":
            child_leaves, index = parse_use_tree(tokens, index, path)
            leaves.extend(child_leaves)
            if index < len(tokens) and tokens[index] == ",":
                index += 1
        if index < len(tokens) and tokens[index] == "}":
            index += 1
        return leaves, index

    if index < len(tokens) and tokens[index] == "*":
        return [(path, "glob", None)], index + 1

    alias = None
    if index < len(tokens) and tokens[index] == "as":
        index += 1
        if index < len(tokens):
            alias = rust_identifier_name(tokens[index]) or tokens[index]
            index += 1
    return ([(path, "name", alias)] if path else []), index


def public_visibility_end(tokens: tuple[str, ...], index: int) -> int:
    index += 1
    if index >= len(tokens) or tokens[index] != "(":
        return index

    depth = 1
    index += 1
    while index < len(tokens) and depth:
        if tokens[index] == "(":
            depth += 1
        elif tokens[index] == ")":
            depth -= 1
        index += 1
    return index


def cfg_condition_from_attribute(source: str, start: int, end: int) -> str | None:
    attribute = source[start:end]
    match = re.fullmatch(
        r"\s*#\s*!?\s*\[\s*cfg\s*\((?P<predicate>.*)\)\s*\]\s*",
        attribute,
        re.DOTALL,
    )
    if match is None:
        return None
    return re.sub(r"\s+", "", match.group("predicate"))


def root_macro_invocation_end(tokens: tuple[str, ...], index: int) -> int | None:
    start = index
    if index < len(tokens) and tokens[index] == "::":
        index += 1
    if index >= len(tokens) or rust_identifier_name(tokens[index]) is None:
        return None
    index += 1
    while (
        index + 1 < len(tokens)
        and tokens[index] == "::"
        and rust_identifier_name(tokens[index + 1]) is not None
    ):
        index += 2
    if index >= len(tokens) or tokens[index] != "!":
        return None
    if start + 1 == index and tokens[start] == "macro_rules":
        return None
    return index + 1


def use_declarations(
    source: str, tokens: tuple[RustToken, ...]
) -> tuple[tuple[UseDeclaration, ...], bool, bool]:
    token_text = tuple(token[0] for token in tokens)
    declarations: list[UseDeclaration] = []
    public_audio_module = False
    root_macro_expansion = False
    brace_depth = 0
    at_item_start = True
    global_cfg: list[str] = []
    pending_cfg: list[str] = []
    index = 0
    while index < len(tokens):
        token = token_text[index]
        if token == "{":
            brace_depth += 1
            index += 1
            continue
        if token == "}":
            brace_depth = max(0, brace_depth - 1)
            if brace_depth == 0:
                at_item_start = True
                pending_cfg.clear()
            index += 1
            continue
        if brace_depth:
            index += 1
            continue
        if token == ";":
            at_item_start = True
            pending_cfg.clear()
            index += 1
            continue
        if not at_item_start:
            index += 1
            continue

        if token == "#":
            bracket = index + 1
            inner_attribute = bracket < len(tokens) and token_text[bracket] == "!"
            if inner_attribute:
                bracket += 1
            if bracket < len(tokens) and token_text[bracket] == "[":
                depth = 1
                attribute_end = bracket + 1
                while attribute_end < len(tokens) and depth:
                    if token_text[attribute_end] == "[":
                        depth += 1
                    elif token_text[attribute_end] == "]":
                        depth -= 1
                    attribute_end += 1
                if depth == 0:
                    condition = cfg_condition_from_attribute(
                        source,
                        tokens[index][1],
                        tokens[attribute_end - 1][2],
                    )
                    if condition is not None:
                        (global_cfg if inner_attribute else pending_cfg).append(condition)
                    index = attribute_end
                    continue

        macro_end = root_macro_invocation_end(token_text, index)
        if macro_end is not None:
            root_macro_expansion = root_macro_expansion or cfg_conditions_can_overlap(
                tuple(global_cfg + pending_cfg)
            )
            at_item_start = False
            pending_cfg.clear()
            index = macro_end
            continue

        public = token == "pub"
        declaration_start = (
            public_visibility_end(token_text, index) if public else index
        )
        if declaration_start >= len(tokens):
            break

        keyword = token_text[declaration_start]
        if public and keyword == "mod":
            public_audio_module = (
                public_audio_module
                or cfg_conditions_can_overlap(tuple(global_cfg + pending_cfg))
                and declaration_start + 1 < len(tokens)
                and token_text[declaration_start + 1] == "audio"
            )
            at_item_start = False
            pending_cfg.clear()
            index = declaration_start + 1
            continue
        if keyword == "extern":
            statement_end = declaration_start + 1
            while statement_end < len(tokens) and token_text[statement_end] != ";":
                statement_end += 1
            statement = token_text[declaration_start + 1 : statement_end]
            if len(statement) >= 2 and statement[:2] == ("crate", "self"):
                alias = "self"
                if len(statement) >= 4 and statement[2] == "as":
                    alias = rust_identifier_name(statement[3]) or statement[3]
                declarations.append(
                    (
                        public,
                        tuple(global_cfg + pending_cfg),
                        ((('crate',), "name", alias),),
                    )
                )
            at_item_start = True
            pending_cfg.clear()
            index = min(statement_end + 1, len(tokens))
            continue
        if keyword != "use":
            at_item_start = False
            pending_cfg.clear()
            index += 1
            continue

        statement_end = declaration_start + 1
        while statement_end < len(tokens) and token_text[statement_end] != ";":
            statement_end += 1
        leaves, _ = parse_use_tree(
            token_text[declaration_start + 1 : statement_end]
        )
        declarations.append(
            (public, tuple(global_cfg + pending_cfg), tuple(leaves))
        )
        at_item_start = True
        pending_cfg.clear()
        index = min(statement_end + 1, len(tokens))
    return tuple(declarations), public_audio_module, root_macro_expansion


@cache
def parse_cfg_expression(predicate: str) -> tuple[object, ...] | None:
    tokens = tuple(CFG_TOKEN.findall(predicate))

    def parse(index: int) -> tuple[tuple[object, ...] | None, int]:
        if index >= len(tokens):
            return None, index
        operator = tokens[index]
        if (
            operator in {"all", "any", "not"}
            and index + 1 < len(tokens)
            and tokens[index + 1] == "("
        ):
            children: list[tuple[object, ...]] = []
            index += 2
            while index < len(tokens) and tokens[index] != ")":
                child, index = parse(index)
                if child is None:
                    return None, index
                children.append(child)
                if index < len(tokens) and tokens[index] == ",":
                    index += 1
            if index >= len(tokens) or tokens[index] != ")":
                return None, index
            if operator == "not" and len(children) != 1:
                return None, index + 1
            return (operator, tuple(children)), index + 1

        atom: list[str] = []
        while index < len(tokens) and tokens[index] not in {",", ")"}:
            atom.append(tokens[index])
            index += 1
        return (("atom", "".join(atom)), index) if atom else (None, index)

    expression, end = parse(0)
    return expression if expression is not None and end == len(tokens) else None


def cfg_expression_atoms(expression: tuple[object, ...]) -> set[str]:
    if expression[0] == "atom":
        return {str(expression[1])}
    return {
        atom
        for child in expression[1]
        for atom in cfg_expression_atoms(child)
    }


def evaluate_cfg_expression(
    expression: tuple[object, ...], values: dict[str, bool]
) -> bool:
    operator = expression[0]
    if operator == "atom":
        return values[str(expression[1])]
    children = expression[1]
    if operator == "not":
        return not evaluate_cfg_expression(children[0], values)
    if operator == "all":
        return all(evaluate_cfg_expression(child, values) for child in children)
    return any(evaluate_cfg_expression(child, values) for child in children)


def cfg_assignment_is_valid(values: dict[str, bool]) -> bool:
    selected_values: dict[str, set[str]] = {}
    for atom, selected in values.items():
        if not selected:
            continue
        assignment = re.fullmatch(
            r"(?P<key>[A-Za-z_][A-Za-z0-9_]*)=(?P<value>.+)", atom
        )
        if assignment is None:
            continue
        key = assignment.group("key")
        value = assignment.group("value")
        selected_values.setdefault(key, set()).add(value)
        if key in SINGLE_VALUE_CFG_KEYS and len(selected_values[key]) > 1:
            return False

    if values.get("windows", False) and values.get("unix", False):
        return False
    target_os = next(iter(selected_values.get("target_os", {""}))).strip('"')
    target_families = {
        value.strip('"') for value in selected_values.get("target_family", set())
    }
    windows_target = target_os == "windows" or "windows" in target_families
    unix_target = "unix" in target_families or target_os in {
        "aix",
        "android",
        "dragonfly",
        "freebsd",
        "fuchsia",
        "haiku",
        "hurd",
        "illumos",
        "ios",
        "linux",
        "macos",
        "netbsd",
        "openbsd",
        "redox",
        "solaris",
        "tvos",
        "visionos",
        "watchos",
    }
    if target_os and target_os != "windows" and values.get("windows", False):
        return False
    if values.get("windows", False) and (
        values.get('target_os="windows"') is False
        or values.get('target_family="windows"') is False
    ):
        return False
    if windows_target and "windows" in values and not values["windows"]:
        return False
    if windows_target and values.get("unix", False):
        return False
    if unix_target and values.get("windows", False):
        return False
    if values.get("unix", False) and values.get('target_family="unix"') is False:
        return False
    if unix_target and "unix" in values and not values["unix"]:
        return False
    return True


@cache
def cfg_conditions_can_overlap(conditions: CfgCondition) -> bool:
    predicates = set(conditions)
    for predicate in predicates:
        if predicate.startswith("not(") and predicate.endswith(")"):
            opposite = predicate[4:-1]
        else:
            opposite = f"not({predicate})"
        if opposite in predicates:
            return False

    expressions = tuple(parse_cfg_expression(predicate) for predicate in conditions)
    if any(expression is None for expression in expressions):
        return True
    parsed = tuple(expression for expression in expressions if expression is not None)
    atoms = sorted(
        {atom for expression in parsed for atom in cfg_expression_atoms(expression)}
    )
    if len(atoms) > 12:
        return True
    for assignment in range(1 << len(atoms)):
        values = {
            atom: bool(assignment & (1 << index))
            for index, atom in enumerate(atoms)
        }
        if cfg_assignment_is_valid(values) and all(
            evaluate_cfg_expression(expression, values) for expression in parsed
        ):
            return True
    return False


def resolve_use_aliases(
    path: tuple[str, ...],
    aliases: dict[str, list[AliasBinding]],
    condition: CfgCondition,
) -> tuple[tuple[str, ...], ...]:
    resolved: set[tuple[str, ...]] = set()

    def visit(
        current: tuple[str, ...],
        active_condition: CfgCondition,
        visited: frozenset[str],
    ) -> None:
        if not current or current[0] not in aliases or current[0] in visited:
            resolved.add(current)
            return

        matched = False
        for target, alias_condition in aliases[current[0]]:
            combined_condition = active_condition + alias_condition
            if not cfg_conditions_can_overlap(combined_condition):
                continue
            matched = True
            visit(
                target + current[1:],
                combined_condition,
                visited | {current[0]},
            )
        if not matched:
            resolved.add(current)

    visit(path, condition, frozenset())
    return tuple(sorted(resolved))


def absolute_sound_module_path(path: tuple[str, ...]) -> tuple[str, ...]:
    if not path:
        return ()
    if path[0] == "crate":
        return path
    if path[0] in crate_self_aliases():
        return ("crate",) + path[1:]
    if path[0] == "self":
        return SOUND_MODULE_PATH + path[1:]
    if path[0] != "super":
        return SOUND_MODULE_PATH + path

    base = list(SOUND_MODULE_PATH)
    index = 0
    while index < len(path) and path[index] == "super":
        if len(base) > 1:
            base.pop()
        index += 1
    return tuple(base) + path[index:]


def exposes_neutral_audio_owner(path: tuple[str, ...]) -> bool:
    absolute = absolute_sound_module_path(path)
    if not absolute:
        return False
    common_length = min(len(absolute), len(NEUTRAL_AUDIO_MODULE_PATH))
    return absolute[:common_length] == NEUTRAL_AUDIO_MODULE_PATH[:common_length]


def neutral_audio_namespace_exports(source: str) -> tuple[str, ...]:
    code = rust_code_without_comments_and_literals(source)
    tokens = tuple(
        (match.group(0), match.start(), match.end())
        for match in RUST_TOKEN.finditer(code)
    )
    declarations, public_audio_module, root_macro_expansion = use_declarations(
        source, tokens
    )

    aliases: dict[str, list[AliasBinding]] = {}
    for public, condition, leaves in declarations:
        if public:
            continue
        for path, kind, explicit_alias in leaves:
            if kind != "name" or not path or explicit_alias == "_":
                continue
            aliases.setdefault(explicit_alias or path[-1], []).append(
                (path, condition)
            )

    exports: list[str] = []
    for public, condition, leaves in declarations:
        if not public or not cfg_conditions_can_overlap(condition):
            continue
        for path, kind, _ in leaves:
            for resolved in resolve_use_aliases(path, aliases, condition):
                if exposes_neutral_audio_owner(resolved):
                    absolute = absolute_sound_module_path(resolved)
                    exports.append(
                        "::".join(absolute) + ("::*" if kind == "glob" else "")
                    )
    if public_audio_module:
        exports.append("pub mod audio")
    if root_macro_expansion:
        exports.append("root macro/include expansion")
    return tuple(exports)


class Frameworks03AudioContractOwnerBoundaryTests(unittest.TestCase):
    def test_sound_root_does_not_reexport_neutral_audio_layout_types(self) -> None:
        source = (SOUND_ROOT / "mod.rs").read_text(encoding="utf-8")

        self.assertEqual((), neutral_audio_namespace_exports(source))
        self.assertNotIn("AudioChannelLayout", source)
        self.assertNotIn("AudioSpeakerChannel", source)
        self.assertFalse((SOUND_ROOT / "channel_layout.rs").exists())

    def test_sound_root_guard_rejects_neutral_audio_namespace_exports(self) -> None:
        forbidden_exports = (
            "pub(crate) use crate::core::framework::audio::*;",
            "#[doc(hidden)]\npub(crate) use crate::core::framework::audio::*;",
            "pub use super::audio;",
            "pub(crate) use crate::core::framework::{audio, project};",
            "pub(crate) use crate::core::framework::*;",
            "pub(crate) use crate::core::{framework::*};",
            "pub(crate) use super::{*};",
            (
                "use crate::core::framework::audio as neutral;\n"
                "pub(crate) use neutral::*;"
            ),
            (
                "use crate::core::framework::audio::{self as neutral};\n"
                "pub(crate) use neutral::*;"
            ),
            (
                "use crate::core::framework::audio as r#type;\n"
                "pub(crate) use r#type::*;"
            ),
            (
                "use crate::core::framework::audio::{self as r#type};\n"
                "pub(crate) use r#type::*;"
            ),
            (
                "use crate::core::framework::audio as neutral;\n"
                "mod helper { use crate::core::framework::sound as neutral; }\n"
                "pub(crate) use neutral::*;"
            ),
            "pub(crate) use crate::core::framework as legacy_framework;",
            (
                "pub(crate) use crate::core::framework::{self as legacy_framework};"
            ),
            "pub(crate) use crate::core::*;",
            "pub(crate) use zircon_runtime::core::framework::audio::*;",
            "pub(crate) use ::zircon_runtime::core::framework::audio::*;",
            (
                "extern crate self as engine;\n"
                "pub(crate) use engine::core::framework::audio::*;"
            ),
            (
                "extern crate self as r#type;\n"
                "pub(crate) use r#type::core::framework::audio::*;"
            ),
            (
                "#[cfg(windows)] use crate::core::framework::audio as neutral;\n"
                "#[cfg(not(windows))] use crate::core::framework::sound as neutral;\n"
                "#[cfg(windows)] pub(crate) use neutral::*;"
            ),
            (
                "#[cfg(all(windows, feature = \"audio\"))] "
                "use crate::core::framework::audio as neutral;\n"
                "#[cfg(not(windows))] "
                "use crate::core::framework::sound as neutral;\n"
                "#[cfg(all(windows, feature = \"audio\"))] "
                "pub(crate) use neutral::*;"
            ),
            (
                "#[cfg(target_os = \"windows\")] "
                "use crate::core::framework::audio as neutral;\n"
                "#[cfg(target_os = \"linux\")] "
                "use crate::core::framework::sound as neutral;\n"
                "#[cfg(target_os = \"windows\")] pub(crate) use neutral::*;"
            ),
            (
                "#[cfg(all(windows, target_os = \"windows\"))] "
                "pub(crate) use crate::core::framework::audio::*;"
            ),
            (
                "#[cfg(all(windows, target_family = \"windows\"))] "
                "pub(crate) use crate::core::framework::audio::*;"
            ),
            (
                "macro_rules! export_audio {\n"
                "    () => { pub(crate) use crate::core::framework::audio::*; }\n"
                "}\n"
                "export_audio!();"
            ),
            "crate::export_audio!();",
            "self::export_audio!();",
            "r#type!();",
            "crate::r#type!();",
            'include!("sound_root_exports.rs");',
            "pub(crate)use crate::core::framework::audio::*;",
            "pub mod audio;",
            "pub(crate)mod audio;",
        )

        for source in forbidden_exports:
            with self.subTest(source=source):
                self.assertTrue(neutral_audio_namespace_exports(source))

    def test_sound_root_guard_ignores_non_code_and_private_audio_imports(self) -> None:
        allowed_sources = (
            "/*\npub(crate) use crate::core::framework::audio::*;\n*/",
            "// pub(crate) use crate::core::framework::audio::*;",
            'const EXAMPLE: &str = "pub(crate) use crate::core::framework::audio::*;";',
            'const EXAMPLE: &str = r#"\npub(crate) use crate::core::framework::audio::*;\n"#;',
            "use crate::core::framework::audio::AudioChannelLayout;",
            "pub use super::SoundManager;",
            "pub use crate::core::framework::{sound::*, project::Project};",
            (
                "use crate::core::framework::sound as neutral;\n"
                "mod helper { use crate::core::framework::audio as neutral; }\n"
                "pub(crate) use neutral::*;"
            ),
            "pub use crate::telemetry::audio::AudioMetrics;",
            "mod helper { pub use super::*; }",
            (
                "#[cfg(windows)] use crate::core::framework::sound as neutral;\n"
                "#[cfg(not(windows))] use crate::core::framework::audio as neutral;\n"
                "#[cfg(windows)] pub(crate) use neutral::*;"
            ),
            (
                "#[cfg(all(windows, feature = \"audio\"))] "
                "use crate::core::framework::sound as neutral;\n"
                "#[cfg(not(windows))] "
                "use crate::core::framework::audio as neutral;\n"
                "#[cfg(all(windows, feature = \"audio\"))] "
                "pub(crate) use neutral::*;"
            ),
            (
                "#[cfg(target_os = \"windows\")] "
                "use crate::core::framework::sound as neutral;\n"
                "#[cfg(target_os = \"linux\")] "
                "use crate::core::framework::audio as neutral;\n"
                "#[cfg(target_os = \"windows\")] pub(crate) use neutral::*;"
            ),
            (
                "#[cfg(windows)] use crate::core::framework::audio as neutral;\n"
                "#[cfg(target_os = \"linux\")] "
                "use crate::core::framework::sound as neutral;\n"
                "#[cfg(target_os = \"linux\")] pub(crate) use neutral::*;"
            ),
            (
                "#![cfg(any())]\n"
                "pub use super::SoundManager;\n"
                "pub(crate) use crate::core::framework::audio::*;"
            ),
            (
                "#[cfg(all(windows, not(windows)))] "
                "pub(crate) use crate::core::framework::audio::*;"
            ),
            (
                "#[cfg(all(windows, not(target_os = \"windows\")))] "
                "pub(crate) use crate::core::framework::audio::*;"
            ),
            (
                "#[cfg(all(windows, not(target_family = \"windows\")))] "
                "pub(crate) use crate::core::framework::audio::*;"
            ),
            (
                "#[cfg(all(windows, not(windows)))] "
                "export_audio!();"
            ),
            'fn helper() { include!("function_body.rs"); }',
            'const EXAMPLE: &str = include_str!("audio.md");',
        )

        for source in allowed_sources:
            with self.subTest(source=source):
                self.assertEqual((), neutral_audio_namespace_exports(source))

    def test_sound_leaf_owners_import_audio_channel_layout_directly(self) -> None:
        indirect_group_import = re.compile(
            r"use\s+super::\{[^}]*\bAudioChannelLayout\b[^}]*\};",
            re.DOTALL,
        )
        for file_name in SOUND_LAYOUT_CONSUMERS:
            source = (SOUND_ROOT / file_name).read_text(encoding="utf-8")
            self.assertIn(
                "use crate::core::framework::audio::AudioChannelLayout;",
                source,
                file_name,
            )
            self.assertNotIn("use super::AudioChannelLayout;", source, file_name)
            self.assertIsNone(indirect_group_import.search(source), file_name)

    def test_sound_contract_tests_import_neutral_audio_types_directly(self) -> None:
        source = (SOUND_ROOT / "tests.rs").read_text(encoding="utf-8")

        self.assertIn(
            "use crate::core::framework::audio::{AudioChannelLayout, AudioSpeakerChannel};",
            source,
        )
        self.assertNotIn("use super::AudioChannelLayout;", source)
        self.assertNotIn("use super::AudioSpeakerChannel;", source)

    def test_neutral_audio_module_remains_the_only_declaration_owner(self) -> None:
        source = (AUDIO_ROOT / "channel_layout.rs").read_text(encoding="utf-8")

        self.assertIn("pub struct AudioChannelLayout", source)
        self.assertIn("pub enum AudioSpeakerChannel", source)
        for sound_source in SOUND_ROOT.rglob("*.rs"):
            content = sound_source.read_text(encoding="utf-8")
            self.assertNotIn("pub struct AudioChannelLayout", content, sound_source)
            self.assertNotIn("pub enum AudioSpeakerChannel", content, sound_source)


if __name__ == "__main__":
    unittest.main()
