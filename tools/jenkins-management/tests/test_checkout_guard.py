"""Windows contract tests for Jenkins' native checkout read guard.

Filesystem fixtures are intentionally confined to the root provisioned by the
Jenkins harness. These tests must fail with a configuration error when that
root was not supplied; they never fall back to the system temporary directory.
"""
from __future__ import annotations

import hashlib
import json
import mmap
import os
from pathlib import Path
import shutil
import sys
import unittest
from unittest.mock import patch
import uuid


MANAGEMENT_ROOT = Path(__file__).absolute().parents[1]
if str(MANAGEMENT_ROOT) not in sys.path:
    sys.path.insert(0, str(MANAGEMENT_ROOT))

import checkout_guard


REGISTERED_FIXTURE_ROOTS = (
    Path(
        "E:/cargo-targets/zircon-local/management-regression/"
        "actualcheckout-20261013/fixtures"
    ),
    Path(
        "E:/cargo-targets/zircon-local/management-regression/"
        "actualcheckout-20261013/runner-integration-01/fixtures"
    ),
    Path(
        "E:/cargo-targets/zircon-local/management-regression/"
        "actualcheckout-20261013/runner-integration-02/fixtures"
    ),
    Path(
        "E:/cargo-targets/zircon-local/management-regression/"
        "actualcheckout-20261013/runner-integration-03/fixtures"
    ),
    Path(
        "E:/cargo-targets/zircon-local/management-regression/"
        "actualcheckout-20261013/runner-integration-04/fixtures"
    ),
    Path(
        "E:/cargo-targets/zircon-local/management-regression/"
        "actualcheckout-20261013/runner-integration-05/fixtures"
    ),
    Path(
        "E:/cargo-targets/zircon-local/management-regression/"
        "actualcheckout-20261013/runner-integration-06/fixtures"
    ),
    Path(
        "E:/cargo-targets/zircon-local/management-regression/"
        "actualcheckout-20261013/runner-integration-07/fixtures"
    ),
)


def _has_reparse_ancestor(candidate: Path) -> bool:
    for part in (candidate, *candidate.parents):
        try:
            status = part.lstat()
        except FileNotFoundError:
            continue
        except OSError:
            return True
        if part.is_symlink() or getattr(status, "st_file_attributes", 0) & 0x400:
            return True
    return False


def _approved_test_root() -> Path:
    configured = os.environ.get("CHECKOUT_GUARD_TEST_ROOT")
    if not configured:
        raise RuntimeError(
            "Jenkins must set CHECKOUT_GUARD_TEST_ROOT to a registered "
            "actualcheckout-20261013 fixture root"
        )
    candidate = Path(configured)
    if not candidate.is_absolute():
        raise RuntimeError("CHECKOUT_GUARD_TEST_ROOT must be an absolute path")
    literal = candidate.absolute()
    registered = next((root for root in REGISTERED_FIXTURE_ROOTS
                       if os.path.normcase(str(literal)) == os.path.normcase(str(root.absolute()))), None)
    if registered is None:
        allowed = "; ".join(str(root) for root in REGISTERED_FIXTURE_ROOTS)
        raise RuntimeError("CHECKOUT_GUARD_TEST_ROOT must equal one registered path: " + allowed)
    if _has_reparse_ancestor(literal):
        raise RuntimeError("CHECKOUT_GUARD_TEST_ROOT must have no symlink or reparse-point ancestors")
    try:
        actual = literal.resolve(strict=True)
        expected = registered.resolve(strict=True)
    except OSError as error:
        raise RuntimeError(
            "The selected registered CHECKOUT_GUARD_TEST_ROOT must already exist"
        ) from error
    if os.path.normcase(str(actual)) != os.path.normcase(str(expected)):
        raise RuntimeError("CHECKOUT_GUARD_TEST_ROOT does not resolve to its registered physical path")
    if not actual.is_dir():
        raise RuntimeError("CHECKOUT_GUARD_TEST_ROOT is not a directory")
    return actual


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class CheckoutGuardTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fixture_root = _approved_test_root()
        if os.name != "nt":
            raise unittest.SkipTest("CheckoutReadGuard requires native Windows handles")

    def setUp(self):
        self.case_root = self.fixture_root / f"checkout-guard-{uuid.uuid4().hex}"
        self.case_root.mkdir()
        self.addCleanup(shutil.rmtree, self.case_root)

    def make_guard(self, source, files, watch_roots=None, ignored_activity=None, watch_interests=None,
                   trusted_native_inputs=None):
        input_map = {}
        for path, digest in files.items():
            # Preserve a string path when a test deliberately exercises a
            # reparse-point alias. Path fixtures use their resolved identity.
            key = str(path.resolve(strict=False)) if isinstance(path, Path) else str(path)
            input_map[key] = digest
        roots = watch_roots if watch_roots is not None else [source]
        root_names = [str(root.resolve(strict=True)) for root in roots]
        ignored = None
        if ignored_activity is not None:
            ignored = {
                str(root.resolve(strict=True)): list(subtrees)
                for root, subtrees in ignored_activity.items()
            }
        guard = checkout_guard.CheckoutReadGuard(
            str(source.resolve(strict=True)),
            files=input_map,
            watch_roots=root_names,
            ignored_activity=ignored,
            watch_interests=watch_interests,
            trusted_native_inputs=trusted_native_inputs,
        )
        self.addCleanup(guard.close)
        return guard

    def test_readers_work_and_file_mutations_wait_until_close(self):
        source = self.case_root / "source"
        source.mkdir()
        item = source / "input.bin"
        original = b"pinned checkout input"
        item.write_bytes(original)

        guard = self.make_guard(source, {item: _sha256(item)}).acquire()
        with item.open("rb") as reader:
            self.assertEqual(reader.read(), original)

        with self.assertRaises(OSError):
            item.write_bytes(b"changed")
        with self.assertRaises(OSError):
            item.unlink()
        with self.assertRaises(OSError):
            item.rename(source / "renamed.bin")
        self.assertEqual(item.read_bytes(), original)
        self.assertTrue(guard.assert_stable()["stable"])

        evidence = guard.finish()
        self.assertTrue(evidence["complete"])
        self.assertTrue(evidence["sourceNoWrites"])
        self.assertTrue(evidence["cancellationComplete"])
        self.assertTrue(evidence["waitersJoined"])
        with self.assertRaises(OSError):
            with item.open("r+b"):
                pass

        guard.close()
        item.write_bytes(b"released")
        renamed = source / "renamed.bin"
        item.rename(renamed)
        renamed.unlink()

    def test_checksum_buffer_reuse_preserves_large_short_and_empty_file_hashes(self):
        source = self.case_root / "checksum-buffer"
        source.mkdir()
        contents = [b"a" * (1024 * 1024 + 17), b"xy", b"",
                    b"z" * (2 * 1024 * 1024 + 7)]
        files = {}
        for index, data in enumerate(contents):
            item = source / ("%02d.bin" % index)
            item.write_bytes(data)
            files[item] = hashlib.sha256(data).hexdigest()
        guard = self.make_guard(source, files).acquire()
        self.assertTrue(guard.assert_stable()["stable"])
        evidence = guard.finish()
        for name in ("complete", "sourceNoWrites", "cancellationComplete", "waitersJoined"):
            self.assertTrue(evidence[name], name)

    def test_existing_writer_and_writable_mapping_are_busy_and_partial_pins_release(self):
        source = self.case_root / "source"
        source.mkdir()
        first = source / "a-input.bin"
        second = source / "z-input.bin"
        first.write_bytes(b"first input")
        second.write_bytes(b"second input")
        inputs = {first: _sha256(first), second: _sha256(second)}

        with second.open("r+b"):
            guard = self.make_guard(source, inputs)
            with self.assertRaises(checkout_guard.CheckoutGuardBusy) as caught:
                guard.acquire()
            self.assertEqual(caught.exception.code, "checkout_source_busy")
            self.assertTrue(caught.exception.retryable)
            # The first pin was acquired before the second file became busy.
            # Its write access must work after acquire rolls back that partial set.
            with first.open("r+b"):
                pass

        stream = second.open("r+b")
        mapped = mmap.mmap(stream.fileno(), 0, access=mmap.ACCESS_WRITE)
        stream.close()
        try:
            guard = self.make_guard(source, inputs)
            with self.assertRaises(checkout_guard.CheckoutGuardBusy) as caught:
                guard.acquire()
            self.assertEqual(caught.exception.code, "checkout_source_busy")
            with first.open("r+b"):
                pass
        finally:
            mapped.close()

        guard = self.make_guard(source, inputs).acquire()
        self.assertEqual(guard.facts()["heldFileCount"], 2)
        guard.close()

    def test_metadata_and_transient_membership_activity_are_detected(self):
        source = self.case_root / "source"
        source.mkdir()
        item = source / "input.bin"
        item.write_bytes(b"stable bytes")

        guard = self.make_guard(source, {item: _sha256(item)}).acquire()
        before = item.stat()
        os.utime(item, ns=(before.st_atime_ns, before.st_mtime_ns + 10_000_000_000))
        self.assertTrue(guard.wait_for_change(10.0))
        with self.assertRaises(checkout_guard.CheckoutGuardChanged):
            guard.assert_stable()
        metadata_evidence = guard.finish()
        self.assertFalse(metadata_evidence["stable"])
        self.assertTrue(metadata_evidence["events"])
        guard.close()

        # Re-pin the now-current identity, then prove that a create/delete
        # pair remains visible even though the directory ends unchanged.
        guard = self.make_guard(source, {item: _sha256(item)}).acquire()
        transient = source / "transient.tmp"
        transient.write_bytes(b"short-lived")
        transient.unlink()
        self.assertTrue(guard.wait_for_change(10.0))
        with self.assertRaises(checkout_guard.CheckoutGuardChanged):
            guard.assert_stable()
        membership_evidence = guard.finish()
        self.assertFalse(membership_evidence["membershipStable"])
        self.assertTrue(membership_evidence["events"])

    def test_nested_external_and_declared_missing_inputs_are_monitored(self):
        source = self.case_root / "source"
        nested = source / "nested"
        external = self.case_root / "external"
        nested.mkdir(parents=True)
        external.mkdir()
        nested_input = nested / "source-input.bin"
        external_input = external / "external-input.bin"
        missing_input = external / "later-input.bin"
        nested_input.write_bytes(b"nested")
        external_input.write_bytes(b"external")

        guard = self.make_guard(
            source,
            {
                nested_input: _sha256(nested_input),
                external_input: _sha256(external_input),
                missing_input: None,
            },
            watch_roots=[source, external],
        ).acquire()
        facts = guard.facts()
        self.assertEqual(facts["heldFileCount"], 2)
        self.assertEqual(facts["missingFileCount"], 1)
        self.assertEqual(external_input.read_bytes(), b"external")

        missing_input.write_bytes(b"appeared")
        self.assertTrue(guard.wait_for_change(10.0))
        with self.assertRaises(checkout_guard.CheckoutGuardChanged):
            guard.assert_stable()
        evidence = guard.finish()
        self.assertFalse(evidence["membershipStable"])
        self.assertTrue(evidence["events"])

    def test_ignored_activity_is_scoped_and_cannot_cover_a_registered_input(self):
        source = self.case_root / "source"
        ignored = source / "generated"
        source.mkdir()
        ignored.mkdir()
        pinned = source / "input.bin"
        pinned.write_bytes(b"pinned")

        guard = self.make_guard(
            source,
            {pinned: _sha256(pinned)},
            ignored_activity={source: ["generated"]},
        ).acquire()
        (ignored / "untracked.bin").write_bytes(b"ignored activity")
        evidence = guard.finish()
        self.assertTrue(evidence["stable"])
        self.assertTrue(evidence["membershipStable"])
        self.assertEqual(evidence["events"], [])
        self.assertGreater(evidence["watchers"][0]["ignoredEventCount"], 0)
        guard.close()

        ignored_input = ignored / "registered.bin"
        ignored_input.write_bytes(b"must not be ignored")
        with self.assertRaises(ValueError):
            self.make_guard(
                source,
                {ignored_input: _sha256(ignored_input)},
                ignored_activity={source: ["generated"]},
            )

    def test_hardlink_or_directory_aliases_are_rejected_when_supported(self):
        source = self.case_root / "source"
        physical = source / "physical"
        physical.mkdir(parents=True)
        item = physical / "input.bin"
        item.write_bytes(b"one physical input")
        digest = _sha256(item)
        alias_kinds = []

        hardlink = source / "hardlink.bin"
        try:
            os.link(item, hardlink)
        except OSError:
            pass
        else:
            alias_kinds.append("hardlink")
            guard = self.make_guard(
                source,
                {item: digest, hardlink: digest},
            )
            with self.assertRaises(checkout_guard.CheckoutGuardChanged):
                guard.acquire()

        if "hardlink" in alias_kinds:
            installed_root = self.case_root / "installed-git"
            git_command = installed_root / "cmd"
            git_command.mkdir(parents=True)
            git_executable = git_command / "git.exe"
            git_bytes = b"trusted installed Git fixture"
            git_executable.write_bytes(git_bytes)
            git_digest = _sha256(git_executable)
            git_alias = installed_root / "bin" / "git.exe"
            git_alias.parent.mkdir()
            os.link(git_executable, git_alias)
            with patch(
                "tools.jenkins.pilot.trusted_tools.git_identity",
                return_value={
                    "path": str(git_executable.resolve(strict=True)),
                    "sha256": git_digest,
                },
            ):
                trusted_git = checkout_guard.capture_trusted_git_input()
            self.assertEqual(trusted_git["kind"], "installed-git")
            self.assertEqual(trusted_git["linkCount"], 2)

            trusted_guard = self.make_guard(
                source,
                {git_executable: git_digest},
                watch_roots=[source, installed_root],
                trusted_native_inputs={trusted_git["path"]: trusted_git},
            ).acquire()
            trusted_identity = next(
                identity for identity in trusted_guard.facts()["handleIdentities"]
                if identity["path"] == trusted_git["path"]
            )
            self.assertEqual(trusted_identity["kind"], "installed-git")
            self.assertEqual(trusted_identity["linkCount"], 2)
            with self.assertRaises(OSError):
                git_alias.write_bytes(b"write through second hardlink")
            with self.assertRaises(OSError):
                git_executable.unlink()
            with self.assertRaises(OSError):
                git_executable.rename(git_command / "renamed-git.exe")
            self.assertEqual(git_executable.read_bytes(), git_bytes)
            self.assertEqual(_sha256(git_executable), git_digest)
            self.assertTrue(trusted_guard.assert_stable()["stable"])

            # Windows can remove a different hardlink name while the pinned
            # handle remains open. The guard must detect the physical identity
            # drift and report a completed, joined monitor shutdown.
            git_alias.unlink()
            self.assertEqual(git_executable.read_bytes(), git_bytes)
            self.assertEqual(_sha256(git_executable), git_digest)
            self.assertEqual(git_executable.stat().st_nlink, 1)
            with self.assertRaises(checkout_guard.CheckoutGuardChanged) as link_changed:
                trusted_guard.assert_stable()
            self.assertEqual(link_changed.exception.details["reason"], "input_identity")
            self.assertEqual(link_changed.exception.details["expected"]["linkCount"], 2)
            self.assertEqual(link_changed.exception.details["actual"]["linkCount"], 1)

            evidence = trusted_guard.finish()
            evidence_message = json.dumps(
                evidence, sort_keys=True, ensure_ascii=False, default=str
            )
            self.assertTrue(evidence["complete"], evidence_message)
            self.assertTrue(evidence["cancellationComplete"], evidence_message)
            self.assertTrue(evidence["waitersJoined"], evidence_message)
            self.assertFalse(evidence["stable"], evidence_message)
            self.assertFalse(evidence["sourceNoWrites"], evidence_message)
            self.assertFalse(evidence["membershipStable"], evidence_message)
            self.assertTrue(any(
                change.get("reason") == "input_identity"
                or change.get("details", {}).get("reason") == "input_identity"
                for change in evidence["inputChanges"]
            ), evidence_message)
            for watcher in evidence["watchers"]:
                self.assertTrue(watcher["cancellationComplete"], evidence_message)
                self.assertTrue(watcher["waiterJoined"], evidence_message)
            trusted_guard.close()

            # Restore the captured identity's link count, then add a third
            # name and prove the original trusted facts are stale.
            os.link(git_executable, git_alias)
            self.assertEqual(git_executable.stat().st_nlink, 2)
            third_alias = installed_root / "bin" / "third-git.exe"
            os.link(git_executable, third_alias)
            self.assertEqual(git_executable.stat().st_nlink, 3)
            stale_trust_guard = self.make_guard(
                source,
                {git_executable: git_digest},
                watch_roots=[source, installed_root],
                trusted_native_inputs={trusted_git["path"]: trusted_git},
            )
            with self.assertRaises(checkout_guard.CheckoutGuardChanged) as changed:
                stale_trust_guard.acquire()
            self.assertEqual(changed.exception.details["reason"], "input_identity")

        directory_alias = source / "directory-alias"
        try:
            os.symlink(physical, directory_alias, target_is_directory=True)
        except (OSError, NotImplementedError):
            pass
        else:
            alias_kinds.append("directory symlink")
            alias_input = str(directory_alias / "input.bin")
            guard = self.make_guard(source, {alias_input: digest})
            with self.assertRaises(checkout_guard.CheckoutGuardChanged):
                guard.acquire()

        if not alias_kinds:
            self.skipTest("Filesystem policy does not permit hardlinks or directory symlinks")

    def test_empty_or_malformed_directory_notifications_fail_closed(self):
        with self.assertRaises(checkout_guard.CheckoutGuardChanged) as overflow:
            checkout_guard.parse_notifications(b"")
        self.assertEqual(overflow.exception.details["reason"], "watch_overflow")

        with self.assertRaises(checkout_guard.CheckoutGuardChanged) as malformed:
            checkout_guard.parse_notifications(b"\x00")
        self.assertEqual(
            malformed.exception.details["reason"],
            "watch_invalid_notification",
        )


    def test_volume_root_interest_detects_input_change_and_joins_watchers(self):
        source = self.case_root / "source"
        source.mkdir()
        item = source / "input.bin"
        item.write_bytes(b"volume-root pinned input")
        drive_root = Path(source.drive + os.sep)
        drive_name = str(drive_root.resolve(strict=True))
        source_name = str(source.resolve(strict=True))
        volume_relative = item.relative_to(drive_root).as_posix()

        guard = self.make_guard(
            source,
            {item: _sha256(item)},
            watch_roots=[drive_root, source],
            watch_interests={
                drive_root: [volume_relative],
                # Keep the source-root observer narrow so only the real volume
                # root proves coverage and reports this input change.
                source: ["watch-sentinel/input.bin"],
            },
        )
        try:
            guard.acquire()
        except checkout_guard.CheckoutGuardChanged as error:
            self.fail("volume-root acquire diagnostic: " + json.dumps({
                "type": type(error).__name__,
                "code": getattr(error, "code", None),
                "details": error.details,
                "guardFacts": guard.facts(),
            }, sort_keys=True, ensure_ascii=False, default=str))
        initial = guard.facts()
        volume_mode = next(
            mode for root, mode in initial["watchModes"].items()
            if os.path.normcase(root) == os.path.normcase(drive_name)
        )
        self.assertTrue(volume_mode["recursive"])
        self.assertIsNotNone(volume_mode["armedAt"])
        self.assertIn(drive_name, [
            root for root in initial["watchRoots"]
            if os.path.normcase(root) == os.path.normcase(drive_name)
        ])
        volume_channels = {
            channel["channel"]: channel for channel in volume_mode["channels"]
        }
        self.assertEqual(set(volume_channels), {"data", "metadata"})
        self.assertEqual(
            {name: channel["notifyFilter"] for name, channel in volume_channels.items()},
            {"data": 0x5B, "metadata": 0x104},
        )
        for channel in volume_channels.values():
            self.assertTrue(channel["recursive"])
            self.assertIsNotNone(channel["armedAt"])

        # Activity in sibling names must remain outside the declared input
        # while the volume-root observer covers the deep input path.
        sibling_file = source / "unrelated-volume-sibling.bin"
        sibling_directory = source / "unrelated-volume-sibling-dir"
        sibling_file.write_bytes(b"unrelated sibling")
        sibling_directory.mkdir()
        sibling_directory.rmdir()
        sibling_file.unlink()
        self.assertTrue(guard.assert_stable()["stable"])

        before = item.stat()
        os.utime(item, ns=(before.st_atime_ns, before.st_mtime_ns + 10_000_000_000))
        self.assertTrue(guard.wait_for_change(10.0))
        with self.assertRaises(checkout_guard.CheckoutGuardChanged):
            guard.assert_stable()
        evidence = guard.finish()
        evidence_facts = {
            "complete": evidence.get("complete"),
            "stable": evidence.get("stable"),
            "sourceNoWrites": evidence.get("sourceNoWrites"),
            "membershipStable": evidence.get("membershipStable"),
            "cancellationComplete": evidence.get("cancellationComplete"),
            "waitersJoined": evidence.get("waitersJoined"),
            "inputChanges": evidence.get("inputChanges"),
            "completionErrors": evidence.get("completionErrors"),
            "errors": evidence.get("errors"),
            "events": evidence.get("events"),
            "watchers": [
                {
                    key: watcher.get(key)
                    for key in (
                        "root",
                        "channel",
                        "notifyFilter",
                        "cancellationComplete",
                        "waiterJoined",
                        "errors",
                    )
                }
                for watcher in evidence.get("watchers", [])
            ],
        }
        evidence_message = json.dumps(
            evidence_facts, sort_keys=True, ensure_ascii=False, default=str
        )
        self.assertTrue(evidence["complete"], evidence_message)
        self.assertTrue(evidence["cancellationComplete"], evidence_message)
        self.assertTrue(evidence["waitersJoined"], evidence_message)
        self.assertFalse(evidence["stable"], evidence_message)
        self.assertFalse(evidence["sourceNoWrites"], evidence_message)
        self.assertFalse(evidence["membershipStable"], evidence_message)
        volume_watchers = [
            watcher for watcher in evidence["watchers"]
            if os.path.normcase(watcher["root"]) == os.path.normcase(drive_name)
        ]
        channel_watchers = {watcher["channel"]: watcher for watcher in volume_watchers}
        self.assertEqual(set(channel_watchers), {"data", "metadata"})
        self.assertEqual(
            {name: watcher["notifyFilter"] for name, watcher in channel_watchers.items()},
            {"data": 0x5B, "metadata": 0x104},
        )
        for watcher in volume_watchers:
            self.assertTrue(watcher["cancellationComplete"])
            self.assertTrue(watcher["waiterJoined"])
        data_watcher = channel_watchers["data"]
        self.assertGreater(data_watcher["ignoredEventCount"], 0)
        self.assertTrue(any(
            event["path"].casefold() == volume_relative.casefold()
            and event["channel"] == "data"
            and event["action"] == 3
            for event in data_watcher["events"]
        ))
        with self.assertRaises(OSError):
            item.unlink()
        guard.close()

    def test_declared_input_without_effective_watch_interest_is_rejected(self):
        source = self.case_root / "source"
        declared = source / "namespace" / "input.bin"
        declared.parent.mkdir(parents=True)
        declared.write_bytes(b"declared")

        with self.assertRaises(ValueError):
            self.make_guard(
                source,
                {declared: _sha256(declared)},
                watch_interests={source: ["namespace/other.bin"]},
            )

    def test_shallow_interest_observes_ancestor_and_ignores_unrelated_activity(self):
        source = self.case_root / "source"
        source.mkdir()
        missing = source / "config" / "Cargo.toml"
        guard = self.make_guard(
            source,
            {missing: None},
            watch_interests={source: ["config/Cargo.toml"]},
        ).acquire()
        source_name = str(source.resolve(strict=True))
        source_mode = guard.facts()["watchModes"][source_name]
        self.assertFalse(source_mode["recursive"])
        self.assertIsNotNone(source_mode["armedAt"])

        (source / "unrelated.bin").write_bytes(b"outside declared namespace")
        (source / "config").mkdir()
        self.assertTrue(guard.wait_for_change(10.0))
        evidence = guard.finish()
        watcher = next(item for item in evidence["watchers"] if item["root"] == source_name)
        self.assertTrue(evidence["complete"])
        self.assertGreater(watcher["ignoredEventCount"], 0)
        self.assertTrue(any(event["path"] == "config" for event in watcher["events"]))
        self.assertFalse(any(event["path"] == "unrelated.bin" for event in watcher["events"]))
        self.assertFalse(evidence["membershipStable"])
        self.assertTrue(evidence["cancellationComplete"])
        self.assertTrue(evidence["waitersJoined"])
        guard.close()


if __name__ == "__main__":
    unittest.main()
