"""Supplement identity and uncertain transport contracts, with no filesystem writes."""
from contextlib import nullcontext
import copy
import json
from pathlib import Path
import sys
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).absolute().parents[1]))
import records_followup as followup
queue = followup.queue

from test_records_queue import InstallCase, RecordsTransportCase


class FollowupTests(unittest.TestCase):
    def inputs(self):
        original = {'taskId': 'original', 'originalIdentity': {'requestId': 'original'},
                    'intakeAccepted': True, 'done': False, 'inventoryDigest': '1' * 64}
        inventory = {'schemaVersion': 1,
                     'authorization': 'user: every 30 minutes; enqueue all tasks, archive evidence, then empty inbox',
                     'originalInventoryDigest': original['inventoryDigest'],
                     'response': {'taskId': 'original', 'formalAcceptance': False},
                     'tasks': [{'taskId': 'original', 'kind': 'task', 'originalIdentity': original['originalIdentity'],
                                'messages': [{'path': 'E:/Git/ZirconEngine/.jenkins/records/extra.md', 'sha256': '2' * 64}],
                                'pendingRequirements': ['full inventory']} ]}
        return original, inventory, {'repo': Path('E:/Git/ZirconEngine')}

    def test_preserves_original_identity(self):
        original, inventory, context = self.inputs()
        self.assertEqual(followup.validate(context, inventory, original)['taskId'], original['taskId'])

    def test_cannot_borrow_other_task_or_revision(self):
        for field, changed in [('taskId', 'other'), ('originalIdentity', {'requestId': 'other'})]:
            original, inventory, context = self.inputs()
            inventory['tasks'][0][field] = changed
            with self.assertRaises(ValueError):
                followup.validate(context, inventory, original)
        original, inventory, context = self.inputs()
        inventory['originalInventoryDigest'] = '3' * 64
        with self.assertRaises(ValueError):
            followup.validate(context, inventory, original)

    def test_rejects_new_native_trigger_in_supplement(self):
        original, inventory, context = self.inputs()
        inventory['tasks'][0]['formalRequests'] = [{'action': 'validate', 'sessionId': 'borrowed'}]
        with self.assertRaises(ValueError):
            followup.validate(context, inventory, original)

    def test_cannot_turn_pending_response_into_acceptance(self):
        original, inventory, context = self.inputs()
        inventory['response']['formalAcceptance'] = True
        with self.assertRaises(ValueError):
            followup.validate(context, inventory, original)

    def test_changed_preparation_reference_blocks_attachment(self):
        original, inventory, context = self.inputs()
        inventory['documents'] = [{'path': str(context['repo'] / 'docs/tests/schema.json'), 'sha256': '4' * 64}]
        with patch.object(queue, 'fingerprint', return_value=({'sha256': '5' * 64}, b'')):
            with self.assertRaisesRegex(ValueError, 'reference changed'):
                followup.validate(context, inventory, original)

    def test_unknown_delivery_is_never_submitted_again(self):
        original, inventory, _ = self.inputs()
        with RecordsTransportCase(queue, inventory, followup=followup, original=original) as case:
            first = followup.submit(case.repo, case.inventory)
            self.assertEqual(queue.sys.path[0], str(case.driver_root))
            second = followup.submit(case.repo, case.inventory)
            self.assertEqual(first, second)
            self.assertEqual(first['phase'], 'delivery_unknown')
            self.assertEqual(len(case.posts), 1)
            endpoint, parameters = case.posts[0]
            self.assertEqual(endpoint, '/job/zircon-records-followup/buildWithParameters')
            self.assertEqual(parameters, {'FOLLOWUP_DIGEST': [first['followupDigest']],
                                          'TASK_ID': ['original'], 'MODE': ['attach']})
            self.assertEqual(first['originalIdentity'], original['originalIdentity'])
            case.api.capability.assert_not_called()
            with self.assertRaisesRegex(PermissionError, 'OpenProcess'):
                followup.work(case.repo, 'original', first['followupDigest'])
            case.api.capability.assert_called_once_with(case.repo)



class FollowupRuntimeRebindTests(unittest.TestCase):
    def test_default_cross_runtime_refusal_even_when_rendered_script_is_unchanged(self):
        for same_source in (False, True):
            with self.subTest(same_source=same_source):
                case = InstallCase(followup, queue, same_source=same_source)
                with self.assertRaisesRegex(ValueError, '--rebind-runtime'):
                    case.run()
                self.assertEqual(case.events, [])

    def test_explicit_rebind_with_equal_script_still_proves_owner_and_preserves_history(self):
        case = InstallCase(followup, queue, same_source=True)
        result = case.run(rebind_runtime=True)
        self.assertEqual(case.current_source, case.old_source)
        self.assertEqual(case.events, ['history', 'post', 'publish'])
        self.assertEqual(case.history['previous'], case.old)
        self.assertEqual(case.history['previousInstallationSha256'], case.pin)
        self.assertTrue(case.history['runtimeRebound'])
        self.assertEqual(case.history['next'], result)
        self.assertEqual(case.published, result)
        self.assertEqual(result['driverDigest'], case.context['config']['driverDigest'])
        self.assertEqual(result['runtimeOperationId'], case.context['config']['runtimeOperationId'])
        self.assertFalse(result['formalAcceptance'])

    def test_exact_pins_and_prior_source_owner_are_required(self):
        for mismatch in ('installPin', 'jobPin', 'jobOwner', 'repositoryOwner', 'template', 'script', 'sandbox'):
            with self.subTest(mismatch=mismatch):
                case = InstallCase(followup, queue)
                overrides = {}
                if mismatch == 'installPin':
                    overrides['upgrade_from'] = '3' * 64
                elif mismatch == 'jobPin':
                    overrides['expected_job_script_sha256'] = '4' * 64
                elif mismatch == 'jobOwner':
                    case.old['job'] = 'foreign'
                elif mismatch == 'repositoryOwner':
                    case.old['repositoryRoot'] = 'E:/Git/Foreign'
                elif mismatch == 'template':
                    case.raw[case.template_name] += b'foreign-code\n'
                elif mismatch == 'script':
                    case.current_source += '\nforeign-code'
                else:
                    case.sandbox = 'false'
                case.refresh_previous()
                with self.assertRaises(ValueError):
                    case.run(rebind_runtime=True, **overrides)
                self.assertEqual(case.events, [])

    def test_nonterminal_build_or_queued_followup_cannot_rebind(self):
        for queued in (False, True):
            with self.subTest(queued=queued):
                case = InstallCase(followup, queue)
                if queued:
                    case.items = [{'task': {'name': followup.JOB,
                                           'url': case.manager.base_url + '/job/' + followup.JOB + '/'}}]
                else:
                    case.builds = [{'building': False, 'result': None}]
                with self.assertRaises(ValueError):
                    case.run(rebind_runtime=True)
                self.assertEqual(case.events, [])

    def test_changed_runtime_health_and_canonical_source_stop_before_history(self):
        for changed in ('source', 'health', 'runtime'):
            with self.subTest(changed=changed):
                case = InstallCase(followup, queue)
                if changed == 'source':
                    original = case.fingerprint
                    def fingerprint(path):
                        pin, raw = original(path)
                        if Path(path).name == case.script_name and case.reads[case.script_name] > 1:
                            return {'sha256': queue.sha(b'foreign-source')}, b'foreign-source'
                        return pin, raw
                    case.fingerprint = fingerprint
                elif changed == 'health':
                    case.api.capability.side_effect = ValueError('runtime no longer healthy')
                else:
                    other = {**case.context, 'config': {**case.context['config'], 'runtimeOperationId': 'start-other'}}
                    case.api.capability.return_value = other
                with self.assertRaises(ValueError):
                    case.run(rebind_runtime=True)
                self.assertEqual(case.events, [])

    def test_post_verification_failure_preserves_old_metadata(self):
        case = InstallCase(followup, queue)
        case.post_reconciles = False
        with self.assertRaisesRegex(ValueError, 'reconciliation'):
            case.run(rebind_runtime=True)
        self.assertEqual(case.events, ['history', 'post'])
        self.assertIsNone(case.published)


if __name__ == '__main__':
    unittest.main()
