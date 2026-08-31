import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SYSTEM_PARAM_ACCESS = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "scene"
    / "ecs"
    / "system"
    / "system_param_access.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(function_name)}\s*\(", source)
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {function_name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class LinearReadOnlyProjectionPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SYSTEM_PARAM_ACCESS.read_text(encoding="utf-8")

    def test_query_projection_uses_one_monotonic_write_cursor(self) -> None:
        body = function_body(self.source, "add_query_access")
        self.assertIn("let mut write_index = 0;", body)
        self.assertIn("write_index < query_access.writes().len()", body)
        self.assertIn("write_id < *component_id", body)
        self.assertIn("write_id == *component_id", body)
        self.assertNotIn("query_access.writes().binary_search", body)

    def test_query_projection_keeps_existing_access_admission_calls(self) -> None:
        body = function_body(self.source, "add_query_access")
        self.assertIn("self.component_access.add_write(*component_id)?;", body)
        self.assertIn("self.component_access.add_read(*component_id)?;", body)
        self.assertIn("self.component_access.add_with(*component_id);", body)
        self.assertIn("self.component_access.add_without(*component_id);", body)

    def test_rust_regressions_cover_projection_and_conflicts(self) -> None:
        self.assertIn(
            "fn runtime60_batch_mixed_query_access_projects_read_only_ids_in_order()",
            self.source,
        )
        self.assertIn(
            "fn runtime60_batch_interleaved_writes_are_skipped_by_the_monotonic_cursor()",
            self.source,
        )
        self.assertIn(
            "fn runtime60_batch_projected_read_keeps_existing_write_conflicts()",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
