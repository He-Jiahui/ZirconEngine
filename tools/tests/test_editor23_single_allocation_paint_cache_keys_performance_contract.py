import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PAINT_NODES = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "host_contract"
    / "paint_template_nodes"
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


class EditorSingleAllocationPaintCacheKeysPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.image_key = (
            PAINT_NODES / "visual_assets" / "loading" / "key.rs"
        ).read_text(encoding="utf-8")
        cls.progress_key = (
            PAINT_NODES
            / "template_material_feedback"
            / "circular_progress"
            / "key.rs"
        ).read_text(encoding="utf-8")

    def test_image_key_writes_into_one_exactly_sized_string(self) -> None:
        body = function_body(self.image_key, "image_pixels_cache_key")
        compact = re.sub(r"\s+", "", body)
        self.assertEqual(body.count("String::with_capacity"), 1)
        self.assertIn("image_pixels_cache_key_capacity", body)
        self.assertIn("write!(&mutkey", compact)
        self.assertNotIn("format!", body)
        self.assertNotIn("to_string", body)

    def test_progress_key_writes_into_one_exactly_sized_string(self) -> None:
        body = function_body(self.progress_key, "circular_progress_image_key")
        compact = re.sub(r"\s+", "", body)
        self.assertEqual(body.count("String::with_capacity"), 1)
        self.assertIn("circular_progress_image_key_capacity", body)
        self.assertIn("write!(&mutkey", compact)
        self.assertNotIn("format!", body)
        self.assertNotIn("to_string", body)

    def test_capacity_helpers_account_for_decimal_and_fixed_hex_fields(self) -> None:
        self.assertIn("fn decimal_digits", self.image_key)
        self.assertIn("fn image_pixels_cache_key_capacity", self.image_key)
        self.assertIn("TINT_HEX_LEN", self.image_key)
        self.assertIn("fn decimal_digits", self.progress_key)
        self.assertIn(
            "fn circular_progress_image_key_capacity", self.progress_key
        )
        self.assertIn("COLOR_HEX_LEN", self.progress_key)
        self.assertIn("PERCENT_HEX_LEN", self.progress_key)

    def test_rust_regressions_lock_the_existing_wire_format(self) -> None:
        self.assertIn("image_cache_key_wire_format_is_stable", self.image_key)
        self.assertIn(
            "circular_progress_cache_key_wire_format_is_stable",
            self.progress_key,
        )


if __name__ == "__main__":
    unittest.main()
