from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/script/vm/behavior_bridge.rs"


class InPlaceBehaviorCallbackCachePerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.production = cls.source.split("#[cfg(test)]", maxsplit=1)[0]
        helper_parts = cls.production.split("fn cache_callback", maxsplit=1)
        cls.helper = (
            helper_parts[1].split("fn manager", maxsplit=1)[0]
            if len(helper_parts) == 2
            else ""
        )

    def test_existing_callback_handle_is_replaced_in_place(self) -> None:
        self.assertIn("callbacks.get_mut(callback)", self.helper)
        self.assertIn("*cached = handle;", self.helper)
        self.assertIn("return false;", self.helper)

    def test_callback_key_is_cloned_only_for_cache_miss(self) -> None:
        self.assertIn("callbacks.insert(callback.clone(), handle);", self.helper)
        self.assertEqual(self.production.count("callback.clone()"), 1)

    def test_refresh_paths_share_the_in_place_cache_helper(self) -> None:
        self.assertIn(
            "self.cache_callback(callback, registration.callback);",
            self.production,
        )
        self.assertIn("self.cache_callback(callback, handle);", self.production)
        self.assertIn(
            "behavior_callback_cache_only_clones_a_missing_key",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
