use super::family::TemplateComponentFamily;

pub(in crate::ui::retained_host::host_contract) fn workbench_control_family(
    control_id: &str,
) -> Option<TemplateComponentFamily> {
    if control_id.starts_with("WorkbenchMini")
        || control_id.starts_with("WorkbenchTool")
        || control_id.starts_with("WorkbenchToolbar")
        || control_id.starts_with("WorkbenchRail")
        || control_id.contains("IconButton")
    {
        Some(TemplateComponentFamily::IconButton)
    } else if control_id.starts_with("WorkbenchCheckbox") {
        Some(TemplateComponentFamily::Checkbox)
    } else if control_id.starts_with("WorkbenchRadio") {
        Some(TemplateComponentFamily::Radio)
    } else if control_id.starts_with("WorkbenchToggle") {
        Some(TemplateComponentFamily::Toggle)
    } else if control_id.starts_with("WorkbenchDrawerTab")
        || control_id.starts_with("WorkbenchLabsTab")
    {
        Some(TemplateComponentFamily::Tab)
    } else if control_id.contains("Segmented") {
        Some(TemplateComponentFamily::SegmentedControl)
    } else if control_id.starts_with("WorkbenchInputSlider")
        || control_id.starts_with("WorkbenchInputRangeSlider")
        || control_id.starts_with("WorkbenchInputStepsSlider")
        || control_id.starts_with("WorkbenchSlider")
    {
        Some(TemplateComponentFamily::Slider)
    } else if control_id == "WorkbenchInputDropdown" || control_id.starts_with("WorkbenchDropdown")
    {
        Some(TemplateComponentFamily::Dropdown)
    } else if control_id.starts_with("WorkbenchInput") || control_id.starts_with("WorkbenchField") {
        Some(TemplateComponentFamily::TextInput)
    } else if control_id.starts_with("WorkbenchList") {
        Some(TemplateComponentFamily::ListRow)
    } else if control_id.starts_with("WorkbenchSceneVirtualItem")
        || (control_id.starts_with("WorkbenchScene") && control_id.ends_with("Item"))
        || control_id.starts_with("WorkbenchEffectAsset")
        || control_id.starts_with("WorkbenchEffectHierarchy")
    {
        Some(TemplateComponentFamily::TreeRow)
    } else if control_id.starts_with("WorkbenchTable")
        || control_id.starts_with("WorkbenchEffectModifier")
    {
        Some(TemplateComponentFamily::TableRow)
    } else if control_id.ends_with("Button") || control_id.contains("Button") {
        Some(TemplateComponentFamily::Button)
    } else {
        None
    }
}
