import subprocess
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory


def _contains_ephemeral_session_child_path(line: str) -> bool:
    normalized = line.replace("\\", "/").casefold()
    return (".codex" + "/sessions/") in normalized


def _repository_test_paths(repo_root: Path, tests_root: Path) -> tuple[Path, ...]:
    if not (repo_root / ".git").exists():
        # Managed validation copies contain only materialized tracked inputs and owned overlays.
        candidates = tests_root.rglob("*")
    else:
        completed = subprocess.run(
            ["git", "ls-files", "--", "tools/tests"],
            cwd=repo_root,
            check=True,
            capture_output=True,
            text=True,
        )
        candidates = (repo_root / relative for relative in completed.stdout.splitlines())
    return tuple(
        path
        for path in candidates
        if path.is_file() and path.suffix.lower() in {".py", ".ps1"}
    )


_TESTS_ROOT = Path(__file__).resolve().parent
_REPO_ROOT = _TESTS_ROOT.parent.parent
_REPOSITORY_TEST_PATHS = _repository_test_paths(_REPO_ROOT, _TESTS_ROOT)


class TrackedTestSessionDependencyTests(unittest.TestCase):
    def test_materialized_copy_nested_under_repository_ignores_ancestor_git(self) -> None:
        with TemporaryDirectory(dir=_TESTS_ROOT) as temporary_directory:
            repo_root = Path(temporary_directory)
            tests_root = repo_root / "tools" / "tests"
            tests_root.mkdir(parents=True)
            materialized_test = tests_root / "test_materialized.py"
            materialized_test.write_text("VALUE = 1\n", encoding="utf-8")

            self.assertEqual(
                (materialized_test,),
                _repository_test_paths(repo_root, tests_root),
            )

    def test_materialized_copy_snapshot_excludes_later_test_files(self) -> None:
        with TemporaryDirectory() as temporary_directory:
            repo_root = Path(temporary_directory)
            tests_root = repo_root / "tools" / "tests"
            tests_root.mkdir(parents=True)
            materialized_test = tests_root / "test_materialized.py"
            materialized_test.write_text("VALUE = 1\n", encoding="utf-8")

            snapshot = _repository_test_paths(repo_root, tests_root)
            (tests_root / "test_created_later.py").write_text("VALUE = 2\n", encoding="utf-8")

            self.assertEqual((materialized_test,), snapshot)

    def test_session_child_path_match_normalizes_case_and_separators(self) -> None:
        reference = ".CoDeX" + "\\SeSsIoNs\\private\\bootstrap.ps1"

        self.assertTrue(_contains_ephemeral_session_child_path(reference))

    def test_untracked_test_files_are_outside_the_repository_guard(self) -> None:
        tests_root = Path(__file__).resolve().parent
        with TemporaryDirectory(dir=tests_root) as temporary_directory:
            untracked_test = Path(temporary_directory) / "test_untracked_session_input.py"
            untracked_test.write_text(
                "SESSION_INPUT = '" + ".codex" + "/sessions/private/bootstrap.ps1'\n",
                encoding="utf-8",
            )

            self.test_tracked_tests_do_not_read_ephemeral_codex_sessions()

    def test_tracked_tests_do_not_read_ephemeral_codex_sessions(self) -> None:
        violations: list[str] = []

        for path in _REPOSITORY_TEST_PATHS:
            text = path.read_text(encoding="utf-8", errors="replace")
            for line_number, line in enumerate(text.splitlines(), start=1):
                if _contains_ephemeral_session_child_path(line):
                    violations.append(f"{path.relative_to(_REPO_ROOT).as_posix()}:{line_number}")

        self.assertEqual([], violations)


if __name__ == "__main__":
    unittest.main()
