import re
import unittest
from pathlib import Path

from tools.runtime_domain_dependency_audit import _rust_code_view, _rust_use_paths


REPO_ROOT = Path(__file__).resolve().parents[2]
FORBIDDEN_KERNEL_PATHS = (
    ("crate", "core", "runtime"),
    ("crate", "core", "CoreError"),
)
FORBIDDEN_QUALIFIED_PATH = re.compile(
    r"\bcrate\s*::\s*core\s*::\s*(?:runtime|CoreError)\b"
)


def kernel_dependency_lines(source: str) -> list[int]:
    code_view = _rust_code_view(source)
    lines = {
        line
        for path, _alias, line in _rust_use_paths(code_view)
        if any(path[: len(forbidden)] == forbidden for forbidden in FORBIDDEN_KERNEL_PATHS)
    }

    for reference in FORBIDDEN_QUALIFIED_PATH.finditer(code_view):
        lines.add(code_view.count("\n", 0, reference.start()) + 1)

    return sorted(lines)


class Frameworks01ContractsKernelTestBoundaryTests(unittest.TestCase):
    def test_framework_contracts_do_not_import_runtime_kernel_implementations(self) -> None:
        framework_root = REPO_ROOT / "zircon_runtime/src/core/framework"
        violations = []

        for path in framework_root.rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            source_lines = source.splitlines()
            for line_number in kernel_dependency_lines(source):
                violations.append(
                    f"{path.relative_to(REPO_ROOT).as_posix()}:{line_number}: "
                    f"{source_lines[line_number - 1].strip()}"
                )

        self.assertEqual(
            [],
            violations,
            "framework contracts/tests must not depend on concrete runtime kernel owners:\n"
            + "\n".join(violations),
        )

    def test_kernel_dependency_scanner_handles_use_trees_aliases_and_multiline_paths(self) -> None:
        source = """
use crate::core::{
    runtime::TaskPool,
    CoreError as KernelError,
};

type ContractResult = Result<(), crate
    :: core
    :: CoreError>;
"""

        self.assertEqual(kernel_dependency_lines(source), [3, 4, 7])

    def test_kernel_dependency_scanner_ignores_comments_and_literals(self) -> None:
        source = r'''
// use crate::core::CoreError;
const DOC: &str = "crate::core::runtime::TaskPool";
/* crate::core::{runtime::TaskPool, CoreError} */
'''

        self.assertEqual(kernel_dependency_lines(source), [])


if __name__ == "__main__":
    unittest.main()
