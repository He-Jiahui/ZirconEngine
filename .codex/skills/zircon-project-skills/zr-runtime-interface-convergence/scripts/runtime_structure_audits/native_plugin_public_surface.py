from __future__ import annotations

import re
import unicodedata
from dataclasses import asdict, dataclass
from pathlib import Path


PLUGIN_NATIVE_NAMESPACE_RE = re.compile(
    r"(?ms)^pub\s+use\s+super::native_plugin_loader::\{(?P<body>.*?)\};"
)
NATIVE_PLUGIN_SYMBOL_CLASSIFICATION_ORDER = (
    "native-abi-contract-public-debt",
    "native-loader-discovery-public-debt",
    "native-live-host-runtime-public-debt",
    "native-behavior-report-public-debt",
    "native-bridge-method-public-debt",
    "native-host-api-adapter-public-debt",
    "unclassified-native-plugin-symbol",
)

NATIVE_PLUGIN_SYMBOL_CLASSIFICATION_DECISIONS = {
    "native-abi-contract-public-debt": {
        "target_owner": "isolated native ABI contract namespace",
        "allowed_public_shape": (
            "narrow ABI contract module used by native build/tooling paths, "
            "not flattened from zircon_runtime::plugin"
        ),
        "required_action": (
            "move ABI structs, status constants, descriptor symbols, and version "
            "constants out of the runtime plugin root re-export"
        ),
    },
    "native-loader-discovery-public-debt": {
        "target_owner": "isolated native loader/discovery namespace",
        "allowed_public_shape": (
            "tooling or export facade that owns manifest discovery and loading explicitly"
        ),
        "required_action": (
            "move loader, candidate, manifest, loaded-plugin, and load-report "
            "symbols behind the native loader owner"
        ),
    },
    "native-live-host-runtime-public-debt": {
        "target_owner": "native live-host tooling/runtime bridge namespace",
        "allowed_public_shape": (
            "isolated live-host bridge used by native plugin tests or export tooling"
        ),
        "required_action": (
            "move live-host commands, runtime behavior descriptors, runtime state "
            "snapshots, and play-mode reports behind the M4 native bridge boundary"
        ),
    },
    "native-behavior-report-public-debt": {
        "target_owner": "native behavior diagnostics namespace",
        "allowed_public_shape": (
            "diagnostic report types reachable through an explicit native diagnostics owner"
        ),
        "required_action": (
            "move behavior call and validation report symbols behind the native "
            "diagnostics owner"
        ),
    },
    "native-bridge-method-public-debt": {
        "target_owner": "native bridge-method tooling/runtime bridge namespace",
        "allowed_public_shape": (
            "isolated bridge-method owner used by native plugin bridge lifecycle "
            "tests or tooling, not flattened from zircon_runtime::plugin"
        ),
        "required_action": (
            "move native bridge-method descriptors, bindings, call table entries, "
            "registration scopes, and live-host bridge reports behind the native "
            "bridge-method owner"
        ),
    },
    "native-host-api-adapter-public-debt": {
        "target_owner": "native host-API adapter namespace",
        "allowed_public_shape": (
            "isolated host-API registration policy/scope used to project versioned "
            "runtime-interface function tables, not flattened from zircon_runtime::plugin"
        ),
        "required_action": (
            "keep current host-API registration policy/scope behind the explicit native "
            "host adapter owner and retire superseded registration versions"
        ),
    },
    "unclassified-native-plugin-symbol": {
        "target_owner": "unknown",
        "allowed_public_shape": "none until classified",
        "required_action": (
            "classify this native plugin symbol with a target owner or remove it "
            "from the public root re-export"
        ),
    },
}

NATIVE_ABI_CONTRACT_SYMBOLS = {
    "NativePluginAbiV3",
    "NativePluginBehaviorV4",
    "NativePluginByteSliceV2",
    "NativePluginByteSliceV3",
    "NativePluginCallbackStatusV2",
    "NativePluginCallbackStatusV3",
    "NativePluginDescriptor",
    "NativePluginEntryReport",
    "NativePluginEntryReportV3",
    "NativePluginHostFunctionTableV3",
    "NativePluginInvokeCommandFnV4",
    "NativePluginOwnedByteBufferV2",
    "NativePluginOwnedByteBufferV3",
    "NativePluginOutputSinkV4",
    "NativePluginOutputWriteFnV4",
    "NativePluginSchemaVersionsV3",
    "ZIRCON_NATIVE_PLUGIN_ABI_VERSION",
    "ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3",
    "ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4",
    "ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL",
    "ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3",
    "ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH",
    "ZIRCON_NATIVE_PLUGIN_STATUS_DENIED",
    "ZIRCON_NATIVE_PLUGIN_STATUS_ERROR",
    "ZIRCON_NATIVE_PLUGIN_STATUS_OK",
    "ZIRCON_NATIVE_PLUGIN_STATUS_PANIC",
}

NATIVE_LOADER_DISCOVERY_SYMBOLS = {
    "LoadedNativePlugin",
    "NativePluginCandidate",
    "NativePluginLoadManifest",
    "NativePluginLoadManifestAbiV3Contract",
    "NativePluginLoadManifestEntry",
    "NativePluginLoadReport",
    "NativePluginLoadProjection",
    "NativePluginLoader",
}

NATIVE_LIVE_HOST_RUNTIME_SYMBOLS = {
    "NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND",
    "NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND",
    "NativePluginLiveHost",
    "NativePluginLiveHostCommand",
    "NativePluginLiveHostLoadReport",
    "NativePluginLiveHostOutcome",
    "NativePluginRuntimeBehaviorCall",
    "NativePluginRuntimeBehaviorDescriptor",
    "NativePluginRuntimeCommandDispatchReport",
    "NativePluginRuntimeDeltaHotUpdateReport",
    "NativePluginRuntimeDeltaHotUpdateRequest",
    "NativePluginRuntimeHotUpdateReport",
    "NativePluginRuntimePlayModeExitReport",
    "NativePluginRuntimePlayModeSnapshot",
    "NativePluginRuntimePluginState",
    "NativePluginRuntimeRegistrationReplayReport",
    "NativePluginRuntimeRegistrationSystemReplay",
    "NativePluginRuntimeStateRestoreReport",
    "NativePluginRuntimeStateSnapshot",
}

NATIVE_BEHAVIOR_REPORT_SYMBOLS = {
    "NativePluginBehaviorCallReport",
    "NativePluginBehaviorHealth",
    "NativePluginBehaviorValidationReport",
    "NativePluginCallbackDiagnostics",
    "NativePluginLiveHostDiagnostics",
}

NATIVE_BRIDGE_METHOD_SYMBOLS = {
    "NativeBridgeCall",
    "NativeBridgeMethodBinding",
    "NativeBridgeMethodDescriptor",
    "NativeBridgeMethodFn",
    "NativeBridgeMethodManifestError",
    "NativeHostBridgeCallScope",
    "NativePluginBridgeMethodCallV3",
    "NativePluginBridgeMethodFnV3",
    "NativePluginBridgeMethodTableV3",
    "NativePluginBridgeMethodV3",
    "NativePluginLiveHostBridgeLifecycleReport",
    "NativePluginLiveHostBridgeReloadReport",
    "native_bridge_method_descriptors_from_manifest",
}

NATIVE_HOST_API_ADAPTER_SYMBOLS = {
    "NativeHostApiV4RegistrationPolicy",
    "NativeHostApiV4RegistrationScope",
}


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _split_use_symbols(body: str) -> list[str]:
    return sorted(
        symbol.strip()
        for symbol in body.replace("\n", " ").split(",")
        if symbol.strip()
    )


def _mask_source_range(result: list[str], source: str, start: int, end: int) -> None:
    for index in range(start, min(end, len(source))):
        if source[index] != "\n":
            result[index] = " "


def _raw_string_end(source: str, index: int) -> int | None:
    if index > 0 and (source[index - 1].isalnum() or source[index - 1] == "_"):
        return None
    prefix_length = next(
        (len(prefix) for prefix in ("br", "cr", "r") if source.startswith(prefix, index)),
        None,
    )
    if prefix_length is None:
        return None
    cursor = index + prefix_length
    while cursor < len(source) and source[cursor] == "#":
        cursor += 1
    if cursor >= len(source) or source[cursor] != '"':
        return None
    terminator = '"' + "#" * (cursor - index - prefix_length)
    closing = source.find(terminator, cursor + 1)
    return len(source) if closing < 0 else closing + len(terminator)


def _quoted_string_end(source: str, index: int) -> int:
    cursor = index + 1
    while cursor < len(source):
        if source[cursor] == "\\":
            cursor = min(cursor + 2, len(source))
            continue
        if source[cursor] == '"':
            return cursor + 1
        cursor += 1
    return len(source)


def _char_literal_end(source: str, index: int) -> int | None:
    cursor = index + 1
    if cursor >= len(source) or source[cursor] == "\n":
        return None
    if source[cursor] == "\\":
        cursor += 1
        if cursor >= len(source):
            return None
        if source[cursor] == "u" and cursor + 1 < len(source) and source[cursor + 1] == "{":
            closing_brace = source.find("}", cursor + 2)
            if closing_brace < 0:
                return None
            cursor = closing_brace + 1
        elif source[cursor] == "x":
            cursor += 3
        else:
            cursor += 1
    else:
        cursor += 1
    return cursor + 1 if cursor < len(source) and source[cursor] == "'" else None


def _mask_rust_non_code(source: str) -> str:
    """Mask Rust comments and literals while preserving offsets and line numbers."""
    result = list(source)
    index = 0
    while index < len(source):
        character = source[index]
        following = source[index + 1] if index + 1 < len(source) else ""
        raw_string_end = _raw_string_end(source, index)
        if raw_string_end is not None:
            _mask_source_range(result, source, index, raw_string_end)
            index = raw_string_end
            continue
        if character == '"':
            string_end = _quoted_string_end(source, index)
            _mask_source_range(result, source, index, string_end)
            index = string_end
            continue
        if character == "'":
            char_end = _char_literal_end(source, index)
            if char_end is not None:
                _mask_source_range(result, source, index, char_end)
                index = char_end
                continue
        if character == "/" and following == "/":
            comment_end = source.find("\n", index + 2)
            comment_end = len(source) if comment_end < 0 else comment_end
            _mask_source_range(result, source, index, comment_end)
            index = comment_end
            continue
        if character == "/" and following == "*":
            cursor = index + 2
            block_depth = 1
            while cursor < len(source) and block_depth:
                if source.startswith("/*", cursor):
                    block_depth += 1
                    cursor += 2
                elif source.startswith("*/", cursor):
                    block_depth -= 1
                    cursor += 2
                else:
                    cursor += 1
            _mask_source_range(result, source, index, cursor)
            index = cursor
            continue
        index += 1
    return "".join(result)


def _is_rust_identifier_start(character: str) -> bool:
    return character == "_" or character.isidentifier()


def _is_rust_identifier_continue(character: str) -> bool:
    return character == "_" or f"a{character}".isidentifier()


def _keyword_at(source: str, index: int, keyword: str) -> bool:
    if not source.startswith(keyword, index):
        return False
    if index >= 2 and source[index - 2 : index] == "r#":
        return False
    before = source[index - 1] if index else ""
    after_index = index + len(keyword)
    after = source[after_index] if after_index < len(source) else ""
    return not (before and _is_rust_identifier_continue(before)) and not (
        after and _is_rust_identifier_continue(after)
    )


def _skip_whitespace(source: str, index: int) -> int:
    while index < len(source) and source[index].isspace():
        index += 1
    return index


def _balanced_end(
    source: str,
    index: int,
    opening: str,
    closing: str,
) -> int | None:
    if index >= len(source) or source[index] != opening:
        return None
    depth = 1
    index += 1
    while index < len(source):
        if source[index] == opening:
            depth += 1
        elif source[index] == closing:
            depth -= 1
            if depth == 0:
                return index + 1
        index += 1
    return None


def _rust_identifier_at(source: str, index: int) -> tuple[str, int] | None:
    identifier_start = index
    if (
        source.startswith("r#", index)
        and index + 2 < len(source)
        and _is_rust_identifier_start(source[index + 2])
    ):
        identifier_start = index + 2
    if identifier_start >= len(source) or not _is_rust_identifier_start(
        source[identifier_start]
    ):
        return None
    identifier_end = identifier_start + 1
    while identifier_end < len(source) and _is_rust_identifier_continue(
        source[identifier_end]
    ):
        identifier_end += 1
    identifier = unicodedata.normalize(
        "NFC",
        source[identifier_start:identifier_end],
    )
    return identifier, identifier_end


def _rust_use_tokens(source: str) -> list[str]:
    tokens: list[str] = []
    index = 0
    while index < len(source):
        if source.startswith("::", index):
            tokens.append("::")
            index += 2
            continue
        if source[index] in "*{},":
            tokens.append(source[index])
            index += 1
            continue

        identifier = _rust_identifier_at(source, index)
        if identifier is not None:
            tokens.append(identifier[0])
            index = identifier[1]
            continue
        index += 1
    return tokens


def _extern_crate_alias(route: str) -> tuple[str, tuple[str, ...]] | None:
    tokens = [
        token
        for token in _rust_use_tokens(route)
        if token not in {"::", "*", "{", "}", ","}
    ]
    if len(tokens) < 2 or tokens[:2] != ["extern", "crate"]:
        return None
    crate_name = tokens[2] if len(tokens) >= 3 else ""
    if not crate_name:
        return None
    alias = crate_name
    if len(tokens) >= 5 and tokens[3] == "as":
        alias = tokens[4]
    target = ("crate",) if crate_name == "self" else (f"extern:{crate_name}",)
    return alias, target


def _root_scope_imports(
    source: str,
) -> tuple[
    list[
        tuple[
            int,
            bool,
            str,
            list[tuple[tuple[str, ...], str | None]],
        ]
    ],
    dict[str, set[tuple[str, ...]]],
]:
    statements: list[
        tuple[
            int,
            bool,
            str,
            list[tuple[tuple[str, ...], str | None]],
        ]
    ] = []
    extern_aliases: dict[str, set[tuple[str, ...]]] = {}
    opening_to_closing = {"{": "}", "[": "]", "(": ")"}
    closing = set(opening_to_closing.values())
    stack: list[str] = []
    index = 0

    while index < len(source):
        character = source[index]
        if character in opening_to_closing:
            stack.append(opening_to_closing[character])
            index += 1
            continue
        if character in closing:
            if stack and stack[-1] == character:
                stack.pop()
            index += 1
            continue
        if stack:
            index += 1
            continue

        start = index
        public = False
        cursor = index
        if _keyword_at(source, cursor, "pub"):
            public = True
            cursor = _skip_whitespace(source, cursor + len("pub"))
            if cursor < len(source) and source[cursor] == "(":
                visibility_end = _balanced_end(source, cursor, "(", ")")
                if visibility_end is None:
                    index += 1
                    continue
                cursor = _skip_whitespace(source, visibility_end)

        is_use = _keyword_at(source, cursor, "use")
        is_extern = _keyword_at(source, cursor, "extern")
        extern_crate_start = _skip_whitespace(source, cursor + len("extern"))
        is_extern_crate = is_extern and _keyword_at(
            source,
            extern_crate_start,
            "crate",
        )
        if not is_use and not is_extern_crate:
            index += 1
            continue

        statement_end = source.find(";", cursor)
        if statement_end < 0:
            index += 1
            continue
        route_start = cursor + (len("use") if is_use else 0)
        route = source[route_start:statement_end].strip()
        if is_use:
            statements.append(
                (start, public, route, _flatten_use_route(route))
            )
        else:
            alias = _extern_crate_alias(source[cursor:statement_end])
            if alias is not None:
                extern_aliases.setdefault(alias[0], set()).add(alias[1])
        index = statement_end + 1

    return statements, extern_aliases


def _root_macro_invocation_locations(
    root: Path,
    plugin_root: Path,
    source: str,
) -> list[dict[str, object]]:
    sanitized = _mask_rust_non_code(source)
    opening_to_closing = {"{": "}", "[": "]", "(": ")"}
    closing = set(opening_to_closing.values())
    stack: list[str] = []
    locations: list[dict[str, object]] = []
    index = 0

    while index < len(sanitized):
        character = sanitized[index]
        if character in opening_to_closing:
            stack.append(opening_to_closing[character])
            index += 1
            continue
        if character in closing:
            if stack and stack[-1] == character:
                stack.pop()
            index += 1
            continue
        if stack:
            index += 1
            continue

        identifier = _rust_identifier_at(sanitized, index)
        if identifier is None:
            index += 1
            continue
        name, identifier_end = identifier
        bang = _skip_whitespace(sanitized, identifier_end)
        delimiter = _skip_whitespace(sanitized, bang + 1)
        is_macro_rules_definition = (
            name == "macro_rules" and not sanitized.startswith("r#", index)
        )
        if (
            not is_macro_rules_definition
            and bang < len(sanitized)
            and sanitized[bang] == "!"
            and delimiter < len(sanitized)
            and sanitized[delimiter] in opening_to_closing
        ):
            line_start = source.rfind("\n", 0, index) + 1
            line_end = source.find("\n", index)
            if line_end < 0:
                line_end = len(source)
            locations.append(
                {
                    "path": _relative(root, plugin_root),
                    "line": source.count("\n", 0, index) + 1,
                    "snippet": source[line_start:line_end].strip(),
                }
            )
        index = identifier_end

    return locations


def _flatten_use_route(route: str) -> list[tuple[tuple[str, ...], str | None]]:
    tokens = _rust_use_tokens(route)

    def parse_item(
        index: int,
        prefix: tuple[str, ...],
    ) -> tuple[list[tuple[tuple[str, ...], str | None]], int]:
        if index >= len(tokens):
            return [], index
        if tokens[index] == "{":
            return parse_group(index + 1, prefix)

        segment = tokens[index]
        path = (*prefix, segment)
        index += 1
        if index < len(tokens) and tokens[index] == "as":
            alias = tokens[index + 1] if index + 1 < len(tokens) else None
            return [(path, alias)], min(index + 2, len(tokens))
        if index < len(tokens) and tokens[index] == "::":
            return parse_item(index + 1, path)
        return [(path, None)], index

    def parse_group(
        index: int,
        prefix: tuple[str, ...],
    ) -> tuple[list[tuple[tuple[str, ...], str | None]], int]:
        paths: list[tuple[tuple[str, ...], str | None]] = []
        while index < len(tokens):
            if tokens[index] == "}":
                return paths, index + 1
            if tokens[index] == ",":
                index += 1
                continue
            parsed, index = parse_item(index, prefix)
            paths.extend(parsed)
        return paths, index

    paths, _ = parse_group(0, ())
    return paths


def _use_binding(path: tuple[str, ...], alias: str | None) -> str | None:
    if alias:
        return None if alias == "_" else alias
    if not path or path[-1] == "*":
        return None
    if path[-1] == "self":
        return path[-2] if len(path) >= 2 else None
    return path[-1]


def _use_aliases(
    statements: list[
        tuple[
            int,
            bool,
            str,
            list[tuple[tuple[str, ...], str | None]],
        ]
    ],
    extern_aliases: dict[str, set[tuple[str, ...]]],
) -> dict[str, set[tuple[str, ...]]]:
    aliases = {binding: set(paths) for binding, paths in extern_aliases.items()}
    for _, _, _, paths in statements:
        for path, alias in paths:
            binding = _use_binding(path, alias)
            if binding:
                target = path[:-1] if path[-1] == "self" else path
                aliases.setdefault(binding, set()).add(target)
    return aliases


def _resolve_use_paths(
    path: tuple[str, ...],
    aliases: dict[str, set[tuple[str, ...]]],
) -> set[tuple[str, ...]]:
    pending: list[tuple[tuple[str, ...], frozenset[str]]] = [(path, frozenset())]
    resolved: set[tuple[str, ...]] = set()
    while pending:
        candidate, visited = pending.pop()
        while candidate and candidate[0] in {"self", "super"}:
            candidate = candidate[1:]
        if not candidate:
            resolved.add(candidate)
            continue
        binding = candidate[0]
        targets = aliases.get(binding)
        if not targets or binding in visited:
            resolved.add(candidate)
            continue
        next_visited = visited | {binding}
        pending.extend(
            ((*target, *candidate[1:]), next_visited)
            for target in targets
        )
    return resolved


def _is_native_use_path(
    path: tuple[str, ...],
    aliases: dict[str, set[tuple[str, ...]]],
) -> bool:
    return any(
        resolved
        and (
            resolved[0] in {"native", "native_plugin_loader"}
            or len(resolved) >= 2
            and resolved[:2]
            in {
                ("plugin", "native"),
                ("plugin", "native_plugin_loader"),
            }
            or len(resolved) >= 3
            and resolved[:3]
            in {
                ("crate", "plugin", "native"),
                ("crate", "plugin", "native_plugin_loader"),
            }
        )
        for resolved in _resolve_use_paths(path, aliases)
    )


def _native_root_reexports(
    root: Path,
    plugin_root: Path,
    source: str,
    native_namespace_symbols: list[str],
) -> tuple[list[str], list[dict[str, object]]]:
    symbols: set[str] = set()
    locations: list[dict[str, object]] = []
    sanitized = _mask_rust_non_code(source)
    statements, extern_aliases = _root_scope_imports(sanitized)
    aliases = _use_aliases(statements, extern_aliases)
    for start, public, route, paths in statements:
        if not public:
            continue
        route = " ".join(route.split())
        if not any(_is_native_use_path(path, aliases) for path, _ in paths):
            continue

        known_symbols = {
            symbol
            for symbol in native_namespace_symbols
            if re.search(rf"\b{re.escape(symbol)}\b", route)
        }
        if known_symbols:
            symbols.update(known_symbols)
        else:
            symbols.add(route)
        locations.append(
            {
                "path": _relative(root, plugin_root),
                "line": source.count("\n", 0, start) + 1,
                "snippet": f"pub use {route};",
            }
        )
    return sorted(symbols), locations


@dataclass
class NativePluginSymbolDecision:
    symbol: str
    classification: str
    target_owner: str
    allowed_public_shape: str
    required_action: str


def _classify_symbol(symbol: str) -> str:
    if symbol in NATIVE_ABI_CONTRACT_SYMBOLS:
        return "native-abi-contract-public-debt"
    if symbol in NATIVE_LOADER_DISCOVERY_SYMBOLS:
        return "native-loader-discovery-public-debt"
    if symbol in NATIVE_LIVE_HOST_RUNTIME_SYMBOLS:
        return "native-live-host-runtime-public-debt"
    if symbol in NATIVE_BEHAVIOR_REPORT_SYMBOLS:
        return "native-behavior-report-public-debt"
    if symbol in NATIVE_BRIDGE_METHOD_SYMBOLS:
        return "native-bridge-method-public-debt"
    if symbol in NATIVE_HOST_API_ADAPTER_SYMBOLS:
        return "native-host-api-adapter-public-debt"
    return "unclassified-native-plugin-symbol"


def _symbol_decisions(symbols: list[str]) -> list[NativePluginSymbolDecision]:
    decisions: list[NativePluginSymbolDecision] = []
    for symbol in sorted(symbols):
        classification = _classify_symbol(symbol)
        decision = NATIVE_PLUGIN_SYMBOL_CLASSIFICATION_DECISIONS[classification]
        decisions.append(
            NativePluginSymbolDecision(
                symbol=symbol,
                classification=classification,
                target_owner=decision["target_owner"],
                allowed_public_shape=decision["allowed_public_shape"],
                required_action=decision["required_action"],
            )
        )
    return decisions


def _group_symbol_decisions(
    decisions: list[NativePluginSymbolDecision],
) -> dict[str, list[str]]:
    grouped = {
        classification: [] for classification in NATIVE_PLUGIN_SYMBOL_CLASSIFICATION_ORDER
    }
    for decision in decisions:
        grouped.setdefault(decision.classification, []).append(decision.symbol)
    return {key: sorted(value) for key, value in grouped.items() if value}


def _native_plugin_migration_debt(
    decision_groups: dict[str, list[str]],
) -> list[str]:
    debt: list[str] = []
    for classification in NATIVE_PLUGIN_SYMBOL_CLASSIFICATION_ORDER:
        symbols = decision_groups.get(classification, [])
        if not symbols:
            continue
        decision = NATIVE_PLUGIN_SYMBOL_CLASSIFICATION_DECISIONS[classification]
        debt.append(
            f"{classification}: {decision['required_action']} "
            f"({len(symbols)} symbol(s))"
        )
    return debt


def _find_locations(root: Path, files: list[Path], pattern: re.Pattern[str]) -> list[dict[str, object]]:
    results: list[dict[str, object]] = []
    for path in files:
        for line_no, line in enumerate(_read_text(path).splitlines(), start=1):
            if pattern.search(line):
                results.append(
                    {
                        "path": _relative(root, path),
                        "line": line_no,
                        "snippet": line.strip(),
                    }
                )
    return results


def native_plugin_public_surface_audit(root: Path) -> dict[str, object]:
    plugin_root = root / "zircon_runtime" / "src" / "plugin" / "mod.rs"
    native_namespace = root / "zircon_runtime" / "src" / "plugin" / "native.rs"
    if not plugin_root.exists():
        return {
            "path": "zircon_runtime/src/plugin/mod.rs",
            "native_namespace_path": "zircon_runtime/src/plugin/native.rs",
            "root_reexport_symbols": [],
            "root_reexport_count": 0,
            "root_reexport_symbols_sample": [],
            "native_namespace_symbols": [],
            "native_namespace_reexport_count": 0,
            "native_namespace_symbols_sample": [],
            "symbol_decisions": [],
            "symbol_decision_count": 0,
            "symbol_decision_groups": {},
            "symbol_decision_group_count": 0,
            "native_namespace_symbol_decisions": [],
            "native_namespace_symbol_decision_count": 0,
            "native_namespace_symbol_decision_groups": {},
            "native_namespace_symbol_group_count": 0,
            "root_symbol_decision_groups": {},
            "unclassified_root_reexport_symbols": [],
            "unclassified_root_reexport_symbol_count": 0,
            "unclassified_native_namespace_symbols": [],
            "unclassified_native_namespace_symbol_count": 0,
            "native_plugin_public_surface_migration_debt": [],
            "native_plugin_public_surface_migration_debt_count": 0,
            "m4_gate_status": "classified-and-clear",
            "root_public_reexport_locations": [],
            "root_public_reexport_location_count": 0,
            "root_macro_invocation_locations": [],
            "root_macro_invocation_count": 0,
            "public_reexport_locations": [],
            "public_reexport_location_count": 0,
            "risks": [],
        }

    source = _read_text(plugin_root)
    native_source = _read_text(native_namespace) if native_namespace.exists() else ""
    native_namespace_symbols: list[str] = []
    namespace_match = PLUGIN_NATIVE_NAMESPACE_RE.search(native_source)
    if namespace_match:
        native_namespace_symbols = _split_use_symbols(namespace_match.group("body"))

    root_reexport_symbols, root_public_reexport_locations = _native_root_reexports(
        root,
        plugin_root,
        source,
        native_namespace_symbols,
    )
    root_macro_invocation_locations = _root_macro_invocation_locations(
        root,
        plugin_root,
        source,
    )
    public_reexport_locations = _find_locations(
        root,
        [native_namespace] if native_namespace.exists() else [],
        re.compile(r"^pub\s+use\s+super::native_plugin_loader::"),
    )

    root_symbol_decisions = _symbol_decisions(root_reexport_symbols)
    root_symbol_decision_groups = _group_symbol_decisions(root_symbol_decisions)
    native_namespace_symbol_decisions = _symbol_decisions(native_namespace_symbols)
    native_namespace_symbol_decision_groups = _group_symbol_decisions(
        native_namespace_symbol_decisions
    )
    unclassified_root_reexport_symbols = root_symbol_decision_groups.get(
        "unclassified-native-plugin-symbol",
        [],
    )
    unclassified_native_namespace_symbols = (
        native_namespace_symbol_decision_groups.get(
            "unclassified-native-plugin-symbol",
            [],
        )
    )
    migration_debt = _native_plugin_migration_debt(root_symbol_decision_groups)

    risks: list[str] = []
    if "pub mod native;" not in source:
        risks.append("zircon_runtime::plugin does not expose the plugin::native namespace.")
    if root_reexport_symbols:
        risks.append(
            "zircon_runtime::plugin publicly re-exports native plugin loader/ABI symbols; "
            "M4 should keep native loading isolated behind zircon_runtime::plugin::native."
        )
    if root_macro_invocation_locations:
        risks.append(
            "zircon_runtime::plugin invokes a macro at the facade root; root declarations and "
            "re-exports must remain explicit so the native hard-cut surface is statically auditable."
        )
    if unclassified_root_reexport_symbols:
        risks.append(
            "native plugin root re-export symbols are not classified by the M4 gate: "
            + ", ".join(unclassified_root_reexport_symbols)
        )
    if not native_namespace_symbols:
        risks.append("zircon_runtime::plugin::native does not expose the native loader namespace.")
    if unclassified_native_namespace_symbols:
        risks.append(
            "native plugin namespace symbols are not classified by the M4 gate: "
            + ", ".join(unclassified_native_namespace_symbols)
        )

    return {
        "path": "zircon_runtime/src/plugin/mod.rs",
        "native_namespace_path": "zircon_runtime/src/plugin/native.rs",
        "root_reexport_symbols": root_reexport_symbols,
        "root_reexport_count": len(root_reexport_symbols),
        "root_reexport_symbols_sample": root_reexport_symbols[:24],
        "native_namespace_symbols": native_namespace_symbols,
        "native_namespace_reexport_count": len(native_namespace_symbols),
        "native_namespace_symbols_sample": native_namespace_symbols[:24],
        "symbol_decisions": [
            asdict(decision) for decision in native_namespace_symbol_decisions
        ],
        "symbol_decision_count": len(native_namespace_symbol_decisions),
        "symbol_decision_groups": native_namespace_symbol_decision_groups,
        "symbol_decision_group_count": len(native_namespace_symbol_decision_groups),
        "native_namespace_symbol_decisions": [
            asdict(decision) for decision in native_namespace_symbol_decisions
        ],
        "native_namespace_symbol_decision_count": len(native_namespace_symbol_decisions),
        "native_namespace_symbol_decision_groups": native_namespace_symbol_decision_groups,
        "native_namespace_symbol_group_count": len(native_namespace_symbol_decision_groups),
        "root_symbol_decision_groups": root_symbol_decision_groups,
        "unclassified_root_reexport_symbols": unclassified_root_reexport_symbols,
        "unclassified_root_reexport_symbol_count": len(unclassified_root_reexport_symbols),
        "unclassified_native_namespace_symbols": unclassified_native_namespace_symbols,
        "unclassified_native_namespace_symbol_count": len(
            unclassified_native_namespace_symbols
        ),
        "native_plugin_public_surface_migration_debt": migration_debt,
        "native_plugin_public_surface_migration_debt_count": len(migration_debt),
        "m4_gate_status": (
            "root-macro-invocation-present"
            if root_macro_invocation_locations
            else "migration-debt-present"
            if migration_debt
            else "classified-and-clear"
        ),
        "root_public_reexport_locations": root_public_reexport_locations,
        "root_public_reexport_location_count": len(root_public_reexport_locations),
        "root_macro_invocation_locations": root_macro_invocation_locations,
        "root_macro_invocation_count": len(root_macro_invocation_locations),
        "public_reexport_locations": public_reexport_locations,
        "public_reexport_location_count": len(public_reexport_locations),
        "risks": risks,
    }
