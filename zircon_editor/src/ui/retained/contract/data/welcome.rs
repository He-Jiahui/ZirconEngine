use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::{FrameRect, TemplatePaneNodeData};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WelcomePaneLayoutData {
    pub has_nodes: bool,
    pub outer_panel: Option<FrameRect>,
    pub recent_panel: Option<FrameRect>,
    pub main_panel: Option<FrameRect>,
    pub hero_panel: Option<FrameRect>,
    pub status_panel: Option<FrameRect>,
    pub new_project_header_panel: Option<FrameRect>,
    pub project_name_field: Option<FrameRect>,
    pub location_field: Option<FrameRect>,
    pub preview_panel: Option<FrameRect>,
    pub validation_panel: Option<FrameRect>,
    pub recent_header_panel: Option<FrameRect>,
    pub recent_list_panel: Option<FrameRect>,
}

impl WelcomePaneLayoutData {
    pub(crate) fn capture(&mut self, node: &TemplatePaneNodeData) {
        self.has_nodes = true;
        let slot = match node.control_id.as_str() {
            "WelcomeOuterPanel" => &mut self.outer_panel,
            "WelcomeRecentPanel" => &mut self.recent_panel,
            "WelcomeMainPanel" => &mut self.main_panel,
            "WelcomeHeroPanel" => &mut self.hero_panel,
            "WelcomeStatusPanel" => &mut self.status_panel,
            "WelcomeNewProjectHeaderPanel" => &mut self.new_project_header_panel,
            "WelcomeProjectNameField" => &mut self.project_name_field,
            "WelcomeLocationField" => &mut self.location_field,
            "WelcomePreviewPanel" => &mut self.preview_panel,
            "WelcomeValidationPanel" => &mut self.validation_panel,
            "WelcomeRecentHeaderPanel" => &mut self.recent_header_panel,
            "WelcomeRecentListPanel" => &mut self.recent_list_panel,
            _ => return,
        };
        if slot.is_some() {
            return;
        }
        let frame = FrameRect {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        };
        if frame.x.is_finite()
            && frame.y.is_finite()
            && frame.width.is_finite()
            && frame.height.is_finite()
            && frame.width > 0.5
            && frame.height > 0.5
        {
            *slot = Some(frame);
        }
    }
}

#[cfg(test)]
pub(crate) fn compile_welcome_pane_layout(
    nodes: &ModelRc<TemplatePaneNodeData>,
) -> WelcomePaneLayoutData {
    let mut layout = WelcomePaneLayoutData::default();
    for node in nodes.iter() {
        layout.capture(node);
    }
    layout
}

#[derive(Clone, Default)]
pub(crate) struct RecentProjectData {
    pub display_name: SharedString,
    pub path: SharedString,
    pub last_opened_label: SharedString,
    pub status_label: SharedString,
    pub invalid: bool,
}

#[derive(Clone, Default)]
pub(crate) struct NewProjectFormData {
    pub project_name: SharedString,
    pub location: SharedString,
    pub project_path_preview: SharedString,
    pub template_label: SharedString,
    pub validation_message: SharedString,
    pub can_create: bool,
    pub can_open_existing: bool,
    pub browse_supported: bool,
}

#[derive(Clone, Default)]
pub(crate) struct WelcomePaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub layout: WelcomePaneLayoutData,
    pub title: SharedString,
    pub subtitle: SharedString,
    pub status_message: SharedString,
    pub form: NewProjectFormData,
    pub recent_projects: ModelRc<RecentProjectData>,
}
