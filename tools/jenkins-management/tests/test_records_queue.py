"""Focused intake contract tests; no compiler, source snapshot, or inbox writes."""
import importlib.util
from pathlib import Path
import unittest
from contextlib import nullcontext
import json
from unittest.mock import patch, Mock

source = Path(__file__).absolute().parents[1] / 'records_queue.py'
spec = importlib.util.spec_from_file_location('records_queue_tests_subject', source)
queue = importlib.util.module_from_spec(spec)
spec.loader.exec_module(queue)


class IntakeTests(unittest.TestCase):
    def inventory(self):
        return {'schemaVersion': 1,
                'authorization': 'user: every 30 minutes; enqueue all tasks, archive evidence, then empty inbox',
                'tasks': [{'taskId': 'one', 'kind': 'task', 'originalIdentity': {'sessionId': 'original'},
                           'pendingRequirements': ['source-ready'], 'messages': [
                               {'path': 'E:/Git/ZirconEngine/.jenkins/records/one.md', 'sha256': '1' * 64}]}]}

    def test_rejects_duplicate_message_in_different_task(self):
        value = self.inventory()
        value['tasks'].append({**value['tasks'][0], 'taskId': 'two'})
        with self.assertRaisesRegex(ValueError, 'unique exact'):
            queue.validate_inventory('E:/Git/ZirconEngine', value)

    def test_rejects_path_traversal_and_outside_inbox(self):
        for path in ('E:/Git/ZirconEngine/.jenkins/records/../state/one.md',
                     'E:/Git/ZirconEngine/.jenkins/state/one.md'):
            value = self.inventory()
            value['tasks'][0]['messages'][0]['path'] = path
            with self.assertRaises(ValueError):
                queue.validate_inventory('E:/Git/ZirconEngine', value)

    def test_rejects_execution_with_other_session_or_external_action(self):
        for payload in ({'action': 'commit-worktree', 'sessionId': 'original'},
                        {'action': 'validate', 'sessionId': 'borrowed'}):
            value = self.inventory()
            value['tasks'][0]['formalRequests'] = [payload]
            with self.assertRaises(ValueError):
                queue.validate_inventory('E:/Git/ZirconEngine', value)

    def test_changed_inbox_is_never_archived_or_removed(self):
        message = {'path': 'E:/Git/ZirconEngine/.jenkins/records/one.md', 'bytes': 7,
                   'mtimeNs': 1, 'sha256': '1' * 64}
        with patch.object(queue, 'fingerprint', return_value=({**message, 'sha256': '2' * 64}, b'changed')):
            with patch.object(queue, 'write_once') as write:
                # Use an existing file only as a stand-in for the existence check.
                with patch.object(Path, 'exists', return_value=True), patch.object(queue, 'plain', side_effect=Path):
                    with self.assertRaisesRegex(ValueError, 'changed'):
                        queue.archive_messages(Path('E:/Git/ZirconEngine'), Path('E:/Git/ZirconEngine/.jenkins/state/records-queue'),
                                               {'messages': [message]}, '1' * 64)
                write.assert_not_called()

    def test_lost_acknowledgement_never_causes_second_submission(self):
        value = self.inventory()
        files = {}
        api = Mock()
        api.record.return_value = None
        manager = Mock()
        manager._request.return_value = (200, b'{"crumbRequestField":"crumb","crumb":"test"}')
        manager._request_with_headers.side_effect = TimeoutError('lost acknowledgement')
        context = {'repo': Path('E:/Git/ZirconEngine'), 'manager': manager}
        installed = {'driverDigest': '1' * 64, 'runtimeOperationId': 'original-runtime'}
        def read(path):
            return (json.loads(files[str(path)]) if str(path) in files else value), {}
        def save(path, raw):
            if str(path) in files:
                self.assertEqual(files[str(path)], raw)
            files[str(path)] = raw
        def atomic(path, record):
            files[str(path)] = queue.encoded(record) + b'\n'
        with patch.object(queue, 'binding', return_value=(api, context, installed)), \
             patch.object(queue, 'read_json', side_effect=read), \
             patch.object(queue, 'write_once', side_effect=save), \
             patch.object(Path, 'exists', side_effect=lambda: False):
            # Replace only this client's existence observation with the memory ledger.
            with patch.object(Path, 'exists', new=lambda path: str(path) in files), \
                 patch.object(queue, 'plain', side_effect=Path), \
                 patch('tools.jenkins.state.atomic.atomic_json', side_effect=atomic), \
                 patch('tools.jenkins.state.locks.process_lock', return_value=nullcontext()):
                first = queue.submit(context['repo'], Path('inventory.json'))
                second = queue.submit(context['repo'], Path('inventory.json'))
        self.assertEqual(first['receipts'][0]['phase'], 'delivery_unknown')
        self.assertEqual(first, second)
        manager._request_with_headers.assert_called_once()

    def cleanup_case(self, deleted=True):
        root = Path('E:/Git/ZirconEngine/.jenkins/state/records-queue')
        message = {'path': 'E:\\Git\\ZirconEngine\\.jenkins\\records\\one.md', 'bytes': 7,
                   'sha256': '1' * 64, 'archivePath': str(root / 'archive/one.md')}
        receipt = {'success': False, 'taskId': 'one', 'inventoryDigest': '2' * 64,
                   'deletedFiles': [{k: message[k] for k in ('path', 'bytes', 'sha256')}] if deleted else [],
                   'error': 'receipt accounting failure'}
        return root, message, receipt

    def test_partial_native_deletion_cannot_be_marked_complete(self):
        root, message, receipt = self.cleanup_case(deleted=False)
        with patch.object(queue, 'plain', side_effect=Path), \
             patch.object(Path, 'exists', return_value=True), \
             patch.object(queue, 'read_json', return_value=(receipt, {})), \
             patch.object(queue, 'write_once') as write:
            with self.assertRaisesRegex(ValueError, 'incomplete'):
                queue.cleanup({}, {}, root, 'one', '2' * 64, [message])
            write.assert_not_called()

    def test_missing_archive_blocks_receipt_reconciliation(self):
        root, message, receipt = self.cleanup_case()
        with patch.object(queue, 'plain', side_effect=Path), \
             patch.object(Path, 'exists', new=lambda path: path.name == 'one.receipt.json'), \
             patch.object(queue, 'read_json', return_value=(receipt, {})), \
             patch.object(queue, 'fingerprint', side_effect=FileNotFoundError('archive missing')), \
             patch.object(queue, 'write_once') as write:
            with self.assertRaises(FileNotFoundError):
                queue.cleanup({}, {}, root, 'one', '2' * 64, [message])
            write.assert_not_called()

    def test_completed_native_deletion_is_reconciled_without_repeat_delete(self):
        root, message, receipt = self.cleanup_case()
        archive_pin = {'path': message['archivePath'], 'bytes': message['bytes'], 'sha256': message['sha256']}
        with patch.object(queue, 'plain', side_effect=Path), \
             patch.object(Path, 'exists', new=lambda path: path.name == 'one.receipt.json'), \
             patch.object(queue, 'read_json', return_value=(receipt, {'sha256': '3' * 64})), \
             patch.object(queue, 'fingerprint', return_value=(archive_pin, b'archive')), \
             patch.dict(queue.os.environ, {'BUILD_URL': 'original-reconciliation-build'}), \
             patch.object(queue, 'write_once') as write, \
             patch.object(queue.subprocess, 'run') as execute:
            result = queue.cleanup({}, {}, root, 'one', '2' * 64, [message])
            self.assertTrue(result['success'])
            self.assertTrue(result['reconciledAfterReceiptFailure'])
            self.assertEqual(result['deletedBytes'], 7)
            write.assert_called_once()
            execute.assert_not_called()


if __name__ == '__main__':
    unittest.main()
