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
#[path = "tests/details.rs"]
mod tests;
