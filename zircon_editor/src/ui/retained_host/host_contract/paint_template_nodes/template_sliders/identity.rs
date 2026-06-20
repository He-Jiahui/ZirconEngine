use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};
use super::super::style_selector::{select_workbench_slider_style, WorkbenchSliderStyle};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_slider(
    node: &TemplatePaneNodeData,
) -> bool {
    uses_workbench_visual_language(node)
        && is_component_family(node, TemplateComponentFamily::Slider)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchSliderStyle {
    select_workbench_slider_style(node)
}
