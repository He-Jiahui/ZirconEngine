"""Build a preview from stable source inputs and bind its actual DLL and EXE."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import struct
import tomllib
from types import SimpleNamespace
import uuid

if __package__ in (None, ""):
    import sys
    from pathlib import Path
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from tools.dev import local_cargo
from tools.jenkins.contracts import JenkinsError
from tools.build.zircon_build_runtime_manifest import write_runtime_artifact_manifest


def sha(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()


def ordinary(path: Path) -> Path:
    for part in (path, *path.parents):
        if part.exists():
            info = part.lstat()
            if part.is_symlink() or getattr(info, 'st_file_attributes', 0) & 1024:
                raise ValueError(f'Reparse path rejected: {part}')
    return path


def output_path(path: Path) -> Path:
    checked = local_cargo.validate_output_path(path)
    if not any(checked.is_relative_to(root / 'zircon-local') for root in local_cargo.storage_roots()):
        raise ValueError('Editor output must be below D/E/F:/cargo-targets/zircon-local')
    return ordinary(checked)


def source_inventory(source: Path) -> dict[str, str]:
    """Include dirty/new inputs and frozen external dependencies; exclude output/control trees."""
    excluded = {'.git', 'target', '__pycache__', '.pytest_cache'}
    root_excluded = {'.codex', '.jenkins', '.opencode', 'dev', 'node_modules'} if source.name != 'source' else set()
    roots = [('source', source)]
    for name in ('external', 'zr_vm'):
        if (source.parent / name).is_dir():
            roots.append((name, source.parent / name))
    result = {}
    for label, root in roots:
        ordinary(root)
        for directory, folders, files in os.walk(root, followlinks=False):
            omitted = excluded | (root_excluded if Path(directory) == source else set())
            folders[:] = sorted(name for name in folders if name not in omitted)
            for name in folders:
                ordinary(Path(directory) / name)
            for name in sorted(files):
                # Windows device names are not static compiler/source files.
                if name.split('.')[0].upper() in {'NUL', 'CON', 'PRN', 'AUX', *('COM' + str(n) for n in range(1, 10)), *('LPT' + str(n) for n in range(1, 10))}:
                    continue
                path = ordinary(Path(directory) / name)
                if not stat.S_ISREG(path.stat().st_mode):
                    raise ValueError(f'Nonordinary source input: {path}')
                result[label + '/' + path.relative_to(root).as_posix()] = sha(path)
    return result


def inventory_digest(mapping: dict[str, str]) -> str:
    rows = [{'path': name, 'sha256': mapping[name]} for name in sorted(mapping, key=str.casefold)]
    return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def copy_assets(source: Path, destination: Path, shipping: bool) -> None:
    for owner in ('zircon_runtime', 'zircon_editor'):
        root = ordinary(source / owner / 'assets')
        if not root.is_dir():
            raise ValueError(f'Assets missing: {root}')
        for directory, folders, files in os.walk(root, followlinks=False):
            relative = Path(directory).relative_to(root)
            folders[:] = sorted(name for name in folders if not (
                shipping and owner == 'zircon_runtime' and
                (relative / name).as_posix() == 'fonts/editor-ui-sources'))
            ordinary(Path(directory))
            output_path(destination / relative).mkdir(parents=True, exist_ok=True)
            for name in folders:
                ordinary(Path(directory) / name)
            for name in files:
                origin = ordinary(Path(directory) / name)
                target = output_path(destination / relative / name)
                if target.exists():
                    if sha(origin) != sha(target):
                        raise ValueError(f'Differing duplicate asset destination: {target}')
                    continue
                shutil.copyfile(origin, target)


def dependency_closure(source: Path, mapping: dict[str, str]) -> list[dict[str, str]]:
    roots = {'source': source, 'zr_vm': source.parent / 'zr_vm', 'external': source.parent / 'external'}
    closure = []
    def dependencies(value):
        if not isinstance(value, dict):
            return
        for key, child in value.items():
            if key in {'dependencies', 'dev-dependencies', 'build-dependencies'} and isinstance(child, dict):
                for name, dependency in child.items():
                    if isinstance(dependency, dict):
                        yield name, dependency
            else:
                yield from dependencies(child)
    pending = ['source/Cargo.toml']
    visited = set()
    while pending:
        name = pending.pop()
        if name in visited:
            continue
        visited.add(name)
        if name not in mapping:
            raise ValueError(f'Workspace manifest missing from sealed inventory: {name}')
        label, relative = name.split('/', 1)
        manifest = roots[label] / relative
        document = tomllib.loads(manifest.read_text(encoding='utf-8'))
        workspace = document.get('workspace', {})
        for member in workspace.get('members', []):
            for member_root in manifest.parent.glob(member):
                member_manifest = member_root / 'Cargo.toml'
                if member_manifest.is_file():
                    pending.append(label + '/' + member_manifest.relative_to(roots[label]).as_posix())
        for package, dependency in dependencies(document):
            if 'path' in dependency:
                actual = ordinary(manifest.parent / dependency['path']).resolve() / 'Cargo.toml'
                if not any(actual.is_relative_to(root) and label + '/' + actual.relative_to(root).as_posix() in mapping
                           for label, root in roots.items()):
                    raise ValueError(f'Local dependency escapes sealed source inventory: {package}: {actual}')
                closure.append({'package': package, 'manifest': name, 'path': dependency['path']})
                for actual_label, root in roots.items():
                    if actual.is_relative_to(root):
                        pending.append(actual_label + '/' + actual.relative_to(root).as_posix())
                        break
            if 'git' in dependency:
                closure.append({'package': package, 'manifest': name, 'git': dependency['git']})
    return closure


def prepare_snapshot(source: Path, destination: Path) -> dict[str, str]:
    destination = output_path(destination)
    if destination.exists():
        raise ValueError('Snapshot destination must be fresh')
    before = source_inventory(source)
    dependency_closure(source, before)
    if local_cargo.free_bytes(destination) < local_cargo.MIN_FREE_BYTES:
        raise ValueError('Snapshot drive requires 35 GiB free')
    destination.mkdir(parents=True)
    origins = {'source': source, 'zr_vm': source.parent / 'zr_vm', 'external': source.parent / 'external'}
    for name, expected in before.items():
        label, relative = name.split('/', 1)
        original = ordinary(origins[label] / relative)
        target = output_path(destination / name)
        target.parent.mkdir(parents=True, exist_ok=True)
        if sha(original) != expected:
            raise ValueError('Source changed during snapshot copy')
        shutil.copyfile(original, target)
        if sha(target) != expected:
            raise ValueError('Snapshot copied bytes differ')
    if source_inventory(source) != before or source_inventory(destination / 'source') != before:
        raise ValueError('Snapshot source inventory changed')
    dependency_closure(destination / 'source', before)
    digest = inventory_digest(before)
    (destination / 'source-manifest.json').write_text(json.dumps(before, indent=2) + '\n', encoding='utf-8')
    receipt = {'status': 'sealed_source', 'source': str(destination / 'source'), 'source_digest': digest}
    (destination / 'source-seal.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    return receipt


def copy_crt(directory: Path | None, destination: Path, shell: Path, environment=None) -> dict[str, dict[str, str]]:
    if directory is None:
        redist = Path(os.environ.get('VSINSTALLDIR', 'E:/Visual Studio')) / 'VC/Redist/MSVC'
        candidates = sorted(redist.glob('*/x64/Microsoft.VC143.CRT'))
        if not candidates:
            raise ValueError('VS x64 CRT redist missing; supply CrtDirectory')
        directory = candidates[-1]
    directory = ordinary(directory)
    receipt = {}
    for name in ('msvcp140.dll', 'vcruntime140.dll', 'vcruntime140_1.dll'):
        path = ordinary(directory / name)
        data = path.read_bytes()
        offset = struct.unpack_from('<I', data, 0x3c)[0]
        if data[:2] != b'MZ' or data[offset:offset + 4] != b'PE\0\0' or struct.unpack_from('<H', data, offset + 4)[0] != 0x8664:
            raise ValueError('CRT input must be an x64 PE DLL')
        literal = str(path).replace("'", "''")
        command = f"$s = Get-AuthenticodeSignature -LiteralPath '{literal}'; if ($s.Status -ne 'Valid' -or $s.SignerCertificate.Subject -notmatch 'Microsoft') {{ exit 2 }}"
        result = subprocess.run([str(shell), '-NoProfile', '-NonInteractive', '-Command', command],
                                env=environment, capture_output=True, check=False, creationflags=subprocess.CREATE_NO_WINDOW)
        if result.returncode:
            raise ValueError(f'Microsoft CRT signature could not be verified: {path}')
        expected = sha(path)
        shutil.copyfile(path, output_path(destination / name))
        if sha(path) != expected or sha(destination / name) != expected:
            raise ValueError('CRT input changed during staging')
        receipt[name] = {'source': str(path), 'sha256': expected}
    return receipt


def build_commands(profile: str, jobs: int) -> list[list[str]]:
    feature = 'shipping-editor' if profile == 'shipping' else 'target-editor-host'
    common = ['--locked', '--no-default-features', '--features', feature, '--jobs', str(jobs),
              '--message-format=json-render-diagnostics']
    if profile == 'shipping':
        common += ['--profile', 'shipping']
    return [
        ['build', '-p', 'zircon_app', '--bin', 'zircon_editor', *common],
        ['rustc', '-p', 'zircon_runtime', '--lib', '--crate-type', 'cdylib', *common],
    ]


def reported_artifact(log: Path, name: str, target: Path) -> Path:
    matches = set()
    finished = False
    for line in log.read_text(encoding='utf-8', errors='replace').splitlines():
        try:
            row = json.loads(line)
        except json.JSONDecodeError:
            continue
        if row.get('reason') == 'build-finished':
            finished = row.get('success') is True
        if row.get('reason') == 'compiler-artifact':
            for filename in row.get('filenames', []):
                path = Path(filename)
                if path.name == name:
                    matches.add(output_path(path))
    if not finished or len(matches) != 1:
        raise ValueError(f'Cargo did not report one successful {name} artifact')
    artifact = matches.pop()
    if not artifact.is_relative_to(target):
        raise ValueError('Cargo artifact lies outside the selected target')
    return artifact


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo-root', type=Path, required=True)
    parser.add_argument('--output-directory', type=Path)
    parser.add_argument('--target-dir', type=Path)
    parser.add_argument('--cargo-profile', choices=['development', 'shipping'], default='development')
    parser.add_argument('--storage-mode', choices=['reuse', 'compact', 'diagnostic'], default='reuse')
    parser.add_argument('--source-snapshot', type=Path)
    parser.add_argument('--source-snapshot-digest')
    parser.add_argument('--jenkins-identity-file', type=Path)
    parser.add_argument('--jobs', type=int, default=2)
    parser.add_argument('--ephemeral', action='store_true')
    parser.add_argument('--skip-smoke-test', action='store_true')
    parser.add_argument('--crt-directory', type=Path)
    parser.add_argument('--prepare-source-snapshot', type=Path)
    args = parser.parse_args(argv)
    try:
        if os.name != 'nt':
            raise ValueError('Editor bundles require Windows')
        if args.jobs < 1:
            raise ValueError('Jobs must be positive')
        repo = ordinary(args.repo_root).resolve()
        if args.jenkins_identity_file:
            os.environ['ZIRCON_JENKINS_IDENTITY_FILE'] = str(args.jenkins_identity_file.resolve())
        if args.prepare_source_snapshot:
            print(json.dumps(prepare_snapshot(repo, args.prepare_source_snapshot)))
            return 0
        from tools.jenkins.frontend import sole_entry_enforced
        if sole_entry_enforced(repo):
            raise ValueError('Jenkins sole-entry gate is active; this independent preview will not dispatch a service')
        if bool(args.source_snapshot) != bool(args.source_snapshot_digest):
            raise ValueError('SourceSnapshot and SourceSnapshotDigest must be paired')
        source = ordinary(args.source_snapshot).resolve() if args.source_snapshot else repo
        before = source_inventory(source)
        dependencies = dependency_closure(source, before)
        source_digest = inventory_digest(before)
        if args.source_snapshot and source_digest != args.source_snapshot_digest.lower():
            raise ValueError('SourceSnapshotDigest differs from the actual input inventory')
        root = Path('D:/cargo-targets/zircon-local')
        target = output_path(args.target_dir or root / 'windows' / ('editor-' + args.cargo_profile) / 'target')
        if args.ephemeral:
            target = output_path(target.parent / ('ephemeral-' + uuid.uuid4().hex) / 'target')
        final = output_path(args.output_directory or root / ('editor-preview-' + uuid.uuid4().hex))
        if not final.parent.is_dir() or final.exists():
            raise ValueError('Output parent must exist and final bundle must be fresh')
        if min(local_cargo.free_bytes(target), local_cargo.free_bytes(final)) < local_cargo.MIN_FREE_BYTES:
            raise ValueError('Target and bundle drives require 35 GiB free')
        # Preserve this owned sibling on failure for inspection; never clean foreign artifacts.
        staging = output_path(final.parent / (final.name + '.staging-' + uuid.uuid4().hex))
        staging.mkdir()
        evidence = output_path(final.parent / 'evidence' / ('editor-preview-' + uuid.uuid4().hex))
        evidence.mkdir(parents=True)
        temporary = output_path(evidence / 'temporary')
        temporary.mkdir()
        environment = dict(os.environ)
        environment.update({'TEMP': str(temporary), 'TMP': str(temporary), 'TMPDIR': str(temporary),
                            'PYTHONDONTWRITEBYTECODE': '1'})
        shell = Path(os.environ['SystemRoot']) / 'System32/WindowsPowerShell/v1.0/powershell.exe'
        commands = build_commands(args.cargo_profile, args.jobs)
        artifacts = {}
        for index, command in enumerate(commands):
            print(f'Building {"editor executable" if index == 0 else "runtime DLL"}; evidence: {evidence}', flush=True)
            if source_inventory(source) != before:
                raise ValueError('Source changed before package build')
            invocation = [str(shell), '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass',
                          '-File', str(repo / 'tools/dev/local-cargo.ps1'), '-RepoRoot', str(source),
                          '-TargetDir', str(target), '-PythonInterpreter', sys.executable, '-IndependentPreview']
            if args.jenkins_identity_file:
                invocation += ['-JenkinsIdentityFile', str(args.jenkins_identity_file)]
            invocation += command
            with (evidence / f'cargo-{index}.log').open('wb') as log:
                completed = subprocess.run(invocation, env=environment, stdout=log, stderr=subprocess.STDOUT,
                                           check=False, creationflags=subprocess.CREATE_NO_WINDOW)
            if completed.returncode:
                raise ValueError(f'Cargo failed ({completed.returncode}); inspect {evidence / f"cargo-{index}.log"}')
            name = 'zircon_editor.exe' if index == 0 else 'zircon_runtime.dll'
            path = reported_artifact(evidence / f'cargo-{index}.log', name, target)
            artifacts[name] = (path, sha(path))
        for name in ('zircon_editor.exe', 'zircon_runtime.dll'):
            origin, expected = artifacts[name]
            ordinary(origin)
            if not origin.is_file() or not origin.stat().st_size:
                raise ValueError(f'Built artifact missing: {origin}')
            if sha(origin) != expected:
                raise ValueError(f'Cargo artifact changed after its build: {name}')
            shutil.copyfile(origin, staging / name)
            if sha(origin) != expected or sha(staging / name) != expected:
                raise ValueError(f'Artifact changed during staging: {name}')
        copy_assets(source, staging / 'assets', args.cargo_profile == 'shipping')
        crt = copy_crt(args.crt_directory, staging, shell, environment)
        feature = 'shipping-editor' if args.cargo_profile == 'shipping' else 'target-editor-host'
        manifest = write_runtime_artifact_manifest(SimpleNamespace(
            repo_root=source, engine_root=staging, mode='release' if args.cargo_profile == 'shipping' else 'debug',
            runtime_features=(feature,), dry_run=False), create_once=True)
        identity = json.loads(manifest.read_bytes())
        if identity['artifact']['sha256'] != sha(staging / 'zircon_runtime.dll') or identity['host_artifacts'] != [
            {'file_name': 'zircon_editor.exe', 'sha256': sha(staging / 'zircon_editor.exe')}]:
            raise ValueError('Runtime BuildSet artifact closure differs')
        if not args.skip_smoke_test:
            completed = subprocess.run([str(staging / 'zircon_editor.exe'), '--help'], cwd=staging,
                                       env=environment, capture_output=True, check=False,
                                       creationflags=subprocess.CREATE_NO_WINDOW)
            if completed.returncode or b'Usage: zircon_editor' not in completed.stdout + completed.stderr:
                raise ValueError('Editor --help smoke check failed')
        if source_inventory(source) != before:
            raise ValueError('Source changed; bundle publication rejected')
        receipt = {'status': 'preview_built', 'source_digest': source_digest, 'source_inventory': before,
                   'commands': commands, 'build_set_id': identity['build_set_id'],
                   'artifact': identity['artifact'], 'host_artifacts': identity['host_artifacts'],
                   'cargo_profile': args.cargo_profile, 'storage_mode': args.storage_mode,
                   'dependency_closure': dependencies, 'official_crt': crt,
                   'compiler_cache_enabled': False, 'smoke_tested': not args.skip_smoke_test,
                   'full_build_acceptance': False, 'native_ui_acceptance': False}
        (evidence / 'build-receipt.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
        ordinary(staging)
        output_path(final)
        if final.exists():
            raise ValueError('Output appeared before publication')
        staging.rename(final)
        print(json.dumps({'status': 'preview_built', 'output_directory': str(final),
                          'evidence_directory': str(evidence),
                          'build_set_id': identity['build_set_id'], 'full_build_acceptance': False}))
        return 0
    except (OSError, ValueError, JenkinsError, struct.error) as error:
        print(f'Editor build failed: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
