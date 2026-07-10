from __future__ import annotations

from pathlib import Path


DYNAMIC_API_TEST_MODULE_MAX_LINES = 250
DYNAMIC_API_TEST_MODULES = (
    "accessibility",
    "api_table",
    "host_request_payloads",
    "host_requests",
    "input_events",
    "profile_control",
    "session_entry_points",
    "session_lifecycle",
    "session_profiles",
    "structure",
    "support",
    "viewport",
)


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _file_line_count(path: Path) -> int:
    return len(_read_text(path).splitlines())


def dynamic_api_test_boundary_audit(root: Path) -> dict[str, object]:
    dynamic_api_root = root / "zircon_runtime" / "src" / "dynamic_api"
    tests_dir = dynamic_api_root / "tests"
    legacy_tests_file = dynamic_api_root / "tests.rs"
    mod_file = tests_dir / "mod.rs"

    test_modules: list[dict[str, object]] = []
    missing_modules: list[str] = []
    missing_mod_declarations: list[str] = []
    oversized_modules: list[dict[str, object]] = []
    mod_source = _read_text(mod_file) if mod_file.exists() else ""

    for module in DYNAMIC_API_TEST_MODULES:
        path = tests_dir / f"{module}.rs"
        if not path.exists():
            missing_modules.append(f"zircon_runtime/src/dynamic_api/tests/{module}.rs")
        else:
            module_entry = {
                "path": _relative(root, path),
                "lines": _file_line_count(path),
            }
            test_modules.append(module_entry)
            if module_entry["lines"] > DYNAMIC_API_TEST_MODULE_MAX_LINES:
                oversized_modules.append(module_entry)

        if f"mod {module};" not in mod_source:
            missing_mod_declarations.append(module)

    risks: list[str] = []
    if legacy_tests_file.exists():
        risks.append(
            "zircon_runtime/src/dynamic_api/tests.rs exists; dynamic API coverage should stay folder-backed by behavior owner."
        )
    if not tests_dir.is_dir():
        risks.append("zircon_runtime/src/dynamic_api/tests/ is missing.")
    if missing_modules:
        risks.append("dynamic API test owner modules are missing from the folder-backed test tree.")
    if missing_mod_declarations:
        risks.append("dynamic_api/tests/mod.rs is missing one or more owner module declarations.")
    if oversized_modules:
        risks.append(
            f"one or more dynamic API test owner modules exceed {DYNAMIC_API_TEST_MODULE_MAX_LINES} lines; split by behavior owner before adding more cases."
        )

    return {
        "legacy_tests_file_exists": legacy_tests_file.exists(),
        "tests_dir_exists": tests_dir.is_dir(),
        "test_modules": test_modules,
        "expected_module_count": len(DYNAMIC_API_TEST_MODULES),
        "missing_modules": missing_modules,
        "missing_mod_declarations": missing_mod_declarations,
        "max_module_lines": DYNAMIC_API_TEST_MODULE_MAX_LINES,
        "oversized_modules": oversized_modules,
        "risks": risks,
    }
