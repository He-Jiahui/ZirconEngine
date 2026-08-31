from __future__ import annotations

import re
from collections import defaultdict
from dataclasses import asdict, dataclass
from pathlib import Path


ENGINE_MODULE_IMPL_RE = re.compile(r"\bimpl\s+EngineModule\s+for\s+([A-Za-z_][A-Za-z0-9_]*)")
MODULE_DESCRIPTOR_RE = re.compile(r"\bfn\s+module_descriptor\s*\(")
STUB_MODULE_RE = re.compile(r"\bstub_module_descriptor\s*\(")


@dataclass
class Location:
    path: str
    line: int
    snippet: str


@dataclass
class RuntimeInventory:
    zircon_crates: list[str]
    all_rs_files: list[Path]
    module_crates: list[str]
    support_crates: list[str]
    hotspots: dict[str, list[tuple[str, int]]]
    descriptor_locations: dict[str, list[Location]]
    stub_usage: dict[str, list[Location]]
    owner_impls: dict[str, list[str]]


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _workspace_members(root: Path) -> list[str]:
    cargo = _read_text(root / "Cargo.toml")
    match = re.search(r"members\s*=\s*\[(.*?)\]", cargo, re.S)
    if not match:
        return []
    return re.findall(r'"([^"]+)"', match.group(1))


def _crate_rs_files(root: Path, crate_name: str) -> list[Path]:
    crate_root = root / crate_name / "src"
    if not crate_root.exists():
        return []
    return sorted(crate_root.rglob("*.rs"))


def _is_production_file(path: Path) -> bool:
    return "tests" not in path.parts and path.name != "tests.rs"


def _scan_production_file(
    root: Path,
    path: Path,
) -> tuple[list[Location], list[Location], list[str], int]:
    relative_path = _relative(root, path)
    descriptors: list[Location] = []
    stubs: list[Location] = []
    owners: list[str] = []
    lines = _read_text(path).splitlines()
    for line_no, line in enumerate(lines, start=1):
        if MODULE_DESCRIPTOR_RE.search(line):
            descriptors.append(
                Location(path=relative_path, line=line_no, snippet=line.strip())
            )
        if STUB_MODULE_RE.search(line):
            stubs.append(
                Location(path=relative_path, line=line_no, snippet=line.strip())
            )
        owner = ENGINE_MODULE_IMPL_RE.search(line)
        if owner:
            owners.append(owner.group(1))
    return descriptors, stubs, owners, len(lines)


def runtime_inventory(root: Path, hotspot_threshold: int) -> RuntimeInventory:
    zircon_crates = [
        member for member in _workspace_members(root) if member.startswith("zircon_")
    ]

    descriptor_locations: dict[str, list[Location]] = {}
    stub_usage: dict[str, list[Location]] = defaultdict(list)
    owner_impls: dict[str, list[str]] = defaultdict(list)
    hotspots: dict[str, list[tuple[str, int]]] = defaultdict(list)
    all_rs_files: list[Path] = []

    for crate_name in zircon_crates:
        production_files = [
            path
            for path in _crate_rs_files(root, crate_name)
            if _is_production_file(path)
        ]
        all_rs_files.extend(production_files)
        crate_descriptors: list[Location] = []
        crate_stubs: list[Location] = []
        crate_owners: list[str] = []
        for path in production_files:
            descriptors, stubs, owners, line_count = _scan_production_file(root, path)
            crate_descriptors.extend(descriptors)
            crate_stubs.extend(stubs)
            crate_owners.extend(owners)
            if line_count >= hotspot_threshold:
                hotspots[crate_name].append((_relative(root, path), line_count))
        descriptor_locations[crate_name] = crate_descriptors
        stub_usage[crate_name] = crate_stubs
        owner_impls[crate_name] = crate_owners
        hotspots[crate_name].sort(key=lambda item: item[1], reverse=True)

    module_crates = sorted(
        crate_name
        for crate_name in zircon_crates
        if descriptor_locations[crate_name] or owner_impls[crate_name]
    )
    support_crates = sorted(
        crate_name for crate_name in zircon_crates if crate_name not in module_crates
    )

    return RuntimeInventory(
        zircon_crates=zircon_crates,
        all_rs_files=sorted(all_rs_files),
        module_crates=module_crates,
        support_crates=support_crates,
        hotspots=dict(hotspots),
        descriptor_locations=descriptor_locations,
        stub_usage=dict(stub_usage),
        owner_impls=dict(owner_impls),
    )


def _classify_module_crates(
    inventory: RuntimeInventory,
    plugin_gap: bool,
) -> dict[str, dict[str, object]]:
    result: dict[str, dict[str, object]] = {}
    for crate_name in inventory.module_crates:
        reasons: list[str] = []
        if inventory.stub_usage[crate_name]:
            reasons.append("stub-module-descriptor")
        if not inventory.owner_impls[crate_name]:
            reasons.append("missing-engine-module-owner")
        if inventory.hotspots.get(crate_name):
            reasons.append("large-production-file")
        if crate_name == "zircon_script" and plugin_gap:
            reasons.append("plugin-runtime-gap")

        if inventory.stub_usage[crate_name]:
            status = "skeleton"
        elif reasons:
            status = "needs-refactor"
        else:
            status = "converged"

        result[crate_name] = {"status": status, "reasons": reasons}
    return result


def module_convergence_report(
    inventory: RuntimeInventory,
    plugin_gap: bool,
) -> dict[str, object]:
    return {
        "module_descriptor_distribution": {
            crate_name: [
                asdict(location)
                for location in inventory.descriptor_locations[crate_name]
            ]
            for crate_name in inventory.module_crates
        },
        "stub_module_descriptor_usage": {
            crate_name: [asdict(location) for location in inventory.stub_usage[crate_name]]
            for crate_name in inventory.module_crates
            if inventory.stub_usage[crate_name]
        },
        "engine_module_owner_coverage": {
            crate_name: inventory.owner_impls[crate_name]
            for crate_name in inventory.module_crates
        },
        "module_classification": _classify_module_crates(inventory, plugin_gap),
        "support_crates": inventory.support_crates,
    }
