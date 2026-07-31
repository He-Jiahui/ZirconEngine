import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


class Frameworks01ContractsKernelTestBoundaryTests(unittest.TestCase):
    def test_framework_contracts_do_not_import_runtime_kernel_implementations(self) -> None:
        framework_root = REPO_ROOT / "zircon_runtime/src/core/framework"
        violations = []

        for path in framework_root.rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            for line_number, line in enumerate(source.splitlines(), start=1):
                if "crate::core::runtime" in line:
                    violations.append(
                        f"{path.relative_to(REPO_ROOT).as_posix()}:{line_number}: {line.strip()}"
                    )

        self.assertEqual(
            [],
            violations,
            "framework contracts/tests must not depend on concrete runtime kernel owners:\n"
            + "\n".join(violations),
        )


if __name__ == "__main__":
    unittest.main()
