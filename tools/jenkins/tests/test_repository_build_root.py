from __future__ import annotations

import copy
import json
import os
from pathlib import Path
import tempfile
import unittest
import uuid
import subprocess

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.paths import resolve_paths
from tools.jenkins.deployment.spec import load_spec
from tools.jenkins.resources.paths import canonical_build_root, physical_path_under


REPO = Path(__file__).absolute().parents[3]
BUILD_ROOT = REPO / '.jenkins/builds'


@unittest.skipUnless(os.name == 'nt', 'Windows physical path admission')
class RepositoryBuildRootTests(unittest.TestCase):
    def test_default_and_explicit_root_have_same_physical_identity(self):
        explicit = canonical_build_root(BUILD_ROOT)
        self.assertEqual(explicit.path, BUILD_ROOT)
        self.assertEqual(canonical_build_root(None), explicit)
        self.assertTrue(explicit.volume_identity)

    def test_similar_repository_paths_and_traversal_are_rejected(self):
        for path in (REPO / 'target', REPO / '.jenkins/jenkins_home',
                     REPO / '.jenkins/builds/nested', REPO / 'nested/.jenkins/builds',
                     str(BUILD_ROOT) + '\\..\\builds'):
            with self.subTest(path=path), self.assertRaises(JenkinsError):
                canonical_build_root(path)

    def test_build_paths_cannot_escape_into_home(self):
        root = canonical_build_root(BUILD_ROOT)
        candidate = root.namespace() / 'preparations/new/tmp'
        self.assertEqual(physical_path_under(root, candidate), candidate)
        with self.assertRaises(JenkinsError):
            physical_path_under(root, REPO / '.jenkins/jenkins_home/output')

    def test_formal_spec_accepts_only_the_selected_repository_root(self):
        original = json.loads((REPO / '.jenkins/deployment-spec.json').read_text(encoding='utf-8'))
        raw = copy.deepcopy(original)
        raw['storage']['buildRoot'] = str(BUILD_ROOT)
        raw['storage']['allowedPhysicalRoots'] = [str(BUILD_ROOT)]
        with tempfile.TemporaryDirectory() as directory:
            spec_path = Path(directory) / 'deployment.json'
            spec_path.write_text(json.dumps(raw), encoding='utf-8')
            paths = resolve_paths(load_spec(spec_path))
            self.assertEqual(paths.build_root, BUILD_ROOT)
            self.assertEqual(paths.build_namespace, BUILD_ROOT / 'zircon-jenkins')
            with self.assertRaises(JenkinsError):
                resolve_paths(load_spec(spec_path), r'D:\cargo-targets')
            raw['storage']['allowedPhysicalRoots'] = [str(REPO / 'nested/.jenkins/builds')]
            spec_path.write_text(json.dumps(raw), encoding='utf-8')
            with self.assertRaises(JenkinsError):
                load_spec(spec_path)

    def test_new_cli_writes_reject_historical_override_but_recovery_retains_it(self):
        from tools.jenkins.cli import configured_build_payload
        self.assertEqual(str(BUILD_ROOT), configured_build_payload('candidate', 'apply', {}, REPO)['buildRoot'])
        with self.assertRaises(JenkinsError):
            configured_build_payload('request', 'submit', {'buildRoot': r'D:\cargo-targets'}, REPO)
        historical = {'buildRoot': r'D:\cargo-targets'}
        self.assertEqual(historical, configured_build_payload('flow', 'reconcile-flow', historical, REPO))

    def test_descendant_junction_is_rejected(self):
        import shutil
        fixture = BUILD_ROOT / 'zircon-jenkins' / ('path-test-' + uuid.uuid4().hex)
        target = fixture / 'physical'
        junction = fixture / 'alias'
        target.mkdir(parents=True)
        try:
            result = subprocess.run(['cmd.exe', '/d', '/c', 'mklink', '/J', str(junction), str(target)], capture_output=True)
            if result.returncode:
                self.skipTest('Host cannot create a test junction')
            with self.assertRaises(JenkinsError):
                physical_path_under(canonical_build_root(BUILD_ROOT), junction / 'output')
        finally:
            if junction.exists():
                junction.rmdir()
            shutil.rmtree(fixture)
