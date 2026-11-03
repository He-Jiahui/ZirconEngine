"""Regressions for conservative consolidation selection, without filesystem fixtures."""
import copy
import importlib.util
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

SOURCE = Path(__file__).parents[1] / "worktree_consolidation.py"
spec = importlib.util.spec_from_file_location("consolidation_under_test", SOURCE)
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


class SelectionTests(unittest.TestCase):
    def test_native_worktree_lock_without_reason_is_preserved(self):
        fields = m.porcelain_fields("worktree E:/task\nHEAD " + "a" * 40 + "\nlocked")
        self.assertIn("locked", fields)
        self.assertEqual(fields["locked"], "")

    def test_clean_contained_worktree_is_eligible(self):
        self.assertEqual(m.disposition(merged=True, dirty=False), "eligible")

    def test_unique_commits_are_never_reclaimed(self):
        self.assertEqual(m.disposition(merged=False, dirty=False), "unique-commits")

    def test_dirty_merged_tree_is_preserved(self):
        self.assertEqual(m.disposition(merged=True, dirty=True), "uncommitted-source")

    def test_unknown_native_owner_is_preserved(self):
        self.assertEqual(m.disposition(merged=True, dirty=False, locked=True), "native-git-operation")

    def test_durable_readers_and_writers_are_preserved(self):
        self.assertEqual(m.disposition(merged=True, dirty=False, holds=["unknown"]), "retained-reader-or-writer")

    def test_ignored_files_are_not_treated_as_disposable(self):
        self.assertEqual(m.disposition(merged=True, dirty=False, ignored=True), "untracked-or-ignored-files")

    def test_active_process_reference_is_preserved(self):
        self.assertEqual(m.disposition(merged=True, dirty=False, process=True), "active-process-reference")

    def test_missing_owner_cannot_be_inferred_from_cleanliness(self):
        self.assertEqual(m.disposition(merged=True, dirty=False, registered=False), "unregistered-owner")


class PlanTests(unittest.TestCase):
    def setUp(self):
        self.plan = {"taskId": "task", "operationId": "op", "repository": str(m.ROOT), "mode": "apply"}

    def test_main_and_remote_refs_cannot_be_selected(self):
        for ref in ("refs/heads/main", "refs/remotes/origin/topic", "refs/tags/topic"):
            plan = {**self.plan, "branches": [{"ref": ref, "oid": "a" * 40}]}
            with self.assertRaises(ValueError):
                m.validate_plan(plan)

    def test_branch_selection_requires_immutable_oid(self):
        with self.assertRaises(ValueError):
            m.validate_plan({**self.plan, "branches": [{"ref": "refs/heads/codex/a", "oid": "HEAD"}]})

    def test_path_components_cannot_escape_branch_namespace(self):
        for ref in ("refs/heads/codex/../main", "refs/heads/codex/a//b", "refs/heads/codex/a.lock"):
            with self.assertRaises(ValueError):
                m.validate_plan({**self.plan, "branches": [{"ref": ref, "oid": "a" * 40}]})

    def test_duplicate_refs_are_rejected_before_mutation(self):
        row = {"ref": "refs/heads/codex/a", "oid": "a" * 40}
        with self.assertRaises(ValueError):
            m.validate_plan({**self.plan, "branches": [row, row]})

    def test_repository_root_and_shared_candidate_are_protected(self):
        for root in (m.ROOT, m.ROOT / ".jenkins/worktrees/integration-queue", m.ROOT / "other"):
            with self.assertRaises(ValueError):
                m.validate_plan({**self.plan, "worktrees": [{"path": str(root), "head": "a" * 40}]})

    def test_decisions_do_not_modify_original_plan(self):
        before = copy.deepcopy(self.plan)
        m.validate_plan(self.plan)
        self.assertEqual(before, self.plan)


class PermissionAdapterTests(unittest.TestCase):
    def test_adapter_retains_canonical_literal_path_boundary(self):
        delivery = SimpleNamespace(commit=SimpleNamespace(selected_paths=None))
        m.install(delivery, m.ROOT)
        for name in ("../foreign.txt", "tools/*.py", ".git/config"):
            with self.subTest(path=name), self.assertRaises(ValueError):
                delivery.commit.selected_paths(m.ROOT, [name], {"controlFiles": []}, {})

    def test_another_repository_cannot_acquire_this_adapter(self):
        delivery = SimpleNamespace(commit=SimpleNamespace(selected_paths=None))
        with self.assertRaises(ValueError):
            m.install(delivery, m.ROOT.parent / "other")
        self.assertIsNone(delivery.commit.selected_paths)


@unittest.skipUnless(os.name == "nt", "Native Windows file allocation")
class AllocationTests(unittest.TestCase):
    def test_shared_hard_link_allocation_is_not_claimed_as_released(self):
        with tempfile.TemporaryDirectory(prefix="allocation-", dir=os.environ["TEMP"]) as folder:
            source = Path(folder) / "owned.bin"
            source.write_bytes(b"allocation-contract" * 1024)
            before = m.file_facts(source)
            self.assertEqual(before["hardLinks"], 1)
            self.assertGreater(before["allocatedBytes"], 0)
            linked = source.with_name("shared.bin")
            os.link(source, linked)
            shared = m.file_facts(source)
            self.assertEqual(shared["hardLinks"], 2)
            self.assertEqual(shared["allocatedBytes"], 0)
            self.assertEqual(before["sha256"], shared["sha256"])


class CandidateInventoryTests(unittest.TestCase):
    def test_new_registration_precedes_output_and_rejects_duplicate_attempt(self):
        with tempfile.TemporaryDirectory(prefix="candidate-inventory-", dir=os.environ["TEMP"]) as folder:
            base = Path(folder)
            run = base / "runs/123"
            run.mkdir(parents=True)
            (run / "artifacts.json").write_text('{"task_id":"zircon-git-sync-123"}')
            delivery = SimpleNamespace(OUTPUT=base / "deliveries")
            output = delivery.OUTPUT / "owned/candidate-checkout-0"
            with patch.object(m, "MERGE_REPORTS", base / "runs"), patch.dict(os.environ,
                    {"JOB_NAME": "zircon-merge-queue", "NODE_NAME": "zircon-windows-agent", "BUILD_NUMBER": "123"}):
                m.register_candidate_output(delivery, output, {"task_id": "owned"})
                self.assertFalse(output.exists())
                self.assertEqual(len(list((run / "candidate-inventory").glob("*.json"))), 1)
                with self.assertRaises(FileExistsError):
                    m.register_candidate_output(delivery, output, {"task_id": "owned"})
                with self.assertRaises(ValueError):
                    m.register_candidate_output(delivery, output.parent.parent / "foreign/candidate-checkout-0", {"task_id": "owned"})

    def test_adapter_preserves_pinned_loader_for_other_modules(self):
        ordinary, branch = SimpleNamespace(), SimpleNamespace(register_candidate_output=None)
        calls = []
        def original(name, *args, **kwargs):
            calls.append((name, args, kwargs))
            return branch if name == "periodic_merge_branch" else ordinary
        utility = SimpleNamespace(load=original)
        delivery = SimpleNamespace(commit=SimpleNamespace(selected_paths=None))
        with patch.dict(m.sys.modules, {"periodic_merge_util": utility}), patch.dict(os.environ, {"JOB_NAME": "zircon-merge-queue"}):
            m.install(delivery, m.ROOT)
            self.assertIs(utility.load("other", "path", "digest"), ordinary)
            self.assertIs(utility.load("periodic_merge_branch", "path", "digest"), branch)
            self.assertIs(branch.register_candidate_output, m.register_candidate_output)
            self.assertEqual(calls, [("other", ("path", "digest"), {}), ("periodic_merge_branch", ("path", "digest"), {})])

    def test_immutable_factory_registration_failure_starts_no_native_git(self):
        component = m.ROOT / ".jenkins/runtime/branch-integration/c3b118ea14f1eb8757672dacd4c32cf715b0c1649c03eebc9f7e461180e0bc0f"
        branch_path = component / "branch_integration.py"
        branch = m.load("prelaunch_branch_contract", {"path": str(branch_path), "sha256": m.sha(branch_path.read_bytes())})
        with tempfile.TemporaryDirectory(prefix="prelaunch-", dir=os.environ["TEMP"]) as folder:
            base = Path(folder)
            def module(name, path):
                return m.load("prelaunch_" + name, {"path": str(path), "sha256": m.sha(Path(path).read_bytes())})
            delivery = SimpleNamespace(OUTPUT=base, module=module,
                                       commit=SimpleNamespace(physical=m.physical, ID=m.ID))
            record = {"request": {"operationId": "owned", "sessionId": "owner"}}
            with patch.object(branch, "register_candidate_output", side_effect=FileNotFoundError("original registry missing")), \
                    patch.dict(os.environ, {"ZR_COMMIT_NATIVE": "1"}):
                with self.assertRaises(FileNotFoundError):
                    branch.candidate_git(delivery, m.ROOT, m.ROOT, "checkout", "a" * 40,
                                         record, base / "queue.json", {})
            self.assertNotIn("candidateGit", record)
            self.assertNotIn("candidateGitAttempt", record)
            self.assertFalse((base / "owned").exists())


if __name__ == "__main__":
    unittest.main(verbosity=2)
