from __future__ import annotations

from pathlib import Path


RUNTIME_API_CHILD_LINE_BUDGET = 700
RUNTIME_API_FACADE_LINE_BUDGET = 20
RUNTIME_API_MODULES = (
    "api_table",
    "constants",
    "events",
    "host_requests",
    "operation",
    "plugin_event_mirror",
    "requests",
    "viewport",
)
RUNTIME_API_FACADE_FORBIDDEN_SNIPPETS = {
    "repr-abi-declaration": "#[repr(",
    "struct-declaration": "pub struct ",
    "enum-declaration": "pub enum ",
    "const-declaration": "pub const ",
    "type-declaration": "pub type ",
}


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _file_line_count(path: Path) -> int:
    return len(_read_text(path).splitlines())


def _non_empty_line_count(path: Path) -> int:
    return sum(1 for line in _read_text(path).splitlines() if line.strip())


def _facade_forbidden_locations(root: Path, path: Path) -> list[dict[str, object]]:
    locations: list[dict[str, object]] = []
    if not path.exists():
        return locations

    for line_no, line in enumerate(_read_text(path).splitlines(), start=1):
        for label, snippet in RUNTIME_API_FACADE_FORBIDDEN_SNIPPETS.items():
            if snippet in line:
                locations.append(
                    {
                        "path": _relative(root, path),
                        "line": line_no,
                        "kind": label,
                        "snippet": line.strip(),
                    }
                )
    return locations


def runtime_api_boundary_audit(root: Path) -> dict[str, object]:
    interface_root = root / "zircon_runtime_interface" / "src"
    facade_file = interface_root / "runtime_api.rs"
    owner_dir = interface_root / "runtime_api"
    facade_text = _read_text(facade_file) if facade_file.exists() else ""

    owner_modules: list[dict[str, object]] = []
    missing_modules: list[str] = []
    missing_mod_declarations: list[str] = []
    missing_reexports: list[str] = []
    oversized_modules: list[dict[str, object]] = []
    actual_modules = sorted(path.stem for path in owner_dir.glob("*.rs")) if owner_dir.exists() else []
    unexpected_modules = [
        module for module in actual_modules if module not in RUNTIME_API_MODULES
    ]

    for module in RUNTIME_API_MODULES:
        path = owner_dir / f"{module}.rs"
        if not path.exists():
            missing_modules.append(
                f"zircon_runtime_interface/src/runtime_api/{module}.rs"
            )
        else:
            module_entry = {
                "path": _relative(root, path),
                "lines": _file_line_count(path),
            }
            owner_modules.append(module_entry)
            if module_entry["lines"] > RUNTIME_API_CHILD_LINE_BUDGET:
                oversized_modules.append(module_entry)

        if f"mod {module};" not in facade_text:
            missing_mod_declarations.append(module)
        if f"pub use {module}::*;" not in facade_text:
            missing_reexports.append(module)

    facade_non_empty_lines = (
        _non_empty_line_count(facade_file) if facade_file.exists() else 0
    )
    forbidden_locations = _facade_forbidden_locations(root, facade_file)

    risks: list[str] = []
    if not facade_file.exists():
        risks.append("zircon_runtime_interface/src/runtime_api.rs is missing.")
    if not owner_dir.is_dir():
        risks.append("zircon_runtime_interface/src/runtime_api/ is missing.")
    if missing_modules:
        risks.append("runtime_api ABI owner modules are missing from the folder-backed tree.")
    if unexpected_modules:
        risks.append("runtime_api folder has unexpected owner modules; update the ABI boundary intentionally.")
    if missing_mod_declarations:
        risks.append("runtime_api.rs is missing one or more owner module declarations.")
    if missing_reexports:
        risks.append("runtime_api.rs is missing one or more owner module re-exports.")
    if facade_non_empty_lines > RUNTIME_API_FACADE_LINE_BUDGET:
        risks.append(
            f"runtime_api.rs exceeds {RUNTIME_API_FACADE_LINE_BUDGET} non-empty lines; keep it a facade over ABI owner modules."
        )
    if forbidden_locations:
        risks.append(
            "runtime_api.rs owns ABI declarations directly; move declarations into the matching owner module."
        )
    if oversized_modules:
        risks.append(
            f"one or more runtime_api owner modules exceed {RUNTIME_API_CHILD_LINE_BUDGET} lines; split by ABI family before adding more records."
        )

    return {
        "facade_exists": facade_file.exists(),
        "owner_dir_exists": owner_dir.is_dir(),
        "owner_modules": owner_modules,
        "expected_module_count": len(RUNTIME_API_MODULES),
        "missing_modules": missing_modules,
        "unexpected_modules": unexpected_modules,
        "missing_mod_declarations": missing_mod_declarations,
        "missing_reexports": missing_reexports,
        "facade_non_empty_lines": facade_non_empty_lines,
        "max_facade_non_empty_lines": RUNTIME_API_FACADE_LINE_BUDGET,
        "facade_forbidden_locations": forbidden_locations,
        "max_module_lines": RUNTIME_API_CHILD_LINE_BUDGET,
        "oversized_modules": oversized_modules,
        "risks": risks,
    }
