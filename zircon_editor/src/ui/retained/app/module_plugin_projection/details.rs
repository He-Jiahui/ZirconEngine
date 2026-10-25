//! The summary stays cached by its published Arc. Only an open details page owns
//! a bounded row projection; action intents belong to that exact selection.
use std::rc::Rc;
use std::sync::{Arc, Weak};

use crate::ui::host::{
    EditorPluginFeatureStatus, EditorPluginStatusReport, PluginEnablementOperationReport,
    PluginEnablementRecovery,
};
use crate::ui::layouts::windows::workbench_host_window::{
    ModulePluginDetailsViewData, ModulePluginFeatureRowViewData, ModulePluginsPaneViewData,
};

use super::rows::module_plugin_feature_action;

// A presentation policy, rather than a catalog or performance acceptance limit.
const FEATURES_PER_PAGE: usize = 8;

#[derive(Default)]
pub(in crate::ui::retained_host::app) struct ModulePluginDetailsState {
    project: String,
    owner: Option<String>,
    feature: Option<String>,
    page: usize,
    package_selection: String,
    focus: String,
    generation: u64,
    report: Weak<EditorPluginStatusReport>,
    cached: Option<(String, u64, Rc<ModulePluginDetailsViewData>)>,
    intent: Option<FeatureIntent>,
}

struct FeatureIntent {
    id: String,
    project: String,
    owner: String,
    feature: String,
    page: usize,
    action: String,
}

fn plugin_recovery_operation<'a>(
    report: &'a EditorPluginStatusReport,
    owner: &str,
) -> Option<&'a PluginEnablementOperationReport> {
    report.enablement.as_ref().filter(|operation| {
        operation.plugin_id == owner && operation.recovery != PluginEnablementRecovery::None
    })
}

fn detail_feature_action(
    report: &EditorPluginStatusReport,
    owner: &str,
    feature: &EditorPluginFeatureStatus,
) -> (String, String) {
    if feature.owner_plugin_id != owner || plugin_recovery_operation(report, owner).is_some() {
        return (String::new(), String::new());
    }
    module_plugin_feature_action(std::slice::from_ref(feature))
}

impl ModulePluginDetailsState {
    fn changed(&mut self) {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("details generation exhausted");
        self.cached = None;
        self.intent = None;
    }

    fn reconcile(&mut self, project: &str, report: &Arc<EditorPluginStatusReport>) {
        if self.project != project {
            self.project = project.to_owned();
            self.owner = None;
            self.feature = None;
            self.package_selection.clear();
            self.page = 0;
            self.focus.clear();
            self.changed();
        }
        if !self.report.ptr_eq(&Arc::downgrade(report)) {
            self.report = Arc::downgrade(report);
            self.changed();
            if self.owner.is_none()
                && !self.package_selection.is_empty()
                && !report
                    .plugins
                    .iter()
                    .any(|plugin| plugin.plugin_id == self.package_selection)
            {
                self.package_selection = report
                    .plugins
                    .first()
                    .map(|plugin| plugin.plugin_id.clone())
                    .unwrap_or_default();
                self.focus = "package".into();
            }
        }
        let Some(owner) = self.owner.as_deref() else {
            return;
        };
        let Some(plugin) = report
            .plugins
            .iter()
            .find(|plugin| plugin.plugin_id == owner)
        else {
            self.owner = None;
            self.feature = None;
            self.package_selection = report
                .plugins
                .first()
                .map(|plugin| plugin.plugin_id.clone())
                .unwrap_or_default();
            self.page = 0;
            self.focus = "package".into();
            self.changed();
            return;
        };
        if let Some(feature) = self.feature.as_deref() {
            match plugin
                .optional_features
                .iter()
                .position(|item| item.id == feature)
            {
                Some(index) => self.page = index / FEATURES_PER_PAGE,
                None => {
                    self.feature = None;
                    self.changed();
                }
            }
        }
        let last = plugin.optional_features.len().saturating_sub(1) / FEATURES_PER_PAGE;
        if self.page > last {
            self.page = last;
            self.feature = None;
            self.changed();
        }
        // Reports may remove a feature or its action without entering recovery.
        // Keep keyboard focus on a control admitted by this same report and page.
        let focus_is_available = match self.focus.as_str() {
            "back" => true,
            "previous" => self.page > 0,
            "next" => self.page < last,
            "features" => !plugin.optional_features.is_empty(),
            "apply" => self
                .feature
                .as_deref()
                .and_then(|id| {
                    plugin
                        .optional_features
                        .iter()
                        .find(|feature| feature.id == id)
                })
                .is_some_and(|feature| {
                    !detail_feature_action(report, &plugin.plugin_id, feature)
                        .1
                        .is_empty()
                }),
            _ => false,
        };
        if !focus_is_available {
            self.focus = if plugin.optional_features.is_empty() {
                "back"
            } else {
                "features"
            }
            .into();
            self.changed();
        }
    }

    fn open(&mut self, owner: &str, report: &EditorPluginStatusReport) -> bool {
        if !report
            .plugins
            .iter()
            .any(|plugin| plugin.plugin_id == owner)
        {
            return false;
        }
        self.package_selection = owner.to_owned();
        self.owner = Some(owner.to_owned());
        self.feature = None;
        self.page = 0;
        self.focus = "features".into();
        self.changed();
        true
    }

    fn back(&mut self) {
        self.owner = None;
        self.feature = None;
        self.page = 0;
        self.focus = "package".into();
        self.changed();
    }

    fn turn_page(&mut self, next: bool, report: &EditorPluginStatusReport) -> bool {
        let Some(plugin) = self.owner.as_deref().and_then(|owner| {
            report
                .plugins
                .iter()
                .find(|plugin| plugin.plugin_id == owner)
        }) else {
            return false;
        };
        let last = plugin.optional_features.len().saturating_sub(1) / FEATURES_PER_PAGE;
        let page = if next {
            self.page.saturating_add(1).min(last)
        } else {
            self.page.saturating_sub(1)
        };
        if page == self.page {
            return false;
        }
        self.page = page;
        self.feature = None;
        self.focus = "features".into();
        self.changed();
        true
    }

    fn select(&mut self, index: usize, identity: &str, report: &EditorPluginStatusReport) -> bool {
        let Some(plugin) = self.owner.as_deref().and_then(|owner| {
            report
                .plugins
                .iter()
                .find(|plugin| plugin.plugin_id == owner)
        }) else {
            return false;
        };
        if index >= FEATURES_PER_PAGE {
            return false;
        }
        let Some(feature) = plugin
            .optional_features
            .get(self.page * FEATURES_PER_PAGE + index)
        else {
            return false;
        };
        if feature.id != identity {
            return false;
        }
        self.feature = Some(feature.id.clone());
        self.focus = "features".into();
        self.changed();
        true
    }

    fn project(
        &mut self,
        project: &str,
        report: &Arc<EditorPluginStatusReport>,
        locale: &str,
        mut text: impl FnMut(&str) -> String,
    ) -> Option<Rc<ModulePluginDetailsViewData>> {
        self.reconcile(project, report);
        self.owner.as_ref()?;
        if let Some((cached_locale, generation, view)) = &self.cached {
            if cached_locale == locale && *generation == self.generation {
                return Some(view.clone());
            }
        }
        // Locale changes also retire the previously displayed action intent.
        if self.cached.is_some() {
            self.changed();
        }
        let owner = self.owner.as_deref()?;
        let plugin = report
            .plugins
            .iter()
            .find(|plugin| plugin.plugin_id == owner)?;
        let state_text = |feature: &EditorPluginFeatureStatus,
                          text: &mut dyn FnMut(&str) -> String| {
            let yes = text("yes");
            let no = text("no");
            format!(
                "{}: {}\n{}: {}\n{}: {}\n{}: {}",
                text("desired"),
                if feature.enabled { &yes } else { &no },
                text("effective"),
                text("unknown"),
                text("required"),
                if feature.required { &yes } else { &no },
                text("blocked"),
                if feature.available { &no } else { &yes }
            )
        };
        let rows = plugin
            .optional_features
            .iter()
            .skip(self.page * FEATURES_PER_PAGE)
            .take(FEATURES_PER_PAGE)
            .map(|feature| ModulePluginFeatureRowViewData {
                feature_id: feature.id.clone(),
                label: format!(
                    "{}\n{}",
                    feature.display_name,
                    state_text(feature, &mut text)
                ),
                desired: feature.enabled,
                required: feature.required,
                blocked: !feature.available,
                effective: text("unknown"),
            })
            .collect();
        let mut view = ModulePluginDetailsViewData {
            plugin_id: plugin.plugin_id.clone(),
            title: plugin.display_name.clone(),
            page: self.page,
            total_features: plugin.optional_features.len(),
            rows,
            back_label: text("back"),
            previous_label: text("previous"),
            next_label: text("next"),
            page_label: format!(
                "{} {}/{}",
                text("page"),
                self.page + 1,
                plugin
                    .optional_features
                    .len()
                    .max(1)
                    .div_ceil(FEATURES_PER_PAGE)
            ),
            empty_label: text("empty"),
            ..ModulePluginDetailsViewData::default()
        };
        if let Some(feature) = self.feature.as_deref().and_then(|id| {
            plugin
                .optional_features
                .iter()
                .find(|feature| feature.id == id)
        }) {
            view.selected_feature_id = feature.id.clone();
            view.selected_identity = format!(
                "{}: {}\n{}: {}",
                text("owner"),
                plugin.plugin_id,
                text("feature"),
                feature.id
            );
            view.selected_name = feature.display_name.clone();
            view.selected_states = state_text(feature, &mut text);
            view.selected_reason = if feature.available {
                String::new()
            } else {
                let mut reasons = feature.diagnostics.clone();
                for dependency in &feature.dependencies {
                    if !dependency.plugin_enabled || !dependency.capability_available {
                        reasons.push(format!(
                            "{}: {} / {}",
                            text("dependency"),
                            dependency.plugin_id,
                            dependency.capability
                        ));
                    }
                }
                if reasons.is_empty() {
                    text("blocked_reason_unknown")
                } else {
                    reasons.join("\n")
                }
            };
            if let Some(operation) = plugin_recovery_operation(report, owner) {
                let reason = operation.status_message();
                if view.selected_reason.is_empty() {
                    view.selected_reason = reason;
                } else {
                    view.selected_reason.push('\n');
                    view.selected_reason.push_str(&reason);
                }
            }
            let (label, action) = detail_feature_action(report, owner, feature);
            if !action.is_empty() {
                let key = match label.as_str() {
                    "Enable Deps" => "enable_dependencies",
                    "Enable Feature" => "enable",
                    "Disable Feature" => "disable",
                    _ => "apply",
                };
                view.action_label = text(key);
                view.action_id = action.clone();
                view.intent_id = format!("feature-intent:{}", self.generation);
                self.intent = Some(FeatureIntent {
                    id: view.intent_id.clone(),
                    project: project.to_owned(),
                    owner: owner.to_owned(),
                    feature: feature.id.clone(),
                    page: self.page,
                    action,
                });
            }
        }
        let view = Rc::new(view);
        self.cached = Some((locale.to_owned(), self.generation, view.clone()));
        Some(view)
    }

    fn take_action(
        &mut self,
        project: &str,
        report: &Arc<EditorPluginStatusReport>,
        id: &str,
    ) -> Option<String> {
        self.reconcile(project, report);
        let intent = self.intent.as_ref()?;
        if intent.id != id
            || intent.project != project
            || self.owner.as_deref() != Some(&intent.owner)
            || self.feature.as_deref() != Some(&intent.feature)
            || self.page != intent.page
        {
            return None;
        }
        let feature = report
            .plugins
            .iter()
            .find(|plugin| plugin.plugin_id == intent.owner)?
            .optional_features
            .iter()
            .find(|feature| feature.id == intent.feature)?;
        let (_, current) = detail_feature_action(report, &intent.owner, feature);
        if current != intent.action || current.is_empty() {
            self.changed();
            return None;
        }
        let action = intent.action.clone();
        self.changed();
        Some(action)
    }
}

use super::super::RetainedEditorHost;
use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::{UiKeyboardInputEvent, UiTemplateActionInvocation},
    template::UiLocalizedTextRef,
};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn project_module_plugin_details(
        &self,
        project: &str,
        report: &Arc<EditorPluginStatusReport>,
        pane: &mut ModulePluginsPaneViewData,
    ) {
        let context = self.runtime.context();
        let i18n = context.i18n();
        let locale = i18n.active_locale();
        let mut state = self.module_plugin_details.borrow_mut();
        pane.details = state.project(project, report, locale.as_str(), |name| {
            let reference = UiLocalizedTextRef {
                key: format!("editor.plugin_details.{name}"),
                table: None,
                fallback: None,
            };
            i18n.resolve_localized_text_for_locale(&locale, &reference)
                .unwrap_or_else(|_| i18n.translate_for_locale(&locale, &reference.key))
                .to_string()
        });
        pane.detail_generation = state.generation;
        pane.detail_locale = locale.as_str().into();
        pane.selected_plugin_id = state.package_selection.as_str().into();
        pane.focused_control = state.focus.as_str().into();
    }

    fn module_plugin_details_report(&self) -> Option<(String, Arc<EditorPluginStatusReport>)> {
        let project = self.runtime.editor_snapshot().ok()?.project_path;
        let report = self
            .editor_manager
            .enter()
            .ok()?
            .published_plugin_status_report();
        Some((project, report))
    }

    pub(in crate::ui::retained_host::app) fn is_module_plugin_details_pane(
        &self,
        pane_id: &str,
    ) -> bool {
        self.editor_manager.enter().is_ok_and(|manager| {
            manager
                .view_instance_ids_for_descriptor_key("editor.module_plugins")
                .iter()
                .any(|id| id.0 == pane_id)
        })
    }

    pub(in crate::ui::retained_host::app) fn select_module_plugin_feature(
        &mut self,
        pane_id: &str,
        index: i32,
        identity_kind: &str,
        identity: &str,
    ) -> bool {
        if identity_kind != "string" || !self.is_module_plugin_details_pane(pane_id) {
            return false;
        }
        let Ok(index) = usize::try_from(index) else {
            return false;
        };
        let Some((project, report)) = self.module_plugin_details_report() else {
            return false;
        };
        let selected = {
            let mut state = self.module_plugin_details.borrow_mut();
            state.reconcile(&project, &report);
            state.select(index, identity, &report)
        };
        if selected {
            self.mark_presentation_dirty();
        }
        selected
    }

    pub(in crate::ui::retained_host::app) fn dispatch_module_plugin_details_invocation(
        &mut self,
        invocation: &UiTemplateActionInvocation,
    ) -> bool {
        if invocation.is_action() {
            return false;
        }
        let control = match invocation.target_id() {
            "workbench.plugin.details.back" => "ModulePluginDetailsBack",
            "workbench.plugin.details.previous" => "ModulePluginDetailsPrevious",
            "workbench.plugin.details.next" => "ModulePluginDetailsNext",
            "workbench.plugin.details.apply" => "ModulePluginDetailsApply",
            _ => return false,
        };
        let intent = match invocation.payload.get("intent") {
            Some(UiValue::String(value)) => value.as_str(),
            _ => "",
        };
        self.dispatch_module_plugin_detail_control(control, intent)
    }

    pub(in crate::ui::retained_host::app) fn dispatch_module_plugin_detail_control(
        &mut self,
        control: &str,
        identity: &str,
    ) -> bool {
        if !matches!(
            control,
            "ModulePluginSelect"
                | "ModulePluginDetailsBack"
                | "ModulePluginDetailsPrevious"
                | "ModulePluginDetailsNext"
                | "ModulePluginDetailsApply"
        ) {
            return false;
        }
        let Some((project, report)) = self.module_plugin_details_report() else {
            return true;
        };
        // Resolve to an owned action while holding only the local selection state.
        // In particular, no document/registry/manager guard reaches plugin mutation.
        let action = {
            let mut state = self.module_plugin_details.borrow_mut();
            state.reconcile(&project, &report);
            match control {
                "ModulePluginSelect" => {
                    state.open(identity, &report);
                    None
                }
                "ModulePluginDetailsBack" => {
                    state.back();
                    None
                }
                "ModulePluginDetailsPrevious" => {
                    state.turn_page(false, &report);
                    None
                }
                "ModulePluginDetailsNext" => {
                    state.turn_page(true, &report);
                    None
                }
                "ModulePluginDetailsApply" => state.take_action(&project, &report, identity),
                _ => None,
            }
        };
        if let Some(action) = action {
            self.dispatch_module_plugin_action(&action);
        }
        self.mark_presentation_dirty();
        true
    }

    pub(in crate::ui::retained_host::app) fn dispatch_module_plugin_details_keyboard(
        &mut self,
        keyboard: &UiKeyboardInputEvent,
    ) -> bool {
        use crate::ui::retained_host::host_contract::globals::UiHostContext;
        let modifiers = &keyboard.metadata.modifiers;
        if modifiers.control
            || modifiers.alt
            || modifiers.super_key
            || self.workbench_window_bridge.content_drawer_is_open()
            || self
                .ui
                .global::<UiHostContext>()
                .get_text_input_focus()
                .is_active()
        {
            return false;
        }
        let owns_focus = self.editor_manager.enter().is_ok_and(|manager| {
            manager.resolve_keyboard_input(keyboard).is_none()
                && manager.current_focused_view().is_some_and(|focused| {
                    manager
                        .view_instance_ids_for_descriptor_key("editor.module_plugins")
                        .contains(&focused)
                })
        });
        if !owns_focus {
            return false;
        }
        let Some((project, report)) = self.module_plugin_details_report() else {
            return false;
        };
        let key = keyboard.logical_key.to_ascii_lowercase();
        let mut pending = None;
        let handled = {
            let mut state = self.module_plugin_details.borrow_mut();
            state.reconcile(&project, &report);
            if state.owner.is_none() {
                match key.as_str() {
                    "arrowdown" | "arrowup" | "home" | "end" => {
                        if report.plugins.is_empty() {
                            return false;
                        }
                        let current = report
                            .plugins
                            .iter()
                            .position(|plugin| plugin.plugin_id == state.package_selection)
                            .unwrap_or(0);
                        let last = report.plugins.len() - 1;
                        let index = match key.as_str() {
                            "home" => 0,
                            "end" => last,
                            "arrowup" => current.saturating_sub(1),
                            _ => current.saturating_add(1).min(last),
                        };
                        state.package_selection = report.plugins[index].plugin_id.clone();
                        state.focus = "package".into();
                        state.changed();
                        true
                    }
                    "enter" | "space" | " " => {
                        let owner = if state.package_selection.is_empty() {
                            report
                                .plugins
                                .first()
                                .map(|plugin| plugin.plugin_id.clone())
                        } else {
                            Some(state.package_selection.clone())
                        };
                        owner.is_some_and(|owner| state.open(&owner, &report))
                    }
                    _ => false,
                }
            } else {
                match key.as_str() {
                    "escape" | "esc" => {
                        state.back();
                        true
                    }
                    "pageup" => {
                        state.turn_page(false, &report);
                        true
                    }
                    "pagedown" => {
                        state.turn_page(true, &report);
                        true
                    }
                    "tab" => {
                        let plugin = report
                            .plugins
                            .iter()
                            .find(|plugin| {
                                Some(plugin.plugin_id.as_str()) == state.owner.as_deref()
                            })
                            .expect("reconciled owner");
                        let mut controls = vec!["back"];
                        if state.page != 0 {
                            controls.push("previous");
                        }
                        if !plugin.optional_features.is_empty() {
                            controls.push("features");
                        }
                        if (state.page + 1) * FEATURES_PER_PAGE < plugin.optional_features.len() {
                            controls.push("next");
                        }
                        if state
                            .feature
                            .as_deref()
                            .and_then(|id| {
                                plugin
                                    .optional_features
                                    .iter()
                                    .find(|feature| feature.id == id)
                            })
                            .is_some_and(|feature| {
                                !detail_feature_action(&report, &plugin.plugin_id, feature)
                                    .1
                                    .is_empty()
                            })
                        {
                            controls.push("apply");
                        }
                        let index = controls
                            .iter()
                            .position(|item| *item == state.focus)
                            .unwrap_or(0);
                        let next = if modifiers.shift {
                            (index + controls.len() - 1) % controls.len()
                        } else {
                            (index + 1) % controls.len()
                        };
                        state.focus = controls[next].into();
                        state.changed();
                        true
                    }
                    "arrowdown" | "arrowup" | "home" | "end" if state.focus == "features" => {
                        let plugin = report
                            .plugins
                            .iter()
                            .find(|plugin| {
                                Some(plugin.plugin_id.as_str()) == state.owner.as_deref()
                            })
                            .expect("reconciled owner");
                        let start = state.page * FEATURES_PER_PAGE;
                        let count = plugin
                            .optional_features
                            .len()
                            .saturating_sub(start)
                            .min(FEATURES_PER_PAGE);
                        if count == 0 {
                            return false;
                        }
                        let current = plugin.optional_features[start..start + count]
                            .iter()
                            .position(|feature| {
                                Some(feature.id.as_str()) == state.feature.as_deref()
                            });
                        let index = match key.as_str() {
                            "home" => 0,
                            "end" => count - 1,
                            "arrowup" => current.unwrap_or(1).saturating_sub(1),
                            _ => current.map_or(0, |index| (index + 1).min(count - 1)),
                        };
                        let identity = &plugin.optional_features[start + index].id;
                        state.select(index, identity, &report)
                    }
                    "enter" | "space" | " " => {
                        match state.focus.as_str() {
                            "back" => state.back(),
                            "previous" => {
                                state.turn_page(false, &report);
                            }
                            "next" => {
                                state.turn_page(true, &report);
                            }
                            "apply" => {
                                let id = state.intent.as_ref().map(|intent| intent.id.clone());
                                pending =
                                    id.and_then(|id| state.take_action(&project, &report, &id));
                            }
                            _ => return false,
                        }
                        true
                    }
                    _ => false,
                }
            }
        };
        if let Some(action) = pending {
            self.dispatch_module_plugin_action(&action);
        }
        if handled {
            self.mark_presentation_dirty();
        }
        handled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::host::EditorPluginStatus;
    use zircon_runtime::core::framework::project::ExportPackagingStrategy;

    fn fixture_report(count: usize) -> Arc<EditorPluginStatusReport> {
        Arc::new(EditorPluginStatusReport {
            enablement: None,
            plugins: ["vendor.first", "vendor.second"]
                .into_iter()
                .map(|owner| EditorPluginStatus {
                    plugin_id: owner.into(),
                    display_name: owner.into(),
                    package_source: "native".into(),
                    package_metadata: None,
                    load_state: "manifest only".into(),
                    enabled: true,
                    required: false,
                    target_modes: vec![],
                    packaging: ExportPackagingStrategy::NativeDynamic,
                    runtime_crate: None,
                    editor_crate: None,
                    runtime_capabilities: vec![],
                    editor_capabilities: vec![],
                    diagnostics: vec![],
                    optional_features: (0..count)
                        .map(|index| EditorPluginFeatureStatus {
                            id: format!("shared.feature.{index}"),
                            display_name: format!("Feature {index}"),
                            owner_plugin_id: owner.into(),
                            enabled: index == 0,
                            required: index == 0,
                            available: index != 0,
                            target_modes: vec![],
                            packaging: ExportPackagingStrategy::NativeDynamic,
                            runtime_crate: None,
                            editor_crate: None,
                            provided_capabilities: vec![],
                            dependencies: vec![],
                            diagnostics: if index == 0 {
                                vec!["resolution blocked".into()]
                            } else {
                                vec![]
                            },
                        })
                        .collect(),
                })
                .collect(),
            diagnostics: vec![],
        })
    }

    fn view(
        state: &mut ModulePluginDetailsState,
        report: &Arc<EditorPluginStatusReport>,
    ) -> Rc<ModulePluginDetailsViewData> {
        state
            .project("project-a", report, "en", str::to_owned)
            .expect("open details")
    }

    fn recovery_report(
        recovery: PluginEnablementRecovery,
        owner: &str,
    ) -> Arc<EditorPluginStatusReport> {
        use crate::ui::host::{
            PluginEnablementApplied, PluginEnablementDurable, PluginEnablementPhase,
        };
        let mut report = (*fixture_report(2)).clone();
        let reverted = recovery == PluginEnablementRecovery::RestartRequired;
        report.enablement = Some(PluginEnablementOperationReport {
            operation_id: "details-recovery-fixture".into(),
            project_root: "fixture-project".into(),
            project_instance: "fixture-instance".into(),
            session_generation: 7,
            plugin_id: owner.into(),
            requested_enabled: false,
            expected_revision: Some(zircon_runtime::core::resource::io::SourceRevision::Missing),
            committed_revision: None,
            phase: if reverted {
                PluginEnablementPhase::Complete
            } else {
                PluginEnablementPhase::Reconcile
            },
            durable: PluginEnablementDurable::Durable,
            applied: PluginEnablementApplied::NotApplied,
            recovery,
            primary_cause: None,
            rollback_cause: None,
            detail: None,
            recovery_admission: None,
        });
        Arc::new(report)
    }

    #[test]
    fn report_refresh_restores_feature_focus_when_apply_is_no_longer_actionable() {
        for change in ["removed", "required", "foreign-owner"] {
            let original = fixture_report(3);
            let mut state = ModulePluginDetailsState::default();
            state.reconcile("project-a", &original);
            assert!(state.open("vendor.first", &original));
            assert!(state.select(1, "shared.feature.1", &original));
            state.focus = "apply".into();
            let ready = view(&mut state, &original);
            assert!(!ready.intent_id.is_empty());
            assert_eq!(state.focus, "apply");

            let mut updated = (*original).clone();
            let features = &mut updated.plugins[0].optional_features;
            match change {
                "removed" => {
                    features.remove(1);
                }
                "required" => {
                    features[1].enabled = true;
                    features[1].required = true;
                }
                "foreign-owner" => features[1].owner_plugin_id = "vendor.second".into(),
                _ => unreachable!(),
            }
            let updated = Arc::new(updated);
            let refreshed = view(&mut state, &updated);
            assert_eq!(state.focus, "features", "{change}");
            assert_eq!(state.package_selection, "vendor.first");
            assert_eq!(
                refreshed.selected_feature_id,
                if change == "removed" {
                    ""
                } else {
                    "shared.feature.1"
                }
            );
            assert!(refreshed.action_id.is_empty() && refreshed.intent_id.is_empty());
            assert!(state
                .take_action("project-a", &updated, &ready.intent_id)
                .is_none());
            let generation = state.generation;
            assert!(Rc::ptr_eq(&refreshed, &view(&mut state, &updated)));
            assert_eq!(state.generation, generation);
            assert_eq!(updated.plugins[1], original.plugins[1]);
            assert_eq!(updated.enablement, original.enablement);

            let index = updated.plugins[0]
                .optional_features
                .iter()
                .position(|feature| feature.id == "shared.feature.2")
                .unwrap();
            assert!(state.select(index, "shared.feature.2", &updated));
            let next = view(&mut state, &updated);
            assert!(!next.intent_id.is_empty());
            assert_eq!(
                state.take_action("project-a", &updated, &next.intent_id),
                Some(next.action_id.clone())
            );
        }
    }

    #[test]
    fn empty_feature_report_restores_back_focus_and_retires_detail_intent() {
        for focus in ["features", "apply", "previous", "next"] {
            let original = fixture_report(19);
            let mut state = ModulePluginDetailsState::default();
            state.reconcile("project-a", &original);
            assert!(state.open("vendor.first", &original));
            assert!(state.turn_page(true, &original));
            assert!(state.select(1, "shared.feature.9", &original));
            state.focus = focus.into();
            let ready = view(&mut state, &original);
            assert_eq!(state.focus, focus);
            assert!(!ready.intent_id.is_empty());

            let mut empty = (*original).clone();
            empty.plugins[0].optional_features.clear();
            let empty = Arc::new(empty);
            let refreshed = view(&mut state, &empty);
            assert_eq!(state.focus, "back", "{focus}");
            assert_eq!(refreshed.page, 0);
            assert!(refreshed.rows.is_empty() && refreshed.selected_feature_id.is_empty());
            assert!(refreshed.action_id.is_empty() && refreshed.intent_id.is_empty());
            assert!(state
                .take_action("project-a", &empty, &ready.intent_id)
                .is_none());
            assert!(Rc::ptr_eq(&refreshed, &view(&mut state, &empty)));
            assert_eq!(empty.plugins[1], original.plugins[1]);

            state.back();
            assert_eq!(state.package_selection, "vendor.first");
            assert_eq!(state.focus, "package");
            assert!(state.owner.is_none() && state.feature.is_none() && state.intent.is_none());
        }
    }

    #[test]
    fn report_refresh_restores_focus_when_pagination_controls_disappear() {
        for focus in ["previous", "next"] {
            let original = fixture_report(19);
            let mut state = ModulePluginDetailsState::default();
            state.reconcile("project-a", &original);
            assert!(state.open("vendor.first", &original));
            assert!(state.turn_page(true, &original));
            assert!(state.select(1, "shared.feature.9", &original));
            state.focus = focus.into();
            let ready = view(&mut state, &original);
            assert_eq!(state.focus, focus);

            let mut updated = (*original).clone();
            let features = &mut updated.plugins[0].optional_features;
            if focus == "previous" {
                *features = features.split_off(FEATURES_PER_PAGE);
            } else {
                features.truncate(13);
            }
            let updated = Arc::new(updated);
            let refreshed = view(&mut state, &updated);
            assert_eq!(state.focus, "features", "{focus}");
            assert_eq!(refreshed.page, usize::from(focus == "next"));
            assert_eq!(refreshed.selected_feature_id, "shared.feature.9");
            assert_eq!(refreshed.action_id, ready.action_id);
            assert_ne!(refreshed.intent_id, ready.intent_id);
            assert!(state
                .take_action("project-a", &updated, &ready.intent_id)
                .is_none());
            assert!(Rc::ptr_eq(&refreshed, &view(&mut state, &updated)));
            assert_eq!(updated.plugins[1], original.plugins[1]);
        }
    }

    #[test]
    fn recovery_retires_detail_apply_intent_and_restores_feature_focus() {
        for recovery in [
            PluginEnablementRecovery::PendingRuntimeApply,
            PluginEnablementRecovery::RuntimeRepairRequired,
            PluginEnablementRecovery::FileRecoveryRequired,
            PluginEnablementRecovery::RestartRequired,
        ] {
            let initial = fixture_report(2);
            let mut state = ModulePluginDetailsState::default();
            state.reconcile("project-a", &initial);
            assert!(state.open("vendor.first", &initial));
            assert!(state.select(1, "shared.feature.1", &initial));
            state.focus = "apply".into();
            let ready = view(&mut state, &initial);
            assert!(!ready.intent_id.is_empty());
            let report = recovery_report(recovery, "vendor.first");
            let operation = report.enablement.clone();
            let blocked = view(&mut state, &report);
            assert!(blocked.action_id.is_empty() && blocked.intent_id.is_empty());
            assert_eq!(blocked.selected_feature_id, "shared.feature.1");
            assert_eq!(blocked.selected_states, ready.selected_states);
            assert_eq!(blocked.rows[1].desired, ready.rows[1].desired);
            assert!(blocked
                .selected_reason
                .contains(&operation.as_ref().unwrap().status_message()));
            assert_eq!(state.focus, "features");
            assert!(state
                .take_action("project-a", &report, &ready.intent_id)
                .is_none());
            assert_eq!(report.enablement, operation);
            assert!(Rc::ptr_eq(&blocked, &view(&mut state, &report)));
        }
    }

    #[test]
    fn clearing_recovery_reenables_only_a_fresh_qualified_detail_intent() {
        let blocked_report = recovery_report(
            PluginEnablementRecovery::PendingRuntimeApply,
            "vendor.first",
        );
        let mut state = ModulePluginDetailsState::default();
        state.reconcile("project-a", &blocked_report);
        assert!(state.open("vendor.first", &blocked_report));
        assert!(state.select(1, "shared.feature.1", &blocked_report));
        assert!(view(&mut state, &blocked_report).action_id.is_empty());
        let ready_report = fixture_report(2);
        let ready = view(&mut state, &ready_report);
        assert_eq!(ready.selected_feature_id, "shared.feature.1");
        assert_eq!(
            ready.action_id,
            "workbench.plugin.feature.enable.#12:vendor.first.shared.feature.1"
        );
        assert!(!ready.intent_id.is_empty());
        assert_eq!(
            state.take_action("project-a", &ready_report, &ready.intent_id),
            Some(ready.action_id.clone())
        );
        assert!(state
            .take_action("project-a", &ready_report, &ready.intent_id)
            .is_none());
    }

    #[test]
    fn detail_recovery_gate_preserves_other_plugin_actions_and_feature_diagnostics() {
        let other_report = recovery_report(
            PluginEnablementRecovery::PendingRuntimeApply,
            "vendor.second",
        );
        let mut state = ModulePluginDetailsState::default();
        state.reconcile("project-a", &other_report);
        assert!(state.open("vendor.first", &other_report));
        assert!(state.select(1, "shared.feature.1", &other_report));
        assert!(!view(&mut state, &other_report).action_id.is_empty());
        let own_report = recovery_report(PluginEnablementRecovery::RestartRequired, "vendor.first");
        view(&mut state, &own_report);
        assert!(state.select(0, "shared.feature.0", &own_report));
        let blocked = view(&mut state, &own_report);
        assert!(blocked.selected_reason.contains("resolution blocked"));
        assert!(blocked.selected_reason.contains("pending change reverted"));
        assert!(!blocked.selected_reason.contains("change saved"));
        assert!(blocked.action_id.is_empty());
        assert!(blocked.rows[0].required && blocked.rows[0].blocked);
    }

    #[test]
    fn feature_pages_cover_all_features_with_honest_status_authority() {
        let report = fixture_report(19);
        let mut state = ModulePluginDetailsState::default();
        state.reconcile("project-a", &report);
        assert!(state.open("vendor.first", &report));
        let mut identities = Vec::new();
        loop {
            let page = view(&mut state, &report);
            assert!(page.rows.len() <= FEATURES_PER_PAGE);
            assert!(page.rows.iter().all(|row| row.effective == "unknown"
                && row.label.contains("desired:")
                && row.label.contains("effective: unknown")
                && row.label.contains("required:")
                && row.label.contains("blocked:")));
            identities.extend(page.rows.iter().map(|row| row.feature_id.clone()));
            if !state.turn_page(true, &report) {
                break;
            }
        }
        assert_eq!(
            identities,
            (0..19)
                .map(|index| format!("shared.feature.{index}"))
                .collect::<Vec<_>>()
        );
        assert!(state.turn_page(false, &report));
        assert!(state.turn_page(false, &report));
        assert!(state.select(0, "shared.feature.0", &report));
        let selected = view(&mut state, &report);
        assert!(selected.rows[0].desired && selected.rows[0].required && selected.rows[0].blocked);
        assert_eq!(selected.selected_reason, "resolution blocked");
        assert!(selected.action_id.is_empty());
        let empty = fixture_report(0);
        state.reconcile("project-a", &empty);
        let empty_view = view(&mut state, &empty);
        assert!(empty_view.rows.is_empty());
        assert_eq!(empty_view.page, 0);
        assert!(!state.turn_page(true, &empty));
    }

    #[test]
    fn selection_survives_reorder_and_resets_on_owner_or_project_change() {
        let original = fixture_report(19);
        let mut state = ModulePluginDetailsState::default();
        state.reconcile("project-a", &original);
        state.open("vendor.first", &original);
        assert!(state.turn_page(true, &original));
        assert!(state.select(1, "shared.feature.9", &original));
        let mut reordered = (*original).clone();
        reordered.plugins.reverse();
        reordered.plugins[1].optional_features.reverse();
        let reordered = Arc::new(reordered);
        let page = view(&mut state, &reordered);
        assert_eq!(page.plugin_id, "vendor.first");
        assert_eq!(page.selected_feature_id, "shared.feature.9");
        assert_eq!(page.page, 1);
        let stable = view(&mut state, &reordered);
        assert!(Rc::ptr_eq(&page, &stable));
        let localized = state
            .project("project-a", &reordered, "zh-CN", |key| format!("中:{key}"))
            .unwrap();
        assert!(!Rc::ptr_eq(&page, &localized));
        assert!(localized.selected_states.contains("中:unknown"));
        state.open("vendor.second", &reordered);
        assert!(view(&mut state, &reordered).selected_feature_id.is_empty());
        state.back();
        assert_eq!(state.package_selection, "vendor.second");
        assert_eq!(state.focus, "package");
        state.open("vendor.second", &reordered);
        assert!(state
            .project("project-b", &reordered, "en", str::to_owned)
            .is_none());
        assert!(state.feature.is_none() && state.intent.is_none() && state.cached.is_none());
        state.open("vendor.first", &reordered);
        let removed = Arc::new(EditorPluginStatusReport::default());
        assert!(state
            .project("project-b", &removed, "en", str::to_owned)
            .is_none());
    }

    #[test]
    fn selected_feature_actions_keep_qualified_owner_and_reject_stale_targets() {
        let report = fixture_report(10);
        let mut state = ModulePluginDetailsState::default();
        state.reconcile("project-a", &report);
        state.open("vendor.first", &report);
        state.select(1, "shared.feature.1", &report);
        let first = view(&mut state, &report);
        assert_eq!(
            first.action_id,
            "workbench.plugin.feature.enable.#12:vendor.first.shared.feature.1"
        );
        state.open("vendor.second", &report);
        assert!(state
            .take_action("project-a", &report, &first.intent_id)
            .is_none());
        state.select(1, "shared.feature.1", &report);
        let second = view(&mut state, &report);
        assert_eq!(
            second.action_id,
            "workbench.plugin.feature.enable.#13:vendor.second.shared.feature.1"
        );
        assert_eq!(
            state.take_action("project-a", &report, &second.intent_id),
            Some(second.action_id.clone())
        );
        assert!(state
            .take_action("project-a", &report, &second.intent_id)
            .is_none());
        let refreshed = view(&mut state, &report);
        state.turn_page(true, &report);
        assert!(state
            .take_action("project-a", &report, &refreshed.intent_id)
            .is_none());
        state.turn_page(false, &report);
        state.select(1, "shared.feature.1", &report);
        let previous = view(&mut state, &report);
        let mut changed = (*report).clone();
        changed.plugins[1].optional_features[1].enabled = true;
        let changed = Arc::new(changed);
        assert!(state
            .take_action("project-a", &changed, &previous.intent_id)
            .is_none());
        assert!(view(&mut state, &changed)
            .action_id
            .starts_with("workbench.plugin.feature.disable."));
    }

    #[test]
    fn closed_package_selection_recovers_when_report_removes_the_back_target() {
        let original = fixture_report(19);
        let mut state = ModulePluginDetailsState::default();
        state.reconcile("project-a", &original);
        assert!(state.open("vendor.second", &original));
        assert!(state.select(1, "shared.feature.1", &original));
        assert!(!view(&mut state, &original).intent_id.is_empty());
        state.back();
        assert_eq!(state.package_selection, "vendor.second");

        let mut removed = (*original).clone();
        removed
            .plugins
            .retain(|plugin| plugin.plugin_id != "vendor.second");
        let removed = Arc::new(removed);
        assert!(state
            .project("project-a", &removed, "en", |_| panic!("closed details"))
            .is_none());
        assert_eq!(state.package_selection, "vendor.first");
        assert_eq!(state.focus, "package");
        assert!(state.owner.is_none() && state.feature.is_none());
        assert_eq!(state.page, 0);
        assert!(state.cached.is_none() && state.intent.is_none());
        let generation = state.generation;
        for _ in 0..100 {
            assert!(state
                .project("project-a", &removed, "en", |_| panic!("closed details"))
                .is_none());
            assert_eq!(state.generation, generation);
            assert!(state.cached.is_none() && state.intent.is_none());
        }

        let mut reordered = (*original).clone();
        reordered.plugins.reverse();
        let reordered = Arc::new(reordered);
        state.reconcile("project-a", &reordered);
        assert_eq!(state.package_selection, "vendor.first");
        let owner = state.package_selection.clone();
        assert!(state.open(&owner, &reordered));
        assert_eq!(view(&mut state, &reordered).plugin_id, "vendor.first");
        state.back();

        let empty = Arc::new(EditorPluginStatusReport::default());
        assert!(state
            .project("project-a", &empty, "en", |_| panic!("closed details"))
            .is_none());
        assert!(state.package_selection.is_empty() && state.owner.is_none());
        assert_eq!(state.focus, "package");
        assert!(state.cached.is_none() && state.intent.is_none());
        state.reconcile("project-a", &original);
        assert!(state.package_selection.is_empty());
        assert!(state.open(&original.plugins[0].plugin_id, &original));
        assert_eq!(view(&mut state, &original).plugin_id, "vendor.first");
    }

    #[test]
    fn closed_details_keep_the_existing_summary_generation_without_feature_pages() {
        let report = fixture_report(2_048);
        let mut summary = super::super::cache::ModulePluginPaneProjectionCache::default();
        let mut state = ModulePluginDetailsState::default();
        let first = summary.get_or_build(
            &report,
            super::super::pane_data::module_plugins_pane_from_status_report,
        );
        for _ in 0..100 {
            let pane =
                summary.get_or_build(&report, |_| panic!("summary must reuse its published Arc"));
            assert!(first.plugins.shares_values_with(&pane.plugins));
            assert!(state
                .project("project-a", &report, "en", |_| panic!(
                    "closed detail projection"
                ))
                .is_none());
            assert!(state.cached.is_none() && state.intent.is_none());
        }
    }
}
