use super::super::surface_hit_test::TemplateNodePointerHit;
use super::path::push_path_segment;
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
        target_value: &str,
    ) -> SharedString {
        let (prefix, target) = match self {
            Self::SceneNode => ("workbench://scene/", target_value),
            Self::ModuleNode => ("workbench://module/", target_value),
            Self::GenericWorkbench => ("workbench://control/", hit.control_id.as_str()),
        };
        let mut target_path = String::with_capacity(prefix.len() + target.len());
        target_path.push_str(prefix);
        push_path_segment(&mut target_path, target);
        target_path.into()
    }

    pub(in crate::ui::retained_host::host_contract) fn menu_items(&self) -> Vec<SharedString> {
        match self {
            Self::SceneNode => vec![
                "Open|action=menu.item.open,icon=folder",
                "Rename|action=menu.item.rename,icon=edit",
                "Duplicate|action=menu.item.duplicate,icon=copy",
                "---",
                "Delete|action=menu.item.delete,danger,icon=trash",
            ],
            Self::ModuleNode => vec![
                "Open Module|action=menu.item.open_module,icon=folder",
                "Pin Module|action=menu.item.pin_module,icon=pin",
                "Reset Module|action=menu.item.reset_module,icon=rotate-ccw",
            ],
            Self::GenericWorkbench => vec![
                "Inspect|action=menu.item.inspect,icon=search",
                "Copy Id|action=menu.item.copy_id,icon=copy",
                "Reveal In Workbench|action=menu.item.reveal_in_workbench,icon=target",
            ],
        }
        .into_iter()
        .map(Into::into)
        .collect()
    }
}
