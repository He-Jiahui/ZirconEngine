"""Check each guard rule in both directions, the CLI exit codes and the read-only contract."""

from contextlib import redirect_stderr, redirect_stdout
import difflib
import fnmatch
import hashlib
import io
import json
from pathlib import Path
import unittest
from types import SimpleNamespace
from unittest.mock import patch as mock_patch

from change_guard import Context, Grants, evaluate, git_patch, main, normalize_rel, parse_unified_diff, synthesize_addition

CARGO = '[package]\nname = "sample"\nversion = "0.1.0"\n\n[profile.shipping]\ninherits = "release"\npanic = "abort"\n'
CONFIG = (
    "[permissions.zircon-strict.filesystem]\n"
    "'E:/Git/ZirconEngine/Cargo.toml' = \"write\"\n"
    "'E:/Git/ZirconEngine/src' = \"write\"\n"
    "'E:/Git/ZirconEngine/tools' = \"write\"\n"
    "'E:/Git/ZirconEngine/.codex' = \"read\"\n"
)


def modify(rel: str, old: str, new: str) -> str:
    return "".join(difflib.unified_diff(old.splitlines(True), new.splitlines(True), f"a/{rel}", f"b/{rel}"))


class GuardTestCase(unittest.TestCase):
    def setUp(self):
        # Synthetic source data lives only in memory. No fixture checkout,
        # temporary files, cleanup or subprocess is needed for these checks.
        self.root = (Path.cwd() / ".change-guard-memory").resolve()
        self.contents = {}
        self.denied = set()
        self.links = set()
        replacements = {
            "read_bytes": lambda path: self.read_bytes(path),
            "read_text": lambda path, encoding="utf-8", errors="strict", **kwargs: self.read_bytes(path).decode(encoding, errors),
            "is_file": lambda path: path in self.contents,
            "is_dir": lambda path: any(path in item.parents for item in self.contents),
            "exists": lambda path: path in self.contents or any(path in item.parents for item in self.contents),
            "is_symlink": lambda path: path in self.links,
            "glob": lambda path, pattern, **kwargs: self.glob(path, pattern),
            "rglob": lambda path, pattern, **kwargs: self.glob(path, pattern, recursive=True),
        }
        if hasattr(Path, "is_junction"):
            replacements["is_junction"] = lambda path: False
        for name, replacement in replacements.items():
            patcher = mock_patch.object(Path, name, replacement)
            patcher.start()
            self.addCleanup(patcher.stop)
        for name in ("write_text", "write_bytes", "mkdir", "unlink", "rename"):
            patcher = mock_patch.object(Path, name, side_effect=AssertionError("guard attempted a filesystem mutation"))
            patcher.start()
            self.addCleanup(patcher.stop)
        self.write("Cargo.toml", CARGO)
        self.write(".codex/config.toml", CONFIG.replace("E:/Git/ZirconEngine", self.root.as_posix()))
        self.write("src/lib.rs", "mod present;\n")
        self.write("src/present.rs", "pub fn value() -> u32 { 1 }\n")

    def write(self, rel: str, text: str) -> str:
        path = self.root / rel
        self.contents[path] = text.encode("utf-8")
        return text

    def read_bytes(self, path: Path) -> bytes:
        if path in self.denied:
            raise PermissionError(str(path))
        if path not in self.contents:
            raise FileNotFoundError(str(path))
        return self.contents[path]

    def glob(self, path: Path, pattern: str, recursive=False):
        files = sorted(item for item in self.contents if item.is_relative_to(path) and item != path)
        return iter(item for item in files if
                    (recursive or len(item.relative_to(path).parts) == len(pattern.split("/")))
                    and fnmatch.fnmatchcase(item.relative_to(path).as_posix(), pattern))

    def added(self, *rels: str) -> str:
        return "".join(synthesize_addition(rel, (self.root / rel).read_text(encoding="utf-8")) for rel in rels)

    def check(self, patch: str, paths=None) -> list:
        grants = Grants.load(self.root / ".codex/config.toml", "zircon-strict")
        return evaluate(Context(self.root, "patch", parse_unified_diff(patch), grants, paths))

    def rules(self, findings) -> set:
        return {(finding.rule, finding.severity) for finding in findings if finding.allowed_by is None}


class DiffParsingTests(GuardTestCase):
    def test_git_headers_cover_add_modify_delete_and_rename(self):
        patch = (
            "diff --git a/src/old.rs b/src/new.rs\nsimilarity index 90%\nrename from src/old.rs\nrename to src/new.rs\n"
            "diff --git a/src/gone.rs b/src/gone.rs\ndeleted file mode 100644\n--- a/src/gone.rs\n+++ /dev/null\n@@ -1 +0,0 @@\n-x\n"
            "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1,2 @@\n mod present;\n+mod extra;\n"
        )
        changes = parse_unified_diff(patch)
        self.assertEqual(["src/new.rs", "src/gone.rs", "src/lib.rs"], [change.path for change in changes])
        self.assertEqual("src/old.rs", changes[0].old_path)
        self.assertIsNone(changes[1].new_path)
        self.assertEqual({2: "mod extra;"}, changes[2].added)

    def test_truncated_hunk_is_rejected(self):
        with self.assertRaises(ValueError):
            parse_unified_diff("--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,2 +1,3 @@\n mod present;\n+x\n")

    def test_paths_cannot_escape_the_repository(self):
        for path in ("../outside.rs", "/outside.rs", "C:/outside.rs", "safe/C:/outside.rs", r"..\outside.rs"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                parse_unified_diff(f"--- /dev/null\n+++ b/{path}\n@@ -0,0 +1 @@\n+x\n")

    def test_quoted_unicode_and_space_paths(self):
        for path in ('src/空 格.rs', 'src/a"b.rs'):
            with self.subTest(path=path):
                self.assertEqual(path, parse_unified_diff(synthesize_addition(path, "x\n"))[0].path)

    def test_overlapping_hunks_and_missing_headers_are_rejected(self):
        patches = (
            "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-x\n+y\n@@ -1 +1 @@\n-x\n+z\n",
            "diff --git a/src/lib.rs b/src/lib.rs\n@@ -1 +1 @@\n-x\n+y\n",
        )
        for text in patches:
            with self.subTest(text=text), self.assertRaises(ValueError):
                parse_unified_diff(text)


    def test_windows_device_paths_are_rejected_before_reading(self):
        with mock_patch("change_guard.os", SimpleNamespace(name="nt")):
            for path in ("src/NUL", "src/CON.rs", "tools/COM1.txt", "src/lpt9/data.rs"):
                with self.subTest(path=path), self.assertRaises(ValueError):
                    normalize_rel(path)


class ScopeTests(GuardTestCase):
    def test_declared_paths_grants_and_backups(self):
        self.write("docs/note.md", "text\n")
        self.write("src/present.rs.bak", "copy\n")
        findings = self.check(self.added("docs/note.md", "src/present.rs.bak"), ["src"])
        self.assertIn(("scope.unlisted-path", "error"), self.rules(findings))
        self.assertIn(("scope.write-grant", "error"), self.rules(findings))
        self.assertIn(("scope.backup-artifact", "error"), self.rules(findings))

    def test_granted_declared_source_passes(self):
        self.write("src/present.rs", "pub fn value() -> u32 { 2 }\n")
        patch = modify("src/present.rs", "pub fn value() -> u32 { 1 }\n", "pub fn value() -> u32 { 2 }\n")
        self.assertEqual(set(), self.rules(self.check(patch, ["src"])))

    def test_scope_findings_cannot_be_suppressed(self):
        self.write("docs/note.md", "guard-allow(scope.write-grant): reviewed by hand\n")
        self.assertIn(("scope.write-grant", "error"), self.rules(self.check(self.added("docs/note.md"))))

    def test_conflict_marker(self):
        self.write("src/present.rs", "<<<<<<< ours\npub fn value() -> u32 { 2 }\n")
        patch = modify("src/present.rs", "pub fn value() -> u32 { 1 }\n", "<<<<<<< ours\npub fn value() -> u32 { 2 }\n")
        self.assertIn(("diff.conflict-marker", "error"), self.rules(self.check(patch)))


    def test_source_module_named_target_is_allowed_but_cargo_output_is_rejected(self):
        self.write("src/lib.rs", "mod target;\n")
        self.write("src/target/mod.rs", "mod child;\n")
        self.write("src/target/child.rs", "pub fn value() {}\n")
        self.assertNotIn(("scope.backup-artifact", "error"),
                         self.rules(self.check(self.added("src/target/child.rs"))))
        self.write("target/output.txt", "generated\n")
        self.assertIn(("scope.backup-artifact", "error"), self.rules(self.check(self.added("target/output.txt"))))


class MarkerTests(GuardTestCase):
    OLD = "// BUG: [CR-TEST-0001] value is wrong\npub fn value() -> u32 { 1 }\n"

    def test_comment_only_removal_fails(self):
        new = "pub fn value() -> u32 { 1 }\n"
        self.write("src/present.rs", new)
        findings = self.check(modify("src/present.rs", self.OLD, new))
        self.assertIn(("marker.removed-without-fix", "error"), self.rules(findings))

    def test_fix_without_test_warns(self):
        new = "pub fn value() -> u32 { 2 }\n"
        self.write("src/present.rs", new)
        rules = self.rules(self.check(modify("src/present.rs", self.OLD, new)))
        self.assertNotIn(("marker.removed-without-fix", "error"), rules)
        self.assertIn(("marker.closed-without-test", "warn"), rules)

    def test_malformed_marker(self):
        new = "// BUG: value is wrong\npub fn value() -> u32 { 1 }\n"
        self.write("src/present.rs", new)
        patch = modify("src/present.rs", "pub fn value() -> u32 { 1 }\n", new)
        self.assertIn(("marker.format", "error"), self.rules(self.check(patch)))


class ModuleTests(GuardTestCase):
    def test_orphan_file_fails_and_declared_child_passes(self):
        self.write("src/orphan.rs", "pub fn lost() {}\n")
        self.assertIn(("rust.unreachable-module", "error"), self.rules(self.check(self.added("src/orphan.rs"))))
        self.write("src/present.rs", "mod child;\npub fn value() -> u32 { 1 }\n")
        self.write("src/present/child.rs", "pub fn found() {}\n")
        self.assertNotIn(("rust.unreachable-module", "error"), self.rules(self.check(self.added("src/present/child.rs"))))

    def test_path_loaded_file_owns_its_directory(self):
        self.write("src/lib.rs", "mod present;\n#[cfg(test)]\n#[path = \"tests/cases.rs\"]\nmod tests;\n")
        self.write("src/tests/cases.rs", "mod child;\n")
        self.write("src/tests/child.rs", "#[test]\nfn works() {}\n")
        self.assertEqual(set(), self.rules(self.check(self.added("src/tests/child.rs"))))

    def test_removing_mod_line_orphans_file(self):
        self.write("src/lib.rs", "")
        findings = self.check(modify("src/lib.rs", "mod present;\n", ""))
        self.assertIn(("rust.unreachable-module", "error"), self.rules(findings))

    def test_removing_inline_module_orphans_its_child(self):
        old = "mod inline {\n    mod child;\n}\n"
        self.write("src/inline/child.rs", "fn data() {}\n")
        self.write("src/lib.rs", "")
        self.assertIn(("rust.unreachable-module", "error"), self.rules(self.check(modify("src/lib.rs", old, ""))))

    def test_removing_path_module_with_unchanged_attribute_orphans_child(self):
        old = '#[path = "support/cases.rs"]\nmod hidden;\nconst VALUE: u32 = 0;\nmod present;\n'
        new = '#[path = "support/cases.rs"]\nconst VALUE: u32 = 0;\nmod present;\n'
        self.write("src/lib.rs", new)
        self.write("src/support/cases.rs", "fn data() {}\n")
        findings = self.check(modify("src/lib.rs", old, new))
        self.assertTrue(any(item.rule == "rust.unreachable-module" and "src/support/cases.rs" in item.message for item in findings))

    def test_deleting_a_declaring_file_reports_the_child_left_behind(self):
        self.write("src/present/child.rs", "fn data() {}\n")
        self.contents.pop(self.root / "src/present.rs")
        text = "--- a/src/present.rs\n+++ /dev/null\n@@ -1,2 +0,0 @@\n-mod child;\n-pub fn value() {}\n"
        self.assertIn(("rust.unreachable-module", "error"), self.rules(self.check(text)))

    def test_file_named_only_by_path_string_is_a_warning(self):
        self.write("src/present.rs", 'pub const FIXTURE: &str = "src/fixtures/sample.rs";\n')
        self.write("src/fixtures/sample.rs", "fn data() {}\n")
        rules = self.rules(self.check(self.added("src/fixtures/sample.rs")))
        self.assertIn(("rust.uncompiled-source", "warn"), rules)
        self.assertNotIn(("rust.unreachable-module", "error"), rules)

    def test_disconnected_parent_does_not_make_child_reachable(self):
        self.write("src/lost.rs", "mod child;\n")
        self.write("src/lost/child.rs", "fn data() {}\n")
        self.assertIn(("rust.unreachable-module", "error"),
                      self.rules(self.check(self.added("src/lost/child.rs"))))

    def test_disconnected_path_declaration_does_not_reach_child(self):
        self.write("src/lost.rs", '#[path = "hidden.rs"]\nmod hidden;\n')
        self.write("src/hidden.rs", "fn data() {}\n")
        self.assertIn(("rust.unreachable-module", "error"),
                      self.rules(self.check(self.added("src/hidden.rs"))))

    def test_comments_and_raw_strings_cannot_declare_modules(self):
        self.write("src/lib.rs", 'mod present;\nconst TEXT: &str = r##"\nmod hidden;\n"##;\n'
                                '/* outer /* nested */ mod hidden; */\n')
        self.write("src/hidden.rs", "fn data() {}\n")
        self.assertIn(("rust.unreachable-module", "error"),
                      self.rules(self.check(self.added("src/hidden.rs"))))

    def test_inline_and_multiline_modules_are_reachable(self):
        self.write("src/lib.rs", "mod inline {\n    mod\n    child;\n}\n")
        self.write("src/inline/child.rs", "fn data() {}\n")
        self.assertNotIn(("rust.unreachable-module", "error"),
                         self.rules(self.check(self.added("src/inline/child.rs"))))

    def test_disabled_auto_tests_need_an_explicit_target(self):
        cargo = CARGO.replace('version = "0.1.0"', 'version = "0.1.0"\nedition = "2021"\nautotests = false')
        self.write("Cargo.toml", cargo)
        self.write("tests/hidden.rs", "#[test]\nfn works() {}\n")
        self.assertIn(("rust.unreachable-module", "error"), self.rules(self.check(self.added("tests/hidden.rs"))))
        self.write("Cargo.toml", cargo + '\n[[test]]\nname = "hidden"\npath = "tests/hidden.rs"\n')
        self.assertNotIn(("rust.unreachable-module", "error"), self.rules(self.check(self.added("tests/hidden.rs"))))

    def test_dependency_path_is_not_a_crate_root(self):
        self.write("Cargo.toml", CARGO + '\n[dependencies.fake]\npath = "src/hidden.rs"\n')
        self.write("src/hidden.rs", "fn data() {}\n")
        self.assertIn(("rust.unreachable-module", "error"), self.rules(self.check(self.added("src/hidden.rs"))))

    def test_rename_out_of_a_target_directory_is_checked(self):
        self.write("src/renamed.rs", "#[test]\nfn works() {}\n")
        text = "diff --git a/tests/old.rs b/src/renamed.rs\nsimilarity index 100%\nrename from tests/old.rs\nrename to src/renamed.rs\n"
        self.assertIn(("rust.unreachable-module", "error"), self.rules(self.check(text)))


class RustSafetyTests(GuardTestCase):
    def change(self, body: str):
        self.write("src/present.rs", body)
        return self.rules(self.check(modify("src/present.rs", "pub fn value() -> u32 { 1 }\n", body)))

    def test_unsafe_needs_safety_comment(self):
        self.assertIn(("rust.unsafe-without-safety", "error"),
                      self.change("pub fn value() -> u32 {\n    unsafe { core::hint::unreachable_unchecked() }\n}\n"))
        self.assertNotIn(("rust.unsafe-without-safety", "error"),
                         self.change("pub fn value() -> u32 {\n    // SAFETY: never reached by contract.\n"
                                     "    unsafe { core::hint::unreachable_unchecked() }\n}\n"))

    def test_safe_public_fn_with_raw_pointer(self):
        self.assertIn(("rust.safe-fn-raw-pointer", "error"),
                      self.change("pub fn value(ptr: *const u32) -> u32 {\n    0\n}\n"))

    def test_pointer_added_to_an_existing_multiline_signature(self):
        old = "pub fn value(\n    number: u32,\n) -> u32 { number }\n"
        new = "pub fn value(\n    number: u32,\n    pointer: *mut u32,\n) -> u32 { number }\n"
        self.write("src/present.rs", new)
        self.assertIn(("rust.safe-fn-raw-pointer", "error"),
                      self.rules(self.check(modify("src/present.rs", old, new))))

    def test_raw_pointer_exemptions(self):
        self.assertEqual(set(), self.change("pub fn value(callback: unsafe fn(*mut u8)) -> u32 {\n    0\n}\n"))
        self.assertEqual(set(), self.change("pub fn value(callback: unsafe fn() -> *mut u8) -> u32 {\n    0\n}\n"))
        self.assertEqual(set(), self.change("pub(crate) fn value(ptr: *const u32) -> u32 {\n    0\n}\n"))
        self.assertEqual({("rust.unsafe-fn-safety-doc", "warn")},
                         self.change("pub unsafe fn value(ptr: *const u32) -> u32 {\n    0\n}\n"))

    def test_catch_unwind_under_abort_profile(self):
        body = "pub fn value() -> u32 {\n    std::panic::catch_unwind(|| 1).unwrap_or(0)\n}\n"
        self.assertIn(("rust.catch-unwind-under-abort", "warn"), self.change(body))

    def test_serde_bypass(self):
        body = (
            "#[derive(serde::Deserialize)]\npub struct Limit(u32);\n\n"
            "impl Limit {\n    pub fn new(value: u32) -> Option<Self> {\n        (value > 0).then_some(Self(value))\n    }\n}\n"
        )
        self.assertIn(("rust.serde-bypass", "warn"), self.change(body))
        safe = body.replace("pub struct", "#[serde(try_from = \"u32\")]\npub struct")
        self.assertNotIn(("rust.serde-bypass", "warn"), self.change(safe))
        unchecked = body.replace("pub struct", '#[serde(from = "u32")]\npub struct')
        self.assertIn(("rust.serde-bypass", "warn"), self.change(unchecked))

    def test_unsafe_text_in_raw_strings_is_not_code(self):
        body = 'pub const TEXT: &str = r#"\nunsafe { something() }\n"#;\n'
        self.assertNotIn(("rust.unsafe-without-safety", "error"), self.change(body))


class FilesystemTests(GuardTestCase):
    def test_delete_requires_visible_containment(self):
        self.write("tools/clean.py", "import shutil\n\ndef clean(path):\n    shutil.rmtree(path)\n")
        self.assertIn(("fs.unbounded-delete", "error"), self.rules(self.check(self.added("tools/clean.py"))))
        self.write("tools/clean.py", "import shutil\n\ndef clean(root, path):\n"
                                     "    if not path.resolve().is_relative_to(root.resolve()):\n"
                                     "        raise ValueError(path)\n    shutil.rmtree(path)\n")
        self.assertEqual(set(), self.rules(self.check(self.added("tools/clean.py"))))

    def test_allow_comment_suppresses_reviewed_delete(self):
        self.write("tools/clean.py", "import shutil\n\ndef clean(path):\n"
                                     "    # guard-allow(fs.unbounded-delete): path is a TemporaryDirectory we created\n"
                                     "    shutil.rmtree(path)\n")
        findings = self.check(self.added("tools/clean.py"))
        self.assertEqual(set(), self.rules(findings))
        self.assertEqual(["fs.unbounded-delete"], [finding.rule for finding in findings if finding.allowed_by])

    def test_suppression_and_containment_strings_are_not_comments_or_checks(self):
        self.write("tools/clean.py", 'reason = "guard-allow(fs.unbounded-delete): reviewed this path"\n'
                                     '# path.is_relative_to(root) is only a comment\nshutil.rmtree(path)\n')
        self.assertIn(("fs.unbounded-delete", "error"), self.rules(self.check(self.added("tools/clean.py"))))

    def test_storage_roots(self):
        self.write("tools/paths.py", 'OUT = home / "x" / "ZirconBuilds"\nCARGO_TARGET_DIR = "target/debug"\n')
        findings = [f for f in self.check(self.added("tools/paths.py")) if f.rule == "storage.output-root"]
        self.assertEqual([1, 2], [finding.line for finding in findings])
        self.write("tools/paths.py", 'from x import ZirconBuildStageExecutor\nCARGO_TARGET_DIR = "D:/cargo-targets/zircon"\n'
                                     'FORBIDDEN = ["D:/targets"]  # reject legacy roots\n')
        self.assertEqual(set(), self.rules(self.check(self.added("tools/paths.py"))))

    def test_root_prefix_traversal_alias_and_unrelated_good_string_do_not_pass(self):
        for value in ("E:/cargo-targets-other/x", "E:/cargo-targets/../outside", "/mnt/e/cargo-targets/x",
                      "relative/out", "C:/temp", "E:/nested/cargo-targets/x"):
            with self.subTest(value=value):
                self.write("tools/paths.py", f'CARGO_TARGET_DIR = "{value}"\n')
                self.assertIn(("storage.output-root", "error"), self.rules(self.check(self.added("tools/paths.py"))))
        self.write("tools/paths.py", 'CARGO_TARGET_DIR = "C:/bad"  # reject this; use "E:/cargo-targets/good"\n')
        self.assertIn(("storage.output-root", "error"), self.rules(self.check(self.added("tools/paths.py"))))


    def test_output_dictionary_values_and_variables(self):
        self.write("tools/paths.py", 'os.environ["CARGO_TARGET_DIR"] = "C:/bad"\n')
        self.assertIn(("storage.output-root", "error"), self.rules(self.check(self.added("tools/paths.py"))))
        self.write("tools/paths.py", "CARGO_TARGET_DIR = target_dir\n")
        self.assertNotIn(("storage.output-root", "error"), self.rules(self.check(self.added("tools/paths.py"))))
        self.write("tools/paths.json", '{"CARGO_TARGET_DIR": "C:/bad"}\n')
        self.assertIn(("storage.output-root", "error"), self.rules(self.check(self.added("tools/paths.json"))))

    def test_selector_names_inside_patterns_and_descriptions_are_not_assignments(self):
        self.write("tools/paths.py", 'PATTERN = re.compile(r"CARGO_TARGET_DIR|--target-dir")\n'
                                     'HELP = "set CARGO_TARGET_DIR to an approved path"\n')
        self.assertNotIn(("storage.output-root", "error"), self.rules(self.check(self.added("tools/paths.py"))))


class TestIntegrityTests(GuardTestCase):
    def test_disabled_and_removed_tests_warn(self):
        old = "#[test]\nfn one() {}\n#[test]\nfn two() {}\n"
        new = "#[test]\n#[ignore]\nfn one() {}\n"
        self.write("src/present.rs", new)
        findings = [f for f in self.check(modify("src/present.rs", old, new)) if f.rule == "test.weakened"]
        self.assertEqual(2, len(findings))


class CommandLineTests(GuardTestCase):
    def run_main(self, *args: str) -> tuple[int, str]:
        out, err = io.StringIO(), io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            code = main(["--repo-root", str(self.root), "--json", *args])
        return code, out.getvalue()

    def files(self) -> dict:
        return {p.relative_to(self.root).as_posix(): data for p, data in self.contents.items()}

    def test_exit_codes_receipt_and_read_only(self):
        self.write("src/orphan.rs", "pub fn lost() {}\n")
        self.write("change.patch", self.added("src/orphan.rs"))
        before = self.files()
        code, output = self.run_main("patch", "--patch", str(self.root / "change.patch"), "--path", "src")
        self.assertEqual(before, self.files())
        receipt = json.loads(output)
        self.assertEqual((1, "failed"), (code, receipt["status"]))
        self.assertEqual(64, len(receipt["input"]["patchDigest"]))
        self.assertEqual(["src/orphan.rs"], receipt["input"]["files"])

        self.write("src/present.rs", "pub fn value() -> u32 { 2 }\n")
        self.write("change.patch", modify("src/present.rs", "pub fn value() -> u32 { 1 }\n", "pub fn value() -> u32 { 2 }\n"))
        self.assertEqual(0, self.run_main("patch", "--patch", str(self.root / "change.patch"), "--path", "src")[0])

        self.write("change.patch", "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,2 +1,3 @@\n+x\n")
        self.assertEqual(2, self.run_main("patch", "--patch", str(self.root / "change.patch"))[0])
        self.assertEqual(2, self.run_main("scan", "../outside")[0])

    def test_scan_reports_whole_files(self):
        self.write("src/orphan.rs", "pub fn lost() {}\n")
        code, output = self.run_main("scan", "src")
        self.assertEqual(1, code)
        self.assertIn("rust.unreachable-module", {finding["rule"] for finding in json.loads(output)["findings"]})

    def test_missing_malformed_and_unknown_profile_config_are_input_errors(self):
        self.write("change.patch", self.added("src/present.rs"))
        config = self.root / ".codex/config.toml"
        original = self.contents.pop(config)
        args = ("patch", "--patch", str(self.root / "change.patch"), "--path", "src")
        self.assertEqual(2, self.run_main(*args)[0])
        self.contents[config] = b"not valid TOML"
        self.assertEqual(2, self.run_main(*args)[0])
        self.contents[config] = original.replace(b"zircon-strict", b"another-profile")
        self.assertEqual(2, self.run_main(*args)[0])

    def test_grants_use_checkout_identity_and_specific_read_denials(self):
        self.write(".codex/config.toml",
                   "[permissions.zircon-strict.filesystem]\n"
                   f"'{self.root.as_posix()}/src' = \"write\"\n"
                   f"'{self.root.as_posix()}/src/private' = \"read\"\n")
        grants = Grants.load(self.root / ".codex/config.toml", "zircon-strict", self.root)
        self.assertEqual("write", grants.mode("src/present.rs"))
        self.assertEqual("read", grants.mode("src/private/value.rs"))
        self.assertEqual("none", grants.mode("docs/note.md"))

    def test_allowlist_is_required_and_contributor_mode_is_explicit(self):
        self.write("change.patch", self.added("src/present.rs"))
        args = ("patch", "--patch", str(self.root / "change.patch"))
        self.assertEqual(2, self.run_main(*args)[0])
        self.contents.pop(self.root / ".codex/config.toml")
        code, output = self.run_main(*args, "--contributor")
        self.assertEqual(0, code)
        self.assertFalse(json.loads(output)["input"]["writeGrantCheck"])
        self.assertEqual(2, self.run_main("--config", "missing", *args, "--contributor")[0])

    def test_mismatched_or_missing_worktree_cannot_pass(self):
        self.write("change.patch", synthesize_addition("src/present.rs", "pub fn changed() {}\n"))
        args = ("patch", "--patch", str(self.root / "change.patch"), "--path", "src")
        code, output = self.run_main(*args)
        self.assertEqual(1, code)
        self.assertIn("input.worktree-mismatch", {item["rule"] for item in json.loads(output)["findings"]})
        self.contents.pop(self.root / "src/present.rs")
        self.assertEqual(1, self.run_main(*args)[0])

    def test_digest_matches_exact_crlf_patch_bytes_and_stdin_stays_open(self):
        patch = self.added("src/present.rs").replace("\n", "\r\n").encode("utf-8")
        self.contents[self.root / "change.patch"] = patch
        code, output = self.run_main("patch", "--patch", str(self.root / "change.patch"), "--path", "src")
        self.assertEqual(0, code)
        self.assertEqual(hashlib.sha256(patch).hexdigest(), json.loads(output)["input"]["patchDigest"])
        stdin = io.TextIOWrapper(io.BytesIO(patch), encoding="utf-8")
        with mock_patch("sys.stdin", stdin):
            code, output = self.run_main("patch", "--patch", "-", "--path", "src")
        self.assertEqual(0, code)
        self.assertFalse(stdin.closed)
        self.assertEqual(hashlib.sha256(patch).hexdigest(), json.loads(output)["input"]["patchDigest"])

    def test_scan_read_errors_and_aliases_are_not_silently_skipped(self):
        self.denied.add(self.root / "src/present.rs")
        self.assertEqual(2, self.run_main("scan", "src")[0])
        self.denied.clear()
        self.links.add(self.root / "src")
        self.assertEqual(2, self.run_main("scan", "src")[0])

    def test_explicit_ignored_source_is_included_and_hidden_directory_input_fails(self):
        def fake_git(root, *args):
            if args[0] == "rev-parse":
                return "a" * 40 + "\n"
            if args[0] == "ls-files" and "--ignored" in args:
                return "src/present.rs\0"
            return ""
        with mock_patch("change_guard.git", side_effect=fake_git):
            text, base = git_patch(self.root, ["src/present.rs"], "HEAD")
            self.assertEqual(["src/present.rs"], [item.path for item in parse_unified_diff(text)])
            self.assertEqual("a" * 40, base)
            with self.assertRaises(ValueError):
                git_patch(self.root, ["src"], "HEAD")

    def test_cli_default_full_access_checks_actual_checkout_and_records_grants(self):
        self.write(".codex/config.toml", 'default_permissions = ":danger-full-access"\n')
        self.write(".codex/note.md", "note\n")
        self.write("change.patch", self.added(".codex/note.md"))
        before = self.files()
        code, output = self.run_main("patch", "--patch", str(self.root / "change.patch"),
                                     "--path", ".codex/note.md")
        self.assertEqual((0, "passed"), (code, json.loads(output)["status"]))
        self.assertTrue(json.loads(output)["input"]["writeGrantCheck"])
        self.assertEqual([".codex/note.md"], json.loads(output)["input"]["files"])
        self.assertEqual(before, self.files())
        grants = Grants.load(self.root / ".codex/config.toml", root=self.root)
        self.assertEqual(self.root.as_posix(), grants.prefix)
        self.assertEqual("write", grants.mode(".codex/note.md"))
        with self.assertRaises(ValueError):
            grants.mode("../outside.md")

    def test_cli_explicit_strict_overrides_full_access_and_keeps_read_denial(self):
        original = self.contents[self.root / ".codex/config.toml"].decode()
        self.write(".codex/config.toml", 'default_permissions = ":danger-full-access"\n' + original)
        self.write(".codex/note.md", "note\n")
        self.write("change.patch", self.added(".codex/note.md"))
        code, output = self.run_main("--profile", "zircon-strict", "patch",
                                     "--patch", str(self.root / "change.patch"), "--path", ".codex")
        self.assertEqual(1, code)
        self.assertTrue(json.loads(output)["input"]["writeGrantCheck"])
        self.assertIn("scope.write-grant", {f["rule"] for f in json.loads(output)["findings"]})

    def test_cli_named_default_keeps_longest_prefix_read_denial(self):
        self.write(".codex/config.toml",
                   'default_permissions = "review.local"\n[permissions."review.local".filesystem]\n'
                   f"'{self.root.as_posix()}' = \"write\"\n"
                   f"'{self.root.as_posix()}/.codex' = \"read\"\n")
        self.write(".codex/note.md", "note\n")
        self.write("change.patch", self.added(".codex/note.md"))
        code, output = self.run_main("patch", "--patch", str(self.root / "change.patch"),
                                     "--path", ".codex")
        self.assertEqual(1, code)
        self.assertIn("scope.write-grant", {f["rule"] for f in json.loads(output)["findings"]})
        self.write("change.patch", self.added("src/present.rs"))
        self.assertEqual(0, self.run_main("patch", "--patch", str(self.root / "change.patch"),
                                        "--path", "src")[0])

    def test_cli_unknown_or_malformed_default_selection_is_an_input_error(self):
        self.write("change.patch", self.added("src/present.rs"))
        original = self.contents[self.root / ".codex/config.toml"].decode()
        for selection in ('"missing"', '":read-only"', '":danger-full-access/extra"',
                          '""', '" zircon-strict"', '"zircon-strict "', '"../zircon-strict"',
                          'true', '17', '[]', '{}'):
            with self.subTest(selection=selection):
                self.write(".codex/config.toml", f"default_permissions = {selection}\n" + original)
                self.assertEqual(2, self.run_main("patch", "--patch", str(self.root / "change.patch"),
                                                "--path", "src")[0])

    def test_cli_full_access_still_requires_and_enforces_allowlist(self):
        self.write(".codex/config.toml", 'default_permissions = ":danger-full-access"\n')
        self.write(".codex/note.md", "note\n")
        self.write("change.patch", self.added(".codex/note.md"))
        args = ("patch", "--patch", str(self.root / "change.patch"))
        self.assertEqual(2, self.run_main(*args)[0])
        code, output = self.run_main(*args, "--path", "src")
        self.assertEqual(1, code)
        receipt = json.loads(output)
        rules = {f["rule"] for f in receipt["findings"]}
        self.assertIn("scope.unlisted-path", rules)
        self.assertNotIn("scope.write-grant", rules)
        self.assertTrue(receipt["input"]["writeGrantCheck"])

    def test_cli_explicit_full_access_selection_is_exact_and_unknown_profiles_fail(self):
        self.write(".codex/note.md", "note\n")
        self.write("change.patch", self.added(".codex/note.md"))
        args = ("patch", "--patch", str(self.root / "change.patch"), "--path", ".codex")
        code, output = self.run_main("--profile", ":danger-full-access", *args)
        self.assertEqual(0, code)
        self.assertTrue(json.loads(output)["input"]["writeGrantCheck"])
        for selection in ("missing", ":danger-full-access-extra", ":DANGER-FULL-ACCESS", ""):
            with self.subTest(selection=selection):
                self.assertEqual(2, self.run_main("--profile", selection, *args)[0])

    def test_cli_explicit_profile_does_not_consume_invalid_unused_default(self):
        original = self.contents[self.root / ".codex/config.toml"].decode()
        self.write(".codex/config.toml", 'default_permissions = false\n' + original)
        self.write("change.patch", self.added("src/present.rs"))
        self.assertEqual(0, self.run_main("--profile", "zircon-strict", "patch",
                                        "--patch", str(self.root / "change.patch"), "--path", "src")[0])



if __name__ == "__main__":
    unittest.main()
