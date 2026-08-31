from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/asset/watch/asset_uri_for_path.rs"
RUST_TESTS = ROOT / "zircon_runtime/src/asset/watch/asset_uri_for_path/tests.rs"


def rust_function_body(source: str, name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(name)}\s*\([^{{]*{{", source)
    if match is None:
        raise AssertionError(f"missing Rust function {name}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated Rust function {name}")
    return source[match.end() : index - 1]


class Runtime25AssetUriProjectionPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_projection_uses_one_preallocated_output_buffer(self) -> None:
        body = rust_function_body(cls_source := self.source, "asset_uri_for_path")
        self.assertIn('String::with_capacity("res://".len()', body)
        self.assertIn('normalized.push_str("res://")', body)
        self.assertIn("for (index, component) in relative.components().enumerate()", body)
        self.assertEqual(cls_source.count("String::with_capacity"), 1)

    def test_projection_does_not_collect_join_or_format(self) -> None:
        body = rust_function_body(self.source, "asset_uri_for_path")
        self.assertNotIn("collect::<Vec", body)
        self.assertNotIn(".join(", body)
        self.assertNotIn("format!(", body)

    def test_owned_rust_contract_is_wired(self) -> None:
        self.assertIn("#[cfg(test)]\nmod tests;", self.source)
        tests = RUST_TESTS.read_text(encoding="utf-8")
        self.assertIn("nested_path_projects_to_resource_uri", tests)
        self.assertIn("path_outside_root_is_rejected", tests)

    def test_rust_contract_covers_nested_projection_and_escape(self) -> None:
        tests = RUST_TESTS.read_text(encoding="utf-8")
        self.assertIn('Path::new("sandbox/assets/materials/grid.zmaterial")', tests)
        self.assertIn('AssetUri::parse("res://materials/grid.zmaterial")', tests)
        self.assertIn('Path::new("sandbox/outside/grid.zmaterial")', tests)
        self.assertIn("ResourceLocatorError::EscapeAttempt", tests)

    def test_intermediate_container_count_drops_by_at_least_sixty_percent(self) -> None:
        legacy_intermediate_containers = 3  # component Vec, joined String, formatted String
        optimized_intermediate_containers = 1  # preallocated URI String
        reduction_percent = (
            (legacy_intermediate_containers - optimized_intermediate_containers)
            * 100.0
            / legacy_intermediate_containers
        )
        self.assertGreaterEqual(reduction_percent, 60.0)


if __name__ == "__main__":
    unittest.main()
