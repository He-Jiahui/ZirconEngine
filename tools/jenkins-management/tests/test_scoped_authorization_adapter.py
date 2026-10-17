"""Scoped management checks preserve State authority without quadratic reader scans."""
from __future__ import annotations

from contextlib import contextmanager
import importlib.util
import json
from pathlib import Path
import sqlite3
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.source import repository_id
from tools.jenkins.state import SCHEMA, State
from tools.jenkins import managed_worktree as formal

ROOT = Path(__file__).resolve().parents[3]


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / "tools/jenkins-management" / filename)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


helper = load("scoped_authorization_helper_tests", "source_preparation.py")
runner = load("scoped_authorization_runner_tests", "runner.py")


class ScopedAuthorizationAdapterTests(unittest.TestCase):
    def setUp(self):
        self.database = sqlite3.connect(":memory:", isolation_level=None)
        self.database.row_factory = sqlite3.Row
        self.database.executescript(SCHEMA)
        self.addCleanup(self.database.close)
        self.state = object.__new__(State)
        self.state.path = ROOT / ".jenkins/state/coordination.sqlite3"
        self.state.busy_timeout_ms = 3000

        @contextmanager
        def connected():
            yield self.database

        self.state.connect = connected
        self.repository = repository_id(ROOT)

    def authorize(self, session, paths, actions=("validation",)):
        return self.state.authorize_session(self.repository, session, "legacy-owner",
            paths, list(actions), {"source": "user", "task": "scoped adapter fixture"})

    def observed(self, state, paths):
        try:
            return state.check_authorization(self.repository, "scope", "validation", paths)
        except JenkinsError as error:
            return error.code, str(error)

    def test_exact_ancestor_prefix_and_invalid_paths_match_driver(self):
        self.authorize("scope", ["src", "exact/file.rs", "external/zr_vm/lib.rs"])
        original_method = State.check_authorization
        adapted = helper.scoped_authorization_state(self.state)
        cases = [None, [], ["src"], ["SRC\\nested\\lib.rs/"],
            ["exact/file.rs"], ["exact/file.rs/child"], ["external/zr_vm/lib.rs"],
            ["src-other/lib.rs"], ["exact/file.rs2"], ["external/zr_vm/lib.rs2"],
            [None], [1], [""], ["/src/lib.rs"], ["\\src\\lib.rs"],
            ["src//lib.rs"], ["src/../lib.rs"], ["C:/src/lib.rs"]]
        for paths in cases:
            with self.subTest(paths=paths):
                self.assertEqual(self.observed(self.state, paths), self.observed(adapted, paths))
        self.assertIs(adapted, helper.scoped_authorization_state(adapted))
        self.assertIs(original_method, State.check_authorization)
        self.assertIsNone(self.state.__dict__.get("check_authorization"))

    def test_live_authority_keeps_narrow_scope_actions_and_revocation(self):
        before = self.authorize("scope", ["src/lib.rs"])
        adapted = helper.scoped_authorization_state(self.state)
        self.assertEqual(before["snapshotDigest"],
            adapted.check_authorization(self.repository, "scope", "validation", ["src/lib.rs"])["snapshotDigest"])
        for action, paths, code in [("implementation", ["src/lib.rs"], "action_not_authorized"),
                                    ("validation", ["src/foreign.rs"], "path_not_authorized")]:
            with self.subTest(action=action), self.assertRaises(JenkinsError) as observed:
                adapted.check_authorization(self.repository, "scope", action, paths)
            self.assertEqual(code, observed.exception.code)
        row = self.database.execute("SELECT * FROM authorizations WHERE session_id='scope'").fetchone()
        self.assertEqual(["src/lib.rs"], json.loads(row["paths"]))
        self.assertEqual(["validation"], json.loads(row["actions"]))
        self.assertEqual(before["snapshotDigest"], row["snapshot_digest"])
        self.database.execute("UPDATE authorizations SET status='revoked' WHERE session_id='scope'")
        with self.assertRaises(JenkinsError) as observed:
            adapted.check_authorization(self.repository, "scope", "validation", ["src/lib.rs"])
        self.assertEqual("action_not_authorized", observed.exception.code)

    def test_large_reader_admission_management_and_formal_checks_never_rescan_scopes(self):
        paths = [f"src/p{number:05d}.rs" for number in range(70000)]
        owned = paths[:2]
        writer = self.authorize("writer", owned, ("implementation", "validation"))
        evidence = {"source": "automatic-local-request", "policy": "local-validation-v1",
            "kind": "managed-worktree-reader", "repositoryId": self.repository,
            "repositoryRoot": str(ROOT), "ownerPrincipal": "windows:S-1-5-21-123-1001",
            "writerPrincipal": "legacy-owner", "inputDigest": "e" * 64}
        original = self.state.check_authorization

        def authority(*args, **kwargs):
            self.assertIsNone(kwargs.get("paths", args[3] if len(args) > 3 else None),
                              "Full reader paths reached the quadratic driver matcher")
            self.assertIs(self.database, kwargs.get("connection"))
            return original(*args, **kwargs)

        with patch.object(self.state, "check_authorization", side_effect=authority) as checked:
            reader = helper.ensure_local_context(self.state, ROOT, "reader", principal="legacy-owner",
                paths=paths, actions=["validation"], evidence=evidence)
            self.assertEqual(1, checked.call_count)
            self.assertEqual(paths, reader["paths"])
            checked.reset_mock()
            config = {"driverDigest": "d" * 64, "runtimeOperationId": "start-test",
                      "managedWorktree": runner.MANAGED_WORKTREE_CAPABILITY}
            payload = {"action": "prepare-worktree", "repositoryId": self.repository,
                "sessionId": "reader", "requestId": "prepare", "operationId": "prepare-1",
                "ownedPaths": owned, **{key: config[key] for key in ("driverDigest", "runtimeOperationId")}}
            with patch("tools.jenkins.state.State", return_value=self.state), \
                    patch.object(runner, "source_helper", return_value=helper), \
                    patch.object(runner, "ensure_request_context"), \
                    patch.object(runner, "plain", side_effect=lambda value: value):
                management = runner.Management(config, ROOT, None, payload)
            self.assertIsInstance(management.state, helper._ScopedAuthorizationState)
            self.assertEqual(1, checked.call_count)
            record = {"sessionId": "writer", "ownedPaths": owned, "writerPrincipal": writer["principal"],
                      "writerAuthorizationSnapshot": writer["snapshotDigest"]}
            result = formal._reader(management, record, dict.fromkeys(paths, "a" * 64), [])
            self.assertEqual(reader, result)
            self.assertEqual(4, checked.call_count)
            self.assertTrue(all(call.kwargs.get("paths") is None for call in checked.call_args_list))
            self.assertEqual(owned, json.loads(self.database.execute(
                "SELECT paths FROM authorizations WHERE session_id='writer'").fetchone()[0]))


if __name__ == "__main__":
    unittest.main()
