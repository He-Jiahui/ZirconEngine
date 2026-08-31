from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
ROUTES = (
    REPO_ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes"
)
ROUTES_ROOT = ROUTES.with_suffix(".rs")


class EditorProfileHitRoutePerformanceContract(unittest.TestCase):
    def test_route_decisions_do_not_format_control_identities(self) -> None:
        sources = "\n".join(
            path.read_text(encoding="utf-8")
            for path in sorted(ROUTES.rglob("*.rs"))
        )
        root = ROUTES_ROOT.read_text(encoding="utf-8")

        self.assertNotIn("format!(", sources)
        self.assertIn("mod identity;", root)
        self.assertEqual(sources.count("profile_control_id("), 3)

    def test_tab_id_is_checked_before_frame_translation(self) -> None:
        source = (ROUTES / "tabs/shared.rs").read_text(encoding="utf-8")
        body = source.split("fn tab_route_hit", 1)[1]

        id_check = body.index("if tab.control_id.as_str() != id")
        frame_translation = body.index("translated(&tab.frame")

        self.assertLess(id_check, frame_translation)
        self.assertNotIn("tab.frame.clone()", body)

    def test_profile_identity_is_parsed_without_allocation(self) -> None:
        identity = ROUTES / "identity.rs"
        self.assertTrue(identity.is_file())
        source = identity.read_text(encoding="utf-8")

        self.assertIn("fn profile_control_id", source)
        self.assertGreaterEqual(source.count("strip_prefix"), 4)
        self.assertNotIn("String", source)
        self.assertNotIn("format!(", source)


if __name__ == "__main__":
    unittest.main()
