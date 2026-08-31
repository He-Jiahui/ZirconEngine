from __future__ import annotations

from pathlib import Path


RUNTIME_API_CHILD_LINE_BUDGET = 700
RUNTIME_API_FACADE_LINE_BUDGET = 220
RUNTIME_API_FACADE_REEXPORT_BUDGET = 6
RUNTIME_API_DOMAINS = (
    "abi",
    "constants",
    "frame",
    "host",
    "session",
)
RUNTIME_API_OWNER_PATHS = (
    "abi/api_shape.rs",
    "abi/api_table.rs",
    "abi/host_api_shape.rs",
    "constants.rs",
    "frame/frame_demand.rs",
    "frame/frame_shape.rs",
    "frame/highlight_set.rs",
    "frame/viewport_pick.rs",
    "host/clipboard.rs",
    "host/host_requests.rs",
    "host/ui_action.rs",
    "host/ui_host_request.rs",
    "session/camera.rs",
    "session/editor_transform.rs",
    "session/events.rs",
    "session/operation.rs",
    "session/plugin_event_mirror.rs",
    "session/requests.rs",
    "session/session.rs",
    "session/session_identity.rs",
    "session/translated_events.rs",
    "session/viewport.rs",
)
RUNTIME_API_DOMAIN_FACADE_PATHS = (
    "abi/mod.rs",
    "frame/mod.rs",
    "host/mod.rs",
    "session/mod.rs",
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


def _owner_source_paths(owner_dir: Path) -> list[str]:
    if not owner_dir.exists():
        return []

    return sorted(
        path.relative_to(owner_dir).as_posix()
        for path in owner_dir.rglob("*.rs")
        if path.name != "mod.rs" and not path.stem.endswith("_tests")
    )


def _domain_path(owner_dir: Path, domain: str) -> Path:
    if domain == "constants":
        return owner_dir / "constants.rs"
    return owner_dir / domain / "mod.rs"


def _domain_facade_glob_reexports(owner_dir: Path) -> list[str]:
    glob_reexports: list[str] = []
    for facade_path in RUNTIME_API_DOMAIN_FACADE_PATHS:
        path = owner_dir / facade_path
        if not path.exists():
            continue
        for line in _read_text(path).splitlines():
            stripped = line.strip()
            if stripped.startswith("pub use ") and "::*;" in stripped:
                glob_reexports.append(facade_path)
                break
    return glob_reexports


def _facade_glob_reexports(facade_text: str) -> list[str]:
    return [
        line.strip()
        for line in facade_text.splitlines()
        if line.strip().startswith("pub use ") and "::*;" in line
    ]


def _facade_reexport_statement_count(facade_text: str) -> int:
    return sum(
        1
        for line in facade_text.splitlines()
        if line.strip().startswith("pub use ")
    )


def runtime_api_boundary_audit(root: Path) -> dict[str, object]:
    interface_root = root / "zircon_runtime_interface" / "src"
    facade_file = interface_root / "runtime_api" / "mod.rs"
    legacy_facade_file = interface_root / "runtime_api.rs"
    owner_dir = interface_root / "runtime_api"
    facade_text = _read_text(facade_file) if facade_file.exists() else ""

    actual_owner_paths = _owner_source_paths(owner_dir)
    missing_domains = [
        domain
        for domain in RUNTIME_API_DOMAINS
        if not _domain_path(owner_dir, domain).exists()
    ]
    missing_modules = [
        f"zircon_runtime_interface/src/runtime_api/{path}"
        for path in RUNTIME_API_OWNER_PATHS
        if not (owner_dir / path).exists()
    ]
    unexpected_modules = [
        path for path in actual_owner_paths if path not in RUNTIME_API_OWNER_PATHS
    ]
    missing_mod_declarations = [
        domain for domain in RUNTIME_API_DOMAINS if f"mod {domain};" not in facade_text
    ]
    missing_reexports = [
        domain
        for domain in RUNTIME_API_DOMAINS
        if f"pub use {domain}::{{" not in facade_text
    ]
    owner_modules = [
        {
            "path": _relative(root, owner_dir / path),
            "lines": _file_line_count(owner_dir / path),
        }
        for path in RUNTIME_API_OWNER_PATHS
        if (owner_dir / path).exists()
    ]
    oversized_modules = [
        module
        for module in owner_modules
        if module["lines"] > RUNTIME_API_CHILD_LINE_BUDGET
    ]
    facade_non_empty_lines = (
        _non_empty_line_count(facade_file) if facade_file.exists() else 0
    )
    forbidden_locations = _facade_forbidden_locations(root, facade_file)
    facade_glob_reexports = _facade_glob_reexports(facade_text)
    facade_reexport_statements = _facade_reexport_statement_count(facade_text)
    domain_facade_glob_reexports = _domain_facade_glob_reexports(owner_dir)

    risks: list[str] = []
    if not facade_file.exists():
        risks.append("zircon_runtime_interface/src/runtime_api/mod.rs is missing.")
    if legacy_facade_file.exists():
        risks.append(
            "zircon_runtime_interface/src/runtime_api.rs was superseded by the folder-backed facade and must stay absent."
        )
    if not owner_dir.is_dir():
        risks.append("zircon_runtime_interface/src/runtime_api/ is missing.")
    if missing_domains:
        risks.append("runtime_api V8 owner domains are missing from the folder-backed tree.")
    if missing_modules:
        risks.append("runtime_api V8 owner modules are missing from their declared domains.")
    if unexpected_modules:
        risks.append("runtime_api folder has unclassified production owner modules.")
    if missing_mod_declarations:
        risks.append("runtime_api/mod.rs is missing one or more V8 domain declarations.")
    if missing_reexports:
        risks.append("runtime_api/mod.rs is missing one or more V8 domain re-exports.")
    if facade_non_empty_lines > RUNTIME_API_FACADE_LINE_BUDGET:
        risks.append(
            f"runtime_api/mod.rs exceeds {RUNTIME_API_FACADE_LINE_BUDGET} non-empty lines; keep it a facade over V8 owner domains."
        )
    if forbidden_locations:
        risks.append(
            "runtime_api/mod.rs owns ABI declarations directly; move declarations into the matching owner module."
        )
    if facade_glob_reexports:
        risks.append("runtime_api/mod.rs must explicitly re-export its V8 domain surfaces.")
    if facade_reexport_statements > RUNTIME_API_FACADE_REEXPORT_BUDGET:
        risks.append(
            "runtime_api/mod.rs has too many re-export statements; group its curated V8 surface by domain."
        )
    if domain_facade_glob_reexports:
        risks.append(
            "runtime_api domain facades must explicitly re-export their owner surfaces."
        )
    if oversized_modules:
        risks.append(
            f"one or more runtime_api owner modules exceed {RUNTIME_API_CHILD_LINE_BUDGET} lines; split by domain before adding more records."
        )

    return {
        "facade_exists": facade_file.exists(),
        "legacy_facade_exists": legacy_facade_file.exists(),
        "owner_dir_exists": owner_dir.is_dir(),
        "owner_domains": RUNTIME_API_DOMAINS,
        "expected_domain_count": len(RUNTIME_API_DOMAINS),
        "missing_domains": missing_domains,
        "owner_modules": owner_modules,
        "expected_module_count": len(RUNTIME_API_OWNER_PATHS),
        "missing_modules": missing_modules,
        "unexpected_modules": unexpected_modules,
        "missing_mod_declarations": missing_mod_declarations,
        "missing_reexports": missing_reexports,
        "facade_non_empty_lines": facade_non_empty_lines,
        "max_facade_non_empty_lines": RUNTIME_API_FACADE_LINE_BUDGET,
        "facade_forbidden_locations": forbidden_locations,
        "facade_glob_reexports": facade_glob_reexports,
        "facade_reexport_statements": facade_reexport_statements,
        "max_facade_reexport_statements": RUNTIME_API_FACADE_REEXPORT_BUDGET,
        "domain_facade_glob_reexports": domain_facade_glob_reexports,
        "max_module_lines": RUNTIME_API_CHILD_LINE_BUDGET,
        "oversized_modules": oversized_modules,
        "risks": risks,
    }
