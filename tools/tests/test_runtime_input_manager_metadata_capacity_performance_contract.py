from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
OUTCOME = ROOT / "zircon_runtime/src/ui/dispatch/input_manager/outcome.rs"
IME_REQUESTS = (
    ROOT / "zircon_runtime/src/ui/dispatch/input_manager/ime_host_requests.rs"
)


def rust_block(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated Rust block: {signature}")


class RuntimeInputManagerMetadataCapacityPerformanceContractTests(unittest.TestCase):
    def test_dispatch_metadata_uses_one_top_level_result_pass(self) -> None:
        source = OUTCOME.read_text(encoding="utf-8")
        body = rust_block(source, "fn collect_dispatch_metadata(")

        self.assertEqual(body.count("for result in results"), 1)
        self.assertIn("host_requests.extend(result.host_requests.iter().cloned())", body)
        self.assertNotIn("flat_map", body)
        self.assertNotIn("results.iter().any", body)

    def test_dispatch_metadata_stops_effect_scans_after_redraw_is_known(self) -> None:
        body = rust_block(
            OUTCOME.read_text(encoding="utf-8"), "fn collect_dispatch_metadata("
        )

        self.assertIn("if !redraw_requested", body)
        self.assertIn("UiDispatchEffect::DirtyRedraw", body)
        self.assertIn("redraw_requested = true", body)

    def test_ime_batch_reserves_from_the_iterator_lower_bound_before_iteration(self) -> None:
        source = IME_REQUESTS.read_text(encoding="utf-8")
        body = rust_block(
            source,
            "pub(super) fn append_ime_host_requests_for_input_method_requests(",
        )

        into_iter = body.index("let requests = requests.into_iter()")
        reserve = body.index("reserve_ime_host_request_capacity")
        request_loop = body.index("for request in requests")
        self.assertLess(into_iter, reserve)
        self.assertLess(reserve, request_loop)
        self.assertIn("requests.size_hint().0", body)
        self.assertNotIn(".collect()", body)

    def test_ime_capacity_uses_the_saturating_maximum_expansion(self) -> None:
        source = IME_REQUESTS.read_text(encoding="utf-8")
        reserve = rust_block(source, "fn reserve_ime_host_request_capacity(")

        self.assertIn("MAX_HOST_REQUESTS_PER_INPUT_METHOD_REQUEST: usize = 3", source)
        self.assertIn("request_count.saturating_mul", reserve)
        self.assertIn("output.reserve", reserve)


if __name__ == "__main__":
    unittest.main()
