"""Real Git regression tests for the Jenkins scoped checkout commit route."""
from __future__ import annotations

from contextlib import contextmanager
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
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

    def test_retry_after_precommit_failure_allows_unrelated_head_advance(self):
        with self.fixture() as f:
            repo = f[0]
            (repo / "owned.txt").write_text("ready fix\n", encoding="utf-8")
            original_git = commit.git

            def fail_before_stage(root, *args, **kwargs):
                if args and args[0] == "add":
                    raise ValueError("temporary index failure")
                return original_git(root, *args, **kwargs)

            with patch.object(commit, "git", side_effect=fail_before_stage):
                with self.assertRaisesRegex(ValueError, "temporary index failure"):
                    self.run_commit(f)
            (repo / "foreign.txt").write_text("another completed task\n", encoding="utf-8")
            commit.git(repo, "commit", "--only", "--no-gpg-sign", "-m", "Foreign task", "--", "foreign.txt")
            foreign_head = commit.git(repo, "rev-parse", "HEAD").stdout.decode().strip()
            result = self.run_commit(f)
            self.assertEqual(result["status"], "complete")
            self.assertEqual(result["headBefore"], foreign_head)
            self.assertEqual(len(result["previousAttempts"]), 1)
            self.assertEqual(commit.git(repo, "show", "HEAD:foreign.txt").stdout, b"another completed task\n")
            self.assertEqual(commit.git(repo, "diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD").stdout.strip(), b"owned.txt")

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

    @unittest.skipUnless(os.name == "nt", "Windows native validation pins")
    def test_validated_hashes_bind_the_receipt_and_mismatch_does_not_stage(self):
        with self.fixture() as f:
            repo, _, _, payload = f
            (repo / "owned.txt").write_bytes(b"validated fix\n")
            payload["expectedFileHashes"] = {"owned.txt": hashlib.sha256(b"validated fix\n").hexdigest()}
            (repo / "owned.txt").write_bytes(b"changed after validation\n")
            before = commit.git(repo, "ls-files", "--stage").stdout
            with self.assertRaisesRegex(ValueError, "declared bytes"):
                self.run_commit(f)
            self.assertEqual(commit.git(repo, "ls-files", "--stage").stdout, before)
            self.assertFalse((repo / ".jenkins/state/management/checkout-commits/task-commit.json").exists())
            (repo / "owned.txt").write_bytes(b"validated fix\n")
            result = self.run_commit(f)
            self.assertEqual(result["verifiedFileHashes"], payload["expectedFileHashes"])
            self.assertEqual(result["request"]["expectedFileHashes"], payload["expectedFileHashes"])
            self.assertEqual(commit.git(repo, "show", "HEAD:owned.txt").stdout, b"validated fix\n")
            (repo / "owned.txt").write_bytes(b"next task\n")
            self.assertEqual(self.run_commit(f)["commit"], result["commit"])
            payload["expectedFileHashes"]["owned.txt"] = hashlib.sha256(b"next task\n").hexdigest()
            with self.assertRaisesRegex(ValueError, "already belongs"):
                self.run_commit(f)

    def test_optional_validation_hashes_require_exact_scope_and_valid_sha256(self):
        with self.fixture() as f:
            repo, _, _, payload = f
            before = commit.git(repo, "ls-files", "--stage").stdout
            digest = hashlib.sha256(b"base\n").hexdigest()
            for value in (None, [], {}, {"foreign.txt": digest},
                          {"owned.txt": digest, "foreign.txt": digest},
                          {"owned.txt": "invalid"}, {"owned.txt": 123}):
                with self.subTest(value=value):
                    payload["expectedFileHashes"] = value
                    with self.assertRaisesRegex(ValueError, "exactly the selected"):
                        self.run_commit(f)
            self.assertEqual(commit.git(repo, "ls-files", "--stage").stdout, before)
            self.assertFalse((repo / ".jenkins").exists())

    @unittest.skipUnless(os.name == "nt", "Windows native validation pins")
    def test_validated_files_cannot_change_or_be_replaced_during_git_and_release_afterwards(self):
        with self.fixture() as f:
            repo, _, _, payload = f
            owned = repo / "owned.txt"
            owned.write_bytes(b"validated fix\n")
            payload["expectedFileHashes"] = {"owned.txt": hashlib.sha256(owned.read_bytes()).hexdigest()}
            replacement = repo / "replacement.txt"
            replacement.write_bytes(b"foreign replacement\n")
            original_git = commit.git
            attempts = []

            def protected_git(root, *args, **kwargs):
                if args and args[0] == "add":
                    with self.assertRaises(PermissionError):
                        owned.write_bytes(b"foreign write\n")
                    with self.assertRaises(PermissionError):
                        os.replace(replacement, owned)
                    attempts.append("write-and-replace-denied")
                return original_git(root, *args, **kwargs)

            with owned.open("r+b"):
                with self.assertRaises(commit.CommitBusy):
                    self.run_commit(f)
            self.assertFalse((repo / ".jenkins/state/management/checkout-commits/task-commit.json").exists())
            with patch.object(commit, "git", side_effect=protected_git):
                result = self.run_commit(f)
            self.assertEqual(attempts, ["write-and-replace-denied"])
            self.assertEqual(commit.git(repo, "show", result["commit"] + ":owned.txt").stdout, b"validated fix\n")
            owned.write_bytes(b"next task after release\n")
            self.assertEqual(owned.read_bytes(), b"next task after release\n")

    def test_submodule_commit_requires_separate_integration(self):
        with self.fixture() as f:
            repo, _, _, payload = f
            head = commit.git(repo, "rev-parse", "HEAD").stdout.decode().strip()
            commit.git(repo, "update-index", "--add", "--cacheinfo", "160000," + head + ",dependency")
            payload["paths"] = ["dependency"]
            with self.assertRaisesRegex(ValueError, "Submodule"):
                self.run_commit(f)


class CommitQueueTests(unittest.TestCase):
    @contextmanager
    def fixture(self):
        with CheckoutCommitTests.fixture(self) as fixture:
            repo, policy, _, _ = fixture
            (repo / ".codex").mkdir()
            (repo / ".codex/config.toml").write_text(
                '[permissions.zircon-strict.filesystem]\n' + json.dumps(str(repo)) + ' = "write"\n',
                encoding="utf-8")
            commit.write_json(repo / ".jenkins/state/deployment/driver.json",
                              {"driverDigest": "selected", "runtimeOperationId": "current"})
            with patch.object(commit, "load_policy", return_value=policy):
                yield fixture

    class Client:
        def __init__(self, repo, request, error=None):
            self.repo, self.request_input, self.error = repo, request, error
            self.posts = 0

        def request(self, path, **kwargs):
            self.posts += 1
            value = commit.read_json(commit.queue_record_path(self.repo, self.request_input["operationId"]))
            if value["request"] != self.request_input or value["status"] != "dispatching":
                raise AssertionError("Exact durable intent must precede HTTP delivery")
            if self.error:
                raise self.error
            return 201, b"", {"Location": "http://127.0.0.1:18080/queue/item/1/"}

    def test_lost_http_response_is_recovered_by_followup_without_duplicate_commit(self):
        with self.fixture() as f:
            repo, _, _, payload = f
            (repo / "owned.txt").write_text("ready fix\n", encoding="utf-8")
            client = self.Client(repo, payload, TimeoutError("response lost"))
            submitted = commit.submit_queued(repo, payload, client)
            self.assertEqual(submitted["status"], "pending")
            self.assertTrue(submitted["automaticFollowup"])
            commit.submit_queued(repo, payload, client)
            self.assertEqual(client.posts, 1)
            result = commit.process_pending(repo)
            self.assertEqual(result["requests"][0]["status"], "complete")
            actual = result["requests"][0]["commit"]
            (repo / "owned.txt").write_text("next task\n", encoding="utf-8")
            self.assertEqual(commit.process_pending(repo)["requests"], [])
            self.assertEqual(commit.submit_queued(repo, payload, client)["commit"], actual)
            self.assertEqual(commit.git(repo, "rev-list", "--count", "HEAD").stdout.strip(), b"2")

    def test_busy_request_stays_durable_and_commits_when_reader_releases(self):
        with self.fixture() as f:
            repo, _, _, payload = f
            (repo / "owned.txt").write_text("ready fix\n", encoding="utf-8")
            commit.submit_queued(repo, payload, self.Client(repo, payload))
            with patch.object(commit, "checkout_reader", return_value="active-source"):
                self.assertEqual(commit.process_pending(repo)["requests"][0]["status"], "waiting")
            with commit.git_lock(repo / ".jenkins/state/locks/management-git.lock"):
                self.assertEqual(commit.process_pending(repo)["requests"][0]["status"], "waiting")
            self.assertEqual(commit.process_pending(repo)["requests"][0]["status"], "complete")

    def test_query_does_not_write_or_submit_and_conflicting_identity_is_rejected(self):
        with self.fixture() as f:
            repo, _, _, payload = f
            client = self.Client(repo, payload)
            self.assertEqual(commit.submit_queued(repo, payload, client, query_only=True)["status"], "not-started")
            self.assertFalse(commit.queue_record_path(repo, payload["operationId"]).exists())
            self.assertEqual(client.posts, 0)
            commit.submit_queued(repo, payload, client)
            changed = {**payload, "paths": ["foreign.txt"]}
            with self.assertRaisesRegex(ValueError, "already belongs"):
                commit.submit_queued(repo, changed, client)

    def test_delivery_claim_is_unique_and_late_queue_update_cannot_undo_completion(self):
        with self.fixture() as f:
            repo, _, _, request = f
            commit.queue_intake(repo, request)
            self.assertTrue(commit.queue_intake(repo, request, claim_delivery=True)[1])
            self.assertFalse(commit.queue_intake(repo, request, claim_delivery=True)[1])
            commit.queue_update(repo, request, {"status": "complete", "commit": "actual"})
            self.assertEqual(commit.queue_update(repo, request, {"status": "waiting"})["commit"], "actual")

    def test_failed_request_requires_explicit_same_identity_retry(self):
        with self.fixture() as f:
            repo, _, _, payload = f
            commit.queue_intake(repo, payload)
            with patch.object(commit, "execute", side_effect=ValueError("invalid request")):
                self.assertEqual(commit.process_pending(repo)["requests"][0]["status"], "failed")
            failed = commit.process_pending(repo)
            self.assertEqual(failed["requests"], [])
            self.assertEqual(failed["failedRequests"], [payload["operationId"]])
            commit.submit_queued(repo, payload, self.Client(repo, payload))
            self.assertEqual(commit.process_pending(repo)["requests"][0]["status"], "no-changes")

    def test_unsubmitted_foreign_edits_are_never_discovered_by_followup(self):
        with self.fixture() as f:
            repo = f[0]
            (repo / "foreign.txt").write_text("foreign work\n", encoding="utf-8")
            self.assertEqual(commit.process_pending(repo)["requests"], [])
            self.assertEqual(commit.git(repo, "rev-list", "--count", "HEAD").stdout.strip(), b"1")

    def test_explicit_deferred_retry_reopens_a_failed_request(self):
        with self.fixture() as f:
            repo, _, _, request = f
            commit.submit_queued(repo, request, self.Client(repo, request))
            commit.queue_update(repo, request, {"status": "failed", "message": "temporary failure"})
            value, accepted = commit.queue_intake(repo, request)
            self.assertTrue(accepted)
            self.assertEqual(value["status"], "pending")
            self.assertNotIn("deliveryAttemptedAt", value)
            self.assertNotIn("message", value)
            self.assertEqual(value["previousFailures"][-1]["message"], "temporary failure")
            self.assertEqual(commit.process_pending(repo)["requests"][0]["status"], "no-changes")

    def test_native_git_timeout_retains_original_request_for_automatic_followup(self):
        with self.fixture() as f:
            repo, _, _, request = f
            (repo / "owned.txt").write_text("ready fix\n", encoding="utf-8")
            commit.queue_intake(repo, request)
            with patch.object(commit, "execute", side_effect=subprocess.TimeoutExpired(["git", "commit"], 120)):
                first = commit.process_pending(repo)
            self.assertEqual(first["requests"][0]["status"], "waiting")
            self.assertEqual(first["requests"][0]["reasonCode"], "native_git_timeout")
            self.assertEqual(first["failedRequests"], [])
            second = commit.process_pending(repo)
            self.assertEqual(second["requests"][0]["status"], "complete")
            observed = commit.queue_observation(repo, request)
            self.assertEqual(observed["request"], request)
            self.assertNotIn("message", commit.read_json(commit.queue_record_path(repo, request["operationId"])))
            self.assertEqual(commit.git(repo, "rev-list", "--count", "HEAD").stdout.strip(), b"2")


class CommitEntrySupportTests(unittest.TestCase):
    @unittest.skipUnless(os.name == "nt", "Windows exclusive-handle recovery")
    def test_abandoned_empty_index_lock_is_preserved_and_live_or_nonempty_locks_wait(self):
        with tempfile.TemporaryDirectory(dir=os.environ["ZR_COMMIT_TEST_ROOT"]) as directory:
            repo = Path(directory)
            (repo / ".git").mkdir()
            lock = repo / ".git/index.lock"
            lock.write_bytes(b"")
            with self.assertRaises(commit.CommitBusy):
                commit.recover_stale_index_lock(repo, "recent")
            old = time.time() - 1200
            os.utime(lock, (old, old))
            with lock.open("rb"):
                with self.assertRaises(commit.CommitBusy):
                    commit.recover_stale_index_lock(repo, "owned")
            recovered = commit.recover_stale_index_lock(repo, "abandoned")
            self.assertEqual(recovered["status"], "quarantined")
            self.assertFalse(lock.exists())
            self.assertEqual(Path(recovered["retainedPath"]).read_bytes(), b"")
            lock.write_bytes(b"pending index data")
            os.utime(lock, (old, old))
            with self.assertRaises(commit.CommitBusy):
                commit.recover_stale_index_lock(repo, "nonempty")
            self.assertEqual(lock.read_bytes(), b"pending index data")

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
