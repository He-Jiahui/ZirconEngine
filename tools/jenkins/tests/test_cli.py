from pathlib import Path
import tempfile
import unittest

from tools.jenkins.cli import runtime_path, dispatch, authorization_action, normalized_payload
from tools.jenkins.contracts import JenkinsError
from tools.jenkins.state import State


class CliBoundaryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        (self.repo / ".jenkins/state").mkdir(parents=True)

    def test_response_and_state_paths_cannot_escape_runtime(self):
        self.assertEqual(self.repo / ".jenkins/state/control.json",
                         runtime_path(self.repo, Path(".jenkins/state/control.json")))
        for path in [".jenkins/../outside.json", "outside.json", ".jenkins/cache/state.sqlite3"]:
            with self.assertRaises(JenkinsError):
                runtime_path(self.repo, Path(path), area="state")

    def test_output_cannot_overwrite_configuration_or_state_before_dispatch(self):
        import io
        from types import SimpleNamespace
        from unittest.mock import patch
        from tools.jenkins.cli import main
        for name in (".jenkins/deployment-spec.json", ".jenkins/state/coordination.sqlite3"):
            with self.subTest(name=name):
                protected = self.repo / name
                protected.write_bytes(b"original")
                output = SimpleNamespace(buffer=io.BytesIO())
                with patch("tools.jenkins.cli.dispatch", return_value={"status": "accepted"}) as handler, patch("tools.jenkins.cli.sys.stdout", output):
                    code = main(["--repository", str(self.repo), "--state", ".jenkins/state/test.sqlite3", "request", "query",
                                 "--output-file", str(protected)])
                self.assertEqual(2, code)
                handler.assert_not_called()
                self.assertEqual(b"original", protected.read_bytes())

    def test_corrupt_state_reports_structured_failure_and_releases_file(self):
        import io
        import json
        from types import SimpleNamespace
        from unittest.mock import patch
        from tools.jenkins.cli import main
        path = self.repo / ".jenkins/state/corrupt.sqlite3"
        path.write_bytes(b"corrupt database")
        output = SimpleNamespace(buffer=io.BytesIO())
        with patch("tools.jenkins.cli.sys.stdout", output):
            code = main(["--repository", str(self.repo), "--state", str(path), "request", "query"])
        self.assertEqual(2, code)
        self.assertEqual("invalid_control_input", json.loads(output.buffer.getvalue())["reasonCode"])
        path.unlink()

    def test_response_path_preserves_normal_protocol_output(self):
        import io
        import json
        from types import SimpleNamespace
        from unittest.mock import patch
        from tools.jenkins.cli import main
        path = self.repo / ".jenkins/state/responses/result.json"
        with patch("tools.jenkins.cli.dispatch", return_value={"status": "accepted"}), patch(
                "tools.jenkins.cli.sys.stdout", SimpleNamespace(buffer=io.BytesIO())):
            code = main(["--repository", str(self.repo), "request", "query", "--output-file", str(path)])
        self.assertEqual(0, code)
        self.assertEqual("accepted", json.loads(path.read_bytes())["status"])

    def test_mutation_without_scoped_session_cannot_reach_owner(self):
        state = State(self.repo / ".jenkins/state/test.sqlite3")
        with self.assertRaises(JenkinsError) as failure:
            dispatch("flow", "register-flow", {}, state, self.repo)
        self.assertEqual("action_not_authorized", failure.exception.code)

    def test_directory_link_cannot_redirect_state(self):
        destination = self.repo / "external"
        destination.mkdir()
        link = self.repo / ".jenkins/state/link"
        try:
            link.symlink_to(destination, target_is_directory=True)
        except OSError:
            self.skipTest("The host cannot create a directory symlink")
        with self.assertRaises(JenkinsError):
            runtime_path(self.repo, link / "state.sqlite3", area="state")

    def test_validation_scope_cannot_apply_patches_or_publish_commits(self):
        from tools.jenkins.contracts import digest
        state = State(self.repo / ".jenkins/state/test.sqlite3")
        repo_id = digest(str(self.repo).casefold())
        state.authorize_session(repo_id, "reader", "fixture", ["source"], ["validation"],
                                {"source": "user", "task": "CLI authorization boundary fixture"})
        payload = {"repositoryId": repo_id, "sessionId": "reader", "ownedPaths": ["source/a.rs"]}
        for domain, action in [("candidate", "apply"), ("candidate", "claim"),
                               ("integration", "publish"), ("integration", "push"),
                               ("integration", "commit-flow"), ("integration", "commit_flow"),
                               ("integration", "forward_revert_flow"), ("integration", "compensate"),
                               ("integration", "reconcile_flow"), ("integration", "forward_revert"),
                               ("flow", "source-apply"),
                               ("flow", "gc-maintenance"),
                               ("admission", "activate-policy"),
                               ("artifact", "gc"), ("notification", "deliver")]:
            with self.assertRaises(JenkinsError) as failure:
                dispatch(domain, action, payload, state, self.repo)
            self.assertEqual("action_not_authorized", failure.exception.code)

    def test_compile_validation_and_deployment_have_separate_authority(self):
        self.assertEqual("validation", authorization_action("execution", "run", {}))
        self.assertEqual("deployment", authorization_action("deployment", "start", {}))
        self.assertEqual("deployment", authorization_action("admission", "activate-policy", {}))
        self.assertEqual("commit", authorization_action("stage", "run-stage", {"stage": "git_commit"}))
        self.assertEqual("implementation", authorization_action("patch", "submit", {}))
        self.assertEqual("implementation", authorization_action("flow", "prepare-patch", {}))

    def test_nested_identity_is_available_to_authorization_and_conflicts_are_rejected(self):
        value = normalized_payload({"identity": {"sessionId": "reader", "generation": 7}})
        self.assertEqual("reader", value["sessionId"])
        self.assertEqual(7, value["generation"])
        with self.assertRaises(JenkinsError) as failure:
            normalized_payload({"sessionId": "other", "identity": {"sessionId": "reader"}})
        self.assertEqual("identity_conflict", failure.exception.code)
        from tools.jenkins.contracts import digest
        state = State(self.repo / ".jenkins/state/test.sqlite3")
        repo_id = digest(str(self.repo).casefold())
        state.authorize_session(repo_id, "reader", "fixture", ["source"], ["validation"],
                                {"source": "user", "task": "Nested workflow identity fixture"})
        payload = {"identity": {"sessionId": "reader"}, "ownedPaths": ["source/a.rs"]}
        with self.assertRaises(JenkinsError) as failure:
            dispatch("integration", "commit-flow", payload, state, self.repo)
        self.assertEqual("action_not_authorized", failure.exception.code)


if __name__ == "__main__":
    unittest.main()
