from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.runtime_domain_dependency_audit import audit_runtime_domain_dependencies


class RuntimeDomainDependencyAuditTests(unittest.TestCase):
    def test_reports_unique_production_cross_domain_references(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            source_root = repo_root / "zircon_runtime" / "src"
            (source_root / "graphics").mkdir(parents=True)
            (source_root / "ui" / "tests").mkdir(parents=True)
            (source_root / "graphics" / "render.rs").write_text(
                "use crate::ui::TextLayout;\n"
                "fn draw() { let _ = crate::scene::SceneHandle::default(); }\n",
                encoding="utf-8",
            )
            (source_root / "graphics" / "self_ref.rs").write_text(
                "use crate::graphics::Renderer;\n",
                encoding="utf-8",
            )
            (source_root / "ui" / "tests" / "ignored.rs").write_text(
                "use crate::graphics::Renderer;\n",
                encoding="utf-8",
            )

            report = audit_runtime_domain_dependencies(repo_root)

            self.assertEqual(report["production_reference_count"], 2)
            self.assertEqual(report["domain_edge_count"], 2)
            self.assertEqual(
                report["matrix"],
                [
                    {
                        "source_domain": "graphics",
                        "target_domain": "scene",
                        "reference_count": 1,
                    },
                    {
                        "source_domain": "graphics",
                        "target_domain": "ui",
                        "reference_count": 1,
                    },
                ],
            )

    def test_ignores_root_files_and_test_owners(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            source_root = repo_root / "zircon_runtime" / "src"
            (source_root / "ui").mkdir(parents=True)
            (source_root / "lib.rs").write_text(
                "pub use crate::graphics::Renderer;\n", encoding="utf-8"
            )
            (source_root / "ui" / "tests.rs").write_text(
                "use crate::graphics::Renderer;\n", encoding="utf-8"
            )
            (source_root / "ui" / "layout_tests.rs").write_text(
                "use crate::graphics::Renderer;\n", encoding="utf-8"
            )
            (source_root / "ui" / "test_layout.rs").write_text(
                "use crate::graphics::Renderer;\n", encoding="utf-8"
            )

            report = audit_runtime_domain_dependencies(repo_root)

            self.assertEqual(report["production_reference_count"], 0)
            self.assertEqual(report["matrix"], [])


if __name__ == "__main__":
    unittest.main()
