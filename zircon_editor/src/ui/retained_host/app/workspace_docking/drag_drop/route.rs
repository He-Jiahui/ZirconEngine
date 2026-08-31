use super::super::super::*;
use crate::ui::retained_host::tab_drag::{
    resolve_host_tab_drop_route_with_workbench_layout_frames, HostDragTargetGroup,
    ResolvedHostTabDropRoute, ResolvedHostTabDropTarget,
};

impl RetainedEditorHost {
    pub(super) fn resolve_drag_drop_route_from_pointer(
        &self,
        tab_id: &str,
        source_group: &str,
        target_group: &str,
        pointer_route: Option<HostShellPointerRoute>,
        x: f32,
        y: f32,
    ) -> Option<ResolvedHostTabDropRoute> {
        if target_group.is_empty() && pointer_route.is_none() {
            return Some(detached_window_drop_route(tab_id, source_group));
        }

        let committed = self.committed_shell_state.as_ref()?;
        resolve_host_tab_drop_route_with_workbench_layout_frames(
            &committed.layout,
            &committed.model,
            &self.chrome_metrics,
            tab_id,
            pointer_route,
            target_group,
            x,
            y,
            committed.layout_frames,
        )
    }
}

fn detached_window_drop_route(instance_id: &str, source_group: &str) -> ResolvedHostTabDropRoute {
    let drawer_source = matches!(source_group, "left" | "right" | "bottom");
    ResolvedHostTabDropRoute {
        target_group: HostDragTargetGroup::Document,
        target_label: if drawer_source {
            "detached drawer window"
        } else {
            "detached window"
        },
        target: ResolvedHostTabDropTarget::DetachToWindow {
            new_window: detached_window_id(instance_id, drawer_source),
        },
    }
}

fn detached_window_id(instance_id: &str, drawer_source: bool) -> MainPageId {
    let prefix = if drawer_source {
        "drawer-window"
    } else {
        "window"
    };
    let suffix = instance_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_' {
                ch
            } else {
                ':'
            }
        })
        .collect::<String>();
    MainPageId::new(format!("{prefix}:{suffix}"))
}
