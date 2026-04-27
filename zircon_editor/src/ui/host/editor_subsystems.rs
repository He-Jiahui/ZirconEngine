use std::collections::BTreeSet;

use zircon_runtime::core::CoreHandle;

pub const EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY: &str = "zircon.editor.enabled_subsystems";
pub const EDITOR_RUNTIME_SANDBOX_ENABLED_CONFIG_KEY: &str = "zircon.editor.runtime_sandbox_enabled";

pub const EDITOR_SUBSYSTEM_ANIMATION_AUTHORING: &str = "editor.extension.animation_authoring";
pub const EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING: &str = "editor.extension.ui_asset_authoring";
pub const EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS: &str = "editor.extension.runtime_diagnostics";
pub const EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING: &str = "editor.extension.native_window_hosting";

pub(crate) const OPTIONAL_EDITOR_SUBSYSTEMS: &[&str] = &[
    EDITOR_SUBSYSTEM_ANIMATION_AUTHORING,
    EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING,
    EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS,
    EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorSubsystemReport {
    enabled_subsystems: Vec<String>,
    disabled_subsystems: Vec<String>,
    diagnostics: Vec<String>,
}

pub(crate) fn editor_subsystem_report_from_core(core: &CoreHandle) -> EditorSubsystemReport {
    let requested = core
        .load_config::<Vec<String>>(EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY)
        .ok();
    let known = OPTIONAL_EDITOR_SUBSYSTEMS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut enabled = requested
        .as_ref()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| known.contains(item.as_str()).then_some(item.clone()))
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_else(|| known.iter().map(|item| (*item).to_string()).collect());

    let disabled = known
        .iter()
        .filter(|item| !enabled.iter().any(|enabled| enabled == *item))
        .map(|item| (*item).to_string())
        .collect::<Vec<_>>();
    let mut diagnostics = Vec::new();
    if let Some(requested) = requested {
        for item in requested {
            if !known.contains(item.as_str()) {
                enabled.insert(item.clone());
                diagnostics.push(format!("custom editor capability enabled: {item}"));
            }
        }
    }

    EditorSubsystemReport {
        enabled_subsystems: enabled.into_iter().collect(),
        disabled_subsystems: disabled,
        diagnostics,
    }
}

pub(crate) fn editor_runtime_sandbox_enabled(core: &CoreHandle) -> bool {
    core.load_config::<bool>(EDITOR_RUNTIME_SANDBOX_ENABLED_CONFIG_KEY)
        .unwrap_or(true)
}

impl EditorSubsystemReport {
    pub(crate) fn default_enabled() -> Self {
        Self {
            enabled_subsystems: OPTIONAL_EDITOR_SUBSYSTEMS
                .iter()
                .map(|item| (*item).to_string())
                .collect(),
            disabled_subsystems: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn enabled_subsystems(&self) -> &[String] {
        &self.enabled_subsystems
    }

    pub fn disabled_subsystems(&self) -> &[String] {
        &self.disabled_subsystems
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn is_enabled(&self, subsystem: &str) -> bool {
        self.enabled_subsystems
            .iter()
            .any(|enabled| enabled == subsystem)
    }
}
