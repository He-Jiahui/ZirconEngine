use zircon_runtime_interface::ui::{component::UiValue, layout::UiSize};

use crate::ui::retained_host::host_contract::{current_host_metrics, menu_popup_text_width};
use crate::ui::retained_host::menu_popup_contract::{
    content_measured_structured_menu_popup_width, menu_popup_content_height,
};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;
use super::module_overflow_menu::{
    UNREAL_WINDOW_MODULE_MENU_CONTROL_ID, WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID,
    WORKBENCH_MODULE_OVERFLOW_TRIGGER_CONTROL_ID,
};
use crate::ui::binding::EditorUiBinding;
use crate::ui::retained_host::host_contract::componentized_workbench_regions::{
    module_tools_menu_admission_is_live, module_tools_menu_spec,
    module_tools_source_chain_is_available,
};
use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::workbench::reference::EditorWorkbenchTemplateSurfaceError;

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn initialize_workbench_window_menu_state(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.refresh_unreal_window_module_menu_items()?;
        // A closed menu must also release its arranged input region at first paint.
        for menu in WORKBENCH_WINDOW_MENUS {
            self.set_workbench_window_menu_open(menu, false)?;
        }
        Ok(())
    }

    pub(super) fn apply_workbench_window_menu_action(
        &mut self,
        source_control_id: &str,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        #[cfg(test)]
        eprintln!(
            "authored-window-menu-action: source={source_control_id} action={action_id} target={}",
            workbench_menu_for_action(source_control_id, action_id)
                .map(|menu| menu.menu_control_id)
                .unwrap_or("<none>"),
        );
        let Some(target) = workbench_menu_for_action(source_control_id, action_id) else {
            return Ok(false);
        };
        if module_tools_menu_spec(target.menu_control_id).is_some()
            && (target.trigger_control_id != source_control_id
                || !target.action_ids.contains(&action_id)
                || !self.module_tools_menu_source_is_live(target.menu_control_id))
        {
            return Err(EditorWorkbenchTemplateSurfaceError::MissingControl {
                control_id: target.menu_control_id,
            }
            .into());
        }
        let open = !self.control_bool(target.menu_control_id, "popup_open");

        for menu in WORKBENCH_WINDOW_MENUS {
            self.set_workbench_window_menu_open(menu, open && menu == target)?;
        }
        #[cfg(test)]
        eprintln!(
            "authored-window-menu-action: target={} requested_open={open} actual_open={} focused={}",
            target.menu_control_id,
            self.control_bool(target.menu_control_id, "popup_open"),
            self.control_bool(target.menu_control_id, "focused"),
        );
        Ok(true)
    }

    pub(super) fn close_workbench_window_menu_control(
        &mut self,
        menu_control_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let Some(menu) = WORKBENCH_WINDOW_MENUS
            .iter()
            .find(|menu| menu.menu_control_id == menu_control_id)
        else {
            return Ok(false);
        };
        self.set_workbench_window_menu_open(menu, false)?;
        Ok(true)
    }

    fn set_workbench_window_menu_open(
        &mut self,
        menu: &WorkbenchWindowMenu,
        open: bool,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if open && menu.menu_control_id == WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID {
            self.refresh_workbench_module_overflow_menu_items()?;
        }
        if open && menu.menu_control_id == UNREAL_WINDOW_MODULE_MENU_CONTROL_ID {
            self.refresh_unreal_window_module_menu_items()?;
        }
        if !self.has_control(menu.menu_control_id) {
            return Ok(());
        }
        if open {
            if module_tools_menu_spec(menu.menu_control_id).is_some() {
                let items = match menu.menu_control_id {
                    "WorkbenchAssetsWorldToolsMenu"
                    | "WorkbenchAssetsGameplayToolsMenu"
                    | "WorkbenchAssetsProductionToolsMenu" => {
                        self.workbench_asset_editor_menu_items(menu.menu_control_id)
                    }
                    "WorkbenchAbilityAnimationToolsMenu" => {
                        self.workbench_ability_editor_menu_items(menu.menu_control_id)
                    }
                    "WorkbenchRenderToolsMenu" => {
                        self.workbench_render_editor_menu_items(menu.menu_control_id)
                    }
                    "WorkbenchHudToolsMenu" => {
                        self.workbench_hud_editor_menu_items(menu.menu_control_id)
                    }
                    _ => unreachable!("known module Tools menu"),
                };
                self.mutate_control_property(
                    menu.menu_control_id,
                    "menu_items",
                    UiValue::Array(items.into_iter().map(UiValue::String).collect()),
                )?;
            }
            if let Some(node_id) = self.control_node_id(menu.menu_control_id) {
                self.template_surface
                    .surface
                    .set_popup_control_anchor(node_id, menu.trigger_control_id.to_string())?;
            }
            self.apply_workbench_window_menu_extent(menu.menu_control_id)?;
        }
        self.set_control_active(menu.trigger_control_id, open)?;
        self.set_visible(menu.menu_control_id, open)?;
        self.set_selected(menu.menu_control_id, open)?;
        self.mutate_control_property(menu.menu_control_id, "popup_open", UiValue::Bool(open))?;
        self.mutate_control_property(menu.menu_control_id, "focused", UiValue::Bool(open))?;
        Ok(())
    }

    pub(crate) fn owns_module_tools_menu(&self, control_id: &str) -> bool {
        module_tools_menu_spec(control_id).is_some()
    }

    pub(crate) fn module_tools_menu_is_open(&self, control_id: &str) -> bool {
        self.control_bool(control_id, "popup_open")
            && self.module_tools_menu_source_is_live(control_id)
    }

    fn module_tools_menu_source_is_live(&self, control_id: &str) -> bool {
        let Some(spec) = module_tools_menu_spec(control_id) else {
            return false;
        };
        let Some(metadata) = self
            .control_node_id(control_id)
            .and_then(|id| self.surface().tree.node(id))
            .and_then(|node| node.template_metadata.as_ref())
        else {
            return false;
        };
        if metadata.component != "PopupMenu"
            || metadata.widget.resolved_behavior(&metadata.component)
                != zircon_runtime_interface::ui::widget::UiWidgetBehavior::Popup
            || metadata.widget.popup_anchor.control_id() != Some(spec.trigger_control_id)
        {
            return false;
        }
        // Host paint models omit collapsed rows; validate the closed popup against
        // the actual compiled projection before using its visible trigger proof.
        let projection = self.host_projection();
        let mut popups = projection
            .nodes
            .iter()
            .filter(|node| node.control_id.as_deref() == Some(spec.menu_control_id));
        let Some(popup) = popups.next() else {
            return false;
        };
        let mut roots = projection
            .nodes
            .iter()
            .filter(|node| node.control_id.as_deref() == Some(spec.root_control_id));
        let Some(root) = roots.next() else {
            return false;
        };
        if popups.next().is_some()
            || roots.next().is_some()
            || popup.source_node_id.as_deref() != Some(spec.popup_source_node_id)
            || !popup
                .source_path
                .as_deref()
                .is_some_and(|path| path.replace('\\', "/").ends_with(spec.source_suffix))
            || popup.parent_id.as_deref() != Some(root.node_id.as_str())
            || popup
                .source_surface_frame
                .as_ref()
                .zip(root.source_surface_frame.as_ref())
                .is_none_or(|(popup, root)| !std::sync::Arc::ptr_eq(popup, root))
        {
            return false;
        }
        // Closed popup admission permits its collapsed node, but never a disabled owner.
        // An open row callback must also observe the real visible parent chain.
        let Some(source) = popup.source_surface_frame.as_ref() else {
            return false;
        };
        let Some(popup_id) = self.control_node_id(control_id) else {
            return false;
        };
        let Some(window_id) = self.control_node_id("WorkbenchWindowRoot") else {
            return false;
        };
        let Some(actual) = source.arranged_tree.get(popup_id) else {
            return false;
        };
        if actual.node_path.0 != popup.node_id.as_str()
            || actual.control_id.as_deref() != Some(control_id)
            || actual.parent != self.control_node_id(spec.root_control_id)
            || !module_tools_source_chain_is_available(
                source,
                popup_id,
                window_id,
                !self.control_bool(control_id, "popup_open"),
            )
        {
            return false;
        }
        let presentation = HostWindowPresentationData {
            workbench_window_nodes:
                crate::ui::retained_host::ui::to_host_contract_workbench_window_nodes(Some(
                    self.host_projection(),
                )),
            ..Default::default()
        };
        module_tools_menu_admission_is_live(&presentation, control_id)
    }

    /// Discover source owners structurally, even while a real owner is disabled or hidden.
    pub(super) fn refresh_mounted_module_source_publication(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let Some((root_id, root_control_id)) = self.mounted_module_root_for_source_publication()?
        else {
            return Ok(());
        };
        let missing =
            |control_id| EditorWorkbenchTemplateSurfaceError::MissingControl { control_id };
        let window_id = self
            .control_node_id("WorkbenchWindowRoot")
            .ok_or_else(|| missing("WorkbenchWindowRoot"))?;
        let mut source_row_ids = std::collections::BTreeSet::new();
        collect_source_publication_ancestors(
            self.surface(),
            root_id,
            window_id,
            root_id,
            &mut source_row_ids,
        )
        .ok_or_else(|| missing(root_control_id))?;
        for menu in WORKBENCH_WINDOW_MENUS {
            let Some(spec) = module_tools_menu_spec(menu.menu_control_id) else {
                continue;
            };
            if spec.root_control_id != root_control_id {
                continue;
            }
            let root = self
                .surface()
                .tree
                .node(root_id)
                .and_then(|node| node.template_metadata.as_ref())
                .ok_or_else(|| missing(root_control_id))?;
            if !root
                .source_path
                .as_deref()
                .is_some_and(|path| path.replace('\\', "/").ends_with(spec.source_suffix))
            {
                return Err(missing(root_control_id).into());
            }
            let trigger_id = self
                .control_node_id(spec.trigger_control_id)
                .ok_or_else(|| missing(spec.trigger_control_id))?;
            let popup_id = self
                .control_node_id(spec.menu_control_id)
                .ok_or_else(|| missing(spec.menu_control_id))?;
            let trigger = self
                .surface()
                .tree
                .node(trigger_id)
                .and_then(|node| node.template_metadata.as_ref())
                .ok_or_else(|| missing(spec.trigger_control_id))?;
            let popup = self
                .surface()
                .tree
                .node(popup_id)
                .ok_or_else(|| missing(spec.menu_control_id))?;
            let metadata = popup
                .template_metadata
                .as_ref()
                .ok_or_else(|| missing(spec.menu_control_id))?;
            if !spec.trigger_source_is_owned(
                root.source_path.as_deref().unwrap_or(""),
                root.instance_path.as_deref(),
                trigger.source_path.as_deref().unwrap_or(""),
                trigger.source_node_id.as_deref().unwrap_or(""),
                trigger.instance_path.as_deref(),
            ) || metadata.source_node_id.as_deref() != Some(spec.popup_source_node_id)
                || metadata.source_path != root.source_path
                || popup.parent != Some(root_id)
                || metadata.component != "PopupMenu"
                || metadata.widget.resolved_behavior(&metadata.component)
                    != zircon_runtime_interface::ui::widget::UiWidgetBehavior::Popup
                || metadata.widget.popup_anchor.control_id() != Some(spec.trigger_control_id)
            {
                #[cfg(test)]
                eprintln!(
                    "mounted-module-popup rejection: root={} root_id={:?} root_path={:?} root_instance={:?} menu={} trigger_id={:?} expected_trigger_source={} actual_trigger_source={:?} actual_trigger_path={:?} actual_trigger_instance={:?} popup_id={:?} expected_popup_source={} actual_popup_source={:?} actual_popup_path={:?} actual_parent={:?} component={} behavior={:?} expected_anchor={} actual_anchor={:?}",
                    root_control_id, root_id, root.source_path, root.instance_path,
                    spec.menu_control_id, trigger_id, spec.trigger_source_node_id,
                    trigger.source_node_id, trigger.source_path, trigger.instance_path,
                    popup_id, spec.popup_source_node_id, metadata.source_node_id,
                    metadata.source_path, popup.parent, metadata.component,
                    metadata.widget.resolved_behavior(&metadata.component),
                    spec.trigger_control_id, metadata.widget.popup_anchor.control_id(),
                );
                return Err(missing(spec.menu_control_id).into());
            }
            for id in [trigger_id, popup_id] {
                collect_source_publication_ancestors(
                    self.surface(),
                    id,
                    window_id,
                    root_id,
                    &mut source_row_ids,
                )
                .ok_or_else(|| missing(spec.menu_control_id))?;
            }
        }
        let mut node_ids = source_row_ids.clone();
        let mut pending = vec![root_id];
        let mut descendants = std::collections::BTreeSet::new();
        while let Some(id) = pending.pop() {
            if !descendants.insert(id) {
                return Err(missing(root_control_id).into());
            }
            let node = self
                .surface()
                .tree
                .node(id)
                .ok_or_else(|| missing(root_control_id))?;
            node_ids.insert(id);
            for child in &node.children {
                if self
                    .surface()
                    .tree
                    .node(*child)
                    .and_then(|node| node.parent)
                    != Some(id)
                {
                    return Err(missing(root_control_id).into());
                }
                pending.push(*child);
            }
        }
        self.template_surface
            .rebind_host_projection_sources(&node_ids, &source_row_ids)?;
        Ok(())
    }

    /// Admission precedes closing the selected popup. A failed child mount keeps
    /// the original published surface, source, focus and popup input state.
    pub(crate) fn dispatch_module_tools_menu_item_state(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) -> Result<Option<EditorUiBinding>, BuiltinHostWindowTemplateBridgeError> {
        if !self.module_tools_menu_is_open(control_id)
            || self.popup_menu_item_is_selectable(control_id, action_id) != Some(true)
        {
            return Ok(None);
        }
        let binding = match control_id {
            "WorkbenchAssetsWorldToolsMenu"
            | "WorkbenchAssetsGameplayToolsMenu"
            | "WorkbenchAssetsProductionToolsMenu" => {
                self.dispatch_workbench_asset_editor_menu_item_state(control_id, action_id)?
            }
            "WorkbenchAbilityAnimationToolsMenu" => {
                self.dispatch_workbench_ability_editor_menu_item_state(control_id, action_id)?
            }
            "WorkbenchRenderToolsMenu" => {
                self.dispatch_workbench_render_editor_menu_item_state(control_id, action_id)?
            }
            "WorkbenchHudToolsMenu" => {
                self.dispatch_workbench_hud_editor_menu_item_state(control_id, action_id)?
            }
            _ => None,
        };
        if binding.is_some() && self.has_control(control_id) {
            self.select_popup_menu_item(control_id, action_id)?;
        }
        Ok(binding)
    }

    fn apply_workbench_window_menu_extent(
        &mut self,
        menu_control_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let scale_factor = normalized_scale_factor(self.presentation_scale_factor);
        let logical_shell_width = (self.mount_frame.width / scale_factor).max(1.0);
        let logical_shell_height = (self.mount_frame.height / scale_factor).max(1.0);
        if self.apply_compiled_asset_creation_menu_extent(
            menu_control_id,
            UiSize::new(logical_shell_width, logical_shell_height),
        )? {
            return Ok(());
        }
        let menu_items = self.control_string_array(menu_control_id, "menu_items");
        let metrics = current_host_metrics();
        let trailing_adornment_reserve =
            (metrics.font_large + metrics.gap_m * 2.0 - metrics.input_pad[1]).max(0.0)
                / scale_factor;
        let fallback_width = self
            .control_float(menu_control_id, "layout_min_width")
            .or_else(|| {
                self.control_node_id(menu_control_id)
                    .and_then(|node_id| self.template_surface.surface.tree.node(node_id))
                    .map(|node| node.constraints.width.preferred)
            })
            .unwrap_or(1.0);
        let width = content_measured_structured_menu_popup_width(
            fallback_width,
            logical_shell_width,
            menu_items.iter().map(String::as_str),
            trailing_adornment_reserve,
            |text| menu_popup_text_width(text) / scale_factor,
        );
        let height = menu_popup_content_height(menu_items.len())
            .min(logical_shell_height)
            .max(1.0);
        self.set_fixed_control_extent(menu_control_id, UiSize::new(width, height))
    }
}

fn collect_source_publication_ancestors(
    surface: &zircon_runtime::ui::surface::UiSurface,
    node_id: zircon_runtime_interface::ui::event_ui::UiNodeId,
    window_id: zircon_runtime_interface::ui::event_ui::UiNodeId,
    mounted_root_id: zircon_runtime_interface::ui::event_ui::UiNodeId,
    node_ids: &mut std::collections::BTreeSet<zircon_runtime_interface::ui::event_ui::UiNodeId>,
) -> Option<()> {
    let mut current = node_id;
    let mut includes_root = false;
    let mut chain = std::collections::BTreeSet::new();
    for _ in 0..surface.tree.nodes.len() {
        if !chain.insert(current) {
            return None;
        }
        let node = surface.tree.node(current)?;
        includes_root |= current == mounted_root_id;
        if current == window_id {
            if !includes_root
                || node.parent.is_some()
                || node.template_metadata.as_ref()?.control_id.as_deref()
                    != Some("WorkbenchWindowRoot")
            {
                return None;
            }
            node_ids.extend(chain);
            return Some(());
        }
        let parent_id = node.parent?;
        if !surface.tree.node(parent_id)?.children.contains(&current) {
            return None;
        }
        current = parent_id;
    }
    None
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct WorkbenchWindowMenu {
    trigger_control_id: &'static str,
    menu_control_id: &'static str,
    action_ids: &'static [&'static str],
}

const WORKBENCH_WINDOW_MENUS: &[WorkbenchWindowMenu] = &[
    WorkbenchWindowMenu {
        trigger_control_id: "MenuSlot0",
        menu_control_id: "UnrealFileMenuPopup",
        action_ids: &["workbench.unreal.menu.file.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "MenuSlot1",
        menu_control_id: "UnrealEditMenuPopup",
        action_ids: &["workbench.unreal.menu.edit.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "MenuSlot2",
        menu_control_id: "UnrealWindowMenuPopup",
        action_ids: &["workbench.unreal.menu.window.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "MenuSlot3",
        menu_control_id: "UnrealToolsMenuPopup",
        action_ids: &["workbench.unreal.menu.tools.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "MenuSlot4",
        menu_control_id: "UnrealBuildMenuPopup",
        action_ids: &["workbench.unreal.menu.build.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "MenuSlot5",
        menu_control_id: "UnrealHelpMenuPopup",
        action_ids: &["workbench.unreal.menu.help.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchToolSelect",
        menu_control_id: "UnrealModesMenuPopup",
        action_ids: &["workbench.unreal.modes.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchToolbarAddActor",
        menu_control_id: "UnrealAddMenuPopup",
        action_ids: &["workbench.unreal.add.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "UnrealToolbarBlueprints",
        menu_control_id: "UnrealBlueprintsMenuPopup",
        action_ids: &["workbench.unreal.blueprints.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "UnrealToolbarCinematics",
        menu_control_id: "UnrealCinematicsMenuPopup",
        action_ids: &["workbench.unreal.cinematics.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "UnrealToolbarSettings",
        menu_control_id: "UnrealSettingsMenuPopup",
        action_ids: &["workbench.unreal.settings.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchRunMode",
        menu_control_id: "WorkbenchRunModeMenu",
        action_ids: &["workbench.run_mode.menu.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchAssetsWorldTools",
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        action_ids: &["workbench.module.assets.world_tools.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchAssetsGameplayTools",
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        action_ids: &["workbench.module.assets.gameplay_tools.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchAssetsProductionTools",
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        action_ids: &["workbench.module.assets.production_tools.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchAbilityAnimationTools",
        menu_control_id: "WorkbenchAbilityAnimationToolsMenu",
        action_ids: &["workbench.module.ability.animation_tools.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchRenderTools",
        menu_control_id: "WorkbenchRenderToolsMenu",
        action_ids: &["workbench.module.render.tools.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchHudTools",
        menu_control_id: "WorkbenchHudToolsMenu",
        action_ids: &["workbench.module.hud.tools.open"],
    },
];

fn workbench_menu_for_action(
    source_control_id: &str,
    action_id: &str,
) -> Option<&'static WorkbenchWindowMenu> {
    WORKBENCH_WINDOW_MENUS.iter().find(|menu| {
        menu.trigger_control_id == source_control_id
            || menu
                .action_ids
                .iter()
                .any(|candidate| *candidate == action_id)
    })
}

fn normalized_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > f32::EPSILON {
        scale_factor
    } else {
        1.0
    }
}
