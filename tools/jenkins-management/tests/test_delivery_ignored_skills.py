"""Regression for explicitly selected canonical skills under an ignored root."""
import importlib.util
from pathlib import Path
import sys
import unittest

spec = importlib.util.spec_from_file_location("delivery_ignored_support", Path(__file__).with_name("test_delivery.py"))
support = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = support
spec.loader.exec_module(support)


class IgnoredSkillDeliveryTests(unittest.TestCase):
    def test_commit_selected_canonical_skill_and_preserve_ignored_state(self):
        case = support.DeliveryTests()
        with case.fixture() as fixture:
            repo, policy, grants, payload, validate, _ = fixture
            (repo / ".gitignore").write_text(".jenkins/\n.codex/\n")
            skill = ".codex/skills/task-skill/SKILL.md"
            path = repo / skill
            path.parent.mkdir(parents=True)
            path.write_text("---\nname: task-skill\ndescription: Task skill\n---\n")
            state = repo / ".codex/state/foreign.json"
            state.parent.mkdir(parents=True)
            state.write_text('{"owner":"another-session"}')
            payload["paths"].append(skill)
            result = support.delivery.execute(repo, payload, policy=policy, grants=grants, validate=validate)
            self.assertEqual(result["status"], "complete", (result.get("reason"), result.get("validatedGitObjects"), result.get("committedGitObjects")))
            self.assertIn(b"name: task-skill", support.git(repo, "show", "HEAD:" + skill).stdout)
            self.assertEqual(state.read_text(), '{"owner":"another-session"}')
            self.assertEqual(support.git(repo, "ls-files", "--", ".codex/state/foreign.json").stdout, b"")
            self.assertEqual(support.git(repo, "show", "HEAD:.gitignore").stdout, b".jenkins/\n")


if __name__ == "__main__":
    unittest.main(verbosity=2)
