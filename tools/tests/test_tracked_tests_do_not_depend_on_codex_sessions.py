import unittest
from pathlib import Path


class TrackedTestSessionDependencyTests(unittest.TestCase):
    def test_tracked_tests_do_not_read_ephemeral_codex_sessions(self) -> None:
        tests_root = Path(__file__).resolve().parent
        forbidden_fragments = (
            ".codex" + "/sessions/",
            ".codex" + "\\sessions\\",
        )
        violations: list[str] = []

        for path in tests_root.rglob("*"):
            if path.suffix.lower() not in {".py", ".ps1"}:
                continue
            text = path.read_text(encoding="utf-8", errors="replace")
            for line_number, line in enumerate(text.splitlines(), start=1):
                if any(fragment in line for fragment in forbidden_fragments):
                    relative = path.relative_to(tests_root.parent.parent).as_posix()
                    violations.append(f"{relative}:{line_number}")

        self.assertEqual([], violations)


if __name__ == "__main__":
    unittest.main()
