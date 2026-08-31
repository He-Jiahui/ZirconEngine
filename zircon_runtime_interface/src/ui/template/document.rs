use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::accessibility::UiAccessibilityContract;
use crate::ui::binding::UiEventKind;
use crate::ui::component::UiComponentEventKind;
use crate::ui::focus::UiFocusContract;
use crate::ui::navigation::UiNavigationContract;
use crate::ui::picking::UiPickPolicy;
use crate::ui::widget::UiWidgetContract;

use super::{UiActionRef, UiBindingTargetAssignment};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBindingWritePermissions {
    pub writes_target: bool,
    pub writes_source: bool,
    pub publishes_command: bool,
}

impl UiBindingWritePermissions {
    pub const TARGET_ONLY: Self = Self {
        writes_target: true,
        writes_source: false,
        publishes_command: false,
    };
    pub const SOURCE_AND_TARGET: Self = Self {
        writes_target: true,
        writes_source: true,
        publishes_command: false,
    };
    pub const COMMAND_ONLY: Self = Self {
        writes_target: false,
        writes_source: false,
        publishes_command: true,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiBindingTriggerTiming {
    Instantiation,
    SourceChange,
    SourceOrTargetChange,
    EventDispatch,
    CommandDispatch,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiBindingMode {
    OneTime,
    OneWay,
    TwoWay,
    #[default]
    Event,
    Command,
}

impl UiBindingMode {
    pub const fn trigger_timing(self) -> UiBindingTriggerTiming {
        match self {
            Self::OneTime => UiBindingTriggerTiming::Instantiation,
            Self::OneWay => UiBindingTriggerTiming::SourceChange,
            Self::TwoWay => UiBindingTriggerTiming::SourceOrTargetChange,
            Self::Event => UiBindingTriggerTiming::EventDispatch,
            Self::Command => UiBindingTriggerTiming::CommandDispatch,
        }
    }

    pub const fn write_permissions(self) -> UiBindingWritePermissions {
        match self {
            Self::OneTime | Self::OneWay | Self::Event => UiBindingWritePermissions::TARGET_ONLY,
            Self::TwoWay => UiBindingWritePermissions::SOURCE_AND_TARGET,
            Self::Command => UiBindingWritePermissions::COMMAND_ONLY,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiBindingRef {
    pub id: String,
    pub event: UiEventKind,
    #[serde(default)]
    pub mode: UiBindingMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_event: Option<UiComponentEventKind>,
    #[serde(default)]
    pub route: Option<String>,
    #[serde(default)]
    pub action: Option<UiActionRef>,
    #[serde(default)]
    pub targets: Vec<UiBindingTargetAssignment>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiTemplateNode {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_asset_id: Option<String>,
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub slot: Option<String>,
    #[serde(default)]
    pub control_id: Option<String>,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub bindings: Vec<UiBindingRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub binding_source_asset_ids: Vec<String>,
    #[serde(default)]
    pub children: Vec<UiTemplateNode>,
    #[serde(default)]
    pub slots: BTreeMap<String, Vec<UiTemplateNode>>,
    #[serde(default)]
    pub attributes: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub slot_attributes: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub style_overrides: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub style_tokens: BTreeMap<String, String>,
    #[serde(default)]
    pub focus: UiFocusContract,
    #[serde(default)]
    pub navigation: UiNavigationContract,
    #[serde(default)]
    pub picking: UiPickPolicy,
    #[serde(default)]
    pub a11y: UiAccessibilityContract,
    #[serde(default)]
    pub widget: UiWidgetContract,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_mode_contract_serializes_trigger_timing_and_write_permissions() {
        let cases = [
            (
                "OneTime",
                UiBindingMode::OneTime,
                UiBindingTriggerTiming::Instantiation,
                UiBindingWritePermissions::TARGET_ONLY,
            ),
            (
                "OneWay",
                UiBindingMode::OneWay,
                UiBindingTriggerTiming::SourceChange,
                UiBindingWritePermissions::TARGET_ONLY,
            ),
            (
                "TwoWay",
                UiBindingMode::TwoWay,
                UiBindingTriggerTiming::SourceOrTargetChange,
                UiBindingWritePermissions::SOURCE_AND_TARGET,
            ),
            (
                "Event",
                UiBindingMode::Event,
                UiBindingTriggerTiming::EventDispatch,
                UiBindingWritePermissions::TARGET_ONLY,
            ),
            (
                "Command",
                UiBindingMode::Command,
                UiBindingTriggerTiming::CommandDispatch,
                UiBindingWritePermissions::COMMAND_ONLY,
            ),
        ];

        for (serialized_mode, expected_mode, expected_trigger, expected_permissions) in cases {
            let binding: UiBindingRef = toml::from_str(&format!(
                "id = \"mode.contract\"\nevent = \"Click\"\nmode = \"{serialized_mode}\"\n"
            ))
            .unwrap();

            assert_eq!(binding.mode, expected_mode);
            assert_eq!(binding.mode.trigger_timing(), expected_trigger);
            assert_eq!(binding.mode.write_permissions(), expected_permissions);
            assert!(toml::to_string(&binding)
                .unwrap()
                .contains(&format!("mode = \"{serialized_mode}\"")));
        }

        let legacy: UiBindingRef = toml::from_str("id = \"legacy\"\nevent = \"Click\"\n").unwrap();
        assert_eq!(legacy.mode, UiBindingMode::Event);
    }

    #[test]
    fn typed_component_event_serde_round_trips_declared_identity() {
        let binding: UiBindingRef = toml::from_str(
            r#"
id = "product.lower_snake"
event = "Click"
component_event = "OpenPopup"
route = "component_lab.open_popup.product"
"#,
        )
        .expect("typed component event should deserialize");

        assert_eq!(
            binding.component_event,
            Some(UiComponentEventKind::OpenPopup)
        );
        let serialized = toml::to_string(&binding).expect("typed component event should serialize");
        assert!(serialized.contains("component_event = \"OpenPopup\""));

        let legacy: UiBindingRef = toml::from_str("id = \"legacy\"\nevent = \"Click\"\n").unwrap();
        assert_eq!(legacy.component_event, None);
    }
}
