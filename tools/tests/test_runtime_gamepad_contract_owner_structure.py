import unittest
from pathlib import Path


class RuntimeGamepadContractOwnerStructureTests(unittest.TestCase):
    def test_gamepad_contract_domains_are_folder_backed(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = repo_root / "zircon_runtime/src/core/framework/input/gamepad.rs"
        owner = owner_path.read_text(encoding="utf-8")
        owner_dir = owner_path.with_suffix("")

        domains = {
            name: (owner_dir / name / "mod.rs").read_text(encoding="utf-8")
            for name in ("axis", "button", "device", "rumble")
        }

        self.assertLessEqual(len(owner.splitlines()), 20)
        for name in domains:
            self.assertIn(f"mod {name};", owner)
            self.assertIn(f"    {name}::{{", owner)

        for moved_anchor in (
            "pub struct GamepadId",
            "pub enum GamepadAxis",
            "pub enum GamepadButton",
            "pub struct GamepadAxisSettings",
            "pub struct GamepadButtonSettings",
            "pub enum GamepadRumbleRequest",
        ):
            self.assertNotIn(moved_anchor, owner)

        expected_files = {
            "device": ("id.rs", "connection_info.rs"),
            "axis": (
                "axis.rs",
                "input.rs",
                "settings.rs",
                "state.rs",
                "transition.rs",
            ),
            "button": (
                "button.rs",
                "axis_settings.rs",
                "settings.rs",
                "value_state.rs",
            ),
            "rumble": ("intensity.rs", "request.rs"),
        }
        for domain, files in expected_files.items():
            domain_root = domains[domain]
            self.assertLessEqual(len(domain_root.splitlines()), 25)
            for filename in files:
                module = Path(filename).stem
                self.assertIn(f"mod {module};", domain_root)
                self.assertTrue((owner_dir / domain / filename).is_file())

        axis_settings = (owner_dir / "axis/settings.rs").read_text(encoding="utf-8")
        button_settings = (owner_dir / "button/settings.rs").read_text(
            encoding="utf-8"
        )
        button_axis_settings = (owner_dir / "button/axis_settings.rs").read_text(
            encoding="utf-8"
        )
        rumble_intensity = (owner_dir / "rumble/intensity.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("pub struct GamepadAxisSettings", axis_settings)
        self.assertIn("pub fn process_value", axis_settings)
        self.assertIn("pub struct GamepadButtonSettings", button_settings)
        self.assertIn("pub fn transition_for_value", button_settings)
        self.assertIn("pub struct GamepadButtonAxisSettings", button_axis_settings)
        self.assertIn("pub fn process_value", button_axis_settings)
        self.assertIn("pub struct GamepadRumbleIntensity", rumble_intensity)
        self.assertIn("pub fn clamped", rumble_intensity)

    def test_gamepad_child_owners_are_in_runtime_12_inventory(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        inventory = (
            repo_root
            / ".codex/skills/zircon-project-skills/"
            "zr-runtime-interface-convergence/scripts/runtime_structure_audits/"
            "input_stack_source_inventory.py"
        ).read_text(encoding="utf-8")
        owner_dir = repo_root / "zircon_runtime/src/core/framework/input/gamepad"

        child_paths = sorted(
            path.relative_to(repo_root).as_posix()
            for path in owner_dir.rglob("*.rs")
        )
        self.assertEqual(len(child_paths), 18)
        for path in child_paths:
            self.assertIn(f'    "{path}",', inventory)


if __name__ == "__main__":
    unittest.main()
