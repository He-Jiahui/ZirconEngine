from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
POPUP_STACK = ROOT / "zircon_runtime/src/ui/surface/popup_stack.rs"
REBUILD = ROOT / "zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs"


def function_body(source: str, name: str) -> str:
    start = source.index(f"fn {name}(")
    boundaries = (
        source.find(marker, start + 1)
        for marker in (
            "\nfn ",
            "\npub(super) fn ",
            "\npub(crate) fn ",
            "\npub fn ",
            "\n    fn ",
            "\n    pub(super) fn ",
            "\n    pub(crate) fn ",
            "\n    pub fn ",
            "\n#[cfg(",
        )
    )
    next_functions = [boundary for boundary in boundaries if boundary >= 0]
    return source[start:] if not next_functions else source[start : min(next_functions)]


class RuntimePopupIdLookupPerformanceContractTests(unittest.TestCase):
    def test_unique_popup_lookup_does_not_allocate_candidate_ids(self) -> None:
        source = POPUP_STACK.read_text(encoding="utf-8")
        body = function_body(source, "unique_popup_state_for_id")

        self.assertIn("popup_stack_id_matches(node, popup_id)", body)
        self.assertNotIn("popup_stack_id_for_node(node) == popup_id", body)

    def test_popup_id_match_borrows_paths_and_parses_numeric_fallback(self) -> None:
        source = POPUP_STACK.read_text(encoding="utf-8")
        body = function_body(source, "popup_stack_id_matches")
        compact = "".join(body.split())

        self.assertIn("node.node_path.0.as_str()==popup_id", compact)
        self.assertIn('popup_id.strip_prefix("node:")', body)
        self.assertIn("parse::<u64>()", body)
        self.assertNotIn("format!(", body)
        self.assertNotIn(".clone()", body)

    def test_tree_seed_reconciles_open_and_stacked_popups_through_indexes(self) -> None:
        source = POPUP_STACK.read_text(encoding="utf-8")
        body = function_body(source, "seed_popup_stack_from_tree_metadata")
        compact = "".join(body.split())

        self.assertIn("let open_popup_by_node", body)
        self.assertIn("open_popup_by_node.get(&popup_node)", compact)
        self.assertIn("let mut stacked_popup_nodes", body)
        self.assertIn("stacked_popup_nodes.insert(record.popup_node)", compact)
        self.assertNotIn("open_popups.iter().any", compact)
        self.assertNotIn("self.input.popup_stack.iter().any", compact)

    def test_popup_dependency_analysis_computes_both_domains_in_one_pass(self) -> None:
        source = POPUP_STACK.read_text(encoding="utf-8")
        body = function_body(source, "popup_dependency_impact")

        self.assertIn("UiPopupDependencyImpact", body)
        self.assertEqual(body.count("for popup in &self.input.popup_stack"), 1)
        self.assertIn("impact.render_extract", body)
        self.assertIn("impact.stack_reconciliation", body)
        self.assertNotIn("popup_trigger_requires_full_render_extract", source)
        self.assertNotIn("popup_stack_requires_reconciliation", source)

    def test_popup_dependency_analysis_indexes_changed_controls_and_owner_ancestors(self) -> None:
        source = POPUP_STACK.read_text(encoding="utf-8")
        body = function_body(source, "popup_dependency_impact")
        compact = "".join(body.split())

        self.assertIn("let mut changed_control_ids", body)
        self.assertIn("let mut changed_node_missing", body)
        self.assertIn("changed_control_ids.contains(control_id)", compact)
        self.assertIn(
            "changed_node_is_ancestor_of(changed_node_ids,owner)",
            compact,
        )
        self.assertNotIn("changed_node_affects_popup_owner", source)

    def test_layout_rebuild_reuses_one_popup_dependency_impact(self) -> None:
        source = REBUILD.read_text(encoding="utf-8")
        body = function_body(source, "rebuild_dirty")
        compact = "".join(body.split())

        self.assertIn(
            "letmutpopup_dependency_node_ids=dirty_node_ids.clone();",
            compact,
        )
        self.assertIn(
            "popup_dependency_node_ids.extend("
            "layout_stats.geometry_changed_node_ids.iter().copied());",
            compact,
        )
        self.assertIn(
            "letpopup_dependency_impact="
            "self.popup_dependency_impact(&popup_dependency_node_ids);",
            compact,
        )
        self.assertEqual(
            compact.count("self.popup_dependency_impact(&popup_dependency_node_ids)"),
            1,
        )
        self.assertIn("!popup_dependency_impact.render_extract", compact)
        self.assertIn("ifpopup_dependency_impact.stack_reconciliation", compact)
        self.assertNotIn("popup_trigger_requires_full_render_extract", body)
        self.assertNotIn("popup_stack_requires_reconciliation", body)


if __name__ == "__main__":
    unittest.main()
