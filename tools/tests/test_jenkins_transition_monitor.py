"""Contract checks for the unattended audit; all fixtures stay in memory."""
import importlib.util
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

DEV = Path(__file__).absolute().parents[1] / 'dev'
sys.path.insert(0, str(DEV))
import jenkins_transition_monitor as monitor


class TransitionAuditTests(unittest.TestCase):
    def report(self):
        return {'jenkins': {'healthy': True}, 'git': {'stableHead': True},
                'state': {'complete': True, 'scheduledMaintenance': {'enabled': True}}, 'disks': []}

    def test_healthy_runtime_does_not_accept_merges(self):
        value = self.report()
        self.assertEqual(monitor.findings(value), [])
        self.assertEqual(monitor.issue_delta([], []), {'notify': False, 'newOrChanged': [], 'resolved': []})

    def test_git_lock_is_a_blocker_even_when_service_healthy(self):
        value = self.report()
        value['git']['indexLock'] = {'exists': True, 'bytes': 0}
        self.assertEqual(monitor.findings(value)[0]['code'], 'index_lock')

    def test_disk_size_never_grants_deletion(self):
        value = self.report()
        value['disks'] = [{'drive': 'E', 'belowBuildReserve': True}]
        self.assertIn('no deletion is authorized', monitor.findings(value)[0]['message'])

    def test_identical_blockers_stay_quiet(self):
        issue = monitor.findings({**self.report(), 'jenkins': {'healthy': False}})
        self.assertFalse(monitor.issue_delta(issue, issue)['notify'])

    def test_recovery_is_reported_once(self):
        issue = monitor.findings({**self.report(), 'jenkins': {'healthy': False}})
        delta = monitor.issue_delta(issue, [])
        self.assertTrue(delta['notify'])
        self.assertEqual(delta['resolved'], issue)

    def test_missing_state_and_conflict_remain_open(self):
        value = self.report()
        value['state'] = {'complete': False}
        value['git']['unmerged'] = ['owned.rs']
        self.assertTrue({'state_unavailable', 'unmerged'}.issubset({x['code'] for x in monitor.findings(value)}))

    def test_permission_grant_cannot_redirect_output_to_source(self):
        root = Path('E:/Git/ZirconEngine')
        with patch.object(monitor.inventory, 'physical_literal', side_effect=lambda p: p):
            with self.assertRaises(ValueError):
                monitor.report_root(root, {str(root / 'tools'): 'write'}, root / 'tools')

    def test_allowed_output_still_requires_write_grant(self):
        with patch.object(monitor.inventory, 'physical_literal', side_effect=lambda p: p):
            with self.assertRaises(ValueError):
                monitor.report_root(Path('E:/Git/ZirconEngine'), {})

    def test_links_are_rejected_before_any_output(self):
        with patch.object(monitor.inventory, 'physical_literal', side_effect=ValueError('alias')):
            with self.assertRaises(ValueError):
                monitor.report_root(Path('E:/Git/ZirconEngine'), {'E:/cargo-targets/zircon-local': 'write'})


if __name__ == '__main__':
    unittest.main()
