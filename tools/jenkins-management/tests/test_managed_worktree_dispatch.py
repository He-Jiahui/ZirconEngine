"""Exercise the production managed-worktree dispatch with its packaged helper."""
from __future__ import annotations

from contextlib import nullcontext
import importlib.util
from pathlib import Path
from types import ModuleType, SimpleNamespace
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location(
    "managed_worktree_dispatch_test_runner", ROOT / "tools/jenkins-management/runner.py"
)
runner = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runner)


class ManagedWorktreeDispatchTests(unittest.TestCase):
    def check_dispatch(self, action, operation):
        actor = runner.Management.__new__(runner.Management)
        actor.action = action
        expected = {"status": "delegated"}
        helper = runner.source_helper()
        with patch.object(runner, "source_helper", return_value=helper):
            with patch.object(helper, operation, return_value=expected) as delegated:
                self.assertIs(actor._dispatch(), expected)
        delegated.assert_called_once()
        args, kwargs = delegated.call_args
        self.assertEqual(args, (actor,))
        self.assertIs(kwargs["preparation_helper"], helper)
        self.assertIsInstance(helper, ModuleType)
        self.assertEqual(
            Path(helper.__file__).resolve(),
            (ROOT / "tools/jenkins-management/source_preparation.py").resolve(),
        )
        self.assertTrue(callable(helper.seal_source))
        self.assertTrue(callable(helper.verify_source))

    def test_inspect_dispatch_loads_packaged_preparation_helper(self):
        self.check_dispatch("inspect-worktree", "managed_inspect_worktree")

    def test_prepare_dispatch_loads_packaged_preparation_helper(self):
        self.check_dispatch("prepare-worktree", "managed_prepare_worktree")



class OrdinaryManagementContextTests(unittest.TestCase):
    def setUp(self):
        self.repo = ROOT
        self.config = {"driverDigest": "d" * 64, "runtimeOperationId": "start-local"}
        self.payload = {"action": "validate", "repositoryId": "r" * 32,
                        "sessionId": "local-chat", "requestId": "local-request",
                        "operationId": "local-operation", "ownedPaths": ["src/lib.rs"],
                        **self.config}

    def test_ordinary_validation_uses_verified_principal_and_required_action(self):
        state, manager, result = object(), object(), {"snapshotDigest": "s" * 64}
        helper = SimpleNamespace(local_request_principal=lambda value: "owner-sid",
                                 admit_ordinary_context=lambda *args, **kwargs: result)
        with patch.object(helper, "local_request_principal", return_value="owner-sid") as principal:
            with patch.object(helper, "admit_ordinary_context", return_value=result) as admit:
                with patch.object(runner, "source_helper", return_value=helper):
                    actual = runner.ensure_request_context(
                        self.config, self.repo, manager, self.payload, state=state)
        self.assertIs(actual, result)
        principal.assert_called_once_with(manager)
        admit.assert_called_once_with(state, self.repo, self.payload, principal="owner-sid",
            driver_digest=self.config["driverDigest"], runtime_operation_id=self.config["runtimeOperationId"],
            actions=["validation"], record_request=True)

    def test_ordinary_actions_use_the_same_bounded_context_with_required_action(self):
        helper = SimpleNamespace(local_request_principal=lambda value: "owner-sid",
                                 admit_ordinary_context=lambda *args, **kwargs: {"snapshotDigest": "s" * 64})
        with patch.object(helper, "admit_ordinary_context", return_value={"snapshotDigest": "s" * 64}) as admit:
            with patch.object(runner, "source_helper", return_value=helper):
                for action in ("create-branch", "create-worktree", "prepare-merge", "apply-patch", "generate",
                               "inspect-worktree"):
                    with self.subTest(action=action):
                        payload = {**self.payload, "action": action}
                        runner.ensure_request_context(self.config, self.repo, object(), payload, state=object())
                        self.assertIs(admit.call_args.args[2], payload)
                        expected = "validation" if action == "inspect-worktree" else "implementation"
                        self.assertEqual([expected], admit.call_args.kwargs["actions"])
        self.assertEqual(admit.call_count, 6)

    def test_observations_and_managed_preparation_keep_their_existing_context(self):
        with patch.object(runner, "source_helper") as helper:
            for extra in ({"observationOnly": True}, {"worktreePreparationRef": "p" * 64},
                          {"action": "prepare-worktree"}, {"action": "prepare-flow"},
                          {"action": "submit-flow"}, {"action": "retire-worktree-reader"},
                          {"action": "commit-worktree"}, {"action": "release-worktree"},
                          {"action": "recover-worktree"}):
                with self.subTest(extra=extra):
                    self.assertIsNone(runner.ensure_request_context(
                        self.config, self.repo, object(), {**self.payload, **extra}, state=object()))
        helper.assert_not_called()

    def test_constructor_admits_before_existing_scoped_check(self):
        events = []
        state = SimpleNamespace(
            connect=lambda: nullcontext(object()),
            check_authorization=lambda *args, **kwargs:
                events.append("check") or {"paths": self.payload["ownedPaths"]})
        with patch("tools.jenkins.state.State", return_value=state), \
                patch("tools.jenkins.source.repository_id", return_value=self.payload["repositoryId"]):
            with patch.object(runner, "plain", side_effect=lambda path: path):
                with patch.object(runner, "ensure_request_context",
                                  side_effect=lambda *args, **kwargs: events.append("admit")) as admit:
                    actor = runner.Management(self.config, self.repo, object(), self.payload)
        self.assertEqual(events, ["admit", "check"])
        self.assertIs(admit.call_args.kwargs["state"], actor.state)
        self.assertIs(actor.state._authority, state)
        self.assertIs(actor.payload, self.payload)

if __name__ == "__main__":
    unittest.main()
