from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HOST = ROOT / "zircon_editor/src/ui/retained_host/host_contract"
NATIVE_POINTER = HOST / "native_pointer"
PANE_ROUTE = NATIVE_POINTER / "routing/pane_route"
PANES = NATIVE_POINTER / "routing/panes"
SURFACE_HIT_TEST = HOST / "surface_hit_test"


class EditorPaneRouteBorrowedHitPerformanceContractTests(unittest.TestCase):
    def test_pane_target_is_generation_borrowed_and_uses_typed_asset_tags(self) -> None:
        target = (PANE_ROUTE / "target.rs").read_text(encoding="utf-8")
        routing = (NATIVE_POINTER / "routing.rs").read_text(encoding="utf-8")

        self.assertIn("enum PanePointerTarget<'a>", target)
        self.assertIn("enum PaneAssetSurface", target)
        self.assertIn("enum PaneAssetReferenceList", target)
        self.assertIn("TemplateNodePointerRouteHit<'a>", target)
        self.assertIn("surface_key: &'a str", target)
        self.assertIn("control_id: Option<&'a str>", target)
        self.assertNotIn("SharedString", target)
        self.assertNotIn("TemplateNodePointerHit)", target)
        self.assertIn("PaneAssetReferenceList", routing)
        self.assertIn("PaneAssetSurface", routing)

    def test_route_geometry_is_shared_by_move_click_and_scroll(self) -> None:
        route = (PANES / "entry/route.rs").read_text(encoding="utf-8")

        self.assertIn("fn route_pointer_to_pane_with_mode<'a>", route)
        self.assertEqual(route.count("route_pointer_to_pane_with_mode("), 3)
        self.assertNotIn("route_pointer_move_to_pane_with_mode", route)
        self.assertNotIn("route_pointer_scroll_to_pane_with_mode", route)

    def test_pane_template_route_never_materializes_owned_hit(self) -> None:
        template_route = (PANES / "pane/entry/template.rs").read_text(encoding="utf-8")
        move_dispatch = (
            NATIVE_POINTER / "move_dispatch/pane/template.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("hit_test_pane_template_node_borrowed", template_route)
        self.assertIn("TemplateNodePointerRouteHit", move_dispatch)
        self.assertNotIn("hit_test_pane_template_node(", template_route)
        self.assertNotIn("TemplateNodePointerHit", move_dispatch)

    def test_owned_template_payload_is_materialized_only_at_activation_boundary(self) -> None:
        template_route = (PANES / "pane/entry/template.rs").read_text(encoding="utf-8")
        target = (
            NATIVE_POINTER / "button_dispatch/pane_callbacks/target/template.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("to_owned_hit", template_route)
        self.assertEqual(target.count("to_owned_hit()"), 1)

    def test_dispatch_input_names_the_route_generation_lifetime(self) -> None:
        pane_input = (
            NATIVE_POINTER
            / "button_dispatch/pane_callbacks/target/entry/input.rs"
        ).read_text(encoding="utf-8")
        result_input = (
            NATIVE_POINTER
            / "button_dispatch/pane_callbacks/target/entry/result_targets/input.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("PanePointerRoute<'a>", pane_input)
        self.assertIn("PanePointerRoute<'generation>", result_input)

    def test_target_producers_do_not_allocate_route_identity_strings(self) -> None:
        sources = "\n".join(
            path.read_text(encoding="utf-8")
            for path in sorted(PANES.rglob("*.rs"))
            if "tests" not in path.parts
        )

        self.assertNotIn('PanePointerTarget::AssetTree("browser".into())', sources)
        self.assertNotIn("surface_key: surface_key.into()", sources)
        self.assertNotIn("PanePointerTarget::Viewport(surface_key.into())", sources)

    def test_dispatch_consumers_materialize_tags_only_at_callback_boundary(self) -> None:
        dispatch_roots = [
            NATIVE_POINTER / "button_dispatch/pane_callbacks/asset_panes",
            NATIVE_POINTER / "move_dispatch/pane/asset.rs",
            NATIVE_POINTER / "scroll_dispatch/pane/asset",
        ]
        sources = "\n".join(
            path.read_text(encoding="utf-8")
            for root in dispatch_roots
            for path in ([root] if root.is_file() else sorted(root.rglob("*.rs")))
        )
        viewport_click = (
            NATIVE_POINTER
            / "button_dispatch/pane_callbacks/viewport/toolbar/click.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("mode.clone()", sources)
        self.assertNotIn("list_kind.clone()", sources)
        self.assertIn("mode.as_str().into()", sources)
        self.assertIn("list_kind.as_str().into()", sources)
        self.assertIn("surface_key.into()", viewport_click)
        self.assertNotIn("surface_key.clone()", viewport_click)

    def test_lower_regression_proves_generation_owned_pane_route_identity(self) -> None:
        tests = (PANES / "entry/route/tests.rs").read_text(encoding="utf-8")

        self.assertIn(
            "pane_pointer_route_borrows_generation_owned_targets_and_materializes_only_for_activation",
            tests,
        )
        self.assertGreaterEqual(tests.count("std::ptr::eq("), 4)


if __name__ == "__main__":
    unittest.main()
