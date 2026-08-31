from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


class Editor04PlaySelectionDomainContractTests(unittest.TestCase):
    def test_selection_reuses_the_instance_qualified_core_world_domain(self) -> None:
        selection_mod = read("zircon_editor/src/scene/selection/mod.rs")
        command_when = read("zircon_editor/src/core/commands/when.rs")

        self.assertNotIn("mod world_domain;", selection_mod)
        self.assertIn("pub use crate::core::play::WorldDomain;", selection_mod)
        self.assertIn("use crate::core::play::WorldDomain;", command_when)
        self.assertNotIn("use crate::scene::selection::WorldDomain;", command_when)

    def test_play_selection_is_partitioned_by_play_instance_identity(self) -> None:
        selection = read(
            "zircon_editor/src/scene/selection/selection_model.rs"
        )

        self.assertIn("BTreeMap<PlayInstanceId, DomainSelection>", selection)
        self.assertIn("WorldDomain::Play(instance)", selection)
        self.assertIn("activate_play_domain", selection)
        self.assertIn("retire_play_domain", selection)
        self.assertNotIn("play: DomainSelection", selection)

    def test_runtime_attachment_selects_the_matching_play_domain(self) -> None:
        state = read(
            "zircon_editor/src/ui/workbench/state/editor_state_play_mode.rs"
        )
        menu = read(
            "zircon_editor/src/ui/host/editor_event_execution/menu_action.rs"
        )

        self.assertIn("activate_play_selection_domain", state)
        self.assertIn("attached_world_domain", menu)
        self.assertIn("activate_play_selection_domain", menu)
        self.assertLess(
            menu.index("request_play("),
            menu.index("activate_play_selection_domain"),
        )

        tick = read(
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs"
        )
        selection = tick.index("sync_active_selection_world_domain")
        hierarchy = tick.index("sync_active_hierarchy_world")
        self.assertLess(selection, hierarchy)

    def test_legacy_unqualified_selection_world_domain_owner_is_deleted(self) -> None:
        self.assertFalse(
            (
                ROOT
                / "zircon_editor/src/scene/selection/world_domain.rs"
            ).exists()
        )


if __name__ == "__main__":
    unittest.main()
