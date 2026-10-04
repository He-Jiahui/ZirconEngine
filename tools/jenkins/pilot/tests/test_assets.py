"""Asset reuse admits binaries while refusing historical state and bad proof."""
import hashlib
from pathlib import Path
import unittest
from unittest.mock import patch
import uuid
import io
import os
import zipfile

from tools.jenkins.pilot.assets import prepare_preserved_assets, verify_prepared_assets, PRESERVATION_PROOF, ASSET_PROOF
from tools.jenkins.pilot.assets import prepared_assets_context
from tools.jenkins.pilot import bootstrap
from tools.jenkins.pilot.jenkins_config import PilotPaths
from tools.jenkins.pilot.contracts import canonical_json, PilotError
from tools.jenkins.pilot.jenkins_config import REQUIRED_PLUGIN_IDS
from tools.jenkins.pilot.native.paths import WorkerStorage


class AssetReuseTests(unittest.TestCase):
    def fixture(self):
        parent = Path(r"D:\cargo-targets\zircon-jenkins")
        source = parent / f"jenkins-pilot-assets-source-{uuid.uuid4().hex}"
        target = parent / f"jenkins-pilot-assets-destination-{uuid.uuid4().hex}"
        names = list(REQUIRED_PLUGIN_IDS) + [f"test-plugin-{index}" for index in range(29-len(REQUIRED_PLUGIN_IDS))]
        war = b"fixture-war"
        files = {"war/jenkins.war": war, "jdk/bin/java.exe": b"fixture-java", "jdk/lib/library": b"fixture-library"}
        plugins = []
        for name in names:
            buffer = io.BytesIO()
            with zipfile.ZipFile(buffer, "w") as archive:
                archive.writestr("META-INF/MANIFEST.MF", name.encode())
                archive.writestr("WEB-INF/lib/plugin.jar", b"fixture-executable-jar")
            payload = buffer.getvalue()
            files[f"jenkins_home/plugins/{name}.jpi"] = payload
            plugins.append({"id": name, "version": "1.0", "sha256": hashlib.sha256(payload).hexdigest(),
                            "path": r"F:\stale-source\plugins\ignored.jpi"})
        manifest = canonical_json({"jenkinsVersion": "2.580.1", "plugins": plugins,
                                   "warPath": r"C:\stale\war", "driverRoot": "ignored"})
        files["pilot-manifest.json"] = manifest
        proof = {"allCopyHashesEqual": True, "destinationRoot": str(source),
                 "files": [{"path": name, "bytes": len(payload), "sha256": hashlib.sha256(payload).hexdigest()}
                           for name, payload in files.items()]}
        with WorkerStorage(source) as backend:
            for name, payload in files.items():
                backend.write_bytes(name, payload)
            backend.write_bytes(PRESERVATION_PROOF, canonical_json(proof))
            backend.write_bytes("credentials.json", b"never-copy-historical-credentials")
            backend.write_bytes("process-state.json", b"never-copy-process-state")
            backend.write_bytes("jenkins_home/jobs/old/config.xml", b"never-copy-old-job")
        with WorkerStorage(target):
            pass
        return source, target, hashlib.sha256(war).hexdigest()

    def run_prepare(self, source, target, war_hash):
        with patch("tools.jenkins.pilot.assets.require_live_storage_owner") as owner, \
             patch("tools.jenkins.pilot.assets.PRESERVED_WAR_SHA256", war_hash), \
             patch("tools.jenkins.pilot.assets.restrict_private_root"), \
             patch("tools.jenkins.pilot.bootstrap._java_major", return_value=21):
            result = prepare_preserved_assets(target, source, repo_root=Path.cwd())
            self.assertGreaterEqual(owner.call_count, 3)
            return result

    def test_fresh_copy_discards_old_absolute_paths_and_all_controller_state(self):
        source, target, war_hash = self.fixture()
        result = self.run_prepare(source, target, war_hash)
        self.assertTrue(result["prepared"])
        self.assertEqual(result["controllerHost"], "127.0.0.1")
        self.assertEqual((result["controllerExecutors"], result["agentExecutors"]), (0, 1))
        self.assertTrue(Path(result["warPath"]).is_relative_to(target))
        self.assertTrue(all(Path(plugin["path"]).is_relative_to(target) for plugin in result["plugins"]))
        self.assertNotIn("driverRoot", result)
        self.assertFalse((target / "process-state.json").exists())
        self.assertFalse((target / "jenkins_home/jobs").exists())
        self.assertNotEqual((target / "credentials.json").read_bytes(), (source / "credentials.json").read_bytes())
        self.assertEqual((source / "credentials.json").read_bytes(), b"never-copy-historical-credentials")
        self.assertEqual(hashlib.sha256((target / ASSET_PROOF).read_bytes()).hexdigest(), result["assetSourceManifestSha256"])
        with patch("tools.jenkins.pilot.assets.PRESERVED_WAR_SHA256", war_hash):
            self.assertEqual(verify_prepared_assets(target), result["assetSourceManifestSha256"])

    def test_historical_jdk_hash_mismatch_refuses_prepared_publication(self):
        source, target, war_hash = self.fixture()
        with WorkerStorage(source) as backend:
            backend.write_bytes("jdk/lib/library", b"modified-since-preservation")
        with self.assertRaisesRegex(PilotError, "historical copy proof"):
            self.run_prepare(source, target, war_hash)
        self.assertFalse((target / ASSET_PROOF).exists())
        self.assertFalse((target / "pilot-manifest.json").exists())

    def test_previous_home_state_refuses_copy_instead_of_adopting_it(self):
        source, target, war_hash = self.fixture()
        with WorkerStorage(target) as backend:
            backend.write_bytes("jenkins_home/config.xml", b"preexisting-controller")
        with self.assertRaisesRegex(PilotError, "controller state"):
            self.run_prepare(source, target, war_hash)
        self.assertEqual((target / "jenkins_home/config.xml").read_bytes(), b"preexisting-controller")
        self.assertFalse((target / "pilot-manifest.json").exists())

    def test_runtime_verification_rejects_tampered_jdk_and_war(self):
        for name in ("jdk/lib/library", "war/jenkins.war"):
            with self.subTest(name=name):
                source, target, war_hash = self.fixture()
                self.run_prepare(source, target, war_hash)
                with WorkerStorage(target) as backend:
                    backend.write_bytes(name, b"tampered-runtime-binary")
                with patch("tools.jenkins.pilot.assets.PRESERVED_WAR_SHA256", war_hash), self.assertRaises(PilotError):
                    verify_prepared_assets(target)

    def test_hpi_and_unlisted_expanded_code_are_rejected(self):
        for name in ("jenkins_home/plugins/evil.hpi", "jenkins_home/plugins/evil/WEB-INF/lib/evil.jar"):
            with self.subTest(name=name):
                source, target, war_hash = self.fixture()
                self.run_prepare(source, target, war_hash)
                with WorkerStorage(target) as backend:
                    backend.write_bytes(name, b"unlisted-executable-code")
                with patch("tools.jenkins.pilot.assets.PRESERVED_WAR_SHA256", war_hash), self.assertRaises(PilotError):
                    verify_prepared_assets(target)

    def test_known_expanded_plugin_matches_zip_and_rejects_tampered_code(self):
        source, target, war_hash = self.fixture()
        self.run_prepare(source, target, war_hash)
        prefix = "jenkins_home/plugins/workflow-job"
        with WorkerStorage(target) as backend:
            backend.write_bytes(prefix + "/META-INF/MANIFEST.MF", b"workflow-job")
            backend.write_bytes(prefix + "/WEB-INF/lib/plugin.jar", b"fixture-executable-jar")
            backend.write_bytes(prefix + "/.timestamp2", b"")
        with patch("tools.jenkins.pilot.assets.PRESERVED_WAR_SHA256", war_hash):
            verify_prepared_assets(target)
            with WorkerStorage(target) as backend:
                backend.write_bytes(prefix + "/WEB-INF/lib/plugin.jar", b"tampered-expanded-code")
            with self.assertRaises(PilotError):
                verify_prepared_assets(target)

    def test_asset_context_retains_binary_and_directory_pins_through_launch(self):
        source, target, war_hash = self.fixture()
        result = self.run_prepare(source, target, war_hash)
        with patch("tools.jenkins.pilot.assets.PRESERVED_WAR_SHA256", war_hash), prepared_assets_context(target) as digest:
            self.assertEqual(digest, result["assetSourceManifestSha256"])
            with self.assertRaises(PermissionError):
                os.rename(target / "jdk/bin/java.exe", target / "jdk/bin/replaced.exe")
            with self.assertRaises(PermissionError):
                os.rename(target / "jenkins_home/plugins", target / "jenkins_home/replaced-plugins")
            with WorkerStorage(target) as backend:
                with self.assertRaises(PilotError):
                    backend.write_bytes("jenkins_home/plugins/new.hpi", b"foreign")

    def test_tampered_startup_is_rejected_before_java_executes(self):
        source, target, war_hash = self.fixture()
        self.run_prepare(source, target, war_hash)
        with WorkerStorage(target) as backend:
            backend.write_bytes("jdk/bin/java.exe", b"tampered-executable")
        with patch("tools.jenkins.pilot.governance.require_live_storage_owner", return_value={}), \
             patch("tools.jenkins.pilot.assets.PRESERVED_WAR_SHA256", war_hash), \
             patch.object(bootstrap, "_java_major") as version, patch.object(bootstrap, "_start_process") as launch:
            with self.assertRaises(PilotError):
                bootstrap.start_controller(target)
            version.assert_not_called()
            launch.assert_not_called()

    def test_security_script_hardlink_is_rejected_without_truncating_foreign_file(self):
        source, target, _ = self.fixture()
        with WorkerStorage(target) as backend:
            backend.write_bytes("foreign-script", b"preserve-foreign")
            backend.ensure_directory("jenkins_home/init.groovy.d")
        os.link(target / "foreign-script", target / "jenkins_home/init.groovy.d/00-pilot-security.groovy")
        with self.assertRaises(PilotError):
            bootstrap._write_security_bootstrap(PilotPaths(target))
        self.assertEqual((target / "foreign-script").read_bytes(), b"preserve-foreign")

    def test_controller_guard_retains_manifest_and_java_through_launch_then_allows_url_update(self):
        source, target, war_hash = self.fixture()
        prepared = self.run_prepare(source, target, war_hash)
        record = {"pid": 20, "creationTime": "child", "keeperPid": 10,
                  "keeperCreationTime": "birth", "keeperJobHandle": 30}
        def launch(*arguments, **options):
            self.assertEqual(options["runtime_assets"], {"root": str(target), "kind": "controller",
                                                       "proofSha256": prepared["assetSourceManifestSha256"]})
            with self.assertRaises(PermissionError):
                os.rename(target / "jdk/bin/java.exe", target / "jdk/bin/swapped.exe")
            with self.assertRaises(PermissionError):
                os.rename(target / "pilot-manifest.json", target / "swapped-manifest.json")
            options["on_record"](record)
            return record
        with patch("tools.jenkins.pilot.governance.require_live_storage_owner", return_value={}), \
             patch("tools.jenkins.pilot.assets.PRESERVED_WAR_SHA256", war_hash), \
             patch("tools.jenkins.pilot.lifetime.wait_runtime_started"), \
             patch.object(bootstrap, "_java_major", return_value=21), \
             patch.object(bootstrap, "_start_process", side_effect=launch), patch.object(bootstrap, "_wait_controller"):
            result = bootstrap.start_controller(target)
        self.assertFalse(result["alreadyRunning"])
        self.assertEqual(bootstrap.read_manifest(target)["controllerUrl"], result["url"])

    def test_agent_archive_is_exact_war_member_and_existing_tamper_is_rejected(self):
        source, target, _ = self.fixture()
        archive_bytes = io.BytesIO()
        payload = b"exact-remoting-archive"
        with zipfile.ZipFile(archive_bytes, "w") as archive:
            archive.writestr("WEB-INF/lib/remoting-fixed.jar", payload)
        with WorkerStorage(target) as backend:
            backend.write_bytes("war/jenkins.war", archive_bytes.getvalue())
        expected = hashlib.sha256(payload).hexdigest()
        self.assertEqual(bootstrap._prepare_agent_archive(PilotPaths(target)), expected)
        self.assertEqual((target / "war/agent.jar").read_bytes(), payload)
        with WorkerStorage(target) as backend:
            backend.write_bytes("war/agent.jar", b"tampered-existing-agent")
        with self.assertRaises(PilotError):
            bootstrap._prepare_agent_archive(PilotPaths(target))
        self.assertEqual((target / "war/agent.jar").read_bytes(), b"tampered-existing-agent")

    def test_host_asset_context_requires_bound_digest_and_allows_manifest_fields_to_update(self):
        source, target, war_hash = self.fixture()
        prepared = self.run_prepare(source, target, war_hash)
        expected = prepared["assetSourceManifestSha256"]
        with patch("tools.jenkins.pilot.assets.PRESERVED_WAR_SHA256", war_hash):
            with prepared_assets_context(target, expected_digest=expected, retain_manifest=False) as digest:
                self.assertEqual(digest, expected)
                manifest = bootstrap.read_manifest(target)
                manifest["controllerUrl"] = "http://127.0.0.1:12345/"
                bootstrap.write_manifest(PilotPaths(target), manifest)
                with self.assertRaises(PermissionError):
                    os.rename(target / "jdk/bin/java.exe", target / "jdk/bin/replaced.exe")
            with self.assertRaises(PilotError):
                with prepared_assets_context(target, expected_digest="0" * 64, retain_manifest=False):
                    self.fail("changed proof binding admitted")


if __name__ == "__main__":
    unittest.main()
