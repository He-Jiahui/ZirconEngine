from __future__ import annotations

import os
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

from tools.check_conventions import _RepositoryPathValidator, audit_document_paths


class DocumentPathAuditTests(unittest.TestCase):
    def test_reports_missing_related_code_and_implementation_files(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            docs_root = repo_root / "docs"
            docs_root.mkdir()
            (repo_root / "src").mkdir()
            (repo_root / "src" / "present.rs").write_text("", encoding="utf-8")
            (docs_root / "module.md").write_text(
                "---\n"
                "related_code:\n"
                "  - src/present.rs\n"
                "  - src/missing.rs\n"
                "implementation_files:\n"
                "  - src/missing_impl.rs\n"
                "tests:\n"
                "  - cargo test -p example\n"
                "---\n\n"
                "# Module\n",
                encoding="utf-8",
            )

            report = audit_document_paths(repo_root)

            self.assertEqual(report["document_count"], 1)
            self.assertEqual(report["checked_path_count"], 3)
            self.assertEqual(report["affected_document_count"], 1)
            self.assertEqual(report["reason_counts"], {"missing path": 2})
            self.assertEqual(report["path_root_counts"], {"src": 2})
            self.assertEqual(
                [(item["field"], item["path"]) for item in report["violations"]],
                [
                    ("implementation_files", "src/missing_impl.rs"),
                    ("related_code", "src/missing.rs"),
                ],
            )

    def test_accepts_existing_files_and_directories(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            docs_root = repo_root / "docs"
            docs_root.mkdir()
            (repo_root / "src" / "feature").mkdir(parents=True)
            (repo_root / "src" / "feature" / "mod.rs").write_text(
                "", encoding="utf-8"
            )
            (docs_root / "module.md").write_text(
                "---\n"
                "related_code:\n"
                "  - src/feature\n"
                "  - src/feature/mod.rs\n"
                "---\n\n"
                "# Module\n",
                encoding="utf-8",
            )

            report = audit_document_paths(repo_root)

            self.assertEqual(report["checked_path_count"], 2)
            self.assertEqual(report["violations"], [])

    def test_checks_repository_paths_declared_by_tests(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            docs_root = repo_root / "docs"
            docs_root.mkdir()
            (repo_root / "tools" / "tests").mkdir(parents=True)
            (repo_root / "tools" / "tests" / "present.py").write_text(
                "", encoding="utf-8"
            )
            (repo_root / "tools" / "check_conventions.py").write_text(
                "", encoding="utf-8"
            )
            (repo_root / "Cargo.toml").write_text("", encoding="utf-8")
            (docs_root / "module.md").write_text(
                "---\n"
                "tests:\n"
                "  - tools/tests/present.py::test_present\n"
                "  - tools/tests/present.py:12:3\n"
                "  - tools/tests/missing.py::test_missing\n"
                "  - .opencode/workflows/missing.md\n"
                "  - Cargo.toml\n"
                "  - tools/tests/*.py\n"
                "  - <owner>/status.rs\n"
                "  - target/generated/report.json\n"
                "  - build/generated/report.json\n"
                "  - zircon_runtime/target/debug/report.json\n"
                "  - zircon_runtime/build/generated/report.json\n"
                "  - https://example.com/tests/guide.rs\n"
                "  - http://example.com/tests/reference.md\n"
                "  - python tools/check_conventions.py --only docs\n"
                "  - cargo test --manifest-path tools/tests/missing.py\n"
                "  - git diff --check -- tools/tests/missing.py\n"
                "  - python -m unittest tools.tests.test_check_conventions -v\n"
                "  - cargo test -p zircon_runtime structure_convention\n"
                "---\n\n"
                "# Module\n",
                encoding="utf-8",
            )

            report = audit_document_paths(repo_root)

            self.assertEqual(report["checked_path_count"], 5)
            self.assertEqual(
                [(item["field"], item["path"]) for item in report["violations"]],
                [
                    ("tests", ".opencode/workflows/missing.md"),
                    ("tests", "tools/tests/missing.py"),
                ],
            )

    def test_rejects_unsafe_paths_declared_by_tests(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            docs_root = repo_root / "docs"
            docs_root.mkdir()
            (docs_root / "module.md").write_text(
                "---\n"
                "tests:\n"
                "  - ../tools/tests/missing.py\n"
                "  - C:/tools/tests/missing.py\n"
                "  - C://tools/tests/missing.py\n"
                "  - C:\\tools\\tests\\missing.py\n"
                "  - /tools/tests/missing.py\n"
                "  - ../outside folder/test.rs\n"
                "  - C:/Program Files/Zircon/test.rs\n"
                "---\n\n"
                "# Module\n",
                encoding="utf-8",
            )

            report = audit_document_paths(repo_root)

            self.assertEqual(report["checked_path_count"], 7)
            self.assertEqual(
                report["reason_counts"],
                {"absolute path": 5, "repository escape": 2},
            )

    def test_rejects_absolute_and_parent_escape_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            docs_root = repo_root / "docs"
            docs_root.mkdir()
            (docs_root / "module.md").write_text(
                "---\n"
                "related_code:\n"
                "  - ../outside.rs\n"
                "  - C:/outside.rs\n"
                "---\n\n"
                "# Module\n",
                encoding="utf-8",
            )

            report = audit_document_paths(repo_root)

            self.assertEqual(
                [item["reason"] for item in report["violations"]],
                ["repository escape", "absolute path"],
            )

    def test_reuses_parent_resolution_for_same_directory_siblings(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            docs_root = repo_root / "docs"
            source_root = repo_root / "src" / "feature"
            docs_root.mkdir()
            source_root.mkdir(parents=True)
            paths = [f"src/feature/item_{index}.rs" for index in range(128)]
            for path in paths:
                (repo_root / path).write_text("", encoding="utf-8")
            (docs_root / "module.md").write_text(
                "---\nrelated_code:\n"
                + "".join(f"  - {path}\n" for path in paths)
                + "---\n\n# Module\n",
                encoding="utf-8",
            )

            report = audit_document_paths(repo_root)

            self.assertEqual(report["violations"], [])
            self.assertEqual(
                report["resolution_metrics"],
                {
                    "unique_path_count": 128,
                    "full_resolution_count": 1,
                    "parent_resolution_count": 1,
                    "reparse_leaf_resolution_count": 0,
                    "relative_segment_resolution_count": 0,
                },
            )

    def test_full_resolves_relative_segments_before_classifying_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            docs_root = repo_root / "docs"
            source_root = repo_root / "src" / "feature"
            docs_root.mkdir()
            source_root.mkdir(parents=True)
            (source_root / "present.rs").write_text("", encoding="utf-8")
            (docs_root / "module.md").write_text(
                "---\n"
                "related_code:\n"
                "  - src/feature/../feature/present.rs\n"
                "  - ../outside.rs\n"
                "---\n\n"
                "# Module\n",
                encoding="utf-8",
            )

            report = audit_document_paths(repo_root)

            self.assertEqual(
                [(item["path"], item["reason"]) for item in report["violations"]],
                [("../outside.rs", "repository escape")],
            )
            self.assertEqual(
                report["resolution_metrics"],
                {
                    "unique_path_count": 2,
                    "full_resolution_count": 2,
                    "parent_resolution_count": 0,
                    "reparse_leaf_resolution_count": 0,
                    "relative_segment_resolution_count": 2,
                },
            )

    def test_rejects_reparse_leaf_and_parent_escapes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            repo_root = temporary_root / "repo"
            outside_root = temporary_root / "outside"
            docs_root = repo_root / "docs"
            docs_root.mkdir(parents=True)
            outside_root.mkdir()
            (outside_root / "secret.rs").write_text("", encoding="utf-8")
            self._create_directory_link(repo_root / "linked", outside_root)
            (docs_root / "module.md").write_text(
                "---\n"
                "related_code:\n"
                "  - linked\n"
                "  - linked/secret.rs\n"
                "---\n\n"
                "# Module\n",
                encoding="utf-8",
            )

            report = audit_document_paths(repo_root)

            self.assertEqual(
                [(item["path"], item["reason"]) for item in report["violations"]],
                [
                    ("linked", "repository escape"),
                    ("linked/secret.rs", "repository escape"),
                ],
            )
            self.assertEqual(
                report["resolution_metrics"],
                {
                    "unique_path_count": 2,
                    "full_resolution_count": 2,
                    "parent_resolution_count": 1,
                    "reparse_leaf_resolution_count": 1,
                    "relative_segment_resolution_count": 0,
                },
            )

    def test_detects_windows_reparse_attribute_without_path_is_junction(self) -> None:
        class WindowsReparseFixture:
            @staticmethod
            def is_symlink() -> bool:
                return False

            @staticmethod
            def lstat() -> SimpleNamespace:
                return SimpleNamespace(
                    st_mode=stat.S_IFDIR,
                    st_file_attributes=stat.FILE_ATTRIBUTE_REPARSE_POINT
                )

        self.assertTrue(
            _RepositoryPathValidator._is_reparse_leaf(WindowsReparseFixture())
        )

    @staticmethod
    def _create_directory_link(link: Path, target: Path) -> None:
        if os.name == "nt":
            subprocess.run(
                ("cmd.exe", "/d", "/c", "mklink", "/J", str(link), str(target)),
                check=True,
                capture_output=True,
                text=True,
            )
            return
        link.symlink_to(target, target_is_directory=True)
