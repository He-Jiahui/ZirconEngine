from pathlib import Path
import re
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
RUNTIME_TEST_ROOT = REPO_ROOT / "zircon_runtime/src/tests"
RUNTIME_ABSORPTION_ROOT = RUNTIME_TEST_ROOT / "contracts"
FIXTURE_OWNER = RUNTIME_ABSORPTION_ROOT / "current_source_fixture.rs"
RUNTIME_15_OUTPUT_ARCHIVE = (
    "docs/plans/_archive/zircon_runtime/runtime/15/"
    "2026-07-09-code-structure-and-module-conventions-output-records.md"
)
FIXTURE_SYMBOL = "runtime_architecture_implementation_output"
EXPECTED_CONSUMERS = {
    'contracts/core_scene/scene_ecs_owners/split_layout.rs',
    'contracts/core_spine_root_generated/split_layout.rs',
    'contracts/dynamic_api_session/shared/split_layout.rs',
    'contracts/ecs_kernel_data/split_layout.rs',
    'contracts/generated_code_guard/split_layout.rs',
    'contracts/input_stack/inventory/split_layout.rs',
    'contracts/input_stack/split_layout.rs',
    'contracts/naming_boundary/split_layout.rs',
    'contracts/review/plugin_importer_dx/d10_bridge_call.rs',
    'contracts/root_entries/module_families/split_layout.rs',
    'contracts/script_binding/split_layout.rs',
    'contracts/tech_stack/split_layout.rs',
}
def rust_cooked_string(value: str, byte_escapes: bool = False) -> str | None:
    """Decode Rust escapes only; malformed literals cannot be compile inputs."""
    result = []
    escapes = {"n": "\n", "r": "\r", "t": "\t", "0": "\0", "\\": "\\", "\"": "\"", "'": "'"}
    i = 0
    while i < len(value):
        if value[i] != "\\":
            result.append(value[i])
            i += 1
            continue
        i += 1
        if i == len(value):
            return None
        escape = value[i]
        if escape in escapes:
            result.append(escapes[escape])
            i += 1
        elif escape == "\n" or value.startswith("\r\n", i):
            i += 2 if escape == "\r" else 1
            while i < len(value) and value[i] in " \t\r\n":
                i += 1
        elif escape == "x":
            digits = value[i + 1:i + 3]
            if not re.fullmatch(r"[0-9a-fA-F]{2}", digits):
                return None
            number = int(digits, 16)
            if number > 0x7f and not byte_escapes:
                return None
            result.append(chr(number))
            i += 3
        elif escape == "u" and not byte_escapes:
            match = re.match(r"u\{([0-9a-fA-F][0-9a-fA-F_]*)\}", value[i:])
            if match is None:
                return None
            digits = match.group(1).replace("_", "")
            if len(digits) > 6:
                return None
            number = int(digits, 16)
            if number > 0x10ffff or 0xd800 <= number <= 0xdfff:
                return None
            result.append(chr(number))
            i += len(match.group(0))
        else:
            return None
    return "".join(result)


def rust_code_and_strings(source: str) -> tuple[str, list[tuple[int, int, str]]]:
    code = list(source)
    strings = []
    raw_literal = re.compile(r'(?:br|r)(#*)"')
    character_literal = re.compile(r"'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\r\n])'")
    i = 0
    while i < len(source):
        start = i
        value = None
        if source.startswith("//", i):
            end = source.find("\n", i)
            i = len(source) if end < 0 else end
        elif source.startswith("/*", i):
            i += 2
            depth = 1
            while i < len(source) and depth:
                if source.startswith("/*", i):
                    depth += 1
                    i += 2
                elif source.startswith("*/", i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
        else:
            raw = raw_literal.match(source, i)
            char = character_literal.match(source, i)
            if raw:
                content = raw.end()
                delimiter = '"' + raw.group(1)
                end = source.find(delimiter, content)
                if end < 0:
                    i = len(source)
                else:
                    value = source[content:end]
                    i = end + len(delimiter)
            elif char:
                i = char.end()
            elif source[i] == '"':
                i += 1
                content = i
                while i < len(source):
                    if source[i] == "\\":
                        i += 2
                    elif source[i] == '"':
                        value = rust_cooked_string(source[content:i], source[start - 1:start] in {"b", "c"})
                        i += 1
                        break
                    else:
                        i += 1
            else:
                i += 1
                continue
        if value is not None:
            strings.append((start, i, value))
        for position in range(start, i):
            if code[position] != "\n":
                code[position] = " "
    return "".join(code), strings


def historical_compile_dependencies(source: str) -> list[str]:
    code, strings = rust_code_and_strings(source)
    paths = []
    closing = {"(": ")", "[": "]", "{": "}"}
    for macro in re.finditer(r"\binclude(?:_str|_bytes)?\s*!\s*([({\[])", code):
        stack = [closing[macro.group(1)]]
        end = macro.end()
        while end < len(code) and stack:
            token = code[end]
            if token in closing:
                stack.append(closing[token])
            elif token == stack[-1]:
                stack.pop()
            end += 1
        # Joining literal pieces also detects include_str!(concat!(...)).
        literal = "".join(value for start, stop, value in strings if macro.end() <= start and stop <= end)
        normalized = re.sub(r"/+", "/", literal.replace("\\", "/"))
        if "docs/plans/" in normalized and (
            "/_archive/" in normalized or normalized.endswith("output-records.md")
        ):
            paths.append(literal)
    return paths


class RuntimeAbsorptionCurrentSourceFixtureTests(unittest.TestCase):
    def test_runtime_absorption_uses_one_tracked_current_source_fixture_owner(self) -> None:
        owner_source = FIXTURE_OWNER.read_text(encoding="utf-8")
        self.assertIn(RUNTIME_15_OUTPUT_ARCHIVE, owner_source)
        self.assertIn(f"fn {FIXTURE_SYMBOL}()", owner_source)
        self.assertIn("fixed_history_archive_source as history", owner_source)
        self.assertEqual([], historical_compile_dependencies(owner_source))
        module_source = (RUNTIME_ABSORPTION_ROOT / "mod.rs").read_text(encoding="utf-8")
        self.assertIn("mod current_source_fixture;", module_source)
        evidence_guard = (
            RUNTIME_ABSORPTION_ROOT / "structure/evidence_ownership.rs"
        ).read_text(encoding="utf-8")
        self.assertIn('runtime_src_path("tests")', evidence_guard)
        self.assertNotIn('runtime_src_path("tests/contracts")', evidence_guard)
        consumers = set()
        for path in RUNTIME_TEST_ROOT.rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            self.assertNotIn(".codex/sessions", source, path.as_posix())
            self.assertEqual([], historical_compile_dependencies(source), path.as_posix())
            self.assertNotIn(
                "20260612-0847-runtime-architecture-implementation.md",
                source, path.as_posix(),
            )
            if path != FIXTURE_OWNER and f"::{FIXTURE_SYMBOL}()" in source:
                consumers.add(path.relative_to(RUNTIME_TEST_ROOT).as_posix())
        self.assertEqual(EXPECTED_CONSUMERS, consumers)
        self.assertEqual(12, len(consumers))
        self.assertFalse(
            (RUNTIME_ABSORPTION_ROOT / "plan_status").exists(),
            "the retired status tree must not be restored for fixture consumers",
        )
        for relative_path in EXPECTED_CONSUMERS:
            source = (RUNTIME_TEST_ROOT / relative_path).read_text(encoding="utf-8")
            self.assertEqual([], historical_compile_dependencies(source), relative_path)
            self.assertNotIn("RUNTIME_ARCHITECTURE_IMPLEMENTATION_OUTPUT", source)
        for relative_path in ["script_absorption/split_layout.rs", "script_host_ledger/split_layout.rs"]:
            source = (RUNTIME_ABSORPTION_ROOT / relative_path).read_text(encoding="utf-8")
            self.assertIn(RUNTIME_15_OUTPUT_ARCHIVE, source)
            self.assertIn("fixed_history_archive_source", source)
            self.assertEqual([], historical_compile_dependencies(source))

    def test_historical_compile_dependency_detector_preserves_readback_literals(self) -> None:
        path = RUNTIME_15_OUTPUT_ARCHIVE
        self.assertEqual([path], historical_compile_dependencies(f'include_str!("{path}")'))
        self.assertEqual([path], historical_compile_dependencies(f'include_bytes!(r#"{path}"#)'))
        self.assertEqual([], historical_compile_dependencies(f'history("{path}")'))
        self.assertEqual([], historical_compile_dependencies('include_str!("source.rs")'))
        self.assertEqual([path], historical_compile_dependencies(f'include_str! /* gap */ (r{"#" * 24}"{path}"{"#" * 24})'))
        self.assertEqual([path], historical_compile_dependencies(f'include_str!(concat!("docs/plans/", "{path[len("docs/plans/"):]}"))'))
        self.assertEqual([path], historical_compile_dependencies(f'include_bytes!(/* outer /* nested */ end */ "{path}")'))
        self.assertEqual([path], historical_compile_dependencies(f'include_str /* before bang */ ! ("{path}")'))

    def test_historical_compile_dependency_detector_ignores_non_code(self) -> None:
        path = RUNTIME_15_OUTPUT_ARCHIVE
        macro = f'include_str!("{path}")'
        for source in [
            "// " + macro,
            "/* outer /* nested */ " + macro + " */",
            f'let text = r{"#" * 24}"{macro}"{"#" * 24};',
            f'let text = "{macro.replace(chr(34), chr(92) + chr(34))}";',
            "let quote = '\\\"'; // " + macro,
            f'let value: &\'static str = history("{path}");',
        ]:
            self.assertEqual([], historical_compile_dependencies(source), source)


    def test_historical_compile_inputs_cover_rust_delimiters_and_cooked_escapes(self) -> None:
        path = RUNTIME_15_OUTPUT_ARCHIVE
        for name in ("include_str", "include_bytes", "include"):
            for opening, closing in (("(", ")"), ("[", "]"), ("{", "}")):
                with self.subTest(macro=name, delimiter=opening):
                    self.assertEqual([path], historical_compile_dependencies(f'{name}! {opening}"{path}"{closing}'))
                    self.assertEqual([path], historical_compile_dependencies(f'{name}! {opening}concat!("docs/plans/", "{path[len("docs/plans/"):]}"){closing}'))
            for encoded in (
                path.replace("plans", r"\x70lans", 1),
                path.replace("plans", r"\u{70}lans", 1),
                path.replace("plans", r"\u{0_0_7_0}lans", 1),
                path.replace("docs/", "docs/\\\n  \t", 1),
            ):
                with self.subTest(macro=name, encoded=encoded):
                    self.assertEqual([path], historical_compile_dependencies(f'{name}! {{"{encoded}"}}'))
        self.assertEqual("\n\r\t\0\\\"'", rust_cooked_string(r"\n\r\t\0\\\"\'"))
        self.assertEqual("p", rust_cooked_string(r"\x70"))
        self.assertEqual("p", rust_cooked_string(r"\u{70}"))
        self.assertEqual("\xff", rust_cooked_string(r"\xff", byte_escapes=True))
        for invalid in (r"\q", r"\x80", r"\u{d800}", r"\u{110000}", r"\u{0000000}", r"\u{_70}"):
            self.assertIsNone(rust_cooked_string(invalid), invalid)

    def test_escape_and_delimiter_examples_inside_non_code_do_not_fake_macros(self) -> None:
        path = RUNTIME_15_OUTPUT_ARCHIVE
        encoded = path.replace("plans", r"\u{70}lans", 1)
        self.assertEqual([], historical_compile_dependencies(f'include_str!(r#"{encoded}"#)'))
        for opening, closing in (("(", ")"), ("[", "]"), ("{", "}")):
            macro = f'include_bytes!{opening}"{encoded}"{closing}'
            for source in (
                "// " + macro,
                "/* outer /* nested */ " + macro + " */",
                f'let text = r###"{macro}"###;',
                'let text = "' + macro.replace("\\", "\\\\").replace('"', '\\"') + '";',
            ):
                self.assertEqual([], historical_compile_dependencies(source), source)
        self.assertEqual([], historical_compile_dependencies('let byte = b"\\xff"; include_str!{"current.rs"}'))


if __name__ == "__main__":
    unittest.main()
