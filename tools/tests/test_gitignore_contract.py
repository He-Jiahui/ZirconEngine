"""Source files must remain visible to ordinary Git status/add operations."""
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]


class GitignoreContractTests(unittest.TestCase):
    def test_tracked_files_are_not_ignored(self):
        paths = subprocess.check_output(["git", "ls-files", "-ci", "--exclude-standard", "-z"], cwd=ROOT)
        self.assertEqual(set(paths.decode().split("\0")) - {""}, set())

    def test_generated_stage_and_shader_caches_are_not_tracked(self):
        paths = subprocess.check_output(["git", "ls-files", "-z"], cwd=ROOT).decode().split("\0")
        self.assertEqual([p for p in paths if p.startswith("stage/") or "/.zircon-cache/" in p], [])

    def test_new_sources_are_visible(self):
        for path in ("tools/tests/new_test.py", ".codex/skills/new/SKILL.md", "examples/woc/new.rs", "examples/vampire/new.txt", "third_party/new/CMakeLists.txt", "docs/layout/new.zui"):
            result = subprocess.run(["git", "check-ignore", "--no-index", "--", path], cwd=ROOT, capture_output=True)
            self.assertEqual(result.returncode, 1, (path, result.stdout))

    def test_runtime_state_stays_ignored(self):
        for path in (".codex/state/new.json", ".codex/outbox/new.json", ".jenkins/state/new.json"):
            self.assertEqual(subprocess.run(["git", "check-ignore", "--no-index", "--", path], cwd=ROOT, capture_output=True).returncode, 0, path)
