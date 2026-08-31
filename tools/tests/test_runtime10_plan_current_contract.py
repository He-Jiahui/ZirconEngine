import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RUNTIME10_PLAN = (
    REPO_ROOT
    / "docs"
    / "plans"
    / "zircon_runtime"
    / "runtime"
    / "10-dynamic-api-and-interface-convergence.md"
)


class Runtime10PlanCurrentContractTests(unittest.TestCase):
    def test_current_contract_is_the_v7_build_set_abi(self) -> None:
        plan = RUNTIME10_PLAN.read_text(encoding="utf-8")

        self.assertIn("## Current V7 ABI State", plan)
        self.assertIn("`ZrRuntimeApiV7`", plan)
        self.assertIn("25-field", plan)
        self.assertIn("23 个函数指针", plan)
        self.assertIn("`zircon_runtime_get_api_v7`", plan)
        self.assertIn("`ZR_RUNTIME_GET_API_SYMBOL_V7`", plan)

        for stale_current_contract in (
            "函数表版本策略维持：`ZrRuntimeApiV3`",
            "经 `ZrRuntimeApiV3` 可达的 18 个函数指针",
            "当前实现：`zircon_runtime_get_api_v3`",
            "V3 符号缺失",
        ):
            self.assertNotIn(stale_current_contract, plan)


if __name__ == "__main__":
    unittest.main()
