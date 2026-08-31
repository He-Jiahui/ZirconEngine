from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
CACHE_RS = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/post_process/"
    "resources/terminal_resource_cache.rs"
)


def source() -> str:
    return CACHE_RS.read_text(encoding="utf-8")


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def cache_impl_body() -> str:
    text = source()
    return text.split("impl<K, V> BoundedResourceCache<K, V>", 1)[1].split(
        "#[cfg(test)]", 1
    )[0]


class Runtime09H2StableTerminalCacheContract(unittest.TestCase):
    def test_cache_entries_track_a_monotonic_last_used_epoch(self) -> None:
        text = compact(source())

        self.assertIn("structBoundedResourceCacheEntry<K,V>", text)
        self.assertIn("last_used:u64", text)
        self.assertIn("access_epoch:u64", text)

    def test_cache_hits_update_in_place_without_shifting_entries(self) -> None:
        body = compact(cache_impl_body())
        hit_body = body.split("fnget_or_insert_with", 1)[1].split(
            "ifself.entries.len()==self.capacity", 1
        )[0]

        self.assertIn("self.entries.iter_mut().find", hit_body)
        self.assertIn("entry.last_used=self.access_epoch", hit_body)
        self.assertNotIn(".remove(", hit_body)
        self.assertNotIn(".push(", hit_body)

    def test_full_cache_replaces_the_oldest_stable_slot(self) -> None:
        body = compact(cache_impl_body())

        self.assertIn("min_by_key(|(_,entry)|entry.last_used)", body)
        self.assertIn("self.entries[oldest_index]=", body)
        self.assertNotIn("self.entries.remove(0)", body)

    def test_lru_hit_promotion_has_a_direct_rust_contract(self) -> None:
        self.assertIn(
            "bounded_resource_cache_hit_promotes_the_entry_without_reordering_slots",
            source(),
        )


if __name__ == "__main__":
    unittest.main()
