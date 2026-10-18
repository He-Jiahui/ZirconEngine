"""Jenkins-only closeout of one completed, hash-bound governance test's fixtures.

Keep the failed original request unchanged. Retry only transient Windows sharing
violations through the same guarded deletion API and journal every actual delete.
"""
from __future__ import annotations

import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import sys
import time
from types import SimpleNamespace
import xml.etree.ElementTree as ET


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=('launch', 'worker'))
    for name in ('plan', 'plan-sha256', 'self-sha256'):
        parser.add_argument('--' + name, required=True)
    args = parser.parse_args()
    require(os.environ.get('JOB_NAME') == 'zircon-scheduled-cleanup' and
            os.environ.get('NODE_NAME') == 'zircon-windows-agent' and
            os.environ.get('BUILD_NUMBER', '').isdigit(), 'real scheduled cleanup agent required')
    # Only stdlib code is loaded until this entry, plan and shared helper are pinned.
    import hashlib
    checksum = lambda raw: hashlib.sha256(raw).hexdigest()
    require(checksum(Path(__file__).read_bytes()) == args.self_sha256, 'closeout source changed')
    raw = Path(args.plan).read_bytes()
    require(checksum(raw) == args.plan_sha256, 'closeout plan changed')
    plan = json.loads(raw)
    helper = Path(plan['helper']['path'])
    require(checksum(helper.read_bytes()) == plan['helper']['sha256'], 'projection helper changed')
    spec = importlib.util.spec_from_file_location('_fixture_closeout_helper', helper)
    c = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(c)
    c.physical(__file__)
    c.physical(args.plan)
    require(Path(sys.executable).absolute() == c.REPO / '.jenkins/runtime/python/python.exe', 'Jenkins interpreter required')
    control = c.verified(plan['controlPlan'])
    require(control['runtimeOperationId'] == os.environ.get('ZIRCON_RUNTIME_OPERATION_ID') and
            control['driverDigest'] == os.environ.get('ZIRCON_DRIVER_DIGEST'), 'original runtime generation changed')
    output = c.physical(Path(plan['outputRoot']) / os.environ['BUILD_NUMBER'])
    require(output.parent == Path('E:/cargo-targets/zircon-local/reports/jenkins-governance-repair-20261019/fixture-closeout'),
            'fixed closeout output root required')
    report = c.physical(plan['originalReport'])
    require(report.parent == Path('E:/cargo-targets/zircon-local/reports/disk-governance-20261015') and
            report.name == 'governance-usable-20261019-r1', 'closeout is limited to the original owned test')
    checks = c.load(control['checksModule'], control['pins'][control['checksModule']], '_fixture_closeout_checks')
    if args.mode == 'launch':
        require(not output.exists(), 'original closeout exists; reconcile without repeating')
        output.mkdir(parents=True)
        c.save(output / 'artifacts.json', {'status': 'running', 'taskId': 'jenkins-governance-repair-20261019',
               'ownerBuild': os.environ['BUILD_NUMBER'], 'plan': c.fact(args.plan, raw), 'originalReport': str(report),
               'scope': 'original generated test fixtures only', 'sourceCopies': 0,
               'retention': 'Keep failed original and native/journal/recovery evidence'})
        summary = {'status': 'failed', 'startedAt': time.time(), 'globalCleanupComplete': False}
        complete = False
        try:
            binding = c.load(control['bindingModule'], control['pins'][control['bindingModule']], '_fixture_closeout_binding')
            proof = binding.observe_binding(c.REPO, control['driverDigest'])
            require(proof['runtimeOperationId'] == control['runtimeOperationId'], 'native runtime changed')
            c.save(output / 'runtime-binding.json', proof)
            native = c.load(control['nativeModule'], control['pins'][control['nativeModule']], '_fixture_closeout_native')
            command = [str(sys.executable), '-B', '-X', 'utf8', str(Path(__file__).absolute()), 'worker',
                       '--plan', args.plan, '--plan-sha256', args.plan_sha256, '--self-sha256', args.self_sha256]
            native_output = output / 'native'
            native_output.mkdir()
            terminal = native._run_owned_command(command, c.REPO, dict(os.environ, TEMP=str(output), TMP=str(output),
                                                     TMPDIR=str(output), PYTHONDONTWRITEBYTECODE='1'), native_output, 240)
            c.save(output / 'native-terminal.json', terminal)
            complete = checks.native_complete(terminal, command, c.REPO)
            require(complete and terminal.get('rootExitCode') == 0, 'closeout native execution incomplete')
            recovered = json.loads(c.read(output / 'recovery.json'))
            require(recovered['status'] == 'recovered-scoped' and recovered['fixturesRemain'] is False,
                    'original fixture closeout remains open')
            summary.update(status='passed', nativeComplete=True, recovery=recovered)
        except BaseException as error:
            summary['error'] = str(error)[:500]
            raise
        finally:
            summary.update(finishedAt=time.time(), nativeComplete=complete)
            c.save(output / 'summary.json', summary)
            c.save(output / 'closeout.json', {'status': summary['status'], 'nativeComplete': complete,
                   'activeOwnedProcesses': [] if complete else ['inspect-original-native-evidence'],
                   'fixturesRemain': (report / 'fixtures').exists(), 'globalCleanupComplete': False})
        print(json.dumps(summary, ensure_ascii=False))
        return 0

    receipt = c.verified(plan['originalReceipt'])
    envelope = c.verified(plan['originalRequest'])
    require(envelope['capsuleDigest'] == receipt['identity']['capsuleDigest'] and
            envelope['driverDigest'] == receipt['identity']['driverDigest'], 'original request capsule differs')
    request = envelope['request']
    native = c.verified(plan['originalNative'])
    c.verified(plan['originalInventory'])
    require(receipt['action'] == 'test' and receipt['identity']['sessionId'] == '01a12af6-7511-70d1-a452-61b80678eb56'
            and request['operationId'] == report.name and 'WinError 32' in receipt['result'].get('reason', ''),
            'not the original known sharing-failed test')
    expected = [str(c.REPO / '.jenkins/runtime/python/python.exe'), '-B', '-X', 'utf8', '-m', 'tools.jenkins.governance.tests']
    require(checks.native_complete(native, expected, c.REPO) and native['rootExitCode'] == 0,
            'original test process is not proven complete')
    job = ET.fromstring((c.REPO / '.jenkins/jenkins_home/jobs/zircon-governance/builds/210/build.xml').read_bytes())
    require(job.findtext('result') == 'FAILURE' and int(job.findtext('duration', '0')) > 0, 'original Jenkins run still active')
    log = c.read(plan['originalTestLog']['path'])
    require(c.fact(plan['originalTestLog']['path'], log) == plan['originalTestLog'], 'original test log changed')
    names = request['payload']['testNames']
    require(len(names) == 7 and all(name.encode() in log for name in names) and re.search(rb'Ran 7 tests.*?OK', log, re.S),
            'original seven test results not established')
    for relative, expected_hash in receipt['sourcePins'].items():
        require(c.checksum(c.read(c.REPO / relative, limit=8 * 1024**2)) == expected_hash,
                'original tested source changed: ' + relative)
    sys.path.insert(0, str(c.REPO))
    from tools.jenkins.governance import fixtures, guarded_delete
    original_verified = guarded_delete.remove_verified
    original_empty = guarded_delete.remove_empty
    retry_budget = time.monotonic() + 30
    retries = []
    journal = output / 'operations.jsonl'

    def record(value):
        with journal.open('ab') as stream:
            stream.write((json.dumps(value, ensure_ascii=False) + '\n').encode('utf-8'))
            stream.flush()
            os.fsync(stream.fileno())

    def guarded_call(function, arg, **kwargs):
        path = arg['path'] if isinstance(arg, dict) else str(arg)
        deleting = function is original_empty or kwargs.get('remove', True)
        if deleting:
            record({'status': 'delete-intent', 'path': path, 'directory': function is original_empty})
        deadline = min(retry_budget, time.monotonic() + 5)
        while True:
            try:
                result = function(arg, **kwargs)
                if deleting:
                    record({'status': 'delete-confirmed', 'path': path, 'result': result})
                return result
            except OSError as error:
                if getattr(error, 'winerror', None) not in (32, 33) or time.monotonic() >= deadline:
                    record({'status': 'failed', 'path': path, 'error': str(error)})
                    raise
                retries.append({'path': path, 'winerror': error.winerror})
                time.sleep(0.1)

    guarded_delete.remove_verified = lambda arg, **kwargs: guarded_call(original_verified, arg, **kwargs)
    guarded_delete.remove_empty = lambda arg, **kwargs: guarded_call(original_empty, arg, **kwargs)
    try:
        # Reuse the original complete capture, per-file identity/hash checks,
        # ancestor/file locks and handle-based deletion; no weaker cleanup path.
        ownership = {**receipt['identity'], 'fixtureRecoveryBuild': os.environ.get('BUILD_URL')}
        cleanup = fixtures.remove(SimpleNamespace(runtime=ownership), report)
    finally:
        guarded_delete.remove_verified = original_verified
        guarded_delete.remove_empty = original_empty
    require(not (report / 'fixtures').exists(), 'original fixture directories remain')
    require(c.fact(plan['originalReceipt']['path'], c.read(plan['originalReceipt']['path'])) == plan['originalReceipt'],
            'original failed receipt must remain unchanged')
    c.save(output / 'recovery.json', {'status': 'recovered-scoped', 'originalBuild': receipt['identity']['jenkinsBuildUrl'],
           'originalReceipt': plan['originalReceipt'], 'originalTestNames': names, 'originalTestsPassed': 7,
           'testedSourceInputsUnchanged': True, 'fixturesRemain': False, 'cleanup': cleanup, 'sharingRetries': retries,
           'cleanupProof': c.fact(report / 'fixture-cleanup.json', c.read(report / 'fixture-cleanup.json')),
           'formalAcceptance': False, 'globalCleanupComplete': False})
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
