"""Real Windows lock/deletion regressions for the fixed Rust cache policy."""
import importlib.util
import io
from contextlib import redirect_stderr, redirect_stdout
import json
import os
from pathlib import Path
import sys
import subprocess
import tempfile
import time
import unittest
from unittest.mock import patch
import xml.etree.ElementTree as ET

sys.path.insert(0, str(Path(__file__).parents[1]))
import incremental_cache_cleanup as cleanup


@unittest.skipUnless(os.name == "nt", "Windows rustc session locking")
class IncrementalCacheCleanupTests(unittest.TestCase):
    def fixture(self):
        return tempfile.TemporaryDirectory(dir=os.environ["ZR_COMMIT_TEST_ROOT"])

    def session(self, root, suffix="abc", content=None):
        session = root / "cache/zircon_runtime-abcd" / ("s-abc-def-" + suffix)
        session.mkdir(parents=True)
        lock = session.parent / "s-abc-def.lock"
        lock.write_bytes(b"")
        for name, body in (content or {"dep-graph.bin": b"compiler metadata", "product.o": b"object"}).items():
            (session / name).write_bytes(body)
        old = time.time() - 3600
        os.utime(lock, (old, old))
        os.utime(session, (old, old))
        return session, lock

    def clean(self, root, **kwargs):
        # Explicit zero tests the existing native deletion contract; production
        # defaults and warm-cache behavior are exercised separately below.
        kwargs.setdefault("keep_sessions", 0)
        return cleanup.clean(root / "cache", root / "output", trigger_free=2**63, **kwargs)

    def test_default_preserves_last_completed_session_under_disk_pressure(self):
        with self.fixture() as directory:
            root = Path(directory)
            session, _ = self.session(root)
            result = cleanup.clean(root / "cache", root / "output", trigger_free=2**63)
            self.assertEqual(result["deletedFiles"], 0)
            self.assertEqual(result["warmSessionsRetained"], 1)
            self.assertEqual(result["sessions"][0]["reason"], "warm-incremental-session")
            self.assertTrue((session / "product.o").exists())

    def test_retires_old_sessions_but_keeps_latest_for_each_unit(self):
        with self.fixture() as directory:
            root = Path(directory)
            old, _ = self.session(root, suffix="old")
            new, _ = self.session(root, suffix="new")
            timestamp = time.time() - 1800
            os.utime(new, (timestamp, timestamp))
            result = self.clean(root, keep_sessions=1)
            self.assertFalse(old.exists())
            self.assertTrue((new / "product.o").exists())
            self.assertEqual(result["warmSessionsRetained"], 1)
            self.assertEqual(result["deletedFiles"], 2)

    def test_unknown_newer_session_does_not_evict_usable_warm_state(self):
        with self.fixture() as directory:
            root = Path(directory)
            good, _ = self.session(root, suffix="good")
            unknown, _ = self.session(root, suffix="unknown", content={"source.rs": b"source"})
            timestamp = time.time() - 1800
            os.utime(unknown, (timestamp, timestamp))
            result = self.clean(root, keep_sessions=1)
            self.assertEqual(result["deletedFiles"], 0)
            self.assertTrue((good / "product.o").exists())
            self.assertTrue((unknown / "source.rs").exists())

    def test_deletes_unlocked_session_and_accounts_hardlinks_without_deleting_other_names(self):
        with self.fixture() as directory:
            root = Path(directory)
            session, _ = self.session(root)
            retained = root / "retained.o"
            os.link(session / "product.o", retained)
            result = self.clean(root)
            self.assertEqual(result["status"], "complete")
            self.assertEqual(result["deletedFiles"], 2)
            self.assertEqual(result["logicalDeletedBytes"], len(b"compiler metadataobject"))
            self.assertFalse(session.exists())
            self.assertEqual(retained.read_bytes(), b"object")
            deleted = [json.loads(line) for line in (root / "output/deletions.jsonl").read_text().splitlines()
                       if json.loads(line)["status"] == "deleted"]
            shared = next(item for item in deleted if item["path"].endswith("product.o"))
            self.assertEqual(shared["hardLinksBefore"], 2)
            self.assertEqual(shared["unlinkedAllocationBytes"], 0)
            self.assertIsNone(result["actualFreedBytes"])

    def test_live_rustc_lock_preserves_every_cache_file(self):
        with self.fixture() as directory:
            root = Path(directory)
            session, lock = self.session(root)
            with cleanup.session_lock(lock):
                result = self.clean(root)
            self.assertEqual(result["deletedFiles"], 0)
            self.assertIn("locked", result["sessions"][0]["reason"])
            self.assertTrue((session / "product.o").exists())

    def test_unknown_source_content_protects_entire_session(self):
        with self.fixture() as directory:
            root = Path(directory)
            session, _ = self.session(root, content={"product.o": b"object", "source.rs": b"source"})
            result = self.clean(root)
            self.assertEqual(result["deletedFiles"], 0)
            self.assertEqual((session / "source.rs").read_bytes(), b"source")
            self.assertTrue((session / "product.o").exists())

    def test_new_session_and_over_budget_session_are_preserved(self):
        for settings in ({"minimum_age": 7200}, {"max_bytes": 1}):
            with self.subTest(settings=settings), self.fixture() as directory:
                root = Path(directory)
                session, _ = self.session(root)
                result = self.clean(root, **settings)
                self.assertEqual(result["deletedFiles"], 0)
                self.assertTrue(session.exists())

    def test_dry_run_preserves_objects_and_has_no_delete_journal_entries(self):
        with self.fixture() as directory:
            root = Path(directory)
            session, _ = self.session(root)
            result = self.clean(root, dry_run=True)
            self.assertGreater(result["selectedBytes"], 0)
            self.assertEqual(result["deletedFiles"], 0)
            self.assertTrue(session.exists())
            self.assertEqual((root / "output/deletions.jsonl").read_text(), "")

    def test_changed_file_is_not_deleted_using_an_old_metadata_plan(self):
        with self.fixture() as directory:
            root = Path(directory)
            session, _ = self.session(root)
            fact = cleanup.cache_files(session)[0]
            Path(fact["path"]).write_bytes(b"changed after selection")
            with self.assertRaisesRegex(ValueError, "changed"):
                cleanup.remove_cache_file(fact)
            self.assertTrue(Path(fact["path"]).exists())

    def test_unexpected_selection_failure_keeps_a_failed_receipt(self):
        with self.fixture() as directory:
            root = Path(directory)
            self.session(root)
            with patch.object(Path, "iterdir", side_effect=RuntimeError("selection failed")):
                with self.assertRaisesRegex(RuntimeError, "selection failed"):
                    self.clean(root)
            receipt = json.loads((root / "output/receipt.json").read_text())
            self.assertEqual(receipt["status"], "failed")
            self.assertEqual(receipt["deletedFiles"], 0)
            self.assertIn("finishedAt", receipt)

    def test_space_threshold_leaves_a_reusable_cache_intact(self):
        with self.fixture() as directory:
            root = Path(directory)
            session, _ = self.session(root)
            result = cleanup.clean(root / "cache", root / "output", trigger_free=0)
            self.assertEqual(result["status"], "enough-free-space")
            self.assertEqual(result["deletedFiles"], 0)
            self.assertTrue(session.exists())

    def test_install_preserves_original_job_schedule_governance_and_parameters(self):
        class Client:
            raw = b'<flow-definition><properties><cron>H/30 * * * *</cron></properties><definition><script>existingCasAndGovernance()</script></definition></flow-definition>'
            def request(self, path, *, method="GET", data=None, content_type=None):
                if method == "POST":
                    self.raw = data
                return 200, self.raw, {}
        client = Client()
        cleanup.install(cleanup.REPO, client)
        cleanup.install(cleanup.REPO, client)
        tree = ET.fromstring(client.raw)
        script = tree.findtext("definition/script")
        self.assertEqual(script.count(cleanup.BEGIN), 1)
        self.assertNotIn("@CACHE_KEYS@", script)
        self.assertNotIn("@ENTRY_SHA@", script)
        self.assertIn("incremental-runtime-binding.json", script)
        for key in cleanup.CACHE_POLICIES:
            self.assertIn(repr(key), script)
        self.assertTrue(script.endswith("existingCasAndGovernance()"))
        self.assertEqual(tree.findtext("properties/cron"), "H/30 * * * *")

    def test_scope_covers_audited_def_caches_without_authorizing_source_or_dependency_roots(self):
        paths = [path for path, _ in cleanup.CACHE_POLICIES.values()]
        self.assertEqual({"D:", "E:", "F:"}, {path.drive for path in paths})
        self.assertEqual(len(paths), len(set(paths)))
        self.assertGreaterEqual(len(paths), 20)
        for path, threshold in cleanup.CACHE_POLICIES.values():
            self.assertEqual(path.parts[1], "cargo-targets")
            self.assertEqual(path.name, "incremental")
            self.assertNotIn("cargo-home", path.parts)
            self.assertEqual(threshold, 80 * cleanup.GIB)

    def test_empty_namespace_is_explicit_and_never_claims_whole_volume_cleanup(self):
        with self.fixture() as directory:
            root = Path(directory)
            (root / "cache").mkdir()
            result = self.clean(root)
            self.assertEqual(result["status"], "cache-empty")
            self.assertEqual(result["discoveredSessions"], 0)
            self.assertTrue(result["namespaceCleanupComplete"])
            self.assertFalse(result["globalCleanupComplete"])

    def test_busy_cache_reports_retention_instead_of_claiming_namespace_cleanup_complete(self):
        with self.fixture() as directory:
            root = Path(directory)
            session, lock = self.session(root)
            with cleanup.session_lock(lock):
                result = self.clean(root)
            self.assertEqual(result["retainedSessions"], 1)
            self.assertFalse(result["namespaceCleanupComplete"])
            self.assertTrue(session.exists())

    def test_render_cli_is_read_only_and_exports_the_same_resolved_stage_as_install(self):
        stream = io.StringIO()
        with patch.object(cleanup, "transport", side_effect=AssertionError("render connected to Jenkins")), patch.object(
                sys, "argv", ["cleanup", "render"]), redirect_stdout(stream):
            cleanup.main()
        rendered = json.loads(stream.getvalue())
        self.assertEqual(len(rendered["scope"]), len(cleanup.CACHE_POLICIES))
        self.assertEqual(rendered["sourcePins"]["entry"]["path"], str(Path(cleanup.__file__).absolute()))
        self.assertIn("incremental-runtime-binding.json", rendered["script"])
        self.assertNotRegex(rendered["script"], r"@[A-Z_]+@")
        self.assertEqual(rendered, cleanup.render_stage(cleanup.REPO))

    def test_standard_powershell_installer_parses_and_uses_resolved_renderer(self):
        installer = Path(__file__).parents[3] / "tools/dev/install-jenkins-scheduled-cleanup.ps1"
        command = "$tokens=$null; $errors=$null; [void][System.Management.Automation.Language.Parser]::ParseFile('" + str(
            installer).replace("'", "''") + "',[ref]$tokens,[ref]$errors); if($errors.Count){$errors | Out-String | Write-Error; exit 1}"
        result = subprocess.run(["C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe", "-NoProfile",
                                 "-NonInteractive", "-Command", command], capture_output=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr.decode("utf8", "replace"))
        source = installer.read_text(encoding="utf-8-sig")
        self.assertIn("$taskCacheModulePath render", source)
        self.assertIn("$taskCacheRender.sourcePins.entry.sha256", source)
        self.assertNotIn("(Get-Content -LiteralPath $taskCacheSourcePath -Raw) +", source)

    def test_real_cas_selector_preserves_large_protected_and_unknown_sets_and_budgets(self):
        source = (Path(__file__).parents[1] / "zircon-scheduled-cleanup.groovy").read_text()
        start = source.index("script: '''", source.index("def selectCandidates")) + len("script: '''")
        selector = source[start:source.index("'''))", start)].replace(chr(92) * 2, chr(92))
        self.assertIn("select --plan $env:ZIRCON_SCHEDULED_CONTROL_PLAN", selector)
        self.assertIn("Get-FileHash", selector)
        import scheduled_control
        with self.fixture() as directory:
            root = Path(directory)
            objects = root / "objects"
            objects.mkdir()
            protected = [format(value, "064x") for value in range(1, 30001)]
            protected.append("AB" * 32)
            for name in protected[29000:30000] + ["ab" * 32, "cd" * 32, "e" * 64, "f" * 64, "unknown-source.rs"]:
                (objects / name).write_bytes(b"obj" if name != "f" * 64 else b"four")
            build_root = str(Path("E:/cargo-targets"))
            inventory = {"buildRoot": build_root, "protectedDigests": protected, "unknownDigests": ["cd" * 32]}
            expected_locator = Path(build_root) / "zircon-local/zircon-jenkins/artifacts/artifacts/objects"
            def object_locator(path):
                self.assertEqual(expected_locator, path)
                return objects
            # Inject only the locator; execute the production selector and its digest/budget guards.
            with patch.object(scheduled_control, "physical", side_effect=object_locator):
                self.assertEqual(scheduled_control.select(inventory, build_root, 1, 3),
                                 {"digests": ["e" * 64], "plannedBytes": 3, "candidateCount": 1})
                self.assertEqual(scheduled_control.select(inventory, build_root, 0, 3)["candidateCount"], 0)
                self.assertEqual(scheduled_control.select(inventory, build_root, 1, 2)["candidateCount"], 0)
            with self.assertRaisesRegex(ValueError, "inventory root/digest lists"):
                scheduled_control.select({"buildRoot": build_root}, build_root, 1, 3)
            self.assertEqual(len(list(objects.iterdir())), 1005)

    def test_production_cli_refuses_local_deletion(self):
        with patch.dict(os.environ, {"JOB_NAME": "", "NODE_NAME": ""}), patch.object(
                sys, "argv", ["cleanup", "run", "--run-id", "incremental-1"]):
            with self.assertRaisesRegex(ValueError, "requires.*Jenkins"):
                cleanup.main()

    def test_production_cli_rejects_an_arbitrary_cleanup_path(self):
        with patch.dict(os.environ, {"JOB_NAME": "zircon-scheduled-cleanup", "NODE_NAME": "zircon-windows-agent"}), patch.object(
                sys, "argv", ["cleanup", "run", "--run-id", "incremental-1", "--cache-key", "D:/Git/source"]):
            with redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as error:
                cleanup.main()
            self.assertEqual(error.exception.code, 2)


if __name__ == "__main__":
    unittest.main(verbosity=2)
