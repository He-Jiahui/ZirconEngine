from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
GRAPH_COMPILE = (
    REPO_ROOT
    / "zircon_plugins"
    / "sound"
    / "runtime"
    / "src"
    / "kira_bridge"
    / "graph_compile.rs"
)


class Runtime99znSoundGraphHierarchyPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = GRAPH_COMPILE.read_text(encoding="utf-8")

    def test_structural_diff_lazily_reuses_before_and_after_hierarchy_indexes(self) -> None:
        self.assertIn("OnceCell::<TrackHierarchyIndex>::new()", self.source)
        self.assertEqual(
            self.source.count("OnceCell::<TrackHierarchyIndex>::new()"),
            2,
        )
        self.assertIn("get_or_init(|| TrackHierarchyIndex::new(before))", self.source)
        self.assertIn("get_or_init(|| TrackHierarchyIndex::new(after))", self.source)

    def test_hierarchy_index_builds_parent_and_child_lookups_once(self) -> None:
        self.assertIn("struct TrackHierarchyIndex", self.source)
        self.assertIn("parents: HashMap<SoundTrackId, Option<SoundTrackId>>", self.source)
        self.assertIn("children: HashMap<SoundTrackId, Vec<SoundTrackId>>", self.source)
        self.assertIn("HashMap::with_capacity(graph.tracks.len())", self.source)

    def test_graph_queries_use_the_shared_index(self) -> None:
        self.assertIn("after_hierarchy", self.source)
        self.assertIn(".has_ancestor_in(*candidate, &structural_candidates)", self.source)
        self.assertIn(".subtree_ids(*root)", self.source)
        self.assertIn(".depth(track.id)", self.source)
        self.assertNotIn("fn track_depth(graph:", self.source)
        self.assertNotIn("fn has_ancestor_in(\n    graph:", self.source)

    def test_subtree_projection_does_not_use_fixed_point_full_map_scans(self) -> None:
        self.assertIn("let mut stack = vec![root];", self.source)
        self.assertIn("self.children.get(&parent)", self.source)
        self.assertNotIn("let before = subtree.len();", self.source)
        self.assertNotIn("for (track, parent) in &parents", self.source)


if __name__ == "__main__":
    unittest.main()
