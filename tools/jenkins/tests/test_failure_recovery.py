import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.resources import canonical_build_root
from tools.jenkins.source import apply_owned_patch
from tools.jenkins.state import State
from tools.jenkins.workflow.execution import ExecutionEngine
from tools.jenkins.workflow.recovery import recover_flow


class FailureRecoveryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.store = self.root / "object-store"
        self.store.mkdir()
        self.source = self.root / "src" / "lib.rs"
        self.source.parent.mkdir()
        # The original is intentionally uncommitted, rather than a Git blob.
        self.original = b"fn value() -> u32 { 2 } // uncommitted change\n"
        self.source.write_bytes(self.original)
        self.state = State(self.root / "state.sqlite3")
        self.state.authorize_session("repo", "session", "user", ["src"],
            ["implementation", "validation"], {"source": "user", "task": "isolated recovery acceptance"})
        self.key = "session:request:attempt"
        self.identity = {"repositoryId": "repo", "sessionId": "session", "requestId": "request",
                         "attemptId": "attempt", "generation": 1}
        self.auth = {"repositoryId": "repo", "sessionId": "session"}
        self.hash = hashlib.sha256(self.original).hexdigest()

    def tearDown(self):
        self.temp.cleanup()

    def apply(self):
        with patch("tools.jenkins.source._approved_object_root", return_value=self.store):
            return apply_owned_patch(self.state, self.root, "attempt", {"src/lib.rs": "fn value() -> u32 { 9 }\n"},
                {"src/lib.rs": self.hash}, operation_id="patch", authorization=self.auth)

    def workflow(self, **extra):
        self.state.put("workflow", self.key, {"identity": self.identity, "repositoryId": "repo",
            "executionId": "not-launched", "repoRoot": str(self.root), "sourceDigest": "source",
            "sourceManifest": {}, "patchOperationRefs": ["patch"], "buildRoot": "D:/cargo-targets",
            "status": "failed", "stages": [{"name": "acceptance", "status": "pending"}], **extra})

    def recover(self):
        with patch("tools.jenkins.workflow.recovery.canonical_build_root", return_value=object()), \
             patch("tools.jenkins.workflow.recovery.physical_path_under", side_effect=lambda root, path: Path(path)):
            return recover_flow(self.state, self.root, self.key, error="syntax failed")

    def test_uncommitted_before_bytes_are_restored_and_replay_is_idempotent(self):
        self.apply()
        self.workflow()
        self.assertEqual("complete", self.recover()["status"])
        self.assertEqual(self.original, self.source.read_bytes())
        self.assertEqual([], self.state.list("path_claims"))
        self.source.write_bytes(b"later foreign editing\n")
        self.assertEqual("complete", self.recover()["status"])
        self.assertEqual(b"later foreign editing\n", self.source.read_bytes())

    def test_foreign_edit_is_preserved_and_compensation_is_blocked(self):
        self.apply()
        self.workflow()
        self.source.write_bytes(b"foreign\n")
        result = self.recover()
        self.assertEqual("blocked", result["status"])
        self.assertEqual("foreign_edit_during_compensation", result["recovery"]["reasonCode"])
        self.assertEqual(b"foreign\n", self.source.read_bytes())

    def test_unproven_native_process_blocks_compensation(self):
        self.apply()
        self.workflow()
        self.state.put("native_job", "job", {"executionId": "not-launched", "status": "running"})
        self.assertEqual("reconciling", self.recover()["status"])
        self.assertNotEqual(self.original, self.source.read_bytes())

    def test_running_orphan_recovers_only_after_real_later_boot(self):
        self.apply()
        self.state.put('resource_policy', 'default', {'cpuBudget': 1, 'memoryBudget': 1, 'diskBudget': 1})
        source, coverage, driver, preparation = (digest(name) for name in ('source', 'coverage', 'driver', 'preparation'))
        build_root = 'D:/cargo-targets'
        driver_binding = {'digest': driver, 'generation': '1', 'runtimeOperationId': 'runtime-orphan'}
        recipe = {'kind': 'cargo', 'buildRoot': build_root, 'preparationKey': preparation,
                  'preparationRoot': str(canonical_build_root(build_root).namespace() / 'preparations' /
                                         preparation / 'inputs' / 'source'),
                  'resourceEstimates': {'cpu': 1, 'memoryBytes': 1, 'diskBytes': 1}}
        recipe_ref = digest(recipe)
        self.state.put('recipe_plan', recipe_ref, {'recipe': recipe, 'recipeDigest': recipe_ref,
            'sourceRef': source, 'coverageRef': coverage, 'driver': driver_binding, 'generation': '1'})
        self.state.put('sealed_input', source, {'status': 'sealed'})
        self.workflow(sourceDigest=source, coverageDigest=coverage, driverDigest=driver, generation='1')
        value = ExecutionEngine(self.state, repo_root=self.root, build_root=build_root).claim(
            source=source, recipe=recipe_ref, driver=driver_binding, consumer_id=self.key)
        eid = value['executionId']
        workflow = self.state.get('workflow', self.key)
        self.state.put('workflow', self.key, {**workflow['payload'], 'executionId': eid})
        execution = self.state.get('execution', eid)
        self.state.put('execution', eid, {**execution['payload'], 'status': 'running', 'refs': 0, 'consumers': []})
        consumer = self.state.get('execution_consumer', f'{eid}:{self.key}')
        self.state.put('execution_consumer', consumer['key'], {**consumer['payload'], 'status': 'released'})
        self.state.put('execution_host', eid, {'status': 'running', 'hostBirthToken': '100'})
        self.state.put('execution_launch', eid, {'status': 'started', 'launcherBirthToken': '100'})
        self.state.put('native_job', 'job', {'executionId': eid, 'owner': eid, 'birthToken': '150', 'status': 'running'})
        with patch('tools.jenkins.processes.boot.windows_boot_filetime', return_value=50):
            result = self.recover()
            self.assertEqual('reconciling', result['status'])
            self.assertNotEqual(self.original, self.source.read_bytes())
            self.assertEqual([], self.state.list('boot_termination'))
        with patch('tools.jenkins.processes.boot.windows_boot_filetime', return_value=200):
            result = self.recover()
            self.assertEqual('complete', result['status'])
            self.assertEqual(self.original, self.source.read_bytes())
            self.assertEqual('failed', self.state.get('execution', eid)['payload']['status'])
            self.assertFalse(self.state.get('boot_termination', eid)['payload']['artifactPublicationAllowed'])
            self.assertIsNone(self.state.get('native_job', 'job')['payload'].get('completeProof'))
            self.assertEqual([], self.state.list('acceptance'))

    def test_legacy_orphan_identity_stays_blocked_and_retains_holds_across_boot_recovery(self):
        self.apply()
        self.workflow()
        eid = 'not-launched'
        self.state.put('resource_policy', 'default', {'cpuBudget': 1, 'memoryBudget': 1, 'diskBudget': 1})
        execution = self.state.put('execution', eid, {'status': 'running', 'refs': 0, 'consumers': [],
            'reservationKey': 'legacy-hold', 'preparationKey': 'legacy-preparation'})
        self.state.put('execution_host', eid, {'status': 'running', 'hostBirthToken': '100'})
        self.state.put('execution_launch', eid, {'status': 'started', 'launcherBirthToken': '100'})
        self.state.put('native_job', 'job', {'executionId': eid, 'owner': eid, 'birthToken': '150', 'status': 'running'})
        self.state.put('resource_reservation', 'legacy-hold', {'status': 'active', 'owner': eid,
            'cpu': 1, 'memoryBytes': 1, 'diskBytes': 1, 'heavyWriter': True})
        self.state.put('pool_writer_hold', 'legacy-preparation', {'status': 'active', 'owner': eid,
            'reservationKey': 'legacy-hold'})
        source, claims, operation = self.source.read_bytes(), self.state.list('path_claims'), self.state.get_operation('patch')
        reservations, writers, jobs = (self.state.list(domain) for domain in
                                      ('resource_reservation', 'pool_writer_hold', 'native_job'))
        for boot in (50, 200):
            with self.subTest(boot=boot), patch('tools.jenkins.processes.boot.windows_boot_filetime', return_value=boot):
                result = self.recover()
            self.assertEqual('blocked', result['status'])
            self.assertEqual('execution_identity_corrupt', result['recovery']['reasonCode'])
            self.assertEqual(execution, self.state.get('execution', eid))
            self.assertEqual(source, self.source.read_bytes())
            self.assertNotEqual(self.original, source)
            self.assertEqual(claims, self.state.list('path_claims'))
            self.assertEqual(operation, self.state.get_operation('patch'))
            self.assertEqual(reservations, self.state.list('resource_reservation'))
            self.assertEqual(writers, self.state.list('pool_writer_hold'))
            self.assertEqual(jobs, self.state.list('native_job'))
            self.assertEqual([], self.state.list('acceptance'))
            self.assertEqual([], self.state.list('artifact_bundle'))
            if boot == 50:
                self.assertEqual([], self.state.list('boot_termination'))
            else:
                self.assertFalse(self.state.get('boot_termination', eid)['payload']['artifactPublicationAllowed'])

    def test_accepted_source_is_preserved_after_later_stage_failure(self):
        self.apply()
        self.workflow(status="accepted", gitCommitSha="committed")
        self.assertEqual("accepted-source-preserved", self.recover()["compensation"])
        self.assertNotEqual(self.original, self.source.read_bytes())

    def test_corrupt_original_object_is_rejected(self):
        self.apply()
        self.workflow()
        (self.store / "inputs" / "objects" / self.hash).write_bytes(b"corrupt")
        self.assertEqual("blocked", self.recover()["status"])
        self.assertNotEqual(self.original, self.source.read_bytes())

    def test_duplicate_patch_does_not_overwrite_later_edit(self):
        first = self.apply()
        self.source.write_bytes(b"foreign\n")
        self.assertEqual(first, self.apply())
        self.assertEqual(b"foreign\n", self.source.read_bytes())

    def prepared_source(self, kind='already-applied-patch', *, modified=False, status='complete'):
        reference = 'prepared-' + kind
        operation = self.state.put_operation('source_preparation', reference, {
            'identity': self.identity, 'repositoryRoot': str(self.root),
            'preparationKind': kind,
        })
        self.state.transition_operation(reference, operation['status'], status, {
            'sourceDigest': 'source', 'ownedPaths': ['src/lib.rs'],
            'canonicalSourceModified': modified,
        })
        self.workflow(patchOperationRefs=[], sourcePreparationRef=reference,
                      sourceManifest={'preparationKind': kind}, ownedPaths=['src/lib.rs'])
        return reference

    def test_failed_readonly_preparations_preserve_later_foreign_edit_and_claim(self):
        for kind in ('current-source', 'already-applied-patch'):
            with self.subTest(kind=kind):
                self.prepared_source(kind)
                self.source.write_bytes(b'foreign editing after preparation\n')
                self.state.put('path_claims', 'foreign', {
                    'repositoryId': 'repo', 'owner': 'foreign', 'path': 'src/lib.rs',
                })
                claims = self.state.list('path_claims')
                with patch('tools.jenkins.workflow.recovery._atomic_write') as write, \
                     patch('tools.jenkins.workflow.recovery.subprocess.run') as git:
                    result = self.recover()
                self.assertEqual('complete', result['status'])
                self.assertEqual('failed', result['workflowStatus'])
                self.assertEqual('coordinator-preparation-preserved', result['recovery']['kind'])
                self.assertFalse(result['recovery']['canonicalSourceModified'])
                self.assertEqual([], result['recovery']['operations'])
                self.assertEqual(b'foreign editing after preparation\n', self.source.read_bytes())
                self.assertEqual(claims, self.state.list('path_claims'))
                write.assert_not_called()
                git.assert_not_called()
                self.source.write_bytes(b'next foreign editing\n')
                self.assertEqual('complete', self.recover()['status'])
                self.assertEqual(b'next foreign editing\n', self.source.read_bytes())

    def test_cancelled_prepared_consumer_does_not_claim_shared_native_termination(self):
        self.prepared_source()
        current = self.state.get('workflow', self.key)
        self.state.put('workflow', self.key, {**current['payload'], 'consumerCancelled': True})
        self.state.put('execution', 'not-launched', {'status': 'running', 'refs': 1, 'consumers': ['other-flow']})
        self.state.put('resource_policy', 'default', {'cpuBudget': 1, 'memoryBudget': 1, 'diskBudget': 1})
        self.state.put('execution_consumer', 'not-launched:' + self.key, {'status': 'released'})
        self.state.put('native_job', 'shared-running', {'executionId': 'not-launched', 'status': 'running'})
        with patch('tools.jenkins.processes.boot.recover_previous_boot', return_value=None), \
             patch('tools.jenkins.workflow.execution.ExecutionEngine.observe'), \
             patch('tools.jenkins.workflow.execution.ExecutionEngine.cancel') as cancel, \
             patch('tools.jenkins.workflow.recovery._termination') as terminate:
            result = self.recover()
        self.assertEqual('complete', result['status'])
        self.assertEqual('consumer-reference-released', result['recovery']['kind'])
        self.assertIs(False, result['recovery']['nativeTerminationAccepted'])
        self.assertIs(False, result['recovery']['canonicalSourceModified'])
        self.assertNotIn('terminationEvidence', result['recovery'])
        self.assertEqual('running', self.state.get('execution', 'not-launched')['payload']['status'])
        self.assertEqual('running', self.state.get('native_job', 'shared-running')['payload']['status'])
        self.assertEqual(self.original, self.source.read_bytes())
        cancel.assert_not_called()
        terminate.assert_not_called()

    def test_readonly_preparation_cannot_close_with_unproven_native_child(self):
        self.prepared_source()
        self.state.put('native_job', 'unknown', {
            'executionId': 'not-launched', 'status': 'running',
        })
        result = self.recover()
        self.assertEqual('reconciling', result['status'])
        self.assertEqual('native_termination_unproven', result['recovery']['reasonCode'])
        self.assertEqual(self.original, self.source.read_bytes())

    def test_write_or_partial_preparation_cannot_claim_readonly_recovery(self):
        for modified, phase in ((True, 'complete'), (False, 'prepared')):
            with self.subTest(modified=modified, phase=phase):
                # Use distinct original operation identities in this fixture.
                kind = 'current-source' if modified else 'already-applied-patch'
                self.prepared_source(kind, modified=modified, status=phase)
                result = self.recover()
                self.assertEqual('blocked', result['status'])
                self.assertEqual('source_preparation_mismatch', result['recovery']['reasonCode'])
                self.assertEqual(self.original, self.source.read_bytes())

    def test_foreign_preparation_identity_cannot_close_this_failed_flow(self):
        self.prepared_source()
        record = self.state.get('workflow', self.key)
        self.state.put('workflow', self.key, {
            **record['payload'], 'identity': {**self.identity, 'sessionId': 'foreign'},
        }, expected_version=record['version'])
        result = self.recover()
        self.assertEqual('blocked', result['status'])
        self.assertEqual('source_preparation_mismatch', result['recovery']['reasonCode'])
        self.assertEqual(self.original, self.source.read_bytes())


if __name__ == "__main__":
    unittest.main()
