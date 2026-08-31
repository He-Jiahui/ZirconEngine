from __future__ import annotations

import re
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable


ROOT_GRAPHICS_REEXPORT_RE = re.compile(
    r"(?ms)^pub\(crate\)\s+use\s+graphics::\{(?P<body>.*?)\};"
)

ROOT_MODULE_DECISIONS: dict[str, tuple[str, str]] = {
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
    return path.read_text(encoding="utf-8")


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
    public_modules = re.findall(r"(?m)^pub\s+mod\s+([A-Za-z_][A-Za-z0-9_]*);", source)
    public_use_locations = _find_locations(root, [root_file], re.compile(r"^pub\s+use\s+"))
    module_decisions = _root_module_decisions(public_modules)
    public_use_decisions = [_public_use_decision(location) for location in public_use_locations]

    graphics_reexports: list[str] = []
    match = ROOT_GRAPHICS_REEXPORT_RE.search(source)
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

    risks: list[str] = []
    if re.search(r"(?m)^pub\s+use\s+plugin::\{", source):
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
            if root_surface_migration_debt
            else "classified-and-clear"
        ),
        "crate_visible_graphics_reexport_count": len(graphics_reexports),
        "crate_visible_graphics_reexports_sample": graphics_reexports[:20],
        "risks": risks,
    }
