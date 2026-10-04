from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SIZING = ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/sizing.rs"

# 读取富文本表格宽度求解，确认收缩事务尊重最小约束并由同一求解器处理可用宽度。
class RuntimeRichTableSizingPerformanceContractTests(unittest.TestCase):
    def test_shrink_solver_enforces_minimums_inside_the_budget_transaction(self) -> None:
        source = SIZING.read_text(encoding="utf-8")
        start = source.index("fn shrink_columns_to_budget(")
        end = source.index("\nfn checked_sum_accumulated(", start)
        solver = source[start:end]

        self.assertIn("SHRINK_SCALE_SEARCH_STEPS", source)
        self.assertIn("lower_scale", solver)
        self.assertIn("upper_scale", solver)
        self.assertIn("resolved_shrink_total", solver)
        self.assertNotIn("collect::<Vec", solver)
        self.assertNotIn("vec![", solver)

    def test_available_width_fit_delegates_shrink_constraints_to_solver(self) -> None:
        source = SIZING.read_text(encoding="utf-8")
        start = source.index("fn fit_columns_to_available_width(")
        end = source.index("\nfn shrink_columns_to_budget(", start)
        fit = source[start:end]

        self.assertIn("shrink_columns_to_budget", fit)
        self.assertNotIn("collect::<Vec", fit)
        self.assertNotIn("* shrink_budget / shrink_total).max(minimum)", fit)


if __name__ == "__main__":
    unittest.main()
