from pathlib import Path
import unittest
from unittest.mock import patch
from unittest.mock import Mock

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.recovery import abandoned_startup_evidence, _verified_startup_containment
from tools.jenkins.deployment.startup_observations import observe_departed_identity
from tools.jenkins.processes.identity import ProcessIdentity


class AbandonedStartupTests(unittest.TestCase):
    def test_control_plane_stop_preserves_restart_component_bindings(self):
        import os
        from tools.jenkins.deployment.host import LifecycleHost
        from tools.jenkins.deployment.recovery import has_terminal_deployment_proofs
        lifecycle = object.__new__(LifecycleHost)
        lifecycle.operation_id = 'control-stop'
        lifecycle.agent_launch_attempted = False
        lifecycle.agent_job = None
        lifecycle._save = Mock()
        proof = {'identity': self.controller, 'complete': True, 'childrenGone': True,
                 'activeProcesses': 0, 'stdoutEof': True, 'stderrEof': True, 'processExitCode': 0}
        lifecycle.job = Mock()
        lifecycle.job.identity.to_dict.return_value = self.controller
        lifecycle.job.process.poll.return_value = None
        lifecycle.job.terminate.return_value.to_dict.return_value = proof
        with patch.dict('os.environ', {'ZIRCON_JENKINS_RECOVERY_CONTROL_ONLY': '1'}):
            result = lifecycle._stop()
        self.assertEqual(os.getpid(), result['hostPid'])
        self.assertTrue(result['controlPlaneOnly'])
        self.assertFalse(result['agentLaunchAttempted'])
        self.assertIsNone(result['agentIdentity'])
        self.assertEqual(self.controller, result['controllerIdentity'])
        operation = {'operationId': lifecycle.operation_id, 'generation': lifecycle.operation_id,
                     'homePath': self.home}
        self.assertTrue(has_terminal_deployment_proofs(operation, result, self.home))

    def setUp(self):
        self.repo = Path.cwd().absolute()
        self.home = str(self.repo / '.jenkins/jenkins_home')
        self.operation = {'operationId': 'old', 'generation': 'old', 'hostPid': 101, 'pid': 101,
                          'creationTime': '100', 'homePath': self.home, 'commandDigest': 'a' * 64,
                          'executable': str(self.repo / '.jenkins/runtime/python/python.exe')}
        self.controller = {'pid': 102, 'creationTime': '120', 'commandDigest': 'b' * 64,
                           'executable': str(self.repo / '.jenkins/runtime/jdk/bin/java.exe')}
        self.host = {'operationId': 'old', 'generation': 'old', 'hostPid': 101,
                     'status': 'starting', 'identity': self.controller, 'agentLaunchAttempted': False}

    def evidence(self, host=None):
        return abandoned_startup_evidence(self.operation, host or self.host, self.home,
                                         self.repo, {}, '127.0.0.1', 18080)

    def test_absence_is_not_confused_with_access_denied(self):
        with patch('tools.jenkins.deployment.startup_observations.process_alive', return_value=False), \
                patch('tools.jenkins.deployment.startup_observations.current_identity') as query:
            self.assertEqual('absent', observe_departed_identity(self.controller)['status'])
            query.assert_not_called()
        with patch('tools.jenkins.deployment.startup_observations.process_alive', return_value=None):
            with self.assertRaises(JenkinsError) as error:
                observe_departed_identity(self.controller)
            self.assertEqual('deployment_identity_inconclusive', error.exception.code)

    def test_birth_identity_protects_live_owner_and_allows_pid_reuse(self):
        with patch('tools.jenkins.deployment.startup_observations.process_alive', return_value=True), \
                patch('tools.jenkins.deployment.startup_observations.current_identity',
                      return_value=ProcessIdentity(102, '120', self.controller['executable'])):
            with self.assertRaises(JenkinsError) as error:
                observe_departed_identity(self.controller)
            self.assertEqual('deployment_owner_live', error.exception.code)
        with patch('tools.jenkins.deployment.startup_observations.process_alive', return_value=True), \
                patch('tools.jenkins.deployment.startup_observations.current_identity',
                      return_value=ProcessIdentity(102, '220', 'unrelated.exe')):
            self.assertEqual('pid-reused', observe_departed_identity(self.controller)['status'])

    def test_query_failure_does_not_authorize_recovery(self):
        with patch('tools.jenkins.deployment.startup_observations.process_alive', return_value=True), \
                patch('tools.jenkins.deployment.startup_observations.current_identity',
                      side_effect=JenkinsError('process_identity_inconclusive', 'denied')):
            with self.assertRaises(JenkinsError) as error:
                observe_departed_identity(self.controller)
            self.assertEqual('deployment_identity_inconclusive', error.exception.code)

    def test_success_keeps_native_and_execution_acceptance_closed(self):
        original = dict(self.host)
        with patch('tools.jenkins.deployment.recovery._verified_startup_containment', return_value='audited'), \
                patch('tools.jenkins.deployment.startup_observations.observe_departed_identity', return_value={'status': 'absent'}) as departed, \
                patch('tools.jenkins.deployment.startup_observations.runtime_owner_inventory', return_value=[]), \
                patch('tools.jenkins.deployment.startup_observations.observe_home_available', return_value={'exclusiveReadProbes': []}), \
                patch('tools.jenkins.deployment.lifecycle.check_port_available') as endpoint:
            result = self.evidence()
        endpoint.assert_called_once_with('127.0.0.1', 18080)
        self.assertEqual(4, departed.call_count)
        self.assertEqual('same-boot-control-plane-recovery', result['kind'])
        self.assertTrue(result['controlPlaneOnly'])
        for field in ('nativeProofIssued', 'validationAcceptanceAllowed', 'executionHoldsReleaseAllowed'):
            self.assertFalse(result[field])
        for field in ('nativeTerminationProof', 'stdoutEof', 'stderrEof', 'processExitCode', 'completeProof'):
            self.assertNotIn(field, result)
        self.assertEqual(original, self.host)

    def test_agent_attempt_or_mismatched_owner_is_rejected_before_probes(self):
        for change in ({'agentLaunchAttempted': True}, {'agentLaunchAttempted': None},
                       {'agentIdentity': self.controller}, {'generation': 'other'},
                       {'hostPid': 202}, {'status': 'running'}):
            with self.subTest(change=change), \
                    patch('tools.jenkins.deployment.recovery._verified_startup_containment') as verified:
                with self.assertRaises(JenkinsError):
                    self.evidence({**self.host, **change})
                verified.assert_not_called()

    def test_another_runtime_owner_blocks_start(self):
        with patch('tools.jenkins.deployment.recovery._verified_startup_containment', return_value='audited'), \
                patch('tools.jenkins.deployment.startup_observations.observe_departed_identity', return_value={'status': 'absent'}), \
                patch('tools.jenkins.deployment.startup_observations.runtime_owner_inventory', return_value=[{'pid': 303}]):
            with self.assertRaises(JenkinsError) as error:
                self.evidence()
            self.assertEqual('deployment_home_owner_live', error.exception.code)

    def test_unreviewed_historical_driver_does_not_inherit_exception(self):
        with self.assertRaises(JenkinsError) as error:
            _verified_startup_containment({'driverDigest': 'unknown', 'runtimeOperationId': 'old'},
                                         self.operation, self.repo)
        self.assertEqual('deployment_startup_containment_unproven', error.exception.code)

    def test_control_plane_host_cannot_change_old_launch_records(self):
        from tools.jenkins.deployment.host import LifecycleHost
        host = object.__new__(LifecycleHost)
        with patch.dict('os.environ', {'ZIRCON_JENKINS_RECOVERY_CONTROL_ONLY': '1'}), \
                patch('tools.jenkins.deployment.host.State') as state:
            host._execution_broker_tick()
            state.assert_not_called()

    def test_control_plane_start_keeps_agent_offline_and_full_readiness_false(self):
        from tools.jenkins.deployment.host import LifecycleHost
        host = object.__new__(LifecycleHost)
        host.job = None
        host.agent_job = None
        host.paths = Mock()
        host.java = self.repo / '.jenkins/runtime/jdk/bin/java.exe'
        host.war = self.repo / '.jenkins/runtime/jenkins.war'
        host.spec = Mock(controller={'listenAddress': '127.0.0.1', 'httpPort': 18080})
        host.operation_id = 'new'
        host._save = Mock()
        host._auth_headers = Mock(return_value={})
        host._start_agent = Mock()
        job = Mock(identity=ProcessIdentity(303, '300', str(host.java)))
        with patch.dict('os.environ', {'ZIRCON_JENKINS_RECOVERY_CONTROL_ONLY': '1'}), \
                patch('tools.jenkins.deployment.host.check_port_available'), \
                patch('tools.jenkins.deployment.host.launch_command', return_value=(['java'], {})), \
                patch('tools.jenkins.deployment.host.spawn_owned', return_value=job), \
                patch('tools.jenkins.deployment.host.wait_for_health', return_value=True), \
                patch('tools.jenkins.deployment.host.DeploymentManager'):
            result = host._start()
        host._start_agent.assert_not_called()
        self.assertEqual('running', result['status'])
        self.assertTrue(result['controlPlaneOnly'])
        self.assertFalse(result['agentLaunchAttempted'])
        self.assertTrue(result['healthProof']['controller'])
        self.assertFalse(result['healthProof']['ready'])


if __name__ == '__main__':
    unittest.main()
