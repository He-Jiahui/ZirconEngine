"""A deployed driver is an immutable, verified input to every workflow."""
from pathlib import Path
import tempfile
import unittest

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.driver import seal_driver, verify_driver, driver_environment


class DriverTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        source = self.root / "tools/jenkins"
        source.mkdir(parents=True)
        (source / "__main__.py").write_text("print('sealed')\n", encoding="utf-8")
        pipeline = self.root / ".jenkins/pipeline"
        pipeline.mkdir(parents=True)
        (pipeline / "steps.groovy").write_text("return this\n", encoding="utf-8")

    def test_same_inputs_share_snapshot_but_changes_get_new_identity(self):
        first = seal_driver(self.root)
        self.assertEqual(first, seal_driver(self.root))
        self.assertTrue(Path(first["launcher"]).is_file())
        (self.root / "tools/jenkins/__main__.py").write_text("print('new')\n", encoding="utf-8")
        second = seal_driver(self.root)
        self.assertNotEqual(first["driverDigest"], second["driverDigest"])
        self.assertEqual("print('sealed')\n", (Path(first["root"]) / "tools/jenkins/__main__.py").read_text())

    def test_corrupt_deployed_source_cannot_be_reused(self):
        snapshot = seal_driver(self.root)
        (Path(snapshot["root"]) / "tools/jenkins/__main__.py").write_text("print('tampered')\n")
        with self.assertRaises(JenkinsError):
            verify_driver(snapshot["root"], expected_digest=snapshot["driverDigest"])
        with self.assertRaises(JenkinsError):
            seal_driver(self.root)

    def test_manifest_cannot_redirect_verification_outside_snapshot(self):
        import json
        snapshot = seal_driver(self.root)
        path = Path(snapshot["root"]) / "driver-manifest.json"
        manifest = json.loads(path.read_text())
        manifest["files"]["../outside.py"] = {"sha256": "0" * 64, "bytes": 0}
        path.write_text(json.dumps(manifest))
        with self.assertRaises(JenkinsError):
            verify_driver(snapshot["root"], expected_digest=snapshot["driverDigest"])

    def test_unlisted_import_cannot_enter_a_verified_snapshot(self):
        snapshot = seal_driver(self.root)
        (Path(snapshot["root"]) / "tools/jenkins/extra.py").write_text("pass\n")
        with self.assertRaises(JenkinsError):
            verify_driver(snapshot["root"], expected_digest=snapshot["driverDigest"])

    def test_driver_environment_requires_dedicated_python(self):
        import sys
        snapshot = seal_driver(self.root)
        with self.assertRaises(JenkinsError):
            driver_environment(snapshot, sys.executable, self.root)
        executable = self.root / ".jenkins/runtime/python/python.exe"
        executable.parent.mkdir(parents=True)
        executable.write_bytes(b"fixture")
        environment = driver_environment(snapshot, executable, self.root)
        self.assertEqual(str(executable), environment["JENKINS_PYTHON"])

    def test_invalid_python_cannot_be_selected_as_a_sealed_driver(self):
        (self.root / "tools/jenkins/broken.py").write_text("value = )\n", encoding="utf-8")
        with self.assertRaises(JenkinsError) as rejected:
            seal_driver(self.root)
        self.assertEqual("driver_syntax_invalid", rejected.exception.code)
        self.assertFalse((self.root / ".jenkins/runtime/drivers").exists())


if __name__ == "__main__":
    unittest.main()
