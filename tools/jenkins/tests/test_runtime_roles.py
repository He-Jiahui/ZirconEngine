"""Explicit client review never overrides native ownership or unknown processes."""
from __future__ import annotations
from contextlib import contextmanager, nullcontext, ExitStack
from copy import deepcopy
import hashlib
import json
from pathlib import Path
from types import SimpleNamespace
from unittest import TestCase
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, canonical_json
from tools.jenkins.deployment import runtime_roles as roles
from tools.jenkins.deployment import prelaunch_upgrade_recovery as recovery
from tools.jenkins.deployment import pre_spawn_recovery_cancel as cancel
from tools.jenkins.tests.test_prelaunch_upgrade_recovery import PrelaunchFixture

ROOT = Path(__file__).absolute().parents[3]


class RoleFixture:
    def __init__(self, manager=None, files=None, target=None):
        self.manager = manager or SimpleNamespace(paths=SimpleNamespace(repo=ROOT, state=ROOT / ".jenkins/state"))
        self.files = files if files is not None else {}
        self.target = target or {"driverDigest": "f" * 64, "root": str(ROOT / ".jenkins/runtime/drivers" / ("f" * 64)),
            "manifest": str(ROOT / ".jenkins/runtime/drivers" / ("f" * 64) / "driver-manifest.json")}
        self.boot = "134358454705000000"
        self.catalog = {"tools/build/zircon_build.py": self.put(ROOT / "tools/build/zircon_build.py", b"reviewed build"),
                        "tools/dev/local_cargo.py": self.put(ROOT / "tools/dev/local_cargo.py", b"reviewed Cargo"),
                        "tools/__init__.py": None, "tools/build/__init__.py": None, "tools/dev/__init__.py": None}
        manifest = {}
        for name in ("tools/jenkins/frontend.py", "tools/jenkins/submission.py"):
            if str(ROOT / name) not in self.files:
                self.put(ROOT / name, b"client submission")
            manifest[name] = {"sha256": self.ref(ROOT / name)["sha256"]}
        self.put(Path(self.target["manifest"]), canonical_json({"files": manifest}))
        self.shim_sha = self.put(roles._SHIM, b"reviewed exact shim")
        self.cmd_sha = self.put(roles._CARGO_CMD, b"reviewed exact cmd")
        self.output = roles._SHIM_TARGET.parent
        self.arguments = [str(ROOT / ".jenkins/runtime/python/python.exe"), "-B", "-X", "utf8",
            str(ROOT / "tools/build/zircon_build.py"), "--targets", "editor", "--mode", "debug",
            "--runtime-features", "target-editor-host,profiling", "--out", str(self.output), "--jobs", "1",
            "--cargo", str(roles._CARGO_CMD)]
        self.identity = {"pid": 12912, "creationTime": str(int(self.boot) + 900),
            "executable": self.arguments[0], "commandDigest": roles._digest_args(tuple(self.arguments))}
        self.inventory = [deepcopy(self.identity)]
        self.argv = {self.identity["pid"]: self.arguments}
        self.directory_id = 900
        self.review_ref = None

    def put(self, path, raw):
        self.files[str(path)] = raw
        return hashlib.sha256(raw).hexdigest()

    def ref(self, path):
        return {"path": str(path), "sha256": hashlib.sha256(self.files[str(path)]).hexdigest()}

    @contextmanager
    def observations(self):
        with ExitStack() as stack:
            stack.enter_context(patch.object(roles, "_CLIENT_SOURCES", self.catalog))
            stack.enter_context(patch.object(roles, "_SHIM_SHA", self.shim_sha))
            stack.enter_context(patch.object(roles, "_CARGO_CMD_SHA", self.cmd_sha))
            stack.enter_context(patch.object(roles, "_plain"))
            stack.enter_context(patch.object(roles, "_raw", side_effect=lambda path: self.files[str(path)]))
            stack.enter_context(patch.object(roles, "verify_driver", return_value=self.target))
            stack.enter_context(patch.object(roles, "windows_boot_filetime", return_value=int(self.boot)))
            stack.enter_context(patch.object(roles, "runtime_owner_inventory", side_effect=lambda root: deepcopy(self.inventory)))
            stack.enter_context(patch.object(roles, "observe_live_identity", side_effect=lambda identity: identity))
            stack.enter_context(patch.object(roles, "_windows_command_line", side_effect=lambda pid: tuple(self.argv[pid])))
            stack.enter_context(patch.object(roles, "observe_departed_identity",
                side_effect=lambda identity: {"pid": identity["pid"], "expectedCreationTime": identity["creationTime"], "status": "absent"}))
            stack.enter_context(patch.object(roles, "build_root_containing",
                side_effect=lambda path: SimpleNamespace(path=Path(Path(path).anchor) / "cargo-targets", volume_identity="native-volume")))
            self.physical = stack.enter_context(patch.object(roles, "physical_path_under", side_effect=lambda root, path, **kw: Path(path)))
            # If an inherited fixture already patches exists, do not autospec its Mock.
            stack.enter_context(patch.object(Path, "exists", lambda path: str(path) in self.files))
            stack.enter_context(patch.object(Path, "is_dir", lambda path: True))
            stack.enter_context(patch.object(Path, "stat", lambda path, **kw: SimpleNamespace(st_dev=123, st_ino=self.directory_id)))
            yield self

    def issue(self):
        document = roles.inspect_runtime_role_review(self.manager, clients=[self.identity], reviewed_target=self.target)
        path = self.manager.paths.state / "recovery-lifecycle/history/review.json"
        self.put(path, canonical_json(document) + b"\n")
        self.review_ref = self.ref(path)
        return roles.prepare_runtime_role_review(self.manager, review_reference=self.review_ref)


class RuntimeRolesTests(TestCase):
    def test_exact_client_retains_raw_inventory_and_lifecycle_priority(self):
        fixture = RoleFixture()
        unknown = {**fixture.identity, "pid": 777, "creationTime": str(int(fixture.boot) + 888)}
        java = {**unknown, "pid": 778, "executable": str(ROOT / ".jenkins/runtime/jdk/bin/java.exe")}
        fixture.inventory += [unknown, java]
        with fixture.observations():
            cap = fixture.issue()
            view = cap.observe(fixture.manager)
            self.assertEqual(fixture.inventory, view["runtimeInventory"])
            self.assertEqual([unknown, java], view["blockingOwners"])
            self.assertEqual("live", view["reviewedForeign"][0]["status"])
            self.assertFalse(view["nativeProofIssued"])
            fixture.put(fixture.manager.paths.state / "deployment/host.json",
                        canonical_json({"controllerIdentity": fixture.identity}))
            self.assertIn(fixture.identity, cap.observe(fixture.manager)["blockingOwners"])
            with self.assertRaises(JenkinsError):
                fixture.issue()

    def test_departure_does_not_adopt_same_pid_replacement(self):
        fixture = RoleFixture()
        with fixture.observations():
            cap = fixture.issue()
            fixture.inventory.clear()
            view = cap.observe(fixture.manager)
            self.assertEqual([], view["blockingOwners"])
            self.assertEqual("departed", view["reviewedForeign"][0]["status"])
            replacement = {**fixture.identity, "creationTime": str(int(fixture.boot) + 1000)}
            fixture.inventory.append(replacement)
            self.assertEqual([replacement], cap.observe(fixture.manager)["blockingOwners"])
            self.assertEqual("departed", cap.observe(fixture.manager)["reviewedForeign"][0]["status"])

    def test_changed_source_namespace_output_identity_or_argv_is_rejected(self):
        for changed in ("source", "namespace", "output", "argv", "ref", "boot", "alias", "observability"):
            with self.subTest(changed=changed):
                fixture = RoleFixture()
                with fixture.observations():
                    cap = fixture.issue()
                    if changed == "source":
                        fixture.files[str(ROOT / "tools/dev/local_cargo.py")] += b" "
                    elif changed == "namespace":
                        fixture.put(ROOT / "tools/__init__.py", b"injected package")
                    elif changed == "output":
                        fixture.directory_id += 1
                    elif changed == "argv":
                        fixture.argv[fixture.identity["pid"]] = [*fixture.arguments, "__lifecycle-host"]
                    elif changed == "ref":
                        fixture.files[fixture.review_ref["path"]] += b" "
                    elif changed == "boot":
                        roles.windows_boot_filetime.return_value += 1
                    elif changed == "alias":
                        fixture.physical.side_effect = JenkinsError("build_root_alias", "alias")
                    else:
                        roles.observe_live_identity.side_effect = JenkinsError("inconclusive", "unreadable")
                    if changed == "argv":
                        view = cap.observe(fixture.manager)
                        self.assertEqual([fixture.identity], view["blockingOwners"])
                        self.assertEqual([], view["reviewedForeign"])
                    else:
                        with self.assertRaises(JenkinsError):
                            cap.observe(fixture.manager)

    def test_unreviewed_script_c_module_output_or_shim_cannot_issue(self):
        for changed in ("c", "copied-script", "module", "home-output", "wrong-command", "extra-option", "wrong-exe"):
            with self.subTest(changed=changed):
                fixture = RoleFixture()
                if changed == "c":
                    fixture.arguments[4:] = ["-c", "from tools.dev.local_cargo import main; main()"]
                elif changed == "copied-script":
                    fixture.arguments[4] = str(fixture.output / "zircon_build.py")
                elif changed == "module":
                    fixture.arguments[4:] = ["-m", "tools.dev.artifact_inventory", "--report", str(fixture.output / "report.json")]
                elif changed == "home-output":
                    fixture.arguments[fixture.arguments.index("--out") + 1] = str(ROOT / ".jenkins/jenkins_home")
                elif changed == "wrong-command":
                    fixture.arguments[fixture.arguments.index("--cargo") + 1] = str(fixture.output / "cargo.cmd")
                elif changed == "extra-option":
                    fixture.arguments += ["--unsafe", "true"]
                else:
                    fixture.identity["executable"] = str(ROOT / ".jenkins/runtime/python/pythonw.exe")
                fixture.identity["commandDigest"] = roles._digest_args(tuple(fixture.arguments))
                with fixture.observations(), self.assertRaises(JenkinsError):
                    fixture.issue()

    def test_exact_cargo_shim_is_reviewed_but_raw_overrides_are_not(self):
        fixture = RoleFixture()
        fixture.arguments[:] = [fixture.identity["executable"], "-B", "-X", "utf8", str(roles._SHIM),
            "rustc", "-p", "zircon_runtime", "--lib", "--crate-type", "cdylib", "--no-default-features",
            "--features", "target-editor-host profiling", "--target-dir", str(roles._SHIM_TARGET / "runtime/lib"), "--locked", "--jobs", "1"]
        fixture.identity["commandDigest"] = roles._digest_args(tuple(fixture.arguments))
        with fixture.observations():
            cap = fixture.issue()
            self.assertEqual("zircon-local-cargo-shim", cap.observe(fixture.manager)["reviewedForeign"][0]["role"])
            fixture.arguments += ["--config=target-dir=C:/bad"]
            fixture.identity["commandDigest"] = roles._digest_args(tuple(fixture.arguments))
            with self.assertRaises(JenkinsError):
                fixture.issue()

    def test_capability_type_and_missing_native_identity_are_not_overridable(self):
        fixture = RoleFixture()
        with fixture.observations():
            cap = fixture.issue()
            with self.assertRaises(JenkinsError):
                roles.ReviewedRuntimeRoles(b"{}", b"{}")
            class Forged(roles.ReviewedRuntimeRoles):
                pass
            with self.assertRaises(JenkinsError):
                object.__new__(Forged).observe(fixture.manager)
            fixture.inventory.clear()
            with self.assertRaises(JenkinsError):
                fixture.issue()
            self.assertEqual("departed", cap.observe(fixture.manager)["reviewedForeign"][0]["status"])

    def test_retained_attempt_exposes_full_reviewed_inventory_without_filtering_native_scope(self):
        fixture = RoleFixture()
        keeper = {**fixture.identity, "pid": 5376, "creationTime": str(int(fixture.boot) + 600)}
        fixture.inventory.append(keeper)
        with fixture.observations():
            cap = fixture.issue()
            probe = cancel._RetainedAttempt.__new__(cancel._RetainedAttempt)
            probe.manager, probe.runtime_review = fixture.manager, cap
            probe.handles, probe.image_pins = {}, {}
            probe.identities = {"keeper": keeper, "console": {**keeper, "pid": 2}, "officialHost": {**keeper, "pid": 3},
                "runner": {**keeper, "pid": 4}, "hostConsole": {**keeper, "pid": 5}, "dispatcher": {**keeper, "pid": 6}}
            probe.holder = {"restrictionPreflight": {"associatedJob": {"memberPids": [5376, 2]}}}
            def in_job(handle, job, member):
                member._obj.value = True
                return True
            probe.api = SimpleNamespace(IsProcessInJob=in_job)
            probe.handles = {"keeper": 1, "console": 2}
            with patch.object(probe, "_verify"), patch.object(cancel, "_descendants", return_value=[2, 3, 4, 5, 5376]):
                scope = probe.verify_scope()
                self.assertEqual(fixture.inventory, scope["runtimeBoundary"]["runtimeInventory"])
                self.assertEqual([keeper], scope["runtimeBoundary"]["blockingOwners"])
                fixture.inventory.append({**keeper, "pid": 9})
                with self.assertRaises(JenkinsError):
                    probe.verify_scope()

    def test_review_bound_restore_and_actual_owned_capture_revalidate_foreign_clients(self):
        fixture = PrelaunchFixture()
        with fixture.observations():
            fixture.setup_failure()
        role_fixture = RoleFixture(fixture.manager, fixture.files, fixture.new_target)
        with fixture.observations(), role_fixture.observations():
            cap = role_fixture.issue()
            fixture.capture_retirement()
            # Preserve the complete original native inventory, with keeper as
            # sole native owner and this one positively reviewed build client.
            role_fixture.inventory.append(fixture.keeper)
            boundary = cap.observe(fixture.manager)
            fixture.retired["runtimeRoleReview"] = role_fixture.review_ref
            fixture.retired["scope"]["runtimeBoundary"] = boundary
            fixture.retired["runtimeOwnerInventoryBefore"] = boundary["runtimeInventory"]
            fixture.retired_ref = fixture.put(Path(fixture.retired_ref["path"]), fixture.retired)
            role_fixture.inventory.remove(fixture.keeper)
            with fixture.digest_patch:
                capability = recovery.prepare_prelaunch_upgrade_recovery(fixture.manager,
                    expected_command_sha256=fixture.command_ref["sha256"], holder_reference=fixture.holder_ref,
                    replacement_request_id=fixture.replacement_id, replacement_target=fixture.new_target,
                    retirement_reference=fixture.retired_ref, runtime_role_review_ref=role_fixture.review_ref)
                before = deepcopy(fixture.files)
                self.assertEqual(role_fixture.review_ref, capability.document["runtimeRoleReview"])
                result = capability.restore(fixture.manager)
                self.assertEqual(role_fixture.review_ref, result["runtimeRoleReview"])
            fixture.request_id = fixture.replacement_id
            fixture.successor.update(requestId=fixture.replacement_id, prelaunchReconciliation=result["reconciliation"],
                                     runtimeRoleReview=role_fixture.review_ref)
            for name, checksum in fixture.successor["operatorFiles"].items():
                path = Path(name)
                if path.is_relative_to(fixture.repo / "tools/jenkins"):
                    fixture.regression_doc["sourceFiles"][path.relative_to(fixture.repo).as_posix()] = checksum
                if name in fixture.files and hashlib.sha256(fixture.files[name]).hexdigest() == checksum:
                    continue
                sealed = Path(fixture.new_target["root"]) / path.relative_to(fixture.repo)
                if str(sealed) in fixture.files:
                    fixture.files[name] = fixture.files[str(sealed)]
            fixture.successor_ref = fixture.put(fixture.successor_path, fixture.successor)
            plan = fixture.prepare()
            self.assertEqual(role_fixture.review_ref, plan.document["runtimeRoleReview"])
            self.assertEqual([role_fixture.identity], plan.document["runtimeOwnerInventory"]["originalRuntimeInventory"])
            plan.revalidate(fixture.manager)
            role_fixture.inventory.clear()
            plan.revalidate(fixture.manager)
            role_fixture.inventory.append({**role_fixture.identity, "pid": 12300})
            with self.assertRaises(JenkinsError):
                plan.revalidate(fixture.manager)
            for name, raw in before.items():
                path = Path(name)
                if path.is_relative_to(fixture.state) and path not in {
                    fixture.paths["terminalCommand"], fixture.successor_path}:
                    self.assertEqual(raw, fixture.files[name])
