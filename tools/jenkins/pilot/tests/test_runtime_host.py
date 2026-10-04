"""Retain real native runtime asset pins across the first Java launch seam."""
import hashlib
import json
import os
import subprocess
import sys
import unittest
from unittest.mock import patch

from tools.jenkins.pilot import runtime_host
from tools.jenkins.pilot.contracts import PilotError, canonical_json
from tools.jenkins.pilot.storage import ManagedStorage
from tools.jenkins.pilot.tests.test_assets import AssetReuseTests


@unittest.skipUnless(os.name == "nt", "requires real Windows runtime file pins")
class RuntimeHostTests(unittest.TestCase):
    def test_runtime_bytes_pinned_at_first_java_popen_through_exit_then_released(self):
        fixture = AssetReuseTests()
        source, root, war_hash = fixture.fixture()
        prepared = fixture.run_prepare(source, root, war_hash)
        java = root / "jdk/bin/java.exe"
        original_popen = subprocess.Popen
        launches = []
        start_file = root / "logs/runtime-start-test.json"
        with ManagedStorage(root).backend() as backend:
            backend.ensure_directory("logs")

        def assert_pins():
            with self.assertRaises(OSError):
                java.write_bytes(b"replace-while-runtime-active")
            with self.assertRaises(OSError):
                os.rename(root / "war/jenkins.war", root / "war/renamed.war")

        class RunningChild:
            def __init__(child_self, process):
                child_self.process = process
            def __getattr__(child_self, name):
                return getattr(child_self.process, name)
            def wait(child_self):
                assert_pins()
                marker = json.loads(start_file.read_bytes())
                self.assertEqual(marker["childPid"], child_self.process.pid)
                self.assertEqual(marker["hostPid"], os.getpid())
                self.assertEqual(marker["runtimeAssets"], binding)
                code = child_self.process.wait(timeout=15)
                assert_pins()
                return code

        def first_java_popen(argv, **kwargs):
            self.assertEqual(argv[0], str(java))
            assert_pins()
            # Controller URL updates must remain possible while the host owns
            # the code pins; the mutable manifest itself is not retained.
            manifest = dict(prepared, controllerUrl="http://127.0.0.1:12345")
            ManagedStorage(root).atomic_write("pilot-manifest.json", canonical_json(manifest))
            launches.append(argv)
            # The fixture contains inert Java bytes. Execute a real child at
            # the first Java Popen seam to verify the complete wait lifetime.
            return RunningChild(original_popen([sys.executable, "-B", "-c", "pass"], **kwargs))

        binding = {"root": str(root), "kind": "controller", "proofSha256": prepared["assetSourceManifestSha256"]}
        with patch("tools.jenkins.pilot.assets.PRESERVED_WAR_SHA256", war_hash), \
             patch.object(runtime_host.subprocess, "Popen", side_effect=first_java_popen):
            self.assertEqual(runtime_host._run_child([str(java), "-version"], binding, start_file=start_file), 0)
        self.assertEqual(len(launches), 1)
        released = root / "jdk/bin/released-java.exe"
        os.rename(java, released)
        os.rename(released, java)

    def test_agent_archive_is_exact_and_pinned_through_child_wait(self):
        fixture = AssetReuseTests()
        source, root, war_hash = fixture.fixture()
        prepared = fixture.run_prepare(source, root, war_hash)
        jar = root / "war/agent.jar"
        ManagedStorage(root).atomic_write("war/agent.jar", b"fixed-war-derived-agent-fixture")
        binding = {"root": str(root), "kind": "agent", "proofSha256": prepared["assetSourceManifestSha256"],
                   "agentJarSha256": hashlib.sha256(jar.read_bytes()).hexdigest()}
        original_popen = subprocess.Popen

        class RunningChild:
            def __init__(child_self, process):
                child_self.process = process
            def wait(child_self):
                with self.assertRaises(OSError):
                    jar.write_bytes(b"changed-agent")
                return child_self.process.wait(timeout=15)

        def launch(argv, **kwargs):
            with self.assertRaises(OSError):
                os.rename(jar, root / "war/renamed-agent.jar")
            return RunningChild(original_popen([sys.executable, "-B", "-c", "pass"], **kwargs))

        with patch("tools.jenkins.pilot.assets.PRESERVED_WAR_SHA256", war_hash), \
             patch.object(runtime_host.subprocess, "Popen", side_effect=launch):
            self.assertEqual(runtime_host._run_child([str(root / "jdk/bin/java.exe"), "-jar", str(jar)], binding), 0)
        os.rename(jar, root / "war/released-agent.jar")

    def test_wrong_fixed_proof_denies_first_java_popen(self):
        fixture = AssetReuseTests()
        source, root, war_hash = fixture.fixture()
        fixture.run_prepare(source, root, war_hash)
        binding = {"root": str(root), "kind": "controller", "proofSha256": "0" * 64}
        with patch("tools.jenkins.pilot.assets.PRESERVED_WAR_SHA256", war_hash), \
             patch.object(runtime_host.subprocess, "Popen") as launch, self.assertRaises(PilotError):
            runtime_host._run_child([str(root / "jdk/bin/java.exe"), "-version"], binding)
        launch.assert_not_called()


if __name__ == "__main__":
    unittest.main()
