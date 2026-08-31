import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_PATH = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "host_contract"
    / "chrome_command_stream"
    / "stream"
    / "image_resources.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(signature + r"[^\{]*\{", source)
    if match is None:
        raise AssertionError(f"missing Rust function: {signature}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated Rust function: {signature}")
    return source[match.end() : index - 1]


class EditorImageResourceMergeOwnershipPerformanceContractTests(unittest.TestCase):
    def test_stream_resource_merge_moves_group_keys_and_generation_maps(self):
        source = SOURCE_PATH.read_text(encoding="utf-8")
        body = function_body(source, r"pub\([^)]*\)\s+fn\s+extend")

        self.assertNotIn(
            "resources.into_entries()",
            body,
            "stream merges must not clone a resource key for every generation",
        )
        self.assertRegex(
            body,
            r"for\s+\(resource_key,\s*mut\s+generations\)\s+in\s+resources\.by_resource_key",
        )
        self.assertRegex(
            body,
            r"entry\(resource_key\)\s*\.or_default\(\)\s*\.append\(&mut generations\)",
        )


if __name__ == "__main__":
    unittest.main()
