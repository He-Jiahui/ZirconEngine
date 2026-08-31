from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RUNTIME_UI = ROOT / "zircon_runtime/src/dynamic_api/session/runtime_ui.rs"
INPUT_PUBLICATION = (
    ROOT
    / "zircon_runtime/src/dynamic_api/session/runtime_ui/input_publication.rs"
)
EVENTS = ROOT / "zircon_runtime/src/dynamic_api/session/events.rs"
MOUSE_MOTION = ROOT / "zircon_runtime/src/ui/surface/input/mouse_motion.rs"
INPUT_MANAGER = ROOT / "zircon_runtime/src/ui/dispatch/input_manager/manager.rs"
INPUT_DISPATCH = ROOT / "zircon_runtime/src/ui/surface/input/dispatch.rs"
POINTER_INPUT = ROOT / "zircon_runtime/src/ui/surface/input/pointer.rs"
EVENT_ROUTING = ROOT / "zircon_runtime/src/ui/surface/surface/event_routing.rs"
RECORD = (
    ROOT
    / "docs/plans/optimize/zircon_editor/01/2026-08-28-runtime-ui-surface-input-publication-authority.md"
)
UNREAL_APPLICATION = (
    ROOT
    / "dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp"
)
UNREAL_HIT_GRID = (
    ROOT
    / "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp"
)


def function_source(source: str, start_anchor: str, end_anchor: str) -> str:
    start = source.index(start_anchor)
    end = source.index(end_anchor, start)
    return source[start:end]


class RuntimeUiSurfaceInputPublicationDesignContractTests(unittest.TestCase):
    def test_unrouted_mouse_motion_is_rejected_before_surface_fanout(self):
        source = RUNTIME_UI.read_text(encoding="utf-8")
        generic = function_source(
            source,
            "    pub(super) fn dispatch_input(",
            "\n    pub(super) fn dispatch_pointer(",
        )
        fast_path = function_source(
            generic,
            "        if matches!(&event, UiInputEvent::MouseMotion(_))",
            "        let root_size = ui_size(viewport_size);",
        )

        self.assertIn("ui.surface_set.input.unrouted_reject_count", fast_path)
        self.assertIn("return Ok(false);", fast_path)
        self.assertNotIn("input_event_for_surface", fast_path)
        self.assertLess(
            generic.index("UiInputEvent::MouseMotion"),
            generic.index("for surface_index in (0..self.surfaces.len()).rev()"),
        )

    def test_focused_input_uses_the_published_owner_before_surface_fanout(self):
        source = RUNTIME_UI.read_text(encoding="utf-8")
        generic = function_source(
            source,
            "    pub(super) fn dispatch_input(",
            "\n    pub(super) fn dispatch_pointer(",
        )
        direct_route = function_source(
            generic,
            "        if input_requires_focus_owner(&event)",
            "        let mut event = Some(event);",
        )

        self.assertIn("self.focused_surface", direct_route)
        self.assertIn("ui.surface_set.input.focus_direct_route_count", direct_route)
        self.assertIn("dispatch_input_to_surface", direct_route)
        self.assertLess(
            generic.index("input_requires_focus_owner(&event)"),
            generic.index("for surface_index in (0..self.surfaces.len()).rev()"),
        )
        self.assertIn(
            "fn refresh_input_owners_from_publication(&mut self)", source
        )
        self.assertIn("focus_before != focus_after", source)

    def test_navigation_and_analog_use_a_published_surface_owner(self):
        source = RUNTIME_UI.read_text(encoding="utf-8")
        generic = function_source(
            source,
            "    pub(super) fn dispatch_input(",
            "\n    pub(super) fn dispatch_pointer(",
        )
        navigation_direct = function_source(
            generic,
            "        if input_requires_navigation_owner(&event)",
            "        let mut event = Some(event);",
        )

        self.assertIn("navigation_surface: Option<usize>", source)
        self.assertIn("self.navigation_surface", navigation_direct)
        self.assertIn("ui.surface_set.input.navigation_direct_route_count", navigation_direct)
        self.assertIn("dispatch_input_to_surface", navigation_direct)
        self.assertIn("UiInputEvent::Navigation(_) | UiInputEvent::Analog(_)", source)
        self.assertIn("has_navigation_candidate()", source)
        self.assertLess(
            generic.index("input_requires_navigation_owner(&event)"),
            generic.index("for surface_index in (0..self.surfaces.len()).rev()"),
        )

    def test_current_source_evidence_captures_both_surface_fanout_paths(self):
        source = RUNTIME_UI.read_text(encoding="utf-8")
        generic = function_source(
            source,
            "    pub(super) fn dispatch_input(",
            "\n    pub(super) fn dispatch_pointer(",
        )
        pointer = function_source(
            source,
            "    fn dispatch_pointer_input(",
            "\n    fn dispatch_pointer_to_surface(",
        )
        pointer_target = function_source(
            source,
            "    fn dispatch_pointer_to_surface(",
            "\n    pub(super) fn next_input_metadata(",
        )

        self.assertIn("for surface_index in (0..self.surfaces.len()).rev()", generic)
        self.assertIn("runtime_surface.rebuild_dirty(root_size)?", generic)
        self.assertIn("for surface_index in (0..self.surfaces.len()).rev()", pointer)
        self.assertIn("runtime_surface.rebuild_dirty(root_size)?", pointer_target)
        self.assertIn("input_event_for_surface(", generic)
        self.assertIn("input_event_for_surface(", pointer)

    def test_uncaptured_pointer_uses_incremental_publication_before_legacy_fanout(self):
        runtime_ui = RUNTIME_UI.read_text(encoding="utf-8")
        publication = INPUT_PUBLICATION.read_text(encoding="utf-8")
        pointer = function_source(
            runtime_ui,
            "    fn dispatch_pointer_input(",
            "\n    fn dispatch_pointer_to_surface(",
        )

        self.assertIn(".query(viewport_size, point, previous_point)", pointer)
        self.assertIn("candidate_surface(query", pointer)
        self.assertLess(
            pointer.index(".query(viewport_size, point, previous_point)"),
            pointer.index("for surface_index in (0..self.surfaces.len()).rev()"),
        )
        for token in (
            "pub(super) struct RuntimeUiInputPublication",
            "surface_hit_generations: Vec<u64>",
            "cells: Vec<Vec<u32>>",
            "surface_footprints: Vec<Vec<u32>>",
            "pub(super) fn publish(",
            "pub(super) fn query(",
            "pub(super) fn candidate_surface(",
            "fn visit_bounded_cells(",
        ):
            self.assertIn(token, publication)
        self.assertNotIn("bounded_cells_for_frame", publication)
        self.assertNotIn("arranged_tree", publication)
        self.assertNotIn("render_extract", publication)
        self.assertNotIn("render commands", publication.lower())

    def test_publication_patch_reuses_occupancy_and_footprint_storage(self):
        publication = INPUT_PUBLICATION.read_text(encoding="utf-8")

        for token in (
            "cell_visit_stamps: Vec<u32>",
            "next_cell_visit_stamp: u32",
            "std::mem::take(&mut self.surface_footprints[surface_index])",
            "fn begin_cell_visit(&mut self) -> u32",
        ):
            self.assertIn(token, publication)
        self.assertNotIn("let mut occupied = vec![false; self.cells.len()]", publication)
        self.assertNotIn("footprint.sort", publication)
        self.assertNotIn("bounded_cells_for_frame", publication)

    def test_pointer_fallback_is_typed_and_invalid_input_never_fans_out(self):
        runtime_ui = RUNTIME_UI.read_text(encoding="utf-8")
        publication = INPUT_PUBLICATION.read_text(encoding="utf-8")
        pointer = function_source(
            runtime_ui,
            "    fn dispatch_pointer_input(",
            "\n    fn dispatch_pointer_to_surface(",
        )

        for token in (
            "pub(super) enum RuntimeUiInputQueryAdmission",
            "Published(RuntimeUiInputQuery)",
            "Unpublished",
            "Rejected(RuntimeUiInputQueryRejectReason)",
        ):
            self.assertIn(token, publication)
        self.assertIn("RuntimeUiInputQueryAdmission::Rejected", pointer)
        self.assertIn("ui.surface_set.input.invalid_pointer_reject_count", pointer)
        self.assertIn("RuntimeUiInputQueryAdmission::Unpublished", pointer)
        self.assertIn(
            "ui.surface_set.input.publication_unavailable_fallback_count", pointer
        )
        self.assertLess(
            pointer.index("RuntimeUiInputQueryAdmission::Rejected"),
            pointer.index("for surface_index in (0..self.surfaces.len()).rev()"),
        )

    def test_direct_focus_and_published_pointer_paths_do_not_rebuild_in_event_chain(self):
        source = RUNTIME_UI.read_text(encoding="utf-8")
        generic = function_source(
            source,
            "    pub(super) fn dispatch_input(",
            "\n    pub(super) fn dispatch_pointer(",
        )
        focus_direct = function_source(
            generic,
            "        if input_requires_focus_owner(&event)",
            "        let mut event = Some(event);",
        )
        pointer = function_source(
            source,
            "    fn dispatch_pointer_input(",
            "\n    fn dispatch_pointer_to_surface(",
        )

        self.assertIn("dispatch_input_to_surface(surface_index, root_size, event, false)", focus_direct)
        self.assertIn("dispatch_pointer_to_surface", pointer)
        self.assertIn("false,", pointer)

    def test_resize_pointer_preserves_physical_point_and_forwards_virtual_query(self):
        publication = INPUT_PUBLICATION.read_text(encoding="utf-8")
        runtime_ui = RUNTIME_UI.read_text(encoding="utf-8")
        manager = INPUT_MANAGER.read_text(encoding="utf-8")
        dispatch = INPUT_DISPATCH.read_text(encoding="utf-8")
        pointer = POINTER_INPUT.read_text(encoding="utf-8")
        event_routing = EVENT_ROUTING.read_text(encoding="utf-8")

        for token in (
            "physical_point: UiPoint",
            "virtual_pointer: Option<UiVirtualPointerPosition>",
            "UiHitTestQuery::new(self.physical_point)",
            ".with_virtual_pointer(virtual_pointer)",
            "map_pointer_axis(",
        ):
            self.assertIn(token, publication)
        self.assertIn("query.hit_test_query()", runtime_ui)
        self.assertIn("dispatch_input_event_with_query", manager)
        self.assertIn("pointer_query: Option<UiHitTestQuery>", dispatch)
        self.assertIn("pointer_query: Option<UiHitTestQuery>", pointer)
        self.assertIn("dispatch_pointer_event_with_query_and_modifiers", pointer)
        self.assertIn(
            "pub(crate) fn dispatch_pointer_event_with_query_and_modifiers",
            event_routing,
        )

    def test_unrouted_mouse_motion_and_duplicate_sync_are_bound_to_the_record(self):
        events = EVENTS.read_text(encoding="utf-8")
        mouse_motion = MOUSE_MOTION.read_text(encoding="utf-8")
        runtime_ui = RUNTIME_UI.read_text(encoding="utf-8")
        input_manager = INPUT_MANAGER.read_text(encoding="utf-8")

        self.assertIn("UiInputEvent::MouseMotion", events)
        self.assertIn("dispatch_runtime_ui_event(|metadata|", events)
        self.assertIn("UiDispatchReply::unhandled()", mouse_motion)
        self.assertIn("UiInputRoutePolicy::Unrouted", mouse_motion)
        self.assertIn("synchronize_text_document_owners(&mut self.surface)", runtime_ui)
        dispatch = function_source(
            input_manager,
            "    pub fn dispatch_input_event(",
            "\n    pub fn dispatch_window_input_pump_event(",
        )
        self.assertIn("self.synchronize_text_document_owners(surface);", dispatch)

    def test_record_is_bound_to_unreal_authority_and_executable_budgets(self):
        record = RECORD.read_text(encoding="utf-8")
        unreal_application = UNREAL_APPLICATION.read_text(
            encoding="utf-8", errors="replace"
        )
        unreal_hit_grid = UNREAL_HIT_GRID.read_text(
            encoding="utf-8", errors="replace"
        )

        for anchor in (
            "RuntimeUiInputPublication",
            "O(1 + C + sum(K_i))",
            "last completely published frame",
            "must not force a full Surface rebuild per native resize event",
            "virtual_pointer.current/previous",
            "stable event rebuild/tree/render scan count is zero",
            "candidate P95 is at most two",
            "E:\\zircon-profiles\\runtime-ui-surface-input-publication-20260901-r13.json",
            "0E09FD6F22F06833B2FBB7080E85C392F4AC42C592617EEDDA9DFBF1AE7264FD",
            "18FC9D1B746A54679AEB325B875C32E301E356DF294791FF426FA6937A090B13",
        ):
            self.assertIn(anchor, record)

        self.assertIn("LocateWindowUnderMouse", unreal_application)
        self.assertIn("LocateWidgetInWindow", unreal_application)
        self.assertIn("GetFocusPath", unreal_application)
        self.assertIn("GetHittestGrid", unreal_application)
        self.assertIn("FHittestGrid::AddWidget", unreal_hit_grid)
        self.assertIn("FHittestGrid::GetBubblePath", unreal_hit_grid)


if __name__ == "__main__":
    unittest.main()
