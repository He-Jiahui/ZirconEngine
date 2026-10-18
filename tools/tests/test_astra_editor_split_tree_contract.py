from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def production(source: str) -> str:
    return source.split("#[cfg(test)]", 1)[0]


class AstraEditorSplitTreeContractTests(unittest.TestCase):
    """Static guard for preserving editor split topology, leaves, focus, and ratios."""

    def test_document_node_is_a_recursive_split_tree_with_leaf_content(self) -> None:
        source = production(read("zircon_editor/src/ui/workbench/layout/document_node.rs"))

        for anchor in (
            "pub enum DocumentNode",
            "SplitNode {",
            "axis: SplitAxis",
            "ratio: f32",
            "first: Box<DocumentNode>",
            "second: Box<DocumentNode>",
            "Tabs(DocumentLeafLayout)",
            "pub(crate) fn node_at_path_mut",
            "pub(crate) fn remove_instance",
            "pub(crate) fn contains",
            "first.normalize_node_ids(used)",
            "second.normalize_node_ids(used)",
        ):
            self.assertIn(anchor, source)
        self.assertIn("#[serde(deny_unknown_fields)]", source)

    def test_normalization_recurses_without_flattening_and_bounds_ratios(self) -> None:
        source = production(read("zircon_editor/src/ui/workbench/layout/manager/normalize.rs"))

        for anchor in (
            "const DEFAULT_SPLIT_RATIO: f32 = 0.5;",
            "const MIN_SPLIT_RATIO: f32 = 0.1;",
            "const MAX_SPLIT_RATIO: f32 = 0.9;",
            "normalize_node_ids(&mut used_node_ids)",
            "normalize_document_splits(&mut activity_window.content_workspace)",
            "normalize_document_splits(&mut window.workspace)",
            "ratio.clamp(MIN_SPLIT_RATIO, MAX_SPLIT_RATIO)",
            "normalize_document_splits(first)",
            "normalize_document_splits(second)",
        ):
            self.assertIn(anchor, source)
        self.assertNotIn("DocumentNode::Tabs(TabStackLayout::default())", source)

    def test_focus_walks_both_branches_and_updates_the_leaf_owner(self) -> None:
        source = production(read("zircon_editor/src/ui/workbench/layout/manager/focus.rs"))

        for anchor in (
            "fn focus_in_document_node(",
            "DocumentNode::Tabs(stack)",
            "stack.tabs.contains(instance_id)",
            "stack.active_tab = Some(instance_id.clone())",
            "DocumentNode::SplitNode { first, second, .. }",
            "Self::focus_in_document_node(first, instance_id)",
            "Self::focus_in_document_node(second, instance_id)",
        ):
            self.assertIn(anchor, source)
        self.assertNotIn("default_activity_window_mut()", source)
        self.assertNotIn("activity_windows.is_empty()", source)

    def test_commands_insert_and_resize_splits_with_fail_closed_ratio_validation(self) -> None:
        source = production(read("zircon_editor/src/ui/workbench/layout/manager/apply.rs"))

        for anchor in (
            "LayoutCommand::CreateSplit",
            "let previous = node.clone();",
            "*node = DocumentNode::SplitNode",
            "ratio: 0.5",
            "LayoutCommand::ResizeSplit",
            "if !ratio.is_finite()",
            "LayoutCommandError::NonFiniteSplitRatio",
            "ratio.clamp(0.1, 0.9)",
            "LayoutCommandError::MissingSplitPath",
            "LayoutCommandError::TargetPathIsNotSplitNode",
        ):
            self.assertIn(anchor, source)
        self.assertLess(source.index("if !ratio.is_finite()"), source.index("ratio.clamp(0.1, 0.9)"))

    def test_regressions_round_trip_nested_topology_and_repair_only_invalid_ratios(self) -> None:
        identity = read("zircon_editor/src/ui/workbench/layout/tests/document_identity_tests.rs")
        ratios = read("zircon_editor/src/ui/workbench/layout/manager/normalize/tests/split_ratio_tests.rs")

        for name in (
            "astra_nested_workspace_reopens_exact_node_ids_ratios_and_content",
            "astra_legacy_nested_workspace_migrates_once_without_repartitioning",
            "astra_duplicate_saved_node_ids_repair_only_later_owners",
        ):
            self.assertIn(f"fn {name}", identity)
        for name in (
            "astra_editor_layout_valid_split_tree_and_ratios_survive_normalization_exactly",
            "astra_editor_layout_invalid_saved_ratios_are_repaired_recursively",
            "astra_editor_layout_negative_saved_ratio_clamps_without_reordering_leaves",
        ):
            self.assertIn(f"fn {name}", ratios)
        self.assertIn("nested split topology must survive normalization", ratios)
        self.assertIn("without_repartitioning", identity)


if __name__ == "__main__":
    unittest.main()
