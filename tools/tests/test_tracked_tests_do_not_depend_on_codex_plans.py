import subprocess
import unittest
from pathlib import Path


class TrackedTestPlanDependencyTests(unittest.TestCase):
    def test_tracked_tools_do_not_read_personal_codex_plans(self) -> None:
        tools_root = Path(__file__).resolve().parents[1]
        forbidden_fragments = (
            ".codex" + "/plans/",
            ".codex" + "\\plans\\",
        )
        violations: list[str] = []

        completed = subprocess.run(
            ["git", "ls-files", "--", "tools"],
            cwd=tools_root.parent,
            check=True,
            capture_output=True,
            text=True,
        )
        for relative in completed.stdout.splitlines():
            path = tools_root.parent / relative
            if not path.is_file():
                continue
            relative_to_tools = path.relative_to(tools_root).as_posix()
            if relative_to_tools.startswith("session_coordinator/tests/"):
                continue
            if path.suffix.lower() not in {".py", ".ps1", ".js", ".mjs", ".ts"}:
                continue
            text = path.read_text(encoding="utf-8", errors="replace")
            for line_number, line in enumerate(text.splitlines(), start=1):
                if any(fragment in line for fragment in forbidden_fragments):
                    relative = path.relative_to(tools_root.parent).as_posix()
                    violations.append(f"{relative}:{line_number}")

        self.assertEqual([], violations)


if __name__ == "__main__":
    unittest.main()
