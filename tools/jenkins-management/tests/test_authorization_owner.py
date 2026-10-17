"""Bounded owner profiles; user-sealed validation grants retain their legacy contract."""
from copy import deepcopy
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import Mock, patch

ROOT = Path(__file__).absolute().parents[3]
spec = importlib.util.spec_from_file_location("profile_authorization_owner", ROOT / "tools/jenkins-management/authorization_owner.py")
owner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(owner)


class AuthorizationProfileTests(unittest.TestCase):
    def setUp(self):
        self.writer = {"principal": "task-owner", "snapshotDigest": "a" * 64,
                       "actions": ["implementation", "validation"], "paths": ["src/owned.rs"]}
        self.state = Mock()
        self.state.check_authorization.return_value = self.writer
        self.state.get.return_value = None
        self.grant = {"task": "synthetic task", "repositoryId": "repository", "sessionId": "writer",
            "principal": "task-owner", "paths": ["src/owned.rs"], "ownedPaths": ["src/owned.rs"],
            "actions": ["implementation", "validation"], "profile": "managed-worktree-writer-v1",
            "operationId": "issue-writer", "handoff": {"path": str(ROOT / ".jenkins/records/synthetic-handoff.md"),
                "sha256": owner.sha(b"synthetic handoff")},
            "responsePath": str(ROOT / ".jenkins/records/synthetic-response.json")}

    def reader(self):
        return {**self.grant, "profile": "managed-worktree-reader-v1", "sessionId": "reader",
                "actions": ["validation"], "paths": ["src/owned.rs", "dependency/helper.rs"],
                "writer": {"sessionId": "writer", "snapshotDigest": "a" * 64, "ownedPaths": ["src/owned.rs"]}}

    def test_writer_is_exact_narrow_implementation_and_validation(self):
        self.assertEqual(["src/owned.rs"], owner.scope_plan(self.grant, self.state, ROOT))
        for changes in ({"actions": ["implementation"]}, {"actions": ["implementation", "validation", "gc"]},
                        {"paths": ["src/owned.rs", "dependency/helper.rs"]},
                        {"predecessor": {"sessionId": "old"}}, {"addedPaths": ["dependency/helper.rs"]}):
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                owner.scope_plan({**self.grant, **changes}, self.state, ROOT)
        self.state.check_authorization.assert_not_called()

    def test_reader_requires_separate_same_principal_active_exact_writer(self):
        grant = self.reader()
        self.assertEqual(grant["paths"], owner.scope_plan(grant, self.state, ROOT))
        self.assertEqual(["implementation", "validation"],
                         [call.args[2] for call in self.state.check_authorization.call_args_list])
        for change in ({"principal": "foreign"}, {"paths": ["src/owned.rs", "dependency"]},
                       {"snapshotDigest": "b" * 64}, {"actions": ["implementation", "validation", "push"]}):
            self.state.check_authorization.return_value = {**self.writer, **change}
            with self.subTest(change=change), self.assertRaises(ValueError):
                owner.scope_plan(grant, self.state, ROOT)
        self.state.check_authorization.return_value = self.writer
        for change in ({"actions": ["implementation", "validation"]}, {"sessionId": "writer"},
                       {"writer": {**grant["writer"], "snapshotDigest": "b" * 64}}):
            with self.subTest(change=change), self.assertRaises(ValueError):
                owner.scope_plan({**grant, **change}, self.state, ROOT)

    def test_legacy_validation_profile_and_predecessor_are_unchanged(self):
        grant = {**self.grant, "actions": ["validation"]}
        grant.pop("profile")
        grant.pop("ownedPaths")
        self.assertEqual(grant["paths"], owner.scope_plan(grant, self.state, ROOT))
        old = {"principal": "task-owner", "snapshotDigest": "a" * 64, "paths": ["src/old.rs"]}
        self.state.check_authorization.return_value = old
        grant.update(paths=["src/old.rs", "src/new.rs"], addedPaths=["src/new.rs"],
                     predecessor={"sessionId": "previous", "snapshotDigest": "a" * 64, "paths": ["src/old.rs"]})
        self.assertEqual(grant["paths"], owner.scope_plan(grant, self.state, ROOT))

    def test_issuance_and_check_only_use_approved_actions(self):
        config = {"grants": {"writer": self.grant}, "rootThreadId": "root",
                  "userGrant": {"path": "synthetic", "sha256": "c" * 64},
                  "driverDigest": "d" * 64, "runtimeOperationId": "runtime"}
        user = {"objectiveSha256": "e" * 64}
        self.state.authorize_session.return_value = {"snapshotDigest": "f" * 64, "status": "active"}
        with patch.object(Path, "read_bytes", return_value=b"synthetic handoff"), patch.object(owner, "write_json"):
            checked = owner.execute(config, user, ROOT, self.state, "writer", check_only=True)
            self.assertEqual(["implementation", "validation"], checked["actions"])
            self.state.authorize_session.assert_not_called()
            issued = owner.execute(config, user, ROOT, self.state, "writer")
        self.assertEqual(["implementation", "validation"], issued["actions"])
        call = self.state.authorize_session.call_args
        self.assertEqual(["src/owned.rs"], call.args[3])
        self.assertEqual(["implementation", "validation"], call.args[4])
        self.assertEqual("managed-worktree-writer-v1", call.args[5]["authorizationProfile"])


if __name__ == "__main__":
    unittest.main()
