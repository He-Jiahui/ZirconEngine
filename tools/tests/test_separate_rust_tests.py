"""Regression checks for preserving Rust tests during physical extraction."""
from __future__ import annotations

from pathlib import Path
import unittest

from tools.audits import separate_rust_tests as audit


class RustTestSeparationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.original_files = audit.KNOWN_FILES
        audit.KNOWN_FILES = set()

    def tearDown(self) -> None:
        audit.KNOWN_FILES = self.original_files

    def test_comments_and_raw_literals_do_not_create_tests(self) -> None:
        source = '''
/* outer /* #[test] */ #[cfg(test)] mod tests {} */
const FIXTURE: &str = r##"#[cfg(test)] mod tests { #[test] fn fake() {} }"##;
'''
        self.assertEqual(audit.test_count(source), 0)
        self.assertEqual(audit.modules(source), [])

    def test_multiline_fixture_bytes_survive_deindentation(self) -> None:
        source = '''
    #[test]
    fn fixture() {
        let fixture = r#"first
            second } #[test]
        third"#;
        assert!(fixture.contains("second"));
    }
'''
        result = audit.deindent(source)
        self.assertTrue(result.startswith('#[test]\nfn fixture()'))
        self.assertIn('first\n            second } #[test]\n        third', result)
        self.assertEqual(audit.token_hash(source), audit.token_hash(result))

    def test_attributes_visibility_and_nested_module_scope_survive(self) -> None:
        source = 'mod owner { #[cfg(all(test, unix))] pub(super) mod checks { #[test] fn a() {} } }'
        module = audit.modules(source)[1]
        self.assertTrue(module.test_only)
        self.assertEqual(module.parents, ('owner',))
        self.assertIn('pub(super)', source[module.keyword:module.delimiter])

    def test_production_conditions_are_not_test_only(self) -> None:
        for condition in ('not(test)', 'any(test, unix)', 'all(feature = "test", unix)',
                          'all(unix, any(test, feature = "fixture"))'):
            source = f'#[cfg({condition})] mod owner {{}}'
            self.assertFalse(audit.modules(source)[0].test_only)

    def test_nested_conditions_that_require_tests_are_detected(self) -> None:
        for condition in ('all(any(test, all(test, unix)), windows)', 'not(not(test))',
                          'all(\n test,\n unix,\n)'):
            source = f'#[cfg({condition})] mod cases {{}}'
            self.assertTrue(audit.modules(source)[0].test_only)

    def test_extracted_inline_module_keeps_child_owner(self) -> None:
        old = audit.ROOT / 'synthetic/src/owner.rs'
        new = audit.ROOT / 'synthetic/src/tests/owner.rs'
        child = audit.ROOT / 'synthetic/src/owner/tests/child.rs'
        audit.KNOWN_FILES = {child}
        result = audit.rewrite_references('mod child;\n', old, new, {}, ('tests',))
        self.assertEqual(result, '#[path = "../owner/tests/child.rs"]\nmod child;\n')

    def test_relative_fixture_and_moved_module_paths_follow_files(self) -> None:
        old = audit.ROOT / 'synthetic/src/owner.rs'
        new = audit.ROOT / 'synthetic/src/tests/owner.rs'
        child = audit.ROOT / 'synthetic/src/owner/checks.rs'
        moved = audit.ROOT / 'synthetic/src/owner/tests/checks.rs'
        audit.KNOWN_FILES = {child}
        source = 'mod checks;\nconst F: &str = include_str!("fixture.json");\n'
        result = audit.rewrite_references(source, old, new, {child: moved})
        self.assertIn('#[path = "../owner/tests/checks.rs"]', result)
        self.assertIn('include_str!("../fixture.json")', result)

    def test_only_package_build_scripts_use_the_crate_root(self) -> None:
        self.assertEqual(audit.module_base(audit.ROOT / 'zircon_runtime/build.rs'),
                         audit.ROOT / 'zircon_runtime')
        source = audit.ROOT / 'synthetic/src/subsystem/build.rs'
        self.assertEqual(audit.module_base(source), source.with_suffix(''))
        destination = audit.test_destination(audit.ROOT / 'zircon_runtime/build.rs')
        self.assertEqual(destination, audit.ROOT / 'zircon_runtime/tests/unit/build.rs')

    def test_preflight_preserves_test_count(self) -> None:
        path = audit.ROOT / 'synthetic/src/owner.rs'
        source = 'fn private() -> u8 { 3 }\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn works() { assert_eq!(private(), 3); }\n}\n'
        report = audit.migrate([path], {path: source}, Path('unused-receipt.json'), False)
        self.assertEqual(report['inline_modules'], 1)
        self.assertEqual(report['test_attributes_before'], 1)
        self.assertEqual(report['test_attributes_after'], 1)
        self.assertEqual(report['extractions'][0]['test_file'], 'synthetic/src/tests/owner.rs')

    def test_build_script_remains_a_cargo_entry_when_tests_import_it(self) -> None:
        build = audit.ROOT / 'zircon_runtime_interface/build.rs'
        build_source = 'fn main() {}\n#[cfg(test)] mod tests { #[test] fn works() {} }\n'
        for name, declaration in (
            ('owner.rs', '#[cfg(test)] #[path = "../../zircon_runtime_interface/build.rs"] mod build_script;'),
            ('owner_tests.rs', '#[path = "../../zircon_runtime_interface/build.rs"] mod build_script;'),
        ):
            with self.subTest(name=name):
                owner = audit.ROOT / 'synthetic/src' / name
                audit.KNOWN_FILES = {owner, build}
                inline, moving, unresolved = audit.inventory(
                    [owner, build], {owner: declaration, build: build_source}
                )
                self.assertNotIn(build, moving)
                self.assertEqual([(build, 'tests')], [(p, m.name) for p, m in inline])
                self.assertEqual([], unresolved)

    def test_default_children_keep_their_own_nested_module_directory(self) -> None:
        old = audit.ROOT / 'synthetic/src/tests.rs'
        new = audit.ROOT / 'synthetic/src/tests/cases.rs'
        child = audit.ROOT / 'synthetic/src/tests/child.rs'
        audit.KNOWN_FILES = {child}
        result = audit.rewrite_references('mod child;\n', old, new, {})
        self.assertEqual(result, 'mod child;\n')


if __name__ == '__main__':
    unittest.main()
