from __future__ import annotations

from pathlib import Path


ECS_QUERY_STATE_MODULE_MAX_LINES = 450
ECS_QUERY_STATE_ROOT_MAX_NON_EMPTY_LINES = 32
ECS_QUERY_STATE_MODULES = (
    "mod",
    "archetype_plan",
    "cache",
    "cached_direct",
    "many_item_array",
    "mutable",
    "read_only",
    "read_only_cached",
    "state",
    "stats",
    "system_param",
)
ECS_QUERY_STATE_ROOT_FORBIDDEN_SNIPPETS = {
    "state-declaration": "pub struct QueryState",
    "state-impl": "impl<D, F> QueryState",
    "cached-direct-impl": "D: CachedQueryData",
    "read-only-impl": "D: QueryData,",
    "mutable-impl": "D: QueryMutData",
    "system-param-impl": "impl<D, F> SystemParam",
}


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _file_line_count(path: Path) -> int:
    return len(_read_text(path).splitlines())


def _non_empty_line_count(path: Path) -> int:
    return sum(1 for line in _read_text(path).splitlines() if line.strip())


def _root_forbidden_locations(root: Path, path: Path) -> list[dict[str, object]]:
    locations: list[dict[str, object]] = []
    if not path.exists():
        return locations

    for line_no, line in enumerate(_read_text(path).splitlines(), start=1):
        for label, snippet in ECS_QUERY_STATE_ROOT_FORBIDDEN_SNIPPETS.items():
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


def ecs_query_state_boundary_audit(root: Path) -> dict[str, object]:
    query_root = root / "zircon_runtime" / "src" / "scene" / "ecs" / "query"
    legacy_file = query_root / "query_state.rs"
    query_mod_file = query_root / "mod.rs"
    owner_dir = query_root / "query_state"
    owner_root = owner_dir / "mod.rs"

    owner_modules: list[dict[str, object]] = []
    missing_modules: list[str] = []
    oversized_modules: list[dict[str, object]] = []
    actual_modules = sorted(path.stem for path in owner_dir.glob("*.rs")) if owner_dir.exists() else []
    unexpected_modules = [
        module for module in actual_modules if module not in ECS_QUERY_STATE_MODULES
    ]

    for module in ECS_QUERY_STATE_MODULES:
        path = owner_dir / f"{module}.rs"
        if not path.exists():
            missing_modules.append(
                f"zircon_runtime/src/scene/ecs/query/query_state/{module}.rs"
            )
            continue

        module_entry = {
            "path": _relative(root, path),
            "lines": _file_line_count(path),
        }
        owner_modules.append(module_entry)
        if module_entry["lines"] > ECS_QUERY_STATE_MODULE_MAX_LINES:
            oversized_modules.append(module_entry)

    query_mod_declares_owner = (
        query_mod_file.exists() and "mod query_state;" in _read_text(query_mod_file)
    )
    root_non_empty_lines = (
        _non_empty_line_count(owner_root) if owner_root.exists() else 0
    )
    root_forbidden_locations = _root_forbidden_locations(root, owner_root)

    risks: list[str] = []
    if legacy_file.exists():
        risks.append(
            "zircon_runtime/src/scene/ecs/query/query_state.rs exists; QueryState should stay folder-backed by query behavior owner."
        )
    if not owner_dir.is_dir():
        risks.append("zircon_runtime/src/scene/ecs/query/query_state/ is missing.")
    if missing_modules:
        risks.append("ECS QueryState owner modules are missing from the folder-backed tree.")
    if unexpected_modules:
        risks.append("ECS QueryState folder has unexpected owner modules; update the boundary intentionally.")
    if not query_mod_declares_owner:
        risks.append("scene/ecs/query/mod.rs is missing the `mod query_state;` declaration.")
    if root_non_empty_lines > ECS_QUERY_STATE_ROOT_MAX_NON_EMPTY_LINES:
        risks.append(
            f"QueryState root exceeds {ECS_QUERY_STATE_ROOT_MAX_NON_EMPTY_LINES} non-empty lines; keep it to module wiring and curated exports."
        )
    if root_forbidden_locations:
        risks.append(
            "QueryState root contains state declarations or behavior impl families that belong in focused child owners."
        )
    if oversized_modules:
        risks.append(
            f"one or more QueryState owner modules exceed {ECS_QUERY_STATE_MODULE_MAX_LINES} lines; split by behavior owner before adding more ECS query paths."
        )

    return {
        "legacy_file_exists": legacy_file.exists(),
        "owner_dir_exists": owner_dir.is_dir(),
        "owner_modules": owner_modules,
        "expected_module_count": len(ECS_QUERY_STATE_MODULES),
        "missing_modules": missing_modules,
        "unexpected_modules": unexpected_modules,
        "query_mod_declares_owner": query_mod_declares_owner,
        "root_non_empty_lines": root_non_empty_lines,
        "max_root_non_empty_lines": ECS_QUERY_STATE_ROOT_MAX_NON_EMPTY_LINES,
        "root_forbidden_locations": root_forbidden_locations,
        "max_module_lines": ECS_QUERY_STATE_MODULE_MAX_LINES,
        "oversized_modules": oversized_modules,
        "risks": risks,
    }
