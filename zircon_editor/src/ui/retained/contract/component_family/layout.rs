use super::family::TemplateComponentFamily;

pub(in crate::ui::retained_host::host_contract) fn category_layout_family(
    category: &str,
    layout_role: &str,
) -> Option<TemplateComponentFamily> {
    match (category, layout_role) {
        ("collection", "grid") => Some(TemplateComponentFamily::Table),
        ("collection", "virtual-list") => Some(TemplateComponentFamily::List),
        ("container", "editor-dock") => Some(TemplateComponentFamily::Window),
        ("container", "flex" | "grid") => Some(TemplateComponentFamily::Container),
        ("selection", "popup") => Some(TemplateComponentFamily::Popup),
        ("feedback", "popup") => Some(TemplateComponentFamily::Tooltip),
        _ => None,
    }
}
