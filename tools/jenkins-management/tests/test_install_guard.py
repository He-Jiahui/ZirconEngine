"""Pinned active-build metadata updates using physical, synthetic evidence only."""
from __future__ import annotations

import copy
from contextlib import closing, contextmanager
import gc
import hashlib
import importlib.util
import json
from pathlib import Path
import sqlite3
import tempfile
import unittest
import xml.etree.ElementTree as ET


ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location(
    "management_install_guard_tests", ROOT / "tools/jenkins-management/install_guard.py")
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False,
                      allow_nan=False).encode("utf-8")


def checksum(raw):
    return hashlib.sha256(raw).hexdigest()


class ActiveUpdateFixture:
    CLIENTS = (
        "tools/dev/local_cargo.py", "tools/dev/local-cargo.ps1",
        "tools/dev/dev-fast-build.ps1", "tools/dev/submit-jenkins-management.ps1",
        "tools/jenkins/frontend.py", "tools/jenkins-management/entry.py",
    )
    RUNS = "job/zircon-management/api/json?tree=builds[number,queueId,building,result]"
    QUEUE = "queue/api/json?tree=items[task[name]]"
    BUILD = ("job/zircon-management/60/api/json?"
             "tree=number,queueId,building,result,actions[parameters[name,value]]")

    def __init__(self, directory, phase, adoption=False, prepared_flow=False):
        self.repo = Path(directory)
        self.driver = "d" * 64
        self.runtime = "start-fixture"
        self.python = self.repo / ".jenkins/runtime/python/python.exe"
        self.codex = self.repo / "host's tool/codex.exe"
        self.write(self.python, b"synthetic Python identity, never executed\n")
        self.write(self.codex, b"synthetic Codex identity, never executed\n")
        clients = {}
        for name in self.CLIENTS:
            clients[name] = checksum(self.write(self.repo / name, ("synthetic client " + name + "\n").encode()))
        tokens = ("AGENT", "REPOSITORY", "PYTHON", "CODEX", "CAPSULE", "DRIVER", "RUNTIME")
        template = ("\ufeff" + "\r\n".join("def " + name.lower() + " = '@@" + name + "@@'"
                                             for name in tokens) + "\r\n").encode("utf-8")
        old_runner = (
            "class Management:\n"
            "    def validate(self, payload):\n"
            "        return {'unchanged': payload}\n"
            "    def worktree_resume_release(self, reference):\n"
            "        return ('original', reference)\n"
            "    def worktree_resume_lineage(self, previous):\n"
            "        return {'runtime': previous, 'revision': 1}\n"
        ).encode()
        new_runner = old_runner.replace(b"('original', reference)", b"('historical', reference)").replace(
            b"'revision': 1", b"'revision': 2")
        before_helper = after_helper = b"# synthetic preparation fixture\n"
        if adoption or prepared_flow:
            # Historical fixtures retain their authenticated release when canonical advances.
            historical = ROOT / ".jenkins/runtime/management/93e0e51ed1df87d12b3697931ca53282aa1a426cee23463abc32a67915b9bc46"
            history_raw = (historical / "capsule.json").read_bytes()
            assert checksum(history_raw) == "93e0e51ed1df87d12b3697931ca53282aa1a426cee23463abc32a67915b9bc46"
            history = json.loads(history_raw)
            assert history["repositoryRoot"] == str(ROOT)
            assert history["files"]["source_preparation.py"] == guard.PREPARED_FLOW_HELPER_AFTER
            assert history["files"]["runner.py"] == guard.PREPARED_FLOW_RUNNER_AFTER
            actual_helper = (historical / "source_preparation.py").read_bytes()
            assert checksum(actual_helper) == guard.PREPARED_FLOW_HELPER_AFTER
            actual_runner = (historical / "runner.py").read_bytes()
            assert checksum(actual_runner) == guard.PREPARED_FLOW_RUNNER_AFTER
            after_helper = actual_helper
            if checksum(after_helper) == guard.PREPARED_FLOW_HELPER_AFTER:
                after_helper = after_helper[:-guard.PREPARED_FLOW_ADDITION_SIZE]
            assert checksum(after_helper) == "8ab962e8c7e39c53e7cdd9f145e719809eb802be1d4777a12a18e741afee70be"
            start = after_helper.index(b"    adoption_rows = None\r\n")
            inventory = b"    actual = _pool_inventory(destination, old if previous else None)\r\n"
            finish = after_helper.index(
                b'    if previous and previous["sourceDigest"] == sealed["sealedInputRef"]:\r\n', start)
            before_helper = after_helper[:start] + inventory + after_helper[finish:]
            assert checksum(before_helper) == "bf6130945bc342a820e12f1597ca2c2b031b30ffa45585f9a55213dddaccc2b4"
            addon = (ROOT / "tools/jenkins-management/checkout-commit.groovy").read_bytes()
            assert checksum(addon) == "59ae711879085d0f7a36f90216def9eac04538fc46d75eb1a3b97b61441277bc"
            self.write(self.repo / "tools/jenkins-management/checkout-commit.groovy", addon)
        if prepared_flow:
            assert checksum(actual_helper) == guard.PREPARED_FLOW_HELPER_AFTER
            assert checksum(actual_runner) == guard.PREPARED_FLOW_RUNNER_AFTER
            before_runner = actual_runner
            for old, new in reversed(guard.PREPARED_FLOW_RUNNER_HUNKS):
                assert before_runner.count(new) == 1
                before_runner = before_runner.replace(new, old, 1)
            assert checksum(before_runner) == guard.PREPARED_FLOW_RUNNER_BEFORE
            old_runner = new_runner = before_runner
        self.old_files = {
            "runner.py": old_runner,
            "source_preparation.py": before_helper,
            "zircon-management.groovy": template,
            "checkout_guard.py": b"# synthetic checkout guard fixture\n",
            "checkout_product.py": b"# synthetic checkout product fixture\n",
        }
        self.old_metadata = {
            "schemaVersion": 1, "repositoryRoot": str(self.repo),
            "driverDigest": self.driver, "runtimeOperationId": self.runtime,
            "agentLabel": "zircon-windows-agent", "clientSources": clients,
            "ordinaryEntry": {"enabled": True, "actions": ["validate"], "formalAcceptance": False},
            "managedWorktree": {"enabled": True, "schemaVersion": 1,
                                "actions": ["create-worktree", "inspect-worktree", "validate"]},
            "checkoutValidation": {"enabled": True, "policy": "actual-checkout", "guard": "checkout_guard.py"},
            "checkoutProduct": {"enabled": True, "helper": "checkout_product.py", "schemaVersion": 2},
        }
        self.old_root, self.old_metadata = self.seal(self.old_metadata, self.old_files)
        self.installed_root, self.installed_metadata = self.old_root, self.old_metadata
        self.new_files = {**self.old_files, "runner.py": new_runner}
        if adoption or prepared_flow:
            self.installed_root, self.installed_metadata = self.seal(self.old_metadata, self.new_files)
            self.new_files = {**self.new_files, "source_preparation.py": after_helper}
        if prepared_flow:
            # Data stages represent old 2f (bf helper), current b412 (8ab helper),
            # and this candidate. All files/manifests/CPS and State rows are real.
            self.installed_root, self.installed_metadata = self.seal(self.installed_metadata, self.new_files)
            self.new_files = {**self.new_files, "source_preparation.py": actual_helper, "runner.py": actual_runner}
        self.new_root, self.new_metadata = self.seal(self.installed_metadata, self.new_files)
        prefix = ""
        if prepared_flow:
            prefix = addon.decode("utf-8-sig").replace("\r\n", "\n").rstrip("\n") + "\n\n"
            assert len(prefix.encode()) == 2352
        old_pipeline = prefix + self.rendered(self.old_root)
        old_config = self.config_xml(prefix + self.rendered(self.installed_root))
        job = self.repo / ".jenkins/jenkins_home/jobs/zircon-management"
        self.config_path = job / "config.xml"
        self.write(self.config_path, old_config if phase == "before-post" else
                   self.config_xml(prefix + self.rendered(self.new_root)))
        installed = {
            "schemaVersion": 1, "job": "zircon-management",
            "capsule": str(self.installed_root), "capsuleSha256": self.installed_root.name,
            "driverDigest": self.driver, "runtimeOperationId": self.runtime,
            "ordinaryEntry": self.old_metadata["ordinaryEntry"],
            "managedWorktree": self.old_metadata["managedWorktree"], "formalAcceptance": False,
        }
        installed_raw = self.save(self.repo / ".jenkins/state/management/installed.json", installed)
        self.save(self.repo / ".jenkins/state/deployment/driver.json",
                  {"driverDigest": self.driver, "runtimeOperationId": self.runtime})
        self.save(self.repo / ".jenkins/state/deployment-operation.json",
                  {"operationId": self.runtime, "state": "running"})
        self.identity = {
            "repositoryId": "a" * 32, "sessionId": "writer-fixture",
            "requestId": "request-fixture", "operationId": "validate-fixture", "action": "validate",
        }
        self.payload = {
            **self.identity, "driverDigest": self.driver, "runtimeOperationId": self.runtime,
            "observationOnly": False, "validation": {"kind": "batch", "contract": "synthetic"},
        }
        key = checksum(encoded({name: self.identity[name] for name in
                                ("repositoryId", "sessionId", "requestId", "operationId")}))
        input_digest = checksum(encoded({name: value for name, value in self.payload.items()
                                         if name != "observationOnly"}))
        self.control = self.repo / ".jenkins/state/controls/management-validate-fixture.60.0.json"
        control_raw = self.save(self.control, self.payload)
        output = self.control.with_name(self.control.stem + ".response.json")
        self.home = job / "builds/60/build.xml"
        flow = ET.Element("flow-build")
        ET.SubElement(flow, "queueId").text = "3000"
        ET.SubElement(ET.SubElement(flow, "execution"), "script").text = old_pipeline
        self.write(self.home, ET.tostring(flow, encoding="utf-8"))
        self.database = self.repo / ".jenkins/state/coordination.sqlite3"
        with closing(sqlite3.connect(self.database)) as connection, connection:
            connection.execute("CREATE TABLE records (domain TEXT, key TEXT, version INTEGER, "
                               "payload TEXT, updated_at TEXT, PRIMARY KEY(domain,key))")
        self.key = key
        self.operation = {"action": "validate", "inputDigest": input_digest,
                          "payload": copy.deepcopy(self.payload)}
        self.save_operation()
        command = (f'"{self.python}" -B -X utf8 "{self.old_root / "runner.py"}" '
                   f'--capsule "{self.old_root}" --input-file "{self.control}" --output-file "{output}"')
        native = {"pid": 1234, "creationTime": "134366488573868974",
                  "executable": str(self.python), "commandDigest": checksum(command.encode("utf-8"))}
        self.native = {**native, "runnerPath": str(self.old_root / "runner.py"),
                       "capsulePath": str(self.old_root), "inputFile": str(self.control),
                       "outputFile": str(output)}
        self.proof = {
            "schemaVersion": 1, "kind": "management-pinned-active-update-v1",
            "repositoryRoot": str(self.repo), "previousInstalledSha256": checksum(installed_raw),
            "previousJobConfigSha256": checksum(old_config), "previousCapsuleSha256": self.installed_root.name,
            "candidateCapsuleSha256": self.new_root.name, "driverDigest": self.driver,
            "runtimeOperationId": self.runtime,
            "builds": [{"number": 60, "queueId": 3000, "operationKey": key,
                        "capsuleSha256": self.old_root.name,
                        "identity": copy.deepcopy(self.identity), "inputDigest": input_digest,
                        "pipelineSha256": checksum(old_pipeline.encode("utf-8")),
                        "inputRef": {"path": str(self.control), "sha256": checksum(control_raw)},
                        "native": native}],
        }
        self.proof_path = self.repo / ".jenkins/state/management/reports/active-update/proof.json"
        self.request = {
            "repositoryRoot": str(self.repo), "proofPath": str(self.proof_path),
            "expectedInstalledSha256": self.proof["previousInstalledSha256"],
            "expectedCandidateCapsuleSha256": self.new_root.name,
            "expectedPreviousJobConfigSha256": self.proof["previousJobConfigSha256"],
            "expectedDriverDigest": self.driver, "expectedRuntimeOperationId": self.runtime,
            "previousCodexExe": str(self.codex), "codexExe": str(self.codex), "phase": phase,
        }
        self.save_proof()
        params = {
            "SESSION_ID": self.identity["sessionId"], "REQUEST_ID": self.identity["requestId"],
            "OPERATION_ID": self.identity["operationId"], "ACTION": "validate",
            "INPUT_JSON": encoded(self.payload).decode("utf-8"),
            "EXPECTED_DRIVER_DIGEST": self.driver, "RUNTIME_OPERATION_ID": self.runtime,
            "QUERY_ONLY": False,
        }
        self.responses = {
            self.RUNS: {"builds": [{"number": 60, "queueId": 3000, "building": True, "result": None}]},
            self.QUEUE: {"items": []},
            self.BUILD: {"number": 60, "queueId": 3000, "building": True, "result": None,
                         "actions": [{"parameters": [{"name": name, "value": value}
                                                     for name, value in params.items()]}]},
        }
        self.reads, self.native_reads = [], []
        self.natives = {self.native["pid"]: self.native}
        if adoption:
            for number, action in ((67, "create-worktree"), (68, "inspect-worktree"), (69, "prepare-worktree")):
                self.add_active(number, action, self.installed_root)
            self.save_proof()

    def add_active(self, number, action, capsule):
        identity = {**self.identity, "operationId": action + "-fixture-" + str(number), "action": action}
        payload = {**self.payload, **identity}
        key = checksum(encoded({name: identity[name] for name in
                                ("repositoryId", "sessionId", "requestId", "operationId")}))
        input_digest = checksum(encoded({name: value for name, value in payload.items() if name != "observationOnly"}))
        control = self.control.with_name("management-" + identity["operationId"] + "." + str(number) + ".0.json")
        control_raw = self.save(control, payload)
        output = control.with_name(control.stem + ".response.json")
        addon = (self.repo / "tools/jenkins-management/checkout-commit.groovy").read_bytes()
        prefix = addon.decode("utf-8-sig").replace("\r\n", "\n").rstrip("\n") + "\n\n"
        assert len(prefix.encode()) == 2352
        script = prefix + self.rendered(capsule)
        flow = ET.Element("flow-build")
        ET.SubElement(flow, "queueId").text = str(3000 + number - 60)
        ET.SubElement(ET.SubElement(flow, "execution"), "script").text = script
        self.write(self.home.parent.parent / str(number) / "build.xml", ET.tostring(flow, encoding="utf-8"))
        operation = {"action": action, "inputDigest": input_digest, "payload": payload}
        self.save_operation(key, operation)
        command = (f'"{self.python}" -B -X utf8 "{capsule / "runner.py"}" '
                   f'--capsule "{capsule}" --input-file "{control}" --output-file "{output}"')
        native = {"pid": 1234 + number - 60, "creationTime": str(134366488573868974 + number),
                  "executable": str(self.python), "commandDigest": checksum(command.encode())}
        self.natives[native["pid"]] = {**native, "runnerPath": str(capsule / "runner.py"),
            "capsulePath": str(capsule), "inputFile": str(control), "outputFile": str(output)}
        self.proof["builds"].append({"number": number, "queueId": 3000 + number - 60,
            "operationKey": key, "identity": identity, "inputDigest": input_digest,
            "capsuleSha256": capsule.name, "pipelineSha256": checksum(script.encode()),
            "inputRef": {"path": str(control), "sha256": checksum(control_raw)}, "native": native})
        path = self.BUILD.replace("/60/", "/" + str(number) + "/")
        params = copy.deepcopy(self.responses[self.BUILD]["actions"][0]["parameters"])
        updates = {"OPERATION_ID": identity["operationId"], "ACTION": action, "INPUT_JSON": encoded(payload).decode()}
        for item in params:
            if item["name"] in updates:
                item["value"] = updates[item["name"]]
        self.responses[path] = {"number": number, "queueId": 3000 + number - 60, "building": True,
                                "result": None, "actions": [{"parameters": params}]}
        self.responses[self.RUNS]["builds"].append({"number": number, "queueId": 3000 + number - 60,
                                                   "building": True, "result": None})

    @staticmethod
    def write(path, raw):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(raw)
        return raw

    def save(self, path, value):
        return self.write(path, encoded(value))

    def seal(self, metadata, files):
        metadata = copy.deepcopy(metadata)
        metadata["files"] = {name: checksum(raw) for name, raw in files.items()}
        raw = encoded(metadata)
        root = self.repo / ".jenkins/runtime/management" / checksum(raw)
        for name, content in files.items():
            self.write(root / name, content)
        self.write(root / "capsule.json", raw)
        return root, metadata

    def replace_candidate(self, *, files=None, metadata=None):
        self.new_root, self.new_metadata = self.seal(
            metadata if metadata is not None else self.new_metadata,
            files if files is not None else self.new_files)
        self.proof["candidateCapsuleSha256"] = self.new_root.name
        self.request["expectedCandidateCapsuleSha256"] = self.new_root.name
        self.save_proof()

    def rendered(self, capsule):
        # Expected CPS evidence is independent of the guard's template renderer.
        values = ("zircon-windows-agent", str(self.repo), str(self.python), str(self.codex),
                  str(capsule), self.driver, self.runtime)
        names = ("agent", "repository", "python", "codex", "capsule", "driver", "runtime")
        return "\n".join("def " + name + " = '" + value.replace("\\", "/").replace("'", "\\'") + "'"
                         for name, value in zip(names, values)) + "\n"

    @staticmethod
    def config_xml(script):
        config = ET.Element("flow-definition")
        ET.SubElement(ET.SubElement(config, "definition"), "script").text = script
        return ET.tostring(config, encoding="utf-8")

    def save_operation(self, key=None, operation=None):
        with closing(sqlite3.connect(self.database)) as connection, connection:
            connection.execute("INSERT OR REPLACE INTO records VALUES (?,?,?,?,?)",
                               ("management_operation", key or self.key, 1,
                                encoded(self.operation if operation is None else operation).decode("utf-8"), "fixture-time"))

    def save_proof(self):
        self.request["proofSha256"] = checksum(self.save(self.proof_path, self.proof))

    def get_json(self, path):
        self.reads.append(path)
        return copy.deepcopy(self.responses[path])

    def observe_native(self, pid):
        self.native_reads.append(pid)
        return copy.deepcopy(self.natives[pid])

    def verify(self):
        return guard.verify_update(self.request, get_json=self.get_json, observe_native=self.observe_native)

    def files(self):
        return {path.relative_to(self.repo).as_posix(): checksum(path.read_bytes())
                for path in self.repo.rglob("*") if path.is_file()}


class InstallGuardTests(unittest.TestCase):
    @contextmanager
    def fixture(self, phase="before-post", adoption=False, prepared_flow=False):
        with tempfile.TemporaryDirectory(prefix="management-active-update-") as directory:
            try:
                yield ActiveUpdateFixture(directory, phase, adoption, prepared_flow)
            finally:
                # Release sqlite's statement-cache cycles before Windows temp cleanup.
                gc.collect()

    def test_metadata_only_update_preserves_active_original_build_before_and_after_post(self):
        for phase, adoption in ((phase, adoption) for phase in ("before-post", "after-post")
                                 for adoption in (False, True)):
            with self.subTest(phase=phase, adoption=adoption), self.fixture(phase, adoption) as fixture:
                before = fixture.files()
                result = fixture.verify()
                self.assertEqual("pinned-active-compatible", result["status"])
                self.assertEqual(phase, result["phase"])
                self.assertEqual(fixture.request["proofSha256"], result["proofSha256"])
                self.assertEqual(fixture.installed_root.name, result["previousCapsuleSha256"])
                self.assertEqual(fixture.new_root.name, result["candidateCapsuleSha256"])
                self.assertNotEqual(result["previousCapsuleSha256"], result["candidateCapsuleSha256"])
                numbers = [60, 67, 68, 69] if adoption else [60]
                self.assertEqual(numbers, result["observedActiveBuildNumbers"])
                expected_capsules = {str(number): (fixture.old_root if number == 60 else fixture.installed_root).name
                                     for number in numbers}
                self.assertEqual(expected_capsules, result["observedActiveCapsules"])
                self.assertEqual("same-byte-identity-adoption-v1" if adoption else "unchanged",
                                 result["sourcePreparationPolicy"])
                self.assertRegex(result["runnerInterfaceSha256"], r"^[0-9a-f]{64}$")
                self.assertEqual("windows-command-line-utf8-sha256-v1", result["commandDigestPolicy"])
                self.assertFalse(result["runtimeChanged"])
                self.assertTrue(result["oldCapsuleRetained"])
                self.assertEqual([fixture.RUNS, fixture.QUEUE] +
                    [fixture.BUILD.replace("/60/", "/" + str(number) + "/") for number in numbers], fixture.reads)
                self.assertEqual([1234 + number - 60 for number in numbers], fixture.native_reads)
                self.assertEqual(before, fixture.files())
                self.assertEqual(phase == "before-post",
                                 checksum(fixture.config_path.read_bytes()) ==
                                 fixture.proof["previousJobConfigSha256"])
                self.assertFalse(Path(fixture.native["outputFile"]).exists())


        for phase in ("before-post", "after-post"):
            with self.subTest(phase=phase, release="prepared-WT"), self.fixture(
                    phase, prepared_flow=True) as fixture:
                fixture.add_active(72, "validate", fixture.installed_root)
                fixture.save_proof()
                before = fixture.files()
                result = fixture.verify()
                self.assertEqual("pinned-active-compatible", result["status"])
                self.assertEqual("prepared-worktree-absent-owned-v1", result["sourcePreparationPolicy"])
                self.assertEqual([60, 72], result["observedActiveBuildNumbers"])
                self.assertEqual({"60": fixture.old_root.name, "72": fixture.installed_root.name},
                                 result["observedActiveCapsules"])
                self.assertEqual(guard.ADOPTION_HELPER_BEFORE, fixture.old_metadata["files"]["source_preparation.py"])
                self.assertEqual(guard.PREPARED_FLOW_HELPER_BEFORE,
                                 fixture.installed_metadata["files"]["source_preparation.py"])
                self.assertEqual(fixture.old_metadata["files"]["runner.py"],
                                 fixture.installed_metadata["files"]["runner.py"])
                self.assertEqual(before, fixture.files())
                self.assertFalse(result["runtimeChanged"])
                self.assertTrue(result["oldCapsuleRetained"])
        for boundary in ("helper-outside-suffix", "runner-outside-flow"):
            with self.subTest(release="prepared-WT", boundary=boundary), self.fixture(
                    prepared_flow=True) as fixture:
                files = dict(fixture.new_files)
                name = "source_preparation.py" if boundary == "helper-outside-suffix" else "runner.py"
                files[name] += b"\n# unrelated candidate drift must be rejected\n"
                fixture.replace_candidate(files=files)
                before = fixture.files()
                with self.assertRaises(ValueError):
                    fixture.verify()
                self.assertEqual(before, fixture.files())

    def test_unlisted_active_build_and_queued_management_work_require_reconciliation(self):
        for boundary in ("active-build", "queued-work", "queued-proof-without-pid"):
            with self.subTest(boundary=boundary), self.fixture() as fixture:
                if boundary == "active-build":
                    fixture.responses[fixture.RUNS]["builds"].append(
                        {"number": 61, "queueId": 3001, "building": True, "result": None})
                else:
                    fixture.responses[fixture.QUEUE]["items"].append({"task": {"name": "zircon-management"}})
                    if boundary == "queued-proof-without-pid":
                        fixture.proof["queued"] = [{"queueId": 3001, "capsuleSha256": fixture.old_root.name}]
                        fixture.save_proof()
                with self.assertRaisesRegex(ValueError, "^pinned-active update:"):
                    fixture.verify()
                self.assertEqual([], fixture.native_reads)

    def test_native_control_and_request_identity_drift_are_rejected(self):
        for boundary in ("birth", "command", "capsule", "control", "control-bytes", "identity",
                         "active-capsule", "runner-path", "unsupported-action"):
            with self.subTest(boundary=boundary), self.fixture() as fixture:
                if boundary == "birth":
                    fixture.native["creationTime"] = str(int(fixture.native["creationTime"]) + 1)
                elif boundary == "command":
                    fixture.native["commandDigest"] = "0" * 64
                elif boundary == "capsule":
                    fixture.native["capsulePath"] = str(fixture.new_root)
                elif boundary == "control":
                    other = fixture.control.with_name("another-delivery.json")
                    fixture.write(other, fixture.control.read_bytes())
                    fixture.native["inputFile"] = str(other)
                elif boundary == "control-bytes":
                    fixture.write(fixture.control, fixture.control.read_bytes() + b"\n")
                elif boundary == "identity":
                    fixture.proof["builds"][0]["identity"]["sessionId"] = "foreign-writer"
                    fixture.save_proof()
                elif boundary == "active-capsule":
                    fixture.proof["builds"][0]["capsuleSha256"] = fixture.new_root.name
                    fixture.save_proof()
                elif boundary == "runner-path":
                    fixture.native["runnerPath"] = str(fixture.new_root / "runner.py")
                else:
                    fixture.identity["action"] = fixture.payload["action"] = fixture.operation["action"] = "gc"
                    fixture.operation["payload"] = copy.deepcopy(fixture.payload)
                    input_digest = checksum(encoded({name: value for name, value in fixture.payload.items()
                                                     if name != "observationOnly"}))
                    fixture.operation["inputDigest"] = input_digest
                    fixture.save_operation()
                    raw = fixture.save(fixture.control, fixture.payload)
                    binding = fixture.proof["builds"][0]
                    binding["identity"]["action"], binding["inputDigest"] = "gc", input_digest
                    binding["inputRef"]["sha256"] = checksum(raw)
                    for item in fixture.responses[fixture.BUILD]["actions"][0]["parameters"]:
                        if item["name"] == "ACTION":
                            item["value"] = "gc"
                        elif item["name"] == "INPUT_JSON":
                            item["value"] = encoded(fixture.payload).decode()
                    fixture.save_proof()
                before = fixture.files()
                with self.assertRaisesRegex(ValueError, "^pinned-active update:"):
                    fixture.verify()
                self.assertEqual(before, fixture.files())

    def test_execution_ast_capsule_client_and_capability_changes_are_rejected(self):
        for boundary in ("execution-ast", "capsule-file", "client", "capability",
                         "active-capsule-bytes", "helper-outside-hunk", "helper-and-metadata", "other-member"):
            adoption = boundary in {"helper-outside-hunk", "helper-and-metadata", "other-member"}
            with self.subTest(boundary=boundary), self.fixture(adoption=adoption) as fixture:
                if boundary == "execution-ast":
                    files = {**fixture.new_files, "runner.py": fixture.new_files["runner.py"].replace(
                        b"return {'unchanged': payload}", b"return {'changed': payload}")}
                    fixture.replace_candidate(files=files)
                elif boundary == "capsule-file":
                    fixture.replace_candidate(files={**fixture.new_files,
                        "source_preparation.py": b"# changed preparation implementation\n"})
                elif boundary == "client":
                    name = fixture.CLIENTS[0]
                    raw = fixture.write(fixture.repo / name, b"# changed ordinary client\n")
                    metadata = copy.deepcopy(fixture.new_metadata)
                    metadata["clientSources"][name] = checksum(raw)
                    fixture.replace_candidate(metadata=metadata)
                elif boundary == "capability":
                    metadata = copy.deepcopy(fixture.new_metadata)
                    metadata["ordinaryEntry"]["formalAcceptance"] = True
                    fixture.replace_candidate(metadata=metadata)
                elif boundary == "active-capsule-bytes":
                    path = fixture.old_root / "runner.py"
                    fixture.write(path, path.read_bytes() + b"# tampered original capsule\n")
                elif boundary == "helper-outside-hunk":
                    fixture.replace_candidate(files={**fixture.new_files,
                        "source_preparation.py": fixture.new_files["source_preparation.py"] + b"# unrelated helper change\n"})
                elif boundary == "helper-and-metadata":
                    fixture.replace_candidate(files={**fixture.new_files,
                        "runner.py": fixture.new_files["runner.py"].replace(b"'revision': 2", b"'revision': 3")})
                else:
                    fixture.replace_candidate(files={**fixture.new_files,
                        "checkout_guard.py": b"# unrelated capsule member change\n"})
                before = fixture.files()
                with self.assertRaisesRegex(ValueError, "^pinned-active update:"):
                    fixture.verify()
                self.assertEqual(before, fixture.files())

    def test_proof_state_and_home_cps_must_bind_the_original_delivery(self):
        for boundary in ("proof-bytes", "state-digest", "state-payload", "home-queue", "home-script",
                         "addon-prefix", "old-cps-rebound"):
            with self.subTest(boundary=boundary), self.fixture(adoption=boundary in {"addon-prefix", "old-cps-rebound"}) as fixture:
                if boundary == "proof-bytes":
                    fixture.write(fixture.proof_path, fixture.proof_path.read_bytes() + b"\n")
                elif boundary == "state-digest":
                    fixture.operation["inputDigest"] = "0" * 64
                    fixture.save_operation()
                elif boundary == "state-payload":
                    fixture.operation["payload"]["validation"]["contract"] = "another-contract"
                    fixture.save_operation()
                else:
                    home = fixture.home if boundary != "addon-prefix" else fixture.home.parent.parent / "67/build.xml"
                    flow = ET.fromstring(home.read_bytes())
                    if boundary == "home-queue":
                        flow.find("queueId").text = "3001"
                    elif boundary == "addon-prefix":
                        flow.find("./execution/script").text = flow.findtext("./execution/script")[1:]
                    elif boundary == "old-cps-rebound":
                        flow.find("./execution/script").text = fixture.rendered(fixture.installed_root)
                    else:
                        flow.find("./execution/script").text += "// changed CPS\n"
                    fixture.write(home, ET.tostring(flow, encoding="utf-8"))
                    if boundary in {"addon-prefix", "old-cps-rebound"}:
                        number = 67 if boundary == "addon-prefix" else 60
                        next(binding for binding in fixture.proof["builds"] if binding["number"] == number)[
                            "pipelineSha256"] = checksum(flow.findtext("./execution/script").encode())
                        fixture.save_proof()
                before = fixture.files()
                with self.assertRaisesRegex(ValueError, "^pinned-active update:"):
                    fixture.verify()
                self.assertEqual(before, fixture.files())


if __name__ == "__main__":
    unittest.main()
