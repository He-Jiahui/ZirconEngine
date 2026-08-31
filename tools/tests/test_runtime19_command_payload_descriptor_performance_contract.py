from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GENERATOR = ROOT / "examples/woc/tools/command_payload_codegen.mjs"
GENERATED = (
    ROOT
    / "examples/woc/native/crates/woc_protocol/src/generated_command_payloads.rs"
)


def rust_function_body(source: str, name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(name)}\s*\([^)]*\)[^{{]*{{", source)
    if match is None:
        raise AssertionError(f"missing function {name}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated function {name}")
    return source[match.end() : index - 1]


class Runtime19CommandPayloadDescriptorPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.generator = GENERATOR.read_text(encoding="utf-8")
        cls.generated = GENERATED.read_text(encoding="utf-8")

    def test_generator_owns_the_binary_lookup_template(self) -> None:
        self.assertIn("binary_search_by_key(&id, |entry| entry.id)", self.generator)
        self.assertEqual(
            self.generator.count(
                "COMMAND_PAYLOAD_CATALOG.iter().find(|entry| entry.id == id)"
            ),
            1,
            "the generator may retain one linear lookup only as the benchmark oracle",
        )

    def test_generated_lookup_uses_the_sorted_catalog_without_allocation(self) -> None:
        body = rust_function_body(self.generated, "command_payload_descriptor")
        self.assertIn("binary_search_by_key(&id, |entry| entry.id)", body)
        self.assertNotIn(".iter().find", body)
        self.assertNotIn("collect", body)

    def test_generated_catalog_is_strictly_sorted_and_unique(self) -> None:
        catalog = self.generated.split(
            "pub const COMMAND_PAYLOAD_CATALOG: &[CommandPayloadDescriptor] = &[", 1
        )[1].split("];", 1)[0]
        ids = [int(value) for value in re.findall(r"\bid:\s*(\d+),", catalog)]
        self.assertEqual(len(ids), 157)
        self.assertEqual(ids, sorted(set(ids)))

    def test_generated_catalog_only_uses_declared_payload_kinds(self) -> None:
        variants = self.generated.split("pub enum CommandPayloadKind {", 1)[1].split(
            "}", 1
        )[0]
        declared = set(re.findall(r"\b([A-Z][A-Za-z0-9_]*)\s*,", variants))
        used = set(re.findall(r"CommandPayloadKind::([A-Z][A-Za-z0-9_]*)", self.generated))
        self.assertEqual(used - declared, set())

    def test_release_gate_emits_raw_paired_samples(self) -> None:
        self.assertIn("for id in u16::MIN..=u16::MAX", self.generated)
        self.assertIn("RUNTIME19_COMMAND_PAYLOAD_LOOKUP_PERF", self.generated)
        self.assertIn("sample_pairs=21", self.generated)
        self.assertIn("percentile_method=nearest_rank", self.generated)


if __name__ == "__main__":
    unittest.main()
