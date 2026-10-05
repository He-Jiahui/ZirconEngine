from __future__ import annotations

import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.patches import register_patch_intent, prepare_patch, reconcile_patch, submit_patch
from tools.jenkins.resources.paths import repository_build_root
from tools.jenkins.source import _atomic_write, repository_id
from tools.jenkins.state import State
from tools.jenkins.state.locks import process_lock
from tools.jenkins.workflow.handler import handle as workflow_handle

DEFAULT_BUILD_ROOT = repository_build_root()


def checksum(text):
    return hashlib.sha256(text.encode('utf-8')).hexdigest()


def change(path, before, after):
    return (f'diff --git a/{path} b/{path}\n--- a/{path}\n+++ b/{path}\n'
            f'@@ -1 +1 @@\n-{before.rstrip()}\n+{after.rstrip()}\n')


class PatchIntakeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        (self.root / 'src').mkdir()
        (self.root / 'src/lib.rs').write_bytes(b'fn value() -> u32 { 1 }\n')
        (self.root / 'Cargo.toml').write_bytes(b'[package]\nname="patch-fixture"\nversion="0.1.0"\n')
        self.state = State(self.root / '.jenkins/state/support.sqlite3')
        self.identity = dict(repositoryId='repository', sessionId='session', requestId='request',
                             attemptId='attempt', generation=1, stageImplementationDigest='d' * 64)
        self.state.authorize_session('repository', 'session', 'user', ['src', 'Cargo.toml'],
                                     ['implementation', 'validation'], {'source': 'user', 'task': 'incremental patch'})
        self.before = 'fn value() -> u32 { 1 }\n'
        self.after = 'fn value() -> u32 { 2 }\n'
        self.payload = dict(identity=self.identity, unifiedPatch=change('src/lib.rs', self.before, self.after),
                            beforeHashes={'src/lib.rs': checksum(self.before)},
                            coverage={'template': 'module_unit'}, declaredDependencies=['Cargo.toml'],
                            buildRoot=str(DEFAULT_BUILD_ROOT))

    def tearDown(self):
        self.temp.cleanup()

    def register(self, payload=None):
        return register_patch_intent(self.state, self.root, payload or self.payload,
                                     trusted_driver_digest='d' * 64)

    def prepare(self, ref):
        return prepare_patch(self.state, self.root,
                             {'identity': self.identity, 'patchRequestRef': ref,
                              'buildRoot': str(DEFAULT_BUILD_ROOT)}, trusted_driver_digest='d' * 64)

    def test_intake_does_not_apply_and_prepare_registers_final_sealed_request_once(self):
        intake = self.register()
        ref = intake['patchRequestRef']
        self.assertEqual(self.before.encode(), (self.root / 'src/lib.rs').read_bytes())
        self.assertIsNone(self.state.get_request('repository', 'session', 'request'))
        prepared = self.prepare(ref)
        self.assertEqual('prepared', prepared['status'])
        self.assertEqual(self.after.encode(), (self.root / 'src/lib.rs').read_bytes())
        operation = self.state.get_operation(prepared['patchOperationRef'])
        self.assertEqual('complete', operation['status'])
        self.assertEqual(checksum(self.after), operation['result']['afterHashes']['src/lib.rs'])
        request = self.state.get_request('repository', 'session', 'request')
        self.assertEqual(prepared['sealedInputRef'], request['payload']['sourceInputDigest'])
        self.assertEqual(ref, request['payload']['patchRequestRef'])
        self.assertEqual(prepared, self.prepare(ref))
        self.assertEqual(1, len(self.state.list('sealed_input')))

    def test_duplicate_intake_after_application_keeps_registered_content_identity(self):
        first = self.register()
        self.prepare(first['patchRequestRef'])
        self.assertEqual(first['patchRequestRef'], self.register()['patchRequestRef'])
        other = {**self.payload, 'unifiedPatch': change('src/lib.rs', self.before, 'fn value() -> u32 { 3 }\n')}
        with self.assertRaises(JenkinsError) as caught:
            self.register(other)
        self.assertEqual('patch_request_conflict', caught.exception.code)

    def test_foreign_edit_and_changed_dependency_block_before_application(self):
        ref = self.register()['patchRequestRef']
        (self.root / 'Cargo.toml').write_text('foreign\n', encoding='utf-8')
        with self.assertRaises(JenkinsError) as caught:
            self.prepare(ref)
        self.assertEqual('patch_dependency_changed', caught.exception.code)
        self.assertEqual(self.before.encode(), (self.root / 'src/lib.rs').read_bytes())
        self.assertIsNone(self.state.get_request('repository', 'session', 'request'))

    def test_shared_checkout_lock_excludes_second_writer(self):
        ref = self.register()['patchRequestRef']
        lock = self.root / '.jenkins/state/patch-locks' / (repository_id(self.root) + '.lock')
        with process_lock(lock):
            with self.assertRaises(JenkinsError) as caught:
                self.prepare(ref)
        self.assertEqual('patch_checkout_busy', caught.exception.code)

    def test_formal_reconcile_rejects_raw_driver_fallback(self):
        marker = self.root / '.jenkins/state/deployment/driver.json'
        marker.parent.mkdir(parents=True, exist_ok=True)
        marker.write_text('{"driverDigest":"old"}', encoding='utf-8')
        payload = {'identity': self.identity, 'patchRequestRef': 'missing-patch',
                   'driverDigest': 'd' * 64, 'stageImplementationDigest': 'd' * 64,
                   'buildRoot': str(DEFAULT_BUILD_ROOT)}
        with self.assertRaises(JenkinsError) as caught:
            workflow_handle('reconcile-patch', payload, self.state, self.root)
        self.assertIn(caught.exception.code, {'driver_binding_missing', 'active_driver_unavailable',
                                               'driver_context_mismatch'})
        self.assertEqual(self.before.encode(), (self.root / 'src/lib.rs').read_bytes())

    def test_wrong_driver_or_owner_cannot_apply_registered_patch(self):
        ref = self.register()['patchRequestRef']
        for field, value in [('attemptId', 'other'), ('generation', 2), ('stageImplementationDigest', 'e' * 64)]:
            payload = {'identity': {**self.identity, field: value}, 'patchRequestRef': ref}
            with self.assertRaises(JenkinsError):
                prepare_patch(self.state, self.root, payload, trusted_driver_digest='d' * 64)
        self.assertEqual(self.before.encode(), (self.root / 'src/lib.rs').read_bytes())

    def test_aborted_partial_patch_is_not_sealed_and_compensation_is_hash_guarded(self):
        (self.root / 'src/other.rs').write_bytes(b'fn other() {}\n')
        second = change('src/other.rs', 'fn other() {}\n', 'fn other() { value(); }\n')
        payload = {**self.payload, 'unifiedPatch': self.payload['unifiedPatch'] + second,
                   'beforeHashes': {**self.payload['beforeHashes'], 'src/other.rs': checksum('fn other() {}\n')}}
        ref = self.register(payload)['patchRequestRef']
        calls = 0

        def fail_second(path, data):
            nonlocal calls
            if path.is_relative_to(self.root / 'src'):
                calls += 1
                if calls == 2:
                    raise OSError('interrupted patch write')
            return _atomic_write(path, data)

        with patch('tools.jenkins.source._atomic_write', side_effect=fail_second):
            with self.assertRaises(OSError):
                self.prepare(ref)
        self.assertEqual([], self.state.list('sealed_input'))
        self.assertIsNone(self.state.get_request('repository', 'session', 'request'))
        recovered = reconcile_patch(self.state, self.root, {'identity': self.identity, 'patchRequestRef': ref})
        self.assertEqual('failed', recovered['status'])
        self.assertEqual('complete', recovered['recovery']['status'])
        self.assertEqual(self.before.encode(), (self.root / 'src/lib.rs').read_bytes())
        self.assertEqual(b'fn other() {}\n', (self.root / 'src/other.rs').read_bytes())
        self.assertEqual([], self.state.list('path_claims'))
        with self.assertRaises(JenkinsError):
            self.prepare(ref)

    def test_foreign_edit_after_patch_failure_prevents_compensation(self):
        ref = self.register()['patchRequestRef']
        with patch('tools.jenkins.patches.source_handle', side_effect=OSError('seal interrupted')):
            with self.assertRaises(OSError):
                self.prepare(ref)
        (self.root / 'src/lib.rs').write_bytes(b'foreign change\n')
        recovered = reconcile_patch(self.state, self.root, {'identity': self.identity, 'patchRequestRef': ref})
        self.assertEqual('blocked', recovered['status'])
        self.assertEqual('foreign_edit_during_compensation', recovered['recovery']['reasonCode'])
        self.assertEqual(b'foreign change\n', (self.root / 'src/lib.rs').read_bytes())

    def test_active_execution_consumer_blocks_patch_compensation(self):
        ref = self.register()['patchRequestRef']
        with patch('tools.jenkins.patches.source_handle', side_effect=OSError('seal interrupted')):
            with self.assertRaises(OSError):
                self.prepare(ref)
        self.state.put('execution_consumer', 'execution:consumer', {
            'executionId': 'execution', 'consumerId': 'consumer',
            'patchOperationId': 'patch-' + ref, 'status': 'active'})
        recovered = reconcile_patch(self.state, self.root,
                                    {'identity': self.identity, 'patchRequestRef': ref})
        self.assertEqual('blocked', recovered['status'])
        self.assertEqual('shared_source_active', recovered['recovery']['reasonCode'])
        self.assertEqual(self.after.encode(), (self.root / 'src/lib.rs').read_bytes())

    def test_active_native_job_blocks_patch_compensation(self):
        ref = self.register()['patchRequestRef']
        with patch('tools.jenkins.patches.source_handle', side_effect=OSError('seal interrupted')):
            with self.assertRaises(OSError):
                self.prepare(ref)
        self.state.put('native_job', 'native-job', {
            'executionId': 'execution', 'patchOperationRef': 'patch-' + ref,
            'status': 'running'})
        recovered = reconcile_patch(self.state, self.root,
                                    {'identity': self.identity, 'patchRequestRef': ref})
        self.assertEqual('blocked', recovered['status'])
        self.assertEqual('shared_source_active', recovered['recovery']['reasonCode'])
        self.assertEqual(self.after.encode(), (self.root / 'src/lib.rs').read_bytes())

    def test_raw_request_dispatch_passes_only_registered_refs_and_reuses_marker(self):
        ref = self.register()['patchRequestRef']
        requests = []

        class Result:
            status = 201
            headers = {'Location': 'http://127.0.0.1:18080/queue/item/1/'}

        def transport(request):
            requests.append(request)
            return Result()

        payload = {'identity': self.identity, 'patchRequestRef': ref}
        first = submit_patch(self.state, self.root, payload, transport=transport)
        second = submit_patch(self.state, self.root, payload, transport=transport)
        self.assertFalse(first['reused'])
        self.assertTrue(second['reused'])
        self.assertEqual(1, len(requests))
        self.assertIn(b'PATCH_REQUEST_REF=', requests[0].data)
        self.assertNotIn(b'unifiedPatch', requests[0].data)
        self.assertNotIn(b'SOURCE_INPUT_DIGEST', requests[0].data)
        self.assertEqual(self.before.encode(), (self.root / 'src/lib.rs').read_bytes())

    def test_patch_dispatch_carries_runtime_operation_fence(self):
        ref = self.register()['patchRequestRef']
        requests = []

        class Result:
            status = 201
            headers = {'Location': 'http://127.0.0.1:18080/queue/item/runtime/'}

        def transport(request):
            requests.append(request)
            return Result()

        with patch('tools.jenkins.patches._runtime_binding', return_value='runtime-1'):
            submit_patch(self.state, self.root,
                         {'identity': self.identity, 'patchRequestRef': ref}, transport=transport)
        self.assertEqual(1, len(requests))
        self.assertIn(b'RUNTIME_OPERATION_ID=runtime-1', requests[0].data)

    def test_patch_dispatch_rejects_runtime_fence_mismatch(self):
        marker = self.root / '.jenkins/state/deployment/driver.json'
        marker.parent.mkdir(parents=True, exist_ok=True)
        marker.write_text('{}', encoding='utf-8')
        with patch('tools.jenkins.deployment.driver.active_driver_binding',
                   return_value={'runtimeOperationId': 'runtime-1', 'digest': 'd' * 64}):
            with self.assertRaises(JenkinsError) as caught:
                from tools.jenkins.patches import _runtime_binding
                _runtime_binding(self.root, self.identity, {'runtimeOperationId': 'runtime-2'})
        self.assertEqual('runtime_binding_mismatch', caught.exception.code)

    def test_old_request_identity_cannot_be_bypassed_by_new_patch_entry(self):
        self.state.submit_request({**self.identity, 'sourceInputDigest': 'a' * 64})
        with self.assertRaises(JenkinsError) as caught:
            self.register()
        self.assertEqual('patch_request_conflict', caught.exception.code)

    def test_attempt_owner_cannot_be_adopted_by_a_different_request(self):
        self.register()
        other = {**self.payload, 'identity': {**self.identity, 'requestId': 'other-request'}}
        with self.assertRaises(JenkinsError) as caught:
            self.register(other)
        self.assertEqual('patch_owner_conflict', caught.exception.code)

    def test_request_created_after_intake_blocks_application_before_source_write(self):
        ref = self.register()['patchRequestRef']
        self.state.submit_request({**self.identity, 'sourceInputDigest': 'a' * 64})
        with self.assertRaises(JenkinsError) as caught:
            self.prepare(ref)
        self.assertEqual('patch_request_conflict', caught.exception.code)
        self.assertEqual(self.before.encode(), (self.root / 'src/lib.rs').read_bytes())

    def test_prepared_retry_keeps_sealed_identity_after_dependency_changes(self):
        ref = self.register()['patchRequestRef']
        prepared = self.prepare(ref)
        (self.root / 'Cargo.toml').write_bytes(b'foreign later dependency\n')
        self.assertEqual(prepared, self.prepare(ref))
        self.assertEqual(1, len(self.state.list('sealed_input')))
        # Preparation may have returned to a CPS caller; no-workflow alone
        # cannot authorize cancelling it during that dispatch window.
        recovery = reconcile_patch(self.state, self.root, {'identity': self.identity, 'patchRequestRef': ref})
        self.assertEqual('blocked', recovery['status'])
        self.assertEqual(self.after.encode(), (self.root / 'src/lib.rs').read_bytes())

    def test_cancel_before_prepare_prevents_late_patch_application(self):
        ref = self.register()['patchRequestRef']
        recovered = reconcile_patch(self.state, self.root, {'identity': self.identity, 'patchRequestRef': ref})
        self.assertEqual('failed', recovered['status'])
        with self.assertRaises(JenkinsError) as caught:
            self.prepare(ref)
        self.assertEqual('patch_recovery_required', caught.exception.code)
        self.assertEqual(self.before.encode(), (self.root / 'src/lib.rs').read_bytes())

    def test_revoked_authorization_blocks_registered_patch(self):
        ref = self.register()['patchRequestRef']
        self.state.revoke_session('repository', 'session')
        with self.assertRaises(JenkinsError) as caught:
            self.prepare(ref)
        self.assertEqual('action_not_authorized', caught.exception.code)
        self.assertEqual(self.before.encode(), (self.root / 'src/lib.rs').read_bytes())

    def test_control_handler_prepares_raw_patch_before_full_source_identity_exists(self):
        from tools.jenkins.cli import dispatch
        payload = {**self.payload, 'ownedPaths': ['src/lib.rs']}
        registered = dispatch('patch', 'register', payload, self.state, self.root)
        prepared = dispatch('flow', 'prepare-patch', {'identity': self.identity,
                    'patchRequestRef': registered['patchRequestRef']}, self.state, self.root)
        self.assertEqual('prepared', prepared['status'])
        result = dispatch('flow', 'register-flow', {'identity': prepared['identity'],
                 'sealedInputRef': prepared['sealedInputRef'], 'patchOperationRef': prepared['patchOperationRef'],
                 'buildRoot': prepared['buildRoot']}, self.state, self.root)
        self.assertEqual('pending', result['status'])


if __name__ == '__main__':
    unittest.main()
