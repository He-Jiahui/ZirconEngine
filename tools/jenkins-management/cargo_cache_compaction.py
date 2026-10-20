"""Jenkins-only NTFS compaction of idle Cargo cache files; no file deletion.

The shared E: cache contains old compiled dependencies outside CAS/incremental
GC. Preserve those reusable bytes and Cargo timestamps while reclaiming physical
allocation. Do not mark source cleanup or product acceptance complete here.
"""
from __future__ import annotations

import argparse
from contextlib import contextmanager, ExitStack
import ctypes
from ctypes import wintypes as wt
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import stat
import sys
import time
import xml.etree.ElementTree as ET

REPO = Path('E:/Git/ZirconEngine')
REPORTS = Path('E:/cargo-targets/zircon-local/reports/jenkins-scheduled-cleanup/compaction')
PROFILE = Path('E:/cargo-targets/zircon-local/win32/codex-default/build/debug')
RUN = re.compile(r'compact-[0-9]+\Z')
ARTIFACT = re.compile(r'[A-Za-z_][A-Za-z0-9_.-]*-[0-9a-f]{16}(?:\.[A-Za-z0-9_]+)*\.(?:rlib|rmeta|pdb|o)\Z')
GIB = 2 ** 30
BEGIN = '// BEGIN ZIRCON CARGO CACHE COMPACTION'
END = '// END ZIRCON CARGO CACHE COMPACTION'


def physical(value):
    path = Path(os.path.abspath(value))
    for parent in (path, *path.parents):
        try:
            info = parent.lstat()
        except FileNotFoundError:
            continue
        if info.st_file_attributes & 0x400 or stat.S_ISLNK(info.st_mode):
            raise ValueError('physical path required: ' + str(parent))
    return path


def checked(path, expected):
    raw = physical(path).read_bytes()
    if hashlib.sha256(raw).hexdigest() != expected:
        raise ValueError('pinned input changed: ' + str(path))
    return raw


def save(path, value):
    path = physical(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('w', encoding='utf-8', newline='\n') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.flush()
        os.fsync(stream.fileno())


def load_module(name, path, expected):
    raw = checked(path, expected)
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    exec(compile(raw, str(path), 'exec'), module.__dict__)
    return module


class FileInfo(ctypes.Structure):
    _fields_ = [('attributes', wt.DWORD), ('created', wt.FILETIME),
                ('accessed', wt.FILETIME), ('written', wt.FILETIME),
                ('volume', wt.DWORD), ('sizeHigh', wt.DWORD), ('sizeLow', wt.DWORD),
                ('links', wt.DWORD), ('indexHigh', wt.DWORD), ('indexLow', wt.DWORD)]


class CompressionInfo(ctypes.Structure):
    _fields_ = [('size', ctypes.c_int64), ('format', wt.WORD),
                ('unitShift', wt.BYTE), ('chunkShift', wt.BYTE),
                ('clusterShift', wt.BYTE), ('reserved', wt.BYTE * 3)]


class StandardInfo(ctypes.Structure):
    _fields_ = [('allocation', ctypes.c_int64), ('end', ctypes.c_int64),
                ('links', wt.DWORD), ('deletePending', wt.BYTE), ('directory', wt.BYTE)]


def api():
    k = ctypes.WinDLL('kernel32', use_last_error=True)
    k.CreateFileW.argtypes = [wt.LPCWSTR, wt.DWORD, wt.DWORD, ctypes.c_void_p,
                             wt.DWORD, wt.DWORD, wt.HANDLE]
    k.CreateFileW.restype = wt.HANDLE
    k.CloseHandle.argtypes = [wt.HANDLE]
    k.CloseHandle.restype = wt.BOOL
    k.GetFileInformationByHandle.argtypes = [wt.HANDLE, ctypes.POINTER(FileInfo)]
    k.GetFileInformationByHandle.restype = wt.BOOL
    k.GetFileInformationByHandleEx.argtypes = [wt.HANDLE, ctypes.c_int, ctypes.c_void_p, wt.DWORD]
    k.GetFileInformationByHandleEx.restype = wt.BOOL
    k.DeviceIoControl.argtypes = [wt.HANDLE, wt.DWORD, ctypes.c_void_p, wt.DWORD,
                                 ctypes.c_void_p, wt.DWORD, ctypes.POINTER(wt.DWORD), ctypes.c_void_p]
    k.DeviceIoControl.restype = wt.BOOL
    k.FlushFileBuffers.argtypes = [wt.HANDLE]
    k.FlushFileBuffers.restype = wt.BOOL
    k.SetFileTime.argtypes = [wt.HANDLE, ctypes.c_void_p, ctypes.c_void_p, ctypes.POINTER(wt.FILETIME)]
    k.SetFileTime.restype = wt.BOOL
    return k


@contextmanager
def directory_handles(root):
    k = api()
    handles = []
    try:
        for path in reversed((physical(root), *Path(root).parents)):
            physical(path)
            h = k.CreateFileW(str(path), 0x80, 3, None, 3, 0x02200000, None)
            if h == wt.HANDLE(-1).value:
                raise ctypes.WinError(ctypes.get_last_error())
            handles.append(h)
        yield
    finally:
        for h in reversed(handles):
            k.CloseHandle(h)


def information(k, h):
    f, c, s = FileInfo(), CompressionInfo(), StandardInfo()
    if not k.GetFileInformationByHandle(h, ctypes.byref(f)):
        raise ctypes.WinError(ctypes.get_last_error())
    if not k.GetFileInformationByHandleEx(h, 8, ctypes.byref(c), ctypes.sizeof(c)):
        raise ctypes.WinError(ctypes.get_last_error())
    if not k.GetFileInformationByHandleEx(h, 1, ctypes.byref(s), ctypes.sizeof(s)):
        raise ctypes.WinError(ctypes.get_last_error())
    return f, {'identity': [f.volume, f.indexHigh, f.indexLow],
               'bytes': (f.sizeHigh << 32) | f.sizeLow, 'links': f.links,
               'mtimeTicks': (f.written.dwHighDateTime << 32) | f.written.dwLowDateTime,
               'attributes': f.attributes, 'allocationBytes': s.allocation,
               'compressionInfoBytes': c.size, 'compressionFormat': c.format}


def file_hash(fd):
    os.lseek(fd, 0, os.SEEK_SET)
    sha = hashlib.sha256()
    while block := os.read(fd, 4 * 1024 * 1024):
        sha.update(block)
    return sha.hexdigest()


def compact_file(path, expected, journal, *, dry_run=False):
    """One original handle denies writes/deletion throughout hash/FSCTL/hash."""
    import msvcrt
    path = physical(path)
    k = api()
    h = k.CreateFileW(str(path), 0xC0000000, 1, None, 3, 0x00200000, None)
    if h == wt.HANDLE(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    fd = None
    try:
        fd = msvcrt.open_osfhandle(h, os.O_RDWR | os.O_BINARY)
        original, before = information(k, h)
        current = os.fstat(fd)
        if (before['bytes'] != expected['bytes'] or current.st_mtime_ns != expected['mtimeNs']
                or before['links'] != 1 or before['attributes'] & (0x10 | 0x400 | 0x4000 | 0x200)
                or before['compressionFormat'] != 0):
            raise ValueError('changed, linked, encrypted, sparse, aliased or compressed file retained')
        before['sha256'] = file_hash(fd)
        fact = {'path': str(path), 'before': before}
        journal.write(json.dumps({**fact, 'status': 'compression-intent' if not dry_run else 'dry-run'}) + '\n')
        journal.flush()
        os.fsync(journal.fileno())
        if dry_run:
            return {**fact, 'status': 'dry-run', 'allocationReclaimedBytes': 0}
        compression, returned = wt.USHORT(1), wt.DWORD()
        if not k.DeviceIoControl(h, 0x9C040, ctypes.byref(compression), 2, None, 0,
                                 ctypes.byref(returned), None):
            raise ctypes.WinError(ctypes.get_last_error())
        if not k.FlushFileBuffers(h):
            raise ctypes.WinError(ctypes.get_last_error())
        # Cargo freshness depends on the original last-write time.
        if not k.SetFileTime(h, None, None, ctypes.byref(original.written)):
            raise ctypes.WinError(ctypes.get_last_error())
        _, after = information(k, h)
        after['sha256'] = file_hash(fd)
        if (any(before[key] != after[key] for key in ('identity', 'bytes', 'links', 'mtimeTicks', 'sha256'))
                or after['compressionFormat'] == 0):
            raise ValueError('compaction postcondition failed; retain original intent and all bytes')
        result = {**fact, 'after': after, 'status': 'compressed',
                  'allocationReclaimedBytes': before['allocationBytes'] - after['allocationBytes']}
        journal.write(json.dumps(result) + '\n')
        journal.flush()
        os.fsync(journal.fileno())
        return result
    finally:
        if fd is not None:
            os.close(fd)
        else:
            k.CloseHandle(h)


def regression(output, cache_module):
    """Exercise the real Windows operation on owned bytes before touching cache."""
    fixture = physical(output / 'fixture')
    fixture.mkdir()
    p = fixture / 'libfixture-0123456789abcdef.rlib'
    raw = (b'Zircon compaction fixture\0' * 65536)
    p.write_bytes(raw)
    os.utime(p, ns=(1_700_000_000_000_000_000,) * 2)
    lock = fixture / '.cargo-lock'
    lock.write_bytes(b'')
    tests = []
    try:
        with directory_handles(fixture), cache_module.session_lock(lock):
            with (output / 'fixture-journal.jsonl').open('a', encoding='utf-8') as journal:
                expected = {'bytes': len(raw), 'mtimeNs': p.stat().st_mtime_ns}
                result = compact_file(p, expected, journal)
                assert result['allocationReclaimedBytes'] > 0 and p.read_bytes() == raw
                assert result['before']['allocationBytes'] >= len(raw)
                assert result['after']['allocationBytes'] == result['after']['compressionInfoBytes']
                tests.append('real-NTFS-compression-preserves-hash-identity-mtime')
                try:
                    compact_file(p, expected, journal)
                except ValueError:
                    tests.append('already-compressed-retained')
                else:
                    raise AssertionError('compressed file was admitted twice')
                busy = fixture / 'libbusy-0123456789abcdef.rlib'
                busy.write_bytes(raw)
                with busy.open('r+b'):
                    try:
                        compact_file(busy, {'bytes': len(raw), 'mtimeNs': busy.stat().st_mtime_ns}, journal)
                    except OSError:
                        tests.append('active-writer-sharing-denial-retained')
                    else:
                        raise AssertionError('active writer was not protected')
                busy.unlink()
                changed = fixture / 'libchanged-0123456789abcdef.rlib'
                changed.write_bytes(raw)
                try:
                    compact_file(changed, {'bytes': len(raw) + 1, 'mtimeNs': changed.stat().st_mtime_ns}, journal)
                except ValueError:
                    tests.append('changed-input-retained')
                else:
                    raise AssertionError('changed input was admitted')
                changed.unlink()
        tests.append('original-Cargo-lock-released')
        save(output / 'regression.json', {'status': 'passed', 'checks': tests,
             'fixtureReclaimedBytes': result['allocationReclaimedBytes'], 'productAcceptance': False})
    finally:
        # Only these exact, newly created test leaves; never recursive cleanup.
        for leaf in ('libfixture-0123456789abcdef.rlib', 'libbusy-0123456789abcdef.rlib',
                     'libchanged-0123456789abcdef.rlib', '.cargo-lock'):
            f = physical(fixture / leaf)
            if f.is_file():
                f.unlink()
        fixture.rmdir()


def ownership_header(path):
    """Read only the bounded leading ownership fields, never an activity journal."""
    import codecs
    path = physical(path)
    with path.open('rb') as stream:
        initial = os.fstat(stream.fileno())
        text = codecs.getincrementaldecoder('utf-8-sig')().decode(stream.read(65536), final=False)
        final = os.fstat(stream.fileno())
    identity = lambda s: (s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns)
    if identity(initial) != identity(final) or identity(final) != identity(path.stat()):
        raise ValueError('ownership manifest changed during bounded observation')
    decoder, cursor, fields = json.JSONDecoder(), 0, {}
    def whitespace(index):
        while index < len(text) and text[index].isspace():
            index += 1
        return index
    cursor = whitespace(cursor)
    if text[cursor:cursor + 1] != '{':
        raise ValueError('ownership manifest header must be an object')
    cursor += 1
    wanted = {'owner', 'task_id', 'output_roots'}
    try:
        for _ in range(32):
            cursor = whitespace(cursor)
            name, cursor = decoder.raw_decode(text, cursor)
            cursor = whitespace(cursor)
            if not isinstance(name, str) or text[cursor:cursor + 1] != ':':
                break
            value, cursor = decoder.raw_decode(text, whitespace(cursor + 1))
            if name in wanted:
                if name in fields:
                    raise ValueError('duplicate ownership header')
                fields[name] = value
            if set(fields) == wanted:
                return fields
            cursor = whitespace(cursor)
            if text[cursor:cursor + 1] != ',':
                break
            cursor += 1
    except json.JSONDecodeError as error:
        raise ValueError('ownership header exceeds bounded read') from error
    raise ValueError('complete bounded ownership header required')


def select_profile(plan, build_number):
    """Rotate explicit registered pools; an empty legacy pool cannot starve D/F."""
    if 'profiles' not in plan:
        root = physical(plan['profileRoot'])
        if root != PROFILE:
            raise ValueError('legacy compaction is limited to the reviewed shared pool')
        return root, {'kind': 'legacy-reviewed-shared-pool'}
    specs = plan['profiles']
    if (not isinstance(specs, list) or not 1 <= len(specs) <= 8
            or not str(build_number).isdigit() or int(build_number) <= 0):
        raise ValueError('bounded profile rotation requires a real build number')
    roots, bindings = [], []
    for spec in specs:
        raw = Path(spec['profileRoot'])
        if not raw.is_absolute() or '..' in raw.parts:
            raise ValueError('absolute physical Cargo profile required')
        root = physical(raw)
        base = Path(root.anchor) / 'cargo-targets/zircon-local/win32'
        if root.anchor[:2].upper() not in {'D:', 'E:', 'F:'} or not root.is_relative_to(base):
            raise ValueError('profile must be in a drive-root D/E/F Cargo namespace')
        relative = root.relative_to(base).parts
        if len(relative) != 3 or relative[1:] != ('build', 'debug') or root in roots:
            raise ValueError('one exact Win32 Cargo debug profile per registration required')
        if root == PROFILE:
            binding = {'kind': 'legacy-reviewed-shared-pool'}
        else:
            fields = ownership_header(spec['ownershipManifest'])
            outputs = fields.get('output_roots')
            if (fields.get('owner') != spec['ownerThreadId'] or fields.get('task_id') != spec['taskId']
                    or not isinstance(outputs, dict) or not isinstance(outputs.get('cargo_build_current'), str)
                    or physical(outputs['cargo_build_current']) != root.parent):
                raise ValueError('profile differs from its original ownership manifest')
            binding = {'owner': fields['owner'], 'taskId': fields['task_id'], 'buildRoot': str(root.parent)}
            checksum = hashlib.sha256(json.dumps(binding, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
            if checksum != spec['ownershipHeaderSha256']:
                raise ValueError('reviewed ownership header changed')
            binding.update(ownershipHeaderSha256=checksum, ownershipManifest=str(physical(spec['ownershipManifest'])))
        roots.append(root)
        bindings.append(binding)
    index = (int(build_number) - 1) % len(roots)
    return roots[index], {**bindings[index], 'rotationIndex': index, 'rotationCount': len(roots)}


def cargo_identity(root, binding):
    """Separate Cargo build dirs may retain fingerprints without rustc_info."""
    marker = physical(root.parent / '.rustc_info.json')
    if marker.is_file():
        raw = marker.read_bytes()
        data = json.loads(raw)
        if 'rustc_fingerprint' not in data or 'outputs' not in data:
            raise ValueError('Cargo compiler identity marker required')
        return {'kind': 'rustc-info', 'path': str(marker),
                'sha256': hashlib.sha256(raw).hexdigest()}
    if not binding.get('ownershipHeaderSha256'):
        raise ValueError('Cargo compiler identity marker required for legacy pool')
    fingerprints = physical(root / '.fingerprint')
    # Bounded recognition inside the acquired Cargo session lock. This does not
    # grant ownership, authorize deletion, or invent a compiler identity marker.
    with os.scandir(fingerprints) as directories:
        for index, directory in enumerate(directories):
            if index >= 128:
                break
            if not directory.is_dir(follow_symlinks=False):
                continue
            physical(directory.path)
            with os.scandir(directory.path) as entries:
                for number, entry in enumerate(entries):
                    if number >= 16:
                        break
                    if not entry.name.endswith('.json') or not entry.is_file(follow_symlinks=False):
                        continue
                    path = physical(entry.path)
                    with path.open('rb') as stream:
                        before = os.fstat(stream.fileno())
                        raw = stream.read(65537)
                        after = os.fstat(stream.fileno())
                    if (len(raw) > 65536 or before.st_size != len(raw)
                            or (before.st_ino, before.st_mtime_ns, before.st_size) !=
                               (after.st_ino, after.st_mtime_ns, after.st_size)):
                        continue
                    try:
                        data = json.loads(raw)
                    except (UnicodeError, ValueError):
                        continue
                    if (all(type(data.get(key)) is int for key in ('rustc', 'target', 'profile'))
                            and isinstance(data.get('deps'), list) and isinstance(data.get('local'), list)):
                        return {'kind': 'owned-build-dir-cargo-fingerprint', 'path': str(path),
                                'sha256': hashlib.sha256(raw).hexdigest(),
                                'rustc': data['rustc'], 'target': data['target'], 'profile': data['profile'],
                                'rustcInfoMissing': True}
    raise ValueError('registered pool has no bounded Cargo compiler identity evidence')


def load_cache(plan):
    # The maintenance actor is versioned independently of its original worktree.
    # Load the cache module's direct import from its exact pinned sibling, never
    # from an inherited Python path or a stale unrelated checkout_commit module.
    dependency = str(physical(Path(plan['cacheModule']).with_name('checkout_commit.py')))
    load_module('checkout_commit', dependency, plan['pins'][dependency])
    return load_module('_cargo_compact_cache', plan['cacheModule'], plan['pins'][plan['cacheModule']])


def worker(plan, output, dry_run):
    started = time.monotonic()
    root, binding = select_profile(plan, os.environ['BUILD_NUMBER'])
    if plan['minimumFreeBytes'] != 80 * GIB:
        raise ValueError('reviewed Cargo cache pressure threshold required')
    cache = load_cache(plan)
    regression(output, cache)
    result = {'status': 'running', 'scope': str(root / 'deps'), 'profileBinding': binding, 'dryRun': dry_run,
              'freeBefore': shutil.disk_usage(root.anchor).free, 'allocationReclaimedBytes': 0,
              'filesCompressed': 0, 'logicalBytesProcessed': 0, 'retained': [],
              'sourceDeleted': 0, 'cacheFilesDeleted': 0, 'sourceCopies': 0, 'globalCleanupComplete': False}
    result.update(sourceSha256=plan['pins'][str(Path(__file__).absolute())],
                  maxSeconds=plan['maxSeconds'], budgetExhausted=False)
    receipt = output / 'receipt.json'
    save(receipt, result)
    try:
        if result['freeBefore'] >= plan['minimumFreeBytes']:
            result['status'] = 'enough-free-space'
            return
        deps = physical(root / 'deps')
        with directory_handles(deps), cache.session_lock(root / '.cargo-lock'):
            result['compilerIdentity'] = cargo_identity(root, binding)
            save(receipt, result)
            selected = []
            for entry in os.scandir(deps):
                st = entry.stat(follow_symlinks=False)
                if (not stat.S_ISREG(st.st_mode) or st.st_file_attributes & (0x400 | 0x800 | 0x4000 | 0x200)
                        or not ARTIFACT.fullmatch(entry.name) or st.st_size < 1024 * 1024
                        or time.time_ns() - st.st_mtime_ns < 600 * 10**9):
                    continue
                selected.append({'path': str(Path(entry.path)), 'bytes': st.st_size, 'mtimeNs': st.st_mtime_ns})
            selected.sort(key=lambda row: row['bytes'], reverse=True)
            selected = selected[:plan['maxFiles']]
            save(output / 'selection.json', {'profileLock': str(root / '.cargo-lock'), 'files': selected,
                 'candidateLogicalBytes': sum(row['bytes'] for row in selected),
                 'method': 'native-handle exclusive write/delete exclusion, SHA256 before/after, NTFS LZNT1; no deletion'})
            with (output / 'operations.jsonl').open('a', encoding='utf-8') as journal:
                for row in selected:
                    if shutil.disk_usage(root.anchor).free >= plan['minimumFreeBytes']:
                        break
                    if time.monotonic() - started >= plan['maxSeconds']:
                        result['budgetExhausted'] = True
                        result['retained'].append({'reason': 'time-budget; continue with next scheduled run'})
                        break
                    if result['logicalBytesProcessed'] + row['bytes'] > plan['maxLogicalBytes']:
                        result['retained'].append({'path': row['path'], 'reason': 'logical-byte-budget'})
                        continue
                    try:
                        done = compact_file(Path(row['path']), row, journal, dry_run=dry_run)
                    except (OSError, ValueError) as error:
                        result['retained'].append({'path': row['path'], 'reason': str(error)})
                    else:
                        result['allocationReclaimedBytes'] += done['allocationReclaimedBytes']
                        result['filesCompressed'] += done['status'] == 'compressed'
                        result['logicalBytesProcessed'] += row['bytes']
                    save(receipt, result)
                    print(json.dumps({'path': row['path'], 'files': result['filesCompressed'],
                          'reclaimedBytes': result['allocationReclaimedBytes'],
                          'freeBytes': shutil.disk_usage(root.anchor).free}), flush=True)
            result['status'] = 'dry-run' if dry_run else ('pressure-relieved' if
                shutil.disk_usage(root.anchor).free >= plan['minimumFreeBytes'] else 'needs-review')
    except cache.CacheBusy as error:
        result.update(status='active-Cargo-writer-retained', reason=str(error))
    except BaseException as error:
        result.update(status='failed', error=str(error))
        raise
    finally:
        result.update(freeAfter=shutil.disk_usage(root.anchor).free, finishedAt=time.time())
        result['observedFreeDelta'] = result['freeAfter'] - result['freeBefore']
        result['spaceAccounting'] = 'Same-handle FileStandardInfo.AllocationSize delta; volume free delta also includes other writers'
        save(receipt, result)


def install(plan_path, plan_sha256, self_sha256):
    """Preserve the live job and timer; install only this bounded maintenance stage."""
    from checkout_commit import transport
    plan = json.loads(checked(plan_path, plan_sha256))
    for path, checksum in plan['pins'].items():
        checked(path, checksum)
    checked(__file__, self_sha256)
    template = r'''// BEGIN ZIRCON CARGO CACHE COMPACTION
def zirconCargoCacheCompaction() {
  node('zircon-windows-agent') {
    timeout(time: 28, unit: 'MINUTES') {
      dir('E:/Git/ZirconEngine/.jenkins/agent/workspace/zircon-scheduled-cleanup') {
        int code = 1
        withEnv(['ZIRCON_COMPACT_DRY_RUN=' + (params.DRY_RUN == true ? '1' : '0')]) {
          try {
            code = powershell(returnStatus: true, label: 'Reclaim idle Cargo cache allocation without deleting artifacts', script: \'\'\'
$ErrorActionPreference = 'Stop'
if ((Get-FileHash -LiteralPath '@SOURCE@' -Algorithm SHA256).Hash.ToLowerInvariant() -ne '@SOURCE_SHA@') { throw 'Pinned compaction source changed' }
$compactArguments = @('launch', '--plan', '@POLICY@', '--plan-sha256', '@POLICY_SHA@', '--self-sha256', '@SOURCE_SHA@')
if ($env:ZIRCON_COMPACT_DRY_RUN -eq '1') { $compactArguments += '--dry-run' }
& 'E:/Git/ZirconEngine/.jenkins/runtime/python/python.exe' -B -X utf8 '@SOURCE@' @compactArguments
exit $LASTEXITCODE
\'\'\')
          } finally {
            def root = 'E:/cargo-targets/zircon-local/reports/jenkins-scheduled-cleanup/compaction/compact-' + env.BUILD_NUMBER
            for (String leaf : ['artifacts.json', 'summary.json', 'receipt.json', 'selection.json', 'regression.json', 'operations.jsonl', 'runtime-binding.json', 'native-terminal.json', 'native/stdout.bin', 'native/stderr.bin']) {
              if (fileExists(root + '/' + leaf)) {
                def target = 'compaction-' + leaf.replace('/', '-')
                def encoding = leaf.endsWith('.bin') ? 'Base64' : 'UTF-8'
                writeFile(file: target, text: readFile(file: root + '/' + leaf, encoding: encoding), encoding: encoding)
                archiveArtifacts(artifacts: target, fingerprint: false)
              }
            }
          }
        }
        if (code != 0) error('Cargo compaction failed; retain original native evidence')
        def result = readJson(readFile(file: 'E:/cargo-targets/zircon-local/reports/jenkins-scheduled-cleanup/compaction/compact-' + env.BUILD_NUMBER + '/summary.json', encoding: 'UTF-8'))
        if (result.nativeComplete != true || result.status != 'passed') error('Cargo compaction native termination unproved')
        def report = result.compaction
        env.ZIRCON_CARGO_COMPACT_ALLOCATION = report.allocationReclaimedBytes.toString()
        echo('Cargo cache physical allocation reclaimed=' + report.allocationReclaimedBytes + '; free=' + report.freeAfter + '; ' + report.status)
        if (!(report.status in ['pressure-relieved', 'enough-free-space', 'dry-run'])) currentBuild.result = 'UNSTABLE'
      }
    }
  }
}
zirconCargoCacheCompaction()
// END ZIRCON CARGO CACHE COMPACTION
'''
    addon = template.replace("\\'", "'")
    for key, value in {'@SOURCE@': Path(__file__).absolute().as_posix(), '@SOURCE_SHA@': self_sha256,
                       '@POLICY@': plan_path.as_posix(), '@POLICY_SHA@': plan_sha256}.items():
        addon = addon.replace(key, value)
    client = transport(REPO)
    route = '/job/zircon-scheduled-cleanup/config.xml'
    code, raw, _ = client.request(route)
    if code != 200:
        raise ValueError('cannot read live scheduled cleanup')
    doc = ET.fromstring(raw)
    script = doc.find('./definition/script')
    if script is None or '// BEGIN ZIRCON INCREMENTAL CACHE CLEANUP' not in (script.text or ''):
        raise ValueError('existing cache/CAS maintenance contract required')
    before = script.text
    if BEGIN in before or END in before:
        first, last = before.index(BEGIN), before.index(END) + len(END)
        before = before[:first] + before[last:].lstrip('\n')
    # The build list must show physical compaction separately from 0-byte CAS GC.
    before = before.replace("currentBuild.description = 'compiler cache ' +",
        "currentBuild.description = 'NTFS physical ' + (env.ZIRCON_CARGO_COMPACT_ALLOCATION ?: '0') + ' bytes; compiler cache ' +")
    script.text = addon + before
    desired = ET.tostring(doc, encoding='utf-8', xml_declaration=True)
    code, current, _ = client.request(route)
    if code != 200 or current != raw:
        raise ValueError('job changed during review; do not overwrite it')
    code, _, _ = client.request(route, method='POST', data=desired, content_type='application/xml')
    if code != 200:
        raise ValueError('compaction stage installation rejected')
    code, actual, _ = client.request(route)
    if code != 200 or ET.fromstring(actual).findtext('./definition/script') != script.text:
        raise ValueError('compaction installation readback differs')
    return {'status': 'installed', 'oldConfigSha256': hashlib.sha256(raw).hexdigest(),
            'newConfigSha256': hashlib.sha256(actual).hexdigest(),
            'addonSha256': hashlib.sha256(addon.encode()).hexdigest(), 'readbackMatched': True,
            'existingScriptPreserved': before == (ET.fromstring(raw).findtext('./definition/script') or ''),
            'schedulePreserved': True, 'cacheFilesDeleted': 0}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=('launch', 'worker', 'install'))
    parser.add_argument('--plan', required=True, type=Path)
    parser.add_argument('--plan-sha256', required=True)
    parser.add_argument('--self-sha256', required=True)
    parser.add_argument('--dry-run', action='store_true')
    args = parser.parse_args()
    if args.mode == 'install':
        print(json.dumps(install(args.plan, args.plan_sha256, args.self_sha256)))
        return 0
    if (os.environ.get('JOB_NAME') != 'zircon-scheduled-cleanup'
            or os.environ.get('NODE_NAME') != 'zircon-windows-agent'
            or Path(sys.executable).absolute() != REPO / '.jenkins/runtime/python/python.exe'
            or not str(os.environ.get('BUILD_NUMBER', '')).isdigit()):
        raise ValueError('real scheduled-cleanup Jenkins agent required')
    checked(__file__, args.self_sha256)
    plan = json.loads(checked(args.plan, args.plan_sha256))
    if (plan['schemaVersion'] != 1 or not 1 <= plan['maxFiles'] <= 512 or not 0 < plan['maxLogicalBytes'] <= 80 * GIB
            or not 0 < plan['maxSeconds'] <= 900):
        raise ValueError('invalid bounded compaction policy')
    select_profile(plan, os.environ['BUILD_NUMBER'])
    for path, checksum in plan['pins'].items():
        checked(path, checksum)
    run = 'compact-' + os.environ['BUILD_NUMBER']
    if not RUN.fullmatch(run):
        raise ValueError('invalid original run identity')
    output = physical(REPORTS / run)
    if args.mode == 'worker':
        worker(plan, output, args.dry_run)
        return 0
    if output.exists():
        raise ValueError('original compaction exists; reconcile instead of launching twice')
    output.mkdir(parents=True)
    save(output / 'artifacts.json', {'taskId': 'jenkins-scheduled-cleanup', 'owner': run,
         'status': 'running', 'output': str(output), 'dependencies': plan['pins'],
         'retention': 'Keep original compression intents, hash/identity receipts and native termination',
         'sourceCopies': 0, 'temporaryFixtures': str(output / 'fixture')})
    result = {'status': 'running', 'runId': run, 'productAcceptance': False, 'startedAt': time.time()}
    try:
        binding = load_module('_cargo_compact_binding', plan['bindingModule'], plan['pins'][plan['bindingModule']])
        proof = binding.observe_binding(REPO, plan['driverDigest'])
        if proof['runtimeOperationId'] != plan['runtimeOperationId']:
            raise ValueError('original runtime generation changed')
        save(output / 'runtime-binding.json', proof)
        native = load_module('_cargo_compact_native', plan['nativeModule'], plan['pins'][plan['nativeModule']])
        checks = load_module('_cargo_compact_checks', plan['checksModule'], plan['pins'][plan['checksModule']])
        command = [str(sys.executable), '-B', '-X', 'utf8', str(Path(__file__).absolute()), 'worker',
                   '--plan', str(args.plan), '--plan-sha256', args.plan_sha256, '--self-sha256', args.self_sha256]
        if args.dry_run:
            command.append('--dry-run')
        env = dict(os.environ, TEMP=str(output), TMP=str(output), TMPDIR=str(output), PYTHONDONTWRITEBYTECODE='1')
        native_output = output / 'native'
        native_output.mkdir()
        terminal = native._run_owned_command(command, REPO, env, native_output, 1500)
        complete = checks.native_complete(terminal, command, REPO)
        save(output / 'native-terminal.json', terminal)
        result.update(status='passed' if complete and terminal.get('rootExitCode') == 0 else 'failed',
                      nativeComplete=complete, exitCode=terminal.get('rootExitCode'))
        if (output / 'receipt.json').exists():
            result['compaction'] = json.loads((output / 'receipt.json').read_text(encoding='utf-8'))
    except BaseException as error:
        result.update(status='failed', error=str(error))
        raise
    finally:
        result['finishedAt'] = time.time()
        save(output / 'summary.json', result)
        manifest = json.loads((output / 'artifacts.json').read_text(encoding='utf-8'))
        stage = result.get('compaction', {})
        manifest.update(status=('needs-review' if stage.get('status') not in
                        {'pressure-relieved', 'enough-free-space', 'dry-run'} else 'delivered')
                        if result.get('nativeComplete') and result.get('exitCode') == 0 else 'failed',
                        nativeComplete=result.get('nativeComplete', False),
                        activeOwnedProcesses=[] if result.get('nativeComplete') else ['native outcome needs reconciliation'],
                        temporaryFixturesRemain=(output / 'fixture').exists(),
                        cacheFilesDeleted=0,
                        physicalAllocationReclaimedBytes=stage.get('allocationReclaimedBytes', 0),
                        finishedAt=result['finishedAt'], globalCleanupComplete=False)
        save(output / 'artifacts.json', manifest)
    print(json.dumps(result))
    return 0 if result['status'] == 'passed' else 1


if __name__ == '__main__':
    raise SystemExit(main())
