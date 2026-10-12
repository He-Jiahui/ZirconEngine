import ctypes
import json
import os
from pathlib import Path
import threading
import time
from types import SimpleNamespace
from unittest.mock import Mock, patch

import pytest

from tools.jenkins.state import atomic
from tools.jenkins.deployment import host as host_module


def windows_error(code):
    error = PermissionError("replacement refused")
    error.winerror = code
    return error


def test_retries_same_atomic_replacement_without_removing_original(tmp_path):
    destination = tmp_path / "pointer.json"
    destination.write_text('{"old":true}')
    real_replace = os.replace
    attempts = []

    def replace(source, target):
        attempts.append((source, target))
        if len(attempts) < 3:
            assert json.loads(destination.read_text()) == {"old": True}
            raise windows_error(32)
        real_replace(source, target)

    with patch.object(atomic.os, "replace", side_effect=replace), patch.object(atomic.os, "name", "nt"):
        atomic.atomic_json(destination, {"new": True})
    assert len(attempts) == 3
    assert len(set(attempts)) == 1
    assert json.loads(destination.read_text()) == {"new": True}


def test_persistent_denial_is_bounded_and_preserves_attempted_record(tmp_path):
    destination = tmp_path / "pointer.json"
    destination.write_text('{"old":true}')
    clock = iter([0.0, 0.0, 0.0, 2.1])
    with patch.object(atomic.os, "replace", side_effect=windows_error(5)), \
            patch.object(atomic.os, "name", "nt"), \
            patch.object(atomic.time, "monotonic", side_effect=lambda: next(clock)), \
            patch.object(atomic.time, "sleep"):
        with pytest.raises(PermissionError):
            atomic.atomic_json(destination, {"new": True})
    assert json.loads(destination.read_text()) == {"old": True}
    pending = list(tmp_path.glob("pointer.json.pending-*"))
    assert len(pending) == 1
    assert json.loads(pending[0].read_text()) == {"new": True}


def test_retry_rechecks_expected_pointer_before_replacement(tmp_path):
    source, destination = tmp_path / "pending", tmp_path / "pointer"
    source.write_text("next")
    destination.write_text("old")

    def replace(_source, _destination):
        destination.write_text("foreign")
        raise windows_error(32)

    def unchanged():
        if destination.read_text() != "old":
            raise ValueError("changed")

    with patch.object(atomic.os, "replace", side_effect=replace) as replaced, \
            patch.object(atomic.os, "name", "nt"):
        with pytest.raises(ValueError, match="changed"):
            atomic.replace_file(source, destination, before_replace=unchanged)
    assert replaced.call_count == 1
    assert destination.read_text() == "foreign"


@pytest.mark.skipif(os.name != "nt", reason="Windows delete sharing contract")
def test_actual_windows_reader_lock_releases_before_pointer_commit(tmp_path):
    from ctypes import wintypes
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD,
                                   wintypes.LPVOID, wintypes.DWORD, wintypes.DWORD, wintypes.HANDLE]
    kernel.CreateFileW.restype = wintypes.HANDLE
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.CloseHandle.restype = wintypes.BOOL
    path = tmp_path / "pointer.json"
    path.write_text('{"old":true}')
    handle = kernel.CreateFileW(str(path), 0x80000000, 0x1 | 0x2, None, 3, 0x80, None)
    assert handle not in (None, ctypes.c_void_p(-1).value), ctypes.get_last_error()
    errors = []
    finished = threading.Event()

    def write():
        try:
            atomic.atomic_json(path, {"new": True})
        except Exception as error:
            errors.append(error)
        finally:
            finished.set()

    worker = threading.Thread(target=write)
    worker.start()
    try:
        assert not finished.wait(.1)
        assert json.loads(path.read_text()) == {"old": True}
    finally:
        assert kernel.CloseHandle(handle)
    worker.join(3)
    assert finished.is_set()
    assert errors == []
    assert json.loads(path.read_text()) == {"new": True}


@pytest.mark.parametrize("fail_at", ["initial", "ready", "persistent"])
def test_start_pointer_failure_cleans_owned_jobs_and_keeps_proofs(tmp_path, fail_at):
    lifecycle = object.__new__(host_module.LifecycleHost)
    lifecycle.spec = SimpleNamespace(controller={"listenAddress": "127.0.0.1", "httpPort": 18080})
    lifecycle.paths = SimpleNamespace(state=tmp_path, repo=tmp_path, home=tmp_path / "home")
    lifecycle.root = tmp_path / "deployment"
    lifecycle.root.mkdir()
    lifecycle.state = lifecycle.root / "host.json"
    lifecycle.java, lifecycle.war = tmp_path / "java", tmp_path / "war"
    lifecycle.operation_id = "start-test"
    lifecycle.job = lifecycle.agent_job = None
    lifecycle._auth_headers = Mock(return_value={})
    controller = Mock()
    agent = Mock()
    for pid, job in [(101, controller), (202, agent)]:
        job.identity.to_dict.return_value = {"pid": pid, "creationTime": str(pid)}
        job.process.poll.return_value = None
        job.terminate.return_value.to_dict.return_value = {"complete": True, "pid": pid}

    def start_agent():
        lifecycle.agent_launch_attempted = True
        lifecycle.agent_job = agent

    lifecycle._start_agent = start_agent

    def save(value):
        if fail_at == "persistent" or value["status"] == ("starting" if fail_at == "initial" else "running"):
            raise windows_error(5)
        atomic.atomic_json(lifecycle.state, value)

    lifecycle._save = save
    with patch.object(host_module, "check_port_available"), \
            patch.object(host_module, "launch_command", return_value=(["java"], {})), \
            patch.object(host_module, "spawn_owned", return_value=controller), \
            patch.object(host_module, "wait_for_health", return_value=True), \
            patch.object(host_module, "DeploymentManager"), \
            patch.dict(os.environ, {"ZIRCON_JENKINS_RECOVERY_CONTROL_ONLY": "0"}):
        with pytest.raises(PermissionError):
            lifecycle._start()
    controller.terminate.assert_called_once()
    controller.close.assert_called_once()
    if fail_at == "ready":
        agent.terminate.assert_called_once()
        agent.close.assert_called_once()
    else:
        agent.terminate.assert_not_called()
    assert lifecycle.job is lifecycle.agent_job is None
    record = (list(lifecycle.root.glob("startup-failed-*.json"))[0]
              if fail_at == "persistent" else lifecycle.state)
    failure = json.loads(record.read_text())
    assert failure["status"] == "failed"
    assert failure["nativeTerminationProof"] == {"complete": True, "pid": 101}
    assert failure["agentTerminationProof"] == ({"complete": True, "pid": 202} if fail_at == "ready" else None)
    if fail_at == "persistent":
        assert failure["stateRecordUncommitted"] is True
