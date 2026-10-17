"""Explicit client review never overrides native ownership or unknown processes."""
from __future__ import annotations
from contextlib import contextmanager, nullcontext, ExitStack
from copy import deepcopy
import hashlib
import json
from pathlib import Path
from types import SimpleNamespace
from unittest import TestCase
from unittest.mock import Mock, patch

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
        document = roles.inspect_runtime_role_review(self.manager, clients=[self.identity], reviewed_target=self.target,
            profile=getattr(self, "profile", None))
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


class PenpotRoleFixture(RoleFixture):
    def __init__(self):
        super().__init__()
        self.profile = roles._PENPOT_PROFILE
        self.script = Path("E:/cargo-targets/zircon-local/tmp/penpot-role-fixture/supervisor.py")
        self.arguments[:] = [self.identity["executable"], "-B", "-X", "utf8", self.script.as_posix()]
        self.identity["commandDigest"] = roles._digest_args(tuple(self.arguments))
        self.inventory[:] = [deepcopy(self.identity)]
        self.penpot_sources = {str(self.script): self.put(self.script, b"reviewed supervisor"),
            str(Path(self.identity["executable"])): self.put(Path(self.identity["executable"]), b"reviewed Python"),
            str(self.output / "read.json"): self.put(self.output / "read.json", b"reviewed input")}

    @contextmanager
    def observations(self):
        with super().observations(), patch.object(roles, "_PENPOT_SCRIPT", self.script), \
                patch.object(roles, "_PENPOT_CLIENT", deepcopy(self.identity)), \
                patch.object(roles, "_PENPOT_SOURCES", self.penpot_sources), \
                patch.object(roles, "_PENPOT_OUTPUTS", [str(self.output)]):
            yield self


class PenpotRuntimeRolesTests(TestCase):
    def test_explicit_profile_is_independent_of_old_build_and_preserves_service_priority(self):
        fixture = PenpotRoleFixture()
        unknown = {**fixture.identity, "pid": 902}
        fixture.inventory.append(unknown)
        with fixture.observations():
            fixture.files[str(ROOT / "tools/build/zircon_build.py")] += b" foreign edit"
            fixture.put(ROOT / "tools/__init__.py", b"unrelated build namespace")
            cap = fixture.issue()
            view = cap.observe(fixture.manager)
            self.assertEqual([unknown], view["blockingOwners"])
            self.assertEqual(fixture.inventory, view["runtimeInventory"])
            self.assertEqual("penpot-existing-runtime-supervisor", view["reviewedForeign"][0]["role"])
            self.assertEqual(fixture.penpot_sources, view["reviewedForeign"][0]["sourceFiles"])
            self.assertEqual(fixture.profile, cap.document["profile"])
            roles.verify_driver.assert_called()
            fixture.put(fixture.manager.paths.state / "deployment/host.json",
                        canonical_json({"host": fixture.identity}))
            self.assertIn(fixture.identity, cap.observe(fixture.manager)["blockingOwners"])
            with self.assertRaises(JenkinsError):
                fixture.issue()

    def test_fixed_closure_target_profile_and_output_identity_are_revalidated(self):
        for changed in ("script", "python", "input", "target", "profile", "output"):
            with self.subTest(changed=changed):
                fixture = PenpotRoleFixture()
                with fixture.observations():
                    cap = fixture.issue()
                    if changed in {"script", "python", "input"}:
                        name = list(fixture.penpot_sources)[{"script": 0, "python": 1, "input": 2}[changed]]
                        fixture.files[name] += b" changed"
                    elif changed == "target":
                        roles.verify_driver.return_value = {**fixture.target, "driverDigest": "e" * 64}
                    elif changed == "profile":
                        value = cap.document
                        value["profile"] = "unreviewed-profile"
                        fixture.put(Path(fixture.review_ref["path"]), canonical_json(value))
                        with self.assertRaises(JenkinsError):
                            roles.prepare_runtime_role_review(fixture.manager,
                                review_reference=fixture.ref(Path(fixture.review_ref["path"])))
                        continue
                    else:
                        fixture.directory_id += 1
                    with self.assertRaises(JenkinsError):
                        cap.observe(fixture.manager)

    def test_profile_cannot_adopt_replacement_or_runtime_command_or_implicit_script(self):
        fixture = PenpotRoleFixture()
        with fixture.observations():
            cap = fixture.issue()
            fixture.profile = None
            with self.assertRaises(JenkinsError):
                fixture.issue()
            fixture.profile = roles._PENPOT_PROFILE
            fixture.argv[fixture.identity["pid"]] = [*fixture.arguments, "__lifecycle-host"]
            self.assertEqual([fixture.identity], cap.observe(fixture.manager)["blockingOwners"])
            with self.assertRaises(JenkinsError):
                fixture.issue()
            fixture.argv[fixture.identity["pid"]] = fixture.arguments
            replacement = {**fixture.identity, "creationTime": str(int(fixture.identity["creationTime"]) + 1)}
            fixture.inventory[:] = [replacement]
            view = cap.observe(fixture.manager)
            self.assertEqual([replacement], view["blockingOwners"])
            self.assertEqual("departed", view["reviewedForeign"][0]["status"])



class CurrentServiceBoundaryTests(TestCase):
    def fixture(self, body):
        fixture = RoleFixture()
        fixture.arguments[:] = [fixture.identity["executable"], "-B", *body]
        fixture.identity["commandDigest"] = roles._digest_args(tuple(fixture.arguments))
        fixture.inventory[:] = [deepcopy(fixture.identity)]
        return fixture

    @contextmanager
    def live(self, fixture):
        with fixture.observations(), patch.object(roles, "observe_departed_identity",
                side_effect=JenkinsError("deployment_owner_live", "still live")):
            yield fixture

    def test_ordinary_script_stdin_and_inline_helpers_coexist_without_authority(self):
        for body in ([str(Path("E:/cargo-targets/foreign/validate.py"))], ["-"], ["-c", "print('read-only')"]):
            with self.subTest(body=body):
                fixture = self.fixture(body)
                with self.live(fixture):
                    view = roles.observe_runtime_process_boundary(fixture.manager)
                self.assertEqual(fixture.inventory, view["runtimeInventory"])
                self.assertEqual([], view["blockingOwners"])
                self.assertEqual(1, len(view["nonServiceProcesses"]))
                self.assertEqual([], view["requestClients"])
                self.assertTrue(view["observationOnly"])
                self.assertFalse(view["terminalProof"])
                for name in roles._FALSE:
                    self.assertFalse(view[name])

    def test_exact_canonical_request_hash_is_distinct_from_ordinary_helpers(self):
        name = "tools/dev/jenkins_validate.py"
        fixture = self.fixture([str(ROOT / name), "status"])
        checksum = fixture.put(ROOT / name, b"reviewed request")
        with self.live(fixture), patch.object(roles, "_REQUEST_CLIENT_SOURCES", {name: checksum}):
            view = roles.observe_runtime_process_boundary(fixture.manager)
            self.assertEqual([], view["blockingOwners"])
            self.assertEqual("canonical-jenkins-request-client", view["requestClients"][0]["role"])
            fixture.files[str(ROOT / name)] += b" changed"
            view = roles.observe_runtime_process_boundary(fixture.manager)
            self.assertEqual([], view["blockingOwners"])
            self.assertEqual([], view["requestClients"])
            self.assertEqual("non-service-unverified-client", view["nonServiceProcesses"][0]["role"])
            self.assertEqual("request-source-verification", view["nonServiceProcesses"][0]["sourceVerification"]["stage"])

    def test_request_import_namespace_cannot_silently_change(self):
        name = "tools/dev/jenkins_validate.py"
        fixture = self.fixture([str(ROOT / name), "status"])
        checksum = fixture.put(ROOT / name, b"reviewed request")
        with self.live(fixture), patch.object(roles, "_REQUEST_CLIENT_SOURCES", {name: checksum}):
            fixture.put(ROOT / "tools/__init__.py", b"new import owner")
            view = roles.observe_runtime_process_boundary(fixture.manager)
            self.assertEqual([], view["blockingOwners"])
            self.assertEqual([], view["requestClients"])
            self.assertEqual("non-service-unverified-client", view["nonServiceProcesses"][0]["role"])
            self.assertEqual("request-source-verification", view["nonServiceProcesses"][0]["sourceVerification"]["stage"])

    def test_installed_management_entry_requires_exact_capsule_and_client_bytes(self):
        name = "tools/jenkins-management/entry.py"
        fixture = self.fixture([str(ROOT / name), "submit", "--repo-root", str(ROOT)])
        names = {name, "tools/dev/local_cargo.py", "tools/dev/local-cargo.ps1",
            "tools/dev/dev-fast-build.ps1", "tools/dev/submit-jenkins-management.ps1", "tools/jenkins/frontend.py"}
        clients = {key: fixture.put(ROOT / key, ("reviewed " + key).encode()) for key in names}
        files = {key: hashlib.sha256(key.encode()).hexdigest()
                 for key in ("runner.py", "source_preparation.py", "zircon-management.groovy",
                             "checkout_guard.py", "checkout_product.py")}
        config = {"repositoryRoot": str(ROOT), "driverDigest": "f" * 64,
            "runtimeOperationId": "runtime", "clientSources": clients, "files": files}
        raw = canonical_json(config)
        checksum = hashlib.sha256(raw).hexdigest()
        capsule = ROOT / ".jenkins/runtime/management" / checksum
        fixture.put(capsule / "capsule.json", raw)
        for key in files:
            fixture.put(capsule / key, key.encode())
        fixture.put(fixture.manager.paths.state / "management/installed.json", canonical_json({
            "schemaVersion": 1, "job": "zircon-management", "capsule": str(capsule),
            "capsuleSha256": checksum, "driverDigest": "f" * 64, "runtimeOperationId": "runtime"}))
        with self.live(fixture), patch.object(roles, "_REQUEST_CLIENT_SOURCES", {name: clients[name]}):
            view = roles.observe_runtime_process_boundary(fixture.manager)
            self.assertEqual("installed-management-request-client", view["requestClients"][0]["role"])
            fixture.files[str(capsule / "runner.py")] += b" changed"
            view = roles.observe_runtime_process_boundary(fixture.manager)
            self.assertEqual([], view["blockingOwners"])
            self.assertEqual([], view["requestClients"])
            self.assertEqual("non-service-unverified-client", view["nonServiceProcesses"][0]["role"])
            self.assertEqual("request-source-verification", view["nonServiceProcesses"][0]["sourceVerification"]["stage"])

    def test_sealed_driver_requests_coexist_but_internal_or_deployment_hosts_block(self):
        fixture = self.fixture([str(ROOT / ".jenkins/runtime/drivers" / ("f" * 64) / "launcher.py"),
            "--repository", str(ROOT), "request", "observe", "--input-file", str(ROOT / ".jenkins/state/request.json")])
        with self.live(fixture):
            view = roles.observe_runtime_process_boundary(fixture.manager)
            self.assertEqual("sealed-driver-request-client", view["requestClients"][0]["role"])
            roles.verify_driver.assert_called()
            for body in ([fixture.arguments[2], "__execution-host"],
                         [str(ROOT / "tools/dev/jenkins_recovery_keeper.py")],
                         ["-m", "tools.jenkins", "deployment", "start"]):
                fixture.arguments[:] = [fixture.identity["executable"], "-B", *body]
                fixture.identity["commandDigest"] = roles._digest_args(tuple(fixture.arguments))
                fixture.inventory[:] = [deepcopy(fixture.identity)]
                self.assertEqual(fixture.inventory, roles.observe_runtime_process_boundary(fixture.manager)["blockingOwners"])

    def test_json_and_jvm_service_owners_have_priority_over_client_argv(self):
        fixture = self.fixture(["-c", "print('ordinary')"])
        path = fixture.manager.paths.state / "deployment/host.json"
        with self.live(fixture):
            fixture.put(path, canonical_json({"host": fixture.identity}))
            self.assertEqual(fixture.inventory, roles.observe_runtime_process_boundary(fixture.manager)["blockingOwners"])
            fixture.put(path, canonical_json({"host": {**fixture.identity,
                "creationTime": str(int(fixture.identity["creationTime"]) - 1)}}))
            self.assertEqual([], roles.observe_runtime_process_boundary(fixture.manager)["blockingOwners"])
            java = {**fixture.identity, "pid": 991, "executable": str(ROOT / ".jenkins/runtime/jdk/bin/java.exe")}
            fixture.inventory.append(java)
            self.assertEqual([java], roles.observe_runtime_process_boundary(fixture.manager)["blockingOwners"])

    def test_sqlite_native_birth_binding_blocks_and_preserves_unknown_row(self):
        import sqlite3
        fixture = self.fixture(["-"])
        payload = json.dumps({"status": "unknown", "pid": fixture.identity["pid"],
                             "birthToken": fixture.identity["creationTime"]})
        row = ("native_job", "historic-unknown", 7, payload)
        fixture.put(fixture.manager.paths.state / "coordination.sqlite3", b"native DB exists")
        connection = Mock()
        connection.execute.side_effect = lambda sql: [row] if sql.startswith("SELECT") else []
        with self.live(fixture), patch.object(sqlite3, "connect", return_value=connection) as connect:
            view = roles.observe_runtime_process_boundary(fixture.manager)
            self.assertEqual(fixture.inventory, view["blockingOwners"])
            self.assertEqual(hashlib.sha256(payload.encode()).hexdigest(), view["nativeBindings"][0]["payloadSha256"])
            self.assertTrue(connect.call_args.kwargs["uri"])
            self.assertIn("?mode=ro", connect.call_args.args[0])
            self.assertEqual("unknown", json.loads(row[3])["status"])
            self.assertFalse(any(sql.args[0].startswith(("UPDATE", "INSERT", "DELETE")) for sql in connection.execute.call_args_list))

    def test_unreadable_live_identity_blocks_but_exact_departure_is_observation_only(self):
        fixture = self.fixture(["-"])
        with self.live(fixture):
            roles._windows_command_line.side_effect = OSError("access denied")
            self.assertEqual(fixture.inventory, roles.observe_runtime_process_boundary(fixture.manager)["blockingOwners"])
            for status in ("absent", "pid-reused"):
                roles.observe_departed_identity.side_effect = None
                roles.observe_departed_identity.return_value = {"pid": fixture.identity["pid"],
                    "expectedCreationTime": fixture.identity["creationTime"], "status": status}
                view = roles.observe_runtime_process_boundary(fixture.manager)
                self.assertEqual([], view["blockingOwners"])
                self.assertEqual(status, view["departedDuringObservation"][0]["departure"]["status"])
                self.assertTrue(view["departedDuringObservation"][0]["observationOnly"])
                self.assertFalse(view["nativeProofIssued"])

    def test_identity_race_does_not_reclassify_a_live_replacement(self):
        fixture = self.fixture(["-"])
        with self.live(fixture):
            roles.observe_live_identity.side_effect = JenkinsError("identity_changed", "birth changed")
            self.assertEqual(fixture.inventory, roles.observe_runtime_process_boundary(fixture.manager)["blockingOwners"])

    def test_request_population_is_dynamic_but_owned_upgrade_snapshot_is_stable(self):
        from tools.jenkins.deployment import owned_upgrade as upgrade
        from tools.jenkins.tests.test_external_native_stop_upgrade import ExternalStoppedOwner
        scenario = ExternalStoppedOwner()
        fixture = RoleFixture(scenario.manager, scenario.files, scenario.targets["b" * 64])
        fixture.arguments[:] = [fixture.identity["executable"], "-B", "-c", "print('ordinary')"]
        fixture.identity["commandDigest"] = roles._digest_args(tuple(fixture.arguments))
        fixture.inventory[:] = [deepcopy(fixture.identity)]
        with scenario.observations(), self.live(fixture):
            scenario.inventory.return_value = fixture.inventory
            plan = scenario.prepare()
            boundary = roles.observe_runtime_process_boundary(scenario.manager, inventory=fixture.inventory)["stableBoundary"]
            fixture.inventory.clear()
            plan.revalidate(scenario.manager)
            self.assertEqual(boundary,
                roles.observe_runtime_process_boundary(scenario.manager, inventory=fixture.inventory)["stableBoundary"])
            scenario.home_probe.assert_called()
            scenario.endpoint.assert_called()
            java = {**fixture.identity, "pid": 910, "executable": str(ROOT / ".jenkins/runtime/jdk/bin/java.exe")}
            fixture.inventory.append(java)
            with self.assertRaises(JenkinsError):
                plan.revalidate(scenario.manager)
            self.assertEqual(scenario.holds, upgrade.protected_holds(scenario.state))


    def test_service_path_strings_in_inline_code_and_request_data_do_not_claim_ownership(self):
        cases = (["-c", "print('.jenkins/state/deployment/driver.json')"],
                 [str(Path("E:/cargo-targets/foreign/inspect.py")), "--input",
                  '{"path":"E:/Git/ZirconEngine/.jenkins/state/recovery-lifecycle/command.json"}'])
        for body in cases:
            with self.subTest(body=body):
                fixture = self.fixture(body)
                with self.live(fixture):
                    view = roles.observe_runtime_process_boundary(fixture.manager)
                self.assertEqual([], view["blockingOwners"])
                self.assertEqual(1, len(view["nonServiceProcesses"]))
                self.assertFalse(view["terminalProof"])

    def test_service_entry_is_recognized_after_other_python_switches(self):
        fixture = self.fixture([str(ROOT / "tools/dev/jenkins_recovery_keeper.py")])
        fixture.arguments.insert(2, "-u")
        fixture.identity["commandDigest"] = roles._digest_args(tuple(fixture.arguments))
        fixture.inventory[:] = [deepcopy(fixture.identity)]
        with self.live(fixture):
            self.assertEqual(fixture.inventory, roles.observe_runtime_process_boundary(fixture.manager)["blockingOwners"])
