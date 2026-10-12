use crate::core::commands::MenuBarModel;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{
    view_template_resource_generation, ViewTemplateResourceGeneration,
};
use crate::ui::retained_host::menu_pointer::{
    current_menu_label_slot_metrics, MenuLabelSlotMetrics,
};
use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use crate::ui::retained_host::runtime_text_metrics_generation;
use crate::ui::widgets::common::{collect_tabs, document_tab_data, host_tab_data};
use crate::ui::workbench::layout::{ActivityDrawerSlot, MainPageId};
use crate::ui::workbench::model::{
    DocumentTabModel, HostPageTabModel, ToolWindowStackModel, WorkbenchViewModel,
};

use super::chrome_template_projection::MENU_CHROME_ASSET;
use super::{HostMenuChromeData, HostWindowShellData, HostWindowSurfaceMetricsData, TabData};

#[derive(Default)]
pub(crate) struct HostChromeProjectionCache {
    host_tabs: Option<HostTabsProjection>,
    left_tabs: Option<SideTabsProjection>,
    right_tabs: Option<SideTabsProjection>,
    bottom_tabs: Option<SideTabsProjection>,
    document_tabs: Option<DocumentTabsProjection>,
    preset_names: Option<PresetNamesProjection>,
    menu_chrome: Option<MenuChromeProjection>,
}

struct HostTabsProjection {
    pages: Vec<HostPageTabModel>,
    active_page: MainPageId,
    tabs: ModelRc<TabData>,
}

struct SideTabsProjection {
    stacks: Vec<(ActivityDrawerSlot, ToolWindowStackModel)>,
    tabs: ModelRc<TabData>,
}

struct DocumentTabsProjection {
    source: Vec<DocumentTabModel>,
    tabs: ModelRc<TabData>,
}

struct PresetNamesProjection {
    source: Vec<String>,
    names: ModelRc<SharedString>,
}

struct MenuChromeProjection {
    menu_bar: MenuBarModel,
    input: MenuChromeProjectionInput,
    resource_generation: ViewTemplateResourceGeneration,
    text_metrics_generation: [u64; 3],
    data: HostMenuChromeData,
}

struct MenuChromeProjectionInput {
    save_project_enabled: bool,
    undo_enabled: bool,
    redo_enabled: bool,
    delete_enabled: bool,
    preset_names: ModelRc<SharedString>,
    active_preset_name: SharedString,
    resolved_preset_name: SharedString,
    outer_margin_bits: u32,
    top_bar_height_bits: u32,
    shell_width_bits: u32,
    // Host typography and spacing may change without a font-resource generation change.
    menu_slot_metrics: MenuLabelSlotMetrics,
}

impl HostChromeProjectionCache {
    pub(crate) fn host_tabs(&mut self, model: &WorkbenchViewModel) -> ModelRc<TabData> {
        let source = &model.host_strip;
        if self.host_tabs.as_ref().is_some_and(|cached| {
            cached.pages == source.pages && cached.active_page == source.active_page
        }) {
            return self.host_tabs.as_ref().unwrap().tabs.clone();
        }

        let tabs = model_rc(
            source
                .pages
                .iter()
                .map(|page| host_tab_data(page, &source.active_page))
                .collect(),
        );
        self.host_tabs = Some(HostTabsProjection {
            pages: source.pages.clone(),
            active_page: source.active_page.clone(),
            tabs: tabs.clone(),
        });
        tabs
    }

    pub(crate) fn left_tabs(&mut self, model: &WorkbenchViewModel) -> ModelRc<TabData> {
        retain_side_tabs(
            &mut self.left_tabs,
            model,
            &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
        )
    }

    pub(crate) fn right_tabs(&mut self, model: &WorkbenchViewModel) -> ModelRc<TabData> {
        retain_side_tabs(
            &mut self.right_tabs,
            model,
            &[
                ActivityDrawerSlot::RightTop,
                ActivityDrawerSlot::RightBottom,
            ],
        )
    }

    pub(crate) fn bottom_tabs(&mut self, model: &WorkbenchViewModel) -> ModelRc<TabData> {
        retain_side_tabs(&mut self.bottom_tabs, model, &[ActivityDrawerSlot::Bottom])
    }

    pub(crate) fn document_tabs(&mut self, model: &WorkbenchViewModel) -> ModelRc<TabData> {
        if self
            .document_tabs
            .as_ref()
            .is_some_and(|cached| cached.source == model.document_tabs)
        {
            return self.document_tabs.as_ref().unwrap().tabs.clone();
        }

        let tabs = model_rc(model.document_tabs.iter().map(document_tab_data).collect());
        self.document_tabs = Some(DocumentTabsProjection {
            source: model.document_tabs.clone(),
            tabs: tabs.clone(),
        });
        tabs
    }

    pub(crate) fn preset_names(&mut self, names: &[String]) -> ModelRc<SharedString> {
        if self
            .preset_names
            .as_ref()
            .is_some_and(|cached| cached.source == names)
        {
            return self.preset_names.as_ref().unwrap().names.clone();
        }

        let model = model_rc(names.iter().cloned().map(SharedString::from).collect());
        self.preset_names = Some(PresetNamesProjection {
            source: names.to_vec(),
            names: model.clone(),
        });
        model
    }

    pub(crate) fn menu_chrome<F>(
        &mut self,
        menu_bar: &MenuBarModel,
        host_shell: &HostWindowShellData,
        delete_enabled: bool,
        metrics: &HostWindowSurfaceMetricsData,
        resolved_preset_name: &SharedString,
        shell_width: f32,
        build: F,
    ) -> HostMenuChromeData
    where
        F: FnOnce() -> HostMenuChromeData,
    {
        let resource_generation = view_template_resource_generation(MENU_CHROME_ASSET, &[]);
        let text_metrics_generation = runtime_text_metrics_generation();
        let menu_slot_metrics = current_menu_label_slot_metrics();
        if self.menu_chrome.as_ref().is_some_and(|cached| {
            resource_generation
                .as_ref()
                .is_some_and(|current| current == &cached.resource_generation)
                && cached.text_metrics_generation == text_metrics_generation
                && cached.menu_bar == *menu_bar
                && cached.input.matches(
                    host_shell,
                    delete_enabled,
                    metrics,
                    resolved_preset_name,
                    shell_width,
                    menu_slot_metrics,
                )
        }) {
            return self.menu_chrome.as_ref().unwrap().data.clone();
        }

        let data = build();
        if let Some(resource_generation) = resource_generation {
            self.menu_chrome = Some(MenuChromeProjection {
                menu_bar: menu_bar.clone(),
                input: MenuChromeProjectionInput::new(
                    host_shell,
                    delete_enabled,
                    metrics,
                    resolved_preset_name,
                    shell_width,
                    menu_slot_metrics,
                ),
                resource_generation,
                text_metrics_generation,
                data: data.clone(),
            });
        } else {
            self.menu_chrome = None;
        }
        data
    }
}

impl MenuChromeProjectionInput {
    fn new(
        host_shell: &HostWindowShellData,
        delete_enabled: bool,
        metrics: &HostWindowSurfaceMetricsData,
        resolved_preset_name: &SharedString,
        shell_width: f32,
        menu_slot_metrics: MenuLabelSlotMetrics,
    ) -> Self {
        Self {
            save_project_enabled: host_shell.save_project_enabled,
            undo_enabled: host_shell.undo_enabled,
            redo_enabled: host_shell.redo_enabled,
            delete_enabled,
            preset_names: host_shell.preset_names.clone(),
            active_preset_name: host_shell.active_preset_name.clone(),
            resolved_preset_name: resolved_preset_name.clone(),
            outer_margin_bits: metrics.outer_margin_px.to_bits(),
            top_bar_height_bits: metrics.top_bar_height_px.to_bits(),
            shell_width_bits: shell_width.to_bits(),
            menu_slot_metrics,
        }
    }

    fn matches(
        &self,
        host_shell: &HostWindowShellData,
        delete_enabled: bool,
        metrics: &HostWindowSurfaceMetricsData,
        resolved_preset_name: &SharedString,
        shell_width: f32,
        menu_slot_metrics: MenuLabelSlotMetrics,
    ) -> bool {
        self.save_project_enabled == host_shell.save_project_enabled
            && self.undo_enabled == host_shell.undo_enabled
            && self.redo_enabled == host_shell.redo_enabled
            && self.delete_enabled == delete_enabled
            && self
                .preset_names
                .shares_values_with(&host_shell.preset_names)
            && self.active_preset_name == host_shell.active_preset_name
            && self.resolved_preset_name == *resolved_preset_name
            && self.outer_margin_bits == metrics.outer_margin_px.to_bits()
            && self.top_bar_height_bits == metrics.top_bar_height_px.to_bits()
            && self.shell_width_bits == shell_width.to_bits()
            && self.menu_slot_metrics == menu_slot_metrics
    }
}

fn retain_side_tabs(
    cache: &mut Option<SideTabsProjection>,
    model: &WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> ModelRc<TabData> {
    if cache
        .as_ref()
        .is_some_and(|cached| side_tab_source_matches(&cached.stacks, model, slots))
    {
        return cache.as_ref().unwrap().tabs.clone();
    }

    let tabs = model_rc(collect_tabs(model, slots));
    let stacks = slots
        .iter()
        .filter_map(|slot| {
            model
                .tool_windows
                .get(slot)
                .map(|stack| (*slot, stack.clone()))
        })
        .collect();
    *cache = Some(SideTabsProjection {
        stacks,
        tabs: tabs.clone(),
    });
    tabs
}

fn side_tab_source_matches(
    cached: &[(ActivityDrawerSlot, ToolWindowStackModel)],
    model: &WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> bool {
    let mut current = slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot).map(|stack| (*slot, stack)));
    cached.iter().all(|(cached_slot, cached_stack)| {
        current
            .next()
            .is_some_and(|(slot, stack)| *cached_slot == slot && cached_stack == stack)
    }) && current.next().is_none()
}

#[cfg(test)]
#[path = "tests/projection_cache.rs"]
mod tests;
