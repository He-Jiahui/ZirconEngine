use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const TAGS_SEARCH_CONTROL: &str = "WorkbenchTagsSearchField";
const TAGS_TABLE_ROWS: &[&str] = &[
    "WorkbenchTagsAbilityActivateRow",
    "WorkbenchTagsStateStunnedRow",
];
static TAG_PROFILES: &[TagSelectionProfile] = &[
    TagSelectionProfile {
        action_id: "workbench.module.tags.ability_activate.select",
        row_control_id: "WorkbenchTagsAbilityActivateRow",
        tag: "Ability.Activate",
        redirect: "",
        owner: "DefaultGameplayTags.ini",
        validation: "Ability.Activate   valid   128 references",
    },
    TagSelectionProfile {
        action_id: "workbench.module.tags.state_stunned.select",
        row_control_id: "WorkbenchTagsStateStunnedRow",
        tag: "Character.State.Stunned",
        redirect: "Character.State.Stun",
        owner: "DefaultGameplayTags.ini",
        validation: "Character.State.Stunned   conflict   redirect required",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn initialize_tags_workspace_state(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.set_selected("WorkbenchTagsSourceRow", true)?;
        self.project_tag_profile(&TAG_PROFILES[0])
    }

    pub(super) fn apply_tags_workspace_action(
        &mut self,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        match action_id {
            "workbench.module.tags.source_default.select" => {
                self.set_selected("WorkbenchTagsSourceRow", true)?;
                self.set_tags_string(
                    "WorkbenchTagsValidationRow",
                    "value_text",
                    "DefaultGameplayTags.ini   2 tags   source loaded",
                )?;
                return Ok(true);
            }
            "workbench.module.tags.search.edit" | "workbench.module.tags.search.commit" => {
                self.apply_tag_search()?;
                return Ok(true);
            }
            "workbench.module.tags.add.invoke" => {
                self.apply_tag_command_feedback(TagCommand::Add)?;
                return Ok(true);
            }
            "workbench.module.tags.rename.invoke" => {
                self.apply_tag_command_feedback(TagCommand::Rename)?;
                return Ok(true);
            }
            _ => {}
        }

        let Some(profile) = TAG_PROFILES
            .iter()
            .find(|profile| profile.action_id == action_id)
        else {
            return Ok(false);
        };
        self.project_tag_profile(profile)?;
        Ok(true)
    }

    fn apply_tag_search(&mut self) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let query = self
            .control_string(TAGS_SEARCH_CONTROL, "value")
            .unwrap_or_default();
        let query = query.trim();
        let mut first_match = None;
        let mut selected_match = false;

        for profile in TAG_PROFILES {
            let matches = contains_ascii_case_insensitive(profile.tag, query);
            self.set_visible(profile.row_control_id, matches)?;
            if matches {
                first_match.get_or_insert(profile);
                selected_match |= self.control_bool(profile.row_control_id, "selected");
            }
        }

        if let Some(profile) = first_match.filter(|_| !selected_match) {
            self.project_tag_profile(profile)?;
        } else if first_match.is_none() {
            self.set_tags_string(
                "WorkbenchTagsValidationRow",
                "value_text",
                format!("No tags match '{query}'"),
            )?;
        }
        Ok(())
    }

    fn project_tag_profile(
        &mut self,
        profile: &TagSelectionProfile,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive_selected(TAGS_TABLE_ROWS, profile.row_control_id)?;
        for (control_id, property, value) in [
            (
                "WorkbenchTagsCenterTitle",
                "text",
                format!("Tag Registry / {}", profile.tag),
            ),
            (
                "WorkbenchTagsRedirectField",
                "value",
                profile.redirect.to_string(),
            ),
            (
                "WorkbenchTagsOwnerField",
                "value",
                profile.owner.to_string(),
            ),
            (
                "WorkbenchTagsValidationRow",
                "value_text",
                profile.validation.to_string(),
            ),
        ] {
            self.set_tags_string(control_id, property, value)?;
        }
        Ok(())
    }

    fn apply_tag_command_feedback(
        &mut self,
        command: TagCommand,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let profile = TAG_PROFILES
            .iter()
            .find(|profile| self.control_bool(profile.row_control_id, "selected"))
            .unwrap_or(&TAG_PROFILES[0]);
        let owner = self
            .control_string("WorkbenchTagsOwnerField", "value")
            .unwrap_or_default();
        let redirect = self
            .control_string("WorkbenchTagsRedirectField", "value")
            .unwrap_or_default();
        let (status, title, output) = match command {
            TagCommand::Add => (
                "Tag add dialog prepared",
                format!("Tag Registry / {}", profile.tag),
                format!("Add Tag   {owner}"),
            ),
            TagCommand::Rename => (
                "Tag rename prepared",
                format!("Tag Registry / {redirect}"),
                format!("Rename ready   {owner}"),
            ),
        };
        self.set_tags_string("WorkbenchStatusReady", "text", status)?;
        self.set_tags_string("WorkbenchStatusMessages", "text", "1 Message")?;
        self.set_tags_string("WorkbenchTagsCenterTitle", "text", title)?;
        self.set_tags_string("WorkbenchTagsValidationRow", "value_text", output)
    }

    fn set_tags_string(
        &mut self,
        control_id: &str,
        property: &str,
        value: impl Into<String>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.mutate_control_property(control_id, property, UiValue::String(value.into()))
    }
}

fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
    needle.is_empty()
        || haystack
            .as_bytes()
            .windows(needle.len())
            .any(|candidate| candidate.eq_ignore_ascii_case(needle.as_bytes()))
}

struct TagSelectionProfile {
    action_id: &'static str,
    row_control_id: &'static str,
    tag: &'static str,
    redirect: &'static str,
    owner: &'static str,
    validation: &'static str,
}

enum TagCommand {
    Add,
    Rename,
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

    use super::*;

    #[test]
    fn source_search_selection_and_rename_keep_distinct_state_domains() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
                .expect("workbench bridge should build");

        assert!(bridge.control_bool("WorkbenchTagsSourceRow", "selected"));
        assert!(bridge.control_bool("WorkbenchTagsAbilityActivateRow", "selected"));

        bridge
            .dispatch_control_state("WorkbenchTagsStateStunnedRow", UiEventKind::Click)
            .expect("stunned tag should dispatch")
            .expect("stunned tag should bind");
        assert!(bridge.control_bool("WorkbenchTagsSourceRow", "selected"));
        assert!(bridge.control_bool("WorkbenchTagsStateStunnedRow", "selected"));
        assert_eq!(
            Some("Character.State.Stun".to_string()),
            bridge.control_string("WorkbenchTagsRedirectField", "value")
        );
        for (control_id, value) in [
            ("WorkbenchTagsRedirectField", "Character.State.Disabled"),
            ("WorkbenchTagsOwnerField", "CustomGameplayTags.ini"),
        ] {
            bridge
                .mutate_control_property(control_id, "value", UiValue::String(value.to_string()))
                .expect("tag property should edit");
        }

        bridge
            .dispatch_control_state("WorkbenchTagsRenameButton", UiEventKind::Click)
            .expect("rename should dispatch")
            .expect("rename should bind");
        assert_eq!(
            Some("Rename ready   CustomGameplayTags.ini".to_string()),
            bridge.control_string("WorkbenchTagsValidationRow", "value_text")
        );
        assert_eq!(
            Some("Tag Registry / Character.State.Disabled".to_string()),
            bridge.control_string("WorkbenchTagsCenterTitle", "text")
        );

        bridge
            .mutate_control_property(
                TAGS_SEARCH_CONTROL,
                "value",
                UiValue::String("ability".to_string()),
            )
            .expect("tag search value should update");
        bridge
            .dispatch_control_state(TAGS_SEARCH_CONTROL, UiEventKind::Change)
            .expect("tag search should dispatch")
            .expect("tag search should bind");
        assert!(bridge
            .control_frame("WorkbenchTagsAbilityActivateRow")
            .is_some());
        assert!(bridge
            .control_frame("WorkbenchTagsStateStunnedRow")
            .is_none());
        assert!(bridge.control_bool("WorkbenchTagsAbilityActivateRow", "selected"));
    }
}
