import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class EditorSettingsWindowMutationWiringContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_settings_actions_have_one_shared_identity_owner(self) -> None:
        actions = self.read("zircon_editor/src/ui/settings/action_ids.rs")
        module = self.read("zircon_editor/src/ui/settings/mod.rs")

        self.assertIn("SETTINGS_TOGGLE_BOOL_ACTION_ID", actions)
        self.assertIn("SETTINGS_RESET_OVERRIDE_ACTION_ID", actions)
        self.assertIn("SETTINGS_DECREMENT_NUMBER_ACTION_ID", actions)
        self.assertIn("SETTINGS_INCREMENT_NUMBER_ACTION_ID", actions)
        self.assertIn("SETTINGS_OPEN_ENUM_ACTION_ID", actions)
        self.assertIn("SETTINGS_SELECT_ENUM_ACTION_ID", actions)
        self.assertIn("SETTINGS_CAPTURE_CHORD_ACTION_ID", actions)
        self.assertIn("SETTINGS_COMMIT_CHORD_ACTION_ID", actions)
        self.assertIn("mod action_ids;", module)
        self.assertIn("pub(crate) use action_ids::", module)

    def test_dynamic_setting_rows_are_hit_tested_without_a_linear_scan(self) -> None:
        hit = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "surface_hit_test/template_node/popup_rows/settings.rs"
        )

        self.assertIn("node.settings_entries.get(row)", hit)
        self.assertIn("SETTINGS_TOGGLE_BOOL_ACTION_ID", hit)
        self.assertIn("SETTINGS_RESET_OVERRIDE_ACTION_ID", hit)
        self.assertIn("SETTINGS_DECREMENT_NUMBER_ACTION_ID", hit)
        self.assertIn("SETTINGS_INCREMENT_NUMBER_ACTION_ID", hit)
        self.assertIn("SETTINGS_OPEN_ENUM_ACTION_ID", hit)
        self.assertIn("SETTINGS_SELECT_ENUM_ACTION_ID", hit)
        self.assertIn("entry.options.get(row)", hit)
        self.assertIn("return Some(TemplatePopupRowTarget::Blocked);", hit)
        self.assertNotIn("node.settings_entries.iter()", hit)

    def test_host_runtime_routes_mutation_through_the_context_coordinator(self) -> None:
        mutation = self.read(
            "zircon_editor/src/ui/host/editor_event_runtime_access/settings_mutation.rs"
        )
        compact = "".join(mutation.split())

        self.assertIn("toggle_bool_setting", mutation)
        self.assertIn("step_numeric_setting", mutation)
        self.assertIn("set_enum_setting", mutation)
        self.assertIn("set_chord_setting", mutation)
        self.assertIn("reset_setting_override", mutation)
        self.assertIn("settings_mutations().set", compact)
        self.assertIn("settings_mutations().clear", compact)
        self.assertNotIn("SettingsPersistenceService", mutation)
        self.assertNotIn("SettingsStore", mutation)

    def test_workbench_dispatch_applies_actions_and_refreshes_the_current_batch(self) -> None:
        dispatch = self.read(
            "zircon_editor/src/ui/retained_host/app/pane_surface_actions/"
            "workbench_surface/option.rs"
        )

        self.assertIn("SETTINGS_TOGGLE_BOOL_ACTION_ID", dispatch)
        self.assertIn("SETTINGS_RESET_OVERRIDE_ACTION_ID", dispatch)
        self.assertIn("SETTINGS_DECREMENT_NUMBER_ACTION_ID", dispatch)
        self.assertIn("SETTINGS_INCREMENT_NUMBER_ACTION_ID", dispatch)
        self.assertIn("SETTINGS_OPEN_ENUM_ACTION_ID", dispatch)
        self.assertIn("SETTINGS_SELECT_ENUM_ACTION_ID", dispatch)
        self.assertIn("refresh_settings_values", dispatch)
        self.assertIn("refresh_settings_values_and_close_editor(&values)", dispatch)
        self.assertIn("receipt.changed()", dispatch)
        self.assertNotIn("_action_id", dispatch)

    def test_paint_projects_a_checkbox_and_reset_icon_for_supported_rows(self) -> None:
        paint = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_settings_window/commands.rs"
        )
        enum_controls = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_settings_window/enum_controls.rs"
        )

        self.assertIn("push_bool_control", paint)
        self.assertIn("push_numeric_stepper", paint)
        self.assertIn("push_enum_control", enum_controls)
        self.assertIn("push_enum_popup", enum_controls)
        self.assertIn("push_reset_control", paint)
        self.assertIn('"checkmark"', paint)
        self.assertIn('"reset"', paint)

    def test_settings_paint_owners_stay_below_the_production_soft_budget(self) -> None:
        root = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_settings_window.rs"
        )
        commands = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_settings_window/commands.rs"
        )
        enum_controls = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_settings_window/enum_controls.rs"
        )

        self.assertIn("mod enum_controls;", root)
        self.assertLessEqual(len(commands.splitlines()), 800)
        self.assertLessEqual(len(enum_controls.splitlines()), 800)

    def test_numeric_step_is_schema_owned_instead_of_key_switched_in_ui(self) -> None:
        definition = self.read("zircon_editor/src/core/settings/definition.rs")
        mutation = self.read(
            "zircon_editor/src/ui/host/editor_event_runtime_access/settings_mutation.rs"
        )

        self.assertRegex(
            definition,
            r"(?s)Int\s*\{\s*minimum: i64,\s*maximum: i64,\s*step: i64,\s*\}",
        )
        self.assertRegex(
            definition,
            r"(?s)Float\s*\{\s*minimum: f64,\s*maximum: f64,\s*step: f64,\s*\}",
        )
        self.assertIn("stepped_numeric_value", definition)
        self.assertNotIn("VIEWPORT_TRANSLATE_STEP_KEY", mutation)
        self.assertNotIn("EDITOR_AUTOSAVE_INTERVAL_SECS_KEY", mutation)

    def test_enum_variants_and_editor_state_have_explicit_projection_owners(self) -> None:
        bridge = self.read(
            "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
            "workbench/settings_window.rs"
        )
        payload = self.read(
            "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
            "workbench/settings_window.rs"
        )
        projection = self.read(
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
            "pane_component_projection/settings_window/mod.rs"
        )
        zui = self.read(
            "zircon_editor/assets/ui/editor/components/workbench/floating/"
            "workbench_preferences.zui"
        )

        self.assertIn("SETTINGS_EDITOR_OPEN_KEY", bridge)
        self.assertIn("SETTINGS_EDITOR_OPEN_KIND", bridge)
        self.assertIn("refresh_settings_values_and_close_editor", bridge)
        self.assertIn("settings_editor_open_key", zui)
        self.assertIn("settings_editor_open_kind", zui)
        self.assertIn('("options", setting_options', payload)
        self.assertIn("options: string_array", projection)

    def test_string_rows_reuse_commit_only_host_text_input(self) -> None:
        actions = self.read("zircon_editor/src/ui/settings/action_ids.rs")
        mutation = self.read(
            "zircon_editor/src/ui/host/editor_event_runtime_access/settings_mutation.rs"
        )
        hit = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "surface_hit_test/template_node/popup_rows/settings.rs"
        )
        target = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "surface_hit_test/template_node/popup_rows/hit.rs"
        )
        dispatch = self.read(
            "zircon_editor/src/ui/retained_host/app/pane_surface_actions/"
            "workbench_surface/edit.rs"
        )
        paint_root = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_settings_window.rs"
        )
        paint = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_settings_window/text_control.rs"
        )
        paint_commands = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_settings_window/commands.rs"
        )

        self.assertIn("SETTINGS_EDIT_STRING_ACTION_ID", actions)
        self.assertIn("SETTINGS_COMMIT_STRING_ACTION_ID", actions)
        self.assertIn("set_string_setting", mutation)
        self.assertIn("SettingSchema::String", mutation)
        self.assertIn("TemplatePopupRowTarget::TextInput", hit)
        self.assertIn("entry.value_text.as_str()", hit)
        self.assertIn('dispatch_kind: "commit_only".into()', target)
        self.assertIn("TemplateComponentFamily::TextInput", target)
        self.assertIn("SETTINGS_COMMIT_STRING_ACTION_ID", dispatch)
        self.assertIn("set_string_setting(control_id, value)", dispatch)
        self.assertIn("mod text_control;", paint_root)
        self.assertIn("focus.value_text", paint)
        self.assertIn("focus.control_id", paint)
        self.assertRegex(
            paint_commands,
            r"(?s)fn push_setting\([^)]*text_input_focus: "
            r"Option<&HostTextInputFocusData>[^)]*\)",
        )
        self.assertNotRegex(
            paint_commands,
            r"(?s)fn push_panel\([^)]*text_input_focus",
        )

    def test_chord_rows_use_typed_exclusive_key_capture(self) -> None:
        definition = self.read("zircon_editor/src/core/settings/definition.rs")
        actions = self.read("zircon_editor/src/ui/settings/action_ids.rs")
        mutation = self.read(
            "zircon_editor/src/ui/host/editor_event_runtime_access/settings_mutation.rs"
        )
        hit = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "surface_hit_test/template_node/popup_rows/settings.rs"
        )
        target = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "surface_hit_test/template_node/popup_rows/hit.rs"
        )
        keyboard = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/window/"
            "text_input/keyboard.rs"
        )
        focus = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/data/"
            "host_interaction/text_focus.rs"
        )
        event_loop_input = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/window/"
            "event_loop/input.rs"
        )
        paint_root = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_settings_window.rs"
        )
        paint = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_settings_window/chord_control.rs"
        )
        dispatch = self.read(
            "zircon_editor/src/ui/retained_host/app/pane_surface_actions/"
            "workbench_surface/edit.rs"
        )

        self.assertIn("Chord(EditorKeyChord)", definition)
        self.assertNotIn("Chord(String)", definition)
        self.assertIn("value.is_valid()", definition)
        self.assertIn("SETTINGS_CAPTURE_CHORD_ACTION_ID", actions)
        self.assertIn("SETTINGS_COMMIT_CHORD_ACTION_ID", actions)
        self.assertIn("set_chord_setting", mutation)
        self.assertIn("value.parse::<EditorKeyChord>()", mutation)
        self.assertIn("TemplatePopupRowTarget::ChordInput", hit)
        self.assertIn('dispatch_kind: "chord_capture".into()', target)
        self.assertIn("chord_capture_focus_active", keyboard)
        self.assertIn("EditorKeyChord::from_keyboard_input", keyboard)
        self.assertIn("dispatch_focused_chord_commit", keyboard)
        self.assertLess(
            keyboard.index("if self.chord_capture_focus_active()"),
            keyboard.index("let text_focus_was_active"),
        )
        self.assertIn("captures_keyboard_chord", focus)
        self.assertIn("text_input_focus_accepts_text", event_loop_input)
        self.assertIn("mod chord_control;", paint_root)
        self.assertIn("captures_keyboard_chord", paint)
        self.assertIn("SETTINGS_COMMIT_CHORD_ACTION_ID", dispatch)
        self.assertIn("set_chord_setting(control_id, value)", dispatch)

    def test_enum_and_color_share_one_active_settings_editor_state(self) -> None:
        bridge = self.read(
            "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
            "workbench/settings_window.rs"
        )
        projection = self.read(
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
            "pane_component_projection/settings_window/mod.rs"
        )
        node = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/data/"
            "template_nodes/node.rs"
        )
        zui = self.read(
            "zircon_editor/assets/ui/editor/components/workbench/floating/"
            "workbench_preferences.zui"
        )
        option = self.read(
            "zircon_editor/src/ui/retained_host/app/pane_surface_actions/"
            "workbench_surface/option.rs"
        )
        combined = bridge + projection + node + zui + option

        self.assertIn("SETTINGS_EDITOR_OPEN_KEY", bridge)
        self.assertIn("SETTINGS_EDITOR_OPEN_KIND", bridge)
        self.assertIn("toggle_settings_editor", bridge)
        self.assertIn("refresh_settings_values_and_close_editor", bridge)
        self.assertIn("settings_editor_open_key", combined)
        self.assertIn("settings_editor_open_kind", combined)
        self.assertIn("settings_editor_open_row", combined)
        self.assertNotIn("settings_enum_open_key", combined)
        self.assertNotIn("settings_enum_open_row", combined)
        self.assertNotIn("toggle_settings_enum", combined)
        self.assertNotIn("close_settings_enum", combined)

    def test_color_rows_use_swatch_and_schema_owned_rgba_steppers(self) -> None:
        definition = self.read("zircon_editor/src/core/settings/definition.rs")
        actions = self.read("zircon_editor/src/ui/settings/action_ids.rs")
        mutation = self.read(
            "zircon_editor/src/ui/host/editor_event_runtime_access/settings_mutation.rs"
        )
        hit = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "surface_hit_test/template_node/popup_rows/settings.rs"
        )
        paint_root = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_settings_window.rs"
        )
        paint = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_settings_window/color_controls.rs"
        )

        self.assertRegex(
            definition,
            r"(?s)Color\s*\{\s*channel_step: u8,\s*\}",
        )
        self.assertIn("SettingColorChannel", definition)
        self.assertIn("stepped_color_value", definition)
        self.assertIn("SETTINGS_OPEN_COLOR_ACTION_ID", actions)
        self.assertIn("SETTINGS_DECREMENT_COLOR_RED_ACTION_ID", actions)
        self.assertIn("SETTINGS_INCREMENT_COLOR_ALPHA_ACTION_ID", actions)
        self.assertIn("step_color_setting", mutation)
        self.assertIn("TemplatePopupRowTarget::Hit", hit)
        self.assertIn("hit_test_open_color", hit)
        self.assertIn("mod color_controls;", paint_root)
        self.assertIn("push_color_swatch", paint)
        self.assertIn("push_color_popup", paint)
        self.assertIn("push_alpha_checkerboard", paint)


if __name__ == "__main__":
    unittest.main()
