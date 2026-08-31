from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


class Editor04PlayInspectorDomainContractTests(unittest.TestCase):
    def test_host_owns_a_generation_qualified_play_inspector_projection(self) -> None:
        controller = read("zircon_editor/src/ui/host/editor_host_event_controller.rs")
        projection = read("zircon_editor/src/ui/host/play_inspector_projection.rs")

        self.assertIn("play_inspector_projection: Mutex<PlayInspectorProjection>", controller)
        self.assertIn("GatewaySessionIdentity", projection)
        self.assertIn("PLAY_INSPECTOR_QUERY_INTERVAL", projection)
        self.assertIn("WorldQueryResult::InspectionFields", projection)
        self.assertIn("WorldQueryResult::NotModified", projection)
        self.assertIn("editable: false", projection)

    def test_route_queries_only_the_attached_play_gateway_for_active_selection(self) -> None:
        route = read(
            "zircon_editor/src/ui/host/editor_host_event_controller/play_inspector.rs"
        )

        self.assertIn("WorldDomain::Play(instance)", route)
        self.assertIn("WorldQuery::inspection_fields", route)
        self.assertIn("query_world_at_identity", route)
        self.assertNotIn("EditorState.world", route)

    def test_editor_snapshot_replaces_authoring_inspector_while_playing(self) -> None:
        snapshot = read(
            "zircon_editor/src/ui/host/editor_event_runtime_access/snapshot.rs"
        )

        self.assertIn("active_hierarchy_world_domain", snapshot)
        self.assertIn("play_inspector_snapshot", snapshot)
        self.assertIn("snapshot.inspector = play_inspector", snapshot)

    def test_retained_tick_refreshes_play_inspector_after_hierarchy_sync(self) -> None:
        tick = read("zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs")

        hierarchy = tick.index("self.sync_active_hierarchy_world();")
        inspector = tick.index("self.sync_active_play_inspector();")
        self.assertLess(hierarchy, inspector)


if __name__ == "__main__":
    unittest.main()
