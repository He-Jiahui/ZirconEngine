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
import zipfile

if __package__ in (None, ""):
    import sys
    from pathlib import Path
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from tools.dev import local_cargo
from tools.jenkins.contracts import JenkinsError
from tools.jenkins.validation.receipts import closure_manifest
from tools.build.zircon_build_product_profile import load_product_build_profile
from tools.build.zircon_build_runtime_manifest import write_runtime_artifact_manifest
from tools.build.zircon_build_size_report import (
    build_product_size_report, enforce_product_size_report, write_deterministic_archive,
)


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


def copy_assets(source: Path, destination: Path, shipping: bool) -> dict[str, str]:
    copied = {}
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
                expected = sha(origin)
                if target.exists():
                    if expected != sha(target):
                        raise ValueError(f'Differing duplicate asset destination: {target}')
                else:
                    shutil.copyfile(origin, target)
                if sha(origin) != expected or sha(target) != expected:
                    raise ValueError(f'Asset changed during staging: {origin}')
                copied[(relative / name).as_posix()] = expected
    return copied


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


def verify_x64_pe(path: Path, *, dll: bool) -> None:
    with ordinary(path).open('rb') as stream:
        header = stream.read(64)
        if len(header) != 64 or header[:2] != b'MZ':
            raise ValueError(f'Windows artifact is not an x64 PE: {path}')
        offset = struct.unpack_from('<I', header, 0x3c)[0]
        stream.seek(offset)
        coff = stream.read(24)
    if (offset < 64 or len(coff) != 24 or coff[:4] != b'PE\0\0'
            or struct.unpack_from('<H', coff, 4)[0] != 0x8664
            or bool(struct.unpack_from('<H', coff, 22)[0] & 0x2000) != dll):
        raise ValueError(f'Windows artifact is not an x64 PE {"DLL" if dll else "executable"}: {path}')


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
        verify_x64_pe(path, dll=True)
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
        common += ['--profile', 'shipping', '--target', 'x86_64-pc-windows-msvc']
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


def editor_bundle_closure(directory: Path) -> dict:
    directory = output_path(directory)
    paths = []
    for current, folders, files in os.walk(directory, followlinks=False):
        ordinary(Path(current))
        for name in folders:
            ordinary(Path(current) / name)
        for name in files:
            path = ordinary(Path(current) / name)
            if not stat.S_ISREG(path.stat().st_mode):
                raise ValueError(f'Nonordinary package file: {path}')
            paths.append(path)
    return closure_manifest(directory, paths)


def verify_editor_bundle(directory: Path, expected: dict) -> dict:
    actual = editor_bundle_closure(directory)
    if actual['files'] != expected.get('files') or actual['digest'] != expected.get('digest'):
        raise ValueError('Editor bundle content differs from its staged closure')
    return actual


def verify_editor_font_assets(assets: Path) -> None:
    assets = output_path(assets).resolve()
    for name in ('default.font.toml', 'editor-ui.font.toml'):
        manifest = ordinary(assets / 'fonts' / name)
        if not manifest.is_file():
            raise ValueError(f'Mandatory font manifest missing: {manifest}')
        descriptor = tomllib.loads(manifest.read_text(encoding='utf-8'))
        source = descriptor.get('source')
        if not isinstance(source, str) or not source.strip():
            raise ValueError(f'Mandatory font manifest source missing: {manifest}')
        relative = Path(source.strip())
        if relative.is_absolute() or relative.drive:
            raise ValueError(f'Font source must be relative to its manifest: {manifest}')
        payload = ordinary(manifest.parent / relative).resolve()
        if not payload.is_relative_to(assets):
            raise ValueError(f'Font source escapes the staged asset directory: {manifest}')
        payload = output_path(payload)
        if not payload.is_file() or not payload.stat().st_size:
            raise ValueError(f'Mandatory font payload missing or empty: {payload}')


def stage_editor_bundle(source: Path, staging: Path, artifacts: dict[str, tuple[Path, str]], *,
                        cargo_profile: str, source_files: dict[str, str],
                        crt_directory: Path | None, shell: Path, environment=None) -> dict:
    """Stage already-built artifacts; execution and acceptance stay with the caller."""
    if cargo_profile not in {'development', 'shipping'}:
        raise ValueError('Editor package profile must be development or shipping')
    if set(artifacts) != {'zircon_editor.exe', 'zircon_runtime.dll'}:
        raise ValueError('Editor package requires exactly its EXE and runtime DLL')
    source = ordinary(source).resolve()
    staging = output_path(staging)
    if not staging.is_dir() or any(staging.iterdir()):
        raise ValueError('Editor staging directory must exist and be empty')
    for name, (origin, expected) in artifacts.items():
        origin = output_path(origin)
        if origin.name != name or not origin.is_file() or sha(origin) != expected:
            raise ValueError(f'Built artifact differs from its reported identity: {name}')
        verify_x64_pe(origin, dll=name.endswith('.dll'))
        shutil.copyfile(origin, staging / name)
        if sha(origin) != expected or sha(staging / name) != expected:
            raise ValueError(f'Artifact changed during staging: {name}')
    assets = copy_assets(source, staging / 'assets', cargo_profile == 'shipping')
    verify_editor_font_assets(staging / 'assets')
    crt = copy_crt(crt_directory, staging, shell, environment)
    feature = 'shipping-editor' if cargo_profile == 'shipping' else 'target-editor-host'
    manifest = write_runtime_artifact_manifest(SimpleNamespace(
        repo_root=source, engine_root=staging, mode='release' if cargo_profile == 'shipping' else 'debug',
        runtime_features=(feature,), dry_run=False,
        effective_target_triple='x86_64-pc-windows-msvc' if cargo_profile == 'shipping' else None), create_once=True)
    manifest_bytes = manifest.read_bytes()
    identity = json.loads(manifest_bytes)
    if identity['artifact'] != {'file_name': 'zircon_runtime.dll', 'sha256': artifacts['zircon_runtime.dll'][1]} or identity['host_artifacts'] != [
        {'file_name': 'zircon_editor.exe', 'sha256': artifacts['zircon_editor.exe'][1]}]:
        raise ValueError('Runtime BuildSet artifact closure differs')
    if source_inventory(source) != source_files:
        raise ValueError('Source changed during editor staging')
    expected_files = {name: artifact[1] for name, artifact in artifacts.items()}
    expected_files.update({'assets/' + name: value for name, value in assets.items()})
    expected_files.update({name: value['sha256'] for name, value in crt.items()})
    expected_files[manifest.relative_to(staging).as_posix()] = hashlib.sha256(manifest_bytes).hexdigest()
    closure = editor_bundle_closure(staging)
    if {entry['path']: entry['sha256'] for entry in closure['files']} != expected_files:
        raise ValueError('Staged contents differ from copied asset and artifact identities')
    return {'build_set': identity, 'official_crt': crt, 'output_closure': closure}


def publish_editor_bundle(staging: Path, final: Path, expected: dict) -> dict:
    staging, final = output_path(staging), output_path(final)
    if staging.parent != final.parent or final.exists():
        raise ValueError('Editor publication requires a fresh sibling destination')
    verify_editor_bundle(staging, expected)
    staging.rename(final)
    return verify_editor_bundle(final, expected)


def verify_editor_archive(archive: Path, expected: dict) -> dict:
    archive = output_path(archive)
    if not archive.is_file():
        raise ValueError('Editor ZIP must be an ordinary file')
    identity = {'path': str(archive), 'bytes': archive.stat().st_size, 'sha256': sha(archive)}
    files = {entry['path']: entry for entry in expected['files']}
    seen = set()
    try:
        with zipfile.ZipFile(archive) as package:
            for member in package.infolist():
                name = member.filename
                if (member.is_dir() or '\\' in name or ':' in name
                        or any(ord(character) < 32 for character in name)
                        or any(part in {'', '.', '..'} for part in name.split('/'))):
                    raise ValueError(f'Editor ZIP contains an unsafe member path: {name!r}')
                if name in seen or name not in files:
                    raise ValueError(f'Editor ZIP member set differs from its staged closure: {name}')
                seen.add(name)
                entry = files[name]
                digest = hashlib.sha256()
                size = 0
                with package.open(member) as stream:
                    for block in iter(lambda: stream.read(1024 * 1024), b''):
                        digest.update(block)
                        size += len(block)
                if member.file_size != entry['size'] or size != entry['size'] or digest.hexdigest() != entry['sha256']:
                    raise ValueError(f'Editor ZIP member differs from its staged closure: {name}')
    except (zipfile.BadZipFile, RuntimeError, EOFError) as error:
        raise ValueError(f'Editor ZIP could not be verified: {error}') from error
    if seen != set(files):
        raise ValueError('Editor ZIP member set differs from its staged closure')
    if archive.stat().st_size != identity['bytes'] or sha(archive) != identity['sha256']:
        raise ValueError('Editor ZIP changed during verification')
    return identity


def publish_editor_shipping_bundle(source: Path, staging: Path, final: Path, package: dict, *,
                                   source_files: dict[str, str]) -> dict:
    source = ordinary(source).resolve()
    staging, final = output_path(staging), output_path(final)
    archive = output_path(final.parent / (final.name + '.zip'))
    if staging.parent != final.parent or final.exists() or archive.exists():
        raise ValueError('Editor shipping publication requires fresh sibling folder and ZIP destinations')
    if source_inventory(source) != source_files:
        raise ValueError('Source changed before editor shipping publication')
    profile = load_product_build_profile('editor-windows-shipping',
        path=ordinary(source / 'tools/export/product_build_profiles.toml'))
    composition = dict(product_kind='editor', cargo_profile='shipping', app_package='zircon_app',
        app_binary='zircon_editor', runtime_package='zircon_runtime', runtime_library='zircon_runtime.dll',
        runtime_features=('shipping-editor',), app_features=('shipping-editor',),
        plugin_carrier='none', asset_scope='editor', include_project_pack=False,
        symbols='external', hard_gate=True, archive=True, target_triple='x86_64-pc-windows-msvc')
    identity = package['build_set']
    if (any(getattr(profile, key) != value for key, value in composition.items())
            or identity.get('runtime_features') != ['shipping-editor'] or identity.get('build_mode') != 'release'
            or identity.get('target') != dict(architecture='x86_64', operating_system='windows',
                pointer_width=64, endian='little', target_triple=profile.target_triple)):
        raise ValueError('Editor shipping policy differs from its built composition or frozen BuildSet target')
    expected = package['output_closure']
    verify_editor_bundle(staging, expected)
    metadata = dict(build_set_id=package['build_set']['build_set_id'],
        product_profile=profile.name, product_kind=profile.product_kind,
        target_triple=profile.target_triple, cargo_profile=profile.cargo_profile,
        runtime_features=profile.runtime_features, app_features=profile.app_features, plugins=(),
        warning_bytes=profile.warning_bytes, hard_limit_bytes=profile.hard_limit_bytes,
        hard_gate=profile.hard_gate, forbidden_path_tokens=profile.forbidden_path_tokens,
        include_project_pack=profile.include_project_pack)

    def report_for(directory: Path, archive_path: Path | None = None) -> dict:
        report = build_product_size_report(directory, archive_path=archive_path, **metadata)
        reported = {entry['path']: (entry['sha256'], entry['bytes']) for entry in report['files']}
        frozen = {entry['path']: (entry['sha256'], entry['size']) for entry in expected['files']}
        if reported != frozen:
            raise ValueError('Editor size report differs from its staged closure')
        enforce_product_size_report(report)
        return report

    # Enforce the folder budget before doing compression or publishing either output.
    report_for(staging)
    pending = output_path(final.parent / (final.name + '.zip.pending-' + uuid.uuid4().hex))
    helper_temporary = output_path(pending.with_suffix(pending.suffix + '.tmp'))
    if pending.exists() or helper_temporary.exists():
        raise ValueError('Editor ZIP staging paths must be fresh')
    write_deterministic_archive(staging, pending, product_kind=profile.product_kind,
        forbidden_path_tokens=profile.forbidden_path_tokens, include_project_pack=profile.include_project_pack)
    pending_identity = verify_editor_archive(pending, expected)
    verify_editor_bundle(staging, expected)
    if source_inventory(source) != source_files:
        raise ValueError('Source changed; editor shipping publication rejected')
    if archive.exists():
        raise ValueError('Editor ZIP output appeared before publication')
    published = publish_editor_bundle(staging, final, expected)
    # Preserve the owned folder if the second rename fails; a receipt is the pair's completion marker.
    output_path(pending).rename(output_path(archive))
    archive_identity = verify_editor_archive(archive, expected)
    if any(archive_identity[key] != pending_identity[key] for key in ('bytes', 'sha256')):
        raise ValueError('Editor ZIP changed during publication')
    report = report_for(final, archive)
    verify_editor_bundle(final, expected)
    if source_inventory(source) != source_files:
        raise ValueError('Source changed after editor shipping publication')
    final_archive_identity = verify_editor_archive(archive, expected)
    reported_archive = report.get('archive', {})
    if (any(final_archive_identity[key] != captured[key]
            for captured in (pending_identity, archive_identity) for key in ('bytes', 'sha256'))
            or any(reported_archive.get(key) != final_archive_identity[key] for key in ('bytes', 'sha256'))
            or reported_archive.get('path') != archive.name):
        raise ValueError('Editor ZIP identity differs from its frozen archive or final size report')
    return {'output_closure': published, 'archive': final_archive_identity, 'size_report': report}


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
        if args.storage_mode != 'reuse':
            raise ValueError('Editor builds support storage mode reuse; compact and diagnostic have no supported executor')
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
        if args.cargo_profile == 'shipping' and output_path(final.parent / (final.name + '.zip')).exists():
            raise ValueError('Editor shipping ZIP destination must be fresh')
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
        package = stage_editor_bundle(source, staging, artifacts, cargo_profile=args.cargo_profile,
                                     source_files=before, crt_directory=args.crt_directory,
                                     shell=shell, environment=environment)
        identity = package['build_set']
        if not args.skip_smoke_test:
            completed = subprocess.run([str(staging / 'zircon_editor.exe'), '--help'], cwd=staging,
                                       env=environment, capture_output=True, check=False,
                                       creationflags=subprocess.CREATE_NO_WINDOW)
            if completed.returncode or b'Usage: zircon_editor' not in completed.stdout + completed.stderr:
                raise ValueError('Editor --help smoke check failed')
        if source_inventory(source) != before:
            raise ValueError('Source changed; bundle publication rejected')
        shipping = None
        if args.cargo_profile == 'shipping':
            shipping = publish_editor_shipping_bundle(source, staging, final, package, source_files=before)
            published = shipping['output_closure']
            report_path = output_path(evidence / 'product-size-report.json')
            with report_path.open('x', encoding='utf-8') as stream:
                stream.write(json.dumps(shipping['size_report'], indent=2) + '\n')
        else:
            published = publish_editor_bundle(staging, final, package['output_closure'])
        status = 'shipping_packaged' if shipping else 'preview_built'
        receipt = {'status': status, 'source_digest': source_digest, 'source_inventory': before,
                   'commands': commands, 'build_set_id': identity['build_set_id'],
                   'artifact': identity['artifact'], 'host_artifacts': identity['host_artifacts'],
                   'cargo_profile': args.cargo_profile, 'storage_mode': args.storage_mode,
                   'dependency_closure': dependencies, 'official_crt': package['official_crt'],
                   'output_closure': published, 'package_bytes': sum(row['size'] for row in published['files']),
                   'compiler_cache_enabled': False, 'smoke_tested': not args.skip_smoke_test,
                   'completed_publication': True, 'full_build_acceptance': False, 'native_ui_acceptance': False}
        if shipping:
            receipt.update(archive=shipping['archive'], product_size_report=str(report_path),
                           product_size_report_sha256=sha(report_path), shipping_acceptance=False)
        with output_path(evidence / 'build-receipt.json').open('x', encoding='utf-8') as stream:
            stream.write(json.dumps(receipt, indent=2) + '\n')
        print(json.dumps({'status': status, 'output_directory': str(final),
                          'evidence_directory': str(evidence),
                          'build_set_id': identity['build_set_id'], 'bundle_digest': published['digest'],
                          'archive_path': shipping['archive']['path'] if shipping else None,
                          'full_build_acceptance': False}))
        return 0
    except (OSError, ValueError, JenkinsError, struct.error) as error:
        print(f'Editor build failed: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
