use std::collections::BTreeMap;

use crate::ui::retained_host::HostShellContentScope;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::ViewContentKind;

const DIAGNOSTIC_CONTENT_KINDS: [ViewContentKind; 2] = [
    ViewContentKind::RuntimeDiagnostics,
    ViewContentKind::PerformanceTimeline,
];

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) enum RuntimeDiagnosticsRefreshTarget {
    #[default]
    None,
    ShellContent(HostShellContentScope),
    FullPresentation,
    Pending,
}

impl RuntimeDiagnosticsRefreshTarget {
    pub(super) fn should_collect_payload(&self) -> bool {
        !matches!(self, Self::None)
    }
}

pub(super) fn runtime_diagnostics_refresh_target(
    model: &WorkbenchViewModel,
) -> RuntimeDiagnosticsRefreshTarget {
    runtime_diagnostics_refresh_target_for_parts(
        &model.document_tabs,
        &model.floating_windows,
        &model.tool_windows,
    )
}

fn runtime_diagnostics_refresh_target_for_parts(
    document_tabs: &[crate::ui::workbench::model::DocumentTabModel],
    floating_windows: &[crate::ui::workbench::model::FloatingWindowModel],
    tool_windows: &BTreeMap<ActivityDrawerSlot, crate::ui::workbench::model::ToolWindowStackModel>,
) -> RuntimeDiagnosticsRefreshTarget {
    let has_non_drawer_target = document_tabs.iter().any(active_diagnostic_document_tab)
        || floating_windows
            .iter()
            .any(|window| window.tabs.iter().any(active_diagnostic_document_tab));
    runtime_diagnostics_refresh_target_for_drawers(has_non_drawer_target, tool_windows)
}

fn active_diagnostic_document_tab(tab: &crate::ui::workbench::model::DocumentTabModel) -> bool {
    tab.active && is_diagnostic_content_kind(tab.content_kind)
}

fn runtime_diagnostics_refresh_target_for_drawers(
    has_non_drawer_target: bool,
    tool_windows: &BTreeMap<ActivityDrawerSlot, crate::ui::workbench::model::ToolWindowStackModel>,
) -> RuntimeDiagnosticsRefreshTarget {
    if has_non_drawer_target {
        return RuntimeDiagnosticsRefreshTarget::FullPresentation;
    }

    let mut target = None;
    for (slot, stack) in tool_windows.iter().filter(|(_, stack)| stack.visible) {
        for tab in stack.tabs.iter().filter(|tab| {
            (tab.active || stack.active_tab.as_ref() == Some(&tab.instance_id))
                && is_diagnostic_content_kind(tab.content_kind)
        }) {
            if stack.active_tab.as_ref() != Some(&tab.instance_id) || target.is_some() {
                return RuntimeDiagnosticsRefreshTarget::FullPresentation;
            }
            target = Some(HostShellContentScope::new(*slot, tab.instance_id.clone()));
        }
    }

    target.map_or(
        RuntimeDiagnosticsRefreshTarget::None,
        RuntimeDiagnosticsRefreshTarget::ShellContent,
    )
}

fn is_diagnostic_content_kind(kind: ViewContentKind) -> bool {
    DIAGNOSTIC_CONTENT_KINDS.contains(&kind)
}

#[cfg(test)]
#[path = "tests/runtime_diagnostics_visibility.rs"]
mod tests;
