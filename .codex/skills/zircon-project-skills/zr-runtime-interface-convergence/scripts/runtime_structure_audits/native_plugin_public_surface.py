from __future__ import annotations

import re
from dataclasses import asdict, dataclass
from pathlib import Path


PLUGIN_NATIVE_REEXPORT_RE = re.compile(
    r"(?ms)^pub\s+use\s+native_plugin_loader::\{(?P<body>.*?)\};"
)
PLUGIN_NATIVE_NAMESPACE_RE = re.compile(
    r"(?ms)^pub\s+use\s+super::native_plugin_loader::\{(?P<body>.*?)\};"
)

NATIVE_PLUGIN_SYMBOL_CLASSIFICATION_ORDER = (
    "native-abi-contract-public-debt",
    "native-loader-discovery-public-debt",
    "native-live-host-runtime-public-debt",
    "native-behavior-report-public-debt",
    "native-bridge-method-public-debt",
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
    "NativePluginBehaviorV3",
    "NativePluginByteSliceV2",
    "NativePluginByteSliceV3",
    "NativePluginCallbackStatusV2",
    "NativePluginCallbackStatusV3",
    "NativePluginDescriptor",
    "NativePluginEntryReport",
    "NativePluginEntryReportV3",
    "NativePluginHostFunctionTableV3",
    "NativePluginOwnedByteBufferV2",
    "NativePluginOwnedByteBufferV3",
    "NativePluginSchemaVersionsV3",
    "ZIRCON_NATIVE_PLUGIN_ABI_VERSION",
    "ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3",
    "ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL",
    "ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3",
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
}

NATIVE_BRIDGE_METHOD_SYMBOLS = {
    "NativeBridgeCall",
    "NativeBridgeMethodBinding",
    "NativeBridgeMethodDescriptor",
    "NativeBridgeMethodFn",
    "NativeBridgeMethodManifestError",
    "NativeHostApiV3RegistrationScope",
    "NativeHostBridgeCallScope",
    "NativePluginBridgeMethodCallV3",
    "NativePluginBridgeMethodFnV3",
    "NativePluginBridgeMethodTableV3",
    "NativePluginBridgeMethodV3",
    "NativePluginLiveHostBridgeLifecycleReport",
    "NativePluginLiveHostBridgeReloadReport",
    "native_bridge_method_descriptors_from_manifest",
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
            "public_reexport_locations": [],
            "public_reexport_location_count": 0,
            "risks": [],
        }

    source = _read_text(plugin_root)
    native_source = _read_text(native_namespace) if native_namespace.exists() else ""
    root_reexport_symbols: list[str] = []
    match = PLUGIN_NATIVE_REEXPORT_RE.search(source)
    if match:
        root_reexport_symbols = _split_use_symbols(match.group("body"))

    native_namespace_symbols: list[str] = []
    namespace_match = PLUGIN_NATIVE_NAMESPACE_RE.search(native_source)
    if namespace_match:
        native_namespace_symbols = _split_use_symbols(namespace_match.group("body"))

    root_public_reexport_locations = _find_locations(
        root,
        [plugin_root],
        re.compile(r"^pub\s+use\s+native_plugin_loader::"),
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
            "migration-debt-present"
            if migration_debt
            else "classified-and-clear"
        ),
        "root_public_reexport_locations": root_public_reexport_locations,
        "root_public_reexport_location_count": len(root_public_reexport_locations),
        "public_reexport_locations": public_reexport_locations,
        "public_reexport_location_count": len(public_reexport_locations),
        "risks": risks,
    }
