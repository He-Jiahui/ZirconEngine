"""Real Git regression tests for the Jenkins scoped checkout commit route."""
from __future__ import annotations

from contextlib import contextmanager
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("scoped_checkout_commit", Path(__file__).parents[1] / "checkout_commit.py")
commit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(commit)


class CheckoutCommitTests(unittest.TestCase):
    @contextmanager
    def fixture(self):
        # Jenkins provides this task-owned fixture root under E:/cargo-targets.
        root = commit.physical(os.environ["ZR_COMMIT_TEST_ROOT"])
        with tempfile.TemporaryDirectory(prefix="scoped-commit-", dir=root) as name:
            repo = Path(name)
            commit.git(repo, "init", "--quiet")
            commit.git(repo, "config", "user.name", "Jenkins regression")
            commit.git(repo, "config", "user.email", "jenkins-regression@localhost")
            for path in ("owned.txt", "foreign.txt", "other.txt"):
                (repo / path).write_text("base\n", encoding="utf-8")
            commit.git(repo, "add", "--", "owned.txt", "foreign.txt", "other.txt")
            commit.git(repo, "-c", "commit.gpgSign=false", "commit", "--quiet", "-m", "baseline")
            policy = {"authorization": {"source": "user", "instruction": "Allow scoped local commits"}, "controlFiles": []}
            payload = {"action": "commit-checkout", "sessionId": "ordinary-chat", "requestId": "task-commit",
                       "operationId": "task-commit", "paths": ["owned.txt"], "message": "Fix owned file"}
            yield repo, policy, {str(repo): "write"}, payload

    def run_commit(self, fixture):
        repo, policy, grants, payload = fixture
        return commit.execute(repo, payload, policy=policy, grants=grants)

    def test_commit_only_task_files_and_preserve_foreign_index_and_working_edits(self):
        with self.fixture() as f:
            repo = f[0]
            (repo / "owned.txt").write_text("owned fix\n", encoding="utf-8")
            (repo / "foreign.txt").write_text("foreign staged\n", encoding="utf-8")
            commit.git(repo, "add", "--", "foreign.txt")
            (repo / "foreign.txt").write_text("foreign working\n", encoding="utf-8")
            (repo / "other.txt").write_text("unrelated dirty\n", encoding="utf-8")
            staged = commit.git(repo, "ls-files", "--stage", "--", "foreign.txt").stdout
            result = self.run_commit(f)
            self.assertEqual(result["status"], "complete")
            self.assertEqual(commit.git(repo, "diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD").stdout.strip(), b"owned.txt")
            self.assertEqual(commit.git(repo, "ls-files", "--stage", "--", "foreign.txt").stdout, staged)
            self.assertEqual(commit.git(repo, "show", "HEAD:foreign.txt").stdout, b"base\n")
            self.assertEqual((repo / "foreign.txt").read_text(), "foreign working\n")
            self.assertEqual((repo / "other.txt").read_text(), "unrelated dirty\n")

    def test_new_and_deleted_files_with_spaces(self):
        with self.fixture() as f:
            repo, _, _, payload = f
            (repo / "owned.txt").unlink()
            (repo / "new file.txt").write_text("new\n", encoding="utf-8")
            payload["paths"] = ["owned.txt", "new file.txt"]
            self.assertEqual(self.run_commit(f)["status"], "complete")
            self.assertEqual(commit.git(repo, "show", "HEAD:new file.txt").stdout, b"new\n")
            self.assertNotEqual(commit.git(repo, "cat-file", "-e", "HEAD:owned.txt", check=False).returncode, 0)

    def test_repeated_request_returns_original_commit(self):
        with self.fixture() as f:
            (f[0] / "owned.txt").write_text("fix\n", encoding="utf-8")
            first = self.run_commit(f)
            (f[0] / "owned.txt").write_text("next task\n", encoding="utf-8")
            second = self.run_commit(f)
            self.assertEqual(first["commit"], second["commit"])
            self.assertTrue(second["reusedReceipt"])
            self.assertEqual(commit.git(f[0], "rev-list", "--count", "HEAD").stdout.strip(), b"2")

    def test_interrupted_receipt_recovers_existing_commit_without_recommit(self):
        with self.fixture() as f:
            repo = f[0]
            (repo / "owned.txt").write_text("fix\n", encoding="utf-8")
            first = self.run_commit(f)
            journal = repo / ".jenkins/state/management/checkout-commits/task-commit.json"
            interrupted = {k: v for k, v in first.items() if k not in {"commit", "finishedAt"}}
            interrupted["status"] = "committing"
            commit.write_json(journal, interrupted)
            result = self.run_commit(f)
            self.assertEqual(result["commit"], first["commit"])
            self.assertTrue(result["recoveredReceipt"])
            self.assertEqual(commit.git(repo, "rev-list", "--count", "HEAD").stdout.strip(), b"2")

    def test_operation_id_cannot_be_reused_for_another_request(self):
        with self.fixture() as f:
            self.run_commit(f)
            f[3]["message"] = "Different request"
            with self.assertRaisesRegex(ValueError, "already belongs"):
                self.run_commit(f)

    def test_source_and_configuration_changes_do_not_require_digests_or_session_grants(self):
        with self.fixture() as f:
            repo, policy, grants, payload = f
            directory = repo / ".codex"
            directory.mkdir()
            (directory / "config.toml").write_text(
                '[permissions.zircon-strict.filesystem]\n' + json.dumps(str(repo)) + ' = "write"\n# changed configuration\n',
                encoding="utf-8")
            (repo / "foreign.txt").write_text("another active task\n", encoding="utf-8")
            (repo / "owned.txt").write_text("task fix\n", encoding="utf-8")
            result = commit.execute(repo, payload, policy=policy)
            self.assertEqual(result["status"], "complete")
            self.assertFalse(result["sourceDigestRequired"])
            self.assertFalse(result["sessionGrantRequired"])
            self.assertFalse(result["formalAcceptance"])

    def test_query_is_read_only_and_no_changes_is_idempotent(self):
        with self.fixture() as f:
            f[3]["observationOnly"] = True
            self.assertEqual(self.run_commit(f)["status"], "not-started")
            self.assertFalse((f[0] / ".jenkins").exists())
            f[3]["observationOnly"] = False
            self.assertEqual(self.run_commit(f)["status"], "no-changes")
            self.assertEqual(commit.git(f[0], "rev-list", "--count", "HEAD").stdout.strip(), b"1")

    def test_active_source_reader_waits_without_git_mutation(self):
        with self.fixture() as f:
            before = commit.git(f[0], "rev-parse", "HEAD").stdout
            with patch.object(commit, "checkout_reader", return_value="active-reader"):
                result = self.run_commit(f)
            self.assertEqual(result["status"], "waiting")
            self.assertEqual(commit.git(f[0], "rev-parse", "HEAD").stdout, before)
            self.assertFalse((f[0] / ".jenkins").exists())

    def test_unsafe_or_unowned_paths_are_rejected(self):
        with self.fixture() as f:
            repo, policy, grants, payload = f
            for path in ["../outside", ".git/config", ".jenkins/state/credentials.json", ".", "*.txt", ":(glob)*", "missing.txt/../owned.txt"]:
                with self.subTest(path=path):
                    payload["paths"] = [path]
                    with self.assertRaises(ValueError):
                        self.run_commit(f)
            with self.assertRaisesRegex(ValueError, "no project write grant"):
                commit.selected_paths(repo, ["owned.txt"], policy, {str(repo): "read"})

    def test_submodule_commit_requires_separate_integration(self):
        with self.fixture() as f:
            repo, _, _, payload = f
            head = commit.git(repo, "rev-parse", "HEAD").stdout.decode().strip()
            commit.git(repo, "update-index", "--add", "--cacheinfo", "160000," + head + ",dependency")
            payload["paths"] = ["dependency"]
            with self.assertRaisesRegex(ValueError, "Submodule"):
                self.run_commit(f)


class CommitEntrySupportTests(unittest.TestCase):
    @unittest.skipUnless(os.name == "nt", "Windows Job lifecycle regression")
    def test_native_timeout_terminates_root_and_descendant_without_hanging(self):
        with tempfile.TemporaryDirectory(dir=os.environ["ZR_COMMIT_TEST_ROOT"]) as directory:
            program = "import subprocess,sys,time;subprocess.Popen([sys.executable,'-c','import time;time.sleep(300)']);time.sleep(300)"
            with self.assertRaises(subprocess.TimeoutExpired):
                commit.run_native_command([sys.executable, "-B", "-c", program], Path(directory), dict(os.environ), timeout=1)

    def test_shared_git_lock_excludes_another_owner_and_releases_on_exit(self):
        with tempfile.TemporaryDirectory(dir=os.environ["ZR_COMMIT_TEST_ROOT"]) as directory:
            lock = Path(directory) / "management-git.lock"
            with commit.git_lock(lock):
                with self.assertRaises(commit.CommitBusy):
                    with commit.git_lock(lock):
                        self.fail("A second owner acquired the shared Git lock")
            with commit.git_lock(lock):
                self.assertEqual(lock.stat().st_size, 1)

    def test_install_preserves_existing_routes_and_is_idempotent(self):
        class Client:
            raw = b'<flow-definition><properties><core>keep</core></properties><definition><script>node(\'zircon-windows-agent\') { oldRunner(\'--capsule\') }</script></definition></flow-definition>'
            posts = 0

            def request(self, path, *, method="GET", data=None, content_type=None):
                if method == "POST":
                    self.raw = data
                    self.posts += 1
                return 200, self.raw, {}

        with tempfile.TemporaryDirectory(dir=os.environ["ZR_COMMIT_TEST_ROOT"]) as directory:
            repo = Path(directory)
            source = repo / "tools/jenkins-management"
            source.mkdir(parents=True)
            addon = commit.BEGIN + "\nif (params.ACTION == 'commit-checkout') { return }\n" + commit.END + "\n"
            (source / "checkout-commit.groovy").write_text(addon, encoding="utf-8")
            client = Client()
            original = ET.fromstring(client.raw).findtext("definition/script")
            with patch.object(commit, "load_policy", return_value={}):
                self.assertEqual(commit.install(repo, client)["status"], "installed")
                self.assertTrue(commit.install(repo, client)["unchanged"])
            self.assertEqual(client.posts, 1)
            result = ET.fromstring(client.raw)
            self.assertEqual(result.findtext("properties/core"), "keep")
            self.assertTrue(result.findtext("definition/script").endswith(original))


if __name__ == "__main__":
    unittest.main(verbosity=2)
