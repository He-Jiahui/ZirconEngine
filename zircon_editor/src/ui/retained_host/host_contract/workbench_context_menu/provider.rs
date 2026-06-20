use super::super::surface_hit_test::TemplateNodePointerHit;
use super::path::{path_segment, target_value_text};
use crate::ui::retained_host::primitives::SharedString;

pub(in crate::ui::retained_host::host_contract) enum WorkbenchContextMenuProvider {
    SceneNode,
    ModuleNode,
    GenericWorkbench,
}

impl WorkbenchContextMenuProvider {
    pub(in crate::ui::retained_host::host_contract) fn target_path(
        &self,
        hit: &TemplateNodePointerHit,
    ) -> SharedString {
        let target = target_value_text(hit);
        match self {
            Self::SceneNode => format!("workbench://scene/{}", path_segment(target.as_str())),
            Self::ModuleNode => format!("workbench://module/{}", path_segment(target.as_str())),
            Self::GenericWorkbench => {
                format!(
                    "workbench://control/{}",
                    path_segment(hit.control_id.as_str())
                )
            }
        }
        .into()
    }

    pub(in crate::ui::retained_host::host_contract) fn menu_items(&self) -> Vec<SharedString> {
        match self {
            Self::SceneNode => vec![
                "Open|icon=folder",
                "Rename|icon=edit",
                "Duplicate|icon=copy",
                "---",
                "Delete|danger,icon=trash",
            ],
            Self::ModuleNode => vec![
                "Open Module|icon=folder",
                "Pin Module|icon=pin",
                "Reset Module|icon=rotate-ccw",
            ],
            Self::GenericWorkbench => vec![
                "Inspect|icon=search",
                "Copy Id|icon=copy",
                "Reveal In Workbench|icon=target",
            ],
        }
        .into_iter()
        .map(Into::into)
        .collect()
    }
}
