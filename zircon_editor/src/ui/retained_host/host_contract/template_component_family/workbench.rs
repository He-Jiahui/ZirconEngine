use super::family::TemplateComponentFamily;

pub(in crate::ui::retained_host::host_contract) fn workbench_control_family(
    control_id: &str,
) -> Option<TemplateComponentFamily> {
    let workbench = control_id.strip_prefix("Workbench");
    if workbench.is_some_and(|id| {
        id.starts_with("Mini")
            || id.starts_with("Tool")
            || id.starts_with("Toolbar")
            || id.starts_with("Rail")
    }) || control_id.contains("IconButton")
    {
        Some(TemplateComponentFamily::IconButton)
    } else if workbench.is_some_and(|id| id.starts_with("Checkbox")) {
        Some(TemplateComponentFamily::Checkbox)
    } else if workbench.is_some_and(|id| id.starts_with("Radio")) {
        Some(TemplateComponentFamily::Radio)
    } else if workbench.is_some_and(|id| id.starts_with("Toggle")) {
        Some(TemplateComponentFamily::Toggle)
    } else if workbench.is_some_and(|id| id.starts_with("DrawerTab") || id.starts_with("LabsTab")) {
        Some(TemplateComponentFamily::Tab)
    } else if control_id.contains("Segmented") {
        Some(TemplateComponentFamily::SegmentedControl)
    } else if workbench.is_some_and(|id| {
        id.starts_with("InputSlider")
            || id.starts_with("InputRangeSlider")
            || id.starts_with("InputStepsSlider")
            || id.starts_with("Slider")
    }) {
        Some(TemplateComponentFamily::Slider)
    } else if control_id == "WorkbenchInputDropdown"
        || workbench.is_some_and(|id| id.starts_with("Dropdown"))
    {
        Some(TemplateComponentFamily::Dropdown)
    } else if workbench.is_some_and(|id| id.starts_with("Input") || id.starts_with("Field")) {
        Some(TemplateComponentFamily::TextInput)
    } else if workbench.is_some_and(|id| id.starts_with("List")) {
        Some(TemplateComponentFamily::ListRow)
    } else if workbench.is_some_and(|id| {
        id.starts_with("SceneVirtualItem")
            || (id.starts_with("Scene") && id.ends_with("Item"))
            || id.starts_with("EffectAsset")
            || id.starts_with("EffectHierarchy")
    }) {
        Some(TemplateComponentFamily::TreeRow)
    } else if workbench
        .is_some_and(|id| id.starts_with("Table") || id.starts_with("EffectModifier"))
    {
        Some(TemplateComponentFamily::TableRow)
    } else if control_id.ends_with("Button") || control_id.contains("Button") {
        Some(TemplateComponentFamily::Button)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "tests/workbench_optimization_tests.rs"]
mod optimization_tests;
