import re
import subprocess
import unittest
from pathlib import Path

from tools.runtime_domain_dependency_audit import _rust_code_view, _rust_use_paths


REPO_ROOT = Path(__file__).resolve().parents[2]
CONTRACT_CRATE_ROOT = REPO_ROOT / "zircon_runtime/crates/zr_contracts"
CONTRACT_ROOT = CONTRACT_CRATE_ROOT / "src/random"
FACADE_ROOT = REPO_ROOT / "zircon_runtime/src/core/framework/random"
KERNEL_ROOT = REPO_ROOT / "zircon_runtime/src/core/runtime/random"
PRODUCT_SOURCE_ROOTS = (
    "zircon_runtime/",
    "zircon_app/",
    "zircon_editor/",
    "zircon_plugins/",
)
ALLOWED_RANDOM_SERVICE_ACCESSOR_PATHS = {
    "zircon_runtime/src/core/runtime/handle/random.rs",
    "zircon_runtime/src/core/runtime/runtime.rs",
    "zircon_runtime/src/core/runtime/state/core_runtime_state.rs",
}
REQUIRED_CONTRACT_FILES = {
    "algorithm.rs",
    "assembly.rs",
    "checkpoint_error.rs",
    "key.rs",
    "mod.rs",
    "service_checkpoint.rs",
    "service_state.rs",
    "state.rs",
    "stream_checkpoint.rs",
}
REQUIRED_KERNEL_FILES = {
    "authority.rs",
    "derivation.rs",
    "error.rs",
    "lease.rs",
    "limits.rs",
    "mod.rs",
    "registry.rs",
    "service.rs",
    "stream.rs",
    "stream_error.rs",
}
IMPLEMENTATION_NAMES = {"RandomService", "RandomStream", "RandomStreamError"}
OLD_IMPLEMENTATION_PREFIXES = tuple(
    prefix + (name,)
    for prefix in (
        ("crate", "core", "framework", "random"),
        ("zircon_runtime", "core", "framework", "random"),
    )
    for name in IMPLEMENTATION_NAMES
)
OLD_IMPLEMENTATION_ROOTS = {prefix[:-1] for prefix in OLD_IMPLEMENTATION_PREFIXES}
OLD_QUALIFIED_PATH = re.compile(
    r"\b(?:crate|zircon_runtime)\s*::\s*core\s*::\s*framework\s*::\s*random\s*"
    r"::\s*(?:RandomService|RandomStream|RandomStreamError)\b"
)
UNQUALIFIED_IMPLEMENTATION_NAME = re.compile(
    rf"(?<![:\w])(?:{'|'.join(sorted(IMPLEMENTATION_NAMES))})\b"
)
STREAM_SELF_CONSTRUCTOR = re.compile(
    r"\bfn\s+[A-Za-z_]\w*\s*\((?P<arguments>.*?)\)\s*->\s*"
    r"(?:Result\s*<\s*)?Self\b",
    re.DOTALL,
)
FORBIDDEN_CONTRACT_IMPLEMENTATION = (
    ("blake3::Hasher", re.compile(r"\bblake3\s*::\s*Hasher\b")),
    ("PCG32_MULTIPLIER", re.compile(r"\bPCG32_MULTIPLIER\b")),
    ("struct RandomService", re.compile(r"\bstruct\s+RandomService\b")),
    ("struct RandomStream", re.compile(r"\bstruct\s+RandomStream\b")),
    ("fn append_stream_key", re.compile(r"\bfn\s+append_stream_key\b")),
    ("fn advance_pcg_state", re.compile(r"\bfn\s+advance_pcg_state\b")),
    ("fn pcg32_xsh_rr", re.compile(r"\bfn\s+pcg32_xsh_rr\b")),
)


def old_random_implementation_lines(source: str) -> list[int]:
    code_view = _rust_code_view(source)
    use_paths = _rust_use_paths(code_view)
    aliases: dict[str, tuple[str, ...]] = {}

    def resolve_alias(path: tuple[str, ...]) -> tuple[str, ...]:
        resolved = path
        seen: set[str] = set()
        while resolved and resolved[0] in aliases and resolved[0] not in seen:
            seen.add(resolved[0])
            resolved = aliases[resolved[0]] + resolved[1:]
        return resolved

    for _ in range(len(use_paths) + 1):
        changed = False
        for path, alias, _line in use_paths:
            resolved = resolve_alias(path)
            if not any(
                prefix[: len(resolved)] == resolved
                for prefix in OLD_IMPLEMENTATION_PREFIXES
            ):
                continue
            effective_alias = alias or resolved[-1]
            if aliases.get(effective_alias) != resolved:
                aliases[effective_alias] = resolved
                changed = True
        if not changed:
            break

    lines = set()
    imports_old_owner_glob = False
    for path, _alias, line in use_paths:
        resolved = resolve_alias(path)
        if resolved in OLD_IMPLEMENTATION_PREFIXES:
            lines.add(line)
        if resolved and resolved[-1] == "*" and resolved[:-1] in OLD_IMPLEMENTATION_ROOTS:
            imports_old_owner_glob = True

    if imports_old_owner_glob:
        for reference in UNQUALIFIED_IMPLEMENTATION_NAME.finditer(code_view):
            lines.add(code_view.count("\n", 0, reference.start()) + 1)

    for reference in OLD_QUALIFIED_PATH.finditer(code_view):
        lines.add(code_view.count("\n", 0, reference.start()) + 1)

    for alias, target in aliases.items():
        for prefix in OLD_IMPLEMENTATION_PREFIXES:
            if prefix[: len(target)] != target:
                continue
            suffix = prefix[len(target) :]
            if not suffix:
                continue
            alias_path = re.compile(
                rf"\b{re.escape(alias)}\s*"
                + "".join(rf"::\s*{re.escape(segment)}\s*" for segment in suffix)
            )
            for reference in alias_path.finditer(code_view):
                lines.add(code_view.count("\n", 0, reference.start()) + 1)
    return sorted(lines)


def raw_u64_stream_constructor_lines(source: str) -> list[int]:
    code_view = _rust_code_view(source)
    return [
        code_view.count("\n", 0, constructor.start()) + 1
        for constructor in STREAM_SELF_CONSTRUCTOR.finditer(code_view)
        if len(re.findall(r":\s*u64\b", constructor.group("arguments"))) > 1
    ]


def implicit_copy_trait_lines(source: str, struct_name: str) -> list[int]:
    code_view = _rust_code_view(source)
    lines = set()
    declaration = re.search(
        rf"(?P<attributes>(?:#\s*\[.*?\]\s*)*)"
        rf"pub\s+struct\s+{re.escape(struct_name)}\b",
        code_view,
        re.DOTALL,
    )
    if declaration is not None:
        attributes = declaration.group("attributes")
        attribute_offset = declaration.start("attributes")
        for trait in re.finditer(r"\b(?:Clone|Copy)\b", attributes):
            lines.add(code_view.count("\n", 0, attribute_offset + trait.start()) + 1)

    copy_trait_aliases = {"Clone", "Copy"}
    for path, alias, _line in _rust_use_paths(code_view):
        normalized = tuple(segment for segment in path if segment not in {"crate", "self"})
        if normalized in {
            ("core", "clone", "Clone"),
            ("std", "clone", "Clone"),
            ("core", "marker", "Copy"),
            ("std", "marker", "Copy"),
        }:
            copy_trait_aliases.add(alias or normalized[-1])

    explicit_impl = re.compile(
        rf"\bimpl(?:\s*<[^>{{}}]*>)?\s+"
        rf"(?P<trait>(?:[A-Za-z_]\w*\s*::\s*)*[A-Za-z_]\w*)\s+for\s+"
        rf"(?:[A-Za-z_]\w*\s*::\s*)*{re.escape(struct_name)}\b"
    )
    for implementation in explicit_impl.finditer(code_view):
        trait_name = re.sub(r"\s+", "", implementation.group("trait")).rsplit("::", 1)[-1]
        if trait_name in copy_trait_aliases:
            lines.add(code_view.count("\n", 0, implementation.start()) + 1)

    return sorted(lines)


RUST_TYPE_PATH = re.compile(
    r"(?<![A-Za-z0-9_])(?:[A-Za-z_]\w*\s*::\s*)*[A-Za-z_]\w*"
)


def random_service_type_aliases(code_view: str) -> set[str]:
    aliases = {"RandomService"}
    for path, alias, _line in _rust_use_paths(code_view):
        if path and path[-1] == "RandomService":
            aliases.add(alias or path[-1])

    type_aliases = list(
        re.finditer(
            r"\btype\s+(?P<alias>[A-Za-z_]\w*)"
            r"(?:\s*<[^;={}>]*>)?\s*=\s*(?P<target>[^;]+);",
            code_view,
        )
    )
    for _ in range(len(type_aliases) + 1):
        changed = False
        for type_alias in type_aliases:
            if random_service_type_paths(type_alias.group("target"), aliases):
                alias = type_alias.group("alias")
                if alias not in aliases:
                    aliases.add(alias)
                    changed = True
        if not changed:
            break
    return aliases


def random_service_authority_alias_lines(source: str) -> list[int]:
    code_view = _rust_code_view(source)
    lines = set()
    aliases = {"RandomService"}
    for path, alias, line in _rust_use_paths(code_view):
        if path and path[-1] == "RandomService" and alias is not None:
            aliases.add(alias)
            lines.add(line)

    type_aliases = list(
        re.finditer(
            r"\btype\s+(?P<alias>[A-Za-z_]\w*)"
            r"(?:\s*<[^;={}>]*>)?\s*=\s*(?P<target>[^;]+);",
            code_view,
        )
    )
    for _ in range(len(type_aliases) + 1):
        changed = False
        for type_alias in type_aliases:
            if not random_service_type_paths(type_alias.group("target"), aliases):
                continue
            lines.add(code_view.count("\n", 0, type_alias.start()) + 1)
            alias = type_alias.group("alias")
            if alias not in aliases:
                aliases.add(alias)
                changed = True
        if not changed:
            break
    return sorted(lines)


def random_service_type_paths(type_source: str, aliases: set[str]) -> list[str]:
    paths = []
    for type_path in RUST_TYPE_PATH.finditer(type_source):
        normalized = re.sub(r"\s+", "", type_path.group(0))
        if normalized.rsplit("::", 1)[-1] in aliases:
            paths.append(normalized)
    return paths


def is_shared_random_service_reference(type_source: str, aliases: set[str]) -> bool:
    shared_reference = re.fullmatch(
        r"&\s*(?:'[A-Za-z_]\w*\s+)?"
        r"(?P<referent>(?:[A-Za-z_]\w*\s*::\s*)*[A-Za-z_]\w*)",
        type_source.strip(),
    )
    if shared_reference is None:
        return False
    referent = shared_reference.group("referent")
    normalized = re.sub(r"\s+", "", referent)
    paths = random_service_type_paths(referent, aliases)
    return len(paths) == 1 and paths[0] == normalized


def random_service_accessor_scan(source: str) -> list[tuple[int, bool]]:
    code_view = _rust_code_view(source)
    aliases = random_service_type_aliases(code_view)
    functions = re.compile(
        r"\bpub(?:\s*\([^)]*\))?\s+"
        r"(?:const\s+)?(?:async\s+)?fn\s+(?P<name>[A-Za-z_]\w*)\s*"
        r"(?:<[^>{}]*>\s*)?"
        r"\((?P<arguments>.*?)\)\s*->\s*(?P<return>[^;{]+)",
        re.DOTALL,
    )
    accessors = []
    for function in functions.finditer(code_view):
        return_type = function.group("return")
        if not random_service_type_paths(return_type, aliases):
            continue
        line = code_view.count("\n", 0, function.start()) + 1
        arguments = re.sub(r"\s+", "", function.group("arguments"))
        is_shared_owner_accessor = (
            function.group("name") == "random_service"
            and arguments == "&self"
            and is_shared_random_service_reference(return_type, aliases)
        )
        accessors.append((line, is_shared_owner_accessor))
    return accessors


def random_service_accessor_escape_lines(source: str) -> list[int]:
    return [line for line, is_shared in random_service_accessor_scan(source) if not is_shared]


def random_service_authority_surface_violations(
    sources: dict[str, str],
) -> list[tuple[str, int, str]]:
    violations = []
    allowed_accessors: dict[str, list[int]] = {
        path: [] for path in ALLOWED_RANDOM_SERVICE_ACCESSOR_PATHS
    }
    for raw_path, source in sources.items():
        path = raw_path.replace("\\", "/")
        for line in random_service_authority_alias_lines(source):
            violations.append((path, line, "forbidden_authority_alias"))
        for line, is_shared in random_service_accessor_scan(source):
            if not is_shared:
                violations.append((path, line, "escaping_accessor"))
            elif path not in ALLOWED_RANDOM_SERVICE_ACCESSOR_PATHS:
                violations.append((path, line, "unexpected_accessor"))
            else:
                allowed_accessors[path].append(line)
        for line in implicit_copy_trait_lines(source, "RandomService"):
            violations.append((path, line, "copyable_authority"))

    for path, lines in allowed_accessors.items():
        if not lines:
            violations.append((path, 0, "missing_shared_accessor"))
        elif len(lines) > 1:
            violations.extend(
                (path, line, "duplicate_shared_accessor") for line in lines[1:]
            )
    return sorted(violations)


def split_top_level_rust_fields(body: str) -> list[tuple[int, str]]:
    fields = []
    field_start = 0
    delimiters = {"(": ")", "[": "]", "{": "}", "<": ">"}
    depths = {closing: 0 for closing in delimiters.values()}
    for offset, character in enumerate(body):
        if character in delimiters:
            depths[delimiters[character]] += 1
        elif character in depths:
            depths[character] = max(0, depths[character] - 1)
        elif character == "," and not any(depths.values()):
            fields.append((field_start, body[field_start:offset]))
            field_start = offset + 1
    fields.append((field_start, body[field_start:]))
    return fields


def core_runtime_inner_fields(source: str) -> tuple[str, list[tuple[str | None, str, int]]]:
    code_view = _rust_code_view(source)
    declaration = re.search(
        r"\bstruct\s+CoreRuntimeInner\b[^\{]*\{",
        code_view,
    )
    if declaration is None:
        return code_view, []

    body_start = declaration.end()
    depth = 1
    body_end = body_start
    while body_end < len(code_view) and depth:
        if code_view[body_end] == "{":
            depth += 1
        elif code_view[body_end] == "}":
            depth -= 1
        body_end += 1
    if depth:
        return code_view, []

    fields = []
    body = code_view[body_start : body_end - 1]
    field_declaration = re.compile(
        r"\s*(?:#\s*\[.*?\]\s*)*"
        r"(?P<visibility>pub(?:\s*\([^)]*\))?\s+)?"
        r"(?P<name>[A-Za-z_]\w*)\s*:\s*(?P<type>.*?)\s*",
        re.DOTALL,
    )
    for segment_start, segment in split_top_level_rust_fields(body):
        field = field_declaration.fullmatch(segment)
        if field is None:
            continue
        fields.append(
            (
                field.group("visibility"),
                field.group("type"),
                body_start + segment_start + field.start("name"),
            )
        )
    return code_view, fields


def random_service_authority_field_lines(source: str) -> list[int]:
    code_view, fields = core_runtime_inner_fields(source)
    aliases = random_service_type_aliases(code_view)
    return [
        code_view.count("\n", 0, name_offset) + 1
        for _visibility, field_type, name_offset in fields
        if random_service_type_paths(field_type, aliases)
    ]


def exposed_random_service_field_lines(source: str) -> list[int]:
    code_view, fields = core_runtime_inner_fields(source)
    aliases = random_service_type_aliases(code_view)
    return [
        code_view.count("\n", 0, name_offset) + 1
        for visibility, field_type, name_offset in fields
        if visibility is not None and random_service_type_paths(field_type, aliases)
    ]


def product_rust_candidate_paths() -> list[Path]:
    result = subprocess.run(
        [
            "git",
            "grep",
            "-l",
            "--untracked",
            "-e",
            "RandomService",
            "-e",
            "RandomStream",
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
    return [
        REPO_ROOT / relative_path
        for relative_path in result.stdout.splitlines()
        if relative_path.startswith(PRODUCT_SOURCE_ROOTS)
    ]


def product_rust_candidate_sources() -> dict[str, str]:
    sources = {}
    for path in product_rust_candidate_paths():
        try:
            source = path.read_text(encoding="utf-8")
        except FileNotFoundError:
            continue
        sources[path.relative_to(REPO_ROOT).as_posix()] = source
    return sources


class Frameworks01RandomContractKernelBoundaryTests(unittest.TestCase):
    def test_random_contract_and_kernel_have_disjoint_physical_owners(self) -> None:
        contract_manifest = (CONTRACT_CRATE_ROOT / "Cargo.toml").read_text(
            encoding="utf-8"
        )
        contract_lib = (CONTRACT_CRATE_ROOT / "src/lib.rs").read_text(
            encoding="utf-8"
        )
        self.assertTrue(CONTRACT_ROOT.is_dir())
        self.assertEqual(
            REQUIRED_CONTRACT_FILES,
            {path.name for path in CONTRACT_ROOT.glob("*.rs")},
        )
        self.assertIn('name = "zr_contracts"', contract_manifest)
        self.assertIn("serde.workspace = true", contract_manifest)
        self.assertIn("thiserror.workspace = true", contract_manifest)
        self.assertNotIn("blake3", contract_manifest)
        self.assertRegex(contract_lib, r"(?m)^pub\s+mod\s+random\s*;")
        self.assertTrue(KERNEL_ROOT.is_dir())
        self.assertEqual(
            REQUIRED_KERNEL_FILES,
            {path.name for path in KERNEL_ROOT.glob("*.rs")},
        )

        workspace_manifest = (REPO_ROOT / "Cargo.toml").read_text(encoding="utf-8")
        runtime_manifest = (
            REPO_ROOT / "zircon_runtime/Cargo.toml"
        ).read_text(encoding="utf-8")
        self.assertIn('"zircon_runtime/crates/zr_contracts"', workspace_manifest)
        self.assertRegex(
            workspace_manifest,
            r"(?m)^zr_contracts\s*=\s*\{[^\n]*"
            r'path\s*=\s*"zircon_runtime/crates/zr_contracts"',
        )
        self.assertRegex(runtime_manifest, r"(?m)^zr_contracts\.workspace\s*=\s*true$")

        runtime_mod = (REPO_ROOT / "zircon_runtime/src/core/runtime/mod.rs").read_text(
            encoding="utf-8"
        )
        framework_mod = (
            REPO_ROOT / "zircon_runtime/src/core/framework/mod.rs"
        ).read_text(encoding="utf-8")
        self.assertRegex(runtime_mod, r"(?m)^\s*pub\s+mod\s+random\s*;")
        self.assertRegex(framework_mod, r"(?m)^\s*pub\s+mod\s+random\s*;")

        facade_files = {path.name for path in FACADE_ROOT.glob("*.rs")}
        self.assertEqual({"mod.rs"}, facade_files)
        facade = (FACADE_ROOT / "mod.rs").read_text(encoding="utf-8")
        self.assertIn("pub use zr_contracts::random::{", facade)
        self.assertNotIn("assembly", facade)
        for retired_module in ("algorithm", "key", "service_state", "state", "tests"):
            self.assertNotRegex(
                facade,
                rf"(?m)^\s*(?:pub\s+)?mod\s+{retired_module}\s*;",
            )

        contract_mod = (CONTRACT_ROOT / "mod.rs").read_text(encoding="utf-8")
        self.assertIn("#[doc(hidden)]", contract_mod)
        self.assertRegex(contract_mod, r"(?m)^pub\s+mod\s+assembly\s*;")

        direct_kernel_consumers = (
            KERNEL_ROOT / "service.rs",
            KERNEL_ROOT / "stream.rs",
            KERNEL_ROOT / "tests.rs",
            REPO_ROOT / "zircon_runtime/src/core/runtime/handle/random.rs",
            REPO_ROOT / "zircon_runtime/src/core/runtime/runtime.rs",
        )
        for path in direct_kernel_consumers:
            source = path.read_text(encoding="utf-8")
            self.assertNotIn("crate::core::framework::random", source)
            self.assertIn("zr_contracts::random", source)

    def test_contract_owner_contains_no_random_execution_implementation(self) -> None:
        violations = []
        for path in CONTRACT_ROOT.glob("*.rs"):
            if path.name == "tests.rs":
                continue
            code_view = _rust_code_view(path.read_text(encoding="utf-8"))
            for label, pattern in FORBIDDEN_CONTRACT_IMPLEMENTATION:
                if pattern.search(code_view):
                    violations.append(f"{path.name}: {label}")
        self.assertEqual([], violations)

    def test_random_identity_and_generation_fail_closed(self) -> None:
        algorithm = (CONTRACT_ROOT / "algorithm.rs").read_text(encoding="utf-8")
        service_state = (CONTRACT_ROOT / "service_state.rs").read_text(
            encoding="utf-8"
        )
        authority = (KERNEL_ROOT / "authority.rs").read_text(encoding="utf-8")
        service = (KERNEL_ROOT / "service.rs").read_text(encoding="utf-8")
        stream = (KERNEL_ROOT / "stream.rs").read_text(encoding="utf-8")

        self.assertIn("pub struct RandomSequenceId(u64);", algorithm)
        self.assertIn("pub const MAX_VALUE: u64 = u64::MAX >> 1;", algorithm)
        self.assertIn("value > Self::MAX_VALUE", algorithm)
        self.assertIn("checked_add(1)", authority)
        self.assertNotIn("saturating_add", authority)
        self.assertIn("pub const fn try_new(", service_state)
        self.assertIn("previous_generation.checked_add(1)", service_state)
        self.assertIn(
            "impl<'de> Deserialize<'de> for RandomSeedReceipt",
            service_state,
        )
        self.assertIn("Self::try_new(", service_state)
        self.assertNotIn(
            "impl RandomSeedReceipt {\n    pub const fn new(",
            service_state,
        )
        self.assertIn("sequence: RandomSequenceId", stream)
        self.assertNotIn("stream_selector: u64", stream)
        self.assertEqual([], raw_u64_stream_constructor_lines(stream))
        self.assertEqual(1, len(re.findall(r"\bpub\s+fn\s+reseed\s*\(", service)))
        self.assertRegex(
            service,
            r"(?s)\bpub\s+fn\s+reseed\s*\(.*?\)\s*->\s*"
            r"Result\s*<\s*RandomSeedReceipt\s*,\s*RandomServiceError\s*>",
        )

    def test_random_state_assembly_preserves_the_validated_increment_invariant(self) -> None:
        assembly = (CONTRACT_ROOT / "assembly.rs").read_text(encoding="utf-8")
        state = (CONTRACT_ROOT / "state.rs").read_text(encoding="utf-8")
        stream = (KERNEL_ROOT / "stream.rs").read_text(encoding="utf-8")

        self.assertIn("sequence: RandomSequenceId", assembly)
        self.assertNotRegex(assembly, r"\bincrement\s*:\s*u64\b")
        self.assertNotIn("debug_assert!", assembly)
        self.assertRegex(
            assembly,
            r"(?s)pub\s+const\s+fn\s+random_state_with_progress\s*\("
            r"\s*current\s*:\s*RandomState\s*,"
            r"\s*generator_state\s*:\s*u64\s*,"
            r"\s*draw_index\s*:\s*u64\s*,?\s*\)\s*->\s*RandomState",
        )
        self.assertNotRegex(
            state,
            r"\bpub(?:\s*\([^)]*\))?\s+(?:const\s+)?fn\s+with_progress\b",
        )
        self.assertIn(
            "random_state_with_progress(self.state, next_state, next_draw_index)",
            stream,
        )

    def test_random_stream_forbids_implicit_copy_and_borrows_state_accessors(self) -> None:
        stream = (KERNEL_ROOT / "stream.rs").read_text(encoding="utf-8")

        self.assertEqual([], implicit_copy_trait_lines(stream, "RandomStream"))
        self.assertIn("pub const fn snapshot(&self)", stream)
        self.assertIn("pub const fn draw_index(&self)", stream)
        self.assertIn("pub const fn sequence_id(&self)", stream)

    def test_random_service_is_single_authority_and_runtime_accessors_borrow_it(self) -> None:
        service = (KERNEL_ROOT / "service.rs").read_text(encoding="utf-8")
        handle = (
            REPO_ROOT / "zircon_runtime/src/core/runtime/handle/random.rs"
        ).read_text(encoding="utf-8")
        runtime = (REPO_ROOT / "zircon_runtime/src/core/runtime/runtime.rs").read_text(
            encoding="utf-8"
        )
        runtime_state = (
            REPO_ROOT / "zircon_runtime/src/core/runtime/state/core_runtime_state.rs"
        ).read_text(encoding="utf-8")

        self.assertEqual([], implicit_copy_trait_lines(service, "RandomService"))
        for accessor in (
            "algorithm",
            "master_seed",
            "master_seed_generation",
            "snapshot",
        ):
            self.assertRegex(
                service,
                rf"\bpub\s+(?:const\s+)?fn\s+{accessor}\s*\(\s*&self\s*\)",
            )
        self.assertRegex(
            service,
            r"\bpub\s+fn\s+acquire_stream\s*\(\s*&self\s*,\s*key\s*:\s*RandomStreamKey\s*,?\s*\)",
        )
        self.assertRegex(
            handle,
            r"\bpub\s+fn\s+random_service\s*\(\s*&self\s*\)\s*->\s*&RandomService",
        )
        self.assertRegex(
            runtime,
            r"\bpub\s+fn\s+random_service\s*\(\s*&self\s*\)\s*->\s*&RandomService",
        )
        self.assertEqual([], random_service_accessor_escape_lines(handle))
        self.assertEqual([], random_service_accessor_escape_lines(runtime))
        self.assertEqual([], random_service_accessor_escape_lines(runtime_state))
        self.assertEqual(1, len(random_service_authority_field_lines(runtime_state)))
        self.assertEqual([], exposed_random_service_field_lines(runtime_state))
        self.assertEqual(
            [],
            random_service_authority_surface_violations(
                product_rust_candidate_sources()
            ),
        )

    def test_random_service_authority_scanners_reject_escape_mutations(self) -> None:
        accessor_source = """
type Authority = RandomService;
impl CoreRuntime {
    pub fn random_service(&self) -> &crate::core::runtime::random::RandomService { todo!() }
    pub(crate) fn detached_random_service<T>(&self) -> Authority { todo!() }
    pub fn random_service_mut(&mut self) -> &mut RandomService { todo!() }
    pub fn random_service_state(&self) -> RandomServiceState { todo!() }
    pub fn random_service_error(&self) -> RandomServiceError { todo!() }
}
"""
        state_source = """
use crate::core::runtime::random::RandomService as Authority;
pub(crate) struct CoreRuntimeInner {
    random_service: RandomService,
    pub(crate) detached_authority: Authority,
    pub(super) qualified_authority:
        crate::core::runtime::random::RandomService,
    pub(crate) persisted_state: RandomServiceState,
}
"""

        self.assertEqual(random_service_accessor_escape_lines(accessor_source), [5, 6])
        self.assertEqual(random_service_authority_field_lines(state_source), [4, 5, 6])
        self.assertEqual(exposed_random_service_field_lines(state_source), [5, 6])

        external_impl_sources = {
            path: "impl Owner {\n    pub fn random_service(&self) -> &RandomService { todo!() }\n}\n"
            for path in ALLOWED_RANDOM_SERVICE_ACCESSOR_PATHS
        }
        external_impl_sources["zircon_runtime/src/extra_random_impl.rs"] = """
impl CoreRuntime {
    pub fn random_service(&self) -> &RandomService { todo!() }
    pub fn detached<T>(&self) -> RandomService { todo!() }
}
impl Clone for RandomService {
    fn clone(&self) -> Self { todo!() }
}
"""
        external_impl_sources["zircon_runtime/src/authority_alias.rs"] = """
pub type Authority = RandomService;
"""
        external_impl_sources["zircon_runtime/src/alias_escape.rs"] = """
use super::authority_alias::Authority;
impl CoreRuntime {
    pub fn detached(&self) -> Authority { todo!() }
}
impl Clone for Authority {
    fn clone(&self) -> Self { todo!() }
}
"""
        self.assertEqual(
            random_service_authority_surface_violations(external_impl_sources),
            [
                ("zircon_runtime/src/authority_alias.rs", 2, "forbidden_authority_alias"),
                ("zircon_runtime/src/extra_random_impl.rs", 3, "unexpected_accessor"),
                ("zircon_runtime/src/extra_random_impl.rs", 4, "escaping_accessor"),
                ("zircon_runtime/src/extra_random_impl.rs", 6, "copyable_authority"),
            ],
        )

    def test_stream_copy_scanner_handles_derive_cfg_attr_and_explicit_impl(self) -> None:
        ordinary_derive = """#[derive(Clone, Copy)]
pub struct RandomStream;
"""
        cfg_attr_derive = """#[cfg_attr(all(), derive(Clone, Copy))]
pub struct RandomStream;
"""
        explicit_impl = """pub struct RandomStream;
impl Clone for RandomStream {
    fn clone(&self) -> Self { todo!() }
}
"""
        aliased_impl = """use std::clone::Clone as Duplicate;
pub struct RandomStream;
impl Duplicate for RandomStream {
    fn clone(&self) -> Self { todo!() }
}
"""

        self.assertEqual(implicit_copy_trait_lines(ordinary_derive, "RandomStream"), [1])
        self.assertEqual(implicit_copy_trait_lines(cfg_attr_derive, "RandomStream"), [1])
        self.assertEqual(implicit_copy_trait_lines(explicit_impl, "RandomStream"), [2])
        self.assertEqual(implicit_copy_trait_lines(aliased_impl, "RandomStream"), [3])

    def test_product_sources_do_not_import_random_implementation_from_contracts(self) -> None:
        violations = []
        for path in product_rust_candidate_paths():
            source = path.read_text(encoding="utf-8")
            source_lines = source.splitlines()
            for line_number in old_random_implementation_lines(source):
                violations.append(
                    f"{path.relative_to(REPO_ROOT).as_posix()}:{line_number}: "
                    f"{source_lines[line_number - 1].strip()}"
                )
        self.assertEqual(
            [],
            violations,
            "framework random implementation consumers remain:\n"
            + "\n".join(violations),
        )

    def test_old_implementation_scanner_handles_groups_aliases_and_qualified_paths(self) -> None:
        source = """
use crate::core::framework::{
    random::{RandomService, RandomState, RandomStream as Stream},
    time::Time,
};
use crate::core as engine_core;
use engine_core::framework as contracts;

type Error = contracts::random::RandomStreamError;
type Service = zircon_runtime::core::framework::random::RandomService;
"""

        self.assertEqual(old_random_implementation_lines(source), [3, 9, 10])

    def test_old_implementation_scanner_ignores_comments_and_literals(self) -> None:
        source = r'''
// use crate::core::framework::random::RandomService;
const DOC: &str = "zircon_runtime::core::framework::random::RandomStream";
/* crate::core::framework::random::RandomStreamError */
'''

        self.assertEqual(old_random_implementation_lines(source), [])

    def test_old_implementation_scanner_handles_implicit_module_leaf_aliases(self) -> None:
        source = """
use crate::core::framework;
use framework::random;

type Service = random::RandomService;
"""

        self.assertEqual(old_random_implementation_lines(source), [5])

    def test_old_implementation_scanner_handles_glob_imports(self) -> None:
        source = """
use crate::core::framework::random::*;

type Service = RandomService;
"""

        self.assertEqual(old_random_implementation_lines(source), [4])

    def test_raw_selector_scanner_rejects_renamed_u64_constructor_parameters(self) -> None:
        source = """
impl RandomStream {
    pub(crate) fn from_seed(seed: u64, selector_value: u64) -> Self {
        todo!()
    }
}
"""

        self.assertEqual(raw_u64_stream_constructor_lines(source), [3])


if __name__ == "__main__":
    unittest.main()
