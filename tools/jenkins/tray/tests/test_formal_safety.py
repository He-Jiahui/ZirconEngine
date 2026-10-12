from __future__ import annotations
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.tray.formal import status
from tools.jenkins.tray.formal_startup import VALUE

class FormalSafetyTests(unittest.TestCase):
    def test_lost_lifecycle_is_degraded_and_cannot_be_stopped_by_pid(self):
        import json
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'deployment-operation.json').write_text(json.dumps({
                'component': 'lifecycle-host', 'state': 'running', 'hostPid': 1,
                'creationTime': 'old', 'executable': 'python.exe'}))
            config = type('C', (), {'url': 'http://127.0.0.1:53748/',
                'paths': type('P', (), {'state': root})(), 'spec': object(),
                'java': root / 'java', 'war': root / 'war'})()
            with patch('tools.jenkins.tray.formal.DeploymentManager.health', return_value={
                    'controller': False, 'ready': False}), \
                 patch('tools.jenkins.tray.formal.identity', return_value=None):
                result = status(config)
            self.assertEqual('degraded', result['state'])
            self.assertTrue(result['ownerLost'])
            self.assertTrue(result['canStart'])
            self.assertFalse(result['canStop'])

    def test_unknown_owner_cannot_stop(self):
        class C:
            url='http://127.0.0.1:53748/'
            state_file=Path(tempfile.gettempdir())/'missing-formal-state.json'
            paths=type('P',(),{'state':Path(tempfile.gettempdir())})()
            spec=type('S',(),{'controller':{'listenAddress':'127.0.0.1','httpPort':53748}})()
            java=war=Path('missing')
        with patch('tools.jenkins.tray.formal.DeploymentManager.health', return_value={'controller':True,'agent':True,'plugins':True,'ready':True}), patch('tools.jenkins.tray.formal.identity', return_value=None):
            result=status(C())
        self.assertFalse(result['canStop'])
        self.assertFalse(result['serviceReady'])

    def test_startup_value_name_is_formal(self):
        self.assertEqual(VALUE, 'ZirconFormalJenkinsTray')

    def test_startup_compatibility_command_uses_canonical_tray_bootstrap(self):
        from unittest.mock import patch
        from tools.jenkins.tray import formal_startup
        with patch.object(formal_startup, '_selected_successor', return_value=None):
            command = formal_startup.command(Path('E:/Git/ZirconEngine'), Path('E:/Git/ZirconEngine/.jenkins/deployment-spec.json'),
                                             Path('E:/Git/ZirconEngine/.jenkins/runtime/python/pythonw.exe'))
        self.assertIn('tools\\jenkins\\tray\\launch.py', command)
        self.assertNotIn('tools\\jenkins_tray\\launch.py', command)
        self.assertIn('--formal --config', command)

    def _assert_successor_command_is_release_pinned(self, kind):
        import hashlib
        import json
        import tempfile
        from tools.jenkins.tray import formal_startup
        with tempfile.TemporaryDirectory(dir=r'E:\cargo-targets\zircon-local\jenkins-support-tests\tmp') as folder:
            repo = Path(folder)
            release_dir = repo / '.jenkins' / 'state' / 'deployment'
            release_dir.mkdir(parents=True)
            release = release_dir / 'owned-successor.json'
            body = {'schemaVersion': 1, 'kind': kind,
                    'targetDriver': {'driverDigest': 'a' * 64}}
            release.write_text(json.dumps(body), encoding='utf-8')
            release_hash = hashlib.sha256(release.read_bytes()).hexdigest()
            (release_dir / 'driver.json').write_text(json.dumps({
                'driverDigest': 'a' * 64,
                'recoveryRelease': {'path': str(release.resolve()), 'sha256': release_hash},
            }), encoding='utf-8')
            command = formal_startup.command(repo, repo / '.jenkins/deployment-spec.json',
                                             repo / '.jenkins/runtime/python/pythonw.exe')
        self.assertIn('tools\\dev\\jenkins_desktop_tray.py', command)
        self.assertIn('--release-sha256 ' + release_hash, command)
        self.assertIn(' --run', command)
        self.assertNotIn('--formal --config', command)

    def test_owned_successor_command_is_release_pinned(self):
        self._assert_successor_command_is_release_pinned("formal-owned-recovery-upgrade-release")

    def test_departed_successor_command_is_release_pinned(self):
        self._assert_successor_command_is_release_pinned("formal-lost-owner-recovery-release")

    def test_owned_control_plane_is_service_ready_but_build_degraded(self):
        import json
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / 'deployment-operation.json').write_text(json.dumps({
                'component': 'lifecycle-host', 'state': 'running', 'hostPid': 1,
                'creationTime': 'birth', 'executable': 'python.exe',
                'controlPlaneOnly': True}))
            config = type('C', (), {'url': 'http://127.0.0.1:53748/',
                'paths': type('P', (), {'state': root})(), 'spec': object(),
                'java': root / 'java', 'war': root / 'war'})()
            with patch('tools.jenkins.tray.formal.DeploymentManager.health', return_value={
                    'controller': True, 'plugins': True, 'agent': False, 'ready': False}), \
                 patch('tools.jenkins.tray.formal.identity', return_value={
                     'birth': 'birth', 'executable': 'python.exe'}):
                result = status(config)
            self.assertEqual('degraded', result['state'])
            self.assertTrue(result['serviceReady'])
            self.assertFalse(result['controller']['ready'])
            self.assertIn('构建执行暂停', result['message'])
            self.assertTrue(result['canStop'])

if __name__ == '__main__': unittest.main()
