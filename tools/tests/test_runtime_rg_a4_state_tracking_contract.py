import unittest
from pathlib import Path


class RuntimeRgA4StateTrackingContractTests(unittest.TestCase):
    """Static guard for the bounded RenderGraph access-scope tracker.

    RG-A4 keeps dependency tracking device-neutral and compact: buffer histories split only at
    affected boundaries, texture scopes describe mip/aspect planes over layer intervals, and
    malformed texture descriptors are rejected before history allocation.  These checks preserve
    those ownership and complexity boundaries while the managed native gate is unavailable.
    """

    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.tracker = self.repo_root / "zircon_runtime/src/render_graph/builder/access_scope_tracker.rs"
        self.buffer_history = (
            self.repo_root
            / "zircon_runtime/src/render_graph/builder/access_scope_tracker/buffer_scope_history.rs"
        )
        self.texture_scope = (
            self.repo_root
            / "zircon_runtime/src/render_graph/builder/access_scope_tracker/texture_scope.rs"
        )
        self.state_plan_tests = self.repo_root / "zircon_runtime/src/render_graph/tests/resource_state_plan.rs"
        self.validation_tests = self.repo_root / "zircon_runtime/src/render_graph/tests/builder_validation.rs"

    @staticmethod
    def _production(source: str) -> str:
        """Exclude test-only helpers when checking production ordering."""
        return source.split("#[cfg(test)]\nmod tests", 1)[0]

    def test_tracker_has_compact_scope_owners_and_a_work_receipt(self) -> None:
        tracker = self._production(self.tracker.read_text(encoding="utf-8"))
        texture = self.texture_scope.read_text(encoding="utf-8")
        # The buffer child keeps its small whole-map comparison helper above the production
        # history implementation, so do not truncate this file at its test module.
        buffer = self.buffer_history.read_text(encoding="utf-8")

        for anchor in (
            "mod buffer_scope_history;",
            "mod texture_scope;",
            "use buffer_scope_history::{BufferScopeHistory, BufferSegment};",
            "use texture_scope::{TexturePlane, TextureScope};",
            "pub(super) struct AccessScopeWorkReceipt",
            "lookup_visits: usize",
            "split_visits: usize",
            "read_visits: usize",
            "update_visits: usize",
            "merge_visits: usize",
            "Texture(HashMap<TexturePlane, BufferScopeHistory>)",
            "Buffer(BufferScopeHistory)",
            "PreparedScopeKind::Texture(TextureScope::new(desc, range))",
        ):
            self.assertIn(anchor, tracker)

        for anchor in (
            "pub(super) struct TexturePlane",
            "mip_start: u32",
            "mip_end: u32",
            "layer_start: u32",
            "layer_end: u32",
            "layer_limit: u32",
            "aspects: [RenderGraphTextureAspect; 2]",
            "(self.mip_start..self.mip_end).flat_map",
        ):
            self.assertIn(anchor, texture)

        self.assertIn("pub(super) struct BufferScopeHistory", buffer)
        self.assertIn("segments: BTreeMap<u64, BufferSegment>", buffer)
        self.assertNotIn("Texture(Vec<TextureCell>)", tracker)
        self.assertNotIn("fn texture_cells(", tracker)

    def test_prepare_scope_validates_texture_shape_before_projection_and_history(self) -> None:
        source = self._production(self.tracker.read_text(encoding="utf-8"))

        for anchor in (
            "fn prepare_scope(",
            "validate_texture_descriptor_for_tracking(resource, desc)?;",
            "let metadata = self.project_texture_view_alias_scope(resource, metadata)?;",
            "let metadata = Self::canonicalize_access_metadata(resource, &descriptor, metadata)?;",
            "PreparedScopeKind::Texture(TextureScope::new(desc, range))",
            "self.ensure_history(identity, &descriptor);",
            "fn validate_texture_descriptor_for_tracking(",
            "desc.shape_validation_error()",
            "desc.sample_count == 0",
            "desc.sample_count > 1 && desc.dimension != TextureDimension::D2",
            "desc.sample_count > 1 && desc.mip_levels > 1",
            "!desc.mip_levels_fit_shape()",
        ):
            self.assertIn(anchor, source)

        validation = source.index("validate_texture_descriptor_for_tracking(resource, desc)?;")
        projection = source.index("let metadata = self.project_texture_view_alias_scope(resource, metadata)?;")
        canonicalization = source.index("let metadata = Self::canonicalize_access_metadata(resource, &descriptor, metadata)?;")
        texture_expansion = source.index("PreparedScopeKind::Texture(TextureScope::new(desc, range))")
        history = source.index("self.ensure_history(identity, &descriptor);")
        self.assertLess(validation, projection)
        self.assertLess(projection, canonicalization)
        self.assertLess(canonicalization, texture_expansion)
        self.assertLess(texture_expansion, history)

    def test_buffer_history_uses_boundary_lookup_and_local_neighbor_merges(self) -> None:
        # See the note above: this child deliberately places cfg(test) helpers before the
        # production `BufferScopeHistory` declaration.
        source = self.buffer_history.read_text(encoding="utf-8")

        for anchor in (
            "pub(super) fn ensure_boundaries(",
            "self.split_at(start, identity, work)?;",
            "self.split_at(end, identity, work)",
            "pub(super) fn overlapping_segments(",
            ".segments\n            .range(..=start)\n            .next_back()",
            ".segments\n            .range(first..end)",
            "fn split_at(",
            "self.segments.range(..=boundary).next_back()",
            "pub(super) fn merge_adjacent_equal_around(",
            ".segments\n            .range(..start)\n            .next_back()",
            ".segments\n                .range((Excluded(left_start), Unbounded))\n                .next()",
            "left.end == right_start && left.history == right.history",
        ):
            self.assertIn(anchor, source)

        whole_map_test = source.index("#[cfg(test)]\n    fn merge_adjacent_equal_whole_map")
        self.assertNotIn(
            "let starts: Vec<_> = self.segments.keys().copied().collect();",
            source[:whole_map_test],
        )

        split = source.index("self.split_at(start, identity, work)?;")
        overlap = source.index("pub(super) fn overlapping_segments(")
        merge = source.index("pub(super) fn merge_adjacent_equal_around(")
        self.assertLess(split, overlap)
        self.assertLess(overlap, merge)

    def test_regressions_cover_large_ranges_arrays_shapes_and_queue_transitions(self) -> None:
        state_plan = self.state_plan_tests.read_text(encoding="utf-8")
        validation = self.validation_tests.read_text(encoding="utf-8")

        for test_name in (
            "fn compiled_state_plan_disjoint_buffer_work_is_bounded_at_ten_thousand_ranges()",
            "fn compiled_state_plan_disjoint_buffer_release_workload_evidence()",
            "fn compact_large_texture_mixed_ranges_match_whole_map_and_exact_predecessors()",
            "fn compiled_state_plan_tracks_a_large_texture_array_with_cardinality_independent_work()",
            "fn state_plan_prepares_texture_scopes_as_compact_ranges()",
            "fn compiled_state_plan_reports_queue_transition_and_legacy_fallback()",
            "fn compiled_state_plan_emits_exact_adjacent_buffer_predecessors()",
            "fn compiled_state_plan_emits_every_exact_partial_texture_predecessor()",
        ):
            self.assertIn(test_name, state_plan)

        for anchor in (
            "const RANGE_COUNT: usize = 10_000;",
            "const ARRAY_LAYERS: u32 = u32::MAX;",
            "const LAYERS: u32 = u32::MAX;",
            "compile_with_whole_map_coalescing_for_test()",
            "tracking_lookup_visits()",
            "tracking_split_visits()",
            "tracking_read_visits()",
            "tracking_update_visits()",
            "tracking_merge_visits()",
            "tracking_work_count()",
            "Texture(Vec<TextureCell>)",
        ):
            self.assertIn(anchor, state_plan)

        for test_name in (
            "fn builder_rejects_invalid_texture_shapes_before_scope_tracking()",
            "fn builder_keeps_large_neutral_texture_shapes_legal()",
            "fn builder_rejects_overlapping_same_kind_access_scopes_in_one_pass()",
            "fn builder_allows_overlapping_read_scopes_for_parent_and_texture_view_alias()",
        ):
            self.assertIn(test_name, validation)

        self.assertIn("TextureDescriptorInvalid", validation)
        self.assertIn("expect_err(\"invalid texture shape must fail before scope tracking\")", validation)


if __name__ == "__main__":
    unittest.main()
