from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
EXECUTORS_RS = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/"
    "builtin_postprocess_executors.rs"
)


def source() -> str:
    return EXECUTORS_RS.read_text(encoding="utf-8")


def metadata_helper_body() -> str:
    text = source()
    return text.split("fn with_borrowed_gpu_metadata", 1)[1].split(
        "pub(super) fn bloom_postprocess_executor", 1
    )[0]


class Runtime09H2BorrowedExecutorMetadataContract(unittest.TestCase):
    def test_helper_moves_and_restores_owned_context_metadata(self) -> None:
        body = metadata_helper_body()
        compact = re.sub(r"\s+", "", body)

        self.assertIn("mem::take(&mut context.pass_name)", body)
        self.assertIn("mem::replace(&mutcontext.executor_id", compact)
        self.assertIn("context.pass_name = pass_name", body)
        self.assertIn("context.executor_id = executor_id", body)

    def test_all_builtin_gpu_calls_use_the_borrowed_metadata_helper(self) -> None:
        text = source()

        self.assertEqual(text.count("with_borrowed_gpu_metadata(context"), 29)
        self.assertEqual(text.count("|pass_name, executor_id, gpu|"), 6)
        self.assertEqual(text.count("|pass_name, _, gpu|"), 23)

    def test_executor_path_has_no_metadata_string_copies(self) -> None:
        text = source()

        self.assertNotIn("context.pass_name.clone()", text)
        self.assertNotIn("context.executor_id.as_str().to_string()", text)

    def test_missing_gpu_restoration_has_a_direct_rust_contract(self) -> None:
        text = source()
        body = metadata_helper_body()
        compact = re.sub(r"\s+", "", body)

        self.assertIn("context.gpu_mut()", body)
        self.assertIn("executor_id.as_str(),pass_name", compact)
        self.assertIn(
            "borrowed_gpu_metadata_restores_context_when_gpu_is_missing", text
        )


if __name__ == "__main__":
    unittest.main()
