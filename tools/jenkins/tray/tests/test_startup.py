from __future__ import annotations
import hashlib, json, tempfile, unittest
from pathlib import Path
from types import SimpleNamespace
from tools.jenkins.tray import startup
from tools.jenkins.tray.config import TrayError

def config(tmp_path: Path):
    repo = tmp_path / "repo folder"
    (repo / "tools" / "jenkins" / "tray").mkdir(parents=True)
    (repo / "tools" / "jenkins" / "install-jenkins-tray-startup.ps1").write_text("# test", encoding="utf-8")
    return SimpleNamespace(repo_root=repo, pilot_root=tmp_path / "pilot", config_path=repo / "profile.json")

def key_for(cfg):
    material = (str(cfg.repo_root).casefold() + "\0" + str(cfg.pilot_root).casefold()).encode()
    return "ZirconJenkinsTray-" + hashlib.sha256(material).hexdigest()[:16]

class StartupTests(unittest.TestCase):
    def test_install_contract(self):
        with tempfile.TemporaryDirectory(dir=r"E:\cargo-targets\zircon-local\jenkins-support-tests\tmp") as folder:
            cfg, seen = config(Path(folder)), {}
            def fake(args, **kwargs):
                seen.update(args=args, kwargs=kwargs)
                return SimpleNamespace(returncode=0, stdout=json.dumps({"action":"Install","key":key_for(cfg),"installed":True}), stderr="")
            old = startup.subprocess.run; startup.subprocess.run = fake
            try: result = startup.install(cfg, dry_run=True)
            finally: startup.subprocess.run = old
            self.assertTrue(result["installed"]); self.assertIn("-DryRun", seen["args"])
            self.assertEqual(startup._value_name(cfg), key_for(cfg)); self.assertNotIn("--start", seen["args"])

    def test_mismatched_key_does_not_leak_output(self):
        with tempfile.TemporaryDirectory(dir=r"E:\cargo-targets\zircon-local\jenkins-support-tests\tmp") as folder:
            cfg, secret = config(Path(folder)), "agent-secret-do-not-return"
            old = startup.subprocess.run; startup.subprocess.run = lambda *a, **k: SimpleNamespace(returncode=0, stdout=json.dumps({"action":"Query","key":"foreign","password":secret}), stderr=secret)
            try:
                with self.assertRaisesRegex(TrayError, "Tray startup script failed") as raised: startup.query(cfg)
            finally: startup.subprocess.run = old
            self.assertNotIn(secret, str(raised.exception))

    def test_nonzero_result_is_rejected(self):
        with tempfile.TemporaryDirectory(dir=r"E:\cargo-targets\zircon-local\jenkins-support-tests\tmp") as folder:
            cfg = config(Path(folder)); old = startup.subprocess.run
            startup.subprocess.run = lambda *a, **k: SimpleNamespace(returncode=1, stdout="", stderr="bad")
            try:
                with self.assertRaises(TrayError): startup.query(cfg)
            finally: startup.subprocess.run = old

if __name__ == "__main__": unittest.main()
