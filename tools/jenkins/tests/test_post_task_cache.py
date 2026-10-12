"""Owned terminal TEMP cleanup: identity, native EOF, references and replays."""
from contextlib import contextmanager
from copy import deepcopy
from pathlib import Path
import os
import tempfile
import time
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, digest, file_digest
from tools.jenkins.resources import Capacity, ResourceManager, canonical_build_root
from tools.jenkins.state import State
from tools.jenkins.workflow.execution import ExecutionKey
from tools.jenkins.workflow import post_task_cache as cache


class PostTaskCacheTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.repo = Path(self.tmp.name)
        self.state = State(self.repo / "state.db")
        self.approved = canonical_build_root(r"E:\cargo-targets")
        self.identity = {"repositoryId": "repo", "sessionId": "session", "requestId": "request",
                         "attemptId": "attempt", "generation": 1, "sourceInputDigest": "a" * 64,
                         "coverageDigest": "b" * 64, "stageImplementationDigest": "c" * 64}
        self.flow_id = "session:request:attempt"
        self.driver = {"digest": "c" * 64, "generation": "1", "runtimeOperationId": "runtime"}
        self.prep = digest({"fixture": self.tmp.name})
        self.temp_base = self.approved.namespace() / "preparations" / self.prep / "tmp"
        recipe = {"kind": "cargo", "preparationKey": self.prep, "buildRoot": str(self.approved.path),
                  "preparationRoot": str(self.temp_base.parent / "inputs" / "source"), "tempRoot": str(self.temp_base),
                  "executionEnvironment": {name: str(self.temp_base) for name in ("TEMP", "TMP", "TMPDIR")}}
        self.recipe_ref = digest(recipe)
        self.state.put("recipe_plan", self.recipe_ref, {"recipe": recipe, "recipeDigest": self.recipe_ref,
            "sourceRef": "a" * 64, "coverageRef": "b" * 64, "driver": self.driver, "generation": "1"})
        self.key = ExecutionKey("a" * 64, self.prep, self.recipe_ref, "c" * 64)
        self.eid = digest({"identity": self.key.digest, "ordinal": 1})[:32]
        self.execution = {"executionKey": self.key.__dict__, "executionKeyDigest": self.key.digest,
                          "ordinal": 1, "sourceDigest": "a" * 64, "sealedInputRef": "a" * 64,
                          "preparationKey": self.prep, "recipeRef": self.recipe_ref, "recipeDigest": self.recipe_ref,
                          "driverDigest": "c" * 64, "driver": self.driver, "generation": "1", "operationId": "runtime",
                          "coverageDigest": "b" * 64, "buildRoot": str(self.approved.path), "status": "passed"}
        self.state.put("execution", self.eid, self.execution)
        self.state.put("execution_binding", digest({"consumer": self.flow_id, "executionKey": self.key.digest}),
                       {"executionId": self.eid, "consumerId": self.flow_id})
        self.state.put("execution_consumer", self.eid + ":" + self.flow_id,
                       {"executionId": self.eid, "consumerId": self.flow_id, "status": "active"})
        self.state.put("workflow", self.flow_id, {"identity": self.identity, "repositoryId": "repo",
            "status": "failed", "executionId": self.eid, "recipeRef": self.recipe_ref,
            "sourceDigest": "a" * 64, "coverageDigest": "b" * 64, "driverDigest": "c" * 64,
            "generation": 1, "buildRoot": str(self.approved.path), "ownedPaths": ["src"]})
        actions = ["validation"] if self._testMethodName == "test_no_gc_grant_assesses_but_does_not_delete" else ["validation", "gc"]
        self.state.authorize_session("repo", "session", "tester", ["src"], actions, {"source": "user", "task": "test"})
        request = {**self.identity, "sealedInputRef": "a" * 64, "driverDigest": "c" * 64,
                   "buildRoot": str(self.approved.path), "runtimeOperationId": "runtime", "ownedPaths": ["src"]}
        self.state.submit_request(request)
        self.state.transition_request("repo", "session", "request", 1, "pending", "failed")
        self.policy = {"cpuBudget": 6, "memoryBudget": 12 * 1024**3, "diskBudget": 40 * 1024**3,
            "diskReserveBytes": 35 * 1024**3, "agingSeconds": 60,
            "postTaskCache": {"enabled": True, "everyTerminalTask": True, "minFreeBytes": 35 * 1024**3,
                "maxFiles": 64, "maxBytes": 256 * 1024**2, "maxSeconds": 10,
                "preserveWarmPreparation": True, "preserveAcceptedReferences": True}}
        self.state.put("resource_policy", "default", self.policy)
        self.snapshot = {"generation": "scan", "cpu": 6, "memoryBytes": 12 * 1024**3,
                         "diskBytes": 50 * 1024**3, "complete": True, "observedAt": time.time(),
                         "buildRoot": str(self.approved.path), "volumeIdentity": self.approved.volume_identity}
        inventory_patch = patch.object(ResourceManager, "capture_inventory", side_effect=lambda _: {**self.snapshot, "observedAt": time.time()})
        inventory_patch.start()
        self.addCleanup(inventory_patch.stop)
        self.payload = {"identity": self.identity, "phase": "assess"}
        self._terminal_jobs()

    def _terminal_jobs(self):
        for role, key, pid in (("execution_host_guard", "guard", 1), ("phase", "phase", 2)):
            identity = {"pid": pid, "creationTime": "123", "executable": "python.exe", "commandDigest": ""}
            proof = {"identity": identity, "complete": True, "processExitCode": 0, "activeProcesses": 0,
                     "stdoutEof": True, "stderrEof": True, "childrenGone": True}
            self.state.put("native_job", key, {"owner": self.eid, "executionId": self.eid, "status": "terminal",
                "role": role, "pid": pid, "birthToken": "123", "executable": "python.exe",
                "completeProof": proof, "terminalProof": proof, **({"guardNativeJobId": "guard"} if role == "phase" else {})})
        self.state.put("execution_launch", self.eid, {"status": "collected", "guardNativeJobId": "guard"})

    def _mutate(self, domain, key, updates):
        row = self.state.get(domain, key)
        self.state.put(domain, key, {**row["payload"], **updates}, expected_version=row["version"])

    def _scratch(self):
        self._mutate("execution", self.eid, {"status": "preparing", "reservationKey": self.eid + ":r"})
        self.state.put("resource_reservation", self.eid + ":r", {"status": "active", "owner": self.eid})
        self.state.put("pool_writer_hold", self.prep, {"status": "active", "owner": self.eid, "reservationKey": self.eid + ":r"})
        self.addCleanup(self._clean_fixture_pool)
        scratch = cache.register_execution_scratch(self.state, self.eid)
        self._mutate("execution", self.eid, {"status": "passed", "taskScratch": {
            "scratchGeneration": scratch["scratchGeneration"], "effectiveEnvironmentDigest": digest(scratch["effectiveEnvironment"])}})
        task_scratch = {"scratchGeneration": scratch["scratchGeneration"], "effectiveEnvironment": scratch["effectiveEnvironment"]}
        self.state.put("execution_host", self.eid, {"status": "terminal", "taskScratch": task_scratch,
            "phaseRecords": [{"phase": "compile", "nativeJobId": "phase", "taskScratch": task_scratch,
                              "effectiveEnvironmentDigest": digest(scratch["effectiveEnvironment"])}]})
        self._mutate("resource_reservation", self.eid + ":r", {"status": "released"})
        self._mutate("pool_writer_hold", self.prep, {"status": "released"})
        return scratch, Path(scratch["path"])

    def _clean_fixture_pool(self):
        # Every fixture pool is derived from this TemporaryDirectory's unique
        # digest; verify the approved namespace before removing fixture bytes.
        import shutil
        path = self.temp_base.parent
        checked = cache.physical_path_under(self.approved, path)
        if checked == path and path.name == self.prep:
            shutil.rmtree(path, ignore_errors=True)

    def test_assessment_idempotent_pins_original_task_and_preserves_warm_unknown(self):
        result = cache.assess(self.payload, self.state, self.repo)
        again = cache.assess(self.payload, self.state, self.repo)
        self.assertEqual(result, again)
        self.assertEqual("terminal", result["nativeEvidence"]["status"])
        self.assertEqual("preserve", result["warmPreparation"]["disposition"])
        self.assertEqual("preserve", result["unknownOwnership"])
        self.assertFalse(result["cleanupRequired"])
        self.assertEqual([], self.state.list("post_task_cache_gc"))
        self.assertFalse(result["JenkinsAcceptance"])

    def test_light_task_assessment_has_no_native_job_or_gc(self):
        self._mutate("workflow", self.flow_id, {"recipeRef": None, "executionId": "light"})
        result = cache.assess(self.payload, self.state, self.repo)
        self.assertEqual("light", result["nativeEvidence"]["status"])
        self.assertEqual("no_registered_task_scratch", result["scratch"]["status"])
        self.assertEqual(2, len(self.state.list("native_job")))

    def test_changed_identity_root_raw_path_and_request_attempt_are_rejected(self):
        for payload in ({**self.payload, "identity": {**self.identity, "attemptId": "other"}},
                        {**self.payload, "buildRoot": r"F:\cargo-targets"},
                        {**self.payload, "candidatePath": "C:/anything"}):
            with self.subTest(payload=payload), self.assertRaises(JenkinsError):
                cache.assess(payload, self.state, self.repo)
        self.assertEqual([], self.state.list("post_task_cache"))

    def test_pending_guard_eof_does_not_freeze_terminal_assessment(self):
        proof = deepcopy(self.state.get("native_job", "guard")["payload"]["completeProof"])
        proof["stdoutEof"] = False
        self._mutate("native_job", "guard", {"completeProof": proof, "terminalProof": proof})
        pending = cache.assess(self.payload, self.state, self.repo)
        self.assertEqual("pending", pending["status"])
        self.assertEqual([], self.state.list("post_task_cache"))
        self._terminal_jobs()
        result = cache.assess(self.payload, self.state, self.repo)
        self.assertEqual(pending["operationId"], result["operationId"])
        self.assertEqual("assessed", result["status"])

    @unittest.skipUnless(os.name == "nt", "Windows handle-relative storage")
    def test_owned_scratch_isolated_bounded_delete_and_replay(self):
        scratch, path = self._scratch()
        payload = b"owned terminal tempfile"
        (path / "one.tmp").write_bytes(payload)
        old_shared = self.temp_base / "foreign.tmp"
        old_shared.write_bytes(b"unknown historical tempfile")
        result = cache.assess(self.payload, self.state, self.repo)
        self.assertTrue(result["cleanupRequired"])
        self.assertEqual(len(payload), result["scratch"]["totalBytes"])
        deleted = cache.garbage_collect({**self.payload, "phase": "gc"}, self.state, self.repo)
        self.assertEqual("complete", deleted["status"])
        self.assertEqual(len(payload), deleted["reclaimedBytes"])
        self.assertFalse((path / "one.tmp").exists())
        self.assertTrue(old_shared.exists())
        replay = cache.garbage_collect({**self.payload, "phase": "gc"}, self.state, self.repo)
        self.assertEqual(deleted, replay)
        self.assertEqual(1, len(self.state.list("post_task_cache_file")))

    @unittest.skipUnless(os.name == "nt", "Windows handle-relative storage")
    def test_active_writer_and_accepted_digest_prevent_deletion(self):
        scratch, path = self._scratch()
        file = path / "accepted.tmp"
        file.write_bytes(b"accepted published bytes")
        checksum = file_digest(file)
        self.state.put("artifact_ref", "published", {"digest": checksum, "status": "active"})
        result = cache.assess(self.payload, self.state, self.repo)
        self.assertFalse(result["cleanupRequired"])
        self.assertEqual("historical_or_active_digest_reference", result["scratch"]["retained"][0]["reasonCode"])
        self.assertTrue(file.exists())
        self.assertEqual("no_eligible_files", cache.garbage_collect(self.payload, self.state, self.repo)["status"])

    @unittest.skipUnless(os.name == "nt", "Windows handle-relative storage")
    def test_new_pool_writer_between_assessment_and_gc_blocks_exact_pool(self):
        scratch, path = self._scratch()
        (path / "one.tmp").write_bytes(b"owned")
        cache.assess(self.payload, self.state, self.repo)
        self._mutate("pool_writer_hold", self.prep, {"status": "active", "owner": "new-execution"})
        with self.assertRaises(JenkinsError) as caught:
            cache.garbage_collect(self.payload, self.state, self.repo)
        self.assertEqual("post_task_writer_active", caught.exception.code)
        self.assertTrue((path / "one.tmp").exists())

    @unittest.skipUnless(os.name == "nt", "Windows handle-relative storage")
    def test_preexisting_scratch_is_protected_not_adopted(self):
        path = self.temp_base / self.eid
        path.mkdir(parents=True)
        self.addCleanup(self._clean_fixture_pool)
        self._mutate("execution", self.eid, {"status": "preparing", "reservationKey": self.eid + ":r"})
        self.state.put("resource_reservation", self.eid + ":r", {"status": "active", "owner": self.eid})
        self.state.put("pool_writer_hold", self.prep, {"status": "active", "owner": self.eid, "reservationKey": self.eid + ":r"})
        with self.assertRaises(JenkinsError) as caught:
            cache.register_execution_scratch(self.state, self.eid)
        self.assertEqual("task_scratch_ownership_unknown", caught.exception.code)
        self.assertIsNone(self.state.get("execution_scratch", self.eid))

    @unittest.skipUnless(os.name == "nt", "Windows handle-relative storage")
    def test_mutated_or_hardlinked_scratch_is_not_removed(self):
        _, path = self._scratch()
        file = path / "file.tmp"
        file.write_bytes(b"before")
        cache.assess(self.payload, self.state, self.repo)
        file.write_bytes(b"after")
        result = cache.garbage_collect(self.payload, self.state, self.repo)
        self.assertEqual("unknown", result["status"])
        self.assertEqual(0, result["reclaimedBytes"])
        self.assertTrue(file.exists())

    @unittest.skipUnless(os.name == "nt", "Windows handle-relative storage")
    def test_hardlinks_and_file_inventory_bound_protect_all_files(self):
        _, path = self._scratch()
        file = path / "file.tmp"
        file.write_bytes(b"content")
        os.link(file, path / "alias.tmp")
        result = cache.assess(self.payload, self.state, self.repo)
        self.assertEqual("protected", result["scratch"]["status"])
        self.assertFalse(result["cleanupRequired"])
        self.assertTrue(file.exists())

    @unittest.skipUnless(os.name == "nt", "Windows handle-relative storage")
    def test_no_gc_grant_assesses_but_does_not_delete(self):
        _, path = self._scratch()
        (path / "owned.tmp").write_bytes(b"temporary")
        result = cache.assess(self.payload, self.state, self.repo)
        self.assertTrue(result["cleanupRequired"])
        self.assertFalse(result["gcAuthorized"])
        with self.assertRaises(JenkinsError):
            cache.garbage_collect(self.payload, self.state, self.repo)
        self.assertTrue((path / "owned.tmp").exists())

    @unittest.skipUnless(os.name == "nt", "Windows handle-relative storage")
    def test_cleanup_hold_blocks_same_pool_but_allows_other_pool_admission(self):
        from concurrent.futures import ThreadPoolExecutor
        from threading import Event
        _, path = self._scratch()
        (path / "owned.tmp").write_bytes(b"temporary")
        cache.assess(self.payload, self.state, self.repo)
        manager = ResourceManager(self.state, Capacity(6, 12 * 1024**3, 40 * 1024**3))
        manager.publish_inventory('fresh', Capacity(6, 12 * 1024**3, 50 * 1024**3), build_root=str(self.approved.path))
        reached, resume = Event(), Event()
        fact = cache._file_fact
        calls = 0
        def paused(*args, **kwargs):
            nonlocal calls
            calls += 1
            if calls == 1:
                reached.set()
                self.assertTrue(resume.wait(5))
            return fact(*args, **kwargs)
        with patch.object(cache, '_file_fact', side_effect=paused), ThreadPoolExecutor(max_workers=1) as worker:
            future = worker.submit(cache.garbage_collect, self.payload, self.state, self.repo)
            try:
                self.assertTrue(reached.wait(5))
                with self.assertRaises(JenkinsError) as caught:
                    manager.admit('next-same', Capacity(1, 1, 1), inventory_generation='fresh',
                                  build_root=str(self.approved.path), writer_key=self.prep, request_id='same')
                self.assertEqual('pool_cache_maintenance_active', caught.exception.code)
                self.assertIsNotNone(manager.admit('next-other', Capacity(1, 1, 1), inventory_generation='fresh',
                                                  build_root=str(self.approved.path), writer_key='unrelated-pool', request_id='other'))
            finally:
                resume.set()
            self.assertEqual('complete', future.result(timeout=5)['status'])
        self.assertFalse((path / "owned.tmp").exists())
        self.assertEqual('released', self.state.get('cache_maintenance_hold', self.prep)['payload']['status'])

    @unittest.skipUnless(os.name == "nt", "Windows handle-relative storage")
    def test_phase_environment_and_directory_generation_change_are_protected(self):
        _, path = self._scratch()
        (path / "owned.tmp").write_bytes(b"temporary")
        row = self.state.get('execution_host', self.eid)
        phases = deepcopy(row['payload']['phaseRecords'])
        phases[0]['taskScratch']['effectiveEnvironment']['TEMP'] = str(self.temp_base)
        self._mutate('execution_host', self.eid, {'phaseRecords': phases})
        result = cache.assess(self.payload, self.state, self.repo)
        self.assertEqual('protected', result['scratch']['status'])
        self.assertEqual('task_scratch_environment_unproven', result['scratch']['reasonCode'])
        self.assertTrue((path / "owned.tmp").exists())

    @unittest.skipUnless(os.name == "nt", "Windows handle-relative storage")
    def test_replaced_directory_identity_is_protected(self):
        import shutil
        _, path = self._scratch()
        (path / "owned.tmp").write_bytes(b"temporary")
        shutil.rmtree(path)
        path.mkdir()
        (path / "foreign.tmp").write_bytes(b"unknown replacement")
        result = cache.assess(self.payload, self.state, self.repo)
        self.assertEqual('protected', result['scratch']['status'])
        self.assertEqual('task_scratch_generation_changed', result['scratch']['reasonCode'])
        self.assertTrue((path / 'foreign.tmp').exists())

    def test_live_exact_native_identity_keeps_cleanup_pending(self):
        from tools.jenkins.processes.identity import current_identity
        import sys
        live = current_identity(os.getpid(), executable=sys.executable).to_dict()
        proof = deepcopy(self.state.get('native_job', 'guard')['payload']['completeProof'])
        proof['identity'] = live
        self._mutate('native_job', 'guard', {'pid': live['pid'], 'birthToken': live['creationTime'],
            'executable': live['executable'], 'completeProof': proof, 'terminalProof': proof})
        result = cache.assess(self.payload, self.state, self.repo)
        self.assertEqual('pending', result['nativeEvidence']['status'])
        self.assertFalse(result['nativeEvidence']['cleanupAllowed'])

    @unittest.skipUnless(os.name == "nt", "Windows handle-relative storage")
    def test_lost_deletion_acknowledgement_never_credits_removed_bytes(self):
        _, path = self._scratch()
        file = path / 'owned.tmp'
        file.write_bytes(b'temporary')
        result = cache.assess(self.payload, self.state, self.repo)
        fact = result['scratch']['files'][0]
        key = digest({'operation': result['operationId'], 'file': fact})
        self.state.put('post_task_cache_file', key, {'operationId': result['operationId'],
            'assessmentDigest': result['resultDigest'], 'file': fact, 'status': 'pending'})
        file.unlink()
        replay = cache.garbage_collect(self.payload, self.state, self.repo)
        self.assertEqual('unknown', replay['status'])
        self.assertEqual(0, replay['reclaimedBytes'])
        self.assertEqual(0, replay['removedFileCount'])

    @unittest.skipUnless(os.name == "nt", "Windows handle-relative storage")
    def test_file_inventory_and_time_bound_protect_all_files(self):
        _, path = self._scratch()
        for index in range(65):
            (path / f'file-{index}.tmp').write_bytes(b'x')
        result = cache.assess(self.payload, self.state, self.repo)
        self.assertEqual('protected', result['scratch']['status'])
        self.assertEqual('post_task_inventory_limit', result['scratch']['reasonCode'])
        self.assertFalse(result['cleanupRequired'])
        self.assertEqual(65, len(list(path.iterdir())))


if __name__ == "__main__":
    unittest.main()
