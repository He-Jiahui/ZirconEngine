from pathlib import Path
import re
import unittest
# GPU 内建执行器临时借出上下文元数据并在调用后归还，避免每次复制字符串；检查缺失 GPU 等恢复边界的 Rust 回归入口。

ROOT = Path(__file__).resolve().parents[2]
EXECUTORS_RS = ROOT / (
    "zircon_runtime/src/graphics/scene/execution/"
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
    def test_helper_shares_label_bytes_and_restores_executor_identity(self) -> None:
        body = metadata_helper_body()
        compact = re.sub(r"\s+", "", body)

        self.assertIn("let pass_name = context.pass_name.clone()", body)
        self.assertIn("mem::replace(&mutcontext.executor_id", compact)
        self.assertNotIn("&mut context.pass_name", body)
        self.assertIn("context.executor_id = executor_id", body)

    def test_all_builtin_gpu_calls_use_the_borrowed_metadata_helper(self) -> None:
        text = source()

        self.assertEqual(text.count("with_borrowed_gpu_metadata(context"), 29)
        self.assertEqual(text.count("|pass_name, executor_id, gpu|"), 6)
        self.assertEqual(text.count("|pass_name, _, gpu|"), 23)

    def test_executor_path_has_no_metadata_string_copies(self) -> None:
        text = source()

        # RenderReportLabel::clone shares Arc<str>; materializing a String here
        # would reintroduce the per-pass allocation this contract protects.
        self.assertNotIn("context.pass_name.to_string()", text)
        self.assertNotIn("context.executor_id.as_str().to_string()", text)

    def test_missing_gpu_restoration_has_a_direct_rust_contract(self) -> None:
        text = source()
        body = metadata_helper_body()
        compact = re.sub(r"\s+", "", body)

        self.assertIn("context.gpu_mut()", body)
        self.assertIn("executor_id.as_str(),pass_name", compact)
        test_source = (ROOT / 'zircon_runtime/src/graphics/scene/tests/builtin_postprocess_executors.rs').read_text(encoding='utf-8')
        self.assertIn("borrowed_gpu_metadata_restores_context_when_gpu_is_missing", test_source)


if __name__ == "__main__":
    unittest.main()
