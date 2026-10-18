"""Keep complete maintenance receipts on the agent, outside Pipeline continuations.

Only the existing sealed driver's maintenance/inventory and maintenance/gc are
invoked. Digest lists never cross the Pipeline boundary. Selection reads the
hash-bound original inventory, and the authoritative driver still decides GC.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import sys
import time

REPO = Path('E:/Git/ZirconEngine')
REPORTS = Path('E:/cargo-targets/zircon-local/reports/jenkins-scheduled-cleanup/control')
ROOTS = {str(Path(d + ':/cargo-targets')) for d in 'DEF'}
ID = re.compile(r'[A-Za-z0-9][A-Za-z0-9_.-]{0,180}\Z')
SHA = re.compile(r'[0-9a-f]{64}\Z')
MAX_RESPONSE = 128 * 1024**2


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def physical(value):
    p = Path(value).absolute()
    require('..' not in p.parts, 'parent traversal')
    for item in (p, *p.parents):
        if item.exists() or item.is_symlink():
            require(not item.is_symlink() and not getattr(item.lstat(), 'st_file_attributes', 0) & 0x400,
                    'path alias')
    return p


def read(path, *, limit=MAX_RESPONSE):
    p = physical(path)
    before = p.stat()
    require(p.is_file() and before.st_nlink == 1 and before.st_size <= limit, 'unbounded/nonordinary receipt')
    raw = p.read_bytes()
    after = p.stat()
    require((before.st_ino, before.st_size, before.st_mtime_ns) ==
            (after.st_ino, after.st_size, after.st_mtime_ns), 'receipt changed during read')
    return raw


def checksum(raw):
    return hashlib.sha256(raw).hexdigest()


def save(path, value):
    raw = json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
    with Path(path).open('xb') as f:
        f.write(raw)
        f.flush()
        os.fsync(f.fileno())


def fact(path, raw):
    return {'path': str(physical(path)), 'sha256': checksum(raw), 'bytes': len(raw)}


def verified(ref):
    require(isinstance(ref, dict) and SHA.fullmatch(ref.get('sha256', '')), 'invalid receipt reference')
    raw = read(ref['path'])
    require(len(raw) == ref['bytes'] and checksum(raw) == ref['sha256'], 'original receipt bytes changed')
    return json.loads(raw)


def project(value, action, payload, ref):
    require(isinstance(value, dict) and isinstance(value.get('status'), str), 'missing authoritative status')
    if (value['status'] in {'failed', 'blocked'} and value.get('reasonCode') == 'execution_busy'
            and value.get('retryable') is True):
        return {'status': value['status'], 'reasonCode': 'execution_busy', 'retryable': True, 'responseRef': ref}
    require(value['status'] == 'accepted' and isinstance(value.get('result'), dict), 'maintenance rejected; inspect responseRef')
    original = value['result']
    require(original.get('operationId') == payload['operationId'] and
            original.get('buildRoot') == payload['buildRoot'], 'original operation/root mismatch')
    result = {k: original[k] for k in ('operationId', 'buildRoot')}
    result['responseRef'] = ref
    if action == 'inventory':
        require(all(isinstance(original.get(k), list) for k in ('protectedDigests', 'unknownDigests')),
                'inventory digest lists missing')
        result.update(protectedDigestCount=len(original['protectedDigests']), unknownDigestCount=len(original['unknownDigests']))
    else:
        removed = original.get('removed')
        amount = original.get('reclaimedBytes')
        require(isinstance(removed, list) and len(removed) <= 64 and type(amount) is int and
                0 <= amount <= 268435456 and original.get('unknownProtected') is True, 'unbounded GC result')
        result.update(removed=removed, reclaimedBytes=amount, unknownProtected=True)
    out = {'status': 'accepted', 'result': result}
    require(len(json.dumps(out).encode()) <= 16384, 'unbounded Pipeline projection')
    return out


def select(inventory, root, max_objects, max_bytes):
    require(root in ROOTS and type(max_objects) is int and 0 <= max_objects <= 64 and
            type(max_bytes) is int and 0 <= max_bytes <= 268435456, 'selection bounds/root')
    require(inventory.get('buildRoot') == root and
            all(isinstance(inventory.get(k), list) for k in ('protectedDigests', 'unknownDigests')),
            'inventory root/digest lists')
    digests = inventory['protectedDigests'] + inventory['unknownDigests']
    require(all(isinstance(d, str) for d in digests), 'invalid protection digest')
    protected = {d.casefold() for d in digests}
    objects = physical(Path(root) / 'zircon-local/zircon-jenkins/artifacts/artifacts/objects')
    return choose(objects, protected, max_objects, max_bytes)


def choose(objects, protected, max_objects, max_bytes):
    chosen, size = [], 0
    if objects.exists():
        for p in sorted(objects.iterdir(), key=lambda p: p.name):
            if len(chosen) >= max_objects:
                break
            if not SHA.fullmatch(p.name) or p.name in protected:
                continue
            if p.is_symlink() or getattr(p.lstat(), 'st_file_attributes', 0) & 0x400 or not p.is_file():
                continue
            length = p.stat().st_size
            if size + length <= max_bytes:
                chosen.append(p.name)
                size += length
    return {'digests': chosen, 'plannedBytes': size, 'candidateCount': len(chosen)}


def regression(output):
    """Real bounded metadata fixtures; no production GC or source-tree fixtures."""
    fixture = output / 'projection-fixture.json'
    objects = output / 'selection-fixture'
    files = [objects / (letter * 64) for letter in ('0', 'a', 'b')]
    payload = {'operationId': 'fixture-inventory', 'buildRoot': str(Path('E:/cargo-targets'))}
    original = {'status': 'accepted', 'result': {**payload, 'protectedDigests': ['a' * 64],
                                               'unknownDigests': [f'{i:064x}' for i in range(20000)]}}
    checks = []
    try:
        save(fixture, original)
        raw = read(fixture)
        ref = fact(fixture, raw)
        view = project(verified(ref), 'inventory', payload, ref)
        require(view['result']['unknownDigestCount'] == 20000 and len(json.dumps(view)) < 2048 and
                'unknownDigests' not in view['result'], 'large receipt projection regression')
        checks.append('large-unknown-list-retained-in-hash-bound-reference')
        objects.mkdir()
        for p in files:
            with p.open('xb') as stream:
                stream.write(b'fixture')
        candidates = choose(objects, {'0' * 64, 'a' * 64}, 1, 7)
        require(candidates == {'digests': ['b' * 64], 'plannedBytes': 7, 'candidateCount': 1},
                'protected or unknown object selected')
        require(choose(objects, set(), 0, 100)['candidateCount'] == 0 and
                choose(objects, set(), 64, 6)['candidateCount'] == 0, 'zero count/byte budgets')
        checks.extend(['protected-and-unknown-objects-never-selected', 'count-and-byte-bounds-enforced'])
        with fixture.open('ab') as stream:
            stream.write(b' ')
        try:
            verified(ref)
        except ValueError:
            checks.append('changed-original-receipt-rejected')
        else:
            raise AssertionError('changed original accepted')
        for name, mutate in [('missing-protected-list', lambda v: v['result'].pop('protectedDigests')),
                             ('wrong-root', lambda v: v['result'].update(buildRoot='C:\\target')),
                             ('wrong-operation', lambda v: v['result'].update(operationId='another'))]:
            value = json.loads(json.dumps(original))
            mutate(value)
            try:
                project(value, 'inventory', payload, ref)
            except ValueError:
                checks.append(name + '-rejected')
            else:
                raise AssertionError(name)
        busy = {'status': 'blocked', 'reasonCode': 'execution_busy', 'retryable': True}
        require(project(busy, 'inventory', payload, ref)['retryable'] is True, 'contention projection')
        checks.append('only-explicit-pre-execution-contention-retryable')
        try:
            project({'status': 'accepted', 'result': {**payload, 'removed': [], 'reclaimedBytes': 0,
                                                     'unknownProtected': False}}, 'gc', payload, ref)
        except ValueError:
            checks.append('unprotected-GC-rejected')
        else:
            raise AssertionError('unprotected GC accepted')
        return {'status': 'passed', 'checks': checks}
    finally:
        if fixture.exists():
            fixture.unlink()
        for p in files:
            if p.exists():
                p.unlink()
        if objects.exists():
            objects.rmdir()


def load(path, expected, name):
    require(checksum(read(path, limit=8 * 1024**2)) == expected, 'pinned dependency changed')
    sys.path.insert(0, str(REPO))
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def render(driver, runtime):
    parent = Path(__file__).absolute().parent
    paths = {'bindingModule': REPO / 'tools/dev/jenkins_cleanup_runtime.py',
             'nativeModule': REPO / 'tools/dev/local_cargo.py', 'checksModule': parent / 'run_checks.py'}
    plan = {'schemaVersion': 1, 'driverDigest': driver, 'runtimeOperationId': runtime,
            **{k: str(v) for k, v in paths.items()},
            'pins': {str(p): checksum(read(p, limit=8 * 1024**2)) for p in paths.values()}}
    raw = json.dumps(plan, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
    return {'plan': plan, 'planJson': raw.decode('utf-8'), 'sha256': checksum(raw)}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=('render', 'control', 'select'))
    for name in ('plan', 'plan-sha256', 'self-sha256', 'input-file', 'action', 'driver', 'runtime'):
        parser.add_argument('--' + name)
    args = parser.parse_args()
    if args.mode == 'render':
        require(SHA.fullmatch(args.driver or '') and ID.fullmatch(args.runtime or ''), 'render identity')
        print(json.dumps(render(args.driver, args.runtime), ensure_ascii=False))
        return 0
    require(os.environ.get('JOB_NAME') == 'zircon-scheduled-cleanup' and
            os.environ.get('NODE_NAME') == 'zircon-windows-agent' and
            Path(sys.executable).absolute() == REPO / '.jenkins/runtime/python/python.exe' and
            os.environ.get('BUILD_NUMBER', '').isdigit(), 'real scheduled cleanup agent required')
    require(checksum(read(__file__, limit=1024**2)) == args.self_sha256, 'scheduled helper changed')
    raw_plan = read(args.plan, limit=1024**2)
    require(checksum(raw_plan) == args.plan_sha256, 'scheduled plan changed')
    plan = json.loads(raw_plan)
    require(plan['schemaVersion'] == 1, 'plan schema')
    selected = json.loads(read(REPO / '.jenkins/state/deployment/driver.json', limit=1024**2))
    require(selected['driverDigest'] == plan['driverDigest'] and selected['runtimeOperationId'] == plan['runtimeOperationId']
            and plan['driverDigest'] == os.environ.get('ZIRCON_DRIVER_DIGEST')
            and plan['runtimeOperationId'] == os.environ.get('ZIRCON_RUNTIME_OPERATION_ID'), 'scheduled runtime changed')
    incoming = physical(args.input_file)
    require(incoming.parent == REPO / '.jenkins/state/controls', 'input must be an original control file')
    payload = json.loads(read(incoming, limit=65536))
    if args.mode == 'select':
        ref = payload['inventoryRef']
        require(physical(ref['path']).parent == REPO / '.jenkins/state/responses', 'inventory response location')
        value = verified(ref)
        require(value.get('status') == 'accepted' and value['result'].get('operationId') == payload['inventoryOperationId'],
                'original inventory operation differs')
        print(json.dumps(select(value['result'], payload['buildRoot'], payload['maxObjects'], payload['maxBytes'])))
        return 0
    require(args.action in {'inventory', 'gc'} and ID.fullmatch(payload.get('operationId', '')) and
            payload.get('buildRoot') in ROOTS, 'bounded maintenance action/root')
    require(payload.get('driverDigest') == plan['driverDigest'] and
            payload.get('runtimeOperationId') == plan['runtimeOperationId'], 'request binding differs')
    response = REPO / '.jenkins/state/responses' / incoming.name
    require(not response.exists(), 'original response exists; reconcile instead of repeating')
    output = physical(REPORTS / os.environ['BUILD_NUMBER'] / incoming.stem)
    require(not output.exists(), 'original native invocation exists; reconcile')
    output.mkdir(parents=True)
    manifest = {'taskId': 'jenkins-scheduled-cleanup', 'owner': os.environ['BUILD_NUMBER'], 'status': 'running',
                'input': str(incoming), 'response': str(response), 'sourceCopies': 0,
                'retention': 'Original response and native/summary receipts; private test metadata removed',
                'dependencies': plan, 'output': str(output)}
    save(output / 'artifacts.json', manifest)
    summary = {'status': 'failed', 'startedAt': time.time(), 'input': fact(incoming, read(incoming, limit=65536))}
    native_complete = False
    try:
        save(output / 'regression.json', regression(output))
        binding = load(plan['bindingModule'], plan['pins'][plan['bindingModule']], '_scheduled_binding')
        proof = binding.observe_binding(REPO, plan['driverDigest'])
        require(proof['runtimeOperationId'] == plan['runtimeOperationId'], 'native runtime generation changed')
        save(output / 'runtime-binding.json', proof)
        native = load(plan['nativeModule'], plan['pins'][plan['nativeModule']], '_scheduled_native')
        checks = load(plan['checksModule'], plan['pins'][plan['checksModule']], '_scheduled_checks')
        command = [str(sys.executable), '-B', str(physical(selected['launcher'])), '--repository', str(REPO),
                   'maintenance', args.action, '--input-file', str(incoming), '--output-file', str(response)]
        native_output = output / 'native'
        native_output.mkdir()
        terminal = native._run_owned_command(command, REPO, dict(os.environ, TEMP=str(output), TMP=str(output),
                                    TMPDIR=str(output), PYTHONDONTWRITEBYTECODE='1'), native_output, 240)
        save(output / 'native-terminal.json', terminal)
        native_complete = checks.native_complete(terminal, command, REPO)
        require(native_complete, 'control CLI native termination unproved')
        raw = read(response)
        ref = fact(response, raw)
        with (output / 'response.json').open('xb') as evidence:
            evidence.write(raw)
        view = project(json.loads(raw), args.action, payload, ref)
        busy = view.get('reasonCode') == 'execution_busy' and view.get('retryable') is True
        require(terminal['rootExitCode'] in (0, 2) if busy else terminal['rootExitCode'] == 0, 'control CLI failed')
        summary.update(status='busy' if busy else 'passed', nativeComplete=True, responseRef=ref,
                       projectedBytes=len(json.dumps(view).encode()), fullResponseBytes=len(raw))
        save(output / 'projection.json', view)
        print(json.dumps(view, ensure_ascii=False, separators=(',', ':')))
    except BaseException as error:
        summary['error'] = str(error)[:400]
        raise
    finally:
        summary['finishedAt'] = time.time()
        save(output / 'summary.json', summary)
        manifest.update(status=summary['status'], nativeComplete=native_complete,
                        activeOwnedProcesses=[] if native_complete else ['inspect-original-native-receipt'],
                        temporaryFixturesRemain=any((output / p).exists() for p in ('projection-fixture.json', 'selection-fixture')),
                        finishedAt=summary['finishedAt'])
        # Keep the original pre-creation manifest and a separate terminal closeout.
        save(output / 'closeout.json', manifest)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
