from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
STACK_RS = ROOT / "zircon_runtime/src/core/framework/render/post_process/stack.rs"


def source() -> str:
    return STACK_RS.read_text(encoding="utf-8")


def effect_disable_body() -> str:
    text = source()
    return text.split("pub fn with_effect_disabled", 1)[1].split(
        "pub fn without_history_resources", 1
    )[0]


class Runtime09H2ZeroCloneEffectDisableContract(unittest.TestCase):
    def test_disabled_outputs_use_a_borrowed_hash_index(self) -> None:
        body = effect_disable_body()

        self.assertIn("HashSet<&str>", body)
        self.assertNotIn(".cloned()", body)
        self.assertIn(".map(String::as_str)", body)

    def test_disabled_output_index_reserves_the_exact_upper_bound(self) -> None:
        body = effect_disable_body()

        self.assertIn("disabled_output_count", body)
        self.assertIn("HashSet::with_capacity(disabled_output_count)", body)

    def test_provider_outputs_are_moved_then_restored_without_name_copies(self) -> None:
        body = effect_disable_body()

        self.assertIn("mem::take(&mut effect.produced_outputs)", body)
        self.assertIn("drop(disabled_outputs)", body)
        self.assertIn("self.effects[index].produced_outputs = outputs", body)

    def test_dependency_cleanup_remains_in_the_mutation_pass(self) -> None:
        body = effect_disable_body()

        self.assertIn(
            "effect.after.retain(|dependency| *dependency != kind)", body
        )
        self.assertNotIn("if disabled_output_groups.is_empty()", body)


if __name__ == "__main__":
    unittest.main()
