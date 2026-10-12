from __future__ import annotations

import unittest

from tools.jenkins.workflow.classification import (
    classify_rust_source,
    rust_lexical_evidence,
    rust_non_doc_tokens,
)


class RustLexicalClassificationTests(unittest.TestCase):
    def test_raw_strings_byte_strings_and_lifetimes_hide_comment_markers(self) -> None:
        text = (
            'r#"before"//one"#; br##"before /* nested */"##; '
            'cr#"before // text"#; fn f<\'a>(x: &\'a str) { let c = \' \'; }'
        )
        evidence = rust_lexical_evidence(text)
        self.assertEqual((), evidence["ordinary_comment_spans"])
        self.assertIn(r'r#"before"//one"#', evidence["tokens"])
        self.assertIn("'a", evidence["tokens"])
        self.assertIn("' '", evidence["tokens"])
        self.assertEqual(evidence["tokens"], rust_non_doc_tokens(text))

    def test_nested_comments_and_true_doc_comments(self) -> None:
        text = "// ordinary\n//// ordinary\n/// outer\n//! inner\n/* outer /* nested */ end */\n/** outer */\n/**/\n/*** ordinary */\n"
        evidence = rust_lexical_evidence(text)
        self.assertEqual(
            ("/// outer", "//! inner", "/** outer */"),
            tuple(text[start:end] for start, end in evidence["doc_comment_spans"]),
        )
        self.assertEqual(5, len(evidence["ordinary_comment_spans"]))
        self.assertEqual(3, len(evidence["doc_comment_spans"]))

    def test_pattern_whitespace_and_spans_are_preserved(self) -> None:
        evidence = rust_lexical_evidence("a\u0085+\u200eb")
        self.assertEqual(("a", "+", "b"), evidence["tokens"])
        self.assertEqual(
            tuple((start, end) for start, end in evidence["token_spans"]),
            ((0, 1), (2, 3), (4, 5)),
        )

    def test_actual_macro_calls_are_detected_outside_strings(self) -> None:
        text = 'let s = "line!() include!(\\"ignored\\")"; line!(); column!(); file!(); include!("x"); include_str!("x"); include_bytes!("x"); #[cfg(feature = "x")] fn f() {}'
        evidence = rust_lexical_evidence(text)
        self.assertEqual(("line!", "column!", "file!"), evidence["line_sensitive_macros"])
        self.assertEqual(("include!", "include_str!", "include_bytes!"), evidence["source_embedding_macros"])
        self.assertNotIn("cfg!", evidence["conditional_build_sensitive_macros"])
        self.assertIn("cfg", evidence["conditional_build_sensitive_attributes"])

    def test_rustc_directive_comment_markers_are_evidence(self) -> None:
        evidence = rust_lexical_evidence("// rustc-link-lib=foo\nfn main() {}\n")
        self.assertTrue(evidence["rustc_directive_comments"])
        self.assertEqual(("rustc-link-lib",), evidence["rustc_directive_comment_markers"])
        self.assertEqual("semantic", classify_rust_source("// rustc-link-lib=foo\n")["kind"])

    def test_bom_crlf_and_space_character_are_legal_input(self) -> None:
        evidence = rust_lexical_evidence("\ufefflet c = ' '\r\nlet raw = r\"line\r\ntext\";\r\n")
        self.assertIn("' '", evidence["tokens"])
        self.assertIn('r"line\r\ntext"', evidence["tokens"])

    def test_selected_code_is_semantic_but_comment_only_lines_are_not(self) -> None:
        source = "fn main() {}\n// note\n"
        self.assertEqual("comments_only", classify_rust_source(source, changed_lines=[2])["kind"])
        self.assertEqual("semantic", classify_rust_source(source, changed_lines=[1])["kind"])
        self.assertEqual("semantic", classify_rust_source("/// docs\n", changed_lines=[1])["kind"])

    def test_ambiguous_or_unterminated_input_fails_closed(self) -> None:
        with self.assertRaises(ValueError):
            rust_lexical_evidence("/* unfinished")
        with self.assertRaises(ValueError):
            rust_lexical_evidence("'")
        self.assertEqual("blocked", classify_rust_source("/* unfinished")["kind"])


if __name__ == "__main__":
    unittest.main()
