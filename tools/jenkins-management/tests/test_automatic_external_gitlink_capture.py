"""Ordinary checkout discovers omitted Gitlinks without changing request identity."""
from __future__ import annotations

import copy
from contextlib import contextmanager
import importlib.util
import os
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch

from tools.jenkins import managed_worktree as formal


ROOT = Path(__file__).resolve().parents[3]
MANAGEMENT = ROOT / "tools/jenkins-management"


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


runner = load("automatic_external_gitlink_test_runner", MANAGEMENT / "runner.py")
preparation = load("automatic_external_gitlink_test_preparation", MANAGEMENT / "source_preparation.py")


def snapshot(head, files):
    value = {"head": head, "files": files, "diffSha256": runner.sha(runner.encoded(files))}
    return {**value, "sourceDigest": runner.sha(runner.encoded(value))}


@contextmanager
def physical_metadata():
    # Synthetic Git metadata only: no checkout, Git process or fixture files.
    with patch.object(runner, "plain", side_effect=Path), \
            patch.object(formal, "_plain", side_effect=Path), \
            patch.object(Path, "is_dir", return_value=True), \
            patch.object(Path, "resolve", lambda value, *args, **kwargs: value):
        yield


class AutomaticExternalGitlinkCaptureTests(unittest.TestCase):
    def management(self):
        value = runner.Management.__new__(runner.Management)
        value.repo = ROOT
        value.payload = {
            "repositoryId": "repository", "sessionId": "original-session",
            "requestId": "original-request", "operationId": "original-operation",
            "action": "validate", "sourceView": "checkout",
            "externalRoots": [str(ROOT.parent / "zr_vm")],
        }
        value.key = "a" * 64
        value.input_digest = runner.sha(runner.encoded(value.payload))
        return value

    def test_omitted_checkout_gitlinks_capture_index_heads_and_complete_input_without_rewriting_identity(self):
        management = self.management()
        dependency = ROOT.parent / "zr_vm"
        names = ["vendor/v" + str(index) for index in range(7)]
        indexes = {name: format(index + 17, "040x") for index, name in enumerate(names)}
        heads = {name: format(index + 1, "040x") for index, name in enumerate(names)}
        foreign_hash = ["b" * 64]
        engine = snapshot("e" * 40, {"owned.rs": "a" * 64})
        original = copy.deepcopy(management.payload)
        original_identity = (management.key, management.input_digest)

        def git(root, *arguments):
            if root == dependency and arguments == ("ls-files", "--stage", "-z"):
                raw = "".join("160000 " + indexes[name] + " 0\t" + name + "\0" for name in names)
            elif arguments == ("ls-files", "--cached", "-z"):
                raw = "\0".join(["vm.rs", *names] if root == dependency else ["lib.rs"]) + "\0"
            else:
                raise AssertionError("Unexpected Git metadata query: " + repr(arguments))
            return SimpleNamespace(stdout=raw.encode("utf-8"))

        def text(root, *arguments):
            if arguments == ("rev-parse", "--show-toplevel"):
                return str(root)
            if arguments == ("rev-parse", "HEAD"):
                return heads[root.relative_to(dependency).as_posix()]
            raise AssertionError("Unexpected Git text query")

        def capture(root, *, payload):
            if root == ROOT:
                return copy.deepcopy(engine)
            self.assertEqual(dependency, root)
            self.assertIsNot(management.payload, payload)
            self.assertEqual(original, {key: value for key, value in payload.items()
                                        if key != "externalGitlinks"})
            self.assertEqual({name: {"indexCommit": indexes[name], "head": heads[name]}
                              for name in names}, payload["externalGitlinks"])
            files = {"vm.rs": "c" * 64, "docs/foreign.md": foreign_hash[0]}
            files.update({name: {"gitlinkCommit": indexes[name],
                                "source": snapshot(heads[name], {"lib.rs": "d" * 64})}
                          for name in names})
            return snapshot("f" * 40, files)

        management.git, management.text, management.source = git, text, capture
        with physical_metadata(), patch.object(runner, "source_helper", return_value=preparation):
            root, before = management.validation_source()
            heads[names[0]] = "9" * 40
            foreign_hash[0] = "8" * 64
            _, after = management.validation_source()
        self.assertEqual(ROOT, root)
        external = before["externalRoots"][0]
        self.assertEqual(str(dependency), external["path"])
        self.assertEqual(9, len(preparation.source_files(external["source"])))
        self.assertEqual("b" * 64, external["source"]["files"]["docs/foreign.md"])
        for index, name in enumerate(names):
            self.assertEqual(indexes[name], external["source"]["files"][name]["gitlinkCommit"])
            self.assertEqual(format(index + 1, "040x"),
                             external["source"]["files"][name]["source"]["head"])
        self.assertEqual(engine, before["source"])
        self.assertEqual(runner.sha(runner.encoded({key: value for key, value in before.items()
                                                  if key != "sourceDigest"})), before["sourceDigest"])
        self.assertNotEqual(before["sourceDigest"], after["sourceDigest"])
        self.assertEqual(original, management.payload)
        self.assertEqual(original_identity, (management.key, management.input_digest))

    def test_explicit_null_empty_or_incorrect_gitlinks_keep_original_strict_source_rejection(self):
        management = self.management()
        dependency = ROOT.parent / "zr_vm"
        self.assertEqual(r"E:\Git\zr_vm".casefold(), str(dependency).casefold())
        names = ["vendor/v" + str(index) for index in range(7)]
        indexes = {name: format(index + 17, "040x") for index, name in enumerate(names)}
        wrong = {name: {"indexCommit": indexes[name], "head": "1" * 40} for name in names}
        wrong[names[0]]["indexCommit"] = "0" * 40

        def git(root, *arguments):
            self.assertEqual(dependency, root)
            if arguments == ("ls-files", "--stage", "-z"):
                raw = "".join("160000 " + indexes[name] + " 0\t" + name + "\0" for name in names)
            elif arguments == ("ls-files", "--cached", "--others", "--exclude-standard", "-z"):
                raw = "\0".join(names) + "\0"
            else:
                raise AssertionError("Unexpected Git metadata query")
            return SimpleNamespace(stdout=raw.encode("utf-8"))

        management.git = git
        management.text = lambda root, *arguments: str(root)
        management.source = runner.Management.source.__get__(management)
        for supplied in (None, {}, wrong):
            with self.subTest(externalGitlinks=supplied):
                management.payload["externalGitlinks"] = copy.deepcopy(supplied)
                original = copy.deepcopy(management.payload)
                with physical_metadata(), patch.object(
                        formal, "_external_capture",
                        side_effect=AssertionError("Explicit declarations must not be replaced")):
                    with self.assertRaisesRegex(
                            ValueError, "Existing external submodules require explicit index and HEAD registration"):
                        management.validation_source()
                self.assertEqual(original, management.payload)


    @unittest.skipUnless(os.name == "nt", "Native validation input pins require Windows")
    def test_validate_source_uses_real_native_pins_and_recaptures_changed_and_absent_files(self):
        from tools.jenkins.resources import canonical_build_root

        area = canonical_build_root(r"E:\cargo-targets").namespace() / "tmp/managed-worktree-tests"
        area.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="validate-native-", dir=area) as temporary:
            root = Path(temporary)
            self.assertEqual(root, root.resolve())
            self.assertTrue(root.is_relative_to(area), "Temporary cleanup must stay inside the physical fixture parent")
            (root / "shared").mkdir()
            first_file = root / "shared/a.rs"
            first_file.write_bytes(b"abc")
            (root / "shared/b.rs").write_bytes(b"def")
            prior_stat = first_file.stat()
            management = self.management()
            management.action = "validate"
            original = copy.deepcopy(management.payload)
            identity = (management.key, management.input_digest)

            def git(selected, *arguments):
                self.assertEqual(root, selected)
                if arguments == ("ls-files", "--cached", "--others", "--exclude-standard", "-z"):
                    raw = b"shared/a.rs\0shared/b.rs\0new/input.rs\0"
                elif arguments == ("diff", "--no-ext-diff", "--no-textconv", "--binary", "--full-index", "HEAD", "--"):
                    raw = b""
                else:
                    raise AssertionError("Unexpected Git metadata query")
                return SimpleNamespace(stdout=raw)

            def text(selected, *arguments):
                self.assertEqual(root, selected)
                self.assertEqual(("rev-parse", "HEAD"), arguments)
                return "e" * 40

            management.git, management.text = git, text
            real_plain = runner.plain

            def helper_only(path):
                # Loading the real helper is allowed; per-file fallback is not.
                self.assertEqual(MANAGEMENT / "source_preparation.py", Path(path),
                                 "Validate must use native input pins for every source file")
                return real_plain(path)

            native = preparation._managed_native()
            with patch.object(runner, "plain", side_effect=helper_only), patch.object(
                    native, "_open_relative_handle", wraps=native._open_relative_handle) as opened:
                before = management.source(root)
                first_file.write_bytes(b"xyz")
                os.utime(first_file, ns=(prior_stat.st_atime_ns, prior_stat.st_mtime_ns))
                (root / "new").mkdir()
                (root / "new/input.rs").write_bytes(b"new")
                after = management.source(root)

            self.assertEqual({"shared/a.rs": runner.sha(b"abc"), "shared/b.rs": runner.sha(b"def"),
                              "new/input.rs": None}, before["files"])
            self.assertEqual({"shared/a.rs": runner.sha(b"xyz"), "shared/b.rs": runner.sha(b"def"),
                              "new/input.rs": runner.sha(b"new")}, after["files"])
            self.assertEqual("e" * 40, before["head"])
            self.assertEqual(runner.sha(b""), before["diffSha256"])
            self.assertEqual(runner.sha(runner.encoded({key: value for key, value in before.items()
                                                      if key != "sourceDigest"})), before["sourceDigest"])
            self.assertNotEqual(before["sourceDigest"], after["sourceDigest"])
            shared = [call for call in opened.call_args_list
                      if call.args[1] == "shared" and call.kwargs["options"] & native._FILE_DIRECTORY_FILE]
            leaves = [call for call in opened.call_args_list
                      if call.args[1] == "a.rs" and not call.kwargs["options"] & native._FILE_DIRECTORY_FILE]
            self.assertEqual(2, len(shared), "A fresh stage must prove the shared parent once per capture")
            self.assertEqual(2, len(leaves), "Both captures must read the actual file again")
            self.assertTrue(all(call.kwargs["share"] == native._FILE_SHARE_READ for call in shared + leaves))
            self.assertEqual(original, management.payload)
            self.assertEqual(identity, (management.key, management.input_digest))


if __name__ == "__main__":
    unittest.main()
