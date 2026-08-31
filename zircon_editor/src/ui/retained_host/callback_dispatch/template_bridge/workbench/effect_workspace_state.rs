use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const EFFECT_SEARCH_CONTROL: &str = "WorkbenchEffectAssetSearch";
const EFFECT_SEARCH_EMPTY_CONTROL: &str = "WorkbenchEffectSearchEmpty";
const EFFECT_ASSET_ROW_CONTROLS: &[&str] = &[
    "WorkbenchEffectHealthRegenRow",
    "WorkbenchEffectDamageFireRow",
];
const EFFECT_MODIFIER_ROWS: &[&str] = &[
    "WorkbenchEffectModifierHealthRow",
    "WorkbenchEffectModifierHealingRow",
    "WorkbenchEffectModifierCapRow",
];
const EFFECT_GRAPH_ROWS: &[&str] = &["WorkbenchEffectGraphRow"];
const EFFECT_PREVIEW_ROWS: &[&str] = &["WorkbenchEffectAttributePreviewRow"];
const EFFECT_PROFILES: &[EffectAssetProfile] = &[
    EffectAssetProfile {
        action_id: "workbench.module.effect.health_regen_row.select",
        row_control_id: "WorkbenchEffectHealthRegenRow",
        asset_name: "GE_HealthRegen",
        tag: "Effect.Health.Regen",
        magnitude: "10.0",
        duration: "10.0 s",
        period: "1.0 s",
        modifier_text: "HealthRegen     Health        Additive       Scalable Float",
        graph_text: "Source HealthRegen  ->  Health Attribute  ->  Clamp Health",
        preview_text: "Preview Level 1       Health +10.0",
        simulation_output: "Simulation Output: +50 health over 10 seconds",
    },
    EffectAssetProfile {
        action_id: "workbench.module.effect.damage_fire_row.select",
        row_control_id: "WorkbenchEffectDamageFireRow",
        asset_name: "GE_DamageFire",
        tag: "Effect.Damage.Fire",
        magnitude: "25.0",
        duration: "5.0 s",
        period: "0.5 s",
        modifier_text: "FireDamage      Health        Additive       Scalable Float",
        graph_text: "Source FireDamage  ->  Health Attribute  ->  Apply Damage",
        preview_text: "Preview Level 1       Health -25.0",
        simulation_output: "Simulation Output: 25 fire damage over 5 seconds",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn initialize_effect_workspace_state(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.project_effect_asset(&EFFECT_PROFILES[0])?;
        self.select_exclusive_selected(EFFECT_MODIFIER_ROWS, "WorkbenchEffectModifierHealthRow")?;
        self.select_exclusive_selected(EFFECT_GRAPH_ROWS, "WorkbenchEffectGraphRow")?;
        self.select_exclusive_selected(EFFECT_PREVIEW_ROWS, "WorkbenchEffectAttributePreviewRow")
    }

    pub(super) fn apply_effect_workspace_action(
        &mut self,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if matches!(
            action_id,
            "workbench.module.effect.search.edit" | "workbench.module.effect.search.commit"
        ) {
            self.apply_effect_search()?;
            return Ok(true);
        }
        if action_id == "workbench.module.effect.apply.invoke" {
            self.apply_effect_feedback()?;
            return Ok(true);
        }
        if let Some((rows, control_id, summary)) = effect_detail_selection(action_id) {
            self.select_exclusive_selected(rows, control_id)?;
            self.set_effect_string(
                "WorkbenchEffectOutputRow",
                "text",
                format!("Selection: {summary}"),
            )?;
            return Ok(true);
        }
        let Some(profile) = EFFECT_PROFILES
            .iter()
            .find(|profile| profile.action_id == action_id)
        else {
            return Ok(false);
        };
        self.project_effect_asset(profile)?;
        Ok(true)
    }

    fn apply_effect_search(&mut self) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let query = self
            .control_string(EFFECT_SEARCH_CONTROL, "query")
            .unwrap_or_default();
        let query = query.trim();
        let mut first_match = None;
        let mut selected_match = false;

        for profile in EFFECT_PROFILES {
            let matches = contains_ascii_case_insensitive(profile.asset_name, query);
            self.set_visible(profile.row_control_id, matches)?;
            if matches {
                first_match.get_or_insert(profile);
                selected_match |= self.control_bool(profile.row_control_id, "selected");
            }
        }

        self.set_visible(EFFECT_SEARCH_EMPTY_CONTROL, first_match.is_none())?;
        if let Some(profile) = first_match.filter(|_| !selected_match) {
            self.project_effect_asset(profile)?;
        }
        Ok(())
    }

    fn project_effect_asset(
        &mut self,
        profile: &EffectAssetProfile,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive_selected(EFFECT_ASSET_ROW_CONTROLS, profile.row_control_id)?;
        for (control_id, property, value) in [
            ("WorkbenchEffectCenterTitle", "text", profile.asset_name),
            ("WorkbenchEffectNameField", "value", profile.asset_name),
            ("WorkbenchEffectTagField", "value", profile.tag),
            ("WorkbenchEffectMagnitudeField", "value", profile.magnitude),
            ("WorkbenchEffectDurationRow", "value", profile.duration),
            ("WorkbenchEffectPeriodRow", "value", profile.period),
            (
                "WorkbenchEffectModifierHealthRow",
                "text",
                profile.modifier_text,
            ),
            ("WorkbenchEffectGraphRow", "text", profile.graph_text),
            (
                "WorkbenchEffectAttributePreviewRow",
                "text",
                profile.preview_text,
            ),
            (
                "WorkbenchEffectOutputRow",
                "text",
                profile.simulation_output,
            ),
        ] {
            self.set_effect_string(control_id, property, value)?;
        }
        self.set_visible(EFFECT_SEARCH_EMPTY_CONTROL, false)?;
        Ok(())
    }

    fn apply_effect_feedback(&mut self) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let effect_name = self
            .control_string("WorkbenchEffectNameField", "value")
            .unwrap_or_default();
        let tag = self
            .control_string("WorkbenchEffectTagField", "value")
            .unwrap_or_default();
        let magnitude = self
            .control_string("WorkbenchEffectMagnitudeField", "value")
            .unwrap_or_default();
        let policy = self
            .control_string("WorkbenchEffectPolicyDropdown", "value_text")
            .unwrap_or_default();
        let stacking = self
            .control_string("WorkbenchEffectStackField", "value")
            .unwrap_or_default();
        self.set_effect_string("WorkbenchStatusReady", "text", "Gameplay effect applied")?;
        self.set_effect_string("WorkbenchStatusMessages", "text", "1 Message")?;
        self.set_effect_string("WorkbenchEffectCenterTitle", "text", effect_name.clone())?;
        self.set_effect_string(
            "WorkbenchEffectAttributePreviewRow",
            "text",
            format!("Preview Level 1       {tag} {magnitude}"),
        )?;
        self.set_effect_string(
            "WorkbenchEffectOutputRow",
            "text",
            format!("Applied {effect_name}   {policy}   {stacking}"),
        )
    }

    fn set_effect_string(
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

fn effect_detail_selection(
    action_id: &str,
) -> Option<(&'static [&'static str], &'static str, &'static str)> {
    match action_id {
        "workbench.module.effect.modifier_health.select" => Some((
            EFFECT_MODIFIER_ROWS,
            "WorkbenchEffectModifierHealthRow",
            "Health modifier",
        )),
        "workbench.module.effect.modifier_healing.select" => Some((
            EFFECT_MODIFIER_ROWS,
            "WorkbenchEffectModifierHealingRow",
            "Incoming Healing modifier",
        )),
        "workbench.module.effect.modifier_cap.select" => Some((
            EFFECT_MODIFIER_ROWS,
            "WorkbenchEffectModifierCapRow",
            "Max Health Cap modifier",
        )),
        "workbench.module.effect.graph_select.select" => {
            Some((EFFECT_GRAPH_ROWS, "WorkbenchEffectGraphRow", "effect graph"))
        }
        "workbench.module.effect.attribute_preview.select" => Some((
            EFFECT_PREVIEW_ROWS,
            "WorkbenchEffectAttributePreviewRow",
            "attribute preview",
        )),
        _ => None,
    }
}

struct EffectAssetProfile {
    action_id: &'static str,
    row_control_id: &'static str,
    asset_name: &'static str,
    tag: &'static str,
    magnitude: &'static str,
    duration: &'static str,
    period: &'static str,
    modifier_text: &'static str,
    graph_text: &'static str,
    preview_text: &'static str,
    simulation_output: &'static str,
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

    use super::*;

    #[test]
    fn effect_asset_selection_search_and_apply_share_one_projection() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
                .expect("workbench bridge should build");

        assert!(bridge.control_bool("WorkbenchEffectHealthRegenRow", "selected"));
        bridge
            .dispatch_control_state("WorkbenchEffectDamageFireRow", UiEventKind::Click)
            .expect("damage effect should dispatch")
            .expect("damage effect should bind");
        assert_eq!(
            Some("GE_DamageFire".to_string()),
            bridge.control_string("WorkbenchEffectCenterTitle", "text")
        );
        assert_eq!(
            Some("Effect.Damage.Fire".to_string()),
            bridge.control_string("WorkbenchEffectTagField", "value")
        );
        assert_eq!(
            Some("25.0".to_string()),
            bridge.control_string("WorkbenchEffectMagnitudeField", "value")
        );

        bridge
            .dispatch_control_state("WorkbenchEffectModifierHealingRow", UiEventKind::Click)
            .expect("healing modifier should dispatch")
            .expect("healing modifier should bind");
        assert!(bridge.control_bool("WorkbenchEffectDamageFireRow", "selected"));
        assert!(bridge.control_bool("WorkbenchEffectModifierHealingRow", "selected"));
        assert!(bridge.control_bool("WorkbenchEffectGraphRow", "selected"));
        assert!(bridge.control_bool("WorkbenchEffectAttributePreviewRow", "selected"));

        for (control_id, value) in [
            ("WorkbenchEffectNameField", "GE_CustomBurn"),
            ("WorkbenchEffectTagField", "Effect.Damage.Custom"),
            ("WorkbenchEffectMagnitudeField", "42.5"),
            ("WorkbenchEffectStackField", "Aggregate by Target"),
        ] {
            bridge
                .mutate_control_property(control_id, "value", UiValue::String(value.to_string()))
                .expect("effect property should edit");
        }
        assert!(bridge
            .select_dropdown_option("WorkbenchEffectPolicyDropdown", "instant")
            .expect("effect policy should select"));

        bridge
            .dispatch_control_state("WorkbenchEffectApplyButton", UiEventKind::Click)
            .expect("effect apply should dispatch")
            .expect("effect apply should bind");
        assert_eq!(
            Some("Applied GE_CustomBurn   Instant   Aggregate by Target".to_string()),
            bridge.control_string("WorkbenchEffectOutputRow", "text")
        );
        assert_eq!(
            Some("GE_CustomBurn".to_string()),
            bridge.control_string("WorkbenchEffectCenterTitle", "text")
        );
        assert_eq!(
            Some("Preview Level 1       Effect.Damage.Custom 42.5".to_string()),
            bridge.control_string("WorkbenchEffectAttributePreviewRow", "text")
        );

        bridge
            .mutate_control_property(
                EFFECT_SEARCH_CONTROL,
                "query",
                UiValue::String("health".to_string()),
            )
            .expect("effect query should update");
        bridge
            .dispatch_control_state(EFFECT_SEARCH_CONTROL, UiEventKind::Change)
            .expect("effect search should dispatch")
            .expect("effect search should bind");
        assert!(bridge
            .control_frame("WorkbenchEffectHealthRegenRow")
            .is_some());
        assert!(bridge
            .control_frame("WorkbenchEffectDamageFireRow")
            .is_none());
        assert!(bridge.control_bool("WorkbenchEffectHealthRegenRow", "selected"));
        assert_eq!(
            Some("GE_HealthRegen".to_string()),
            bridge.control_string("WorkbenchEffectCenterTitle", "text")
        );
    }
}
