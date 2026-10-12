"""Static contract checks for the pinned user-desktop tray launcher.

These tests intentionally do not start a tray, call Win32, touch Jenkins state,
or exercise the service.  The release preflight and the live desktop proof are
separate acceptance steps owned by the deployment operator.
"""
from __future__ import annotations

import ast
from copy import deepcopy
import importlib.util
from pathlib import Path
import sys
import unittest
from unittest.mock import patch


REPO = Path(__file__).resolve().parents[3]
LAUNCHER_PATH = REPO / "tools/dev/jenkins_desktop_tray.py"


def _load_launcher():
    spec = importlib.util.spec_from_file_location("zircon_jenkins_desktop_tray_contract", LAUNCHER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("unable to load desktop launcher contract")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class DesktopTrayLauncherContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = LAUNCHER_PATH.read_text(encoding="utf-8")
        cls.tree = ast.parse(cls.source, filename=str(LAUNCHER_PATH))
        cls.launcher = _load_launcher()

    def test_formal_closure_is_explicit_and_canonical(self):
        expected = {
            "tools/jenkins/tray/__init__.py",
            "tools/jenkins/tray/__main__.py",
            "tools/jenkins/tray/formal.py",
            "tools/jenkins/tray/formal_app.py",
            "tools/jenkins/tray/formal_native.py",
            "tools/jenkins/tray/formal_startup.py",
            "tools/jenkins/tray/launch.py",
            "tools/jenkins/tray/menu.py",
            "tools/jenkins/tray/native_windows.py",
            "tools/jenkins/tray/operations.py",
            "tools/jenkins/tray/persistence.py",
            "tools/jenkins/tray/service.py",
            "tools/jenkins/tray/startup.py",
            "tools/jenkins/tray/terminal.py",
            "tools/jenkins/tray/win32_shell.py",
            "tools/jenkins/tray/worker_environment.py",
            "tools/jenkins/tray/app.py",
            "tools/jenkins/tray/config.py",
            "tools/jenkins/tray/lifecycle.py",
        }
        actual = {path.as_posix() for path in self.launcher.required_tray_source_relatives()}
        self.assertEqual(actual, expected)
        self.assertEqual(self.launcher.FORMAL_BOOTSTRAP_RELATIVE.as_posix(), "tools/jenkins/tray/launch.py")
        self.assertEqual(self.launcher.KEEPER_DESCRIPTOR_RELATIVE.as_posix(), ".jenkins/state/formal-lifecycle/recovery-keeper.json")
        self.assertNotIn("tools/jenkins_tray/", self.source)

    def test_release_sections_and_interpreter_pins_are_required(self):
        for token in (
            "operatorFiles",
            "runtimeFiles",
            "targetDriver",
            "deploymentSpec",
            "workflowSpec",
            "driver.json",
            "python.exe",
            "pythonw.exe",
            "driver-manifest.json",
            "recoveryRelease",
            "recovery-keeper.json",
        ):
            self.assertIn(token, self.source)
        self.assertIn("jenkins_desktop_tray.py", self.source)
        self.assertIn("launcher_unpinned", self.source)
        self.assertIn("actorCodeRoot", self.source)
        self.assertIn("formal-owned-recovery-upgrade-release", self.source)

    def test_operator_inventory_maps_sealed_actor_sources_only(self):
        repo = Path("E:/Git/ZirconEngine")
        actor = repo / ".jenkins/runtime/drivers" / ("a" * 64)
        tray = repo / "tools/jenkins/tray/formal_app.py"
        wrapper = repo / "tools/dev/jenkins_desktop_tray.py"
        self.assertEqual(
            self.launcher._effective_operator_path(str(tray), repo=repo, actor_root=actor),
            actor / "tools/jenkins/tray/formal_app.py",
        )
        self.assertEqual(
            self.launcher._effective_operator_path(str(wrapper), repo=repo, actor_root=actor),
            wrapper,
        )

    def test_user_desktop_identity_is_proven_before_formal_import(self):
        for token in (
            "OpenProcessToken",
            "GetTokenInformation",
            "ConvertSidToStringSidW",
            "ProcessIdToSessionId",
            "Shell_TrayWnd",
            "FindWindowW",
            "OpenInputDesktop",
            "OpenDesktopW",
            "SetThreadDesktop",
            "WinSta0",
            "Default",
        ):
            self.assertIn(token, self.source)
        import_index = self.source.index("from tools.jenkins.tray.formal_app import run_formal")
        self.assertLess(self.source.index("_verify_user_desktop()"), import_index)

    def test_release_cli_path_is_normalized_before_canonical_validation(self):
        """The argparse Path must cross the string-only release boundary."""
        calls = [
            node for node in ast.walk(self.tree)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Name)
            and node.func.id == "_canonical_file"
            and any(
                isinstance(keyword.value, ast.Constant)
                and keyword.arg == "label"
                and keyword.value.value == "release record"
                for keyword in node.keywords
            )
        ]
        self.assertEqual(len(calls), 1)
        self.assertEqual(ast.unparse(calls[0].args[0]), "os.fspath(release_path)")

    def test_default_desktop_access_includes_menu_creation(self):
        """OpenDesktop must grant the menu class access used by TrackPopupMenu."""
        self.assertIn("DESKTOP_CREATEMENU = 0x0004", self.source)
        access = next(
            node for node in ast.walk(self.tree)
            if isinstance(node, ast.Assign)
            and any(isinstance(target, ast.Name) and target.id == "access" for target in node.targets)
        )
        names = {node.id for node in ast.walk(access.value) if isinstance(node, ast.Name)}
        self.assertIn("DESKTOP_CREATEMENU", names)

    def test_departed_runtime_desktop_marker_requires_its_exact_reviewed_plan(self):
        release_path = REPO / ".jenkins/state/deployment/reviewed-lost-runtime.json"
        marker = {
            "plan": {"path": str(REPO / ".jenkins/state/migration-baseline/reviewed-plan.json"), "sha256": "a" * 64},
            "originalCommand": {"path": str(REPO / ".jenkins/state/recovery-lifecycle/command.json"), "sha256": "c" * 64},
            "originalDescriptor": {"path": str(REPO / ".jenkins/state/formal-lifecycle/recovery-keeper.json"), "sha256": "d" * 64},
        }
        payload = {"schemaVersion": 1, "kind": "formal-lost-owner-recovery-release",
                   "lostRecoveryRuntime": marker, "requestId": "b" * 32, "scope": "execution-runtime",
                   "expectedPreviousOwner": {"runtime": "departed"}, "expectedLifecycleFence": {"legacy": "pinned"},
                   "runtimeAcceptance": False, "cargoAcceptance": False}
        plan = {"recoveryVariant": "lost-recovery-runtime", "originalCommandStatus": "unknown", "requestId": payload["requestId"],
                "scope": payload["scope"], "owner": payload["expectedPreviousOwner"],
                "lifecycleFence": payload["expectedLifecycleFence"], **marker}
        for boundary in ("matching", "matching-committed", "extra-marker", "wrong-original-path", "wrong-plan-variant", "wrong-status", "wrong-request", "changed-plan"):
            with self.subTest(boundary=boundary):
                document, reviewed = deepcopy(payload), deepcopy(plan)
                if boundary == "matching-committed":
                    reviewed.update(recoveryVariant="departed-committed-recovery-runtime", originalCommandStatus="committed")
                elif boundary == "wrong-status": reviewed["originalCommandStatus"] = "committed"
                elif boundary == "extra-marker": document["lostRecoveryRuntime"]["unchecked"] = {}
                elif boundary == "wrong-original-path": document["lostRecoveryRuntime"]["originalCommand"]["path"] = str(REPO / "other.json")
                elif boundary == "wrong-plan-variant": reviewed["recoveryVariant"] = "failed-own-start"
                elif boundary == "wrong-request": reviewed["requestId"] = "e" * 32
                elif boundary == "changed-plan": document["lostRecoveryRuntime"]["plan"]["sha256"] = "f" * 64
                with patch.object(self.launcher, "_canonical_dir", return_value=REPO), \
                        patch.object(self.launcher, "_canonical_file", side_effect=lambda value, **_: Path(value)), \
                        patch.object(self.launcher, "_sha256", return_value="a" * 64), \
                        patch.object(self.launcher, "_read_json", side_effect=[document, reviewed]), \
                        self.assertRaises(self.launcher.LauncherError) as error:
                    self.launcher.verify_release(release_path, "a" * 64, repo=REPO)
                expected = ("release_configuration" if boundary in {"matching", "matching-committed"} else
                            "release_digest_mismatch" if boundary == "changed-plan" else "release_kind")
                self.assertEqual(error.exception.code, expected)

    def test_launcher_has_no_service_or_startup_side_effects(self):
        for forbidden in (
            "DeploymentManager",
            "subprocess",
            "CreateProcess",
            "Popen",
            "winreg",
            "set_current_user",
            "Run\\\\",
            ".jenkins_tray/launch.py",
        ):
            self.assertNotIn(forbidden, self.source)
        call_names = {
            node.func.attr
            for node in ast.walk(self.tree)
            if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)
        }
        self.assertNotIn("write_text", call_names)
        self.assertNotIn("write_bytes", call_names)
        self.assertNotIn("mkdir", call_names)
        self.assertNotIn("terminate", call_names)
        self.assertNotIn("kill", call_names)

    def test_formal_controls_remain_keeper_dispatches(self):
        formal = (REPO / "tools/jenkins/tray/formal.py").read_text(encoding="utf-8")
        self.assertIn("from ..deployment.keeper_control import request_keeper_action", formal)
        self.assertIn('request_keeper_action(config, "start")', formal)
        self.assertIn('request_keeper_action(config, "stop")', formal)
        self.assertIn('request_keeper_action(config, "restart")', formal)

    def test_import_occurs_only_after_release_and_desktop_preflight(self):
        imports = [node for node in ast.walk(self.tree) if isinstance(node, (ast.Import, ast.ImportFrom))]
        formal = [node for node in imports
                  if isinstance(node, ast.ImportFrom) and "tools.jenkins.tray.formal_app" in (node.module or "")]
        self.assertEqual(len(formal), 1)
        self.assertIsInstance(formal[0], ast.ImportFrom)
        self.assertIn("run_formal", {alias.name for alias in formal[0].names})
        self.assertIn("verify_release(args.release, args.release_sha256)", self.source)
        self.assertIn("_current_pythonw(pin)", self.source)
        self.assertIn("_verify_user_desktop()", self.source)
        self.assertIn("sys.path.insert(0, str(pin.driver_root))", self.source)
        self.assertIn("runtime.get(\"selected\")", self.source)


if __name__ == "__main__":
    unittest.main()
