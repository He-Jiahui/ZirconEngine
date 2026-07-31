import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


class Frameworks01RuntimeErrorOwnerBoundaryTests(unittest.TestCase):
    def test_core_error_is_kernel_owned_without_framework_compatibility(self) -> None:
        runtime_error = REPO_ROOT / "zircon_runtime/src/core/runtime/error.rs"
        framework_error = REPO_ROOT / "zircon_runtime/src/core/framework/error.rs"
        runtime_mod = (
            REPO_ROOT / "zircon_runtime/src/core/runtime/mod.rs"
        ).read_text(encoding="utf-8")
        framework_mod = (
            REPO_ROOT / "zircon_runtime/src/core/framework/mod.rs"
        ).read_text(encoding="utf-8")
        core_mod = (REPO_ROOT / "zircon_runtime/src/core/mod.rs").read_text(
            encoding="utf-8"
        )

        self.assertTrue(runtime_error.is_file())
        self.assertFalse(framework_error.exists())
        self.assertIn("pub(super) mod error;", runtime_mod)
        self.assertNotIn("pub use error::", runtime_mod)
        self.assertNotRegex(framework_mod, r"(?m)^\s*pub\s+mod\s+error\s*;")
        self.assertNotIn("framework::error", core_mod)
        self.assertRegex(
            core_mod,
            r"pub use runtime::error::\{CoreError, CoreResult\};",
        )

        module_order = (
            REPO_ROOT
            / "zircon_runtime/src/core/runtime/descriptors/module_order.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "use super::super::error::{CoreError, CoreResult};", module_order
        )

        definitions = []
        stale_consumers = []
        definition_pattern = re.compile(
            r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?enum\s+CoreError\b"
        )
        for root in (
            REPO_ROOT / "zircon_runtime/src",
            REPO_ROOT / "zircon_runtime/tests",
            REPO_ROOT / "zircon_app/src",
            REPO_ROOT / "zircon_app/tests",
            REPO_ROOT / "zircon_editor/src",
            REPO_ROOT / "zircon_editor/tests",
            REPO_ROOT / "zircon_plugins",
        ):
            for path in root.rglob("*.rs"):
                source = path.read_text(encoding="utf-8")
                relative = path.relative_to(REPO_ROOT).as_posix()
                if definition_pattern.search(source):
                    definitions.append(relative)
                if (
                    "core::framework::error" in source
                    or "framework::error::" in source
                    or "core::runtime::CoreError" in source
                    or "core::runtime::CoreResult" in source
                ):
                    stale_consumers.append(relative)

        self.assertEqual(
            ["zircon_runtime/src/core/runtime/error.rs"], definitions
        )
        self.assertEqual([], stale_consumers)


if __name__ == "__main__":
    unittest.main()
