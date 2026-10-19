from __future__ import annotations

import re
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable


ROOT_GRAPHICS_REEXPORT_RE = re.compile(
    r"(?ms)^pub\(crate\)\s+use\s+graphics::\{(?P<body>.*?)\};"
)

ROOT_MODULE_DECISIONS: dict[str, tuple[str, str]] = {
    "browser": ("target-host-entry", "Wasm target-web browser host facade; never a native root seat"),
    "animation": (
        "runtime-module-entry",
        "Runtime 14 keeps animation as the runtime playback module family root",
    ),
    "core": (
        "stable-facade",
        "runtime-internal core spine entry for runtime/framework/manager/math/resource",
    ),
    "dynamic_api": (
        "stable-facade",
        "host-facing runtime API namespace with separate ABI boundary guards",
    ),
    "engine_module": (
        "stable-facade",
        "module/service/manager/driver descriptor contract surface",
    ),
    "plugin": (
        "namespace-entry",
        "plugin namespace is allowed, but native loader symbols must not be flattened",
    ),
    "prelude": (
        "stable-facade",
        "developer convenience facade; contents should stay curated",
    ),
    "builtin": (
        "namespace-entry",
        "runtime-owned builtin module assembly namespace; concrete helper types stay under builtin",
    ),
    "asset": ("runtime-module-entry", "absorbed runtime asset module entry"),
    "diagnostic_log": ("runtime-module-entry", "absorbed diagnostics/log module entry"),
    "foundation": ("runtime-module-entry", "absorbed runtime foundation module entry"),
    "input": ("runtime-module-entry", "absorbed runtime input module entry"),
    "navigation": (
        "runtime-module-entry",
        "Runtime 14 keeps navigation as the built-in fallback runtime module family root",
    ),
    "operation": (
        "runtime-module-entry",
        "runtime-owned operation registry and task lifecycle surface consumed by module families",
    ),
    "platform": ("runtime-module-entry", "absorbed runtime platform module entry"),
    "scene": ("runtime-module-entry", "absorbed runtime scene/world/ECS module entry"),
    "script": ("runtime-module-entry", "absorbed runtime script/VM module entry"),
    "text": (
        "runtime-module-entry",
        "shared runtime text services and public typography contract surface",
    ),
    "ui": ("runtime-module-entry", "absorbed runtime UI module entry"),
    "graphics": (
        "graphics-rhi-deferred",
        "graphics owner module; root shape is deferred until active RHI/WGPU work settles",
    ),
    "render_graph": (
        "graphics-rhi-deferred",
        "render graph contract module; review with M6 graphics/RHI public surface",
    ),
    "rhi": (
        "graphics-rhi-deferred",
        "neutral RHI contract module; review with M6 graphics/RHI public surface",
    ),
}

ROOT_MODULE_CLASSIFICATION_ORDER = (
    "stable-facade",
    "namespace-entry",
    "runtime-module-entry",
    "graphics-rhi-deferred",
    "target-host-entry",
    "unclassified-root-module",
)


@dataclass
class Location:
    path: str
    line: int
    snippet: str


@dataclass
class RootModuleDecision:
    module: str
    classification: str
    reason: str


@dataclass
class PublicUseDecision:
    path: str
    line: int
    snippet: str
    classification: str
    reason: str


def _read_text(path: Path) -> str:
    return path.read_bytes().decode("utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _find_locations(
    root: Path,
    files: Iterable[Path],
    pattern: re.Pattern[str],
) -> list[Location]:
    results: list[Location] = []
    for path in files:
        for line_no, line in enumerate(_read_text(path).splitlines(), start=1):
            if pattern.search(line):
                results.append(
                    Location(
                        path=_relative(root, path),
                        line=line_no,
                        snippet=line.strip(),
                    )
                )
    return results


def _split_use_symbols(use_body: str) -> list[str]:
    symbols: list[str] = []
    for raw in use_body.replace("\n", " ").split(","):
        symbol = raw.strip()
        if not symbol or symbol in {"self", "super"}:
            continue
        symbols.append(symbol)
    return symbols


def _root_module_decisions(public_modules: list[str]) -> list[RootModuleDecision]:
    decisions: list[RootModuleDecision] = []
    for module in sorted(public_modules):
        classification, reason = ROOT_MODULE_DECISIONS.get(
            module,
            (
                "unclassified-root-module",
                "root module is not covered by the M1 root-surface decision table",
            ),
        )
        decisions.append(RootModuleDecision(module, classification, reason))
    return decisions


def _public_use_decision(location: Location) -> PublicUseDecision:
    if "crate::core::resource" in location.snippet:
        classification = "stable-facade"
        reason = "keeps the runtime-internal resource spine reachable without a legacy crate"
    elif "zircon_runtime_reflection_macros" in location.snippet:
        classification = "stable-facade"
        reason = "developer-facing reflection macro facade"
    else:
        classification = "unclassified-public-use"
        reason = "public use is not covered by the M1 root-surface decision table"

    return PublicUseDecision(
        path=location.path,
        line=location.line,
        snippet=location.snippet,
        classification=classification,
        reason=reason,
    )


def _group_module_decisions(
    decisions: list[RootModuleDecision],
) -> dict[str, list[str]]:
    grouped = {classification: [] for classification in ROOT_MODULE_CLASSIFICATION_ORDER}
    for decision in decisions:
        grouped.setdefault(decision.classification, []).append(decision.module)
    return {key: sorted(value) for key, value in grouped.items() if value}


def runtime_root_surface_audit(root: Path) -> dict[str, object]:
    root_file = root / "zircon_runtime" / "src" / "lib.rs"
    source = _read_text(root_file)
    try:
        root_items = _root_items(source)
    except ValueError:
        root_items = []
    public_modules = [item[-2] for item in root_items
                      if len(item) >= 4 and item[-4:-2] == ["pub", "mod"]]
    lexical_source = "\n".join("".join(item).replace("pubuse", "pub use ").replace("pub(crate)use", "pub(crate) use ")
                               for item in root_items)
    public_use_locations = [Location(_relative(root, root_file),
        next((i for i, line in enumerate(source.splitlines(), 1) if ("pub use " + item[2]) in line), 0),
        "pub use " + "".join(item[2:])) for item in root_items if item[:2] == ["pub", "use"]]
    module_decisions = _root_module_decisions(public_modules)
    public_use_decisions = [_public_use_decision(location) for location in public_use_locations]

    graphics_reexports: list[str] = []
    match = ROOT_GRAPHICS_REEXPORT_RE.search(lexical_source)
    if match:
        graphics_reexports = _split_use_symbols(match.group("body"))

    unclassified_modules = [
        decision.module
        for decision in module_decisions
        if decision.classification == "unclassified-root-module"
    ]
    unclassified_public_uses = [
        asdict(decision)
        for decision in public_use_decisions
        if decision.classification == "unclassified-public-use"
    ]
    backend_public_debt_modules = [
        decision.module
        for decision in module_decisions
        if decision.classification == "backend-public-debt"
    ]
    root_surface_migration_debt: list[str] = []
    if backend_public_debt_modules:
        root_surface_migration_debt.append(
            "backend modules exposed at runtime root: "
            + ", ".join(sorted(backend_public_debt_modules))
        )
    if graphics_reexports:
        root_surface_migration_debt.append(
            f"crate-visible graphics re-export fan-out: {len(graphics_reexports)} symbols"
        )
    if unclassified_modules:
        root_surface_migration_debt.append(
            "unclassified runtime root modules: " + ", ".join(sorted(unclassified_modules))
        )
    if unclassified_public_uses:
        root_surface_migration_debt.append(
            f"unclassified runtime root public uses: {len(unclassified_public_uses)}"
        )

    try:
        contract = validate_root_contract(source, _read_text(root / ROOT_CONTRACT_DOC),
                                          _read_text(root / "zircon_runtime/Cargo.toml"))
    except (OSError, UnicodeError) as error:
        contract = {"risks": [f"root declaration inputs unavailable: {error}"],
                    "profile_public_modules": {}, "physical_line_count": source.count("\n") + int(bool(source) and not source.endswith("\n")),
                    "source_bytes": len(source.encode("utf-8"))}
    risks = list(contract["risks"])
    for relative in ROOT_CURRENT_MIRRORS:
        try:
            if ROOT_CURRENT_ANCHOR not in _read_text(root / relative):
                risks.append(f"root current mirror missing declaration contract: {relative}")
        except (OSError, UnicodeError):
            risks.append(f"root current mirror missing: {relative}")
    if re.search(r"(?m)^pub\s+use\s+plugin::\{", lexical_source):
        risks.append("runtime crate root publicly flattens plugin symbols")
    if graphics_reexports:
        risks.append(
            "runtime crate root still has crate-visible graphics re-export fan-out; "
            "M6 should move internal graphics access behind owner modules."
        )
    for backend_module in ("rhi_wgpu",):
        if backend_module in public_modules:
            risks.append(
                f"runtime crate root exposes backend module `{backend_module}` directly; "
                "review this when graphics/RHI public contracts are narrowed."
            )

    return {
        "path": "zircon_runtime/src/lib.rs",
        "public_modules": sorted(public_modules),
        "public_module_count": len(public_modules),
        "distinct_public_module_count": len(set(public_modules)),
        "declaration_count": sum(len(item) >= 3 and item[-3] == "mod" for item in root_items),
        "path_attribute_count": sum(item[i:i + 4] == ["#", "[", "path", "="] for item in root_items for i in range(len(item) - 3)),
        "cfg_attribute_count": sum(item[i:i + 4] == ["#", "[", "cfg", "("] for item in root_items for i in range(len(item) - 3)),
        "expected_declaration_count": 28,
        "expected_path_attribute_count": 27,
        "expected_cfg_attribute_count": 25,
        **contract,
        "public_use_locations": [asdict(location) for location in public_use_locations],
        "public_use_location_count": len(public_use_locations),
        "module_decisions": [asdict(decision) for decision in module_decisions],
        "module_decision_count": len(module_decisions),
        "module_decision_groups": _group_module_decisions(module_decisions),
        "public_use_decisions": [asdict(decision) for decision in public_use_decisions],
        "public_use_decision_count": len(public_use_decisions),
        "unclassified_public_modules": sorted(unclassified_modules),
        "unclassified_public_module_count": len(unclassified_modules),
        "unclassified_public_use_locations": unclassified_public_uses,
        "unclassified_public_use_location_count": len(unclassified_public_uses),
        "root_surface_migration_debt": root_surface_migration_debt,
        "root_surface_migration_debt_count": len(root_surface_migration_debt),
        "m1_gate_status": (
            "migration-debt-present"
            if root_surface_migration_debt or risks
            else "classified-and-clear"
        ),
        "crate_visible_graphics_reexport_count": len(graphics_reexports),
        "crate_visible_graphics_reexports_sample": graphics_reexports[:20],
        "risks": risks,
    }


# The current canonical table owns module identities in both languages.
ROOT_CONTRACT_DOC = "docs/crates/zircon_runtime/core/root_surface.md"
ROOT_CURRENT_MIRRORS = (
    ROOT_CONTRACT_DOC,
    "docs/architecture/runtime-root-surface-m1.md",
    "docs/architecture/runtime-interface-convergence.md",
    "docs/architecture/generated-code-boundary.md",
    "docs/architecture/runtime-architecture-review-m0.md",
)
ROOT_CURRENT_ANCHOR = (
    "current root raw public declarations 24/24; distinct namespaces 22/22; "
    "native target-client public modules 21/21"
)
ROOT_ALLOWED_ITEMS = (
    "extern crate self as zircon_runtime;",
    '#[cfg(all(feature="native-host",target_arch="wasm32"))] '
    'compile_error!("native-host requires a native target; use --no-default-features --features core-min for the portable Wasm core");',
    '#[cfg(feature="dev-dynamic-linking")] '
    '#[allow(unused_imports,clippy::single_component_path_imports)] use zr_dev_deps_dylib as _;',
    "pub use crate::core::resource;",
    "pub use zircon_runtime_reflection_macros::{zircon_host_function,zircon_host_module,ZirconScriptType,};",
)
ROOT_HEADER = '#![cfg_attr(all(test,windows),recursion_limit="256")]'


def _root_tokens(source: str) -> list[str]:
    """Small fail-closed lexer: comments cannot supply items or hide tokens."""
    tokens: list[str] = []
    i = 0
    while i < len(source):
        if source[i] in " \t\r\n":
            i += 1
        elif source.startswith("//", i):
            end = source.find("\n", i)
            i = len(source) if end < 0 else end + 1
        elif source.startswith("/*", i):
            depth, i = 1, i + 2
            while i < len(source) and depth:
                if source.startswith("/*", i):
                    depth, i = depth + 1, i + 2
                elif source.startswith("*/", i):
                    depth, i = depth - 1, i + 2
                else:
                    i += 1
            if depth:
                raise ValueError("unterminated root comment")
        elif source[i] == '"':
            start, i = i, i + 1
            while i < len(source) and source[i] != '"':
                i += 2 if source[i] == "\\" else 1
            if i >= len(source):
                raise ValueError("unterminated root string")
            tokens.append(source[start:i + 1])
            i += 1
        elif source[i].isascii() and (source[i].isalnum() or source[i] == "_"):
            start, i = i, i + 1
            while i < len(source) and source[i].isascii() and (source[i].isalnum() or source[i] == "_"):
                i += 1
            tokens.append(source[start:i])
        elif source[i] in "#![](){},;:=":
            tokens.append(source[i])
            i += 1
        else:
            raise ValueError(f"unknown root token at {i}")
    return tokens


def _root_items(source: str) -> list[list[str]]:
    tokens = _root_tokens(source)
    header = _root_tokens(ROOT_HEADER)
    if tokens[:len(header)] != header:
        raise ValueError("root header identity changed")
    tokens = tokens[len(header):]
    items, start, stack = [], 0, []
    for i, token in enumerate(tokens):
        if token in ("[", "(", "{"):
            stack.append(token)
        elif token in ("]", ")", "}"):
            if not stack or stack.pop() != {"]": "[", ")": "(", "}": "{"}[token]:
                raise ValueError("unbalanced root item")
        elif token == ";" and not stack:
            items.append(tokens[start:i + 1])
            start = i + 1
    if stack or start != len(tokens):
        raise ValueError("unknown or incomplete root item")
    return items


def _root_registry(canonical: str) -> list[tuple[str, str, str, str]]:
    marker = "<!-- root-declaration-identities:"
    if canonical.count(marker + "start -->") != 1 or canonical.count(marker + "end -->") != 1:
        raise ValueError("missing or duplicate root identity table")
    if canonical.index(marker + "end -->") < canonical.index(marker + "start -->"):
        raise ValueError("root identity table order")
    body = canonical.split(marker + "start -->")[1].split(marker + "end -->")[0]
    rows = [tuple(cell.strip() for cell in line.strip().strip("|").split("|"))
            for line in body.splitlines() if line.startswith("| ")]
    if rows[:2] != [("visibility", "name", "path", "cfg"), ("---", "---", "---", "---")]:
        raise ValueError("root identity table header")
    rows = rows[2:]
    if any(len(row) != 4 for row in rows):
        raise ValueError("root identity table columns")
    if len(rows) != 28 or len(set(rows)) != 28:
        raise ValueError("root identity table must have 28 unique declarations")
    if any(not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", name) for _, name, _, _ in rows):
        raise ValueError("invalid root module identifier")
    if any(path != "-" and (not re.fullmatch(r"[A-Za-z0-9_/]+\.rs", path) or any(part in ("", ".", "..") for part in path.split("/"))) for _, _, path, _ in rows):
        raise ValueError("invalid root module path")
    if any(vis not in ("pub", "pub(crate)", "private") for vis, _, _, _ in rows):
        raise ValueError("unknown root visibility")
    if sum(path != "-" for _, _, path, _ in rows) != 27 or sum(cfg != "true" for _, _, _, cfg in rows) != 23:
        raise ValueError("root path/cfg attachment inventory")
    public = [name for vis, name, _, _ in rows if vis == "pub"]
    if len(public) != 24 or len(set(public)) != 22:
        raise ValueError("root public declaration/distinct inventory")
    return rows


def _root_cfg(expression: str, features: set[str], wasm: bool, test: bool = False) -> bool:
    tokens, pos = _root_tokens(expression), 0
    def value() -> bool:
        nonlocal pos
        if pos >= len(tokens):
            raise ValueError("incomplete root cfg")
        token, pos = tokens[pos], pos + 1
        if token in ("true", "test", "windows"):
            return {"true": True, "test": test, "windows": not wasm}[token]
        if token in ("feature", "target_arch"):
            if tokens[pos:pos + 1] != ["="] or pos + 1 >= len(tokens):
                raise ValueError("invalid root cfg predicate")
            literal, pos = tokens[pos + 1], pos + 2
            if len(literal) < 2 or not literal.startswith('"') or not literal.endswith('"'):
                raise ValueError("invalid root cfg value")
            if token == "target_arch" and literal != '"wasm32"':
                raise ValueError("unknown root target cfg")
            return literal[1:-1] in features if token == "feature" else wasm
        if token not in ("all", "any", "not") or tokens[pos:pos + 1] != ["("]:
            raise ValueError("unknown root cfg grammar")
        pos += 1
        values = []
        while pos < len(tokens) and tokens[pos] != ")":
            values.append(value())
            if tokens[pos:pos + 1] == [","]:
                pos += 1
            elif tokens[pos:pos + 1] != [")"]:
                raise ValueError("invalid root cfg separator")
        if tokens[pos:pos + 1] != [")"] or not values or (token == "not" and len(values) != 1):
            raise ValueError("invalid root cfg arity")
        pos += 1
        return all(values) if token == "all" else any(values) if token == "any" else not values[0]
    result = value()
    if pos != len(tokens):
        raise ValueError("trailing root cfg tokens")
    return result


def _root_feature_closure(features: dict[str, object], selected: str) -> set[str]:
    result: set[str] = set()
    pending = [selected]
    while pending:
        name = pending.pop()
        if name in result:
            continue
        entries = features.get(name)
        if not isinstance(entries, list) or any(not isinstance(item, str) for item in entries):
            raise ValueError(f"missing or invalid Cargo feature {name}")
        result.add(name)
        for item in entries:
            if item in features:
                pending.append(item)
            elif not (item.startswith("dep:") or "/" in item):
                raise ValueError(f"unknown Cargo feature dependency {item}")
    return result


def validate_root_contract(source: str, canonical: str, cargo: str) -> dict[str, object]:
    import tomllib
    risks: list[str] = []
    profiles: dict[str, list[str]] = {}
    try:
        rows = _root_registry(canonical)
        expected = []
        for vis, name, path, cfg in rows:
            item = (f"#[cfg({cfg})]" if cfg != "true" else "")
            item += f'#[path="{path}"]' if path != "-" else ""
            item += (vis + " " if vis != "private" else "") + f"mod {name};"
            expected.append(_root_tokens(item))
        allowed = [_root_tokens(item) for item in ROOT_ALLOWED_ITEMS]
        remaining = []
        for item in _root_items(source):
            if item in allowed:
                allowed.remove(item)
            else:
                remaining.append(item)
        if allowed or remaining != expected:
            raise ValueError("root declaration/path/cfg/item identity changed")
        features = tomllib.loads(cargo)["features"]
        for _, _, _, expression in rows:
            _root_cfg(expression, set(features), False, True)
            tokens = _root_tokens(expression)
            for i in range(len(tokens) - 2):
                if tokens[i:i + 2] == ["feature", "="] and tokens[i + 2][1:-1] not in features:
                    raise ValueError("unknown root Cargo feature predicate")
        for name, wasm, count in (("core-min", False, 4), ("asset-types", False, 5),
                                   ("graphics", False, 14), ("target-client", False, 21),
                                   ("target-web", True, 17)):
            closure = _root_feature_closure(features, name)
            active = [module for vis, module, _, cfg in rows
                      if vis == "pub" and _root_cfg(cfg, closure, wasm)]
            if len(active) != count or len(active) != len(set(active)):
                raise ValueError(f"root cfg/profile identity changed: {name}")
            if (wasm and "native-host" in closure) or (("browser" in active) != wasm):
                raise ValueError(f"root browser/native reachability changed: {name}")
            profiles[name] = active
        browser = next(row for row in rows if row[1] == "browser")
        for bits in range(8):
            closure = {name for bit, name in enumerate(("target-web", "native-host")) if bits & (1 << bit)}
            wasm = bool(bits & 4)
            if _root_cfg(browser[3], closure, wasm) != (wasm and "target-web" in closure and "native-host" not in closure):
                raise ValueError("root browser predicate truth table changed")
        # Exhaust the three independent alias predicates, not just named profiles.
        for bits in range(8):
            closure = {name for bit, name in enumerate(("native-host", "asset-types", "graphics"))
                       if bits & (1 << bit)}
            for alias in ("asset", "plugin"):
                if sum(_root_cfg(cfg, closure, False) for vis, name, _, cfg in rows
                       if vis == "pub" and name == alias) > 1:
                    raise ValueError(f"root alias predicates overlap: {alias}")
    except (ValueError, KeyError, TypeError, IndexError) as error:
        risks.append(str(error))
    physical_lines = source.count("\n") + int(bool(source) and not source.endswith("\n"))
    if physical_lines > 127:
        risks.append("root exceeds 127 physical lines")
    return {"risks": risks, "profile_public_modules": profiles,
            "physical_line_count": physical_lines, "source_bytes": len(source.encode("utf-8"))}
