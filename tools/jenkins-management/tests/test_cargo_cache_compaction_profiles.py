"""Ownership, bounded reads and deterministic scheduled Cargo pool rotation."""
from pathlib import Path
import hashlib
import importlib.util
import json
import os
import pytest

pytestmark = pytest.mark.skipif(os.name != 'nt', reason='Windows physical Cargo namespace and NTFS contract')
SOURCE = Path(__file__).parents[1] / 'cargo_cache_compaction.py'
spec = importlib.util.spec_from_file_location('compaction_profiles_under_test', SOURCE)
c = importlib.util.module_from_spec(spec)
spec.loader.exec_module(c)


def registered(tmp_path, **changes):
    root = Path('D:/cargo-targets/zircon-local/win32/owned-task/build/debug')
    metadata = {'owner': 'original-thread', 'task_id': 'owned-task',
                'output_roots': {'cargo_build_current': str(root.parent)},
                'processes': ['retained-history'] * 12000}
    metadata.update(changes)
    path = tmp_path / 'artifacts.json'
    path.write_text(json.dumps(metadata), encoding='utf-8')
    binding = {'owner': 'original-thread', 'taskId': 'owned-task', 'buildRoot': str(root.parent)}
    return {'profileRoot': str(root), 'ownershipManifest': str(path),
            'ownerThreadId': 'original-thread', 'taskId': 'owned-task',
            'ownershipHeaderSha256': hashlib.sha256(json.dumps(binding, sort_keys=True, separators=(',', ':')).encode()).hexdigest()}


def test_empty_legacy_pool_cannot_starve_registered_pool(tmp_path):
    registered_pool = registered(tmp_path)
    plan = {'profiles': [{'profileRoot': str(c.PROFILE)}, registered_pool]}
    assert c.select_profile(plan, '405')[0] == c.PROFILE
    root, binding = c.select_profile(plan, '406')
    assert root == Path(registered_pool['profileRoot'])
    assert binding['owner'] == 'original-thread' and binding['rotationIndex'] == 1
    assert c.select_profile(plan, '407')[0] == c.PROFILE


def test_large_activity_journal_is_neither_loaded_nor_an_owner_change(tmp_path):
    value = registered(tmp_path)
    assert Path(value['ownershipManifest']).stat().st_size > 65536
    root, original = c.select_profile({'profiles': [value]}, '1')
    path = Path(value['ownershipManifest'])
    document = json.loads(path.read_text())
    document['processes'].append('new history, same owner and physical namespace')
    path.write_text(json.dumps(document), encoding='utf-8')
    assert c.select_profile({'profiles': [value]}, '2') == (root, original)


@pytest.mark.parametrize('change', [
    {'owner': 'foreign-thread'}, {'task_id': 'another-task'},
    {'output_roots': {'cargo_build_current': 'F:/cargo-targets/zircon-local/win32/another/build'}}])
def test_changed_original_owner_or_output_root_is_rejected(tmp_path, change):
    value = registered(tmp_path, **change)
    with pytest.raises(ValueError, match='original ownership'):
        c.select_profile({'profiles': [value]}, '1')


@pytest.mark.parametrize('root', ['C:/cargo-targets/zircon-local/win32/task/build/debug',
    'D:/targets/zircon-local/win32/task/build/debug',
    'E:/Git/ZirconEngine/build/debug',
    'D:/cargo-targets/zircon-local/win32/task/source/build/debug'])
def test_source_and_nonphysical_compiler_namespaces_are_rejected(root):
    with pytest.raises(ValueError):
        c.select_profile({'profiles': [{'profileRoot': root}]}, '1')


def test_changed_header_digest_does_not_adopt_same_named_pool(tmp_path):
    value = registered(tmp_path)
    value['ownershipHeaderSha256'] = '0' * 64
    with pytest.raises(ValueError, match='header changed'):
        c.select_profile({'profiles': [value]}, '1')


def test_missing_late_or_duplicate_ownership_is_not_inferred(tmp_path):
    value = registered(tmp_path)
    path = Path(value['ownershipManifest'])
    path.write_text('{"processes":"' + 'x' * 70000 + '","owner":"original-thread"}', encoding='utf-8')
    with pytest.raises(ValueError, match='bounded read'):
        c.select_profile({'profiles': [value]}, '1')
    path.write_text('{"owner":"original-thread","owner":"foreign","task_id":"owned-task","output_roots":{}}', encoding='utf-8')
    with pytest.raises(ValueError, match='duplicate'):
        c.select_profile({'profiles': [value]}, '1')


def test_legacy_policy_stays_narrow_and_duplicates_are_rejected(tmp_path):
    assert c.select_profile({'profileRoot': str(c.PROFILE)}, '1')[0] == c.PROFILE
    with pytest.raises(ValueError, match='legacy compaction'):
        c.select_profile({'profileRoot': 'D:/cargo-targets/zircon-local/win32/task/build/debug'}, '1')
    with pytest.raises(ValueError, match='one exact'):
        c.select_profile({'profiles': [{'profileRoot': str(c.PROFILE)}] * 2}, '1')
    with pytest.raises(ValueError, match='real build number'):
        c.select_profile({'profiles': [{'profileRoot': str(c.PROFILE)}]}, 'not-a-build')


def test_native_compression_keeps_hashes_mtime_and_active_writers(tmp_path, monkeypatch):
    # Exercise the actual independently published actor layout, with no implicit
    # dependency-directory entry in sys.path.
    module = SOURCE.with_name('incremental_cache_cleanup.py')
    dependency = SOURCE.with_name('checkout_commit.py')
    pins = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in (module, dependency)}
    cache = c.load_cache({'cacheModule': str(module), 'pins': pins})
    c.regression(tmp_path, cache)
    result = json.loads((tmp_path / 'regression.json').read_text())
    assert result['status'] == 'passed' and result['fixtureReclaimedBytes'] > 0
    assert 'active-writer-sharing-denial-retained' in result['checks']
    assert not (tmp_path / 'fixture').exists()


def test_registered_build_dir_uses_real_fingerprint_without_inventing_marker(tmp_path):
    root = tmp_path / 'build/debug'
    evidence = root / '.fingerprint/owned-0123456789abcdef/lib-owned.json'
    evidence.parent.mkdir(parents=True)
    raw = json.dumps({'rustc': 123, 'target': 456, 'profile': 789, 'deps': [], 'local': []}).encode()
    evidence.write_bytes(raw)
    identity = c.cargo_identity(root, {'ownershipHeaderSha256': 'reviewed'})
    assert identity['kind'] == 'owned-build-dir-cargo-fingerprint'
    assert identity['sha256'] == hashlib.sha256(raw).hexdigest()
    assert identity['rustcInfoMissing'] is True
    assert not (root.parent / '.rustc_info.json').exists()
    with pytest.raises(ValueError, match='legacy pool'):
        c.cargo_identity(root, {'kind': 'legacy-reviewed-shared-pool'})


def test_unknown_fingerprint_cannot_identify_registered_pool(tmp_path):
    root = tmp_path / 'build/debug'
    evidence = root / '.fingerprint/unknown/fake.json'
    evidence.parent.mkdir(parents=True)
    evidence.write_text('{"rustc": "guess", "target": 456, "profile": 789}')
    with pytest.raises(ValueError, match='no bounded Cargo'):
        c.cargo_identity(root, {'ownershipHeaderSha256': 'reviewed'})


def test_existing_invalid_marker_is_not_bypassed_by_fingerprint(tmp_path):
    root = tmp_path / 'build/debug'
    root.mkdir(parents=True)
    (root.parent / '.rustc_info.json').write_text('{}')
    with pytest.raises(ValueError, match='identity marker'):
        c.cargo_identity(root, {'ownershipHeaderSha256': 'reviewed'})


def test_independent_cache_actor_rejects_changed_direct_dependency():
    module = SOURCE.with_name('incremental_cache_cleanup.py')
    dependency = SOURCE.with_name('checkout_commit.py')
    with pytest.raises(ValueError, match='pinned input changed'):
        c.load_cache({'cacheModule': str(module), 'pins': {str(dependency): '0' * 64}})
