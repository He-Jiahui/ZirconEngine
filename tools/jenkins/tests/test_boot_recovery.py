import tempfile
from pathlib import Path
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.processes.boot import recover_previous_boot, boot_abort_evidence, windows_boot_filetime
from tools.jenkins.resources import Capacity, ResourceManager
from tools.jenkins.state import State
from tools.jenkins.deployment.recovery import previous_boot_deployment_evidence, has_terminal_deployment_proofs


class BootRecoveryTests(unittest.TestCase):
    def test_boot_query_treats_localized_failure_as_unavailable_and_accepts_only_decimal_evidence(self):
        import subprocess
        failed = subprocess.CompletedProcess([], 1, b'', b'\xbe\xdc\xbe\xf8')
        with patch('tools.jenkins.processes.boot.os.name', 'nt'), patch('tools.jenkins.processes.boot.subprocess.run', return_value=failed) as query:
            with self.assertRaises(JenkinsError) as caught:
                windows_boot_filetime()
            self.assertEqual('boot_evidence_unavailable', caught.exception.code)
            self.assertNotIn('text', query.call_args.kwargs)
        good = subprocess.CompletedProcess([], 0, b'134355592655000000\r\n', b'')
        with patch('tools.jenkins.processes.boot.os.name', 'nt'), patch('tools.jenkins.processes.boot.subprocess.run', return_value=good):
            self.assertEqual(134355592655000000, windows_boot_filetime())

    def test_real_terminal_proofs_allow_retry_but_incomplete_or_unbound_proofs_do_not(self):
        import sys
        from tools.jenkins.processes import NativeJob
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            jobs = []
            try:
                for component in ('controller', 'agent'):
                    job = NativeJob.launch((sys.executable, '-B', '-c', "print('component exited')"),
                                          cwd=root, log_dir=root / component)
                    jobs.append(job)
                    job.wait(timeout_seconds=10)
                    self.assertTrue(job.proof().complete)
                operation = {'operationId': 'op', 'generation': 'op', 'homePath': str(root / 'home')}
                host = {'operationId': 'op', 'generation': 'op', 'status': 'failed',
                        'identity': jobs[0].identity.to_dict(), 'agentIdentity': jobs[1].identity.to_dict(),
                        'nativeTerminationProof': jobs[0].proof().to_dict(),
                        'agentTerminationProof': jobs[1].proof().to_dict()}
                home = operation['homePath']
                self.assertTrue(has_terminal_deployment_proofs(operation, host, home))
                self.assertTrue(has_terminal_deployment_proofs(operation, {**host, 'status': 'stopped'}, home))
                self.assertFalse(has_terminal_deployment_proofs(operation, {**host, 'generation': 'other'}, home))
                self.assertFalse(has_terminal_deployment_proofs(operation, host, 'different-home'))
                for key, value in [('stdoutEof', False), ('childrenGone', False), ('activeProcesses', 1), ('processExitCode', None)]:
                    with self.subTest(key=key):
                        bad = {**host, 'nativeTerminationProof': {**host['nativeTerminationProof'], key: value}}
                        self.assertFalse(has_terminal_deployment_proofs(operation, bad, home))
                self.assertFalse(has_terminal_deployment_proofs(operation,
                    {**host, 'agentIdentity': {**host['agentIdentity'], 'creationTime': '1'}}, home))
                before_agent = {**host, 'agentIdentity': None, 'agentTerminationProof': None,
                                'agentLaunchAttempted': False}
                self.assertTrue(has_terminal_deployment_proofs(operation, before_agent, home))
                self.assertFalse(has_terminal_deployment_proofs(operation,
                    {**before_agent, 'agentLaunchAttempted': True}, home))
            finally:
                for job in jobs:
                    if not job.proof().complete:
                        job.terminate(timeout_seconds=10)
                    job.close()

    def test_boot_recovery_requires_explicit_proof_that_agent_launch_was_never_attempted(self):
        operation = {'operationId': 'op', 'generation': 'op', 'hostPid': 1, 'pid': 1,
                     'creationTime': '100', 'homePath': 'home'}
        host = {'operationId': 'op', 'generation': 'op', 'hostPid': 1,
                'controllerIdentity': {'pid': 2, 'creationTime': '120'}, 'agentLaunchAttempted': False}
        with patch('tools.jenkins.deployment.recovery.windows_boot_filetime', return_value=200):
            evidence = previous_boot_deployment_evidence(operation, host, 'home')
            self.assertEqual(2, len(evidence['identities']))
            self.assertFalse(evidence['validationAcceptanceAllowed'])
            with self.assertRaises(JenkinsError) as error:
                previous_boot_deployment_evidence(operation, {**host, 'agentLaunchAttempted': True}, 'home')
            self.assertEqual('deployment_recovery_identity_missing', error.exception.code)

    def test_deployment_recovery_requires_later_boot_and_all_component_identities(self):
        operation = {"operationId": "op", "generation": "op", "hostPid": 1,
            "pid": 1, "creationTime": "100", "homePath": "home"}
        host = {"operationId": "op", "generation": "op", "hostPid": 1,
            "controllerIdentity": {"pid": 2, "creationTime": "120"},
            "agentIdentity": {"pid": 3, "creationTime": "130"}}
        with patch("tools.jenkins.deployment.recovery.windows_boot_filetime", return_value=50):
            with self.assertRaises(JenkinsError) as error:
                previous_boot_deployment_evidence(operation, host, "home")
            self.assertEqual("deployment_termination_unproven", error.exception.code)
        with patch("tools.jenkins.deployment.recovery.windows_boot_filetime", return_value=200):
            evidence = previous_boot_deployment_evidence(operation, host, "home")
            self.assertEqual("interrupted", evidence["outcome"])
            self.assertFalse(evidence["validationAcceptanceAllowed"])
            self.assertNotIn("completeProof", evidence)
            for bad in ({**host, "agentIdentity": None}, {**host, "operationId": "other"}):
                with self.assertRaises(JenkinsError):
                    previous_boot_deployment_evidence(operation, bad, "home")

    def test_only_actual_later_boot_allows_failure_recovery(self):
        with tempfile.TemporaryDirectory() as temp:
            state = State(Path(temp) / "state.sqlite3")
            state.put("execution", "e", {"status": "blocked"})
            state.put("execution_host", "e", {"hostBirthToken": "100"})
            state.put("execution_launch", "e", {"launcherBirthToken": "100"})
            state.put("native_job", "j", {"executionId": "e", "owner": "e", "birthToken": "150", "status": "running"})
            state.put("resource_reservation", "r", {"owner": "e", "status": "active"})
            state.put("pool_writer_hold", "p", {"owner": "e", "status": "active", "reservationKey": "r"})
            manager = ResourceManager(state, Capacity(1, 1, 1))
            with patch("tools.jenkins.processes.boot.windows_boot_filetime", return_value=50):
                with self.assertRaises(JenkinsError) as error:
                    recover_previous_boot(state, "e")
                self.assertEqual("previous_boot_unproven", error.exception.code)
                self.assertEqual([], state.list("boot_termination"))
                with self.assertRaises(JenkinsError):
                    manager.release("r", writer_key="p", execution_id="e")
            with patch("tools.jenkins.processes.boot.windows_boot_filetime", return_value=200):
                evidence = recover_previous_boot(state, "e")
                self.assertEqual("failed", evidence["outcome"])
                self.assertFalse(evidence["artifactPublicationAllowed"])
                self.assertEqual(evidence, boot_abort_evidence(state, "e"))
                self.assertTrue(manager.release("r", writer_key="p", execution_id="e"))
                self.assertNotIn("completeProof", state.get("native_job", "j")["payload"])
                self.assertEqual([], state.list("validation_receipt"))
                state.put("native_job", "new", {"executionId": "e", "birthToken": "250"})
                with self.assertRaises(JenkinsError):
                    boot_abort_evidence(state, "e")


if __name__ == "__main__":
    unittest.main()
