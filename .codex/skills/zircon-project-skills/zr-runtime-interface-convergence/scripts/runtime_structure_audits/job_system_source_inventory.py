from __future__ import annotations

from pathlib import Path


JOB_SYSTEM_MODULE_MAX_LINES = 500
JOB_SYSTEM_MODULES = (
    "diagnostics",
    "job_handle",
    "job_scheduler",
    "mod",
    "parallel_for",
    "pool",
    "pools",
    "report",
    "thread_assignment",
)
EXPECTED_JOB_SYSTEM_GUARD_FILE_COUNT = 2
JOB_SYSTEM_GUARD_FILES = (
    "zircon_runtime/src/tests/runtime_absorption/job_system.rs",
    "zircon_runtime/src/tests/runtime_absorption/job_system/mirror_docs.rs",
)
RAYON_CLASSIFICATIONS = {
    "zircon_runtime/src/core/runtime/tasks/pool.rs": "core-task-pool-rayon-owner",
    "zircon_runtime/src/core/runtime/tasks/parallel_for.rs": "core-task-parallel-for-owner",
}


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _is_test_path(relative_path: str) -> bool:
    file_name = relative_path.rsplit("/", maxsplit=1)[-1]
    return (
        "/tests/" in relative_path
        or file_name == "tests.rs"
        or file_name.endswith("_tests.rs")
    )


def _rust_source_files(root: Path) -> list[Path]:
    files = [
        path
        for path in root.rglob("*.rs")
        if path.is_file() and not _is_test_path(_relative(root.parent, path))
    ]
    files.sort()
    return files


def _line_mentions_rayon(line: str) -> bool:
    return (
        "use rayon" in line
        or "rayon::" in line
        or ".par_iter(" in line
        or ".par_chunks" in line
        or ".into_par_iter(" in line
    )


def collect_direct_rayon_references(root: Path) -> list[dict[str, object]]:
    source_root = root / "zircon_runtime" / "src"
    references: list[dict[str, object]] = []
    if not source_root.is_dir():
        return references

    for path in _rust_source_files(source_root):
        relative = _relative(root, path)
        for line_no, line in enumerate(_read_text(path).splitlines(), start=1):
            if not _line_mentions_rayon(line):
                continue

            references.append(
                {
                    "path": relative,
                    "line": line_no,
                    "snippet": line.strip(),
                    "classification": RAYON_CLASSIFICATIONS.get(relative),
                }
            )
    return references
