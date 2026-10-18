from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock, patch

import pytest

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment import independent_host as host


def configured_host(tmp_path):
    home = tmp_path / 'home'
    node = home / 'nodes/zircon-windows-agent/config.xml'
    node.parent.mkdir(parents=True)
    node.write_text('<slave><numExecutors>4</numExecutors></slave>')
    job = home / 'jobs/zircon-flow/config.xml'
    job.parent.mkdir(parents=True)
    job.write_text('<flow-definition><script>current-user-script</script></flow-definition>')
    value = object.__new__(host.IndependentLifecycleHost)
    value.job = value.agent_job = None
    value.paths = SimpleNamespace(home=home)
    value.spec = SimpleNamespace(controller={'listenAddress': '127.0.0.1', 'httpPort': 18080})
    value.java = Path('java.exe')
    value.war = Path('jenkins.war')
    value.operation_id = 'service-runtime'
    value._save = Mock()
    value._auth_headers = Mock(return_value={})
    return value


def test_start_preserves_live_jobs_and_four_executor_node(tmp_path):
    value = configured_host(tmp_path)
    before = host.home_configuration(value.paths.home)
    controller, agent = Mock(), Mock()
    controller.identity.to_dict.return_value = {'pid': 10}
    agent.identity.to_dict.return_value = {'pid': 11}
    value._start_agent = Mock(side_effect=lambda: setattr(value, 'agent_job', agent))
    with patch.object(host, 'check_port_available'), \
            patch.object(host, 'launch_command', return_value=(['java'], {})), \
            patch.object(host, 'spawn_owned', return_value=controller), \
            patch.object(host, 'wait_for_health', return_value=True), \
            patch('tools.jenkins.deployment.host.DeploymentManager.bootstrap', side_effect=AssertionError('must preserve Home')), \
            patch('tools.jenkins.deployment.host.DeploymentManager.register_jobs', side_effect=AssertionError('must preserve jobs')):
        result = value._start()
    assert result['status'] == 'running'
    assert result['homeConfigurationAfter'] == before == host.home_configuration(value.paths.home)
    assert result['agentIdentity'] == {'pid': 11}


def test_failed_start_keeps_actual_owned_cleanup_evidence(tmp_path):
    value = configured_host(tmp_path)
    controller = Mock()
    controller.identity.to_dict.return_value = {'pid': 10}
    proof = {'complete': True, 'identity': {'pid': 10}}
    value._stop = Mock(return_value={'nativeTerminationProof': proof})
    with patch.object(host, 'check_port_available'), \
            patch.object(host, 'launch_command', return_value=(['java'], {})), \
            patch.object(host, 'spawn_owned', return_value=controller), \
            patch.object(host, 'wait_for_health', return_value=False):
        with pytest.raises(JenkinsError, match='did not become healthy'):
            value._start()
    value._stop.assert_called_once()
    failure = value._save.call_args.args[0]
    assert failure['status'] == 'failed' and failure['nativeTerminationProof'] == proof


def test_current_configuration_is_observed_without_permanent_hash_gate(tmp_path):
    value = configured_host(tmp_path)
    path = value.paths.home / 'nodes/zircon-windows-agent/config.xml'
    before = host.home_configuration(value.paths.home)
    path.write_text('<slave><numExecutors>6</numExecutors></slave>')
    assert host.home_configuration(value.paths.home) != before


def test_dead_agent_is_restarted_without_stopping_controller(tmp_path):
    value = configured_host(tmp_path)
    value.job, value.agent_job = Mock(), Mock()
    value.job.process.poll.return_value = None
    value.agent_job.process.poll.return_value = 1
    old = value.agent_job
    old.proof.return_value.to_dict.return_value = {'complete': True, 'identity': {'pid': 11}}
    fresh = Mock()
    fresh.identity.to_dict.return_value = {'pid': 12}
    value.root = tmp_path
    value.state = tmp_path / 'host.json'
    value.state.write_text('{"operationId":"service-runtime"}')
    value._start_agent = Mock(side_effect=lambda: setattr(value, 'agent_job', fresh))
    with patch.object(host.LifecycleHost, '_execution_broker_tick'):
        value._execution_broker_tick()
    old.close.assert_called_once()
    value.job.terminate.assert_not_called()
    assert value._save.call_args.args[0]['agentIdentity'] == {'pid': 12}


def test_controller_exit_keeps_owned_agent_identity_for_automatic_restart(tmp_path):
    import json
    value = configured_host(tmp_path)
    value.state = tmp_path / 'host.json'
    previous = {'operationId': 'service-runtime', 'agentIdentity': {'pid': 11},
                'controllerIdentity': {'pid': 10}, 'controlPlaneOnly': False,
                'agentLaunchAttempted': True}
    value.state.write_text(json.dumps(previous))
    del value._save
    with patch.object(host.LifecycleHost, '_save') as write:
        value._save({'operationId': 'service-runtime', 'status': 'stopped', 'nativeTerminationProof': {'complete': True}})
    assert write.call_args.args[0]['agentIdentity'] == {'pid': 11}
    assert write.call_args.args[0]['controlPlaneOnly'] is False
