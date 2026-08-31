from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_plugins/animation/runtime/src/manager/graph.rs"


def source_text() -> str:
    return SOURCE.read_text(encoding="utf-8")


def compact_source() -> str:
    return "".join(source_text().split())


class Runtime99ZLSinglePassLegacyGraphEvaluationPerformanceContract(unittest.TestCase):
    def test_graph_evaluation_builds_a_first_definition_borrowed_index(self) -> None:
        compact = compact_source()

        self.assertIn("usestd::collections::{HashMap,HashSet};", compact)
        self.assertIn("HashMap::with_capacity(graph.nodes.len())", compact)
        self.assertIn("node_index.entry(id).or_insert(node)", compact)

    def test_recursive_collection_appends_into_one_preallocated_result(self) -> None:
        compact = compact_source()

        self.assertIn("letmutclips=Vec::with_capacity(graph.nodes.len());", compact)
        self.assertIn("visited:&mutHashSet<&'astr>", compact)
        self.assertIn("clips:&mutVec<AnimationGraphClipInstance>", compact)
        self.assertIn("visited.insert(node_id)", compact)
        self.assertNotIn("visited.insert(node_id.to_string())", compact)
        self.assertNotIn(")->Vec<AnimationGraphClipInstance>", compact)

    def test_branch_weights_only_touch_the_newly_appended_clip_slice(self) -> None:
        compact = compact_source()

        self.assertIn("letclip_start=clips.len();", compact)
        self.assertIn("forclipin&mutclips[clip_start..]", compact)
        self.assertIn("letadditive_start=clips.len();", compact)
        self.assertIn("forclipin&mutclips[additive_start..]", compact)

    def test_collector_does_not_restore_linear_node_scans_or_recursive_vectors(self) -> None:
        source = source_text()

        self.assertEqual(source.count("graph.nodes.iter().find_map"), 1)
        self.assertNotIn("clips.extend(\n                        collect_graph_clips", source)
        self.assertNotIn("Some(vec![AnimationGraphClipInstance", source)


if __name__ == "__main__":
    unittest.main()
