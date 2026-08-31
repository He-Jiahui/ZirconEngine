use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_tab_like_workbench_button(
    node: &TemplatePaneNodeData,
) -> bool {
    let control_id = node.control_id.as_str();
    let action_id = node.action_id.as_str();
    control_id.starts_with("PageTab")
        || control_id.starts_with("DockTab")
        || is_workbench_module_tab_control(control_id)
        || is_workbench_module_tab_action(action_id)
        || is_asset_browser_tab_like_control(control_id)
        || is_asset_browser_tab_like_action(action_id)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_module_tab_button(
    node: &TemplatePaneNodeData,
) -> bool {
    let control_id = node.control_id.as_str();
    let action_id = node.action_id.as_str();
    is_workbench_module_tab_control(control_id) || is_workbench_module_tab_action(action_id)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_asset_browser_tab_like_button(
    node: &TemplatePaneNodeData,
) -> bool {
    let control_id = node.control_id.as_str();
    let action_id = node.action_id.as_str();
    is_asset_browser_tab_like_control(control_id) || is_asset_browser_tab_like_action(action_id)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_asset_browser_toolbar_chip_button(
    node: &TemplatePaneNodeData,
) -> bool {
    let control_id = node.control_id.as_str();
    let action_id = node.action_id.as_str();
    is_asset_browser_toolbar_chip_control(control_id)
        || is_asset_browser_toolbar_chip_action(action_id)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_asset_browser_utility_tab_button(
    node: &TemplatePaneNodeData,
) -> bool {
    let control_id = node.control_id.as_str();
    let action_id = node.action_id.as_str();
    is_asset_browser_utility_tab_control(control_id)
        || is_asset_browser_utility_tab_action(action_id)
}

fn is_workbench_module_tab_control(control_id: &str) -> bool {
    matches!(
        control_id,
        "WorkbenchModuleScene"
            | "WorkbenchModuleEffect"
            | "WorkbenchModuleAbility"
            | "WorkbenchModuleTags"
            | "WorkbenchModulePerception"
            | "WorkbenchModuleMaterial"
            | "WorkbenchModuleBehavior"
            | "WorkbenchModuleRender"
            | "WorkbenchModuleAssets"
            | "WorkbenchModuleVfx"
            | "WorkbenchModuleHud"
    )
}

fn is_workbench_module_tab_action(action_id: &str) -> bool {
    matches!(
        action_id,
        "workbench.module.scene"
            | "workbench.module.effect"
            | "workbench.module.ability"
            | "workbench.module.tags"
            | "workbench.module.perception"
            | "workbench.module.material"
            | "workbench.module.behavior"
            | "workbench.module.render"
            | "workbench.module.assets"
            | "workbench.module.vfx"
            | "workbench.module.hud"
            | "workbench.module.scene.select"
            | "workbench.module.effect.select"
            | "workbench.module.ability.select"
            | "workbench.module.tags.select"
            | "workbench.module.perception.select"
            | "workbench.module.material.select"
            | "workbench.module.behavior.select"
            | "workbench.module.render.select"
            | "workbench.module.assets.select"
            | "workbench.module.vfx.select"
            | "workbench.module.hud.select"
    )
}

fn is_asset_browser_tab_like_control(control_id: &str) -> bool {
    is_asset_browser_toolbar_chip_control(control_id)
        || (control_id.starts_with("AssetBrowser") && control_id.ends_with("TabButton"))
        || (control_id.starts_with("AssetsActivity") && control_id.ends_with("TabButton"))
        || is_asset_browser_utility_tab_control(control_id)
}

fn is_asset_browser_tab_like_action(action_id: &str) -> bool {
    is_asset_browser_toolbar_chip_action(action_id)
        || is_asset_browser_utility_tab_action(action_id)
}

fn is_asset_browser_toolbar_chip_control(control_id: &str) -> bool {
    ((control_id.starts_with("AssetBrowserKind") || control_id.starts_with("AssetsActivityKind"))
        && (control_id.ends_with("Chip") || control_id.ends_with("Button")))
        || control_id.starts_with("AssetBrowserViewMode")
        || control_id.starts_with("AssetsActivityViewMode")
}

fn is_asset_browser_toolbar_chip_action(action_id: &str) -> bool {
    matches!(
        action_id,
        "workbench.asset.kind_filter.set" | "workbench.asset.view_mode.set"
    )
}

fn is_asset_browser_utility_tab_control(control_id: &str) -> bool {
    matches!(
        control_id,
        "AssetBrowserPreviewTabButton"
            | "AssetBrowserReferencesTabButton"
            | "AssetBrowserMetadataTabButton"
            | "AssetBrowserPluginsTabButton"
            | "AssetsActivityPreviewButton"
            | "AssetsActivityReferencesTabButton"
            | "AssetsActivityMetadataTabButton"
            | "AssetsActivityPluginsTabButton"
    )
}

fn is_asset_browser_utility_tab_action(action_id: &str) -> bool {
    action_id == "workbench.asset.utility_tab.set"
}
