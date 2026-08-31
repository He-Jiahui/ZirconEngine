from __future__ import annotations

import hashlib
import unittest

from tools.zircon_export.source_template_contents_artifact import (
    contents_artifact_handoff_diagnostics,
)


class _EncodeCountingStr(str):
    def __init__(self, value: str) -> None:
        super().__init__()
        self.encode_calls = 0

    def encode(self, encoding: str = "utf-8", errors: str = "strict") -> bytes:
        self.encode_calls += 1
        return super().encode(encoding, errors)


class Tooling03SourceTemplateContentBytesPerformanceContractTests(
    unittest.TestCase
):
    def test_matching_content_is_encoded_once(self) -> None:
        source = "fn main() { println!(\"zircon\"); }\n"
        encoded = source.encode("utf-8")
        contents = _EncodeCountingStr(source)

        diagnostics = contents_artifact_handoff_diagnostics(
            [
                {
                    "path": "src/main.rs",
                    "purpose": "entrypoint",
                    "byte_length": len(encoded),
                    "content_digest": hashlib.sha256(encoded).hexdigest(),
                }
            ],
            [
                {
                    "path": "src/main.rs",
                    "purpose": "entrypoint",
                    "contents": contents,
                }
            ],
        )

        self.assertEqual([], diagnostics)
        self.assertEqual(1, contents.encode_calls)

    def test_length_mismatch_reuses_bytes_for_digest_check(self) -> None:
        source = "fn main() {}\n"
        encoded = source.encode("utf-8")
        contents = _EncodeCountingStr(source)

        diagnostics = contents_artifact_handoff_diagnostics(
            [
                {
                    "path": "src/main.rs",
                    "purpose": "entrypoint",
                    "byte_length": len(encoded) + 1,
                    "content_digest": hashlib.sha256(encoded).hexdigest(),
                }
            ],
            [
                {
                    "path": "src/main.rs",
                    "purpose": "entrypoint",
                    "contents": contents,
                }
            ],
        )

        self.assertEqual(1, len(diagnostics))
        self.assertIn("byte length", diagnostics[0])
        self.assertEqual(1, contents.encode_calls)


if __name__ == "__main__":
    unittest.main()
