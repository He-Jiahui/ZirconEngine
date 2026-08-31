from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
EXTRACT = (
    ROOT
    / "zircon_runtime/src/core/framework/render/advanced_lighting/extract.rs"
)
EXTRACT_TESTS = (
    ROOT
    / "zircon_runtime/src/core/framework/render/advanced_lighting/extract/tests.rs"
)


def function_region(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for offset in range(opening, len(source)):
        character = source[offset]
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
            if depth == 0:
                return source[start : offset + 1]
    raise AssertionError(f"unterminated function: {signature}")


class Runtime99BorrowedFogVolumeFilterPerformanceContractTests(unittest.TestCase):
    def test_filter_returns_borrowed_volumes_in_source_order(self) -> None:
        source = EXTRACT.read_text(encoding="utf-8")
        function = function_region(source, "pub fn fog_volumes_for_layers")
        normalized = " ".join(function.split())

        self.assertIn("pub fn fog_volumes_for_layers<'a>", normalized)
        self.assertIn("-> impl Iterator<Item = &'a FogVolumeData> + 'a", normalized)
        self.assertIn("self.fog_volumes .iter()", normalized)
        self.assertIn(
            ".filter(move |volume| volume.layer_mask.intersects(render_layers))",
            normalized,
        )

    def test_filter_does_not_clone_or_materialize_a_vector(self) -> None:
        source = EXTRACT.read_text(encoding="utf-8")
        function = function_region(source, "pub fn fog_volumes_for_layers")

        self.assertNotIn("Vec<", function)
        self.assertNotIn(".cloned()", function)
        self.assertNotIn(".collect", function)
        self.assertNotIn("to_owned", function)

    def test_rust_behavior_test_locks_borrowed_identity_and_exhaustion(self) -> None:
        source = EXTRACT_TESTS.read_text(encoding="utf-8")
        behavior = function_region(
            source,
            "fn render_advanced_extract_filters_fog_volumes_for_camera_layers",
        )
        normalized = " ".join(behavior.split())

        self.assertIn("let mut visible = extract.fog_volumes_for_layers", behavior)
        self.assertIn(
            "std::ptr::eq( visible.next().unwrap(), &extract.fog_volumes[1] )",
            normalized,
        )
        self.assertIn("assert!(visible.next().is_none())", behavior)
        self.assertNotIn("visible[", behavior)
        self.assertNotIn(".collect", behavior)


if __name__ == "__main__":
    unittest.main()
