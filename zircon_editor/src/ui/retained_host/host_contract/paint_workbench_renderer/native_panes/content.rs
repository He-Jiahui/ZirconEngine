use crate::ui::retained_host::hierarchy_pointer::current_hierarchy_row_metrics;

use super::super::super::data::{
    FrameRect, HostPaneInteractionStateData, HostTextInputFocusData, PaneData,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::intersect;
use super::super::welcome;
use super::{assets, hierarchy, scrollbar};

pub(in crate::ui::retained_host::host_contract) fn draw_native_pane_content(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> bool {
    let Some(effective_clip) = effective_native_clip(clip, frame.paint_clip()) else {
        return native_content_is_present(pane);
    };
    let clip = &effective_clip;
    match pane.kind.as_str() {
        "Welcome" => welcome::draw_welcome_native_content(frame, pane, body, clip),
        "Hierarchy" => {
            let content_present = native_content_is_present(pane);
            let viewport = hierarchy::hierarchy_viewport_frame(pane, body);
            let row_metrics = current_hierarchy_row_metrics();
            hierarchy::draw_hierarchy_rows(
                frame,
                pane,
                &viewport,
                clip,
                interaction,
                text_input_focus,
                row_metrics,
            );
            scrollbar::draw_hierarchy_scrollbar(
                frame,
                pane,
                &viewport,
                clip,
                interaction,
                row_metrics,
            );
            content_present
        }
        "Assets" => {
            let hover = assets::draw_activity_asset_tree_hover_overlay(
                frame,
                pane,
                body,
                clip,
                interaction,
            );
            let scrollbars =
                scrollbar::draw_activity_asset_scrollbars(frame, pane, body, clip, interaction);
            hover || scrollbars
        }
        "AssetBrowser" => {
            scrollbar::draw_browser_asset_scrollbars(frame, pane, body, clip, interaction)
        }
        _ => false,
    }
}

fn native_content_is_present(pane: &PaneData) -> bool {
    match pane.kind.as_str() {
        "Welcome" => pane.welcome.layout.has_nodes || !pane.welcome.title.is_empty(),
        "Hierarchy" => pane.hierarchy.hierarchy_nodes.row_count() > 0,
        _ => false,
    }
}

fn effective_native_clip(
    pane_clip: &FrameRect,
    paint_clip: Option<&FrameRect>,
) -> Option<FrameRect> {
    match paint_clip {
        Some(damage) => intersect(pane_clip, damage),
        None => Some(pane_clip.clone()),
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ui::retained_host::primitives::{ModelRc, VecModel};

    use super::super::super::super::data::{HierarchyPaneData, SceneNodeData, WelcomePaneData};
    use super::*;

    fn rect(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
        FrameRect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn native_content_clip_is_limited_to_frame_damage() {
        let pane = rect(10.0, 20.0, 100.0, 80.0);

        assert_eq!(
            effective_native_clip(&pane, Some(&rect(40.0, 30.0, 20.0, 25.0))),
            Some(rect(40.0, 30.0, 20.0, 25.0))
        );
        assert_eq!(effective_native_clip(&pane, None), Some(pane));
    }

    #[test]
    fn native_content_clip_rejects_disjoint_damage() {
        assert!(effective_native_clip(
            &rect(10.0, 20.0, 100.0, 80.0),
            Some(&rect(200.0, 200.0, 20.0, 20.0)),
        )
        .is_none());
    }

    #[test]
    fn disjoint_damage_reports_logical_native_content_without_hiding_empty_states() {
        let body = rect(10.0, 20.0, 100.0, 80.0);
        let interaction = HostPaneInteractionStateData::default();
        let mut frame = HostRgbaFrame::filled(320, 240, [0, 0, 0, 0]);
        frame.replace_paint_clip(Some(rect(200.0, 200.0, 20.0, 20.0)));

        let welcome = PaneData {
            kind: "Welcome".into(),
            welcome: WelcomePaneData {
                title: "Welcome".into(),
                ..WelcomePaneData::default()
            },
            ..PaneData::default()
        };
        assert!(draw_native_pane_content(
            &mut frame,
            &welcome,
            &body,
            &body,
            &interaction,
            None,
        ));

        let hierarchy = PaneData {
            kind: "Hierarchy".into(),
            hierarchy: HierarchyPaneData {
                hierarchy_nodes: ModelRc::from(Rc::new(VecModel::from(vec![
                    SceneNodeData::default(),
                ]))),
                ..HierarchyPaneData::default()
            },
            ..PaneData::default()
        };
        assert!(draw_native_pane_content(
            &mut frame,
            &hierarchy,
            &body,
            &body,
            &interaction,
            None,
        ));

        let mut full_frame = HostRgbaFrame::recording_only(320, 240);
        for pane_kind in [
            "Welcome",
            "Hierarchy",
            "Assets",
            "AssetBrowser",
            "Inspector",
        ] {
            let empty = PaneData {
                kind: pane_kind.into(),
                ..PaneData::default()
            };
            assert!(!draw_native_pane_content(
                &mut frame,
                &empty,
                &body,
                &body,
                &interaction,
                None,
            ));
            assert!(!draw_native_pane_content(
                &mut full_frame,
                &empty,
                &body,
                &body,
                &interaction,
                None,
            ));
        }
    }
}
