import unittest
from pathlib import Path

# 本文件只读取 Rust 源码做静态结构断言，不执行 Rust 测试或分配图集资源。
# 读取域须与被检查内容的实际所属文件一致。

# 从本测试文件定位仓库根，不依赖当前工作目录。
REPO_ROOT = Path(__file__).resolve().parents[2]
# 算术守卫读取实现文件，不会自动展开 Rust 的 path 测试挂载。
SHELF = REPO_ROOT / "zircon_runtime/src/text/atlas/shelf_allocator.rs"

# 读取文本图集 shelf 准入，确认坐标采用受检算术且纵向溢出由 Rust 回归锚定。
# 普通 TestCase 暴露两个文本守卫，均没有跳过条件。
class RuntimeTextAtlasShelfAdmissionContractTests(unittest.TestCase):
    # 四个包含断言只锚定受检加法的源码文本，不证明运行时路径。
    def test_shelf_bounds_use_checked_coordinate_arithmetic(self) -> None:
        source = SHELF.read_text(encoding="utf-8")

        self.assertIn("checked_add(size.x)", source)
        self.assertIn("checked_add(size.y)", source)
        self.assertIn("checked_add(self.shelf_height)", source)
        self.assertIn("checked_add(self.padding_px)", source)

    # 本方法沿用实现文件输入，但下方断言要求的是直属 Rust 测试中的名称。
    def test_shelf_has_vertical_overflow_regression(self) -> None:
        source = SHELF.read_text(encoding="utf-8")

        # BUG: [CR-ZCA-R118-0001] 正常运行本方法时，只读实现文件却搜索直属 Rust 测试名；
        # 该名称在实现中为 0 处、在独立测试文件中为 1 处，下方 assertIn 因而抛出 AssertionError。
        # 这是读取域与断言对象失配的静态证明，尚未执行此测试。
        self.assertIn(
            "render_text_atlas_shelf_rejects_vertical_coordinate_overflow",
            source,
        )


# 直接执行时交给 unittest 收集本模块的普通 test_ 方法。
if __name__ == "__main__":
    unittest.main()
