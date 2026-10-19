"""Regression for candidate recovery, queue fairness and predecessor isolation."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest

SOURCE = Path(__file__).parents[1] / 'merge_queue_recovery.py'
if not SOURCE.exists():
    SOURCE = Path(__file__).with_name('merge_queue_recovery.py')
spec = importlib.util.spec_from_file_location('merge_queue_recovery_test_subject', SOURCE)
mq = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mq)


def record(op, attempt=0):
    return {'status': 'committed', 'sourceCommit': op, 'committedAt': int(op),
            'request': {'operationId': op, 'paths': ['own.py']},
            'periodicIntegration': {'build': attempt}}


class FairnessTests(unittest.TestCase):
    def test_twenty_one_requests_are_all_selected_in_three_rounds(self):
        records = {str(n): record(str(n)) for n in range(21)}
        seen = set()
        for build in range(1, 4):
            for op in mq.select_operations(records):
                seen.add(op)
                records[op]['periodicIntegration']['build'] = build
        self.assertEqual(seen, set(records))

    def test_blocked_oldest_does_not_starve_new_requests(self):
        records = {str(n): record(str(n), 99 if n < 8 else 0) for n in range(21)}
        self.assertTrue(all(int(op) >= 8 for op in mq.select_operations(records)))

    def test_dependencies_are_reordered_inside_the_fair_batch(self):
        records = {'1': record('1'), '2': record('2')}
        records['1']['periodicIntegration']['prerequisiteOperations'] = ['2']
        self.assertEqual(mq.select_operations(records), ['2', '1'])

    def test_unknown_dependency_never_expands_scope_or_batch(self):
        records = {'1': record('1'), '2': record('2', 10), '3': record('3')}
        records['1']['periodicIntegration']['prerequisiteOperations'] = ['2', 'unknown']
        self.assertEqual(set(mq.select_operations(records, limit=2)), {'1', '3'})

    def test_active_candidate_owner_is_serviced_before_waiters(self):
        records = {str(n): record(str(n)) for n in range(21)}
        self.assertEqual(mq.select_operations(records, active_owner='20')[0], '20')

    def test_already_contained_requests_do_not_consume_batch(self):
        rec = record('1'); rec['integrationStatus'] = 'awaiting-main-sync'
        self.assertEqual(mq.select_operations({'1': rec}), [])


class ScopeTests(unittest.TestCase):
    def test_known_history_waits_for_its_original_predecessor(self):
        with self.assertRaises(mq.QueueBlocked) as caught:
            mq.require_scoped_history(record('1'), ['own.py', 'previous.py'], ['own.py'], ['previous-op'])
        self.assertEqual(caught.exception.status, 'waiting-prerequisite')
        self.assertEqual(caught.exception.prerequisites, ['previous-op'])

    def test_foreign_changes_in_own_commit_stay_blocked(self):
        with self.assertRaises(mq.QueueBlocked) as caught:
            mq.require_scoped_history(record('1'), ['foreign.py'], ['foreign.py'], ['previous-op'])
        self.assertEqual(caught.exception.status, 'scope-blocked')

    def test_unknown_history_cannot_be_published(self):
        with self.assertRaises(mq.QueueBlocked) as caught:
            mq.require_scoped_history(record('1'), ['own.py', 'foreign.py'], ['own.py'], [])
        self.assertEqual(caught.exception.status, 'scope-blocked')

    def test_scoped_successor_proceeds_after_prerequisite_is_on_main(self):
        mq.require_scoped_history(record('1'), ['own.py'], ['own.py'], [])

    def test_case_aliases_cannot_expand_scope(self):
        with self.assertRaises(mq.QueueBlocked):
            mq.require_scoped_history(record('1'), ['own.py'], ['own.py', 'OWN.py'], [])

    def test_compiler_recipes_are_preserved_as_deferred_not_passed(self):
        original = {'validation': [{'kind': 'cargo', 'arguments': ['check', '--all-targets']}, {'kind': 'python-static', 'paths': ['a.py']}]}
        before = copy.deepcopy(original)
        selected, deferred = mq.validation_request(original)
        self.assertEqual(original, before)
        self.assertEqual(len(selected['validation']), 1)
        self.assertEqual(deferred, [original['validation'][0]])


class RecoveryTests(unittest.TestCase):
    def setUp(self):
        self.files = {}
        self.root = Path('candidate')
        self.runtime = {'driverDigest': 'driver', 'runtimeOperationId': 'runtime'}
        self.record = {'request': {'operationId': 'op'}, 'sourceCommit': 'source', 'candidateRoot': 'candidate',
                       'candidateHead': 'merged', 'mainHeadBefore': 'main'}
        self.marker = {'path': 'candidate', 'status': 'active', 'operationId': 'op', 'mainHead': 'main'}
        self.facts = {'candidateRoot': 'candidate', 'head': 'merged', 'commonDirectory': 'common',
                      'repositoryGitDirectory': 'common', 'indexLocked': False, 'unfinishedOperations': [], 'dirty': False}
        self.command = {'operationId': 'op', 'operationKind': 'merge', 'candidateRoot': 'candidate', 'cwd': 'candidate',
                        'runtime': self.runtime, 'command': ['git.exe', 'merge', '--no-edit', '--no-gpg-sign', 'source']}
        self.command['executable'] = self.ref('git.exe', b'executable')
        self.native = {'rootExitCode': 0, 'pid': 1, 'creationTime': 'born', 'command': self.command['command'], 'cwd': 'candidate', 'terminal': True}
        self.info = {'nativeComplete': True, 'operationKind': 'merge', 'rootExitCode': 0}
        for field, value in [('commandRef', self.command), ('nativeTerminalRef', self.native), ('startedRef', self.native)]:
            self.info[field] = self.ref(field, json.dumps(value).encode())
        for field in ['stdoutRef', 'stderrRef']:
            self.info[field] = self.ref(field, b'')
        self.record['candidateGit'] = self.info
        self.record['candidateValidation'] = {'status': 'passed', 'head': 'merged', 'runtime': self.runtime,
            'steps': [{'complete': True, 'cwd': 'candidate', 'command': ['check'], 'native': {'terminal': True}}]}

    def ref(self, name, raw):
        self.files[Path(name)] = raw
        return {'path': name, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}

    def prove(self):
        return mq.prove_candidate_terminal(self.record, self.marker, self.facts, self.files.__getitem__, lambda n, c, r: bool(n and n.get('terminal')))

    def test_exact_terminal_candidate_can_be_recovered(self):
        self.assertTrue(self.prove()['nativeEvidenceVerified'])

    def test_wrong_owner_cannot_release_candidate(self):
        self.marker['operationId'] = 'other'
        with self.assertRaises(ValueError): self.prove()

    def test_changed_head_cannot_release_candidate(self):
        self.facts['head'] = 'foreign'
        with self.assertRaises(ValueError): self.prove()

    def test_existing_index_lock_is_preserved(self):
        self.facts['indexLocked'] = True
        with self.assertRaises(ValueError): self.prove()

    def test_dirty_candidate_is_preserved(self):
        self.facts['dirty'] = True
        with self.assertRaises(ValueError): self.prove()

    def test_unfinished_merge_is_preserved(self):
        self.facts['unfinishedOperations'] = ['MERGE_HEAD']
        with self.assertRaises(ValueError): self.prove()

    def test_changed_evidence_hash_is_rejected(self):
        self.files[Path('nativeTerminalRef')] += b' '
        with self.assertRaises(ValueError): self.prove()

    def test_pid_reuse_is_rejected(self):
        changed = {**self.native, 'creationTime': 'other'}
        self.info['startedRef'] = self.ref('startedRef', json.dumps(changed).encode())
        with self.assertRaises(ValueError): self.prove()

    def test_unknown_native_validation_is_preserved(self):
        self.record['candidateValidation']['steps'][0]['native']['terminal'] = False
        with self.assertRaises(ValueError): self.prove()

    def test_pending_preparation_is_preserved(self):
        self.record['candidateValidation']['preparationPending'] = True
        with self.assertRaises(ValueError): self.prove()

    def test_unknown_validation_status_is_preserved(self):
        self.record['candidateValidation']['status'] = 'waiting'
        with self.assertRaises(ValueError): self.prove()

    def test_original_request_is_never_modified(self):
        before = copy.deepcopy(self.record)
        self.prove()
        self.assertEqual(self.record, before)


if __name__ == '__main__':
    unittest.main()
