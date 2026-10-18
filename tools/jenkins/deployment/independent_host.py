"""Start the existing Home without reprovisioning jobs or executor settings.

The selected immutable backend continues to own agent and execution protocols.
This installed lifecycle entry only changes service hosting and startup policy.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import sys
import time

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.deployment.driver import active_driver_binding, _plain
from tools.jenkins.deployment.host import LifecycleHost
from tools.jenkins.deployment.lifecycle import check_port_available, launch_command, spawn_owned, wait_for_health
from tools.jenkins.processes.identity import current_identity


def home_configuration(home: Path) -> dict:
    """Record current configuration for diagnostics, not a permanent startup pin."""
    records = {}
    for group in ('jobs', 'nodes'):
        root = home / group
        if root.exists():
            for path in sorted(root.glob('*/config.xml')):
                _plain(path)
                records[path.relative_to(home).as_posix()] = hashlib.sha256(path.read_bytes()).hexdigest()
    return records


class IndependentLifecycleHost(LifecycleHost):
    def _save(self, value):
        # Preserve the last owned identities when the sealed backend publishes
        # controller-exit proofs, so the watchdog can restart the same service.
        if self.state.exists():
            previous = json.loads(self.state.read_text(encoding='utf-8'))
            if previous.get('operationId') == value.get('operationId') == self.operation_id:
                for name in ('controllerIdentity', 'agentIdentity', 'agentLaunchAttempted',
                             'controlPlaneOnly', 'preservedExistingHome', 'serviceHosting'):
                    if name not in value and name in previous:
                        value[name] = previous[name]
        return super()._save(value)

    def _execution_broker_tick(self):
        # Remoting failure must not take the controller down. Only the host
        # holding the actual agent Job can close it and retry its handshake.
        if self.job is not None and self.job.process.poll() is None:
            now = time.monotonic()
            agent_dead = self.agent_job is None or self.agent_job.process.poll() is not None
            if agent_dead and now >= getattr(self, '_agent_retry_after', 0):
                self._agent_retry_after = now + 15
                if self.agent_job is not None:
                    self.agent_job.wait(timeout_seconds=30, cleanup_descendants=True)
                    proof = self.agent_job.proof(requested=False).to_dict()
                    self.agent_job.close()
                    self.agent_job = None
                    from tools.jenkins.state.atomic import atomic_json
                    atomic_json(self.root / ('agent-exit-' + str(time.time_ns()) + '.json'), proof)
                try:
                    self._start_agent()
                    value = json.loads(self.state.read_text(encoding='utf-8'))
                    value.update(agentIdentity=self.agent_job.identity.to_dict(), agentLaunchAttempted=True,
                                 status='running', healthProof={'controller': True, 'agent': True, 'ready': True})
                    self._save(value)
                except Exception as error:
                    # Retry only this owned dead agent; keep controller Jobs.
                    print(json.dumps({'agentRestartPending': True,
                                      'reasonCode': getattr(error, 'code', type(error).__name__)}), flush=True)
        return super()._execution_broker_tick()

    def _start(self):
        if self.job is not None:
            return {'status': 'running', 'identity': self.job.identity.to_dict()}
        self.agent_launch_attempted = False
        before = home_configuration(self.paths.home)
        check_port_available(self.spec.controller['listenAddress'], int(self.spec.controller['httpPort']))
        args, env = launch_command(self.paths, self.java, self.war, int(self.spec.controller['httpPort']))
        self.job = spawn_owned(self.paths, args, env, component='controller')
        value = {'status': 'starting', 'operationId': self.operation_id, 'generation': self.operation_id,
                 'hostPid': os.getpid(), 'identity': self.job.identity.to_dict(),
                 'controllerIdentity': self.job.identity.to_dict(), 'agentLaunchAttempted': False,
                 'controlPlaneOnly': False, 'startedAt': time.time(), 'preservedExistingHome': True,
                 'homeConfigurationBefore': before, 'serviceHosting': 'windows-native-independent'}
        self._save(value)
        try:
            endpoint = f"http://{self.spec.controller['listenAddress']}:{self.spec.controller['httpPort']}"
            if not wait_for_health(endpoint, 300, headers=self._auth_headers(), process=self.job):
                raise JenkinsError('controller_health_timeout', 'Jenkins did not become healthy')
            # Deliberately no bootstrap/register_jobs: normal service restarts
            # must honor the user's live jobs, node count and plugin settings.
            self._start_agent()
            value.update(status='running', agentIdentity=self.agent_job.identity.to_dict(),
                         agentLaunchAttempted=True, homeConfigurationAfter=home_configuration(self.paths.home),
                         healthProof={'controller': True, 'agent': True, 'ready': True})
            self._save(value)
            return value
        except Exception as exc:
            failure = {**value, 'status': 'failed', 'reasonCode': getattr(exc, 'code', 'startup_failed'),
                       'agentLaunchAttempted': self.agent_launch_attempted}
            try:
                stopped = self._stop()
                failure.update({k: v for k, v in stopped.items() if k in
                                ('nativeTerminationProof', 'agentTerminationProof', 'agentIdentity')})
            except Exception:
                failure['cleanupPending'] = True
            self._save(failure)
            raise


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--spec', type=Path, required=True)
    parser.add_argument('--java', type=Path, required=True)
    parser.add_argument('--war', type=Path, required=True)
    parser.add_argument('--operation-id', required=True)
    parser.add_argument('--policy', type=Path, required=True)
    parser.add_argument('--policy-sha256', required=True)
    args = parser.parse_args()
    host = IndependentLifecycleHost(args.spec, args.java, args.war, args.operation_id)
    binding = active_driver_binding(host.paths.repo)
    if binding['runtimeOperationId'] != args.operation_id:
        raise JenkinsError('driver_generation_changed', 'Selected service runtime changed')
    # Bind this child to its issued start; no task/test acceptance is inferred.
    deadline = time.monotonic() + 15
    while True:
        attempt = json.loads((host.paths.state / 'independent-service/current.json').read_text(encoding='utf-8'))
        if attempt.get('identity', {}).get('pid') == os.getpid():
            break
        if time.monotonic() >= deadline:
            raise JenkinsError('independent_service_unproven', 'Service child was not registered by its owner')
        time.sleep(.1)
    actual = current_identity(os.getpid(), command_line=tuple([sys.executable, str(host.paths.repo / 'tools/dev/jenkins_service_host.py'), *sys.argv[1:]])).to_dict()
    if (attempt['operationId'] != args.operation_id or attempt['identity'] != actual
            or attempt['policySha256'] != args.policy_sha256):
        raise JenkinsError('independent_service_unproven', 'Service child identity changed')
    host.run()
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
