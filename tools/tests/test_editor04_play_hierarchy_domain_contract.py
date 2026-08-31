from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


class Editor04PlayHierarchyDomainContractTests(unittest.TestCase):
    def test_host_owns_independent_edit_and_play_world_sync_pumps(self) -> None:
        controller = read("zircon_editor/src/ui/host/editor_host_event_controller.rs")
        world_sync = read("zircon_editor/src/ui/host/editor_world_sync.rs")

        self.assertIn("edit_world_sync: Mutex<WorldSyncPump>", controller)
        self.assertIn("play_world_sync: Mutex<WorldSyncPump>", controller)
        self.assertIn("domain: WorldDomain", world_sync)
        self.assertIn("unwatch_world_for_view", world_sync)
        self.assertNotIn("watch_edit_world_for_view", world_sync)
        self.assertNotIn("pump_edit_world_invalidations", world_sync)

    def test_play_hierarchy_projection_queries_only_the_play_gateway(self) -> None:
        route = read(
            "zircon_editor/src/ui/host/editor_host_event_controller/play_hierarchy.rs"
        )
        projection = read("zircon_editor/src/ui/host/play_hierarchy_projection.rs")

        self.assertIn("WorldQuery::hierarchy", route)
        self.assertIn("WorldDomain::Play(instance)", route)
        self.assertIn("WorldQueryResult::HierarchyRows", projection)
        self.assertIn("GatewaySessionIdentity", projection)
        self.assertIn("SceneInspectionHierarchyFragment::patch", projection)
        self.assertIn("SceneInspectionHierarchyFragment::reflow", projection)
        self.assertNotIn("EditorState.world", route + projection)

    def test_retained_tick_ticks_runtime_before_domain_hierarchy_sync(self) -> None:
        tick = read("zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs")
        runtime_tick = tick.index("self.runtime.pump_runtime_event_consumers()")
        hierarchy_sync = tick.index("self.sync_active_hierarchy_world();")

        self.assertLess(runtime_tick, hierarchy_sync)
        self.assertNotIn("self.pump_edit_world_invalidations();", tick)

    def test_terminal_paths_retire_play_world_watches_before_backend_retirement(self) -> None:
        controller = read("zircon_editor/src/ui/host/editor_host_event_controller.rs")
        terminal = controller.index("self.shutdown_play_world_sync();")
        detach = controller.index("self.detach_terminal_play_gateway()")
        retire = controller.index(".retire_terminal_backend()")

        self.assertLess(terminal, detach)
        self.assertLess(detach, retire)

    def test_authoring_publication_is_not_used_as_play_hierarchy_data(self) -> None:
        publication = read("zircon_editor/src/ui/host/scene_inspection_publication.rs")

        self.assertIn("active_hierarchy_world_domain", publication)
        self.assertIn("WorldDomain::Play", publication)
        self.assertIn("play_hierarchy_projection", publication)

    def test_failed_play_projection_remains_retryable(self) -> None:
        watch = read(
            "zircon_editor/src/ui/retained_host/app/hierarchy_world_watch.rs"
        )
        refresh = read(
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/scene_hierarchy_refresh.rs"
        )

        self.assertIn("projection_pending: bool", watch)
        self.assertIn("mark_projection_pending", refresh)
        self.assertIn("complete_projection", refresh)
        query_match = refresh.split(".query_play_hierarchy_fragment", 1)[1].split(
            "pub(in crate::ui::retained_host::app) fn consume_scene_hierarchy_fragment",
            1,
        )[0]
        error_branch = query_match.split("Err(error) =>", 1)[1]
        self.assertNotIn("complete_projection", error_branch)


if __name__ == "__main__":
    unittest.main()
